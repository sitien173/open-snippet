use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};

mod conflicts;
mod creds;
mod git;

pub use creds::{
    credential_key, git_remote_callbacks, validate_credential_callback_url,
    validate_https_pat_remote, CredentialStore, Secret, SyncCredential, WindowsCredentialStore,
};
pub use git::GitBackend;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMode {
    HttpsPat { host: String, username: String },
    Ssh,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SyncStatus {
    pub branch: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub last_tick_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TickReport {
    NoChanges,
    Synced { committed: bool },
    Conflict { dir: PathBuf },
}

#[derive(Debug)]
pub enum SyncError {
    Git(git2::Error),
    State(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git(error) => write!(f, "{error}"),
            Self::State(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<git2::Error> for SyncError {
    fn from(value: git2::Error) -> Self {
        Self::Git(value)
    }
}

pub type SyncResult<T> = Result<T, SyncError>;

pub trait SyncBackend: Send + Sync {
    fn init(&self, remote: &str, auth: AuthMode, local_dir: &Path) -> SyncResult<()>;
    fn tick(&self) -> SyncResult<TickReport>;
    fn status(&self) -> SyncStatus;
}

pub trait NotifySink: Send + Sync {
    fn sync_conflict(&self, path: &Path);
}

#[derive(Default)]
pub struct NoopNotifySink;

impl NotifySink for NoopNotifySink {
    fn sync_conflict(&self, _path: &Path) {}
}

pub struct DriverHandle {
    trigger_tx: mpsc::Sender<()>,
}

impl DriverHandle {
    pub async fn trigger(&self) {
        let _ = self.trigger_tx.send(()).await;
    }
}

pub fn spawn_driver(
    backend: Arc<dyn SyncBackend>,
    mut watcher: watch::Receiver<Arc<crate::store::watcher::SnapshotInner>>,
) -> DriverHandle {
    let (trigger_tx, mut trigger_rx) = mpsc::channel::<()>(8);
    tracing::info!("starting sync driver");
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    tracing::debug!("sync driver interval tick");
                    let _ = backend.tick();
                }
                changed = watcher.changed() => {
                    if changed.is_err() {
                        tracing::debug!("sync driver watcher closed");
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    tracing::debug!("sync driver watcher tick");
                    let _ = backend.tick();
                }
                maybe_trigger = trigger_rx.recv() => {
                    if maybe_trigger.is_none() {
                        tracing::debug!("sync driver trigger channel closed");
                        break;
                    }
                    tracing::debug!("sync driver manual tick");
                    let _ = backend.tick();
                }
            }
        }
    });
    DriverHandle { trigger_tx }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use git2::{IndexAddOption, Repository, Signature};
    use tempfile::TempDir;

    use super::{AuthMode, GitBackend, SyncBackend, TickReport};

    fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    fn commit_all(repo: &Repository, message: &str) {
        let mut index = repo.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = Signature::now("openmacro", "openmacro@example.com").unwrap();
        let parents = repo
            .head()
            .ok()
            .and_then(|head| head.target())
            .map(|oid| repo.find_commit(oid).unwrap())
            .into_iter()
            .collect::<Vec<_>>();
        let parent_refs = parents.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )
        .unwrap();
    }

    fn seed_bare_remote() -> (TempDir, String) {
        let root = TempDir::new().unwrap();
        let bare_dir = root.path().join("remote.git");
        Repository::init_bare(&bare_dir).unwrap();

        let seed_dir = root.path().join("seed");
        let repo = Repository::init(&seed_dir).unwrap();
        write_file(&seed_dir, "README.md", "seed\n");
        commit_all(&repo, "seed");
        let mut remote = repo.remote("origin", bare_dir.to_str().unwrap()).unwrap();
        remote
            .push(&["refs/heads/master:refs/heads/master"], None)
            .unwrap();

        (root, bare_dir.to_string_lossy().into_owned())
    }

    #[test]
    fn init_and_tick_report_clean_branch_status() {
        let (_root, remote) = seed_bare_remote();
        let local = TempDir::new().unwrap();
        let backend = GitBackend::for_tests();

        backend.init(&remote, AuthMode::Ssh, local.path()).unwrap();

        let report = backend.tick().unwrap();
        let status = backend.status();

        assert!(matches!(
            report,
            TickReport::Synced { .. } | TickReport::NoChanges
        ));
        assert_eq!(status.branch.as_deref(), Some("master"));
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
    }

    #[test]
    fn conflict_dir_segment_is_unix_timestamp_sortable() {
        let (_root, remote) = seed_bare_remote();
        let local = TempDir::new().unwrap();
        let backend = GitBackend::for_tests();
        backend.init(&remote, AuthMode::Ssh, local.path()).unwrap();

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let dir = backend.conflict_dir_for(ts);
        let tail = dir.file_name().unwrap().to_string_lossy().to_string();

        assert_eq!(tail, ts.to_string());
        assert_eq!(
            dir,
            PathBuf::from(local.path())
                .join(".conflicts")
                .join(ts.to_string())
        );
    }
}
