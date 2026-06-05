use std::fs;

use openmacro_lib::store::{load_from_root, LoadResult, VarKind};
use tempfile::TempDir;

fn write_yaml(root: &TempDir, relative_path: &str, contents: &str) {
    let path = root.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn load(root: &TempDir) -> LoadResult {
    load_from_root(root.path()).unwrap()
}

#[test]
fn loads_valid_yaml_file() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "snippets/alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );

    let result = load(&root);

    assert!(result.errors.is_empty());
    assert_eq!(result.snippets.len(), 1);
    assert_eq!(result.snippets[0].trigger, ";sig");
    assert_eq!(result.snippets[0].replace, "hello");
    assert_eq!(result.snippets[0].id, "snippets/alpha.yaml::;sig");
    assert_eq!(
        result.snippets[0].source_file,
        root.path().join("snippets/alpha.yaml")
    );
}

#[test]
fn missing_version_is_captured_as_error() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "missing-version.yaml",
        r#"
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );

    let result = load(&root);

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert!(
        result.errors[0]
            .message
            .contains("version"),
        "{}",
        result.errors[0].message
    );
}

#[test]
fn unknown_version_is_captured_as_error() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "unknown-version.yaml",
        r#"
version: 2
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );

    let result = load(&root);

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert!(
        result.errors[0]
            .message
            .contains("version"),
        "{}",
        result.errors[0].message
    );
}

#[test]
fn malformed_yaml_is_isolated_to_its_file() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "good.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );
    write_yaml(
        &root,
        "broken.yaml",
        r#"
version: 1
snippets:
  - trigger: ;oops
    replace: "unterminated
"#,
    );

    let result = load(&root);

    assert_eq!(result.snippets.len(), 1);
    assert_eq!(result.snippets[0].trigger, ";sig");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].path, root.path().join("broken.yaml"));
}

#[test]
fn vars_round_trip_all_supported_kinds() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "vars.yaml",
        r#"
version: 1
snippets:
  - trigger: ;vars
    replace: body
    vars:
      - name: text_name
        kind: text
        label: Text
        default: alpha
        required: true
      - name: textarea_name
        kind: textarea
        label: Textarea
        default: beta
      - name: choice_name
        kind: choice
        label: Choice
        options: [one, two]
      - name: number_name
        kind: number
        label: Number
        default: "42"
      - name: datetime_name
        kind: datetime
        label: Datetime
        format: "%Y-%m-%d"
      - name: clipboard_name
        kind: clipboard
        label: Clipboard
      - name: cursor_name
        kind: cursor
        label: Cursor
      - name: shell_name
        kind: shell
        label: Shell
      - name: form_name
        kind: form
        label: Form
"#,
    );

    let result = load(&root);
    let vars = &result.snippets[0].vars;

    assert!(result.errors.is_empty());
    assert_eq!(vars.len(), 9);
    assert_eq!(vars[0].kind, VarKind::Text);
    assert_eq!(vars[1].kind, VarKind::Textarea);
    assert_eq!(vars[2].kind, VarKind::Choice);
    assert_eq!(vars[2].options, vec!["one".to_string(), "two".to_string()]);
    assert_eq!(vars[3].kind, VarKind::Number);
    assert_eq!(vars[4].kind, VarKind::Datetime);
    assert_eq!(vars[4].format.as_deref(), Some("%Y-%m-%d"));
    assert_eq!(vars[5].kind, VarKind::Clipboard);
    assert_eq!(vars[6].kind, VarKind::Cursor);
    assert_eq!(vars[7].kind, VarKind::Shell);
    assert_eq!(vars[8].kind, VarKind::Form);
    assert!(vars[0].required);
}

#[test]
fn duplicate_trigger_within_file_is_captured_as_error() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "dupe.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: first
  - trigger: ;sig
    replace: second
"#,
    );

    let result = load(&root);

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert!(
        result.errors[0]
            .message
            .contains("duplicate"),
        "{}",
        result.errors[0].message
    );
}
