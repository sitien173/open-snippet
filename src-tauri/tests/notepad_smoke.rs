#[cfg(windows)]
#[test]
#[ignore]
fn notepad_smoke() {
    use openmacro_lib::inject::{
        clipboard::{capture_clipboard, set_clipboard_text},
        InjectPlan, Injector,
    };
    use std::{thread, time::Duration};

    if std::env::var("OPENMACRO_E2E").ok().as_deref() != Some("1") {
        return;
    }

    set_clipboard_text("phase2-before").expect("failed to seed clipboard");
    let mut child = std::process::Command::new("notepad")
        .spawn()
        .expect("failed to spawn notepad");
    thread::sleep(Duration::from_millis(250));

    let mut injector = Injector::new();
    injector
        .inject(InjectPlan {
            backspaces: 0,
            text: "phase2-long-".repeat(300),
            caret_left: 0,
            max_clipboard_bytes: 32_768,
            clipboard_timeout: Duration::from_secs(1),
        })
        .expect("failed to inject long replacement");
    thread::sleep(Duration::from_millis(250));

    let restored = capture_clipboard()
        .expect("failed to capture clipboard")
        .text_content();

    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(restored.as_deref(), Some("phase2-before"));
}
