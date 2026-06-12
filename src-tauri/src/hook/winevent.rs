//! Foreground tracking and denylist integration for the hook thread.

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use crate::matcher::MatchBuffer;

use super::{HookEvent, ResetCause};

pub static DENYLISTED: AtomicBool = AtomicBool::new(false);
static LAST_FOREGROUND_HWND: AtomicIsize = AtomicIsize::new(0);

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
    apply_foreground_basename(exe_basename)
}

pub fn apply_foreground_basename(exe_basename: &str) -> HookEvent {
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

pub fn handle_foreground_window_event<F>(
    last_hwnd: &AtomicIsize,
    hwnd: isize,
    basename_lookup: F,
) -> Option<HookEvent>
where
    F: FnOnce(isize) -> Option<String>,
{
    if hwnd == 0 || last_hwnd.swap(hwnd, Ordering::Relaxed) == hwnd {
        return None;
    }

    let exe_basename = basename_lookup(hwnd).unwrap_or_default();
    Some(apply_foreground_basename(&exe_basename))
}

pub fn reset_foreground_tracking() {
    LAST_FOREGROUND_HWND.store(0, Ordering::Relaxed);
}

#[cfg(windows)]
pub fn foreground_event_from_hwnd(hwnd: isize) -> Option<HookEvent> {
    handle_foreground_window_event(&LAST_FOREGROUND_HWND, hwnd, lookup_process_basename)
}

#[cfg(windows)]
fn lookup_process_basename(hwnd: isize) -> Option<String> {
    use std::path::Path;

    use windows::core::PWSTR;
    use windows::Win32::{
        Foundation::{CloseHandle, HWND},
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };

    let hwnd = HWND(hwnd as *mut core::ffi::c_void);
    let mut process_id = 0u32;
    unsafe {
        // SAFETY: hwnd is an opaque handle provided by the OS or tests and used only for process lookup.
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }
    if process_id == 0 {
        return None;
    }

    let process = unsafe {
        // SAFETY: requests read-only query rights for the owning process.
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
    }
    .ok()?;

    let mut buffer = vec![0u16; 260];
    let mut len = buffer.len() as u32;
    let result = unsafe {
        // SAFETY: buffer is valid writable memory and len is initialized to its capacity.
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
    };
    unsafe {
        // SAFETY: process handle came from OpenProcess above.
        let _ = CloseHandle(process);
    }
    if result.is_err() {
        return None;
    }

    let path = String::from_utf16_lossy(&buffer[..len as usize]);
    Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase())
}

pub mod testing {
    use std::sync::atomic::AtomicIsize;

    use crate::matcher::MatchBuffer;

    use super::{apply_foreground_change, HookEvent};

    pub fn inject_foreground_change(buffer: &mut MatchBuffer, exe_basename: &str) -> HookEvent {
        apply_foreground_change(buffer, exe_basename)
    }

    pub fn inject_foreground_window_event<F>(
        last_hwnd: &AtomicIsize,
        hwnd: isize,
        basename_lookup: F,
    ) -> Option<HookEvent>
    where
        F: FnOnce(isize) -> Option<String>,
    {
        super::handle_foreground_window_event(last_hwnd, hwnd, basename_lookup)
    }
}

#[doc(hidden)]
pub mod test_sync {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::DENYLISTED;
    use crate::{hook::set_confirm_armed, inject::SUSPEND};

    pub fn global_state_guard() -> MutexGuard<'static, ()> {
        let guard = GLOBAL_STATE_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        DENYLISTED.store(false, std::sync::atomic::Ordering::Relaxed);
        super::reset_foreground_tracking();
        SUSPEND.store(false, std::sync::atomic::Ordering::Relaxed);
        set_confirm_armed(false);
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
    use std::sync::atomic::{AtomicIsize, Ordering};

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

    #[test]
    fn repeated_foreground_event_for_same_window_is_ignored() {
        let _guard = super::test_sync::global_state_guard();
        let last_hwnd = AtomicIsize::new(0);

        let first = testing::inject_foreground_window_event(&last_hwnd, 42, |_| {
            Some("keepassxc.exe".to_string())
        });
        let second = testing::inject_foreground_window_event(&last_hwnd, 42, |_| {
            Some("notepad.exe".to_string())
        });

        assert_eq!(
            first,
            Some(crate::hook::HookEvent::Reset(ResetCause::ForegroundChange))
        );
        assert!(second.is_none());
        assert!(super::is_denylisted());
    }

    #[test]
    fn missing_process_name_fails_closed_without_panicking() {
        let _guard = super::test_sync::global_state_guard();
        let last_hwnd = AtomicIsize::new(0);

        let event = testing::inject_foreground_window_event(&last_hwnd, 7, |_| None);

        assert_eq!(
            event,
            Some(crate::hook::HookEvent::Reset(ResetCause::ForegroundChange))
        );
        assert!(!super::is_denylisted());
    }
}
