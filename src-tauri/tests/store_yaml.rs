use std::{
    fs,
    path::{Path, PathBuf},
};

use openmacro_lib::store::{load_from_root, LoadResult, LoggingConfig, VarKind};
use tempfile::TempDir;

fn write_yaml(root: &TempDir, relative_path: &str, contents: &str) {
    let path = root.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link)
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }
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
fn loader_rejects_yaml_symlink_target_outside_root() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(
        outside.path().join("outside.yaml"),
        r#"
version: 1
snippets:
  - trigger: ;outside
    replace: escaped
"#,
    )
    .unwrap();
    let link = root.path().join("linked.yaml");
    if create_file_symlink(&outside.path().join("outside.yaml"), &link).is_err() {
        return;
    }

    let result = load(&root);

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert!(
        result.errors[0].message().contains("snippets root"),
        "{}",
        result.errors[0].message()
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
        result.errors[0].message().contains("version"),
        "{}",
        result.errors[0].message()
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
        result.errors[0].message().contains("version"),
        "{}",
        result.errors[0].message()
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
    assert_eq!(result.errors[0].path(), root.path().join("broken.yaml"));
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
        cmd: [cmd, /c, ver]
        timeout_ms: 1000
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
    assert_eq!(
        vars[7].cmd,
        vec!["cmd".to_string(), "/c".to_string(), "ver".to_string()]
    );
    assert_eq!(vars[7].timeout_ms, Some(1000));
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
        result.errors[0].message().contains("duplicate"),
        "{}",
        result.errors[0].message()
    );
}

#[test]
fn shell_cmd_string_is_captured_as_parse_error() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "shell-string.yaml",
        r#"
version: 1
snippets:
  - trigger: ;shell
    replace: "{{out}}"
    vars:
      - name: out
        kind: shell
        cmd: "echo hi"
        timeout_ms: 1000
"#,
    );

    let result = load(&root);

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert!(matches!(
        result.errors[0],
        openmacro_lib::store::LoadError::Parse { .. }
    ));
}

#[test]
fn shell_var_without_timeout_is_rejected() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "shell-missing-timeout.yaml",
        r#"
version: 1
snippets:
  - trigger: ;shell
    replace: "{{out}}"
    vars:
      - name: out
        kind: shell
        cmd: [cmd, /c, ver]
"#,
    );

    let result = load(&root);

    assert!(result.snippets.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0],
        openmacro_lib::store::LoadError::ShellTimeoutInvalid {
            path: root.path().join("shell-missing-timeout.yaml"),
            trigger: ";shell".to_string(),
            name: "out".to_string(),
        }
    );
}

#[test]
fn valid_shell_var_loads_with_cmd_and_timeout() {
    let root = TempDir::new().unwrap();
    write_yaml(
        &root,
        "shell-valid.yaml",
        r#"
version: 1
snippets:
  - trigger: ;shell
    replace: "{{out}}"
    vars:
      - name: out
        kind: shell
        cmd: [cmd, /c, ver]
        timeout_ms: 1000
        confirm: true
"#,
    );

    let result = load(&root);

    assert!(result.errors.is_empty());
    assert_eq!(result.snippets.len(), 1);
    let var = &result.snippets[0].vars[0];
    assert_eq!(var.kind, VarKind::Shell);
    assert_eq!(var.cmd, vec!["cmd", "/c", "ver"]);
    assert_eq!(var.timeout_ms, Some(1000));
    assert!(var.confirm);
}

#[test]
fn shipped_default_yaml_loads_without_errors() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shipped_root = manifest_dir.parent().unwrap().join("snippets");

    let result = load_from_root(&shipped_root).unwrap();

    assert!(result.errors.is_empty());
    assert_eq!(result.snippets.len(), 7);
    assert!(result
        .snippets
        .iter()
        .any(|snippet| snippet.trigger == ";sig"));
    assert!(result
        .snippets
        .iter()
        .any(|snippet| snippet.trigger == ";head"));
}

#[test]
fn logging_config_defaults_match_design() {
    let cfg: LoggingConfig = serde_yaml::from_str("{}").unwrap();

    assert_eq!(cfg.level, "info");
    assert!(cfg.modules.is_empty());
    assert!(cfg.file.enabled);
    assert_eq!(cfg.file.max_files, 7);
    assert!(!cfg.verbose_content);
    assert_eq!(cfg.frontend.level, "info");
    assert!(cfg.frontend.modules.is_empty());
}

#[test]
fn logging_config_deserializes_overrides() {
    let cfg: LoggingConfig = serde_yaml::from_str(
        r#"
level: debug
modules:
  openmacro::matcher: trace
file:
  enabled: false
  max_files: 3
verbose_content: true
frontend:
  level: warn
  modules:
    settings: debug
"#,
    )
    .unwrap();

    assert_eq!(cfg.level, "debug");
    assert_eq!(cfg.modules["openmacro::matcher"], "trace");
    assert!(!cfg.file.enabled);
    assert_eq!(cfg.file.max_files, 3);
    assert!(cfg.verbose_content);
    assert_eq!(cfg.frontend.level, "warn");
    assert_eq!(cfg.frontend.modules["settings"], "debug");
}
