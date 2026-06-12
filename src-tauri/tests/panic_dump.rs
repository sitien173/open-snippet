use openmacro_lib::crash::{write_dump_to_dir, PanicDumpReport};
use tempfile::TempDir;

#[test]
fn write_dump_to_dir_creates_crash_log() {
    let root = TempDir::new().unwrap();
    let report = PanicDumpReport {
        timestamp_secs: 1_717_171_717,
        thread_name: Some("panic-test".to_string()),
        location: Some("src/test.rs:10:2".to_string()),
        payload: "boom".to_string(),
        backtrace: "stack".to_string(),
        context: Some("panic dump test".to_string()),
    };

    let path = write_dump_to_dir(root.path(), &report).unwrap();
    let contents = std::fs::read_to_string(path).unwrap();

    assert!(contents.contains("payload: boom"));
    assert!(contents.contains("thread: panic-test"));
}
