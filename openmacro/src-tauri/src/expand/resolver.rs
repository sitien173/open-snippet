use super::{
    clipboard_var::{resolve_clipboard, ClipboardReader},
    datetime::resolve_datetime,
    strip_cursor_token,
};
use crate::store::{Snippet, VarKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub text: String,
    pub cursor_chars_after_token: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    UnknownPlaceholder { name: String },
    UnterminatedPlaceholder,
}

pub struct Resolver;

impl Resolver {
    pub fn resolve(
        snippet: &Snippet,
        clipboard_reader: &mut impl ClipboardReader,
    ) -> Result<Resolved, ResolveError> {
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
            let value = resolve_placeholder(snippet, name, arg, clipboard_reader)?;
            rendered.push_str(&value);
            rest = &after_open[end + 2..];
        }

        rendered.push_str(rest);
        let (text, cursor_chars_after_token) =
            strip_cursor_token(&rendered).expect("loader validates cursor token count");

        Ok(Resolved {
            text,
            cursor_chars_after_token,
        })
    }
}

fn split_placeholder(token: &str) -> (&str, Option<&str>) {
    match token.split_once(':') {
        Some((name, arg)) => (name, Some(arg)),
        None => (token, None),
    }
}

fn resolve_placeholder(
    snippet: &Snippet,
    name: &str,
    arg: Option<&str>,
    clipboard_reader: &mut impl ClipboardReader,
) -> Result<String, ResolveError> {
    if let Some(var) = snippet.vars.iter().find(|var| var.name == name) {
        return Ok(match var.kind {
            VarKind::Text
            | VarKind::Textarea
            | VarKind::Choice
            | VarKind::Number
            | VarKind::Shell
            | VarKind::Form => var.default.clone().unwrap_or_default(),
            VarKind::Datetime => resolve_datetime(arg.or(var.format.as_deref()), "%Y-%m-%d %H:%M:%S"),
            VarKind::Clipboard => resolve_clipboard(clipboard_reader),
            VarKind::Cursor => "$|$".to_string(),
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
