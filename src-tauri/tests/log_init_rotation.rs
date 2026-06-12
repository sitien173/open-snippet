use std::{fs, sync::Mutex};

use tempfile::TempDir;

use openmacro_lib::log_init::rotation::prune_old_logs;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn prune_old_logs_keeps_newest_matching_files_only() {
    let _guard = ENV_LOCK.lock().unwrap();
    let old_log_dir = std::env::var_os("OPENMACRO_LOG_DIR");
    let temp = TempDir::new().unwrap();
    std::env::set_var("OPENMACRO_LOG_DIR", temp.path());

    for day in 1..=7 {
        fs::write(
            temp.path().join(format!("openmacro.log.2026-06-0{day}")),
            day.to_string(),
        )
        .unwrap();
    }
    fs::write(temp.path().join("random.txt"), "keep").unwrap();

    prune_old_logs(3).unwrap();

    let mut remaining = fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    remaining.sort();

    assert_eq!(
        remaining,
        vec![
            "openmacro.log.2026-06-05".to_string(),
            "openmacro.log.2026-06-06".to_string(),
            "openmacro.log.2026-06-07".to_string(),
            "random.txt".to_string(),
        ]
    );

    match old_log_dir {
        Some(value) => std::env::set_var("OPENMACRO_LOG_DIR", value),
        None => std::env::remove_var("OPENMACRO_LOG_DIR"),
    }
}
