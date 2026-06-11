//! Win32 low-level keyboard hook thread.

use std::{
    fmt,
    panic::{self, AssertUnwindSafe},
    ptr,
    sync::mpsc,
    thread::{self, JoinHandle},
};

use super::{channel, HookConsumer, HookEvent, HookProducer, ResetCause, RING_CAPACITY};

pub struct Hook;

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TranslateOutcome {
    Char(char),
    DeadKey,
    None,
}

#[cfg(windows)]
struct HklHex(windows::Win32::UI::Input::KeyboardAndMouse::HKL);

#[cfg(windows)]
impl fmt::Display for HklHex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0 .0 as usize)
    }
}

#[cfg(windows)]
struct Codepoint(char);

#[cfg(windows)]
impl fmt::Display for Codepoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\\u{{{:04x}}}", self.0 as u32)
    }
}

pub struct HookHandle {
    join_handle: Option<JoinHandle<()>>,
    thread_id: Option<u32>,
}

struct HookReady {
    thread_id: Option<u32>,
    result: Result<(), String>,
}

#[cfg(windows)]
pub(super) fn translate_with_layout(
    vk_code: u32,
    scan_code: u32,
    hkl: windows::Win32::UI::Input::KeyboardAndMouse::HKL,
    key_state: &[u8; 256],
) -> TranslateOutcome {
    use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;

    if hkl.0.is_null() {
        return TranslateOutcome::None;
    }

    let mut utf16 = [0u16; 8];
    let translated = unsafe { ToUnicodeEx(vk_code, scan_code, key_state, &mut utf16, 2, hkl) };

    match translated {
        1 => char::from_u32(utf16[0] as u32)
            .map(TranslateOutcome::Char)
            .unwrap_or(TranslateOutcome::None),
        -1 => TranslateOutcome::DeadKey,
        0 => TranslateOutcome::None,
        _ => TranslateOutcome::None,
    }
}

