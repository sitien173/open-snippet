# Phase 3 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Used `rtrb::RingBuffer` with thin `HookProducer` / `HookConsumer` wrappers so the overflow semantics are explicit and testable without pulling the full hook thread into unit tests.

### Spec deviations
- none

### Tradeoffs accepted
- The non-Windows `Hook::start()` path is a no-op compatibility shim that immediately signals readiness and pushes a reset marker; this keeps all-platform tests compiling while the real hook install remains Windows-only.

### Assumptions
- Returning `bool` from `HookProducer::push` is sufficient for the overflow-drop-newest contract because the callback only needs a cheap success/failure signal and never retries on overflow.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test overflow_drops_newest` initially failed to compile due to the incorrect `GetKeyboardLayout` import and a moved `Result` in the hook-thread readiness handshake.
- GREEN: `cargo test spsc_preserves_event_order`
- GREEN: `cargo test overflow_drops_newest`
- Source audit: `src/hook/thread.rs` callback path contains no `Vec::push`, no `format!`, and no `Box::new`.

## Task 2
### Decisions made (not in spec)
- Centralized the denylist state in `hook::winevent::DENYLISTED` with pure helper functions (`is_denylisted_process`, `apply_foreground_change`) so the consumer-side gating logic can reuse the same state without needing a live WinEvent hook in unit tests.

### Spec deviations
- none

### Tradeoffs accepted
- The real `SetWinEventHook` install is still deferred to the later thread/orchestrator wiring; Task 2 implements and tests the foreground-reset and denylist semantics behind that hook first.

### Assumptions
- Denylist matching is done against the lowercase executable basename exactly as the spec states, so path components and mixed casing are normalized before comparison.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test foreground_change_to_denylisted_process_clears_buffer_and_sets_gate` failed with unresolved `winevent::testing` and `is_denylisted` symbols because the foreground shim and gate state did not exist yet.
- GREEN: `cargo test foreground_change_to_denylisted_process_clears_buffer_and_sets_gate`
- GREEN: `cargo test non_denylisted_foreground_change_clears_buffer_without_gate`

## Task 3
### Decisions made (not in spec)
- Split injector responsibilities into a `KeyboardSink` plus `ClipboardBackend` so the mocked ordering test can stay deterministic while the Windows path still exercises real clipboard capture/restore.
- `set_clipboard_text` seeds both `CF_UNICODETEXT` and `CF_TEXT`, which gives the Windows round-trip test a concrete multi-format clipboard to verify.

### Spec deviations
- none

### Tradeoffs accepted
- The current Windows paste path snapshots/restores clipboard formats via generic global-memory byte copies; this is broad enough for the tested formats without adding format-specific decoding branches beyond the UTF-16 helper used for assertions.

### Assumptions
- Using `plan.text.len()` as the 4096-byte clipboard cutoff is acceptable for this phase because the static expansion bodies under test are ASCII; Phase 4+ can tighten this to exact encoded byte length if needed.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test injector_sends_backspace_then_text_with_mocked_sink` failed with unresolved injector and clipboard helper symbols because the injection surface did not exist yet.
- GREEN: `cargo test injector_sends_backspace_then_text_with_mocked_sink`
- GREEN: `cargo test clipboard_snapshot_restores_multiple_formats -- --nocapture`

## Task 4
### Decisions made (not in spec)
- Kept the runtime bootstrap intentionally minimal: `engine::start_runtime()` now exposes a stable setup hook and pause state while the real long-lived hook/runtime ownership remains isolated from the pure orchestration logic and tests.

### Spec deviations
- E2E deviation: `tests/notepad_smoke.rs` compiles as an ignored `OPENMACRO_E2E=1`-gated Windows smoke scaffold, but it was not runnable in the worker flow here because reliable foreground Notepad/UIAutomation interaction is not available in this session.

### Tradeoffs accepted
- The current orchestrator integration is logic-complete for pause/denylist/reset/max-length gating and injector ordering, but the full app-runtime hook ownership is still conservative rather than aggressively auto-starting a long-lived Win32 pump from Tauri setup.

### Assumptions
- Tray `Pause/Resume` wiring is satisfied by flipping `engine::PAUSED` through `engine::toggle_paused()` in `lib.rs`, with the orchestrator tests covering the atomic gate behavior itself.

### Follow-ups for human
- If Phase 3 requires a true manual end-to-end desktop demo before Phase 4, rerun the ignored smoke test in an interactive Windows desktop session with `OPENMACRO_E2E=1`.

### Test evidence
- GREEN: `cargo test --all-features`
- GREEN: `cargo clippy --all-targets -- -D warnings`
- Wiring evidence: `pause_toggle_flips_atomic_state` and `paused_orchestrator_drops_char_input` cover the `PAUSED` gate semantics used by the tray handler.
- Smoke evidence: `tests/notepad_smoke.rs` compiles and is `#[ignore]`; not executed in this worker environment.
