//! Win32 low-level keyboard hook thread.

use std::{
    panic::{self, AssertUnwindSafe},
    ptr,
    sync::mpsc,
    thread::{self, JoinHandle},
};

use super::{channel, HookConsumer, HookEvent, HookProducer, ResetCause, RING_CAPACITY};

pub struct Hook;

pub struct HookHandle {
    join_handle: Option<JoinHandle<()>>,
    thread_id: Option<u32>,
}

struct HookReady {
    thread_id: Option<u32>,
    result: Result<(), String>,
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
                GetKeyState, GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, VK_BACK, VK_CAPITAL,
                VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
                UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
                WM_KEYDOWN, WM_SYSKEYDOWN,
            },
        },
    };

    static HOOK_PRODUCER: AtomicPtr<HookProducer> = AtomicPtr::new(ptr::null_mut());
    const LLKHF_INJECTED_FLAG: u32 = 0x10;

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            let message = wparam.0 as u32;
            if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                // SAFETY: lparam is provided by the OS for WH_KEYBOARD_LL callbacks.
                let keyboard = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
                if crate::inject::SUSPEND.load(Ordering::Relaxed)
                    && (keyboard.flags.0 & LLKHF_INJECTED_FLAG) != 0
                {
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

            let mut utf16 = [0u16; 8];
            // SAFETY: fixed buffers are stack-allocated and valid for the duration of the call.
            let translated = ToUnicodeEx(
                vk_code,
                scan_code,
                &key_state,
                &mut utf16,
                0,
                GetKeyboardLayout(0),
            );

            if translated == 1 {
                char::from_u32(utf16[0] as u32)
            } else {
                None
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
