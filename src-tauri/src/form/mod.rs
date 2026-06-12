//! Interactive form flow for snippet variables.

mod focus;
mod runner;

pub use focus::{
    capture_foreground, capture_foreground_with, restore_foreground, restore_foreground_with,
    FocusBackend, FocusError, ForegroundWindow, NoopFocusBackend, SystemFocusBackend,
};
pub use runner::{
    restore_on_submit, FormError, FormOutcome, FormRunner, NoopWindowSink, WindowSink,
};
