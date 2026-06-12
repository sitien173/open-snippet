use std::{
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
};

use openmacro_lib::{
    commands::snippets::{
        get_store_settings_dto_inner, get_store_settings_inner, list_load_errors_inner,
        list_snippets_inner, load_snippet_store_state, reload_snippets_inner, save_snippet_inner,
        set_store_settings_inner, SaveSnippetDto,
    },
    store::{StoreSettings, VarDecl, VarKind},
};
use tempfile::TempDir;

fn snippets_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

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

#[test]
fn save_reload_and_list_returns_new_trigger() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );
    let state = load_snippet_store_state().unwrap();

    save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: root.path().join("alpha.yaml"),
            original_trigger: None,
            original_trigger_literal: None,
            trigger: ";log".to_string(),
            trigger_literal: false,
            replace: "line one".to_string(),
            vars: vec![VarDecl {
                name: "clip".to_string(),
                kind: VarKind::Clipboard,
                label: Some("Clipboard".to_string()),
                default: None,
                required: false,
                options: Vec::new(),
                format: None,
                cmd: Vec::new(),
                timeout_ms: None,
                confirm: false,
            }],
        },
    )
    .unwrap();

    let reload = reload_snippets_inner(&state).unwrap();
    let snippets = list_snippets_inner(&state);

    assert_eq!(reload.loaded, 2);
    assert!(reload.errors.is_empty());
    assert!(list_load_errors_inner(&state).is_empty());
    assert!(snippets.iter().any(|snippet| {
        snippet.trigger == ";log"
            && snippet.replace == "line one"
            && snippet.file_relative == "alpha.yaml"
            && snippet.id == "alpha.yaml::;log"
            && snippet.vars.len() == 1
    }));
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_snippet_rejects_same_file_trigger_collision() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );
    let state = load_snippet_store_state().unwrap();

    let error = save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: root.path().join("alpha.yaml"),
            original_trigger: None,
            original_trigger_literal: None,
            trigger: ";sig".to_string(),
            trigger_literal: false,
            replace: "collision".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap_err();

    assert_eq!(error, "trigger collision: ;sig");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_snippet_rejects_path_outside_snippets_root() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );
    fs::write(
        outside.path().join("outside.yaml"),
        r#"
version: 1
snippets: []
"#,
    )
    .unwrap();
    let state = load_snippet_store_state().unwrap();

    let error = save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: outside.path().join("outside.yaml"),
            original_trigger: None,
            original_trigger_literal: None,
            trigger: ";owned".to_string(),
            trigger_literal: false,
            replace: "outside".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap_err();

    assert!(error.contains("snippets root"), "{error}");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_snippet_rejects_settings_file_target() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "_settings.yaml",
        r#"
version: 1
snippets: []
"#,
    );
    let state = load_snippet_store_state().unwrap();

    let error = save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: root.path().join("_settings.yaml"),
            original_trigger: None,
            original_trigger_literal: None,
            trigger: ";owned".to_string(),
            trigger_literal: false,
            replace: "settings".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap_err();

    assert!(error.contains("_settings.yaml"), "{error}");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_snippet_rejects_non_yaml_target() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.txt",
        r#"
version: 1
snippets: []
"#,
    );
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets: []
"#,
    );
    let state = load_snippet_store_state().unwrap();

    let error = save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: root.path().join("alpha.txt"),
            original_trigger: None,
            original_trigger_literal: None,
            trigger: ";owned".to_string(),
            trigger_literal: false,
            replace: "text".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap_err();

    assert!(error.contains("YAML"), "{error}");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_snippet_rejects_symlink_target_outside_snippets_root() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets: []
"#,
    );
    fs::write(
        outside.path().join("outside.yaml"),
        r#"
version: 1
snippets: []
"#,
    )
    .unwrap();
    let link = root.path().join("linked.yaml");
    if create_file_symlink(&outside.path().join("outside.yaml"), &link).is_err() {
        std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
        return;
    }
    let state = load_snippet_store_state().unwrap();

    let error = save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: link,
            original_trigger: None,
            original_trigger_literal: None,
            trigger: ";owned".to_string(),
            trigger_literal: false,
            replace: "linked".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap_err();

    assert!(error.contains("snippets root"), "{error}");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_reload_and_list_preserves_trigger_literal() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: sig
    trigger_literal: true
    replace: hello
