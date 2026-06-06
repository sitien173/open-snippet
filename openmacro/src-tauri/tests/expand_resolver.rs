use std::path::PathBuf;

use chrono::Local;
use openmacro_lib::{
    expand::{ClipboardReader, ResolveError, Resolver},
    store::{Snippet, VarDecl, VarKind},
};

#[derive(Default)]
struct StubClipboard {
    text: Option<String>,
}

impl ClipboardReader for StubClipboard {
    fn read_text(&mut self) -> Option<String> {
        self.text.clone()
    }
}

fn snippet(replace: &str, vars: Vec<VarDecl>) -> Snippet {
    Snippet {
        id: "test::snippet".to_string(),
        trigger: ";test".to_string(),
        replace: replace.to_string(),
        vars,
        source_file: PathBuf::from("test.yaml"),
    }
}

fn var(name: &str, kind: VarKind, format: Option<&str>) -> VarDecl {
    VarDecl {
        name: name.to_string(),
        kind,
        label: None,
        default: None,
        required: false,
        options: Vec::new(),
        format: format.map(str::to_string),
    }
}

#[test]
fn resolves_builtin_datetime_placeholders() {
    let snippet = snippet("{{date}}|{{time}}|{{datetime}}", Vec::new());
    let mut clipboard = StubClipboard::default();

    let resolved = Resolver::resolve(&snippet, &mut clipboard).unwrap();

    assert_eq!(
        resolved.text,
        format!(
            "{}|{}|{}",
            Local::now().format("%Y-%m-%d"),
            Local::now().format("%H:%M:%S"),
            Local::now().format("%Y-%m-%d %H:%M:%S"),
        )
    );
    assert_eq!(resolved.cursor_chars_after_token, None);
}

#[test]
fn resolves_declared_var_kinds_before_builtin_fallbacks() {
    let snippet = snippet(
        "{{name}}|{{stamp}}|{{clip}}|{{cursor_here}}",
        vec![
            VarDecl {
                name: "name".to_string(),
                kind: VarKind::Text,
                label: None,
                default: Some("alpha".to_string()),
                required: false,
                options: Vec::new(),
                format: None,
            },
            var("stamp", VarKind::Datetime, Some("%Y")),
            var("clip", VarKind::Clipboard, None),
            var("cursor_here", VarKind::Cursor, None),
        ],
    );
    let mut clipboard = StubClipboard {
        text: Some("copied".to_string()),
    };

    let resolved = Resolver::resolve(&snippet, &mut clipboard).unwrap();

    assert_eq!(
        resolved.text,
        format!("alpha|{}|copied|", Local::now().format("%Y"))
    );
    assert_eq!(resolved.cursor_chars_after_token, Some(0));
}

#[test]
fn resolves_strftime_arg_and_clipboard_text() {
    let snippet = snippet("{{date:%Y}} {{clipboard}}", Vec::new());
    let mut clipboard = StubClipboard {
        text: Some("copied".to_string()),
    };

    let resolved = Resolver::resolve(&snippet, &mut clipboard).unwrap();

    assert_eq!(
        resolved.text,
        format!("{} copied", Local::now().format("%Y"))
    );
}

#[test]
fn missing_clipboard_text_resolves_as_empty_string() {
    let snippet = snippet("({{clipboard}})", Vec::new());
    let mut clipboard = StubClipboard::default();

    let resolved = Resolver::resolve(&snippet, &mut clipboard).unwrap();

    assert_eq!(resolved.text, "()");
}

#[test]
fn unknown_placeholder_is_an_error() {
    let snippet = snippet("{{missing}}", Vec::new());
    let mut clipboard = StubClipboard::default();

    let error = Resolver::resolve(&snippet, &mut clipboard).unwrap_err();

    assert_eq!(
        error,
        ResolveError::UnknownPlaceholder {
            name: "missing".to_string(),
        }
    );
}
