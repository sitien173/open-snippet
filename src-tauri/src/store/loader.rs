//! Recursive YAML loader for snippet files.

use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::expand::{strip_cursor_token, CursorTokenError};

use super::{Snippet, StoreSettings, VarDecl, VarKind};

pub(crate) const SETTINGS_FILE_NAME: &str = "_settings.yaml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadResult {
    pub settings: StoreSettings,
    pub snippets: Vec<Snippet>,
    pub errors: Vec<LoadError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io {
        path: PathBuf,
        message: String,
    },
    Parse {
        path: PathBuf,
        message: String,
    },
    MissingVersion {
        path: PathBuf,
    },
    UnsupportedVersion {
        path: PathBuf,
        version: u32,
    },
    RelativePath {
        path: PathBuf,
    },
    DuplicateTrigger {
        path: PathBuf,
        trigger: String,
    },
    InvalidSettings {
        path: PathBuf,
        message: String,
    },
    TooManyCursorTokens {
        path: PathBuf,
        trigger: String,
    },
    ShellCmdNotArray {
        path: PathBuf,
        trigger: String,
        name: String,
    },
    ShellTimeoutInvalid {
        path: PathBuf,
        trigger: String,
        name: String,
    },
    RootEscape {
        path: PathBuf,
        message: String,
    },
}

impl LoadError {
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. }
            | Self::Parse { path, .. }
            | Self::MissingVersion { path }
            | Self::UnsupportedVersion { path, .. }
            | Self::RelativePath { path }
            | Self::DuplicateTrigger { path, .. }
            | Self::InvalidSettings { path, .. }
            | Self::TooManyCursorTokens { path, .. }
            | Self::ShellCmdNotArray { path, .. }
            | Self::ShellTimeoutInvalid { path, .. }
            | Self::RootEscape { path, .. } => path,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Io { message, .. } | Self::Parse { message, .. } => message.clone(),
            Self::MissingVersion { .. } => "missing required version: 1".to_string(),
            Self::UnsupportedVersion { version, .. } => format!("unsupported version: {version}"),
            Self::RelativePath { .. } => "failed to derive relative path".to_string(),
            Self::DuplicateTrigger { trigger, .. } => {
                format!("duplicate trigger in file: {trigger}")
            }
            Self::InvalidSettings { message, .. } => message.clone(),
            Self::TooManyCursorTokens { trigger, .. } => {
                format!("too many cursor tokens in snippet: {trigger}")
            }
            Self::ShellCmdNotArray { trigger, name, .. } => {
                format!(
                    "shell var `{name}` in snippet `{trigger}` must declare a non-empty cmd array"
                )
            }
            Self::ShellTimeoutInvalid { trigger, name, .. } => {
                format!(
                    "shell var `{name}` in snippet `{trigger}` must declare timeout_ms <= 10000"
                )
            }
            Self::RootEscape { message, .. } => message.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RootDocument {
    version: Option<u32>,
    snippets: Vec<SnippetDocument>,
}

#[derive(Debug, Deserialize)]
struct SnippetDocument {
    trigger: String,
    #[serde(default)]
    trigger_literal: bool,
    replace: String,
    #[serde(default)]
    vars: Vec<VarDecl>,
}

pub fn load_from_root(root: &Path) -> io::Result<LoadResult> {
    let settings = match load_settings(root) {
        Ok(settings) => settings,
        Err(error) => {
            return Ok(LoadResult {
                settings: StoreSettings::default(),
                snippets: Vec::new(),
                errors: vec![error],
            });
        }
    };

    let canonical_root = fs::canonicalize(root)?;
    let mut files = Vec::new();
    let mut errors = Vec::new();
    collect_yaml_files(root, &canonical_root, &mut files, &mut errors)?;
    files.sort();

    let mut snippets = Vec::new();
    let mut trigger_index = HashMap::new();
    let mut duplicate_indexes = HashSet::new();
    let mut duplicate_triggers = HashSet::new();

    for path in files {
        match load_file(root, &path, &settings) {
            Ok(loaded) => {
                for snippet in loaded {
                    if let Some(previous_index) =
                        trigger_index.insert(snippet.trigger.clone(), snippets.len())
                    {
                        duplicate_indexes.insert(previous_index);
                        duplicate_triggers.insert(snippet.trigger.clone());
                        errors.push(LoadError::DuplicateTrigger {
                            path: snippet.source_file.clone(),
                            trigger: snippet.trigger.clone(),
                        });
                    }
                    snippets.push(snippet);
                }
            }
            Err(error) => errors.push(error),
        }
    }

    if !duplicate_indexes.is_empty() {
        snippets = snippets
            .into_iter()
            .enumerate()
            .filter_map(|(index, snippet)| {
                (!duplicate_indexes.contains(&index)
                    && !duplicate_triggers.contains(&snippet.trigger))
                .then_some(snippet)
            })
            .collect();
    }

    Ok(LoadResult {
        settings,
        snippets,
        errors,
    })
}

