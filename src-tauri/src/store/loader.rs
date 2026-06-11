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
            | Self::ShellTimeoutInvalid { path, .. } => path,
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
                snippets: Vec::new(),
                errors: vec![error],
            });
        }
    };

    let mut files = Vec::new();
    collect_yaml_files(root, &mut files)?;
    files.sort();

    let mut snippets = Vec::new();
    let mut errors = Vec::new();
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

    Ok(LoadResult { snippets, errors })
}

fn collect_yaml_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_yaml_files(&path, files)?;
        } else if is_yaml_file(&path) && !is_settings_file(&path) {
            files.push(path);
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