impl Hook {
    #[tracing::instrument]
    pub fn start() -> Result<(HookHandle, HookConsumer), String> {
        tracing::info!("starting keyboard hook");
        let (mut producer, consumer) = channel(RING_CAPACITY);
        let (ready_tx, ready_rx) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                #[cfg(windows)]
                {
                    use windows::Win32::System::Threading::GetCurrentThreadId;

                    let thread_id = unsafe { GetCurrentThreadId() };
                    let result = run_hook_thread(&mut producer, |result| {
                        let _ = ready_tx.send(HookReady {
                            thread_id: Some(thread_id),
                            result,
                        });
                    });
                    if let Err(error) = result {
                        tracing::error!(error = %error, "hook thread exited with error");
                    }
                }

                #[cfg(not(windows))]
                {
                    let _ = producer.push(HookEvent::Reset(ResetCause::ImeOrComposition));
                    let _ = ready_tx.send(HookReady {
                        thread_id: None,
                        result: Ok(()),
                    });
                }
            }));

            if let Err(payload) = result {
                let _ = crate::crash::write_caught_panic_dump("hook thread", payload.as_ref());
                tracing::error!("hook thread panicked");
                let _ = ready_tx.send(HookReady {
                    thread_id: None,
                    result: Err("hook thread panicked".to_string()),
                });
            }
        });

        let ready = ready_rx.recv().map_err(|error| error.to_string())?;
        match ready.result {
            Ok(()) => {
                tracing::info!("keyboard hook started");
                Ok((
                    HookHandle {
                        join_handle: Some(join_handle),
                        thread_id: ready.thread_id,
                    },
                    consumer,
                ))
            }
            Err(error) => {
                tracing::error!(error = %error, "keyboard hook failed to start");
                let _ = join_handle.join();
                Err(error)
            }
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::{translate_with_layout, TranslateOutcome};
    use windows::{
        core::w,
        Win32::UI::Input::KeyboardAndMouse::{
            LoadKeyboardLayoutW, KLF_NOTELLSHELL, VK_A, VK_SHIFT,
        },
    };

    #[test]
    fn translate_with_layout_returns_ascii_lowercase_for_vk_a() {
        let Ok(hkl) = (unsafe { LoadKeyboardLayoutW(w!("00000409"), KLF_NOTELLSHELL) }) else {
            return;
        };

        let key_state = [0u8; 256];
        let outcome = translate_with_layout(VK_A.0 as u32, 0, hkl, &key_state);

        assert_eq!(outcome, TranslateOutcome::Char('a'));
    }

    #[test]
    fn translate_with_layout_returns_ascii_uppercase_for_shift_vk_a() {
        let Ok(hkl) = (unsafe { LoadKeyboardLayoutW(w!("00000409"), KLF_NOTELLSHELL) }) else {
            return;
        };

        let mut key_state = [0u8; 256];
        key_state[VK_SHIFT.0 as usize] = 0x80;
        let outcome = translate_with_layout(VK_A.0 as u32, 0, hkl, &key_state);

        assert_eq!(outcome, TranslateOutcome::Char('A'));
    }

    #[test]
    fn translate_with_layout_handles_vk_return() {
        use windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN;
        let Ok(hkl) = (unsafe { LoadKeyboardLayoutW(w!("00000409"), KLF_NOTELLSHELL) }) else {
            return;
        };

        let key_state = [0u8; 256];
        let outcome = translate_with_layout(VK_RETURN.0 as u32, 0, hkl, &key_state);

        assert_eq!(outcome, TranslateOutcome::Char('\r'));
    }

    #[test]
    fn should_ignore_event_logic() {
        use super::{should_ignore_event, LLKHF_INJECTED_FLAG};
        use crate::inject::sendinput::INJECTED_MARKER;

        // 1. Marked injected events are ignored even when suspend is false
        assert!(
            should_ignore_event(false, LLKHF_INJECTED_FLAG, INJECTED_MARKER),
            "marked injected events should be ignored when suspend is false"
        );
        assert!(
            should_ignore_event(true, LLKHF_INJECTED_FLAG, INJECTED_MARKER),
            "marked injected events should be ignored when suspend is true"
        );

        // 2. Unmarked injected events are not ignored solely due to LLKHF_INJECTED_FLAG
        assert!(
            !should_ignore_event(false, LLKHF_INJECTED_FLAG, 0),
            "unmarked injected events should not be ignored when suspend is false"
        );
        assert!(
            !should_ignore_event(true, LLKHF_INJECTED_FLAG, 0),
            "unmarked injected events should not be ignored when suspend is true"
        );

        // 3. Non-injected events are not ignored
        assert!(
            !should_ignore_event(false, 0, INJECTED_MARKER),
            "non-injected events with marker should not be ignored"
        );
        assert!(
            !should_ignore_event(false, 0, 0),
            "non-injected events without marker should not be ignored"
        );
    }
}

impl Drop for HookHandle {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(thread_id) = self.thread_id {
            unsafe {
                use windows::Win32::{
                    Foundation::{LPARAM, WPARAM},
                    UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT},
                };

                let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

const LLKHF_INJECTED_FLAG: u32 = 0x10;

fn should_ignore_event(_suspend: bool, flags: u32, dw_extra_info: usize) -> bool {
    (flags & LLKHF_INJECTED_FLAG) != 0 && dw_extra_info == crate::inject::sendinput::INJECTED_MARKER
}

#[cfg(windows)]
fn run_hook_thread(
    producer: &mut HookProducer,
    signal_ready: impl FnOnce(Result<(), String>),
) -> windows::core::Result<()> {
    use std::sync::atomic::{AtomicPtr, Ordering};

    use windows::Win32::{
        Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyState, GetKeyboardLayout, GetKeyboardState, VK_BACK, VK_CAPITAL, VK_LCONTROL,
                VK_LMENU, VK_LSHIFT, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW,
                GetWindowThreadProcessId, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
                HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
            },
        },
    };

