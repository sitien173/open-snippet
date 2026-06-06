use std::{fs, path::{Path, PathBuf}};

use git2::{Index, Repository};

use super::{SyncError, SyncResult};

pub fn write_conflicts(repo: &Repository, conflict_root: &Path) -> SyncResult<()> {
    fs::create_dir_all(conflict_root).map_err(|error| SyncError::State(error.to_string()))?;
    let index = repo.index()?;
    for conflict in conflict_entries(&index)? {
        if let Some(parent) = conflict_root.join(&conflict).parent() {
            fs::create_dir_all(parent).map_err(|error| SyncError::State(error.to_string()))?;
        }
        let source = repo.workdir().unwrap().join(&conflict);
        if source.exists() {
            fs::copy(&source, conflict_root.join(&conflict))
                .map_err(|error| SyncError::State(error.to_string()))?;
        }
    }
    Ok(())
}

fn conflict_entries(index: &Index) -> SyncResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for conflict in index.conflicts()? {
        let conflict = conflict?;
        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|entry| PathBuf::from(String::from_utf8_lossy(&entry.path).to_string()))
            .ok_or_else(|| SyncError::State("conflict entry missing path".to_string()))?;
        paths.push(path);
    }
    Ok(paths)
}
