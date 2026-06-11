//! Foreground tracking and denylist integration for the hook thread.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::matcher::MatchBuffer;

use super::{HookEvent, ResetCause};

pub static DENYLISTED: AtomicBool = AtomicBool::new(false);

const DENYLIST: &[&str] = &[
    "1password.exe",
    "keepass.exe",
    "keepassxc.exe",
    "bitwarden.exe",
    "lastpass.exe",
    "consent.exe",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundState {
    pub hwnd: isize,
    pub exe_basename: String,
}

pub fn is_denylisted() -> bool {
    DENYLISTED.load(Ordering::Relaxed)
}

pub fn set_denylisted(value: bool) {
    DENYLISTED.store(value, Ordering::Relaxed);
}

pub fn apply_foreground_change(buffer: &mut MatchBuffer, exe_basename: &str) -> HookEvent {
    buffer.reset();
    let denylisted = is_denylisted_process(exe_basename);
    set_denylisted(denylisted);
    tracing::debug!(
        exe_basename = %exe_basename,
        denylisted,
        "foreground process changed"
    );
    HookEvent::Reset(ResetCause::ForegroundChange)
}

pub fn is_denylisted_process(exe_basename: &str) -> bool {
    let lowered = exe_basename.to_ascii_lowercase();
    DENYLIST.iter().any(|candidate| *candidate == lowered)
}

pub mod testing {
    use crate::matcher::MatchBuffer;

    use super::{apply_foreground_change, HookEvent};

    pub fn inject_foreground_change(buffer: &mut MatchBuffer, exe_basename: &str) -> HookEvent {
        apply_foreground_change(buffer, exe_basename)
    }
}

#[cfg(test)]
pub(crate) mod test_sync {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::DENYLISTED;
    use crate::inject::SUSPEND;

    pub(crate) fn global_state_guard() -> MutexGuard<'static, ()> {
        let guard = GLOBAL_STATE_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        DENYLISTED.store(false, std::sync::atomic::Ordering::Relaxed);
        SUSPEND.store(false, std::sync::atomic::Ordering::Relaxed);
        guard
    }

    static GLOBAL_STATE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
}

#[cfg(test)]
mod tests {
    use crate::{
        hook::{winevent::testing, ResetCause},
        inject::SUSPEND,
        matcher::MatchBuffer,
    };
    use std::sync::atomic::Ordering;

    #[test]
    fn foreground_change_to_denylisted_process_clears_buffer_and_sets_gate() {
        let _guard = super::test_sync::global_state_guard();
        let mut buffer = MatchBuffer::new(64);
        buffer.push_char(';');
        buffer.push_char('s');

        let event = testing::inject_foreground_change(&mut buffer, "ConSent.exe");

        assert_eq!(
            event,
            crate::hook::HookEvent::Reset(ResetCause::ForegroundChange)
        );
        assert_eq!(buffer.as_str(), "");
        assert!(super::is_denylisted());
    }

    #[test]
    fn non_denylisted_foreground_change_clears_buffer_without_gate() {
        let _guard = super::test_sync::global_state_guard();
        let mut buffer = MatchBuffer::new(64);
        buffer.push_char(';');
        buffer.push_char('x');

        let event = testing::inject_foreground_change(&mut buffer, "notepad.exe");

        assert_eq!(
            event,
            crate::hook::HookEvent::Reset(ResetCause::ForegroundChange)
        );
        assert_eq!(buffer.as_str(), "");
        assert!(!super::is_denylisted());
    }

    #[test]
    fn global_state_guard_resets_suspend_flag() {
        SUSPEND.store(true, Ordering::Relaxed);

        let _guard = super::test_sync::global_state_guard();

        assert!(!SUSPEND.load(Ordering::Relaxed));
    }
}