fn collect_yaml_files(
    root: &Path,
    canonical_root: &Path,
    files: &mut Vec<PathBuf>,
    errors: &mut Vec<LoadError>,
) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            errors.push(LoadError::RootEscape {
                path,
                message: "invalid snippet path: symlink or junction escapes snippets root"
                    .to_string(),
            });
            continue;
        }
        if metadata.is_dir() {
            let canonical_dir = fs::canonicalize(&path)?;
            if path_is_within_root(&canonical_dir, canonical_root) {
                collect_yaml_files(&path, canonical_root, files, errors)?;
            } else {
                errors.push(LoadError::RootEscape {
                    path,
                    message: "invalid snippet path: path escapes snippets root".to_string(),
                });
            }
        } else if is_yaml_file(&path) && !is_settings_file(&path) {
            let canonical_path = fs::canonicalize(&path)?;
            if path_is_within_root(&canonical_path, canonical_root) {
                files.push(path);
            } else {
                errors.push(LoadError::RootEscape {
                    path,
                    message: "invalid snippet path: path escapes snippets root".to_string(),
                });
            }
        }
    }
    Ok(())
}

fn is_yaml_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("yaml" | "yml")
    )
}

fn is_settings_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some(SETTINGS_FILE_NAME)
}

pub(crate) fn settings_path(root: &Path) -> PathBuf {
    root.join(SETTINGS_FILE_NAME)
}

pub(crate) fn load_settings(root: &Path) -> Result<StoreSettings, LoadError> {
    let path = settings_path(root);
    if !path.exists() {
        return Ok(StoreSettings::default());
    }

    let contents = fs::read_to_string(&path).map_err(|error| LoadError::Io {
        path: path.clone(),
        message: error.to_string(),
    })?;
    let settings: StoreSettings =
        serde_yaml::from_str(&contents).map_err(|error| LoadError::Parse {
            path: path.clone(),
            message: error.to_string(),
        })?;

    validate_trigger_prefix(&path, &settings.trigger_prefix)?;
    Ok(settings)
}

fn load_file(
    root: &Path,
    path: &Path,
    settings: &StoreSettings,
) -> Result<Vec<Snippet>, LoadError> {
    let contents = fs::read_to_string(path).map_err(|error| LoadError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let document: RootDocument =
        serde_yaml::from_str(&contents).map_err(|error| LoadError::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

    let version = document.version.ok_or_else(|| LoadError::MissingVersion {
        path: path.to_path_buf(),
    })?;

    if version != 1 {
        return Err(LoadError::UnsupportedVersion {
            path: path.to_path_buf(),
            version,
        });
    }

    let relative_path = path
        .strip_prefix(root)
        .map_err(|_| LoadError::RelativePath {
            path: path.to_path_buf(),
        })?;
    let relative_path = normalize_relative_path(relative_path);

    let mut seen_triggers = HashSet::new();
    let mut snippets = Vec::with_capacity(document.snippets.len());
    for snippet in document.snippets {
        let effective_trigger =
            effective_trigger(&snippet.trigger, snippet.trigger_literal, settings);
        if !seen_triggers.insert(effective_trigger.clone()) {
            return Err(LoadError::DuplicateTrigger {
                path: path.to_path_buf(),
                trigger: effective_trigger,
            });
        }
        validate_snippet(path, &snippet)?;

        snippets.push(Snippet {
            id: format!("{relative_path}::{effective_trigger}"),
            trigger: effective_trigger,
            raw_trigger: snippet.trigger,
            trigger_literal: snippet.trigger_literal,
            replace: snippet.replace,
            vars: snippet.vars,
            source_file: path.to_path_buf(),
        });
    }

    Ok(snippets)
}

fn validate_snippet(path: &Path, snippet: &SnippetDocument) -> Result<(), LoadError> {
    if let Err(CursorTokenError::TooManyTokens) = strip_cursor_token(&snippet.replace) {
        return Err(LoadError::TooManyCursorTokens {
            path: path.to_path_buf(),
            trigger: snippet.trigger.clone(),
        });
    }

    for var in &snippet.vars {
        if var.kind != VarKind::Shell {
            continue;
        }

        if var.cmd.is_empty() {
            return Err(LoadError::ShellCmdNotArray {
                path: path.to_path_buf(),
                trigger: snippet.trigger.clone(),
                name: var.name.clone(),
            });
        }

        if var.timeout_ms.is_none() || var.timeout_ms.unwrap_or_default() > 10_000 {
            return Err(LoadError::ShellTimeoutInvalid {
                path: path.to_path_buf(),
                trigger: snippet.trigger.clone(),
                name: var.name.clone(),
            });
        }
    }

    Ok(())
}

pub(crate) fn validate_trigger_prefix(path: &Path, trigger_prefix: &str) -> Result<(), LoadError> {
    let length = trigger_prefix.chars().count();
    let valid = (1..=3).contains(&length)
        && trigger_prefix
            .chars()
            .all(|ch| !ch.is_alphanumeric() && !ch.is_whitespace());
    if valid {
        Ok(())
    } else {
        Err(LoadError::InvalidSettings {
            path: path.to_path_buf(),
            message:
                "invalid trigger_prefix: must be 1-3 punctuation/symbol characters with no letters, digits, or whitespace"
                    .to_string(),
        })
    }
}

pub(crate) fn effective_trigger(
    trigger: &str,
    trigger_literal: bool,
    settings: &StoreSettings,
) -> String {
    if trigger_literal || trigger.starts_with(&settings.trigger_prefix) {
        trigger.to_string()
    } else {
        format!("{}{}", settings.trigger_prefix, trigger)
    }
}

fn normalize_relative_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn path_is_within_root(path: &Path, root: &Path) -> bool {
    #[cfg(windows)]
    {
        let root = normalized_windows_path(root);
        let path = normalized_windows_path(path);
        path == root || path.starts_with(&format!("{root}/"))
    }

    #[cfg(not(windows))]
    {
        path == root || path.starts_with(root)
    }
}

#[cfg(windows)]
fn normalized_windows_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}
