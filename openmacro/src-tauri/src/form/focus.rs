use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ForegroundWindow(pub isize);

unsafe impl Send for ForegroundWindow {}
unsafe impl Sync for ForegroundWindow {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusError {
    Refused,
}

impl fmt::Display for FocusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused => f.write_str("foreground restore refused"),
        }
    }
}

impl std::error::Error for FocusError {}

pub trait FocusBackend: Send + Sync + 'static {
    fn capture_foreground(&self) -> Option<ForegroundWindow>;
    fn restore_foreground(&self, hwnd: ForegroundWindow) -> Result<(), FocusError>;
}

#[derive(Default)]
pub struct SystemFocusBackend;

#[derive(Default)]
pub struct NoopFocusBackend;

impl FocusBackend for NoopFocusBackend {
    fn capture_foreground(&self) -> Option<ForegroundWindow> {
        None
    }

    fn restore_foreground(&self, _hwnd: ForegroundWindow) -> Result<(), FocusError> {
        Ok(())
    }
}

impl FocusBackend for SystemFocusBackend {
    fn capture_foreground(&self) -> Option<ForegroundWindow> {
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

            let hwnd = unsafe {
                // SAFETY: reading the current foreground window handle has no aliasing requirements.
                GetForegroundWindow()
            };
            if hwnd.0.is_null() {
                None
            } else {
                Some(ForegroundWindow(hwnd.0 as isize))
            }
        }

        #[cfg(not(windows))]
        {
            None
        }
    }

    fn restore_foreground(&self, hwnd: ForegroundWindow) -> Result<(), FocusError> {
        #[cfg(windows)]
        {
            restore_foreground_impl(hwnd)
        }

        #[cfg(not(windows))]
        {
            let _ = hwnd;
            Err(FocusError::Refused)
        }
    }
}

pub fn capture_foreground() -> Option<ForegroundWindow> {
    capture_foreground_with(&SystemFocusBackend)
}

pub fn capture_foreground_with(backend: &impl FocusBackend) -> Option<ForegroundWindow> {
    backend.capture_foreground()
}

pub fn restore_foreground(hwnd: ForegroundWindow) -> Result<(), FocusError> {
    restore_foreground_with(&SystemFocusBackend, hwnd)
}

pub fn restore_foreground_with(
    backend: &impl FocusBackend,
    hwnd: ForegroundWindow,
) -> Result<(), FocusError> {
    backend.restore_foreground(hwnd)
}

#[cfg(windows)]
fn restore_foreground_impl(hwnd: ForegroundWindow) -> Result<(), FocusError> {
    use windows::Win32::{
        Foundation::{BOOL, HWND},
        System::Threading::{AttachThreadInput, GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            AllowSetForegroundWindow, GetForegroundWindow, GetWindowThreadProcessId,
            SetForegroundWindow, ASFW_ANY,
        },
    };

    unsafe {
        // SAFETY: grants foreground-set permission to any process before our focused handoff attempt.
        let _ = AllowSetForegroundWindow(ASFW_ANY);
    }

    let target = HWND(hwnd.0 as *mut core::ffi::c_void);
    let direct = unsafe {
        // SAFETY: target HWND is an opaque handle captured from the OS.
        SetForegroundWindow(target)
    };
    if direct.as_bool() {
        return Ok(());
    }

    let current_hwnd = unsafe {
        // SAFETY: reading the current foreground window handle has no aliasing requirements.
        GetForegroundWindow()
    };
    let target_thread = unsafe {
        // SAFETY: querying the owning thread of a HWND is safe for opaque OS handles.
        GetWindowThreadProcessId(target, None)
    };
    let foreground_thread = unsafe {
        // SAFETY: querying the owning thread of a HWND is safe for opaque OS handles.
        GetWindowThreadProcessId(current_hwnd, None)
    };
    let self_thread = unsafe {
        // SAFETY: returns the current thread id.
        GetCurrentThreadId()
    };

    let mut attached_target = false;
    let mut attached_foreground = false;
    if target_thread != 0 && target_thread != self_thread {
        attached_target = unsafe {
            // SAFETY: temporary thread input attachment for foreground activation retry.
            AttachThreadInput(self_thread, target_thread, true)
        }
        .as_bool();
    }
    if foreground_thread != 0 && foreground_thread != self_thread && foreground_thread != target_thread {
        attached_foreground = unsafe {
            // SAFETY: temporary thread input attachment for foreground activation retry.
            AttachThreadInput(self_thread, foreground_thread, true)
        }
        .as_bool();
    }

    let attached_result = unsafe {
        // SAFETY: target HWND is the opaque window we are attempting to refocus.
        SetForegroundWindow(target)
    };

    if attached_foreground {
        unsafe {
            // SAFETY: detaches the earlier successful thread input attachment.
            let _ = AttachThreadInput(self_thread, foreground_thread, false);
        }
    }
    if attached_target {
        unsafe {
            // SAFETY: detaches the earlier successful thread input attachment.
            let _ = AttachThreadInput(self_thread, target_thread, false);
        }
    }

    if attached_result == BOOL(1) {
        Ok(())
    } else {
        Err(FocusError::Refused)
    }
}
