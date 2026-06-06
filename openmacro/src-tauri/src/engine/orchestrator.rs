//! Runtime orchestration for hook, matcher, and injection.

use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

use crate::{
    commands::prefs::Prefs,
    expand::{
        shell::{NoopShellBackend, ShellBackend},
        ResolveError, ResolveNotifySink, Resolver,
    },
    form::{restore_on_submit, FocusBackend, FormOutcome, FormRunner, NoopFocusBackend, NoopWindowSink},
    hook::{HookEvent, ResetCause},
    inject::{InjectError, InjectPlan, Injector, KeyboardSink, SUSPEND},
    matcher::{MatchBuffer, Matcher, Reset},
    store::{Snippet, VarKind},
};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub static PAUSED: AtomicBool = AtomicBool::new(false);
const DEFAULT_MAX_EXPANSION_LEN: usize = 32_768;

pub struct EngineHandle;

pub fn start_runtime() -> EngineHandle {
    EngineHandle
}

pub fn is_paused() -> bool {
    PAUSED.load(Ordering::Relaxed)
}

pub fn set_paused(value: bool) {
    PAUSED.store(value, Ordering::Relaxed);
}

pub fn toggle_paused() -> bool {
    let next = !is_paused();
    set_paused(next);
    next
}

#[derive(Default)]
pub struct NoopNotifySink;

impl ResolveNotifySink for NoopNotifySink {
    fn unknown_placeholder(&self, _snippet_id: &str, _name: &str) {}
}

pub struct Orchestrator<
    S: KeyboardSink + Send + 'static,
    B: crate::inject::clipboard::ClipboardBackend,
    N: ResolveNotifySink = NoopNotifySink,
> {
    matcher: Matcher,
    buffer: MatchBuffer,
    snippets: HashMap<Arc<str>, Snippet>,
    injector: Arc<Mutex<Injector<S, B>>>,
    notifier: N,
    prefs: Arc<RwLock<Prefs>>,
    shell_backend: Arc<dyn ShellBackend>,
    runtime: tokio::runtime::Handle,
    form_runner: Arc<FormRunner>,
    focus: Arc<dyn FocusBackend>,
    max_expansion_len: usize,
}

impl<S: KeyboardSink + Send + 'static, B: crate::inject::clipboard::ClipboardBackend> Orchestrator<S, B> {
    pub fn new(snippets: Vec<Snippet>, injector: Injector<S, B>, runtime: tokio::runtime::Handle) -> Self {
        Self::new_with_state(
            snippets,
            injector,
            NoopNotifySink,
            runtime,
            Arc::new(FormRunner::new_with_sink(NoopWindowSink)),
            Arc::new(NoopFocusBackend),
            Arc::new(RwLock::new(Prefs::default())),
            Arc::new(NoopShellBackend),
        )
    }
}

impl<R: tauri::Runtime> ResolveNotifySink for AppHandle<R> {
    fn unknown_placeholder(&self, snippet_id: &str, name: &str) {
        let _ = self
            .notification()
            .builder()
            .title("openmacro placeholder error")
            .body(format!("Unknown placeholder `{name}` in `{snippet_id}`."))
            .show();
    }

    fn confirm_shell(&self, _snippet_id: &str, _name: &str, _args: &[String]) -> bool {
        false
    }
}

