//! Snippet storage loading and persistence.

mod loader;
mod model;
pub mod watcher;

pub(crate) use loader::{effective_trigger, load_settings, settings_path, validate_trigger_prefix};
pub use loader::{load_from_root, LoadError, LoadResult};
pub use model::{
    ExpandMode, LoggingConfig, LoggingFileConfig, LoggingFrontendConfig, Snippet, StoreSettings,
    VarDecl, VarKind,
};
pub use watcher::{watch_root, SnapshotInner, Store};
