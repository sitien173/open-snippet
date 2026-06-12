use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use tauri_plugin_autostart::ManagerExt;
use tempfile::NamedTempFile;

use crate::engine::set_paused;

pub const MIN_EXPANSION_LEN: usize = 1;
pub const MAX_EXPANSION_LEN: usize = 262_144;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prefs {
    pub paused: bool,
    pub autostart: bool,
    pub max_expansion_len: usize,
    pub shell_consent: bool,
    #[serde(default)]
    pub last_crash_check: Option<u64>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            paused: false,
            autostart: false,
            max_expansion_len: 32_768,
            shell_consent: false,
            last_crash_check: None,
        }
    }
}

pub struct PrefsState {
    path: PathBuf,
    prefs: Arc<RwLock<Prefs>>,
}

pub trait AutostartController {
    fn set_enabled(&self, enabled: bool) -> Result<(), String>;
}

struct NoopAutostartController;

impl AutostartController for NoopAutostartController {
    fn set_enabled(&self, _enabled: bool) -> Result<(), String> {
        Ok(())
    }
}

struct TauriAutostartController<'a> {
    app: &'a tauri::AppHandle,
}

impl AutostartController for TauriAutostartController<'_> {
    fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let manager = self.app.autolaunch();
        if enabled {
            manager.enable().map_err(|error| error.to_string())
        } else {
            manager.disable().map_err(|error| error.to_string())
        }
    }
}

impl PrefsState {
    pub fn new(path: PathBuf, prefs: Prefs) -> Self {
        Self {
            path,
            prefs: Arc::new(RwLock::new(prefs)),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn prefs_handle(&self) -> Arc<RwLock<Prefs>> {
        Arc::clone(&self.prefs)
    }
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn get_prefs(state: tauri::State<'_, PrefsState>) -> Prefs {
    let prefs = get_prefs_inner(state.inner());
    tracing::debug!(
        paused = prefs.paused,
        autostart = prefs.autostart,
        max_expansion_len = prefs.max_expansion_len,
        shell_consent = prefs.shell_consent,
        "loaded prefs"
    );
    prefs
}

#[tauri::command]
#[tracing::instrument(skip(state, prefs), fields(paused = prefs.paused, autostart = prefs.autostart, shell_consent = prefs.shell_consent))]
pub fn set_prefs(
    app: tauri::AppHandle,
    state: tauri::State<'_, PrefsState>,
    prefs: Prefs,
) -> Result<(), String> {
    tracing::info!("saving prefs");
    let autostart = TauriAutostartController { app: &app };
    set_prefs_with_autostart_inner(state.inner(), prefs, &autostart)
}

pub fn load_prefs_state() -> Result<PrefsState, String> {
    let path = prefs_path()?;
    let prefs = if path.exists() {
        read_prefs(&path)?
    } else {
        let prefs = Prefs::default();
        write_prefs(&path, &prefs)?;
        prefs
    };
    set_paused(prefs.paused);
    Ok(PrefsState::new(path, prefs))
}

pub fn get_prefs_inner(state: &PrefsState) -> Prefs {
    state.prefs.read().unwrap().clone()
}

pub fn set_prefs_inner(state: &PrefsState, prefs: Prefs) -> Result<(), String> {
    set_prefs_with_autostart_inner(state, prefs, &NoopAutostartController)
}

pub fn set_prefs_with_autostart_inner(
    state: &PrefsState,
    prefs: Prefs,
    autostart: &dyn AutostartController,
) -> Result<(), String> {
    validate_prefs(&prefs)?;
    let previous = get_prefs_inner(state);
    if previous.autostart != prefs.autostart {
        autostart.set_enabled(prefs.autostart)?;
    }
    write_prefs(state.path(), &prefs)?;
    *state.prefs.write().unwrap() = prefs.clone();
    set_paused(prefs.paused);
    Ok(())
}

pub fn prefs_path() -> Result<PathBuf, String> {
    if let Some(override_path) = std::env::var_os("OPENMACRO_PREFS_PATH") {
        return Ok(PathBuf::from(override_path));
    }

    let Some(config_dir) = dirs::config_dir() else {
        return Err("config directory unavailable".to_string());
    };
    Ok(config_dir.join("openmacro").join("prefs.json"))
}

fn read_prefs(path: &PathBuf) -> Result<Prefs, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let prefs: Prefs = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    validate_prefs(&prefs)?;
    Ok(prefs)
}

fn write_prefs(path: &PathBuf, prefs: &Prefs) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(format!("missing parent directory for {}", path.display()));
    };
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let serialized = serde_json::to_vec_pretty(prefs).map_err(|error| error.to_string())?;
    let mut temp = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    use std::io::Write;
    temp.write_all(&serialized)
        .map_err(|error| error.to_string())?;
    temp.persist(path)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

fn validate_prefs(prefs: &Prefs) -> Result<(), String> {
    if !(MIN_EXPANSION_LEN..=MAX_EXPANSION_LEN).contains(&prefs.max_expansion_len) {
        return Err(format!(
            "invalid max_expansion_len: must be between {MIN_EXPANSION_LEN} and {MAX_EXPANSION_LEN}"
        ));
    }
    Ok(())
}
