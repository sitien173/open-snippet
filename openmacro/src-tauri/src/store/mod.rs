//! Snippet storage loading and persistence.

mod loader;
mod model;
pub mod watcher;

pub use loader::{load_from_root, LoadError, LoadResult};
pub use model::{Snippet, VarDecl, VarKind};
pub use watcher::{SnapshotInner, Store};
