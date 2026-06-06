use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use openmacro_lib::{
    expand::{ClipboardReader, Resolver},
    form::{
        capture_foreground_with, restore_on_submit, FocusBackend, FocusError, ForegroundWindow,
        FormError, FormRunner, WindowSink,
    },
    store::{Snippet, VarDecl, VarKind},
};

#[derive(Default)]
struct StubClipboard;

impl ClipboardReader for StubClipboard {
    fn read_text(&mut self) -> Option<String> {
        None
    }
}

#[derive(Default, Clone)]
struct MockFocusBackend {
    captured: Arc<Mutex<Option<ForegroundWindow>>>,
    restored: Arc<Mutex<Vec<ForegroundWindow>>>,
}

impl MockFocusBackend {
    fn with_capture(hwnd: ForegroundWindow) -> Self {
        Self {
            captured: Arc::new(Mutex::new(Some(hwnd))),
            restored: Arc::default(),
        }
    }

    fn restored(&self) -> Vec<ForegroundWindow> {
        self.restored.lock().unwrap().clone()
    }
}

impl FocusBackend for MockFocusBackend {
    fn capture_foreground(&self) -> Option<ForegroundWindow> {
        *self.captured.lock().unwrap()
    }

    fn restore_foreground(&self, hwnd: ForegroundWindow) -> Result<(), FocusError> {
        self.restored.lock().unwrap().push(hwnd);
        Ok(())
    }
}

#[derive(Default, Clone)]
struct MockWindowSink {
    opened: Arc<Mutex<Vec<String>>>,
}

impl MockWindowSink {
    fn opened(&self) -> Vec<String> {
        self.opened.lock().unwrap().clone()
    }
}

impl WindowSink for MockWindowSink {
    fn open_form(&self, snippet_id: &str, _hwnd: ForegroundWindow) -> Result<(), FormError> {
        self.opened.lock().unwrap().push(snippet_id.to_string());
        Ok(())
    }
}

fn snippet(replace: &str, vars: Vec<VarDecl>) -> Snippet {
    Snippet {
        id: "test::snippet".to_string(),
        trigger: ";sig".to_string(),
        replace: replace.to_string(),
        vars,
        source_file: PathBuf::from("test.yaml"),
    }
}

#[test]
fn capture_foreground_returns_registered_value() {
    let focus = MockFocusBackend::with_capture(ForegroundWindow(42));

    let captured = capture_foreground_with(&focus);

    assert_eq!(captured, Some(ForegroundWindow(42)));
}

#[tokio::test(flavor = "current_thread")]
async fn restore_is_called_with_captured_value_on_submit() {
    let focus = MockFocusBackend::with_capture(ForegroundWindow(77));
    let window_sink = MockWindowSink::default();
    let runner = Arc::new(FormRunner::new_with_sink(window_sink.clone()));
    let snippet = snippet("{{name}}", vec![VarDecl {
        name: "name".to_string(),
        kind: VarKind::Form,
        label: Some("Name".to_string()),
        default: None,
        required: true,
        options: Vec::new(),
        format: None,
    }]);
    let runner_task = {
        let runner = Arc::clone(&runner);
        let snippet = snippet.clone();
        tokio::spawn(async move { runner.run(&snippet, ForegroundWindow(77)).await })
    };
    tokio::task::yield_now().await;

    runner.submit(
        &snippet.id,
        BTreeMap::from([("name".to_string(), "Ada".to_string())]),
    );

    let outcome = runner_task.await.unwrap().unwrap();
    restore_on_submit(&focus, ForegroundWindow(77), &outcome).unwrap();

    assert_eq!(window_sink.opened(), vec![snippet.id.clone()]);
    assert_eq!(focus.restored(), vec![ForegroundWindow(77)]);
}

#[tokio::test(flavor = "current_thread")]
async fn restore_is_not_called_on_cancel() {
    let focus = MockFocusBackend::with_capture(ForegroundWindow(11));
    let runner = Arc::new(FormRunner::new_with_sink(MockWindowSink::default()));
    let snippet = snippet("{{name}}", vec![VarDecl {
        name: "name".to_string(),
        kind: VarKind::Form,
        label: Some("Name".to_string()),
        default: None,
        required: true,
        options: Vec::new(),
        format: None,
    }]);
    let runner_task = {
        let runner = Arc::clone(&runner);
        let snippet = snippet.clone();
        tokio::spawn(async move { runner.run(&snippet, ForegroundWindow(11)).await })
    };
    tokio::task::yield_now().await;

    runner.cancel(&snippet.id);

    let outcome = runner_task.await.unwrap().unwrap();
    restore_on_submit(&focus, ForegroundWindow(11), &outcome).unwrap();

    assert_eq!(focus.restored(), Vec::<ForegroundWindow>::new());
}

#[tokio::test(flavor = "current_thread")]
async fn reentrancy_rejects_second_run() {
    let runner = Arc::new(FormRunner::new_with_sink(MockWindowSink::default()));
    let snippet = snippet("{{name}}", vec![VarDecl {
        name: "name".to_string(),
        kind: VarKind::Form,
        label: Some("Name".to_string()),
        default: None,
        required: true,
        options: Vec::new(),
        format: None,
    }]);

    let first = {
        let runner = Arc::clone(&runner);
        let snippet = snippet.clone();
        tokio::spawn(async move { runner.run(&snippet, ForegroundWindow(1)).await })
    };
    tokio::task::yield_now().await;
    let second = runner.run(&snippet, ForegroundWindow(2)).await.unwrap_err();

    runner.cancel(&snippet.id);
    let _ = first.await.unwrap();

    assert_eq!(second, FormError::AlreadyOpen);
}

#[test]
fn form_values_overlay_produces_resolved_text() {
    let snippet = snippet(
        "Hello {{name}}",
        vec![VarDecl {
            name: "name".to_string(),
            kind: VarKind::Form,
            label: Some("Name".to_string()),
            default: Some("default".to_string()),
            required: true,
            options: Vec::new(),
            format: None,
        }],
    );
    let mut clipboard = StubClipboard;
    let values = BTreeMap::from([("name".to_string(), "Ada".to_string())]);

    let resolved = Resolver::resolve(&snippet, &mut clipboard, Some(&values)).unwrap();

    assert_eq!(resolved.text, "Hello Ada");
}
