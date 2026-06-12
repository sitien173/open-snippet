use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use git2::{
    build::RepoBuilder, BranchType, FetchOptions, IndexAddOption, PushOptions, Repository,
    Signature, StatusOptions,
};

use super::{
    conflicts::write_conflicts, git_remote_callbacks, AuthMode, CredentialStore, NoopNotifySink,
    NotifySink, SyncBackend, SyncError, SyncResult, SyncStatus, TickReport, WindowsCredentialStore,
};

#[derive(Default)]
struct GitState {
    local_dir: Option<PathBuf>,
    remote: Option<String>,
    auth: Option<AuthMode>,
    status: SyncStatus,
}

pub struct GitBackend {
    state: Mutex<GitState>,
    credentials: Arc<dyn CredentialStore>,
    notify: Arc<dyn NotifySink>,
}

impl GitBackend {
    pub fn new(credentials: Arc<dyn CredentialStore>, notify: Arc<dyn NotifySink>) -> Self {
        Self {
            state: Mutex::new(GitState::default()),
            credentials,
            notify,
        }
    }

    pub fn for_tests() -> Self {
        Self::new(Arc::new(WindowsCredentialStore), Arc::new(NoopNotifySink))
    }

    pub fn conflict_dir_for(&self, unix_ts: u64) -> PathBuf {
        let state = self.state.lock().unwrap();
        state
            .local_dir
            .clone()
            .unwrap_or_default()
            .join(".conflicts")
            .join(unix_ts.to_string())
    }

    fn local_dir(&self) -> SyncResult<PathBuf> {
        self.state
            .lock()
            .unwrap()
            .local_dir
            .clone()
            .ok_or_else(|| SyncError::State("sync backend not initialized".to_string()))
    }

    fn open_repo(&self) -> SyncResult<Repository> {
        Repository::open(self.local_dir()?).map_err(SyncError::from)
    }

    fn auth(&self) -> SyncResult<AuthMode> {
        self.state
            .lock()
            .unwrap()
            .auth
            .clone()
            .ok_or_else(|| SyncError::State("missing sync auth mode".to_string()))
    }

    fn commit_all(repo: &Repository, message: &str) -> SyncResult<bool> {
        let mut options = StatusOptions::new();
        options.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(&mut options))?;
        if statuses.is_empty() {
            tracing::debug!("sync commit skipped; no local changes");
            return Ok(false);
        }

