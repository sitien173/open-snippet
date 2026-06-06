use std::sync::Mutex;

use openmacro_lib::{
    log_body,
    log_init::{
        redact::{redact_str, FieldKind},
        set_verbose_content, verbose_content,
    },
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_verbose_env<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().unwrap();
    let old = std::env::var_os("OPENMACRO_LOG_VERBOSE");
    match value {
        Some(value) => std::env::set_var("OPENMACRO_LOG_VERBOSE", value),
        None => std::env::remove_var("OPENMACRO_LOG_VERBOSE"),
    }

    let result = f();

    match old {
        Some(old) => std::env::set_var("OPENMACRO_LOG_VERBOSE", old),
        None => std::env::remove_var("OPENMACRO_LOG_VERBOSE"),
    }
    result
}

#[test]
fn redacts_sensitive_field_kinds_by_default() {
    with_verbose_env(None, || {
        set_verbose_content(false);

        assert_eq!(
            redact_str("hello", FieldKind::SnippetBody),
            "<redacted len=5>"
        );
        assert_eq!(
            redact_str("hello", FieldKind::ClipboardText),
            "<redacted len=5>"
        );
        assert_eq!(
            redact_str("hello", FieldKind::FormValue),
            "<redacted len=5>"
        );
        assert_eq!(redact_str("hello", FieldKind::Credential), "<redacted>");
        assert_eq!(redact_str("hello", FieldKind::Token), "<redacted>");
    });
}

#[test]
fn preserves_safe_field_kinds() {
    with_verbose_env(None, || {
        set_verbose_content(false);

        assert_eq!(redact_str(";sig", FieldKind::TriggerName), ";sig");
        assert_eq!(
            redact_str("default.yaml::;sig", FieldKind::SnippetId),
            "default.yaml::;sig"
        );
        assert_eq!(
            redact_str(r"C:\tmp\file.txt", FieldKind::Path),
            r"C:\tmp\file.txt"
        );
    });
}

#[test]
fn redaction_length_counts_unicode_code_points() {
    with_verbose_env(None, || {
        set_verbose_content(false);

        assert_eq!(
            redact_str("aé💙", FieldKind::SnippetBody),
            "<redacted len=3>"
        );
    });
}

#[test]
fn verbose_env_wins_over_config() {
    with_verbose_env(Some("1"), || {
        set_verbose_content(false);

        assert!(verbose_content());
        assert_eq!(
            redact_str("secret body", FieldKind::SnippetBody),
            "secret body"
        );
        assert_eq!(redact_str("token", FieldKind::Token), "token");
    });
}

#[test]
fn config_verbose_unredacts_when_env_is_unset() {
    with_verbose_env(None, || {
        set_verbose_content(true);

        assert!(verbose_content());
        assert_eq!(
            redact_str("secret body", FieldKind::FormValue),
            "secret body"
        );
    });
}

#[test]
fn log_body_macro_uses_snippet_body_redaction() {
    with_verbose_env(None, || {
        set_verbose_content(false);
        assert_eq!(log_body!("abc"), "<redacted len=3>");
    });

    with_verbose_env(Some("1"), || {
        set_verbose_content(false);
        assert_eq!(log_body!("abc"), "abc");
    });
}
