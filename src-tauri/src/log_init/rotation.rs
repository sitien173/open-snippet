use std::{fs, io, path::Path};

use super::log_dir;

pub fn prune_old_logs(max_files: usize) -> io::Result<()> {
    prune_old_logs_in_dir(&log_dir(), max_files)
}

pub fn prune_old_logs_in_dir(dir: &Path, max_files: usize) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| is_rotated_log_file(&entry.path()))
        .collect::<Vec<_>>();
    files.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));

    for entry in files.into_iter().skip(max_files) {
        fs::remove_file(entry.path())?;
    }

    Ok(())
}

fn is_rotated_log_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("openmacro.log."))
        .unwrap_or(false)
}
