//! Snippet expansion resolution pipeline.

mod clipboard_var;
mod cursor;
mod datetime;
mod resolver;

pub use cursor::{strip_cursor_token, CursorTokenError};
pub use resolver::{ResolveError, Resolved, Resolver};
pub use clipboard_var::ClipboardReader;
