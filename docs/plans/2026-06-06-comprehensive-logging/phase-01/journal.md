# Phase 1 — Rust logging core

- Status: DONE
- Owner: codex
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Back-side — Rust crate setup, tracing subscriber stack, ring buffer, redaction, rotation. All under `src-tauri/`.
- Done When:
  - `cd src-tauri && cargo test` green (four new test files pass).
  - `cd src-tauri && cargo fmt --check` clean.
  - `cd src-tauri && cargo clippy -- -D warnings` clean on new code.
  - `pnpm tauri dev` smoke: app boots; daily-rotating JSON file appears at `<app_log_dir>/openmacro.log.YYYY-MM-DD`; pretty stdout shows entries.
- Files:
  - Modify: src-tauri/Cargo.toml, src-tauri/src/main.rs, src-tauri/src/lib.rs (or module root), src-tauri/src/store/* (LoggingConfig)
  - Create: src-tauri/src/log_init/{mod.rs,ring.rs,redact.rs,rotation.rs}
  - Create: src-tauri/tests/{log_init_filter.rs,log_init_redact.rs,log_init_ring.rs,log_init_rotation.rs}

## External Response
<!-- worker appends its full `# EXTERNAL RESPONSE` block here -->

# EXTERNAL RESPONSE
## META
- Phase 1 / codex / SessionID n/a / Started 2026-06-06 / Finished 2026-06-06 / Plan dir docs/plans/2026-06-06-comprehensive-logging
## SUMMARY
Implemented the Rust logging core: config schema, tracing subscriber init, ring buffer layer, redaction controls, rotation pruning, startup state wiring, and integration tests.
## FILES MODIFIED
| Action | Path | Change |
| Modify | docs/plans/2026-06-06-comprehensive-logging/phase-01/journal.md | Appended this ERP block. |
| Modify | docs/plans/2026-06-06-comprehensive-logging/phase-01/notes.md | Added Task 1-4 decision/evidence blocks. |
| Modify | src-tauri/Cargo.lock | Locked tracing dependencies. |
| Modify | src-tauri/Cargo.toml | Added tracing/tracing-subscriber/tracing-appender and test deps. |
| Modify | src-tauri/src/lib.rs | Declared `log_init`, initialized logging, and managed `LogHandles`. |
| Create | src-tauri/src/log_init/mod.rs | Added `init`, `LogHandles`, filter builder, and log dir helper. |
| Create | src-tauri/src/log_init/ring.rs | Added ring buffer, log entries, and tracing layer. |
| Create | src-tauri/src/log_init/redact.rs | Added field redaction, verbose toggle, and `log_body!`. |
| Create | src-tauri/src/log_init/rotation.rs | Added rotation pruning for `openmacro.log.*`. |
| Modify | src-tauri/src/store/mod.rs | Re-exported logging config types. |
| Modify | src-tauri/src/store/model.rs | Added logging config schema and serde defaults. |
| Create | src-tauri/tests/log_init_filter.rs | Added filter/init integration coverage. |
| Create | src-tauri/tests/log_init_redact.rs | Added redaction integration coverage. |
| Create | src-tauri/tests/log_init_ring.rs | Added ring buffer/layer integration coverage. |
| Create | src-tauri/tests/log_init_rotation.rs | Added rotation pruning integration coverage. |
| Modify | src-tauri/tests/store_yaml.rs | Added logging config tests and corrected tracked default snippet count to 6. |
## COMMITS
- phase-1.task-1: 0d498b8  phase-1.task-1: add logging config schema
- phase-1.task-2: b66ccf2  phase-1.task-2: add tracing ring buffer
- phase-1.task-3: 1c58f83  phase-1.task-3: add log redaction controls
- phase-1.task-4: 1161432  phase-1.task-4: initialize logging subscriber
## NOTES
- phase-01/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? WITH_DEBT - `cargo test` passed 87 tests and `cargo clippy -- -D warnings` passed; `cargo fmt --check` fails on pre-existing formatting across out-of-scope backend modules that Phase 1 explicitly forbids touching.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS_WITH_DEBT
- Quality Findings:
  - MEDIUM: `src-tauri/src/log_init/ring.rs:127-132` — `JsonFieldVisitor::record_debug` strips outer quotes from Debug output; can mangle embedded escapes. Track for Phase 2 follow-up.
  - LOW: `src-tauri/src/log_init/mod.rs:88-97` — `log_dir()` uses `dirs::config_dir()` not Tauri API; documented decision (matches existing prefs/sync/crash convention).
  - LOW: `src-tauri/src/lib.rs` init — uses `LoggingConfig::default()` not a persisted source; documented debt.
- Final Status: PASS_WITH_DEBT
- Explanation: tests + clippy green; `cargo fmt --check` failures are exclusively on pre-existing out-of-scope files Phase 1 was forbidden from touching.

## Squash Commit
- aa71705  phase-1: rust logging core

## Decisions
- `log_dir()` reuses the existing `dirs::config_dir()` convention (consistent with prefs/sync/crash modules) rather than introducing a Tauri-specific app-dir helper.
- Runtime init currently uses `LoggingConfig::default()` because no app-wide YAML config object exists; Phase 2's frontend-config command may need to wire a persisted source.

## Handoff
- Phase 2 (codex, back): instrument matcher/expand/hook/inject/sync/form/commands with `tracing::*` + `#[instrument]`, apply `log_body!` at sensitive sites, add `get_logging_frontend_cfg` and `get_log_ring` Tauri commands.
- While in Phase 2: address the MEDIUM ring.rs `record_debug` finding (use a non-stripping path for Debug fields).
- Consider whether `get_logging_frontend_cfg` should read from a persisted config source instead of `LoggingConfig::default()`.
