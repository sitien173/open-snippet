//! Snippet expansion resolution pipeline.

mod clipboard_var;
mod cursor;
mod datetime;
mod resolver;
pub mod shell;

pub use clipboard_var::ClipboardReader;
pub use cursor::{strip_cursor_token, CursorTokenError};
pub use resolver::{ResolveError, ResolveNotifySink, Resolved, Resolver};
