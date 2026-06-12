use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use openmacro_lib::{
    commands::logging::{get_log_ring_inner, get_logging_frontend_cfg_inner},
    log_body,
    log_init::{init, ring::LogEntry, ring::RingBuffer, LogHandles},
    store::{LoggingConfig, LoggingFileConfig, LoggingFrontendConfig},
};
use serde_json::json;
use tempfile::TempDir;
use tracing::Level;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn handles() -> LogHandles {
    let (_writer, guard) = tracing_appender::non_blocking(std::io::sink());
    LogHandles {
        _file_guard: guard,
        ring: Arc::new(RingBuffer::new(2000)),
    }
}

fn entry(message: &str) -> LogEntry {
    LogEntry {
        seq: 0,
        ts_unix_ms: 0,
        level: Level::INFO,
        target: "commands_logging".to_string(),
        message: message.to_string(),
        fields: json!({}),
        span_path: Vec::new(),
    }
}

#[test]
fn get_log_ring_returns_all_then_delta_entries() {
    let handles = handles();
    handles.ring.push(entry("first"));
    handles.ring.push(entry("second"));

    let all = get_log_ring_inner(&handles, 0);

    assert_eq!(all.len(), 2);
    assert!(all[0].seq < all[1].seq);
    assert_eq!(all[0].message, "first");
    assert_eq!(all[1].message, "second");

    let last_seq = all.last().unwrap().seq;
    handles.ring.push(entry("third"));

    let delta = get_log_ring_inner(&handles, last_seq);

    assert_eq!(delta.len(), 1);
    assert!(delta[0].seq > last_seq);
    assert_eq!(delta[0].message, "third");
}

#[test]
fn get_logging_frontend_cfg_returns_effective_config_frontend_shape() {
    let mut modules = HashMap::new();
    modules.insert("settings".to_string(), "debug".to_string());
    let cfg = LoggingConfig {
        level: "warn".to_string(),
        modules: HashMap::new(),
        file: LoggingFileConfig::default(),
        verbose_content: true,
        frontend: LoggingFrontendConfig {
            level: "trace".to_string(),
            modules,
        },
    };

    let frontend = get_logging_frontend_cfg_inner(&cfg);

    assert_eq!(frontend.level, "trace");
    assert_eq!(frontend.modules["settings"], "debug");
}

#[test]
fn init_writes_structured_json_file_with_redacted_body() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old_log_dir = std::env::var_os("OPENMACRO_LOG_DIR");
    let old_rust_log = std::env::var_os("RUST_LOG");
    let temp = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_LOG_DIR", temp.path());
    std::env::set_var("RUST_LOG", "info");

    let handles = init(&LoggingConfig {
        file: LoggingFileConfig {
            enabled: true,
            max_files: 7,
        },
        ..LoggingConfig::default()
    });
    tracing::info!(body = %log_body!("secret body"), "json file redaction test");
    drop(handles);

    let contents = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("openmacro.log.")
        })
        .map(|entry| fs::read_to_string(entry.path()).unwrap())
        .expect("expected log file");

    assert!(contents.contains(r#""target""#));
    assert!(contents.contains(r#""level""#));
    assert!(contents.contains(r#""body":"<redacted len=11>""#));

    match old_log_dir {
        Some(value) => std::env::set_var("OPENMACRO_LOG_DIR", value),
        None => std::env::remove_var("OPENMACRO_LOG_DIR"),
    }
    match old_rust_log {
        Some(value) => std::env::set_var("RUST_LOG", value),
        None => std::env::remove_var("RUST_LOG"),
    }
}
