use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use openmacro_lib::{
    commands::prefs::Prefs,
    engine::{NoopNotifySink, Orchestrator},
    form::{FormRunner, NoopFocusBackend, NoopWindowSink},
    hook::{ConfirmKey, HookEvent, ResetCause},
    inject::{clipboard::MockClipboardBackend, Injector, KeyboardAction, KeyboardSink},
    store::{ExpandMode, Snippet},
};

#[derive(Default)]
struct MockSink {
    actions: Vec<KeyboardAction>,
}

impl KeyboardSink for MockSink {
    fn send(&mut self, action: KeyboardAction) {
        self.actions.push(action);
    }
}

struct TestGuard(#[allow(dead_code)] std::sync::MutexGuard<'static, ()>);

impl Drop for TestGuard {
    fn drop(&mut self) {
        openmacro_lib::hook::set_confirm_armed(false);
    }
}

fn test_guard() -> TestGuard {
    let guard = openmacro_lib::hook::winevent::test_sync::global_state_guard();
    openmacro_lib::hook::set_confirm_armed(false);
    TestGuard(guard)
}

fn snippet(trigger: &str, replace: &str) -> Snippet {
    Snippet {
        id: format!("test::{trigger}"),
        trigger: trigger.to_string(),
        raw_trigger: trigger.to_string(),
        trigger_literal: false,
        replace: replace.to_string(),
        vars: Vec::new(),
        source_file: PathBuf::from("test.yaml"),
    }
}

fn manual_orchestrator() -> Orchestrator<MockSink, MockClipboardBackend> {
    let injector = Injector::new_with_sink(MockSink::default());
    let mut orchestrator = Orchestrator::new_with_state(
        vec![snippet(";sig", "hello")],
        injector,
        NoopNotifySink,
        tokio::runtime::Handle::current(),
        Arc::new(FormRunner::new_with_sink(NoopWindowSink)),
        Arc::new(NoopFocusBackend),
        Arc::new(RwLock::new(Prefs::default())),
        Arc::new(openmacro_lib::expand::shell::NoopShellBackend),
    );
    orchestrator.set_expand_mode(ExpandMode::Manual);
    orchestrator
}

#[tokio::test(flavor = "current_thread")]
async fn normal_char_after_arm_disarms_without_expanding() {
    let _guard = test_guard();
    let mut orchestrator = manual_orchestrator();
    for ch in ";sig".chars() {
        let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
    }

    assert!(!orchestrator.handle_event(HookEvent::Char('x')).unwrap());
    assert!(!orchestrator
        .handle_event(HookEvent::Confirm(ConfirmKey::Tab))
        .unwrap());
    assert!(orchestrator.injector().sink().actions.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn backspace_after_arm_disarms_without_expanding() {
    let _guard = test_guard();
    let mut orchestrator = manual_orchestrator();
    for ch in ";sig".chars() {
        let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
    }

    assert!(!orchestrator.handle_event(HookEvent::Backspace).unwrap());
    assert!(!orchestrator
        .handle_event(HookEvent::Confirm(ConfirmKey::Enter))
        .unwrap());
    assert!(orchestrator.injector().sink().actions.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn reset_events_disarm_manual_match() {
    let _guard = test_guard();
    let resets = [
        ResetCause::ArrowKey,
        ResetCause::Home,
        ResetCause::End,
        ResetCause::PageUp,
        ResetCause::PageDown,
        ResetCause::ImeOrComposition,
        ResetCause::CapsToggle,
        ResetCause::ForegroundChange,
    ];

    for reset in resets {
        let mut orchestrator = manual_orchestrator();
        for ch in ";sig".chars() {
            let _ = orchestrator.handle_event(HookEvent::Char(ch)).unwrap();
        }

        assert!(!orchestrator.handle_event(HookEvent::Reset(reset)).unwrap());
        assert!(!orchestrator
            .handle_event(HookEvent::Confirm(ConfirmKey::Tab))
            .unwrap());
        assert!(
            orchestrator.injector().sink().actions.is_empty(),
            "reset {reset:?} should disarm without injection"
        );
    }
}
