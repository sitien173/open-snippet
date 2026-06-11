use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use openmacro_lib::{
    commands::prefs::Prefs,
    expand::{
        shell::{decode_stdout, ShellBackend, ShellError, TokioShellBackend},
        ClipboardReader, ResolveError, ResolveNotifySink, Resolver,
    },
    store::{Snippet, VarDecl, VarKind},
};

type ShellCall = (Vec<String>, PathBuf, Duration);
type ConfirmCall = (String, String, Vec<String>);

#[derive(Default)]
struct StubClipboard;

impl ClipboardReader for StubClipboard {
    fn read_text(&mut self) -> Option<String> {
        None
    }
}

#[derive(Clone)]
struct RecordingShellBackend {
    result: Result<String, ShellError>,
    calls: Arc<Mutex<Vec<ShellCall>>>,
}

impl RecordingShellBackend {
    fn succeed(output: &str) -> Self {
        Self {
            result: Ok(output.to_string()),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn calls(&self) -> Vec<ShellCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl ShellBackend for RecordingShellBackend {
    fn run(&self, args: &[String], cwd: &Path, timeout: Duration) -> Result<String, ShellError> {
        self.calls
            .lock()
            .unwrap()
            .push((args.to_vec(), cwd.to_path_buf(), timeout));
        self.result.clone()
    }
}

struct TestNotifySink {
    confirm_result: bool,
    confirms: Arc<Mutex<Vec<ConfirmCall>>>,
}

impl TestNotifySink {
    fn new(confirm_result: bool) -> Self {
        Self {
            confirm_result,
            confirms: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn confirms(&self) -> Vec<ConfirmCall> {
        self.confirms.lock().unwrap().clone()
    }
}

impl ResolveNotifySink for TestNotifySink {
    fn unknown_placeholder(&self, _snippet_id: &str, _name: &str) {}

    fn confirm_shell(&self, snippet_id: &str, name: &str, args: &[String]) -> bool {
        self.confirms.lock().unwrap().push((
            snippet_id.to_string(),
            name.to_string(),
            args.to_vec(),
        ));
        self.confirm_result
    }
}

fn shell_var(confirm: bool) -> VarDecl {
    VarDecl {
        name: "out".to_string(),
        kind: VarKind::Shell,
        label: None,
        default: None,
        required: false,
        options: Vec::new(),
        format: None,
        cmd: vec![
            "cmd".to_string(),
            "/c".to_string(),
            "echo hello".to_string(),
        ],
        timeout_ms: Some(250),
        confirm,
    }
}

fn shell_snippet(confirm: bool) -> Snippet {
    Snippet {
        id: "test::;shell".to_string(),
        trigger: ";shell".to_string(),
        replace: "{{out}}".to_string(),
        vars: vec![shell_var(confirm)],
        source_file: PathBuf::from(r"F:\projects_new\textblaze\openmacro\snippets\shell.yaml"),
    }
}

fn shell_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn resolver_returns_shell_disabled_without_consent() {
    let prefs = Prefs::default();
    let notify = TestNotifySink::new(true);
    let backend = RecordingShellBackend::succeed("hello");
    let resolver = Resolver::new(&prefs)
        .with_notify_sink(&notify)
        .with_shell_backend(&backend);
    let mut clipboard = StubClipboard;

    let error = resolver
        .resolve(&shell_snippet(false), &mut clipboard, None)
        .unwrap_err();

    assert_eq!(
        error,
        ResolveError::ShellDisabled {
            name: "out".to_string(),
        }
    );
    assert!(backend.calls().is_empty());
}

#[test]
fn resolver_confirm_false_blocks_shell_execution() {
    let prefs = Prefs {
        shell_consent: true,
        ..Prefs::default()
    };
    let notify = TestNotifySink::new(false);
    let backend = RecordingShellBackend::succeed("hello");
    let resolver = Resolver::new(&prefs)
        .with_notify_sink(&notify)
        .with_shell_backend(&backend);
    let mut clipboard = StubClipboard;

    let error = resolver
        .resolve(&shell_snippet(true), &mut clipboard, None)
        .unwrap_err();

    assert_eq!(
        error,
        ResolveError::ShellDeclined {
            name: "out".to_string(),
        }
    );
    assert_eq!(
        notify.confirms(),
        vec![(
            "test::;shell".to_string(),
            "out".to_string(),
            vec![
                "cmd".to_string(),
                "/c".to_string(),
                "echo hello".to_string()
            ],
        )]
    );
    assert!(backend.calls().is_empty());
}

#[test]
fn resolver_runs_shell_with_snippet_parent_as_cwd() {
    let prefs = Prefs {
        shell_consent: true,
        ..Prefs::default()
    };
    let notify = TestNotifySink::new(true);
    let backend = RecordingShellBackend::succeed("hello");
    let resolver = Resolver::new(&prefs)
        .with_notify_sink(&notify)
        .with_shell_backend(&backend);
    let mut clipboard = StubClipboard;

    let resolved = resolver
        .resolve(&shell_snippet(true), &mut clipboard, Some(&BTreeMap::new()))
        .unwrap();

    assert_eq!(resolved.text, "hello");
    assert_eq!(
        backend.calls(),
        vec![(
            vec![
                "cmd".to_string(),
                "/c".to_string(),
                "echo hello".to_string()
            ],
            PathBuf::from(r"F:\projects_new\textblaze\openmacro\snippets"),
            Duration::from_millis(250),
        )]
    );
}

#[cfg(windows)]
#[test]
fn tokio_shell_backend_clears_non_path_env_vars() {
    let _guard = shell_test_guard();
    std::env::set_var("OPENMACRO_SHELL_TEST_SECRET", "present");
    let backend = TokioShellBackend;

    let output = backend
        .run(
            &[
                "cmd".to_string(),
                "/c".to_string(),
                "if defined OPENMACRO_SHELL_TEST_SECRET (echo yes) else (echo no)".to_string(),
            ],
            Path::new("."),
            Duration::from_secs(2),
        )
        .unwrap();

    std::env::remove_var("OPENMACRO_SHELL_TEST_SECRET");
    assert_eq!(output, "no");
}

#[cfg(windows)]
#[test]
fn tokio_shell_backend_times_out() {
    let _guard = shell_test_guard();
    let backend = TokioShellBackend;

    let error = backend
        .run(
            &[
                "cmd".to_string(),
                "/c".to_string(),
                "ping -n 6 127.0.0.1 >nul".to_string(),
            ],
            Path::new("."),
            Duration::from_millis(100),
        )
        .unwrap_err();

    assert_eq!(error, ShellError::Timeout);
}

#[cfg(windows)]
#[test]
fn tokio_shell_backend_rejects_non_utf8_stdout() {
    let _guard = shell_test_guard();
    let error = decode_stdout(vec![0xFF, 0xFE]).unwrap_err();

    assert_eq!(error, ShellError::NonUtf8);
}
