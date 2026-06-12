# Phase 4 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Resolved declared snippet vars by name to `default` when present, else empty string, because Phase 4 has declarations but no form-value runtime yet.
- Kept `$|$` untouched in Task 1 so cursor-token validation can land as an isolated loader/resolver change in Task 2.

### Spec deviations
- none

### Tradeoffs accepted
- Used `chrono::Local::now()` directly in resolver tests, which keeps the implementation simple but makes the assertions depend on current local time formatting.

### Assumptions
- Declared vars should shadow built-in placeholder names even when they have no default value.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test expand_resolver` failed with unresolved `expand::{ClipboardReader, ResolveError, Resolver}` imports and missing `chrono`.
- GREEN: `cargo test --test expand_resolver` passed after adding the resolver module, re-exports, and `chrono`.

## Task 2
### Decisions made (not in spec)
- Promoted `LoadError` from a stringly struct to a variant enum and added `path()`/`message()` helpers so loader validation can expose `TooManyCursorTokens` directly without spreading match logic through the store tests.
- Counted caret movement in UTF-16 code units from the already-resolved text suffix after `$|$`, which keeps the value aligned with the `SendInput` left-arrow count Phase 4 needs later.

### Spec deviations
- none

### Tradeoffs accepted
- `strip_cursor_token` is called both by loader validation and by the resolver, which duplicates one cheap scan but keeps the loader check and runtime strip behavior simple and independent.

### Assumptions
- A single `$|$` token is legal anywhere in `replace`, including adjacent to placeholder output.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test cursor_math` failed because `LoadError::TooManyCursorTokens` did not exist and the resolver still returned raw `$|$`.
- GREEN: `cargo test --test cursor_math --test expand_resolver --test store_yaml` passed after adding cursor stripping, UTF-16 math, loader validation, and the typed `LoadError` variants.

## Task 3
### Decisions made (not in spec)
- Added dedicated `expand::datetime` and `expand::clipboard_var` helpers and kept them internal to `expand/`, since the phase only requires them as resolver plumbing rather than as public library APIs.
- Resolved declared `VarKind::Cursor` to the literal `$|$` token so the existing cursor-strip pass remains the single place that computes UTF-16 caret offsets.

### Spec deviations
- none

### Tradeoffs accepted
- Datetime var resolution uses the placeholder arg when present and otherwise falls back to the var declaration’s `format`, which keeps the placeholder override local and avoids another config layer.

### Assumptions
- Clipboard-backed placeholders should return empty string on non-text clipboard contents for both built-in and declared clipboard vars.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test expand_resolver` failed because declared `datetime`, `clipboard`, and `cursor` vars were still treated as plain defaults.
- GREEN: `cargo test --test expand_resolver --test cursor_math --test store_yaml` passed after dispatching by `VarKind` and extracting the datetime/clipboard helper modules.
- Clipboard read ordering: resolver expansion is called in the orchestrator before `Injector::inject`, so `clipboard` placeholders read the user clipboard before the injector snapshot/paste path overwrites and restores it.

## Task 4
### Decisions made (not in spec)
- Made `ClipboardBackend` extend the resolver’s `ClipboardReader` trait so the orchestrator can read clipboard text from the same backend instance it later uses for paste/restore, without adding a second clipboard service.
- Implemented `NotifySink` directly for `tauri::AppHandle` via `tauri-plugin-notification`, while keeping the orchestrator generic so unit tests can inject a fake notifier.
- Cleared the matcher buffer after every resolved hit and after unknown-placeholder failures so consecutive triggers behave like real expansions instead of accumulating typed trigger text.

### Spec deviations
- none

### Tradeoffs accepted
- Added the notification plugin dependency purely for unknown-placeholder tray feedback rather than hand-rolling OS notification calls; that is extra dependency surface but keeps the AppHandle integration straightforward and version-aligned with Tauri 2.
- Shared test-only global-state locking between `hook::winevent` and `engine::orchestrator` to keep `DENYLISTED`, `PAUSED`, and `SUSPEND` deterministic under Rust’s parallel test runner.

### Assumptions
- Sending `caret_left` as repeated left-arrow key events immediately after paste/unicode fallback is sufficient for the “after settle” requirement in this phase.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --lib --test expand_resolver --test cursor_math --test store_yaml` initially failed because the orchestrator still injected static text, did not reset the match buffer after expansions, and its tests raced on global denylist/pause state.
- GREEN: `cargo test --lib --test expand_resolver --test cursor_math --test store_yaml` passed after resolver wiring, caret-left injection, AppHandle notifier plumbing, post-hit buffer reset, and shared test-state locking.
- GREEN: `cargo test --all-features` passed with `41 passed, 1 ignored`.
- GREEN: `cargo clippy --all-targets -- -D warnings` passed clean.
- Coverage tool: `cargo llvm-cov --version` failed because `cargo-llvm-cov` is not installed in this environment, so the ≥80% coverage target is recorded as a tool-availability deviation.
