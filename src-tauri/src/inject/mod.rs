//! Text injection back into the active application.

pub mod clipboard;
pub mod sendinput;

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use clipboard::{
    ClipboardBackend, ClipboardPasteResult, MockClipboardBackend, SystemClipboardBackend,
};
use sendinput::WindowsKeyboardSink;

pub static SUSPEND: AtomicBool = AtomicBool::new(false);
const PRE_INJECT_DELAY: Duration = Duration::from_millis(5);
const POST_BACKSPACE_DELAY: Duration = Duration::from_millis(0);
const UNICODE_DIRECT_THRESHOLD_CHARS: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardAction {
    Backspace,
    LeftArrow,
    Unicode(char),
    Paste(String),
}

pub trait KeyboardSink: Send + 'static {
    fn send(&mut self, action: KeyboardAction);

    fn send_batch(&mut self, actions: &[KeyboardAction]) {
        for action in actions {
            self.send(action.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub struct InjectPlan {
    pub backspaces: usize,
    pub text: String,
    pub caret_left: usize,
    pub max_clipboard_bytes: usize,
    pub clipboard_timeout: Duration,
}

#[derive(Debug)]
pub struct InjectError {
    message: String,
}

impl InjectError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for InjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for InjectError {}

pub struct Injector<S: KeyboardSink, B: ClipboardBackend> {
    sink: S,
    clipboard: B,
}

impl Injector<WindowsKeyboardSink, SystemClipboardBackend> {
    pub fn new() -> Self {
        Self {
            sink: WindowsKeyboardSink,
            clipboard: SystemClipboardBackend,
        }
    }
}

impl Default for Injector<WindowsKeyboardSink, SystemClipboardBackend> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: KeyboardSink> Injector<S, MockClipboardBackend> {
    pub fn new_with_sink(sink: S) -> Self {
        Self {
            sink,
            clipboard: MockClipboardBackend,
        }
    }
}

impl<S: KeyboardSink, B: ClipboardBackend> Injector<S, B> {
    pub fn new_with_parts(sink: S, clipboard: B) -> Self {
        Self { sink, clipboard }
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn clipboard_mut(&mut self) -> &mut B {
        &mut self.clipboard
    }

    #[tracing::instrument(skip(self, plan), fields(backspaces = plan.backspaces, caret_left = plan.caret_left, text_chars = plan.text.chars().count()))]
    pub fn inject(&mut self, plan: InjectPlan) -> Result<(), InjectError> {
        let text_chars = plan.text.chars().count();
        // SECURITY: injected text is snippet output/user content; log only redacted body plus counts.
        tracing::debug!(
            text = %crate::log_body!(&plan.text),
            max_clipboard_bytes = plan.max_clipboard_bytes,
            "injecting text"
        );
        SUSPEND.store(true, Ordering::Relaxed);
        // Let the target application commit the final typed character before we erase the trigger.
        thread::sleep(PRE_INJECT_DELAY);

        let mut backspaces = Vec::with_capacity(plan.backspaces);
        for _ in 0..plan.backspaces {
            backspaces.push(KeyboardAction::Backspace);
        }
        if !backspaces.is_empty() {
            self.sink.send_batch(&backspaces);
        }

        if !POST_BACKSPACE_DELAY.is_zero() {
            thread::sleep(POST_BACKSPACE_DELAY);
        }

        let pasted = if text_chars <= UNICODE_DIRECT_THRESHOLD_CHARS {
            tracing::debug!(
                text_chars,
                "using direct unicode injection for short replacement"
            );
            false
        } else if plan.text.len() <= plan.max_clipboard_bytes {
            match self
                .clipboard
                .paste(&mut self.sink, &plan.text, plan.clipboard_timeout)
            {
                ClipboardPasteResult::Pasted => true,
                ClipboardPasteResult::PastedRestoreFailed(error) => {
                    tracing::warn!(error = %error, "clipboard restore failed after paste");
                    true
                }
                ClipboardPasteResult::Failed(error) => {
                    tracing::debug!(error = %error, "clipboard paste failed before paste action");
                    false
                }
            }
        } else {
            false
        };

        if !pasted {
            tracing::debug!("clipboard paste failed or bypassed; falling back to unicode input");
            let mut actions = Vec::with_capacity(text_chars);
            for ch in plan.text.chars() {
                actions.push(KeyboardAction::Unicode(ch));
            }
            self.sink.send_batch(&actions);
        } else {
            tracing::debug!("clipboard paste succeeded");
        }

        let mut caret_actions = Vec::with_capacity(plan.caret_left);
        for _ in 0..plan.caret_left {
            caret_actions.push(KeyboardAction::LeftArrow);
        }
        if !caret_actions.is_empty() {
            self.sink.send_batch(&caret_actions);
        }

        SUSPEND.store(false, Ordering::Relaxed);
        tracing::debug!("text injection completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::hook::winevent::test_sync::global_state_guard;

    use super::{
        clipboard::{ClipboardBackend, ClipboardPasteResult},
        InjectError, InjectPlan, Injector, KeyboardAction, KeyboardSink,
    };
    use crate::expand::ClipboardReader;

    #[derive(Default)]
    struct MockSink {
        actions: Vec<KeyboardAction>,
    }

    impl KeyboardSink for MockSink {
        fn send(&mut self, action: KeyboardAction) {
            self.actions.push(action);
        }
    }

    #[derive(Clone)]
    struct RecordingClipboardBackend {
        log: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingClipboardBackend {
        fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
            Self { log }
        }
    }

    impl ClipboardReader for RecordingClipboardBackend {
        fn read_text(&mut self) -> Option<String> {
            None
        }
    }

    impl ClipboardBackend for RecordingClipboardBackend {
        fn paste(
            &mut self,
            sink: &mut dyn KeyboardSink,
            text: &str,
            _timeout: Duration,
        ) -> ClipboardPasteResult {
            self.log.lock().unwrap().push(format!("set:{text}"));
            sink.send(KeyboardAction::Paste(text.to_string()));
            self.log.lock().unwrap().push("restore".to_string());
            ClipboardPasteResult::Pasted
        }
    }

    struct RestoreFailingClipboardBackend;

    impl ClipboardReader for RestoreFailingClipboardBackend {
        fn read_text(&mut self) -> Option<String> {
            None
        }
    }

    impl ClipboardBackend for RestoreFailingClipboardBackend {
        fn paste(
            &mut self,
            sink: &mut dyn KeyboardSink,
            text: &str,
            _timeout: Duration,
        ) -> ClipboardPasteResult {
            sink.send(KeyboardAction::Paste(text.to_string()));
            ClipboardPasteResult::PastedRestoreFailed(InjectError::new("restore failed"))
        }
    }

    #[test]
    fn injector_sends_backspace_then_text_with_mocked_sink() {
        let _guard = global_state_guard();
        let sink = MockSink::default();
        let mut injector = Injector::new_with_sink(sink);
        injector
            .inject(InjectPlan {
                backspaces: 3,
                text: "ok".to_string(),
                caret_left: 2,
                max_clipboard_bytes: 4_096,
                clipboard_timeout: Duration::from_millis(50),
            })
            .unwrap();

        let actions = &injector.sink().actions;
        assert_eq!(
            actions,
            &[
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Unicode('o'),
                KeyboardAction::Unicode('k'),
                KeyboardAction::LeftArrow,
                KeyboardAction::LeftArrow,
            ]
        );
    }

    #[test]
    fn injector_uses_unicode_for_500_ascii_chars() {
        let _guard = global_state_guard();
        let sink = MockSink::default();
        let mut injector = Injector::new_with_sink(sink);
        let text = "a".repeat(500);
        injector
            .inject(InjectPlan {
                backspaces: 0,
                text,
                caret_left: 0,
                max_clipboard_bytes: 4_096,
                clipboard_timeout: Duration::from_millis(50),
            })
            .unwrap();

        let actions = &injector.sink().actions;
        assert_eq!(actions.len(), 500);
        assert!(actions
            .iter()
            .all(|action| matches!(action, KeyboardAction::Unicode('a'))));
    }

    #[test]
    fn injector_sends_backspaces_before_clipboard_paste_and_restores_afterward() {
        let _guard = global_state_guard();
        let log = Arc::new(Mutex::new(Vec::new()));
        let sink = MockSink::default();
        let clipboard = RecordingClipboardBackend::new(Arc::clone(&log));
        let mut injector = Injector::new_with_parts(sink, clipboard);
        let text = "a".repeat(3000);

        injector
            .inject(InjectPlan {
                backspaces: 2,
                text: text.clone(),
                caret_left: 1,
                max_clipboard_bytes: 4_096,
                clipboard_timeout: Duration::from_millis(50),
            })
            .unwrap();

        assert_eq!(
            injector.sink().actions,
            vec![
                KeyboardAction::Backspace,
                KeyboardAction::Backspace,
                KeyboardAction::Paste(text),
                KeyboardAction::LeftArrow,
            ]
        );
        assert_eq!(
            log.lock().unwrap().as_slice(),
            &[format!("set:{}", "a".repeat(3000)), "restore".to_string()]
        );
    }

    #[test]
    fn injector_does_not_fallback_to_unicode_after_post_paste_restore_failure() {
        let _guard = global_state_guard();
        let sink = MockSink::default();
        let clipboard = RestoreFailingClipboardBackend;
        let mut injector = Injector::new_with_parts(sink, clipboard);
        let text = "a".repeat(3000);

        injector
            .inject(InjectPlan {
                backspaces: 1,
                text: text.clone(),
                caret_left: 0,
                max_clipboard_bytes: 4_096,
                clipboard_timeout: Duration::from_millis(50),
            })
            .unwrap();

        assert_eq!(
            injector.sink().actions,
            vec![KeyboardAction::Backspace, KeyboardAction::Paste(text)]
        );
    }

    #[cfg(windows)]
    #[test]
    fn clipboard_snapshot_restores_multiple_formats() {
        use crate::inject::clipboard::{capture_clipboard, restore_clipboard, set_clipboard_text};
        let _guard = global_state_guard();

        let Ok(()) = set_clipboard_text("phase3-a") else {
            return;
        };
        let Ok(snapshot) = capture_clipboard() else {
            return;
        };
        let Ok(()) = set_clipboard_text("phase3-b") else {
            return;
        };
        let Ok(()) = restore_clipboard(&snapshot) else {
            return;
        };
        let Ok(restored) = capture_clipboard() else {
            return;
        };

        assert!(restored.formats.len() >= 2);
        assert_eq!(restored.text_content().as_deref(), Some("phase3-a"));
    }
}
