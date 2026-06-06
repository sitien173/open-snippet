//! SendInput keyboard sink.

use super::{KeyboardAction, KeyboardSink};

#[derive(Default)]
pub struct WindowsKeyboardSink;

impl KeyboardSink for WindowsKeyboardSink {
    fn send(&mut self, action: KeyboardAction) {
        #[cfg(windows)]
        {
            send_windows_action(action);
        }

        #[cfg(not(windows))]
        {
            let _ = action;
        }
    }
}

#[cfg(windows)]
fn send_windows_action(action: KeyboardAction) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK, VK_CONTROL, VK_LEFT, VIRTUAL_KEY,
    };

    fn send_inputs(inputs: &mut [INPUT]) {
        unsafe {
            // SAFETY: buffer is stack-local and valid for the duration of the SendInput call.
            let _ = SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    match action {
        KeyboardAction::Backspace => {
            let mut inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_BACK.0),
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_BACK.0),
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
            ];
            send_inputs(&mut inputs);
        }
        KeyboardAction::LeftArrow => {
            let mut inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_LEFT.0),
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_LEFT.0),
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
            ];
            send_inputs(&mut inputs);
        }
        KeyboardAction::Unicode(ch) => {
            let mut utf16 = [0u16; 2];
            for unit in ch.encode_utf16(&mut utf16).iter().copied() {
                let mut inputs = [
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wScan: unit,
                                dwFlags: KEYEVENTF_UNICODE,
                                ..Default::default()
                            },
                        },
                    },
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wScan: unit,
                                dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0),
                                ..Default::default()
                            },
                        },
                    },
                ];
                send_inputs(&mut inputs);
            }
        }
        KeyboardAction::Paste(_) => {
            let mut inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_CONTROL.0),
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(b'V' as u16),
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(b'V' as u16),
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_CONTROL.0),
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
            ];
            send_inputs(&mut inputs);
        }
    }
}
