//! Runtime orchestration for hook, matcher, and injection.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

use crate::{
    hook::{HookEvent, ResetCause},
    inject::{InjectError, InjectPlan, Injector, KeyboardSink, SUSPEND},
    matcher::{MatchBuffer, Matcher, Reset},
    store::Snippet,
};

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

pub struct Orchestrator<S: KeyboardSink, B: crate::inject::clipboard::ClipboardBackend> {
    matcher: Matcher,
    buffer: MatchBuffer,
    snippets: HashMap<Arc<str>, Snippet>,
    injector: Injector<S, B>,
    max_expansion_len: usize,
}

impl<S: KeyboardSink, B: crate::inject::clipboard::ClipboardBackend> Orchestrator<S, B> {
    pub fn new(snippets: Vec<Snippet>, injector: Injector<S, B>) -> Self {
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
            injector,
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

                if snippet.replace.chars().count() > self.max_expansion_len {
                    return Ok(false);
                }

                self.injector.inject(InjectPlan {
                    backspaces: hit.trigger_len_chars,
                    text: snippet.replace.clone(),
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

    pub fn injector(&self) -> &Injector<S, B> {
        &self.injector
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
    use std::path::PathBuf;

    use crate::{
        hook::{winevent::set_denylisted, HookEvent, ResetCause},
        inject::{clipboard::MockClipboardBackend, Injector, KeyboardAction, KeyboardSink},
        store::Snippet,
    };

    use super::{is_paused, set_paused, toggle_paused, Orchestrator};

    #[derive(Default)]
    struct MockSink {
        actions: Vec<KeyboardAction>,
    }

    impl KeyboardSink for MockSink {
        fn send(&mut self, action: KeyboardAction) {
            self.actions.push(action);
        }
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

    #[test]
    fn pause_toggle_flips_atomic_state() {
        set_paused(false);
        assert!(!is_paused());
        assert!(toggle_paused());
        assert!(is_paused());
        assert!(!toggle_paused());
        assert!(!is_paused());
    }

    #[test]
    fn paused_orchestrator_drops_char_input() {
        let injector = Injector::new_with_sink(MockSink::default());
        let mut orchestrator = Orchestrator::new(vec![snippet(";sig", "hello")], injector);
        set_paused(true);

        let injected = orchestrator.handle_event(HookEvent::Char(';')).unwrap();

        assert!(!injected);
        assert!(orchestrator.injector().sink().actions.is_empty());
        set_paused(false);
    }

    #[test]
    fn max_expansion_len_cap_blocks_injection() {
        let injector = Injector::new_with_sink(MockSink::default());
        let mut orchestrator =
            Orchestrator::new(vec![snippet(";sig", "too long replacement")], injector);
        orchestrator.set_max_expansion_len(3);

        for ch in ";sig".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert!(orchestrator.injector().sink().actions.is_empty());
    }

    #[test]
    fn foreground_reset_clears_partial_match() {
        let injector = Injector::<MockSink, MockClipboardBackend>::new_with_sink(MockSink::default());
        let mut orchestrator = Orchestrator::new(vec![snippet(";sig", "hello")], injector);
        let _ = orchestrator.handle_event(HookEvent::Char(';')).unwrap();
        let _ = orchestrator.handle_event(HookEvent::Char('s')).unwrap();

        let injected = orchestrator
            .handle_event(HookEvent::Reset(ResetCause::ForegroundChange))
            .unwrap();
        let after = orchestrator.handle_event(HookEvent::Char('i')).unwrap();

        assert!(!injected);
        assert!(!after);
    }

    #[test]
    fn denylisted_gate_blocks_char_input() {
        let injector = Injector::new_with_sink(MockSink::default());
        let mut orchestrator = Orchestrator::new(vec![snippet(";sig", "hello")], injector);
        set_denylisted(true);

        let injected = orchestrator.handle_event(HookEvent::Char(';')).unwrap();

        assert!(!injected);
        assert!(orchestrator.injector().sink().actions.is_empty());
        set_denylisted(false);
    }
}
