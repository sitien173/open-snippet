use std::fs;

use openmacro_lib::store::load_from_root;
use tempfile::TempDir;

fn write_yaml(root: &TempDir, relative_path: &str, contents: &str) {
    let path = root.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn default_prefix_preserves_existing_literal_trigger() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;foo
    replace: hello
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(result.snippets.len(), 1);
    assert_eq!(result.snippets[0].trigger, ";foo");
}

#[test]
fn custom_prefix_applies_to_bare_trigger() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "_settings.yaml",
        r#"
trigger_prefix: ":"
"#,
    );
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: email
    replace: hello
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(result.snippets.len(), 1);
    assert_eq!(result.snippets[0].trigger, ":email");
    assert_eq!(result.snippets[0].id, "alpha.yaml:::email");
}

#[test]
fn existing_prefixed_trigger_is_not_double_prepended() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "_settings.yaml",
        r#"
trigger_prefix: ":"
"#,
    );
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: :email
    replace: hello
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(result.snippets.len(), 1);
    assert_eq!(result.snippets[0].trigger, ":email");
}

#[test]
fn trigger_literal_uses_raw_trigger() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "_settings.yaml",
        r#"
trigger_prefix: ":"
"#,
    );
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    trigger_literal: true
    replace: hello
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(result.snippets.len(), 1);
    assert_eq!(result.snippets[0].trigger, ";sig");
}

#[test]
fn invalid_prefix_is_rejected() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "_settings.yaml",
        r#"
trigger_prefix: "ab"
"#,
    );
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: email
    replace: hello
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].path(), root.path().join("_settings.yaml"));
    assert!(
        result.errors[0].message().contains("trigger_prefix"),
        "{}",
        result.errors[0].message()
    );
}

#[test]
fn duplicate_effective_trigger_is_rejected_with_effective_trigger_in_error() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "_settings.yaml",
        r#"
trigger_prefix: ":"
"#,
    );
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: email
    replace: first
"#,
    );
    write_yaml(
        &root,
        "beta.yaml",
        r#"
version: 1
snippets:
  - trigger: :email
    replace: second
"#,
    );

    let result = load_from_root(root.path()).unwrap();

    assert!(result.snippets.is_empty(), "{:?}", result.snippets);
    assert_eq!(result.errors.len(), 1);
    assert!(
        result.errors[0].message().contains(":email"),
        "{}",
        result.errors[0].message()
    );
}
