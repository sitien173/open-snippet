use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    SnippetBody,
    ClipboardText,
    FormValue,
    Credential,
    Token,
    TriggerName,
    SnippetId,
    Path,
}

pub fn set_verbose_content(config_verbose: bool) {
    let verbose = std::env::var("OPENMACRO_LOG_VERBOSE")
        .map(|value| value == "1")
        .unwrap_or(config_verbose);
    verbose_flag().store(verbose, Ordering::Relaxed);
}

pub fn verbose_content() -> bool {
    verbose_flag().load(Ordering::Relaxed)
}

pub fn redact_str(s: &str, kind: FieldKind) -> String {
    if verbose_content() && is_sensitive(kind) {
        return s.to_string();
    }

    match kind {
        FieldKind::SnippetBody | FieldKind::ClipboardText | FieldKind::FormValue => {
            format!("<redacted len={}>", s.chars().count())
        }
        FieldKind::Credential | FieldKind::Token => "<redacted>".to_string(),
        FieldKind::TriggerName | FieldKind::SnippetId | FieldKind::Path => s.to_string(),
    }
}

fn verbose_flag() -> &'static AtomicBool {
    static VERBOSE_CONTENT: OnceLock<AtomicBool> = OnceLock::new();
    VERBOSE_CONTENT.get_or_init(|| AtomicBool::new(false))
}

fn is_sensitive(kind: FieldKind) -> bool {
    matches!(
        kind,
        FieldKind::SnippetBody
            | FieldKind::ClipboardText
            | FieldKind::FormValue
            | FieldKind::Credential
            | FieldKind::Token
    )
}

#[macro_export]
macro_rules! log_body {
    ($body:expr) => {{
        if $crate::log_init::verbose_content() {
            $body.to_string()
        } else {
            $crate::log_init::redact::redact_str(
                $body,
                $crate::log_init::redact::FieldKind::SnippetBody,
            )
        }
    }};
}
