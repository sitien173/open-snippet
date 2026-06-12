# Phase 6 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Added `capture_foreground_with` / `restore_foreground_with` helpers alongside the required no-arg wrappers so the focus path can be tested without real HWNDs.
- Introduced the form module scaffold and the resolver’s optional form-values argument early so the new integration test file could compile against the intended Phase 6 API surface.

### Spec deviations
- none

### Tradeoffs accepted
- The initial `FormRunner` implementation is only a scaffold at this task boundary; it exists so the focus test can compile, while the runner behavior itself is finished in later tasks.

### Assumptions
- A `ForegroundWindow(isize)` wrapper is sufficient for crossing async/task boundaries as long as the production Windows path converts to and from the pointer-based `HWND` in the `windows` crate.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test form_focus` failed because the entire `form` API surface was still a stub and `Resolver::resolve` lacked the new overlay parameter.
- GREEN: `cargo test --test form_focus capture_foreground_returns_registered_value -- --exact` passed after implementing the focus backend/wrappers and wiring the new form module exports.

## Task 2
### Decisions made (not in spec)
- Kept `WindowSink` as a small trait with only `open_form`, and used a separate `restore_on_submit` helper for the focus handoff; that keeps the runner’s responsibility to “open + await submit/cancel” aligned with the spec.
- Used a fixed `"form"` Tauri window label for the single in-flight form window because re-entrancy is explicitly rejected rather than queued.

### Spec deviations
- none

### Tradeoffs accepted
- The default `AppWindowSink` centres the window using monitor work-area math in the Rust backend, but it does not yet close/reuse an existing form window because Phase 6 only requires single-open rejection rather than richer lifecycle management.

### Assumptions
- Treating an unexpected `FormState::Pending` receipt as a cancelled outcome is acceptable defensive behavior since only `Submitted` and `Cancelled` should ever be sent through the oneshot.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: the initial async submit/cancel tests hung because they raced `tokio::spawn` before the runner installed its in-flight sender.
- GREEN: `cargo test --test form_focus` passed after yielding once in the async tests and implementing the actual runner state machine, re-entrancy guard, and submit/cancel signaling.

## Task 3
### Decisions made (not in spec)
- Moved `Injector` behind `Arc<Mutex<_>>` inside the orchestrator so the delayed form-submit path can reuse the exact same injection backend without cloning clipboard or sink state.
- Used `Arc<dyn FocusBackend>` plus a default `NoopFocusBackend` in the orchestrator constructor so non-form tests and code paths stay cheap while the form branch can still be fully mocked.

### Spec deviations
- none

### Tradeoffs accepted
- The async form-submit branch silently drops failures from form running, focus restore, re-resolution, or delayed injection instead of surfacing a user notification; that keeps the branch small in this phase and avoids inventing new UI/error semantics not in spec.

### Assumptions
- If foreground capture fails for a form snippet, the safest behavior is to reset the matcher buffer and skip opening the form rather than injecting or backspacing anything.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --lib --test form_focus` initially failed because the orchestrator still used the old direct-inject path and its unit tests did not supply the required runtime/form/focus dependencies.
- GREEN: `cargo test --lib --test form_focus` passed with `24 passed (2 suites, 4.11s)` after branching form snippets through captured focus + async `FormRunner`, deferring backspaces until submit, and adding the cancel-path no-inject/no-restore assertion.

## Task 4
### Decisions made (not in spec)
- Added a dedicated `commands::form` module with thin `form_submit` / `form_cancel` wrappers so the IPC surface stays symmetric with the existing prefs/snippet command organization.
- Managed a single shared `Arc<FormRunner>` from Tauri startup and reused it both for orchestrator wiring and IPC command state instead of constructing separate runners.

### Spec deviations
- none

### Tradeoffs accepted
- The final validation work also updated existing resolver tests to pass the new optional form-values argument; that is direct fallout from the Phase 6 API change, but it does touch pre-existing test files outside the new form command module.

### Assumptions
- Returning `Result<(), String>` from the form IPC commands is sufficient for the settings/form webview caller because the underlying runner methods are not expected to fail on valid in-flight submit/cancel events.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: the first full `cargo test --all-features` run failed because the resolver signature change left stale call sites in `cursor_math.rs` / `expand_resolver.rs`, and my first cursor fixture rewrite accidentally introduced mojibake that broke the UTF-16 assertion.
- GREEN: `cargo test --all-features` passed with `51 passed, 1 ignored`, and `cargo clippy --all-targets --all-features -- -D warnings` passed after wiring the form commands in `lib.rs`, updating the stale tests, and fixing the cursor fixture with explicit Unicode escapes.
