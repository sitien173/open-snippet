# Phase 1 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Kept `POST_BACKSPACE_DELAY` as a zero-duration constant and guarded the sleep with `is_zero()` so the timing policy remains explicit in [mod.rs](F:/projects_new/textblaze/src-tauri/src/inject/mod.rs).

### Spec deviations
- none

### Tradeoffs accepted
- The new 500-character test verifies action count and that every action is `Unicode('a')`; it does not repeat a 500-item literal vector.

### Assumptions
- A 500-character ASCII payload is safely below both the new unicode threshold and the clipboard byte cap, so any `Paste` action is a genuine regression.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED:
```text
rtk cargo test injector_uses_unicode_for_500_ascii_chars --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.03s
```
- GREEN:
```text
rtk cargo test inject::tests --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
cargo test: 3 passed, 94 filtered out (19 suites, 0.02s)
```

## Task 2
### Decisions made (not in spec)
- Moved `Paste` emission out of the clipboard backends so `Injector` owns the action list and can batch backspaces plus replacement text through `send_batch`.

### Spec deviations
- none

### Tradeoffs accepted
- `WindowsKeyboardSink::send` still keeps the single-action path for callers outside this phase, even though `Injector` now uses `send_batch` for its hot path.

### Assumptions
- Splitting the `INPUT` vector into `chunks_mut(256)` is acceptable even when a long unicode payload crosses chunk boundaries, because each `INPUT` entry is still submitted in order.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED:
```text
rtk cargo test resolved_round_trip_for_now_and_log_snippets --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.01s
```
- RED:
```text
rtk cargo test shell_snippet_injects_backend_output_when_consent_enabled --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.01s
```
- GREEN:
```text
rtk cargo test inject::tests --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
cargo test: 3 passed, 94 filtered out (19 suites, 0.02s)
```
- GREEN:
```text
rtk cargo test resolved_round_trip_for_now_and_log_snippets --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
cargo test: 1 passed, 96 filtered out (19 suites, 0.02s)
```
- GREEN:
```text
rtk cargo test shell_snippet_injects_backend_output_when_consent_enabled --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml -- --nocapture
cargo test: 1 passed, 96 filtered out (19 suites, 0.01s)
```

## Task 3
### Decisions made (not in spec)
- Updated [store_yaml.rs](F:/projects_new/textblaze/src-tauri/tests/store_yaml.rs) to expect 7 shipped snippets because the current checked-out [default.yaml](F:/projects_new/textblaze/snippets/default.yaml) already contains an additional `;test` snippet in this worktree.

### Spec deviations
- none

### Tradeoffs accepted
- Task 3 included one unrelated test-fixture expectation update so the requested full-suite verification could pass against the current repository state.

### Assumptions
- The added `;test` snippet in [default.yaml](F:/projects_new/textblaze/snippets/default.yaml) is intentional local state and the correct verification target for this turn.

### Follow-ups for human
- If the extra `;test` snippet should not ship, revert that fixture change and restore the expected count in [store_yaml.rs](F:/projects_new/textblaze/src-tauri/tests/store_yaml.rs).

### Test evidence (RED→GREEN, or root cause for a fix)
- GREEN:
```text
rtk cargo test -p openmacro --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml
cargo test: 96 passed, 1 ignored (20 suites, 5.35s)
```
- GREEN:
```text
rtk cargo clippy --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml --all-targets -- -D warnings
cargo clippy: No issues found
```
