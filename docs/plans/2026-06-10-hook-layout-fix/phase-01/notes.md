# Phase 1 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Kept `translate_with_layout` in [thread.rs](F:/projects_new/textblaze/src-tauri/src/hook/thread.rs) instead of adding a new sibling module, because the logic is only used by the keyboard hook and is already unit-testable there.

### Spec deviations
- none

### Tradeoffs accepted
- The en-US layout tests skip early if `LoadKeyboardLayoutW("00000409", KLF_NOTELLSHELL)` fails on the local machine instead of hard-failing for environment setup outside the phase scope.

### Assumptions
- `scan_code = 0` is sufficient for the ASCII `VK_A` coverage requested by the spec.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED:
```text
rtk cargo test translate_with_layout -- --nocapture
error[E0433]: cannot find `TextServices` in `UI`
error[E0432]: unresolved import `windows::Win32::UI::TextServices`
error[E0609]: no field `0` on type `Result<HKL, windows_result::error::Error>`
```
- RED:
```text
rtk cargo test translate_with_layout -- --nocapture
error[E0308]: mismatched types
164 | translate_with_layout(VK_A.0 as u32, 0, HKL(0), &key_state),
    |                                         ^ expected `*mut c_void`, found `usize`
```
- GREEN:
```text
rtk cargo test translate_with_layout -- --nocapture
cargo test: 3 passed, 93 filtered out (19 suites, 0.00s)
```

## Task 2
### Decisions made (not in spec)
- Logged the HKL and codepoint through small `Display` wrappers to avoid heap allocation in the low-level hook callback while still emitting the exact formats the spec requested.

### Spec deviations
- none

### Tradeoffs accepted
- Kept the TRACE event at the `translate_key` site rather than at the push callsite, because this is still inside the OS callback path and keeps the layout, translation outcome, and logging in one place.

### Assumptions
- `GetForegroundWindow()` returning a null `HWND` is the only invalid-window case that needs explicit fallback handling here.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- GREEN regression check:
```text
rtk cargo test translate_with_layout -- --nocapture
cargo test: 3 passed, 93 filtered out (19 suites, 0.00s)
```

## Task 3
### Decisions made (not in spec)
- Ran the requested root-level verification with `--manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml` because [Cargo.toml](F:/projects_new/textblaze/src-tauri/Cargo.toml) is not at the repository root.

### Spec deviations
- `cargo test -p openmacro` did not pass in the current worktree; failures are in pre-existing modified files outside this phase.

### Tradeoffs accepted
- No files touched, verification only.

### Assumptions
- The failing injection/orchestrator assertions are unrelated to this phase because they point at already-dirty files outside [thread.rs](F:/projects_new/textblaze/src-tauri/src/hook/thread.rs).

### Follow-ups for human
- Re-run `rtk cargo test -p openmacro --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml` after resolving the existing failures in [inject/mod.rs](F:/projects_new/textblaze/src-tauri/src/inject/mod.rs) and [engine/orchestrator.rs](F:/projects_new/textblaze/src-tauri/src/engine/orchestrator.rs).

### Test evidence (RED→GREEN, or root cause for a fix)
- Verification result:
```text
rtk cargo test -p openmacro --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml
test result: FAILED. 28 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.23s

engine::orchestrator::tests::resolved_round_trip_for_now_and_log_snippets
engine::orchestrator::tests::shell_snippet_injects_backend_output_when_consent_enabled
inject::tests::injector_sends_backspace_then_text_with_mocked_sink
```
- Verification result:
```text
rtk cargo clippy --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml --all-targets -- -D warnings
cargo clippy: No issues found
```