"#,
    );
    let state = load_snippet_store_state().unwrap();

    save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: root.path().join("alpha.yaml"),
            original_trigger: Some("sig".to_string()),
            original_trigger_literal: Some(true),
            trigger: "sig".to_string(),
            trigger_literal: true,
            replace: "updated".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap();

    let reload = reload_snippets_inner(&state).unwrap();
    let snippets = list_snippets_inner(&state);
    let saved = fs::read_to_string(root.path().join("alpha.yaml")).unwrap();

    assert_eq!(reload.loaded, 1);
    assert!(reload.errors.is_empty());
    assert!(saved.contains("trigger_literal: true"), "{saved}");
    assert!(snippets.iter().any(|snippet| {
        snippet.trigger == "sig"
            && snippet.trigger_literal
            && snippet.replace == "updated"
            && snippet.id == "alpha.yaml::sig"
    }));
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn list_snippets_exposes_raw_and_effective_trigger_under_custom_prefix() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
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

    let state = load_snippet_store_state().unwrap();
    let snippets = list_snippets_inner(&state);

    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].trigger, "email");
    assert_eq!(snippets[0].effective_trigger, ":email");
    assert_eq!(snippets[0].id, "alpha.yaml:::email");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn save_replaces_bare_trigger_under_custom_prefix_instead_of_appending() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
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
    let state = load_snippet_store_state().unwrap();

    save_snippet_inner(
        &state,
        SaveSnippetDto {
            source_file: root.path().join("alpha.yaml"),
            original_trigger: Some("email".to_string()),
            original_trigger_literal: Some(false),
            trigger: "email".to_string(),
            trigger_literal: false,
            replace: "updated".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap();

    let reload = reload_snippets_inner(&state).unwrap();
    let snippets = list_snippets_inner(&state);
    let saved = fs::read_to_string(root.path().join("alpha.yaml")).unwrap();

    assert_eq!(reload.loaded, 1);
    assert!(reload.errors.is_empty());
    assert_eq!(saved.matches("- trigger: email").count(), 1, "{saved}");
    assert!(snippets.iter().any(|snippet| {
        snippet.trigger == "email"
            && snippet.effective_trigger == ":email"
            && snippet.replace == "updated"
    }));
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn store_settings_round_trip_through_backend_helpers() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());
    write_yaml(
        &root,
        "alpha.yaml",
        r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
    );
    let state = load_snippet_store_state().unwrap();

    assert_eq!(
        get_store_settings_inner(&state).unwrap(),
        StoreSettings {
            trigger_prefix: ";".to_string(),
            expand_mode: openmacro_lib::store::ExpandMode::Manual,
        }
    );

    set_store_settings_inner(
        &state,
        StoreSettings {
            trigger_prefix: ":".to_string(),
            expand_mode: openmacro_lib::store::ExpandMode::Auto,
        },
    )
    .unwrap();

    assert_eq!(
        get_store_settings_inner(&state).unwrap(),
        StoreSettings {
            trigger_prefix: ":".to_string(),
            expand_mode: openmacro_lib::store::ExpandMode::Auto,
        }
    );
    assert!(fs::read_to_string(root.path().join("_settings.yaml"))
        .unwrap()
        .contains("trigger_prefix: ':'"));
    assert!(fs::read_to_string(root.path().join("_settings.yaml"))
        .unwrap()
        .contains("expand_mode: auto"));
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}

#[test]
fn test_get_store_settings_dto_migration() {
    let _guard = snippets_test_guard();
    let root = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_SNIPPETS_ROOT", root.path());

    // 1. Fresh install: settings file does not exist.
    // get_store_settings should return expand_mode_missing: false
    let state = load_snippet_store_state().unwrap();
    let settings = get_store_settings_dto_inner(&state).unwrap();
    assert!(!settings.expand_mode_missing);

    // 2. Upgrade: settings file exists but lacks expand_mode.
    write_yaml(&root, "_settings.yaml", "trigger_prefix: ':'\n");
    let settings = get_store_settings_dto_inner(&state).unwrap();
    assert!(settings.expand_mode_missing);

    // 3. Saved settings: settings file exists and contains expand_mode.
    write_yaml(
        &root,
        "_settings.yaml",
        "trigger_prefix: ':'\nexpand_mode: manual\n",
    );
    let settings = get_store_settings_dto_inner(&state).unwrap();
    assert!(!settings.expand_mode_missing);

    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}
