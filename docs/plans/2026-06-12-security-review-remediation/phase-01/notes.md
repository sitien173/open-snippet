# Phase 1 - Decision Notes

## Task 1

### Decisions made (not in spec)
- Added save-path regression coverage in `src-tauri/tests/commands_roundtrip.rs`.
- Added loader symlink escape regression coverage in `src-tauri/tests/store_yaml.rs`.

### Spec deviations
- none

### Tradeoffs accepted
- Symlink tests skip only if the OS refuses to create test symlinks, to avoid requiring elevated Windows privileges.

### Assumptions
- Existing `save_snippet_inner` helpers are the correct backend IPC seam for path validation tests.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test --test commands_roundtrip -- --test-threads=1` failed 4 new save-path tests because invalid paths returned `Ok(())`.
- RED: `rtk cargo test --test store_yaml -- --test-threads=1` failed `loader_rejects_yaml_symlink_target_outside_root` because the linked outside snippet loaded.
- GREEN after Task 2: `rtk cargo test --test commands_roundtrip -- --test-threads=1` passed 11 tests.
- GREEN after Task 2: `rtk cargo test --test store_yaml -- --test-threads=1` passed 13 tests.

## Task 2

### Decisions made (not in spec)
- Chose the conservative loader policy: symlinked snippet files/directories are rejected instead of selectively allowed.
- Save path validation canonicalizes the root and parent/target path before read/write and rejects reserved settings and non-YAML targets.

### Spec deviations
- none

### Tradeoffs accepted
- New snippet files still require an existing valid parent, matching current `save_snippet_inner` behavior that reads the target document before writing it.

### Assumptions
- Lowercase `.yaml` and `.yml` remain the accepted snippet extensions, matching the existing loader.

### Follow-ups for human
- none

### Test evidence
- GREEN: `rtk cargo test --test commands_roundtrip -- --test-threads=1` passed 11 tests.
- GREEN: `rtk cargo test --test store_yaml -- --test-threads=1` passed 13 tests.

## Task 3

### Decisions made (not in spec)
- Added shared HTTPS host validation in `src-tauri/src/sync/creds.rs`.
- Enforced host equality before PAT storage and again inside the credential callback before secret lookup.

### Spec deviations
- none

### Tradeoffs accepted
- Enforced host equality, not exact remote equality, per the cross-validation resolution.

### Assumptions
- HTTPS PAT auth is valid only for `https://` remotes with a parseable host.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test --test sync_roundtrip -- --test-threads=1` failed to compile because `validate_credential_callback_url` did not exist.
- GREEN: `rtk cargo test --test sync_roundtrip -- --test-threads=1` passed 3 tests.

## Task 4

### Decisions made (not in spec)
- Chose `max_expansion_len` bounds of `1..=262144`, preserving the existing default of `32768`.
- Added an `AutostartController` test seam. The Tauri command path uses the Tauri autostart plugin; non-command helper calls use a no-op controller unless tests provide one.
- Autostart enable/disable is applied before prefs are persisted; plugin failure leaves memory and JSON unchanged.

### Spec deviations
- none

### Tradeoffs accepted
- `set_prefs_inner` remains usable for non-command internal updates without touching OS autostart; the Tauri command path is the OS-applying path.

### Assumptions
- Existing crash-recovery internal prefs writes should not re-apply autostart when the autostart value is unchanged.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test --test prefs_roundtrip -- --test-threads=1` failed to compile because the prefs bounds/autostart seam did not exist.
- GREEN: `rtk cargo test --test prefs_roundtrip -- --test-threads=1` passed 4 tests.
- GREEN: `rtk cargo test --test commands_roundtrip --test sync_roundtrip --test prefs_roundtrip --test store_yaml -- --test-threads=1` passed 31 tests.
- GREEN: `rtk cargo test --all-features -- --test-threads=1` passed 137 tests, 1 ignored.
- RED: `rtk cargo clippy --all-targets --all-features -- -D warnings` initially failed on pre-existing `clippy::too_many_arguments` in `src-tauri/src/engine/orchestrator.rs:206`, then on existing test lint issues in matcher tests and `commands_roundtrip`.
- GREEN: `rtk cargo clippy --all-targets --all-features -- -D warnings` passed with no issues after minimal lint-only fixes.
- GREEN after final fixes: `rtk cargo test --test commands_roundtrip --test sync_roundtrip --test prefs_roundtrip --test store_yaml -- --test-threads=1` passed 31 tests.
- GREEN after final fixes: `rtk cargo test --all-features -- --test-threads=1` passed 137 tests, 1 ignored.

## Integration Check Fixes

### Decisions made (not in spec)
- Added a focused `#[allow(clippy::too_many_arguments)]` on the existing `build_orchestrator` helper because refactoring that runtime construction path would be broader than the Phase 1 security work.
- Cleaned existing clippy warnings in Rust tests that were compiled by the required all-target clippy gate.

### Spec deviations
- Touched `src-tauri/src/engine/orchestrator.rs`, `src-tauri/tests/matcher_manual_mode.rs`, and `src-tauri/tests/matcher_armed_dismiss.rs` solely to satisfy the exact Phase 1 clippy integration check.

### Tradeoffs accepted
- Used lint-scoped annotations for intentional guard/lifetime patterns instead of refactoring unrelated matcher tests.

### Assumptions
- The exact clippy gate in Phase 1 is authoritative even when it exposes pre-existing warnings outside the Phase 1 file list.

### Follow-ups for human
- none

### Test evidence
- GREEN: `rtk cargo clippy --all-targets --all-features -- -D warnings` passed with no issues.
