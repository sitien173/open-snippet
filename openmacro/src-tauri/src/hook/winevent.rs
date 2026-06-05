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

pub fn apply_foreground_change(
    buffer: &mut MatchBuffer,
    exe_basename: &str,
) -> HookEvent {
    buffer.reset();
    set_denylisted(is_denylisted_process(exe_basename));
    HookEvent::Reset(ResetCause::ForegroundChange)
}

pub fn is_denylisted_process(exe_basename: &str) -> bool {
    let lowered = exe_basename.to_ascii_lowercase();
    DENYLIST.iter().any(|candidate| *candidate == lowered)
}

pub mod testing {
    use crate::matcher::MatchBuffer;

    use super::{apply_foreground_change, HookEvent};

    pub fn inject_foreground_change(
        buffer: &mut MatchBuffer,
        exe_basename: &str,
    ) -> HookEvent {
        apply_foreground_change(buffer, exe_basename)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hook::{winevent::testing, ResetCause},
        matcher::MatchBuffer,
    };

    #[test]
    fn foreground_change_to_denylisted_process_clears_buffer_and_sets_gate() {
        let mut buffer = MatchBuffer::new(64);
        buffer.push_char(';');
        buffer.push_char('s');

        let event = testing::inject_foreground_change(&mut buffer, "ConSent.exe");

        assert_eq!(event, crate::hook::HookEvent::Reset(ResetCause::ForegroundChange));
        assert_eq!(buffer.as_str(), "");
        assert!(super::is_denylisted());
    }

    #[test]
    fn non_denylisted_foreground_change_clears_buffer_without_gate() {
        let mut buffer = MatchBuffer::new(64);
        buffer.push_char(';');
        buffer.push_char('x');

        let event = testing::inject_foreground_change(&mut buffer, "notepad.exe");

        assert_eq!(event, crate::hook::HookEvent::Reset(ResetCause::ForegroundChange));
        assert_eq!(buffer.as_str(), "");
        assert!(!super::is_denylisted());
    }
}
