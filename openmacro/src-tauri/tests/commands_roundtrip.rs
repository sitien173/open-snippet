use std::{
    fs,
    sync::{Mutex, OnceLock},
};

use openmacro_lib::{
    commands::snippets::{
        list_load_errors_inner, list_snippets_inner, load_snippet_store_state,
        reload_snippets_inner, save_snippet_inner, SaveSnippetDto,
    },
    store::{VarDecl, VarKind},
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
            trigger: ";log".to_string(),
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
            trigger: ";sig".to_string(),
            replace: "collision".to_string(),
            vars: Vec::new(),
        },
    )
    .unwrap_err();

    assert_eq!(error, "trigger collision: ;sig");
    std::env::remove_var("OPENMACRO_SNIPPETS_ROOT");
}
