//! Win32 low-level keyboard hook thread.

use std::{
    ptr,
    sync::mpsc,
    thread::{self, JoinHandle},
};

use super::{channel, HookConsumer, HookEvent, HookProducer, ResetCause, RING_CAPACITY};

pub struct Hook;

pub struct HookHandle {
    join_handle: Option<JoinHandle<()>>,
}

impl Hook {
    pub fn start() -> Result<(HookHandle, HookConsumer), String> {
        let (mut producer, consumer) = channel(RING_CAPACITY);
        let (ready_tx, ready_rx) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            #[cfg(windows)]
            {
                let result = run_hook_thread(&mut producer);
                let _ = ready_tx.send(result.map_err(|error| error.to_string()));
            }

            #[cfg(not(windows))]
            {
                let _ = producer.push(HookEvent::Reset(ResetCause::ImeOrComposition));
                let _ = ready_tx.send(Ok(()));
            }
        });

        match ready_rx.recv().map_err(|error| error.to_string())? {
            Ok(()) => Ok((
                HookHandle {
                    join_handle: Some(join_handle),
                },
                consumer,
            )),
            Err(error) => {
                let _ = join_handle.join();
                Err(error)
            }
        }
    }
}

impl Drop for HookHandle {
    fn drop(&mut self) {
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

#[cfg(windows)]
fn run_hook_thread(producer: &mut HookProducer) -> windows::core::Result<()> {
    use std::sync::atomic::{AtomicPtr, Ordering};

    use windows::Win32::{
        Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyboardLayout, GetKeyState, GetKeyboardState, ToUnicodeEx, VK_BACK,
                VK_CAPITAL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
                MSG, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL,
                WM_KEYDOWN, WM_SYSKEYDOWN,
            },
        },
    };

    static HOOK_PRODUCER: AtomicPtr<HookProducer> = AtomicPtr::new(ptr::null_mut());

    unsafe extern "system" fn keyboard_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code == HC_ACTION as i32 {
            let message = wparam.0 as u32;
            if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                // SAFETY: lparam is provided by the OS for WH_KEYBOARD_LL callbacks.
                let keyboard = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
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
    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE::default(), 0)? };

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
