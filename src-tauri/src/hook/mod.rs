//! Keyboard hook integration for input capture.

mod ring;
mod thread;
pub mod winevent;

pub use ring::{channel, HookConsumer, HookProducer, RING_CAPACITY};
pub use thread::{is_confirm_armed, set_confirm_armed, Hook, HookHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetCause {
    ArrowKey,
    Home,
    End,
    PageUp,
    PageDown,
    ImeOrComposition,
    CapsToggle,
    ForegroundChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKey {
    Tab,
    Enter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    Char(char),
    Confirm(ConfirmKey),
    Backspace,
    Reset(ResetCause),
}