        let mut index = repo.index()?;
        index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("openmacro", "openmacro@example.com")?;
        let parents = match repo.head() {
            Ok(head) => match head.target() {
                Some(oid) => vec![repo.find_commit(oid)?],
                None => Vec::new(),
            },
            Err(_) => Vec::new(),
        };
        let parent_refs = parents.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;
        tracing::info!(
            changed_paths = statuses.len(),
            "committed local sync changes"
        );
        Ok(true)
    }

    fn current_branch(repo: &Repository) -> SyncResult<String> {
        repo.head()?
            .shorthand()
            .map(str::to_string)
            .ok_or_else(|| SyncError::State("detached head".to_string()))
    }

    fn fetch(&self, repo: &Repository, branch: &str) -> SyncResult<()> {
        let auth = self.auth()?;
        let mut remote = repo.find_remote("origin")?;
        let mut options = FetchOptions::new();
        options.remote_callbacks(git_remote_callbacks(&auth, self.credentials.as_ref()));
        tracing::debug!(branch = %branch, "fetching sync remote");
        remote.fetch(&[branch], Some(&mut options), None)?;
        Ok(())
    }

    fn rebase(&self, repo: &Repository, branch: &str) -> SyncResult<Option<PathBuf>> {
        tracing::debug!(branch = %branch, "rebasing sync branch");
        let head_ref = repo.head()?;
        let head_annotated = repo.reference_to_annotated_commit(&head_ref)?;
        let upstream_ref = repo.find_reference(&format!("refs/remotes/origin/{branch}"))?;
        let upstream_annotated = repo.reference_to_annotated_commit(&upstream_ref)?;
        let mut rebase =
            repo.rebase(Some(&head_annotated), Some(&upstream_annotated), None, None)?;
        let signature = Signature::now("openmacro", "openmacro@example.com")?;

        loop {
            match rebase.next() {
                Some(Ok(_)) => {
                    if repo.index()?.has_conflicts() {
                        let ts = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let dir = self.conflict_dir_for(ts);
                        write_conflicts(repo, &dir)?;
                        rebase.abort()?;
                        self.notify.sync_conflict(&dir);
                        tracing::warn!(dir = %dir.display(), "sync conflict detected");
                        return Ok(Some(dir));
                    }
                    rebase.commit(None, &signature, None)?;
                }
                Some(Err(error)) => {
                    rebase.abort()?;
                    return Err(SyncError::from(error));
                }
                None => break,
            }
        }

        rebase.finish(Some(&signature))?;
        Ok(None)
    }

    fn push(&self, repo: &Repository, branch: &str) -> SyncResult<()> {
        let auth = self.auth()?;
        let mut remote = repo.find_remote("origin")?;
        let mut options = PushOptions::new();
        options.remote_callbacks(git_remote_callbacks(&auth, self.credentials.as_ref()));
        tracing::debug!(branch = %branch, "pushing sync branch");
        remote.push(
            &[&format!("refs/heads/{branch}:refs/heads/{branch}")],
            Some(&mut options),
        )?;
        Ok(())
    }

    fn update_status(&self, repo: &Repository) -> SyncResult<SyncStatus> {
        let branch = Self::current_branch(repo)?;
        let local_oid = repo
            .head()?
            .target()
            .ok_or_else(|| SyncError::State("missing head target".to_string()))?;
        let upstream = repo
            .find_branch(&branch, BranchType::Local)?
            .upstream()
            .ok()
            .and_then(|branch| branch.get().target());
        let (ahead, behind) = match upstream {
            Some(upstream_oid) => repo.graph_ahead_behind(local_oid, upstream_oid)?,
            None => (0, 0),
        };
        let status = SyncStatus {
            branch: Some(branch),
            ahead,
            behind,
            last_tick_unix: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        };
        tracing::debug!(
            branch = status.branch.as_deref().unwrap_or_default(),
            ahead = status.ahead,
            behind = status.behind,
            "updated sync status"
        );
        self.state.lock().unwrap().status = status.clone();
        Ok(status)
    }
}

impl Default for GitBackend {
    fn default() -> Self {
        Self::for_tests()
    }
}

impl SyncBackend for GitBackend {
    #[tracing::instrument(skip(self, auth), fields(remote = %remote, local_dir = %local_dir.display()))]
    fn init(&self, remote: &str, auth: AuthMode, local_dir: &Path) -> SyncResult<()> {
        tracing::info!(
            remote = %crate::log_init::redact::redact_str(
                remote,
                crate::log_init::redact::FieldKind::Path
            ),
            local_dir = %local_dir.display(),
            "initializing git sync backend"
        );
        let repo = if local_dir.join(".git").exists() {
            Repository::open(local_dir)?
        } else {
            let mut builder = RepoBuilder::new();
            let mut options = FetchOptions::new();
            options.remote_callbacks(git_remote_callbacks(&auth, self.credentials.as_ref()));
            builder.fetch_options(options);
            builder.clone(remote, local_dir)?
        };

        let mut state = self.state.lock().unwrap();
        state.local_dir = Some(local_dir.to_path_buf());
        state.remote = Some(remote.to_string());
        state.auth = Some(auth);
        drop(state);

        let _ = self.update_status(&repo);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn tick(&self) -> SyncResult<TickReport> {
        tracing::debug!("running sync tick");
        let repo = self.open_repo()?;
        let branch = Self::current_branch(&repo)?;
        self.fetch(&repo, &branch)?;
        let committed = Self::commit_all(&repo, "openmacro: local")?;
        if let Some(dir) = self.rebase(&repo, &branch)? {
            let _ = self.update_status(&repo);
            return Ok(TickReport::Conflict { dir });
        }
        self.push(&repo, &branch)?;
        let status = self.update_status(&repo)?;
        if committed || status.ahead > 0 || status.behind > 0 {
            tracing::info!(
                committed,
                ahead = status.ahead,
                behind = status.behind,
                "sync tick synced"
            );
            Ok(TickReport::Synced { committed })
        } else {
            tracing::debug!("sync tick found no changes");
            Ok(TickReport::NoChanges)
        }
    }

    fn status(&self) -> SyncStatus {
        self.state.lock().unwrap().status.clone()
    }
}
