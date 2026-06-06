//! Text injection back into the active application.

pub mod clipboard;
pub mod sendinput;

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use clipboard::{ClipboardBackend, MockClipboardBackend, SystemClipboardBackend};
use sendinput::WindowsKeyboardSink;

pub static SUSPEND: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardAction {
    Backspace,
    LeftArrow,
    Unicode(char),
    Paste(String),
}

pub trait KeyboardSink: Send + 'static {
    fn send(&mut self, action: KeyboardAction);
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

    pub fn inject(&mut self, plan: InjectPlan) -> Result<(), InjectError> {
        SUSPEND.store(true, Ordering::Relaxed);

        for _ in 0..plan.backspaces {
            self.sink.send(KeyboardAction::Backspace);
        }

        let paste_result = if plan.text.len() <= plan.max_clipboard_bytes {
            self.clipboard
                .paste(&mut self.sink, &plan.text, plan.clipboard_timeout)
        } else {
            Err(InjectError::new("clipboard paste bypassed by size cap"))
        };

        if paste_result.is_err() {
            for ch in plan.text.chars() {
                self.sink.send(KeyboardAction::Unicode(ch));
            }
        }

        for _ in 0..plan.caret_left {
            self.sink.send(KeyboardAction::LeftArrow);
        }

        SUSPEND.store(false, Ordering::Relaxed);
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
                KeyboardAction::Paste("ok".to_string()),
                KeyboardAction::LeftArrow,
                KeyboardAction::LeftArrow,
            ]
        );
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
