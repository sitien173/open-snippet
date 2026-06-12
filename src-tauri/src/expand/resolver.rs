use std::{collections::BTreeMap, path::Path, time::Duration};

use crate::{
    commands::prefs::Prefs,
    log_init::redact::{redact_str, FieldKind},
    store::{Snippet, VarKind},
};

use super::{
    clipboard_var::{resolve_clipboard, ClipboardReader},
    datetime::resolve_datetime,
    shell::{NoopShellBackend, ShellBackend, ShellError},
    strip_cursor_token,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub text: String,
    pub cursor_chars_after_token: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    UnknownPlaceholder { name: String },
    UnterminatedPlaceholder,
    ShellDisabled { name: String },
    ShellDeclined { name: String },
    ShellFailed { name: String, error: ShellError },
}

pub trait ResolveNotifySink: Send + Sync {
    fn unknown_placeholder(&self, snippet_id: &str, name: &str);

    fn confirm_shell(&self, _snippet_id: &str, _name: &str, _args: &[String]) -> bool {
        false
    }
}

#[derive(Default)]
pub struct NoopResolveNotifySink;

impl ResolveNotifySink for NoopResolveNotifySink {
    fn unknown_placeholder(&self, _snippet_id: &str, _name: &str) {}
}

static NOOP_SHELL_BACKEND: NoopShellBackend = NoopShellBackend;
static NOOP_NOTIFY_SINK: NoopResolveNotifySink = NoopResolveNotifySink;

pub struct Resolver<'a> {
    prefs: &'a Prefs,
    shell_backend: &'a dyn ShellBackend,
    notify: &'a dyn ResolveNotifySink,
}

impl<'a> Resolver<'a> {
    pub fn new(prefs: &'a Prefs) -> Self {
        Self {
            prefs,
            shell_backend: &NOOP_SHELL_BACKEND,
            notify: &NOOP_NOTIFY_SINK,
        }
    }

    pub fn with_shell_backend(mut self, shell_backend: &'a dyn ShellBackend) -> Self {
        self.shell_backend = shell_backend;
        self
    }

    pub fn with_notify_sink(mut self, notify: &'a dyn ResolveNotifySink) -> Self {
        self.notify = notify;
        self
    }

    #[tracing::instrument(skip(self, snippet, clipboard_reader, form_values), fields(snippet_id = %snippet.id, has_form_values = form_values.is_some()))]
    pub fn resolve(
        &self,
        snippet: &Snippet,
        clipboard_reader: &mut impl ClipboardReader,
        form_values: Option<&BTreeMap<String, String>>,
    ) -> Result<Resolved, ResolveError> {
        // SECURITY: snippet replacement bodies are user content; log only through log_body! redaction.
        tracing::debug!(
            snippet_id = %snippet.id,
            body = %crate::log_body!(&snippet.replace),
            has_form_values = form_values.is_some(),
            "resolving snippet"
        );
        let mut rendered = String::with_capacity(snippet.replace.len());
        let mut rest = snippet.replace.as_str();

        while let Some(start) = rest.find("{{") {
            rendered.push_str(&rest[..start]);
            let after_open = &rest[start + 2..];
            let Some(end) = after_open.find("}}") else {
                return Err(ResolveError::UnterminatedPlaceholder);
            };
            let token = &after_open[..end];
            let (name, arg) = split_placeholder(token);
            let value =
                self.resolve_placeholder(snippet, name, arg, clipboard_reader, form_values)?;
            rendered.push_str(&value);
            rest = &after_open[end + 2..];
        }

        rendered.push_str(rest);
        let (text, cursor_chars_after_token) =
            strip_cursor_token(&rendered).expect("loader validates cursor token count");

        tracing::debug!(
            snippet_id = %snippet.id,
            output_chars = text.chars().count(),
            cursor_chars_after_token,
            "snippet resolved"
        );

        Ok(Resolved {
            text,
            cursor_chars_after_token,
        })
    }

    #[tracing::instrument(skip(self, snippet, clipboard_reader, form_values), fields(snippet_id = %snippet.id, placeholder = %name))]
    fn resolve_placeholder(
        &self,
        snippet: &Snippet,
        name: &str,
        arg: Option<&str>,
        clipboard_reader: &mut impl ClipboardReader,
        form_values: Option<&BTreeMap<String, String>>,
    ) -> Result<String, ResolveError> {
        if let Some(values) = form_values {
            if let Some(value) = values.get(name) {
                // SECURITY: form values are user content and may contain secrets.
                tracing::debug!(
                    snippet_id = %snippet.id,
                    placeholder = %name,
                    value = %redact_str(value, FieldKind::FormValue),
                    "resolved form placeholder"
                );
                return Ok(value.clone());
            }
        }

        if let Some(var) = snippet.vars.iter().find(|var| var.name == name) {
            return Ok(match var.kind {
                VarKind::Text
                | VarKind::Textarea
                | VarKind::Choice
                | VarKind::Number
                | VarKind::Form => var.default.clone().unwrap_or_default(),
                VarKind::Datetime => {
                    resolve_datetime(arg.or(var.format.as_deref()), "%Y-%m-%d %H:%M:%S")
                }
                VarKind::Clipboard => resolve_clipboard(clipboard_reader),
                VarKind::Cursor => "$|$".to_string(),
                VarKind::Shell => self.resolve_shell_var(snippet, var)?,
            });
        }

        match name {
            "date" => Ok(resolve_datetime(arg, "%Y-%m-%d")),
            "time" => Ok(resolve_datetime(arg, "%H:%M:%S")),
            "datetime" => Ok(resolve_datetime(arg, "%Y-%m-%d %H:%M:%S")),
            "clipboard" => Ok(resolve_clipboard(clipboard_reader)),
            "cursor" => Ok("$|$".to_string()),
            _ => Err(ResolveError::UnknownPlaceholder {
                name: name.to_string(),
            }),
        }
    }

    #[tracing::instrument(skip(self, snippet, var), fields(snippet_id = %snippet.id, var = %var.name))]
    fn resolve_shell_var(
        &self,
        snippet: &Snippet,
        var: &crate::store::VarDecl,
    ) -> Result<String, ResolveError> {
        if !self.prefs.shell_consent {
            tracing::debug!(snippet_id = %snippet.id, var = %var.name, "shell variable disabled");
            return Err(ResolveError::ShellDisabled {
                name: var.name.clone(),
            });
        }
        if var.confirm && !self.notify.confirm_shell(&snippet.id, &var.name, &var.cmd) {
            tracing::debug!(snippet_id = %snippet.id, var = %var.name, "shell variable declined");
            return Err(ResolveError::ShellDeclined {
                name: var.name.clone(),
            });
        }

        let cwd = snippet
            .source_file
            .parent()
            .unwrap_or_else(|| Path::new("."));
        self.shell_backend
            .run(
                &var.cmd,
                cwd,
                Duration::from_millis(var.timeout_ms.unwrap_or_default()),
            )
            .map_err(|error| ResolveError::ShellFailed {
                name: var.name.clone(),
                error,
            })
    }
}

fn split_placeholder(token: &str) -> (&str, Option<&str>) {
    match token.split_once(':') {
        Some((name, arg)) => (name, Some(arg)),
        None => (token, None),
    }
}
