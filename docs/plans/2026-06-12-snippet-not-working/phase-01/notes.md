# Phase 01 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Restored the pre-phase-1 suspend gate in `should_ignore_event` while keeping the marker-specific ignore path for injected events outside an active injection window.

### Spec deviations
- none

### Tradeoffs accepted
- The fix intentionally keeps ignoring all injected keydowns during `SUSPEND` instead of trying to distinguish app-injected versus third-party injected events in that window, because the regression was caused by removing that broader gate.

### Assumptions
- The leaked `HookEvent::Backspace` and `vk_code=231` logs are injected keydowns that re-entered the low-level hook while `SUSPEND` was active.

### Follow-ups for human
- Manual GUI smoke in `pnpm tauri dev` is still needed to confirm the real application no longer logs queued backspace / `vk_code=231` during expansion.

### Test evidence (RED→GREEN, or root cause for a fix)
- Root cause evidence: `git show 1f17fec:src-tauri/src/hook/thread.rs` ignored any injected keydown when `SUSPEND` was true, but current `should_ignore_event` stopped using `suspend` and only ignored events with `dwExtraInfo == INJECTED_MARKER`.
- RED: `rtk cargo test --lib should_ignore_event_logic --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture` failed after the test was updated to require `should_ignore_event(true, LLKHF_INJECTED_FLAG, 0)`.
- GREEN: the same `cargo test --lib should_ignore_event_logic ...` command passed after restoring the suspend gate.
- GREEN: `rtk cargo test --test matcher_armed_dismiss --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture` passed (3 tests).
