use std::{
    fs,
    path::PathBuf,
    sync::RwLock,
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::engine::set_paused;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prefs {
    pub paused: bool,
    pub autostart: bool,
    pub max_expansion_len: usize,
    pub shell_consent: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            paused: false,
            autostart: false,
            max_expansion_len: 32_768,
            shell_consent: false,
        }
    }
}

pub struct PrefsState {
    path: PathBuf,
    prefs: RwLock<Prefs>,
}

impl PrefsState {
    pub fn new(path: PathBuf, prefs: Prefs) -> Self {
        Self {
            path,
            prefs: RwLock::new(prefs),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[tauri::command]
pub fn get_prefs(state: tauri::State<'_, PrefsState>) -> Prefs {
    get_prefs_inner(state.inner())
}

#[tauri::command]
pub fn set_prefs(state: tauri::State<'_, PrefsState>, prefs: Prefs) -> Result<(), String> {
    set_prefs_inner(state.inner(), prefs)
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
    serde_json::from_str(&contents).map_err(|error| error.to_string())
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
    temp.persist(path).map_err(|error| error.error.to_string())?;
    Ok(())
}
