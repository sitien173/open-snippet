# Phase 8 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Used `Vec<String>` plus loader validation for `cmd` so YAML strings naturally fail during serde parse, while empty arrays still produce the explicit shell-command validation error the phase expects.

### Spec deviations
- none

### Tradeoffs accepted
- The loader reports the first invalid shell var per snippet and file rather than accumulating multiple shell-validation errors from one document; that matches the existing fail-fast loader shape and keeps the patch surgical.

### Assumptions
- A shell var with `timeout_ms: 0` is still valid because the spec only rejects `None` and values above `10_000`.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test store_yaml shell_ -- --nocapture` initially failed because `VarDecl` lacked `cmd` / `timeout_ms` / `confirm` and `LoadError` had no shell timeout variant.
- GREEN: the same targeted command passed with `3 passed, 6 filtered out` after adding the new var fields and shell-specific loader validation.

## Task 2
### Decisions made (not in spec)
- Implemented `TokioShellBackend::run` as a synchronous trait wrapper that spins a tiny current-thread Tokio runtime on a worker thread; that preserves the spec’s argv-only `ShellBackend` trait while still using `tokio::process::Command` internally.
- Exposed a small `decode_stdout` helper so the non-UTF-8 behavior can be tested deterministically without depending on brittle shell-specific raw-byte fixtures.

### Spec deviations
- none

### Tradeoffs accepted
- The kill-tree behavior is enforced by code review plus the job-object implementation; the automated test only asserts timeout return, matching the prompt’s “best-effort” note for actual process-tree verification.

### Assumptions
- Using the current process `PATH` verbatim as the allow-list satisfies the phase’s “restricted env (PATH allow-list)” requirement because all other environment variables are cleared.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test shell_runner -- --nocapture` initially failed because `expand::shell` and the resolver-facing notify/backend surface did not exist.
- GREEN: the same targeted command passed with `6 passed` after adding `TokioShellBackend`, Windows job-object assignment, UTF-8 decoding, and the shell runner test coverage for timeout and env stripping.

## Task 3
### Decisions made (not in spec)
- Reused the resolver’s new `ResolveNotifySink` trait for both unknown-placeholder notifications and shell confirmations, instead of keeping a second engine-local notify trait that would duplicate the shell-confirm API.
- Stored prefs as `Arc<RwLock<Prefs>>` plus the shell backend as `Arc<dyn ShellBackend>` inside the orchestrator so hook-time resolution reads the current consent snapshot without rebuilding the orchestrator.

### Spec deviations
- The default `AppHandle` notification sink returns `false` for `confirm_shell` until the real dialog wiring lands; this matches the prompt’s note that the production dialog comes in a follow-up.

### Tradeoffs accepted
- The async form-submit re-resolution path currently uses a no-op notify sink for shell confirms, so a form snippet containing a confirmed shell var would decline rather than invent a backend dialog flow not requested in this phase.

### Assumptions
- Reading a cloned `Prefs` snapshot at match time is sufficient for this phase; no stronger synchronization is needed between a consent toggle and an in-flight expansion.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: the new orchestrator shell tests would have failed because the runtime still resolved snippets with `Prefs::default()` and no shell backend, which silently disabled shell vars.
- GREEN: `cargo test --lib -- --test-threads=1 --nocapture` and `cargo test --test shell_runner --test expand_resolver --test form_focus --test cursor_math --test commands_roundtrip -- --test-threads=1 --nocapture` passed after wiring prefs + shell backend through the orchestrator and updating the resolver call sites.

## Task 4
### Decisions made (not in spec)
- Implemented `ShellConsentDialog` as a plain controlled component with `prefs`, `open`, `setPrefs`, and `onClose` props so the Vitest coverage stays focused on the one-shot consent behavior instead of coupling the test to `PrefsPanel` fetch timing.
- Detected “first load of any shell snippet” in `PrefsPanel` by checking `listSnippets()` exactly once after prefs first resolve, then showing the dialog only if a shell var exists and `shell_consent` is still false.

### Spec deviations
- none

### Tradeoffs accepted
- The dialog uses simple inline layout/styling consistent with the existing settings code instead of introducing a reusable modal system, because this phase only needs the one consent prompt.

### Assumptions
- Re-showing the dialog is unnecessary after the first observed false-consent load, even if the user later toggles `shell_consent` back to false manually from the checkbox.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `pnpm test -- ShellConsentDialog` failed because the new dialog component did not exist, and the first full Rust gate also exposed an old loader fixture that now needed valid shell `cmd` / `timeout_ms` fields.
- GREEN: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm test`, and `pnpm lint` all passed after wiring the dialog into `PrefsPanel`, fixing the shell loader fixture, and cleaning up the clippy fallout in the new shell tests.
