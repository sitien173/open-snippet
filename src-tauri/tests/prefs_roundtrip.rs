use std::sync::{Mutex, OnceLock};

use openmacro_lib::{
    commands::prefs::{
        get_prefs_inner, load_prefs_state, set_prefs_inner, set_prefs_with_autostart_inner,
        AutostartController, Prefs, MAX_EXPANSION_LEN,
    },
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
        last_crash_check: Some(123),
    };

    set_prefs_inner(&state, prefs.clone()).unwrap();
    let reloaded = load_prefs_state().unwrap();

    assert_eq!(get_prefs_inner(&state), prefs);
    assert_eq!(get_prefs_inner(&reloaded), prefs);
    assert!(is_paused());
}

#[derive(Default)]
struct MockAutostart {
    calls: Mutex<Vec<bool>>,
    fail: bool,
}

impl AutostartController for MockAutostart {
    fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        self.calls.lock().unwrap().push(enabled);
        if self.fail {
            Err("autostart failed".to_string())
        } else {
            Ok(())
        }
    }
}

#[test]
fn rejects_invalid_max_expansion_len_without_persisting() {
    let _guard = prefs_test_guard();
    let root = TempDir::new().unwrap();
    let prefs_path = root.path().join("prefs.json");
    std::env::set_var("OPENMACRO_PREFS_PATH", &prefs_path);
    let state = load_prefs_state().unwrap();

    let mut prefs = get_prefs_inner(&state);
    prefs.max_expansion_len = MAX_EXPANSION_LEN + 1;
    let error = set_prefs_inner(&state, prefs).unwrap_err();

    assert!(error.contains("max_expansion_len"), "{error}");
    assert_eq!(get_prefs_inner(&state).max_expansion_len, 32_768);
    assert_eq!(
        load_prefs_state()
            .unwrap()
            .prefs_handle()
            .read()
            .unwrap()
            .max_expansion_len,
        32_768
    );
}

#[test]
fn autostart_enable_applies_before_persisting() {
    let _guard = prefs_test_guard();
    let root = TempDir::new().unwrap();
    let prefs_path = root.path().join("prefs.json");
    std::env::set_var("OPENMACRO_PREFS_PATH", &prefs_path);
    let state = load_prefs_state().unwrap();
    let autostart = MockAutostart::default();

    let mut prefs = get_prefs_inner(&state);
    prefs.autostart = true;
    set_prefs_with_autostart_inner(&state, prefs.clone(), &autostart).unwrap();

    assert_eq!(*autostart.calls.lock().unwrap(), vec![true]);
    assert_eq!(get_prefs_inner(&state), prefs);
    assert!(std::fs::read_to_string(&prefs_path)
        .unwrap()
        .contains("\"autostart\": true"));
}

#[test]
fn autostart_failure_does_not_persist_or_update_memory() {
    let _guard = prefs_test_guard();
    let root = TempDir::new().unwrap();
    let prefs_path = root.path().join("prefs.json");
    std::env::set_var("OPENMACRO_PREFS_PATH", &prefs_path);
    let state = load_prefs_state().unwrap();
    let autostart = MockAutostart {
        calls: Mutex::new(Vec::new()),
        fail: true,
    };

    let mut prefs = get_prefs_inner(&state);
    prefs.autostart = true;
    let error = set_prefs_with_autostart_inner(&state, prefs, &autostart).unwrap_err();

    assert!(error.contains("autostart failed"), "{error}");
    assert_eq!(*autostart.calls.lock().unwrap(), vec![true]);
    assert!(!get_prefs_inner(&state).autostart);
    assert!(std::fs::read_to_string(&prefs_path)
        .unwrap()
        .contains("\"autostart\": false"));
}
