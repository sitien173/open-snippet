//! Text injection back into the active application.

pub mod clipboard;
pub mod sendinput;

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use clipboard::{ClipboardBackend, MockClipboardBackend, SystemClipboardBackend};
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

        let mut actions = Vec::with_capacity(plan.backspaces + text_chars.max(1));
        for _ in 0..plan.backspaces {
            actions.push(KeyboardAction::Backspace);
        }

        let paste_result = if text_chars <= UNICODE_DIRECT_THRESHOLD_CHARS {
            tracing::debug!(
                text_chars,
                "using direct unicode injection for short replacement"
            );
            Err(InjectError::new("short replacement uses unicode path"))
        } else if plan.text.len() <= plan.max_clipboard_bytes {
            self.clipboard
                .paste(&mut self.sink, &plan.text, plan.clipboard_timeout)
        } else {
            Err(InjectError::new("clipboard paste bypassed by size cap"))
        };

        if !POST_BACKSPACE_DELAY.is_zero() {
            thread::sleep(POST_BACKSPACE_DELAY);
        }

        if paste_result.is_err() {
            tracing::debug!("clipboard paste failed or bypassed; falling back to unicode input");
            for ch in plan.text.chars() {
                actions.push(KeyboardAction::Unicode(ch));
            }
        } else {
            tracing::debug!("clipboard paste succeeded");
            actions.push(KeyboardAction::Paste(plan.text.clone()));
        }
        self.sink.send_batch(&actions);

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
    use std::time::Duration;

    use super::{InjectPlan, Injector, KeyboardAction, KeyboardSink};

    #[derive(Default)]
    struct MockSink {
        actions: Vec<KeyboardAction>,
    }

    impl KeyboardSink for MockSink {
        fn send(&mut self, action: KeyboardAction) {
            self.actions.push(action);
        }
    }

    #[test]
    fn injector_sends_backspace_then_text_with_mocked_sink() {
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

    #[cfg(windows)]
    #[test]
    fn clipboard_snapshot_restores_multiple_formats() {
        use crate::inject::clipboard::{capture_clipboard, restore_clipboard, set_clipboard_text};

        set_clipboard_text("phase3-a").unwrap();
        let snapshot = capture_clipboard().unwrap();
        set_clipboard_text("phase3-b").unwrap();
        restore_clipboard(&snapshot).unwrap();
        let restored = capture_clipboard().unwrap();

        assert!(restored.formats.len() >= 2);
        assert_eq!(restored.text_content().as_deref(), Some("phase3-a"));
    }
}
