use std::sync::{Mutex, OnceLock};

use openmacro_lib::{
    commands::prefs::{get_prefs_inner, load_prefs_state, set_prefs_inner, Prefs},
    engine::{is_paused, set_paused},
};
use tempfile::TempDir;

fn prefs_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    std::env::remove_var("OPENMACRO_PREFS_PATH");
    set_paused(false);
    guard
}

#[test]
fn prefs_round_trip_persists_across_reads() {
    let _guard = prefs_test_guard();
    let root = TempDir::new().unwrap();
    let prefs_path = root.path().join("prefs.json");
    std::env::set_var("OPENMACRO_PREFS_PATH", &prefs_path);

    let state = load_prefs_state().unwrap();
    let prefs = Prefs {
        paused: true,
        autostart: true,
        max_expansion_len: 2048,
        shell_consent: true,
    };

    set_prefs_inner(&state, prefs.clone()).unwrap();
    let reloaded = load_prefs_state().unwrap();

    assert_eq!(get_prefs_inner(&state), prefs);
    assert_eq!(get_prefs_inner(&reloaded), prefs);
    assert!(is_paused());
}
