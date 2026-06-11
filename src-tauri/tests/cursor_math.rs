use std::{fs, path::PathBuf};

use openmacro_lib::{
    commands::prefs::Prefs,
    expand::{ClipboardReader, Resolver},
    store::{load_from_root, LoadError, Snippet},
};
use tempfile::TempDir;

#[derive(Default)]
struct StubClipboard;

impl ClipboardReader for StubClipboard {
    fn read_text(&mut self) -> Option<String> {
        None
    }
}

fn snippet(replace: &str) -> Snippet {
    Snippet {
        id: "test::snippet".to_string(),
        trigger: ";test".to_string(),
        raw_trigger: ";test".to_string(),
        trigger_literal: false,
        replace: replace.to_string(),
        vars: Vec::new(),
        source_file: PathBuf::from("test.yaml"),
    }
}

fn write_yaml(root: &TempDir, relative_path: &str, contents: &str) {
    let path = root.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn no_cursor_token_returns_none() {
    let mut clipboard = StubClipboard;
    let prefs = Prefs::default();

    let resolved = Resolver::new(&prefs)
        .resolve(&snippet("alpha"), &mut clipboard, None)
        .unwrap();

    assert_eq!(resolved.text, "alpha");
    assert_eq!(resolved.cursor_chars_after_token, None);
}

#[test]
fn single_cursor_token_uses_ascii_utf16_count() {
    let mut clipboard = StubClipboard;
    let prefs = Prefs::default();

    let resolved = Resolver::new(&prefs)
        .resolve(&snippet("abc$|$def"), &mut clipboard, None)
        .unwrap();

    assert_eq!(resolved.text, "abcdef");
    assert_eq!(resolved.cursor_chars_after_token, Some(3));
}

#[test]
fn single_cursor_token_counts_utf16_units_after_token() {
    let mut clipboard = StubClipboard;
    let prefs = Prefs::default();

    let resolved = Resolver::new(&prefs)
        .resolve(&snippet("x$|$\u{1F642}\u{754C}"), &mut clipboard, None)
        .unwrap();

    assert_eq!(resolved.text, "x\u{1F642}\u{754C}");
    assert_eq!(resolved.cursor_chars_after_token, Some(3));
}

#[test]
fn loader_rejects_more_than_one_cursor_token() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "cursor.yaml",
        r#"
version: 1
snippets:
  - trigger: ;cursor
    replace: one $|$ two $|$ three
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.snippets.is_empty());
    assert_eq!(
        result.errors,
        vec![LoadError::TooManyCursorTokens {
            path: root.path().join("cursor.yaml"),
            trigger: ";cursor".to_string(),
        }]
    );
}
