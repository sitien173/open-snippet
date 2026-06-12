use crate::{
    log_init::{ring::LogEntry, LogHandles},
    store::{LoggingConfig, LoggingFrontendConfig},
};

#[tauri::command]
#[tracing::instrument(skip(config))]
pub fn get_logging_frontend_cfg(config: tauri::State<'_, LoggingConfig>) -> LoggingFrontendConfig {
    get_logging_frontend_cfg_inner(config.inner())
}

#[tauri::command]
#[tracing::instrument(skip(handles))]
pub fn get_log_ring(handles: tauri::State<'_, LogHandles>, since_seq: u64) -> Vec<LogEntry> {
    get_log_ring_inner(handles.inner(), since_seq)
}

pub fn get_logging_frontend_cfg_inner(config: &LoggingConfig) -> LoggingFrontendConfig {
    tracing::debug!(
        level = %config.frontend.level,
        module_count = config.frontend.modules.len(),
        "read logging frontend config"
    );
    config.frontend.clone()
}

pub fn get_log_ring_inner(handles: &LogHandles, since_seq: u64) -> Vec<LogEntry> {
    let entries = handles.ring.slice_since(since_seq);
    tracing::debug!(since_seq, returned = entries.len(), "read log ring");
    entries
}
