//! Keyboard hook integration for input capture.

mod ring;
mod thread;
pub mod winevent;

pub use ring::{channel, HookConsumer, HookProducer, RING_CAPACITY};
pub use thread::{Hook, HookHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetCause {
    ImeOrComposition,
    CapsToggle,
    ForegroundChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    Char(char),
    Backspace,
    Reset(ResetCause),
}
