use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use openmacro_lib::{
    commands::sync::{sync_init_inner, sync_status_inner, sync_tick_now_inner, SyncCommandState},
    sync::{
        credential_key, validate_credential_callback_url, AuthMode, CredentialStore, GitBackend,
        NoopNotifySink, SyncBackend, SyncCredential, SyncError, SyncStatus, TickReport,
        WindowsCredentialStore,
    },
};
use tempfile::TempDir;

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn seed_remote() -> (TempDir, String) {
    let root = TempDir::new().unwrap();
    let bare = root.path().join("remote.git");
    let seed = root.path().join("seed");
    let bare_repo = git2::Repository::init_bare(&bare).unwrap();
    let repo = git2::Repository::init(&seed).unwrap();
    write_file(&seed, "shared.txt", "seed\n");

    let mut index = repo.index().unwrap();
    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("openmacro", "openmacro@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "seed", &tree, &[])
        .unwrap();
    let mut remote = repo.remote("origin", bare.to_str().unwrap()).unwrap();
    remote
        .push(&["refs/heads/master:refs/heads/master"], None)
        .unwrap();
    drop(bare_repo);

    (root, bare.to_string_lossy().into_owned())
}

#[derive(Default)]
struct MockCredentialStore {
    entries: Mutex<HashMap<String, SyncCredential>>,
}

impl CredentialStore for MockCredentialStore {
    fn read(&self, key: &str) -> Result<Option<SyncCredential>, String> {
        Ok(self.entries.lock().unwrap().get(key).cloned())
    }

    fn write(&self, key: &str, credential: &SyncCredential) -> Result<(), String> {
        self.entries
            .lock()
            .unwrap()
            .insert(key.to_string(), credential.clone());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        self.entries.lock().unwrap().remove(key);
        Ok(())
    }
}

#[derive(Default)]
struct MockBackend {
    init_calls: Mutex<Vec<String>>,
}

impl MockBackend {
    fn init_count(&self) -> usize {
        self.init_calls.lock().unwrap().len()
    }
}

impl SyncBackend for MockBackend {
    fn init(&self, remote: &str, _auth: AuthMode, _local_dir: &Path) -> Result<(), SyncError> {
        self.init_calls.lock().unwrap().push(remote.to_string());
        Ok(())
    }

    fn tick(&self) -> Result<TickReport, SyncError> {
        Ok(TickReport::NoChanges)
    }

    fn status(&self) -> SyncStatus {
        SyncStatus::default()
    }
}

#[test]
fn https_pat_remote_host_mismatch_rejects_before_storing_or_init() {
    let local = TempDir::new().unwrap();
    let backend = Arc::new(MockBackend::default());
    let store = Arc::new(MockCredentialStore::default());
    let state =
        SyncCommandState::new_for_tests(local.path().to_path_buf(), backend.clone(), store.clone());

    let error = sync_init_inner(
        &state,
        "https://github.com/openmacro/snippets.git".to_string(),
        AuthMode::HttpsPat {
            host: "evil.example".to_string(),
            username: "alice".to_string(),
        },
        Some("pat-secret".to_string()),
    )
    .unwrap_err();

    assert!(error.contains("remote host"), "{error}");
    assert_eq!(backend.init_count(), 0);
    assert!(store
        .read(&credential_key("evil.example"))
        .unwrap()
        .is_none());
}

#[test]
fn credential_callback_url_mismatch_rejects_before_secret_lookup() {
    let auth = AuthMode::HttpsPat {
        host: "github.com".to_string(),
        username: "alice".to_string(),
    };

    let error =
        validate_credential_callback_url(&auth, "https://gitlab.com/openmacro/snippets.git")
            .unwrap_err();

    assert!(error.contains("credential URL host mismatch"), "{error}");
}

#[test]
#[tracing_test::traced_test]
fn client_b_sees_client_a_change_after_ticks() {
    let (_root, remote) = seed_remote();
    let client_a = TempDir::new().unwrap();
    let client_b = TempDir::new().unwrap();

    let state_a = SyncCommandState::new_for_tests(
        client_a.path().to_path_buf(),
        Arc::new(GitBackend::new(
            Arc::new(WindowsCredentialStore),
            Arc::new(NoopNotifySink),
        )),
        Arc::new(WindowsCredentialStore),
    );
    let state_b = SyncCommandState::new_for_tests(
        client_b.path().to_path_buf(),
        Arc::new(GitBackend::new(
            Arc::new(WindowsCredentialStore),
            Arc::new(NoopNotifySink),
        )),
        Arc::new(WindowsCredentialStore),
    );

    sync_init_inner(&state_a, remote.clone(), AuthMode::Ssh, None).unwrap();
    sync_init_inner(&state_b, remote, AuthMode::Ssh, None).unwrap();

    write_file(client_a.path(), "shared.txt", "client-a\n");
    let _ = sync_tick_now_inner(&state_a).unwrap();
    let _ = sync_tick_now_inner(&state_b).unwrap();
    let status_b = sync_status_inner(&state_b).unwrap();

    assert_eq!(
        fs::read_to_string(client_b.path().join("shared.txt"))
            .unwrap()
            .replace("\r\n", "\n"),
        "client-a\n"
    );
    assert_eq!(status_b.ahead, 0);
    assert_eq!(status_b.behind, 0);
}
