use std::{collections::HashMap, sync::Mutex};

use openmacro_lib::{
    log_init::{build_env_filter_directive, init, verbose_content},
    store::{LoggingConfig, LoggingFileConfig, LoggingFrontendConfig},
};
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env<T>(rust_log: Option<&str>, verbose: Option<&str>, f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().unwrap();
    let old_rust_log = std::env::var_os("RUST_LOG");
    let old_verbose = std::env::var_os("OPENMACRO_LOG_VERBOSE");
    let old_log_dir = std::env::var_os("OPENMACRO_LOG_DIR");

    match rust_log {
        Some(value) => std::env::set_var("RUST_LOG", value),
        None => std::env::remove_var("RUST_LOG"),
    }
    match verbose {
        Some(value) => std::env::set_var("OPENMACRO_LOG_VERBOSE", value),
        None => std::env::remove_var("OPENMACRO_LOG_VERBOSE"),
    }

    let result = f();

    match old_rust_log {
        Some(value) => std::env::set_var("RUST_LOG", value),
        None => std::env::remove_var("RUST_LOG"),
    }
    match old_verbose {
        Some(value) => std::env::set_var("OPENMACRO_LOG_VERBOSE", value),
        None => std::env::remove_var("OPENMACRO_LOG_VERBOSE"),
    }
    match old_log_dir {
        Some(value) => std::env::set_var("OPENMACRO_LOG_DIR", value),
        None => std::env::remove_var("OPENMACRO_LOG_DIR"),
    }

    result
}

#[test]
fn default_config_builds_info_filter() {
    with_env(None, None, || {
        assert_eq!(
            build_env_filter_directive(&LoggingConfig::default()),
            "info"
        );
    });
}

#[test]
fn module_overrides_are_appended_to_config_filter() {
    with_env(None, None, || {
        let mut modules = HashMap::new();
        modules.insert("openmacro::matcher".to_string(), "debug".to_string());
        modules.insert("openmacro::sync".to_string(), "trace".to_string());
        let cfg = LoggingConfig {
            level: "warn".to_string(),
            modules,
            file: LoggingFileConfig::default(),
            verbose_content: false,
            frontend: LoggingFrontendConfig::default(),
        };

        assert_eq!(
            build_env_filter_directive(&cfg),
            "warn,openmacro::matcher=debug,openmacro::sync=trace"
        );
    });
}

#[test]
fn rust_log_env_wins_over_config_filter() {
    with_env(Some("openmacro::matcher=trace,error"), None, || {
        let cfg = LoggingConfig {
            level: "warn".to_string(),
            ..LoggingConfig::default()
        };

        assert_eq!(
            build_env_filter_directive(&cfg),
            "openmacro::matcher=trace,error"
        );
    });
}

#[test]
fn init_sets_verbose_from_env_without_panicking() {
    let temp = TempDir::new().unwrap();
    with_env(None, Some("1"), || {
        std::env::set_var("OPENMACRO_LOG_DIR", temp.path());
        let cfg = LoggingConfig {
            file: LoggingFileConfig {
                enabled: false,
                max_files: 7,
            },
            verbose_content: false,
            ..LoggingConfig::default()
        };

        let handles = init(&cfg);

        assert!(handles.ring.is_empty());
        assert!(verbose_content());
    });
}
