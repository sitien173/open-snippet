#[cfg(windows)]
#[test]
#[ignore]
fn notepad_smoke() {
    use openmacro_lib::{
        engine::Orchestrator,
        form::{capture_foreground, restore_foreground, ForegroundWindow},
        hook::HookEvent,
        inject::{
            clipboard::{capture_clipboard, set_clipboard_text, SystemClipboardBackend},
            sendinput::WindowsKeyboardSink,
            Injector, KeyboardAction, KeyboardSink,
        },
        store::{ExpandMode, Snippet},
    };
    use std::{
        collections::BTreeSet,
        path::PathBuf,
        process::Child,
        thread,
        time::{Duration, Instant},
    };
    use windows::Win32::{
        Foundation::{BOOL, HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            EnumChildWindows, EnumWindows, GetWindowTextLengthW, GetWindowTextW,
            GetWindowThreadProcessId, IsWindowVisible, SendMessageW, WM_GETTEXT, WM_GETTEXTLENGTH,
        },
    };

    if std::env::var("OPENMACRO_E2E").ok().as_deref() != Some("1") {
        return;
    }

    const CLIPBOARD_SENTINEL: &str = "phase5-before";
    let long_replacement = "phase5-long-".repeat(300);

    run_trigger_case(";ok", "phase5-short", CLIPBOARD_SENTINEL, false);
    run_trigger_case(";long", &long_replacement, CLIPBOARD_SENTINEL, true);

    fn run_trigger_case(
        trigger: &str,
        replacement: &str,
        clipboard_sentinel: &str,
        expect_clipboard_restore: bool,
    ) {
        set_clipboard_text(clipboard_sentinel).expect("failed to seed clipboard");
        let notepad = NotepadGuard::spawn();
        notepad.focus();
        let _runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build runtime");
        let mut orchestrator = Orchestrator::<WindowsKeyboardSink, SystemClipboardBackend>::new(
            vec![snippet(trigger, replacement)],
            Injector::new(),
            _runtime.handle().clone(),
        );
        orchestrator.set_expand_mode(ExpandMode::Auto);

        type_trigger_and_expand(&mut orchestrator, trigger);

        if expect_clipboard_restore {
            let restored = wait_for_clipboard_text(clipboard_sentinel, Duration::from_secs(5));
            assert_eq!(restored.as_deref(), Some(clipboard_sentinel));
        }

        let observed = wait_for_notepad_text(notepad.hwnd, replacement, Duration::from_secs(5));
        assert_eq!(observed.as_deref(), Some(replacement));
    }

    fn snippet(trigger: &str, replace: &str) -> Snippet {
        Snippet {
            id: format!("e2e::{trigger}"),
            trigger: trigger.to_string(),
            raw_trigger: trigger.to_string(),
            trigger_literal: false,
            replace: replace.to_string(),
            vars: Vec::new(),
            source_file: PathBuf::from("notepad_smoke.yaml"),
        }
    }

    fn type_trigger_and_expand(
        orchestrator: &mut Orchestrator<WindowsKeyboardSink, SystemClipboardBackend>,
        trigger: &str,
    ) {
        let mut sink = WindowsKeyboardSink;
        for ch in trigger.chars() {
            sink.send(KeyboardAction::Unicode(ch));
            thread::sleep(Duration::from_millis(20));
            let _ = orchestrator
                .handle_event(HookEvent::Char(ch))
                .expect("failed to handle trigger char");
        }
        thread::sleep(Duration::from_millis(300));
    }

    fn wait_for_clipboard_text(expected: &str, timeout: Duration) -> Option<String> {
        let started = Instant::now();
        let mut last = None;
        while started.elapsed() < timeout {
            if let Some(text) = capture_clipboard()
                .ok()
                .and_then(|snapshot| snapshot.text_content())
            {
                last = Some(text);
            }
            if last.as_deref() == Some(expected) {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        last
    }

    struct NotepadGuard {
        child: Child,
        hwnd: ForegroundWindow,
    }

    impl NotepadGuard {
        fn spawn() -> Self {
            let existing_windows = visible_top_level_windows();
            let child = std::process::Command::new("notepad")
                .spawn()
                .expect("failed to spawn notepad");
            let hwnd =
                wait_for_notepad_window(child.id(), &existing_windows, Duration::from_secs(5))
                    .expect("failed to locate notepad window");
            let _ = restore_foreground(hwnd);
            thread::sleep(Duration::from_millis(300));
            Self { child, hwnd }
        }

        fn focus(&self) {
            let _ = restore_foreground(self.hwnd);
            thread::sleep(Duration::from_millis(150));
        }
    }

    impl Drop for NotepadGuard {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn wait_for_notepad_window(
        pid: u32,
        existing_windows: &BTreeSet<isize>,
        timeout: Duration,
    ) -> Option<ForegroundWindow> {
        let started = Instant::now();
        while started.elapsed() < timeout {
            if let Some(hwnd) = find_window_for_pid_or_new_notepad(pid, existing_windows) {
                return Some(ForegroundWindow(hwnd.0 as isize));
            }
            if let Some(hwnd) = capture_foreground().filter(|hwnd| {
                let raw = hwnd.0;
                !existing_windows.contains(&raw)
                    && is_notepad_title(HWND(raw as *mut core::ffi::c_void))
            }) {
                return Some(hwnd);
            }
            thread::sleep(Duration::from_millis(50));
        }
        None
    }

    fn find_window_for_pid_or_new_notepad(
        pid: u32,
        existing_windows: &BTreeSet<isize>,
    ) -> Option<HWND> {
        struct EnumState {
            pid: u32,
            existing_windows: BTreeSet<isize>,
            hwnd: Option<HWND>,
        }

        unsafe extern "system" fn enum_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let state = unsafe { &mut *(lparam.0 as *mut EnumState) };
            let visible = unsafe {
                // SAFETY: hwnd is provided by EnumWindows.
                IsWindowVisible(hwnd).as_bool()
            };
            if !visible {
                return BOOL(1);
            }

            let mut window_pid = 0u32;
            unsafe {
                // SAFETY: hwnd is provided by EnumWindows and window_pid points to valid memory.
                let _ = GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
            }
            if window_pid == state.pid {
                state.hwnd = Some(hwnd);
                return BOOL(0);
            }
            if !state.existing_windows.contains(&(hwnd.0 as isize)) && is_notepad_title(hwnd) {
                state.hwnd = Some(hwnd);
                return BOOL(0);
            }

            BOOL(1)
        }

        let mut state = EnumState {
            pid,
            existing_windows: existing_windows.clone(),
            hwnd: None,
        };
        unsafe {
            // SAFETY: callback and state pointer remain valid for the duration of EnumWindows.
            let _ = EnumWindows(
                Some(enum_windows),
                LPARAM((&mut state as *mut EnumState) as isize),
            );
        }
        state.hwnd
    }

    fn visible_top_level_windows() -> BTreeSet<isize> {
        struct EnumState(BTreeSet<isize>);

        unsafe extern "system" fn enum_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let visible = unsafe {
                // SAFETY: hwnd is provided by EnumWindows.
                IsWindowVisible(hwnd).as_bool()
            };
            if visible {
                let state = unsafe { &mut *(lparam.0 as *mut EnumState) };
                state.0.insert(hwnd.0 as isize);
            }
            BOOL(1)
        }

        let mut state = EnumState(BTreeSet::new());
        unsafe {
            // SAFETY: callback and state pointer remain valid for the duration of EnumWindows.
            let _ = EnumWindows(
                Some(enum_windows),
                LPARAM((&mut state as *mut EnumState) as isize),
            );
        }
        state.0
    }

    fn is_notepad_title(hwnd: HWND) -> bool {
        let len = unsafe {
            // SAFETY: hwnd is an opaque top-level window handle from EnumWindows.
            GetWindowTextLengthW(hwnd)
        };
        if len <= 0 {
            return false;
        }

        let mut buffer = vec![0u16; len as usize + 1];
        let written = unsafe {
            // SAFETY: buffer is valid writable memory and hwnd is an opaque top-level handle.
            GetWindowTextW(hwnd, &mut buffer)
        };
        if written <= 0 {
            return false;
        }

        let title = String::from_utf16_lossy(&buffer[..written as usize]);
        title.contains("Notepad")
    }

    fn wait_for_notepad_text(
        hwnd: ForegroundWindow,
        expected: &str,
        timeout: Duration,
    ) -> Option<String> {
        let started = Instant::now();
        let mut last = None;
        while started.elapsed() < timeout {
            last = find_notepad_text(hwnd, expected);
            if last.as_deref() == Some(expected) {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        last
    }

    fn find_notepad_text(hwnd: ForegroundWindow, expected: &str) -> Option<String> {
        struct EnumState(Vec<String>);

        unsafe extern "system" fn enum_children(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let state = unsafe { &mut *(lparam.0 as *mut EnumState) };
            if let Some(text) = read_window_text(hwnd) {
                state.0.push(text);
            }
            BOOL(1)
        }

        let mut state = EnumState(Vec::new());
        let root = HWND(hwnd.0 as *mut core::ffi::c_void);
        unsafe {
            // SAFETY: callback and state pointer remain valid for the duration of EnumChildWindows.
            let _ = EnumChildWindows(
                root,
                Some(enum_children),
                LPARAM((&mut state as *mut EnumState) as isize),
            );
        }
        state.0.into_iter().find(|text| text.contains(expected))?;
        Some(expected.to_string())
    }

    fn read_window_text(hwnd: HWND) -> Option<String> {
        let len = unsafe {
            // SAFETY: hwnd is an opaque child-window handle returned by EnumChildWindows.
            SendMessageW(hwnd, WM_GETTEXTLENGTH, WPARAM(0), LPARAM(0)).0
        };
        if len <= 0 {
            return None;
        }

        let mut buffer = vec![0u16; len as usize + 1];
        let written = unsafe {
            // SAFETY: buffer is valid writable memory and hwnd is a child window handle.
            SendMessageW(
                hwnd,
                WM_GETTEXT,
                WPARAM(buffer.len()),
                LPARAM(buffer.as_mut_ptr() as isize),
            )
            .0
        };
        if written <= 0 {
            return None;
        }

        Some(String::from_utf16_lossy(&buffer[..written as usize]))
    }
}
