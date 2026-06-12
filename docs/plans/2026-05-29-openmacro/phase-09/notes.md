# Phase 9 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Stored sync backend state behind a small internal mutex (`remote`, `auth`, `local_dir`, last status) so the trait stays object-safe and the later Tokio driver can own `Arc<dyn SyncBackend>` without extra app-specific wrappers.
- Added a minimal `spawn_driver` helper that listens to the existing watcher snapshot channel and a manual trigger channel; it already debounces file-change ticks with the phase’s required 500 ms sleep.

### Spec deviations
- none

### Tradeoffs accepted
- The first backend pass uses no-op remote callbacks, which is sufficient for the SSH/file-path tests in this task while leaving real credential callbacks for task 2.

### Assumptions
- The sync working tree root passed to `SyncBackend::init` is the `sync/` directory itself, so conflict captures should live at `<local_dir>/.conflicts/<unix-ts>/`.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test sync:: --lib -- --nocapture` initially failed because the sync module was only a stub and `git2` was not in the dependency graph.
- GREEN: the same focused command passed with `2 passed, 21 filtered out` after adding `SyncBackend`, `GitBackend`, conflict-path helpers, and the interval/file-change driver scaffold.

## Task 2
### Decisions made (not in spec)
- Wrapped Credential Manager directly through the existing `windows` crate instead of pulling in an extra third-party wrapper crate; that keeps the secret path fully inside this repo and avoids guessing at an external crate API surface.
- Added a `Secret` newtype with a hard-redacted `Debug` impl and used that in `SyncCredential` so tests can enforce the no-log/no-display contract cheaply.

### Spec deviations
- Used direct Win32 Credential Manager calls rather than `windows-credentials` / `wincred`, because the expected library crate surface was not available in a form that fit the current codebase cleanly.

### Tradeoffs accepted
- The real credential round-trip test runs only on Windows and uses a deterministic test key; non-Windows builds still compile the trait surface but report “Credential Manager unavailable” if the real store is called.

### Assumptions
- Matching both raw `ERROR_NOT_FOUND` and HRESULT `0x80070490` is sufficient to normalize “credential missing” across the Windows APIs exercised here.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test sync::creds --lib -- --nocapture` first failed because the credential types/store trait did not exist, and then the real Windows round-trip exposed a missing-entry codepath mismatch (`0x80070490`).
- GREEN: the same focused command passed with `3 passed, 23 filtered out` after implementing the Credential Manager wrapper, redacted secret type, git credential callbacks, and the missing-entry normalization.

## Task 3
### Decisions made (not in spec)
- Added `SyncCommandState` as a thin app-managed wrapper around a trait-object backend plus credential store, with small `*_inner` helpers for tests so the round-trip/conflict cases do not need a Tauri runtime.
- Used `OPENMACRO_SYNC_ROOT` as a test-only-friendly override for the sync working tree path, mirroring the snippet/prefs path override pattern already present in the app.

### Spec deviations
- none

### Tradeoffs accepted
- The initial command layer keeps remote/auth configuration in memory only; it wires the periodic driver and manual commands for the current session without adding a separate persisted sync-config file that the phase did not explicitly request.

### Assumptions
- Local file-path remotes used in the tests are valid under `AuthMode::Ssh`, since they exercise the git transport path without needing an actual SSH server.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test sync_roundtrip --test sync_conflict -- --test-threads=1 --nocapture` failed because the sync command module did not exist, then the first runtime pass exposed two concrete issues: Windows checkout CRLF normalization in the round-trip assertion and conflict capture happening after `rebase.abort()`.
- GREEN: the same integration command passed with `2 passed` after adding the sync command state/inner helpers, wiring `lib.rs`, normalizing the text assertion, and capturing conflict files before aborting the rebase.

## Task 4
### Decisions made (not in spec)
- Split the frontend sync command wrappers into `src/lib/sync.ts` instead of overloading `lib/snippets.ts`, so the new typed DTOs (`SyncStatus`, `TickReport`, `SyncAuthMode`) stay scoped to sync concerns.
- Replaced the old snippet-reload diagnostics panel entirely with the git-sync controls the phase requested, keeping only the minimal local state needed for auth selection, inline validation, and status refreshes.

### Spec deviations
- none

### Tradeoffs accepted
- The panel derives the HTTPS host from the remote URL with a small `URL` parse helper and falls back to `"unknown"` for unparsable remotes, rather than adding a separate explicit host field the spec did not ask for.

### Assumptions
- Surfacing the raw `last_tick_unix` integer is acceptable for this phase’s status row; the prompt required the data, not a formatted timestamp presentation.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `pnpm test -- SyncPanel` failed because the old reload-diagnostics panel was still mounted, and the full Rust gate then exposed one setup ownership bug (the watcher had been moved before cloning a receiver for the sync driver) plus a clippy cleanup in the credential writer.
- GREEN: `cargo test --all-features -- --test-threads=1`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm test`, and `pnpm lint` all passed after replacing `SyncPanel`, adding `lib/sync.ts`, cloning the watcher receiver before moving the store, and folding the Win32 credential struct writes into an initializer.