    static HOOK_PRODUCER: AtomicPtr<HookProducer> = AtomicPtr::new(ptr::null_mut());

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            let message = wparam.0 as u32;
            if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                // SAFETY: lparam is provided by the OS for WH_KEYBOARD_LL callbacks.
                let keyboard = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
                if should_ignore_event(
                    crate::inject::SUSPEND.load(Ordering::Relaxed),
                    keyboard.flags.0,
                    keyboard.dwExtraInfo,
                ) {
                    return unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) };
                }
                let producer_ptr = HOOK_PRODUCER.load(Ordering::Relaxed);
                if !producer_ptr.is_null() {
                    // SAFETY: callback thread stores a valid HookProducer pointer before hook install and clears it on teardown.
                    let producer = unsafe { &mut *producer_ptr };
                    if keyboard.vkCode == VK_BACK.0 as u32 {
                        let _ = producer.push(HookEvent::Backspace);
                    } else if keyboard.vkCode == VK_CAPITAL.0 as u32 {
                        let _ = producer.push(HookEvent::Reset(ResetCause::CapsToggle));
                    } else if let Some(ch) = translate_key(keyboard.vkCode, keyboard.scanCode) {
                        let _ = producer.push(HookEvent::Char(ch));
                    }
                }
            }
        }

        // SAFETY: forwarding hook events to the next hook is required by the hook contract.
        unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
    }

    fn translate_key(vk_code: u32, scan_code: u32) -> Option<char> {
        unsafe {
            let mut key_state = [0u8; 256];
            if GetKeyboardState(&mut key_state).is_err() {
                return None;
            }

            key_state[VK_LSHIFT.0 as usize] = (GetKeyState(VK_LSHIFT.0 as i32) as u8) & 0x80;
            key_state[VK_RSHIFT.0 as usize] = (GetKeyState(VK_RSHIFT.0 as i32) as u8) & 0x80;
            key_state[VK_LCONTROL.0 as usize] = (GetKeyState(VK_LCONTROL.0 as i32) as u8) & 0x80;
            key_state[VK_RCONTROL.0 as usize] = (GetKeyState(VK_RCONTROL.0 as i32) as u8) & 0x80;
            key_state[VK_LMENU.0 as usize] = (GetKeyState(VK_LMENU.0 as i32) as u8) & 0x80;
            key_state[VK_RMENU.0 as usize] = (GetKeyState(VK_RMENU.0 as i32) as u8) & 0x80;

            let hwnd = GetForegroundWindow();
            let hkl = if hwnd.0.is_null() {
                GetKeyboardLayout(0)
            } else {
                let thread_id = GetWindowThreadProcessId(hwnd, None);
                if thread_id == 0 {
                    GetKeyboardLayout(0)
                } else {
                    GetKeyboardLayout(thread_id)
                }
            };

            let outcome = translate_with_layout(vk_code, scan_code, hkl, &key_state);
            match outcome {
                TranslateOutcome::Char(ch) => {
                    tracing::trace!(
                        vk_code,
                        scan_code,
                        hkl = %HklHex(hkl),
                        outcome = "char",
                        codepoint = %Codepoint(ch),
                        "translated key"
                    );
                    Some(ch)
                }
                TranslateOutcome::DeadKey => {
                    tracing::trace!(
                        vk_code,
                        scan_code,
                        hkl = %HklHex(hkl),
                        outcome = "dead",
                        "translated key"
                    );
                    None
                }
                TranslateOutcome::None => {
                    tracing::trace!(
                        vk_code,
                        scan_code,
                        hkl = %HklHex(hkl),
                        outcome = "none",
                        "translated key"
                    );
                    None
                }
            }
        }
    }

    HOOK_PRODUCER.store(producer as *mut HookProducer, Ordering::Relaxed);

    // SAFETY: installing a WH_KEYBOARD_LL hook with a static callback is required to receive events.
    let hook = match unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE::default(), 0)
    } {
        Ok(hook) => {
            signal_ready(Ok(()));
            hook
        }
        Err(error) => {
            signal_ready(Err(error.to_string()));
            return Err(error);
        }
    };

    let mut message = MSG::default();
    // SAFETY: standard message pump for the hook thread.
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.into() {
        // SAFETY: forwarding messages through the thread pump is standard Win32 behavior.
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    // SAFETY: hook handle was returned by SetWindowsHookExW and remains valid until thread teardown.
    unsafe {
        UnhookWindowsHookEx(hook)?;
    }
    HOOK_PRODUCER.store(ptr::null_mut(), Ordering::Relaxed);
    Ok(())
}
