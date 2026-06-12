use std::{
    any::Any,
    backtrace::Backtrace,
    fs,
    path::{Path, PathBuf},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct PanicDumpReport {
    pub timestamp_secs: u64,
    pub thread_name: Option<String>,
    pub location: Option<String>,
    pub payload: String,
    pub backtrace: String,
    pub context: Option<String>,
}

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let report = PanicDumpReport {
            timestamp_secs: current_timestamp_secs(),
            thread_name: thread::current().name().map(str::to_string),
            location: info.location().map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            }),
            payload: payload_from_hook(info),
            backtrace: Backtrace::force_capture().to_string(),
            context: Some("panic hook".to_string()),
        };
        let _ = write_dump(&report);
        default_hook(info);
    }));
}

pub fn write_caught_panic_dump(
    context: &str,
    payload: &(dyn Any + Send),
) -> Result<PathBuf, String> {
    let report = PanicDumpReport {
        timestamp_secs: current_timestamp_secs(),
        thread_name: thread::current().name().map(str::to_string),
        location: None,
        payload: payload_message(payload),
        backtrace: Backtrace::force_capture().to_string(),
        context: Some(context.to_string()),
    };
    write_dump(&report)
}

pub fn write_dump(report: &PanicDumpReport) -> Result<PathBuf, String> {
    write_dump_to_dir(&crashes_root()?, report)
}

pub fn write_dump_to_dir(dir: &Path, report: &PanicDumpReport) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("{}.log", report.timestamp_secs));
    fs::write(&path, format_report(report)).map_err(|error| error.to_string())?;
    Ok(path)
}

pub fn crashes_root() -> Result<PathBuf, String> {
    if let Some(override_root) = std::env::var_os("OPENMACRO_CRASH_DIR") {
        return Ok(PathBuf::from(override_root));
    }

    let Some(config_dir) = dirs::config_dir() else {
        return Err("config directory unavailable".to_string());
    };
    Ok(config_dir.join("openmacro").join("crashes"))
}

pub fn newest_crash_timestamp_after(checkpoint: u64) -> Result<Option<u64>, String> {
    let root = crashes_root()?;
    if !root.exists() {
        return Ok(None);
    }

    let mut newest: Option<u64> = None;
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(timestamp) = stem.parse::<u64>() else {
            continue;
        };
        if timestamp > checkpoint {
            newest = Some(newest.map_or(timestamp, |current| current.max(timestamp)));
        }
    }

    Ok(newest)
}

pub fn path_mtime_secs(path: &Path) -> Option<u64> {
    let metadata = fs::metadata(path).ok()?;
    system_time_to_secs(metadata.modified().ok()?)
}

pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn system_time_to_secs(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|value| value.as_secs())
}

fn payload_from_hook(info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn format_report(report: &PanicDumpReport) -> String {
    format!(
        "timestamp: {}\nthread: {}\ncontext: {}\nlocation: {}\npayload: {}\nbacktrace:\n{}\n",
        report.timestamp_secs,
        report.thread_name.as_deref().unwrap_or("<unnamed>"),
        report.context.as_deref().unwrap_or("<none>"),
        report.location.as_deref().unwrap_or("<unknown>"),
        report.payload,
        report.backtrace
    )
}
