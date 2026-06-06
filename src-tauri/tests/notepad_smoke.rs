#[cfg(windows)]
#[test]
#[ignore]
fn notepad_smoke() {
    if std::env::var("OPENMACRO_E2E").ok().as_deref() != Some("1") {
        return;
    }

    let mut child = std::process::Command::new("notepad")
        .spawn()
        .expect("failed to spawn notepad");

    let _ = child.kill();
    let _ = child.wait();
}