impl<S: KeyboardSink + Send + 'static, B: crate::inject::clipboard::ClipboardBackend, N: ResolveNotifySink>
    Orchestrator<S, B, N>
{
    pub fn new_with_notifier(
        snippets: Vec<Snippet>,
        injector: Injector<S, B>,
        notifier: N,
        runtime: tokio::runtime::Handle,
        form_runner: Arc<FormRunner>,
        focus: Arc<dyn FocusBackend>,
    ) -> Self {
        Self::new_with_state(
            snippets,
            injector,
            notifier,
            runtime,
            form_runner,
            focus,
            Arc::new(RwLock::new(Prefs::default())),
            Arc::new(NoopShellBackend),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_state(
        snippets: Vec<Snippet>,
        injector: Injector<S, B>,
        notifier: N,
        runtime: tokio::runtime::Handle,
        form_runner: Arc<FormRunner>,
        focus: Arc<dyn FocusBackend>,
        prefs: Arc<RwLock<Prefs>>,
        shell_backend: Arc<dyn ShellBackend>,
    ) -> Self {
        let mut matcher = Matcher::new();
        let _ = matcher.rebuild(&snippets);
        let snippets = snippets
            .into_iter()
            .map(|snippet| (Arc::<str>::from(snippet.id.clone()), snippet))
            .collect();

        Self {
            matcher,
            buffer: MatchBuffer::new(64),
            snippets,
            injector: Arc::new(Mutex::new(injector)),
            notifier,
            prefs,
            shell_backend,
            runtime,
            form_runner,
            focus,
            max_expansion_len: DEFAULT_MAX_EXPANSION_LEN,
        }
    }

    pub fn set_max_expansion_len(&mut self, max: usize) {
        self.max_expansion_len = max;
    }

    pub fn handle_event(&mut self, event: HookEvent) -> Result<bool, InjectError> {
        match event {
            HookEvent::Char(ch) => {
                if is_paused()
                    || crate::hook::winevent::is_denylisted()
                    || SUSPEND.load(Ordering::Relaxed)
                {
                    return Ok(false);
                }

                let Some(hit) = self.matcher.on_char(&mut self.buffer, ch) else {
                    return Ok(false);
                };

                let Some(snippet) = self.snippets.get(&hit.snippet_id) else {
                    return Ok(false);
                };

                let mut injector = self.injector.lock().unwrap();
                let prefs = self.prefs.read().unwrap().clone();
                let resolver = Resolver::new(&prefs)
                    .with_notify_sink(&self.notifier)
                    .with_shell_backend(self.shell_backend.as_ref());
                let resolved = match resolver.resolve(snippet, injector.clipboard_mut(), None) {
                    Ok(resolved) => resolved,
                    Err(ResolveError::UnknownPlaceholder { name }) => {
                        self.buffer.reset();
                        self.notifier.unknown_placeholder(&snippet.id, &name);
                        return Ok(false);
                    }
                    Err(_) => return Ok(false),
                };

                if resolved.text.chars().count() > self.max_expansion_len {
                    return Ok(false);
                }

                if snippet.vars.iter().any(|var| var.kind == VarKind::Form) {
                    let Some(hwnd) = self.focus.capture_foreground() else {
                        self.buffer.reset();
                        return Ok(false);
                    };
                    self.buffer.reset();
                    let snippet = snippet.clone();
                    let injector = Arc::clone(&self.injector);
                    let focus = Arc::clone(&self.focus);
                    let form_runner = Arc::clone(&self.form_runner);
                    let prefs = Arc::clone(&self.prefs);
                    let shell_backend = Arc::clone(&self.shell_backend);
                    let max_expansion_len = self.max_expansion_len;
                    let notifier = Arc::new(NoopNotifySink);
                    self.runtime.spawn(async move {
                        let outcome = form_runner.run(&snippet, hwnd).await;
                        let Ok(outcome) = outcome else {
                            return;
                        };
                        let FormOutcome::Submitted(values) = &outcome else {
                            return;
                        };

                        let mut clipboard = injector.lock().unwrap();
                        let prefs = prefs.read().unwrap().clone();
                        let resolver = Resolver::new(&prefs)
                            .with_notify_sink(notifier.as_ref())
                            .with_shell_backend(shell_backend.as_ref());
                        let Ok(resolved) = resolver.resolve(
                            &snippet,
                            clipboard.clipboard_mut(),
                            Some(values),
                        ) else {
                            return;
                        };
                        if resolved.text.chars().count() > max_expansion_len {
                            return;
                        }
                        if restore_on_submit(focus.as_ref(), hwnd, &outcome).is_err() {
                            return;
                        }
                        let _ = clipboard.inject(InjectPlan {
                            backspaces: hit.trigger_len_chars,
                            text: resolved.text,
                            caret_left: resolved.cursor_chars_after_token.unwrap_or(0),
                            max_clipboard_bytes: 4_096,
                            clipboard_timeout: std::time::Duration::from_millis(50),
                        });
                    });
                    return Ok(true);
                }

                self.buffer.reset();
                injector.inject(InjectPlan {
                    backspaces: hit.trigger_len_chars,
                    text: resolved.text,
                    caret_left: resolved.cursor_chars_after_token.unwrap_or(0),
                    max_clipboard_bytes: 4_096,
                    clipboard_timeout: std::time::Duration::from_millis(50),
                })?;
                Ok(true)
            }
            HookEvent::Backspace => {
                self.buffer.pop_char();
                Ok(false)
            }
            HookEvent::Reset(cause) => {
                self.buffer.reset_with(map_reset(cause));
                Ok(false)
            }
        }
    }

    pub fn injector(&self) -> std::sync::MutexGuard<'_, Injector<S, B>> {
        self.injector.lock().unwrap()
    }
}

fn map_reset(cause: ResetCause) -> Reset {
    match cause {
        ResetCause::ImeOrComposition => Reset::ImeCompositionStart,
        ResetCause::CapsToggle => Reset::CapsLockToggled,
        ResetCause::ForegroundChange => Reset::FocusChanged,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex, RwLock},
        sync::atomic::Ordering,
    };

    use crate::{
        commands::prefs::Prefs,
        expand::{
            shell::{ShellBackend, ShellError},
            ResolveNotifySink,
        },
        form::{
            FocusBackend, FocusError, ForegroundWindow, FormRunner, NoopFocusBackend,
            NoopWindowSink,
        },
        hook::{winevent::set_denylisted, HookEvent, ResetCause},
        inject::{
            clipboard::{MockClipboardBackend, TestClipboardBackend},
            Injector, KeyboardAction, KeyboardSink,
        },
        store::{Snippet, VarDecl, VarKind},
    };

    use super::{is_paused, set_paused, toggle_paused, NoopNotifySink, Orchestrator};

    #[derive(Default)]
    struct MockSink {
        actions: Vec<KeyboardAction>,
    }

    impl KeyboardSink for MockSink {
        fn send(&mut self, action: KeyboardAction) {
            self.actions.push(action);
        }
    }

    #[derive(Default)]
    struct MockNotifier {
        messages: Mutex<Vec<(String, String)>>,
    }

    impl ResolveNotifySink for MockNotifier {
        fn unknown_placeholder(&self, snippet_id: &str, name: &str) {
            self.messages
                .lock()
                .unwrap()
                .push((snippet_id.to_string(), name.to_string()));
        }

        fn confirm_shell(&self, _snippet_id: &str, _name: &str, _args: &[String]) -> bool {
            true
        }
    }

    #[derive(Clone)]
    struct MockShellBackend {
        output: String,
        calls: Arc<Mutex<Vec<Vec<String>>>>,
    }

    impl MockShellBackend {
        fn new(output: &str) -> Self {
            Self {
                output: output.to_string(),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl ShellBackend for MockShellBackend {
        fn run(
            &self,
            args: &[String],
            _cwd: &std::path::Path,
            _timeout: std::time::Duration,
        ) -> Result<String, ShellError> {
            self.calls.lock().unwrap().push(args.to_vec());
            Ok(self.output.clone())
        }
    }

    #[derive(Default)]
    struct MockFocusBackend {
        captured: Option<ForegroundWindow>,
        restored: Arc<std::sync::Mutex<Vec<ForegroundWindow>>>,
    }

    impl FocusBackend for MockFocusBackend {
        fn capture_foreground(&self) -> Option<ForegroundWindow> {
            self.captured
        }

        fn restore_foreground(&self, hwnd: ForegroundWindow) -> Result<(), FocusError> {
            self.restored.lock().unwrap().push(hwnd);
            Ok(())
        }
    }

    fn test_guard() -> impl Drop {
        let guard = crate::hook::winevent::test_sync::global_state_guard();
        set_paused(false);
        crate::inject::SUSPEND.store(false, Ordering::Relaxed);
        guard
    }

    fn snippet(trigger: &str, replace: &str) -> Snippet {
        Snippet {
            id: format!("test::{trigger}"),
            trigger: trigger.to_string(),
            replace: replace.to_string(),
            vars: Vec::new(),
            source_file: PathBuf::from("test.yaml"),
        }
    }

    fn snippet_with_vars(trigger: &str, replace: &str, vars: Vec<VarDecl>) -> Snippet {
        Snippet {
            vars,
            ..snippet(trigger, replace)
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pause_toggle_flips_atomic_state() {
        let _guard = test_guard();
        set_paused(false);
        assert!(!is_paused());
        assert!(toggle_paused());
        assert!(is_paused());
        assert!(!toggle_paused());
        assert!(!is_paused());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn paused_orchestrator_drops_char_input() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let mut orchestrator =
            Orchestrator::new(vec![snippet(";sig", "hello")], injector, tokio::runtime::Handle::current());
        set_paused(true);

        let injected = orchestrator.handle_event(HookEvent::Char(';')).unwrap();

        assert!(!injected);
        assert!(orchestrator.injector().sink().actions.is_empty());
        set_paused(false);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn max_expansion_len_cap_blocks_injection() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let mut orchestrator = Orchestrator::new(
            vec![snippet(";sig", "too long replacement")],
            injector,
            tokio::runtime::Handle::current(),
        );
        orchestrator.set_max_expansion_len(3);

        for ch in ";sig".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert!(orchestrator.injector().sink().actions.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn foreground_reset_clears_partial_match() {
        let _guard = test_guard();
        let injector = Injector::<MockSink, MockClipboardBackend>::new_with_sink(MockSink::default());
        let mut orchestrator =
            Orchestrator::new(vec![snippet(";sig", "hello")], injector, tokio::runtime::Handle::current());
        let _ = orchestrator.handle_event(HookEvent::Char(';')).unwrap();
        let _ = orchestrator.handle_event(HookEvent::Char('s')).unwrap();

        let injected = orchestrator
            .handle_event(HookEvent::Reset(ResetCause::ForegroundChange))
            .unwrap();
        let after = orchestrator.handle_event(HookEvent::Char('i')).unwrap();

        assert!(!injected);
        assert!(!after);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn denylisted_gate_blocks_char_input() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let mut orchestrator =
            Orchestrator::new(vec![snippet(";sig", "hello")], injector, tokio::runtime::Handle::current());
        set_denylisted(true);

        let injected = orchestrator.handle_event(HookEvent::Char(';')).unwrap();

        assert!(!injected);
        assert!(orchestrator.injector().sink().actions.is_empty());
        set_denylisted(false);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolved_round_trip_for_now_and_log_snippets() {
        let _guard = test_guard();
        let injector =
            Injector::new_with_parts(MockSink::default(), TestClipboardBackend::with_text("copied"));
        let mut orchestrator = Orchestrator::new(
            vec![
                snippet(";now", "{{date:%Y}}"),
                snippet_with_vars(
                    ";log",
                    "{{clip}}$|$ tail",
                    vec![VarDecl {
                        name: "clip".to_string(),
                        kind: VarKind::Clipboard,
                        label: None,
                        default: None,
                        required: false,
                        options: Vec::new(),
                        format: None,
                        cmd: Vec::new(),
                        timeout_ms: None,
                        confirm: false,
                    }],
                ),
            ],
            injector,
            tokio::runtime::Handle::current(),
        );

        for ch in ";now".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }
        for ch in ";log".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert_eq!(
            orchestrator.injector().sink().actions,
            vec![
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Paste(chrono::Local::now().format("%Y").to_string()),
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Paste("copied tail".to_string()),
                KeyboardAction::LeftArrow,
                KeyboardAction::LeftArrow,
                KeyboardAction::LeftArrow,
                KeyboardAction::LeftArrow,
                KeyboardAction::LeftArrow,
            ]
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unknown_placeholder_notifies_and_skips_injection() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let notifier = MockNotifier::default();
        let mut orchestrator = Orchestrator::new_with_notifier(
            vec![snippet(";oops", "{{missing}}")],
            injector,
            notifier,
            tokio::runtime::Handle::current(),
            Arc::new(FormRunner::new_with_sink(NoopWindowSink)),
            Arc::new(NoopFocusBackend),
        );

        for ch in ";oops".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert!(orchestrator.injector().sink().actions.is_empty());
        assert_eq!(
            orchestrator.notifier.messages.into_inner().unwrap(),
            vec![("test::;oops".to_string(), "missing".to_string())]
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shell_snippet_respects_prefs_consent() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let prefs = Arc::new(RwLock::new(Prefs::default()));
        let shell_backend = Arc::new(MockShellBackend::new("hello"));
        let mut orchestrator = Orchestrator::new_with_state(
            vec![snippet_with_vars(
                ";sh",
                "{{out}}",
                vec![VarDecl {
                    name: "out".to_string(),
                    kind: VarKind::Shell,
                    label: None,
                    default: None,
                    required: false,
                    options: Vec::new(),
                    format: None,
                    cmd: vec!["cmd".to_string(), "/c".to_string(), "echo hello".to_string()],
                    timeout_ms: Some(200),
                    confirm: false,
                }],
            )],
            injector,
            NoopNotifySink,
            tokio::runtime::Handle::current(),
            Arc::new(FormRunner::new_with_sink(NoopWindowSink)),
            Arc::new(NoopFocusBackend),
            prefs,
            shell_backend.clone(),
        );

        for ch in ";sh".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert!(orchestrator.injector().sink().actions.is_empty());
        assert!(shell_backend.calls().is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shell_snippet_injects_backend_output_when_consent_enabled() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let prefs = Arc::new(RwLock::new(Prefs {
            shell_consent: true,
            ..Prefs::default()
        }));
        let shell_backend = Arc::new(MockShellBackend::new("hello"));
        let mut orchestrator = Orchestrator::new_with_state(
            vec![snippet_with_vars(
                ";sh",
                "{{out}}",
                vec![VarDecl {
                    name: "out".to_string(),
                    kind: VarKind::Shell,
                    label: None,
                    default: None,
                    required: false,
                    options: Vec::new(),
                    format: None,
                    cmd: vec!["cmd".to_string(), "/c".to_string(), "echo hello".to_string()],
                    timeout_ms: Some(200),
                    confirm: false,
                }],
            )],
            injector,
            NoopNotifySink,
            tokio::runtime::Handle::current(),
            Arc::new(FormRunner::new_with_sink(NoopWindowSink)),
            Arc::new(NoopFocusBackend),
            prefs,
            shell_backend.clone(),
        );

        for ch in ";sh".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert_eq!(
            orchestrator.injector().sink().actions,
            vec![
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Paste("hello".to_string()),
            ]
        );
        assert_eq!(
            shell_backend.calls(),
            vec![vec!["cmd".to_string(), "/c".to_string(), "echo hello".to_string()]]
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn form_cancel_preserves_literal_trigger_and_skips_restore() {
        let _guard = test_guard();
        let injector = Injector::new_with_sink(MockSink::default());
        let runner = Arc::new(FormRunner::new_with_sink(NoopWindowSink));
        let restored = Arc::new(std::sync::Mutex::new(Vec::new()));
        let focus = Arc::new(MockFocusBackend {
            captured: Some(ForegroundWindow(55)),
            restored: Arc::clone(&restored),
        });
        let mut orchestrator = Orchestrator::new_with_notifier(
            vec![snippet_with_vars(
                ";form",
                "Hello {{name}}",
                vec![VarDecl {
                    name: "name".to_string(),
                    kind: VarKind::Form,
                    label: Some("Name".to_string()),
                    default: None,
                    required: true,
                    options: Vec::new(),
                    format: None,
                    cmd: Vec::new(),
                    timeout_ms: None,
                    confirm: false,
                }],
            )],
            injector,
            NoopNotifySink,
            tokio::runtime::Handle::current(),
            Arc::clone(&runner),
            focus,
        );

        for ch in ";form".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }
        tokio::task::yield_now().await;
        runner.cancel("test::;form");
        tokio::task::yield_now().await;

        assert!(orchestrator.injector().sink().actions.is_empty());
        assert!(restored.lock().unwrap().is_empty());
    }
}
