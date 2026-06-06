use std::{fs, sync::Arc};

use openmacro_lib::{
    commands::sync::{sync_init_inner, sync_status_inner, sync_tick_now_inner, SyncCommandState},
    sync::{AuthMode, GitBackend, NoopNotifySink, WindowsCredentialStore},
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
    index.add_all(["*"], git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("openmacro", "openmacro@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "seed", &tree, &[]).unwrap();
    let mut remote = repo.remote("origin", bare.to_str().unwrap()).unwrap();
    remote.push(&["refs/heads/master:refs/heads/master"], None).unwrap();
    drop(bare_repo);

    (root, bare.to_string_lossy().into_owned())
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
