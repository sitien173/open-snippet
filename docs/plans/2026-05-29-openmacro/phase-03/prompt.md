## Original User Request
openmacro Phase 3: real Windows keyboard hook + clipboard-paste injection + pause hotkey + foreground/denylist resets. End-to-end static expansion. No placeholders, no forms, no shell yet (those are Phases 4/6-7/8).

## Phase
Wire `hook/`, `inject/`, and `engine::orchestrator` so a `WH_KEYBOARD_LL` thread feeds keystrokes into the Phase-2 matcher and on a match the injector backspaces + pastes the static `snippet.replace`. Pause via `Ctrl+Alt+Pause` hotkey. Reset on focus change + denylist process names. Smoke test gated `#[cfg(windows)]` + `OPENMACRO_E2E=1`.

## Tasks
- task-1: **`hook/{thread,ring,winevent}`** — Dedicated OS thread runs `SetWindowsHookEx(WH_KEYBOARD_LL)` + `GetMessage` pump. Callback translates VK + modifiers via `ToUnicodeEx`, pushes `HookEvent { Char(c), Backspace, Reset(ResetCause) }` to a lock-free SPSC ring (`rtrb::RingBuffer` or `crossbeam::queue::ArrayQueue`, capacity 1024). Callback does **no** allocation, no logging, no formatting. Detect IME composition / dead keys / Caps and emit `Reset(ImeOrComposition | CapsToggle)`. Public API: `Hook::start() -> (HookHandle, Consumer<HookEvent>)`. Unit-testable parts (ring SPSC ordering, overflow-drops-newest, ResetCause enum): use a fake producer, run on all platforms. The `SetWindowsHookEx` install is `#[cfg(windows)]` only.
- task-2: **`hook::winevent` foreground tracker + denylist** — `SetWinEventHook(EVENT_SYSTEM_FOREGROUND, …)` on the same hook thread (or a sibling thread sharing the message loop). Maintain `ForegroundState { hwnd, exe_basename: String }`. On change → emit `Reset(ForegroundChange)` to the ring. While `exe_basename.to_lowercase()` is in the denylist `{"1password.exe","keepass.exe","keepassxc.exe","bitwarden.exe","lastpass.exe","consent.exe"}`, set an `AtomicBool DENYLISTED` and gate the matcher: matcher consumer drops chars while `DENYLISTED || PAUSED`. Add a `winevent::testing` shim that lets a test inject a foreground-change with mock exe basename — unit test asserts buffer is cleared and `DENYLISTED` flips for `consent.exe`.
- task-3: **`inject/{clipboard,sendinput}`** — Two strategies. `clipboard_paste(text)`: open clipboard, enumerate every present format (`EnumClipboardFormats`), save each (`GetClipboardData` + own copy of bytes), `EmptyClipboard`, set `CF_UNICODETEXT` to `text`, send Ctrl+V via `SendInput`, sleep 10 ms settle, restore every saved format, close clipboard. `sendinput_typeout(text)`: per-character `SendInput` Unicode events. Public `Injector::inject(plan: InjectPlan)` decides: if `text.len() <= 4096` and clipboard lock acquired within 50 ms → paste path; else → sendinput. Always sends `SendInput` Backspace × `trigger_len_chars` first. Suspend matcher (set a `SUSPEND` atomic) for the duration; resume after settle. Unit test: mock `SendInput` via a trait `KeyboardSink` injected into `Injector::new`; verify backspace-then-text order, format save/restore on a real clipboard via `#[cfg(windows)]` test.
- task-4: **`engine::orchestrator` + global pause hotkey + max-length cap** — Tokio task: consume `HookEvent` stream, feed `MatchBuffer` + `Matcher`, on `MatchHit { snippet_id, trigger_len_chars }` look up snippet, drop if `replace.chars().count() > max_expansion_len` (default 32_768), else `Injector::inject(InjectPlan { backspaces: trigger_len_chars, text: replace })`. Register global hotkey `Ctrl+Alt+Pause` via `RegisterHotKey`, listen on the hook thread's message loop, toggle `PAUSED` atomic. Tray Pause/Resume menu also flips this. Smoke test `tests/notepad_smoke.rs` (`#[cfg(windows)]`, `#[ignore]` unless `OPENMACRO_E2E=1`): spawn Notepad via `Command::new("notepad")`, locate its edit control via UI Automation (`uiautomation` crate) or `FindWindowExW` for `Edit`, set foreground, type `;sig\n` using SendInput, assert the edit's value text equals the configured snippet body, restore clipboard, kill Notepad.

## Context
- Cargo.toml: add `windows = "0.58"` (or latest 0.5x) with features `Win32_UI_Input_KeyboardAndMouse`, `Win32_System_DataExchange`, `Win32_System_Memory`, `Win32_UI_WindowsAndMessaging`, `Win32_UI_Accessibility`, `Win32_Foundation`, `Win32_Graphics_Gdi` (for ToUnicodeEx layout). Add `rtrb = "0.3"` (or `crossbeam-queue`). For UIA in the smoke test: `uiautomation = "0.18"` (or `windows::Win32::UI::Accessibility` direct — your call, document in notes).
- Atomics in shared state: `static PAUSED: AtomicBool`, `static SUSPEND: AtomicBool`, `static DENYLISTED: AtomicBool`. **No** `Mutex` on the keystroke hot path.
- `unsafe` blocks: scope narrowly, each gets a one-line `// SAFETY: ...` justification. Clippy with `-D warnings` must pass.
- Phase 2 hot-path audit rule still applies: no `Vec::push`, no `format!`, no `Box::new` in the LL hook callback.

## Files
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/hook/{mod.rs,thread.rs,ring.rs,winevent.rs}`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/inject/{mod.rs,clipboard.rs,sendinput.rs}`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/engine/{mod.rs,orchestrator.rs}`
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/notepad_smoke.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/lib.rs` (start orchestrator on app setup; expose `PAUSED` flag to tray menu)

## Done When
- `cd F:/projects_new/textblaze/openmacro/src-tauri && cargo test --all-features` green (excluding `#[ignore]` tests)
- `cd F:/projects_new/textblaze/openmacro/src-tauri && cargo clippy --all-targets -- -D warnings` clean
- Unit tests: SPSC ring ordering + overflow-drops-newest; denylist matcher (case-insensitive basename); foreground-reset clears buffer; clipboard save/restore covers ≥2 formats in a `#[cfg(windows)]` test; backspace-then-text ordering in injector via mocked `KeyboardSink`
- Source audit: search `src/hook/thread.rs` callback function — assert no `Vec::push`, no `format!`, no `Box::new`. Document the audit in notes.md task-1 Test evidence.
- E2E smoke test compiles and is gated `#[ignore]` / runs only with `OPENMACRO_E2E=1`. Document attempted run result in notes.md task-4 — if not runnable in worker env (no foreground Notepad available), record as deviation.
- Tray Pause/Resume menu (Phase 1 placeholder) now flips `PAUSED` atomic — verify via a quick `#[cfg(windows)]` test or `notes.md` evidence describing the wiring.

## Rules
Follow `F:/projects_new/textblaze/.agents/shared/worker-contract.md` — TDD per task (failing unit test first where logic is testable on all platforms; for Win32 wrappers, write the smallest abstraction that lets you mock the syscall and TDD the abstraction). One commit per task `phase-3.task-<M>: …`. Notes at `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-03/notes.md`. Journal at `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-03/journal.md`.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
