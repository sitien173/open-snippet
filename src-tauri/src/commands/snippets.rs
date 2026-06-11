use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::store::{
    effective_trigger, load_from_root, load_settings, settings_path, validate_trigger_prefix,
    LoadError, Snippet, StoreSettings, VarDecl,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnippetDto {
    pub id: String,
    pub trigger: String,
    pub effective_trigger: String,
    pub trigger_literal: bool,
    pub replace: String,
    pub vars: Vec<VarDecl>,
    pub source_file: String,
    pub file_relative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveSnippetDto {
    pub source_file: PathBuf,
    pub original_trigger: Option<String>,
    #[serde(default)]
    pub original_trigger_literal: Option<bool>,
    pub trigger: String,
    #[serde(default)]
    pub trigger_literal: bool,
    pub replace: String,
    pub vars: Vec<VarDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoadErrorDto {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReloadResult {
    pub loaded: usize,
    pub errors: Vec<LoadErrorDto>,
}

#[derive(Debug, Clone)]
pub struct SnippetStoreSnapshot {
    pub revision: u64,
    pub snippets: Vec<Snippet>,
    pub errors: Vec<LoadError>,
}

pub struct SnippetStoreState {
    root: PathBuf,
    snapshot: Arc<RwLock<SnippetStoreSnapshot>>,
    watcher: Mutex<Option<crate::store::Store>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RootDocument {
    version: u32,
    snippets: Vec<SnippetDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnippetDocument {
    trigger: String,
    #[serde(default)]
    trigger_literal: bool,
    replace: String,
    #[serde(default)]
    vars: Vec<VarDecl>,
}

impl SnippetStoreState {
    pub fn load(root: PathBuf) -> Result<Self, String> {
        let loaded = load_from_root(&root).map_err(|error| error.to_string())?;
        Ok(Self {
            root,
            snapshot: Arc::new(RwLock::new(SnippetStoreSnapshot {
                revision: 0,
                snippets: loaded.snippets,
                errors: loaded.errors,
            })),
            watcher: Mutex::new(None),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn snapshot(&self) -> SnippetStoreSnapshot {
        self.snapshot.read().unwrap().clone()
    }

    pub fn replace_snapshot(&self, snippets: Vec<Snippet>, errors: Vec<LoadError>) {
        let mut snapshot = self.snapshot.write().unwrap();
        snapshot.revision += 1;
        snapshot.snippets = snippets;
        snapshot.errors = errors;
    }

    pub fn apply_watcher_snapshot(&self, snapshot: &crate::store::SnapshotInner) {
        let mut current = self.snapshot.write().unwrap();
        current.revision = snapshot.revision;
        current.snippets = snapshot.snippets.clone();
        current.errors = snapshot.errors.clone();
    }

    pub fn snapshot_handle(&self) -> Arc<RwLock<SnippetStoreSnapshot>> {
        Arc::clone(&self.snapshot)
    }

    pub fn set_watcher(&self, watcher: crate::store::Store) {
        *self.watcher.lock().unwrap() = Some(watcher);
    }

    pub fn trigger_reload(&self) {
        if let Some(watcher) = self.watcher.lock().unwrap().as_ref() {
            watcher.reload_now();
        }
    }
}

pub fn load_snippet_store_state() -> Result<SnippetStoreState, String> {
    SnippetStoreState::load(snippets_root()?)
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn list_snippets(state: tauri::State<'_, SnippetStoreState>) -> Vec<SnippetDto> {
    let snippets = list_snippets_inner(state.inner());
    tracing::debug!(count = snippets.len(), "listed snippets");
    snippets
}

#[tauri::command]
#[tracing::instrument(skip(state, payload), fields(trigger = %payload.trigger, source_file = %payload.source_file.display()))]
pub fn save_snippet(
    state: tauri::State<'_, SnippetStoreState>,
    payload: SaveSnippetDto,
) -> Result<(), String> {
    tracing::info!("saving snippet");
    save_snippet_inner(state.inner(), payload)
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn reload_snippets(state: tauri::State<'_, SnippetStoreState>) -> Result<ReloadResult, String> {
    let result = reload_snippets_inner(state.inner())?;
    tracing::info!(
        loaded = result.loaded,
        errors = result.errors.len(),
        "reloaded snippets"
    );
    Ok(result)
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn list_load_errors(state: tauri::State<'_, SnippetStoreState>) -> Vec<LoadErrorDto> {
    let errors = list_load_errors_inner(state.inner());
    tracing::debug!(count = errors.len(), "listed snippet load errors");
    errors
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn get_store_settings(
    state: tauri::State<'_, SnippetStoreState>,
) -> Result<StoreSettings, String> {
    get_store_settings_inner(state.inner())
}

#[tauri::command]
#[tracing::instrument(skip(state, settings), fields(trigger_prefix = %settings.trigger_prefix))]
pub fn set_store_settings(
    state: tauri::State<'_, SnippetStoreState>,
    settings: StoreSettings,
) -> Result<(), String> {
    set_store_settings_inner(state.inner(), settings)
}

pub fn list_snippets_inner(state: &SnippetStoreState) -> Vec<SnippetDto> {
    let snapshot = state.snapshot();
    snapshot
        .snippets
        .into_iter()
        .map(|snippet| SnippetDto {
            file_relative: relative_path(&snippet.source_file, state.root()),
            source_file: snippet.source_file.display().to_string(),
            id: snippet.id,
            trigger: snippet.raw_trigger,
            effective_trigger: snippet.trigger,
            trigger_literal: snippet.trigger_literal,
            replace: snippet.replace,
            vars: snippet.vars,
        })
        .collect()
}

pub fn save_snippet_inner(
    state: &SnippetStoreState,
    payload: SaveSnippetDto,
) -> Result<(), String> {
    let mut document = read_yaml_document(&payload.source_file)?;
    let settings = get_store_settings_inner(state)?;

    let replacement = SnippetDocument {
        trigger: payload.trigger.clone(),
        trigger_literal: payload.trigger_literal,
        replace: payload.replace,
        vars: payload.vars,
    };

    let original_trigger = payload.original_trigger.as_deref();
    let original_trigger_literal = payload.original_trigger_literal;
    let mut replaced = false;
    for snippet in &mut document.snippets {
        let trigger_matches = Some(snippet.trigger.as_str()) == original_trigger;
        let literal_matches = original_trigger_literal
            .map(|value| snippet.trigger_literal == value)
            .unwrap_or(true);
        if trigger_matches && literal_matches {
            *snippet = replacement.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        document.snippets.push(replacement.clone());
    }

    let mut effective_triggers = std::collections::HashSet::new();
    for snippet in &document.snippets {
        let snippet_effective_trigger =
            effective_trigger(&snippet.trigger, snippet.trigger_literal, &settings);
        if !effective_triggers.insert(snippet_effective_trigger.clone()) {
            return Err(format!("trigger collision: {snippet_effective_trigger}"));
        }
    }
    write_yaml_document(&payload.source_file, &document)
}

pub fn get_store_settings_inner(state: &SnippetStoreState) -> Result<StoreSettings, String> {
    match load_settings(state.root()) {
        Ok(settings) => Ok(settings),
        Err(error) => Err(error.message()),
    }
}

pub fn set_store_settings_inner(
    state: &SnippetStoreState,
    settings: StoreSettings,
) -> Result<(), String> {
    let path = settings_path(state.root());
    validate_trigger_prefix(&path, &settings.trigger_prefix).map_err(|error| error.message())?;
    write_store_settings(&path, &settings)?;
    state.trigger_reload();
    Ok(())
}

pub fn reload_snippets_inner(state: &SnippetStoreState) -> Result<ReloadResult, String> {
    state.trigger_reload();
    let loaded = load_from_root(state.root()).map_err(|error| error.to_string())?;
    let errors = loaded.errors.iter().map(load_error_dto).collect::<Vec<_>>();
    let loaded_count = loaded.snippets.len();
    state.replace_snapshot(loaded.snippets, loaded.errors);
    Ok(ReloadResult {
        loaded: loaded_count,
        errors,
    })
}

pub fn list_load_errors_inner(state: &SnippetStoreState) -> Vec<LoadErrorDto> {
    state.snapshot().errors.iter().map(load_error_dto).collect()
}

fn load_error_dto(error: &LoadError) -> LoadErrorDto {
    LoadErrorDto {
        path: error.path().display().to_string(),
        message: error.message(),
    }
}

fn read_yaml_document(path: &Path) -> Result<RootDocument, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_yaml::from_str(&contents).map_err(|error| error.to_string())
}

fn write_yaml_document(path: &Path, document: &RootDocument) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("missing parent directory for {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let serialized = serde_yaml::to_string(document).map_err(|error| error.to_string())?;
    let mut temp = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    use std::io::Write;
    temp.write_all(serialized.as_bytes())
        .map_err(|error| error.to_string())?;
    temp.persist(path)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

fn write_store_settings(path: &Path, settings: &StoreSettings) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("missing parent directory for {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let serialized = serde_yaml::to_string(settings).map_err(|error| error.to_string())?;
    let mut temp = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    use std::io::Write;
    temp.write_all(serialized.as_bytes())
        .map_err(|error| error.to_string())?;
    temp.persist(path)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn snippets_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("OPENMACRO_SNIPPETS_ROOT") {
        return Ok(PathBuf::from(root));
    }

    let Some(appdata_dir) = std::env::var_os("APPDATA").map(PathBuf::from) else {
        return Err("APPDATA is not set".to_string());
    };
    Ok(appdata_dir.join("openmacro").join("snippets"))
}
