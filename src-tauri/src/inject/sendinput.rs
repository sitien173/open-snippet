//! SendInput keyboard sink.

use super::{KeyboardAction, KeyboardSink};

#[derive(Default)]
pub struct WindowsKeyboardSink;

impl KeyboardSink for WindowsKeyboardSink {
    fn send(&mut self, action: KeyboardAction) {
        tracing::debug!(
            action = keyboard_action_label(&action),
            "sending keyboard action"
        );
        #[cfg(windows)]
        {
            send_windows_action(action);
        }

        #[cfg(not(windows))]
        {
            let _ = action;
        }
    }

    fn send_batch(&mut self, actions: &[KeyboardAction]) {
        tracing::debug!(count = actions.len(), "sending keyboard action batch");
        #[cfg(windows)]
        {
            send_windows_actions(actions);
        }

        #[cfg(not(windows))]
        {
            let _ = actions;
        }
    }
}

fn keyboard_action_label(action: &KeyboardAction) -> &'static str {
    match action {
        KeyboardAction::Backspace => "backspace",
        KeyboardAction::LeftArrow => "left_arrow",
        KeyboardAction::Unicode(_) => "unicode",
        KeyboardAction::Paste(_) => "paste",
    }
}

#[cfg(windows)]
fn send_windows_action(action: KeyboardAction) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_LEFT,
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

#[cfg(windows)]
fn send_windows_actions(actions: &[KeyboardAction]) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_LEFT,
    };

    const MAX_INPUTS_PER_CALL: usize = 256;

    fn send_inputs(inputs: &mut [INPUT]) {
        unsafe {
            // SAFETY: buffer remains valid for the duration of each SendInput call.
            let _ = SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn push_key(inputs: &mut Vec<INPUT>, vk: VIRTUAL_KEY) {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    ..Default::default()
                },
            },
        });
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    dwFlags: KEYEVENTF_KEYUP,
                    ..Default::default()
                },
            },
        });
    }

    fn push_unicode(inputs: &mut Vec<INPUT>, ch: char) {
        let mut utf16 = [0u16; 2];
        for unit in ch.encode_utf16(&mut utf16).iter().copied() {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wScan: unit,
                        dwFlags: KEYEVENTF_UNICODE,
                        ..Default::default()
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wScan: unit,
                        dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0),
                        ..Default::default()
                    },
                },
            });
        }
    }

    let mut inputs = Vec::with_capacity(actions.len() * 2);
    for action in actions {
        match action {
            KeyboardAction::Backspace => push_key(&mut inputs, VIRTUAL_KEY(VK_BACK.0)),
            KeyboardAction::LeftArrow => push_key(&mut inputs, VIRTUAL_KEY(VK_LEFT.0)),
            KeyboardAction::Unicode(ch) => push_unicode(&mut inputs, *ch),
            KeyboardAction::Paste(_) => {
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_CONTROL.0),
                            ..Default::default()
                        },
                    },
                });
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(b'V' as u16),
                            ..Default::default()
                        },
                    },
                });
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(b'V' as u16),
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                });
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(VK_CONTROL.0),
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                });
            }
        }
    }

    for chunk in inputs.chunks_mut(MAX_INPUTS_PER_CALL) {
        send_inputs(chunk);
    }
}
