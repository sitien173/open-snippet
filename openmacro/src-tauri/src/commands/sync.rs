use std::{
    path::PathBuf,
    sync::Arc,
};

use serde::Serialize;
use tempfile::TempDir;

use crate::sync::{
    credential_key, git_remote_callbacks, AuthMode, CredentialStore, GitBackend, NoopNotifySink,
    NotifySink, Secret, SyncBackend, SyncStatus, TickReport, WindowsCredentialStore,
};

pub struct SyncCommandState {
    sync_root: PathBuf,
    backend: Arc<dyn SyncBackend>,
    credential_store: Arc<dyn CredentialStore>,
}

impl SyncCommandState {
    pub fn new(sync_root: PathBuf) -> Self {
        let credential_store: Arc<dyn CredentialStore> = Arc::new(WindowsCredentialStore);
        let notify: Arc<dyn NotifySink> = Arc::new(NoopNotifySink);
        let backend: Arc<dyn SyncBackend> =
            Arc::new(GitBackend::new(Arc::clone(&credential_store), notify));
        Self {
            sync_root,
            backend,
            credential_store,
        }
    }

    pub fn new_for_tests(
        sync_root: PathBuf,
        backend: Arc<dyn SyncBackend>,
        credential_store: Arc<dyn CredentialStore>,
    ) -> Self {
        Self {
            sync_root,
            backend,
            credential_store,
        }
    }

    pub fn backend(&self) -> Arc<dyn SyncBackend> {
        Arc::clone(&self.backend)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TickReportDto {
    pub kind: String,
    pub dir: Option<String>,
    pub committed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyncStatusDto {
    pub branch: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub last_tick_unix: Option<u64>,
}

#[tauri::command]
pub fn sync_test_connection(
    state: tauri::State<'_, SyncCommandState>,
    remote: String,
    auth: AuthMode,
    pat: Option<String>,
) -> Result<(), String> {
    sync_test_connection_inner(state.inner(), remote, auth, pat)
}

#[tauri::command]
pub fn sync_init(
    state: tauri::State<'_, SyncCommandState>,
    remote: String,
    auth: AuthMode,
    pat: Option<String>,
) -> Result<(), String> {
    sync_init_inner(state.inner(), remote, auth, pat)
}

#[tauri::command]
pub fn sync_tick_now(state: tauri::State<'_, SyncCommandState>) -> Result<TickReportDto, String> {
    sync_tick_now_inner(state.inner()).map(Into::into)
}

#[tauri::command]
pub fn sync_status(state: tauri::State<'_, SyncCommandState>) -> Result<SyncStatusDto, String> {
    sync_status_inner(state.inner()).map(Into::into)
}

pub fn sync_root() -> Result<PathBuf, String> {
    if let Some(override_path) = std::env::var_os("OPENMACRO_SYNC_ROOT") {
        return Ok(PathBuf::from(override_path));
    }
    let Some(config_dir) = dirs::config_dir() else {
        return Err("config directory unavailable".to_string());
    };
    Ok(config_dir.join("openmacro").join("sync"))
}

pub fn sync_test_connection_inner(
    state: &SyncCommandState,
    remote: String,
    auth: AuthMode,
    pat: Option<String>,
) -> Result<(), String> {
    maybe_store_pat(state.credential_store.as_ref(), &auth, pat)?;

    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let repo = git2::Repository::init(temp.path()).map_err(|error| error.to_string())?;
    let mut remote_handle = repo
        .remote_anonymous(&remote)
        .map_err(|error| error.to_string())?;
    let callbacks = git_remote_callbacks(&auth, state.credential_store.as_ref());
    remote_handle
        .connect_auth(git2::Direction::Fetch, Some(callbacks), None)
        .map_err(|error| error.to_string())?;
    let _ = remote_handle.list().map_err(|error| error.to_string())?;
    remote_handle.disconnect().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn sync_init_inner(
    state: &SyncCommandState,
    remote: String,
    auth: AuthMode,
    pat: Option<String>,
) -> Result<(), String> {
    maybe_store_pat(state.credential_store.as_ref(), &auth, pat)?;
    std::fs::create_dir_all(&state.sync_root).map_err(|error| error.to_string())?;
    state
        .backend
        .init(&remote, auth, &state.sync_root)
        .map_err(|error| error.to_string())
}

pub fn sync_tick_now_inner(state: &SyncCommandState) -> Result<TickReport, String> {
    state.backend.tick().map_err(|error| error.to_string())
}

pub fn sync_status_inner(state: &SyncCommandState) -> Result<SyncStatus, String> {
    Ok(state.backend.status())
}

fn maybe_store_pat(
    store: &dyn CredentialStore,
    auth: &AuthMode,
    pat: Option<String>,
) -> Result<(), String> {
    if let (AuthMode::HttpsPat { host, username }, Some(pat)) = (auth, pat) {
        store.write(
            &credential_key(host),
            &crate::sync::SyncCredential {
                username: username.clone(),
                secret: Secret::new(pat),
            },
        )?;
    }
    Ok(())
}

impl From<TickReport> for TickReportDto {
    fn from(value: TickReport) -> Self {
        match value {
            TickReport::NoChanges => Self {
                kind: "no_changes".to_string(),
                dir: None,
                committed: false,
            },
            TickReport::Synced { committed } => Self {
                kind: "synced".to_string(),
                dir: None,
                committed,
            },
            TickReport::Conflict { dir } => Self {
                kind: "conflict".to_string(),
                dir: Some(dir.to_string_lossy().into_owned()),
                committed: false,
            },
        }
    }
}

impl From<SyncStatus> for SyncStatusDto {
    fn from(value: SyncStatus) -> Self {
        Self {
            branch: value.branch,
            ahead: value.ahead,
            behind: value.behind,
            last_tick_unix: value.last_tick_unix,
        }
    }
}
