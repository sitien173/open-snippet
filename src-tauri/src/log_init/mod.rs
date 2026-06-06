use std::{fs, path::PathBuf, sync::Arc};

pub mod redact;
pub mod ring;
pub mod rotation;

pub use redact::{set_verbose_content, verbose_content};

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::store::LoggingConfig;

use self::ring::{RingBuffer, RingLayer};

pub struct LogHandles {
    pub _file_guard: tracing_appender::non_blocking::WorkerGuard,
    pub ring: Arc<RingBuffer>,
}

pub fn init(cfg: &LoggingConfig) -> LogHandles {
    set_verbose_content(cfg.verbose_content);

    let ring = Arc::new(RingBuffer::new(2000));
    let filter = build_env_filter(cfg);
    let stdout_layer = fmt::layer().pretty().with_target(true).with_ansi(true);

    let log_dir = log_dir();
    let file_enabled = cfg.file.enabled && fs::create_dir_all(&log_dir).is_ok();
    if file_enabled {
        let _ = rotation::prune_old_logs(cfg.file.max_files);
    }

    let (file_writer, file_guard) = if file_enabled {
        let file_appender = tracing_appender::rolling::daily(log_dir, "openmacro.log");
        tracing_appender::non_blocking(file_appender)
    } else {
        tracing_appender::non_blocking(std::io::sink())
    };

    let file_layer = if file_enabled {
        Some(
            fmt::layer()
                .json()
                .with_writer(file_writer)
                .with_current_span(true),
        )
    } else {
        None
    };

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .with(RingLayer::new(Arc::clone(&ring)));
    let _ = subscriber.try_init();

    LogHandles {
        _file_guard: file_guard,
        ring,
    }
}

pub fn build_env_filter(cfg: &LoggingConfig) -> EnvFilter {
    let directive = build_env_filter_directive(cfg);
    EnvFilter::try_new(directive)
        .unwrap_or_else(|_| EnvFilter::try_new("info").expect("default filter is valid"))
}

pub fn build_env_filter_directive(cfg: &LoggingConfig) -> String {
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.is_empty() {
            return rust_log;
        }
    }

    let mut directives = vec![cfg.level.clone()];
    let mut modules = cfg.modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| left.0.cmp(right.0));
    directives.extend(
        modules
            .into_iter()
            .map(|(target, level)| format!("{target}={level}")),
    );
    directives.join(",")
}

pub fn log_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("OPENMACRO_LOG_DIR") {
        return PathBuf::from(dir);
    }

    dirs::config_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("openmacro")
        .join("logs")
}
