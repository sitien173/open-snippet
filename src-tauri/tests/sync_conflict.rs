use std::{fs, path::PathBuf, sync::{Arc, Mutex}};

use openmacro_lib::{
    commands::sync::{sync_init_inner, sync_tick_now_inner, SyncCommandState},
    sync::{AuthMode, GitBackend, NotifySink, TickReport, WindowsCredentialStore},
};
use tempfile::TempDir;

#[derive(Default)]
struct RecordingNotifySink {
    conflicts: Mutex<Vec<PathBuf>>,
}

impl NotifySink for RecordingNotifySink {
    fn sync_conflict(&self, path: &std::path::Path) {
        self.conflicts.lock().unwrap().push(path.to_path_buf());
    }
}

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
    git2::Repository::init_bare(&bare).unwrap();
    let repo = git2::Repository::init(&seed).unwrap();
    write_file(&seed, "shared.txt", "line\n");

    let mut index = repo.index().unwrap();
    index.add_all(["*"], git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("openmacro", "openmacro@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "seed", &tree, &[]).unwrap();
    let mut remote = repo.remote("origin", bare.to_str().unwrap()).unwrap();
    remote.push(&["refs/heads/master:refs/heads/master"], None).unwrap();

    (root, bare.to_string_lossy().into_owned())
}

#[test]
fn conflict_tick_captures_files_under_conflict_dir_and_notifies() {
    let (_root, remote) = seed_remote();
    let client_a = TempDir::new().unwrap();
    let client_b = TempDir::new().unwrap();
    let notify = Arc::new(RecordingNotifySink::default());

    let state_a = SyncCommandState::new_for_tests(
        client_a.path().to_path_buf(),
        Arc::new(GitBackend::new(
            Arc::new(WindowsCredentialStore),
            Arc::new(RecordingNotifySink::default()),
        )),
        Arc::new(WindowsCredentialStore),
    );
    let state_b = SyncCommandState::new_for_tests(
        client_b.path().to_path_buf(),
        Arc::new(GitBackend::new(Arc::new(WindowsCredentialStore), notify.clone())),
        Arc::new(WindowsCredentialStore),
    );

    sync_init_inner(&state_a, remote.clone(), AuthMode::Ssh, None).unwrap();
    sync_init_inner(&state_b, remote, AuthMode::Ssh, None).unwrap();

    write_file(client_a.path(), "shared.txt", "client-a\n");
    write_file(client_b.path(), "shared.txt", "client-b\n");
    let _ = sync_tick_now_inner(&state_a).unwrap();
    let report = sync_tick_now_inner(&state_b).unwrap();

    let TickReport::Conflict { dir } = report else {
        panic!("expected conflict report");
    };

    assert!(dir.starts_with(client_b.path()));
    assert!(dir.components().any(|component| component.as_os_str() == ".conflicts"));
    assert!(dir.join("shared.txt").exists());
    assert_eq!(notify.conflicts.lock().unwrap().as_slice(), &[dir]);
}
