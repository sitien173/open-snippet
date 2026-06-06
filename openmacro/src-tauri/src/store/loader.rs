//! Recursive YAML loader for snippet files.

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::expand::{strip_cursor_token, CursorTokenError};

use super::{Snippet, VarDecl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadResult {
    pub snippets: Vec<Snippet>,
    pub errors: Vec<LoadError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
    MissingVersion { path: PathBuf },
    UnsupportedVersion { path: PathBuf, version: u32 },
    RelativePath { path: PathBuf },
    DuplicateTrigger { path: PathBuf, trigger: String },
    TooManyCursorTokens { path: PathBuf, trigger: String },
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
            | Self::TooManyCursorTokens { path, .. } => path,
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
            Self::TooManyCursorTokens { trigger, .. } => {
                format!("too many cursor tokens in snippet: {trigger}")
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
    replace: String,
    #[serde(default)]
    vars: Vec<VarDecl>,
}

pub fn load_from_root(root: &Path) -> io::Result<LoadResult> {
    let mut files = Vec::new();
    collect_yaml_files(root, &mut files)?;
    files.sort();

    let mut snippets = Vec::new();
    let mut errors = Vec::new();

    for path in files {
        match load_file(root, &path) {
            Ok(mut loaded) => snippets.append(&mut loaded),
            Err(error) => errors.push(error),
        }
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
        } else if is_yaml_file(&path) {
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

fn load_file(root: &Path, path: &Path) -> Result<Vec<Snippet>, LoadError> {
    let contents = fs::read_to_string(path).map_err(|error| LoadError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let document: RootDocument = serde_yaml::from_str(&contents).map_err(|error| LoadError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let version = document
        .version
        .ok_or_else(|| LoadError::MissingVersion {
            path: path.to_path_buf(),
        })?;

    if version != 1 {
        return Err(LoadError::UnsupportedVersion {
            path: path.to_path_buf(),
            version,
        });
    }

    let relative_path = path.strip_prefix(root).map_err(|_| LoadError::RelativePath {
        path: path.to_path_buf(),
    })?;
    let relative_path = normalize_relative_path(relative_path);

    let mut seen_triggers = HashSet::new();
    let mut snippets = Vec::with_capacity(document.snippets.len());
    for snippet in document.snippets {
        if !seen_triggers.insert(snippet.trigger.clone()) {
            return Err(LoadError::DuplicateTrigger {
                path: path.to_path_buf(),
                trigger: snippet.trigger,
            });
        }
        validate_snippet(path, &snippet)?;

        snippets.push(Snippet {
            id: format!("{relative_path}::{}", snippet.trigger),
            trigger: snippet.trigger,
            replace: snippet.replace,
            vars: snippet.vars,
            source_file: path.to_path_buf(),
        });
    }

    Ok(snippets)
}

fn validate_snippet(path: &Path, snippet: &SnippetDocument) -> Result<(), LoadError> {
    match strip_cursor_token(&snippet.replace) {
        Ok(_) => Ok(()),
        Err(CursorTokenError::TooManyTokens) => Err(LoadError::TooManyCursorTokens {
            path: path.to_path_buf(),
            trigger: snippet.trigger.clone(),
        }),
    }
}

fn normalize_relative_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
