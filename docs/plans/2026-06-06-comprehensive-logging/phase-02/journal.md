# Phase 2 — Rust instrumentation + IPC commands

- Status: DONE
- Owner: codex
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Back-side — adopting `tracing` across backend modules and adding two new Tauri commands the frontend (Phase 3) will consume.
- Done When:
  - `cd src-tauri && cargo test` green (existing suites + new `commands_logging.rs`).
  - `cd src-tauri && cargo clippy -- -D warnings` clean.
  - `pnpm tauri dev` smoke: triggering a snippet expansion produces structured tracing entries with spans in stdout and the JSON file; bodies appear as `<redacted len=N>` by default; `OPENMACRO_LOG_VERBOSE=1` shows plain content.
  - `get_log_ring(0)` returns all entries; `get_log_ring(last_seq)` returns only newer.
  - `get_logging_frontend_cfg` returns the YAML `logging.frontend` block (or sensible defaults).
- Files:
  - Modify: src-tauri/src/commands/*.rs (instrument)
  - Modify: src-tauri/src/{matcher,expand,hook,inject,sync,form}/**.rs (instrument + log_body!)
  - Modify: src-tauri/src/log_init/ring.rs (fix MEDIUM Phase 1 finding: stop quote-stripping in record_debug)
  - Modify: src-tauri/src/lib.rs / main module (invoke_handler! registration for new commands)
  - Create: src-tauri/src/commands/logging.rs (new commands)
  - Create: src-tauri/tests/commands_logging.rs
  - Modify: src-tauri/tests/matcher_basic.rs and src-tauri/tests/sync_roundtrip.rs (add #[traced_test] smoke)

## External Response
<!-- worker appends its full `# EXTERNAL RESPONSE` block here -->

# EXTERNAL RESPONSE
## META
- Phase 2 / codex / SessionID 019e9c74-7ac4-7d20-a0e7-3ff8bff27bb3 / Started 2026-06-06 / Finished 2026-06-06 / Plan dir docs/plans/2026-06-06-comprehensive-logging
## SUMMARY
Instrumented the Rust backend with tracing/redaction, fixed the Phase 1 ring debug-field finding, and added logging IPC commands with tests.
## FILES MODIFIED
| Action | Path | Change |
| Modify | docs/plans/2026-06-06-comprehensive-logging/phase-02/journal.md | Appended this ERP block. |
| Create | docs/plans/2026-06-06-comprehensive-logging/phase-02/notes.md | Added Task 1-4 decision/evidence blocks. |
| Modify | src-tauri/src/commands/form.rs | Added tracing spans/events with form value counts only. |
| Create | src-tauri/src/commands/logging.rs | Added `get_logging_frontend_cfg` and `get_log_ring` commands plus inner helpers. |
| Modify | src-tauri/src/commands/mod.rs | Registered logging command module. |
| Modify | src-tauri/src/commands/prefs.rs | Added tracing to prefs commands. |
| Modify | src-tauri/src/commands/snippets.rs | Added tracing to snippet commands. |
| Modify | src-tauri/src/commands/sync.rs | Added tracing to sync commands and PAT-safe metadata. |
| Modify | src-tauri/src/expand/clipboard_var.rs | Added redacted clipboard placeholder logging. |
| Modify | src-tauri/src/expand/resolver.rs | Added spans and redacted snippet/form logging. |
| Modify | src-tauri/src/expand/shell.rs | Added shell execution logs with redacted stdout. |
| Modify | src-tauri/src/form/focus.rs | Added foreground capture/restore logging. |
| Modify | src-tauri/src/form/runner.rs | Added form lifecycle logs with redacted submitted values. |
| Modify | src-tauri/src/hook/ring.rs | Added hook event logs with redacted character content. |
| Modify | src-tauri/src/hook/thread.rs | Added hook lifecycle logs. |
| Modify | src-tauri/src/hook/winevent.rs | Added foreground denylist logs. |
| Modify | src-tauri/src/inject/clipboard.rs | Added redacted clipboard paste/read logs. |
| Modify | src-tauri/src/inject/mod.rs | Added injection span/logs with redacted output body. |
| Modify | src-tauri/src/inject/sendinput.rs | Added keyboard action-type logs without payloads. |
| Modify | src-tauri/src/lib.rs | Managed effective `LoggingConfig` and registered logging commands. |
| Modify | src-tauri/src/log_init/ring.rs | Stopped stripping quotes from debug-rendered fields. |
| Modify | src-tauri/src/matcher/automaton.rs | Added matcher rebuild/match spans and logs. |
| Modify | src-tauri/src/sync/conflicts.rs | Added conflict-copy metadata logging. |
| Modify | src-tauri/src/sync/creds.rs | Added credential-store logs without secrets. |
| Modify | src-tauri/src/sync/git.rs | Added Git sync operation spans/logs. |
| Modify | src-tauri/src/sync/mod.rs | Added sync driver logs. |
| Create | src-tauri/tests/commands_logging.rs | Covered log ring deltas, frontend config shape, and JSON file redaction. |
| Modify | src-tauri/tests/log_init_ring.rs | Added debug string escape regression test. |
| Modify | src-tauri/tests/matcher_basic.rs | Added traced smoke annotation to matcher test. |
| Modify | src-tauri/tests/sync_roundtrip.rs | Added traced smoke annotation to sync test. |
## COMMITS
- phase-2.task-1: de37bc6  phase-2.task-1: preserve debug field escapes
- phase-2.task-2: f9045f1  phase-2.task-2: instrument backend tracing
- phase-2.task-3: b1bf62b  phase-2.task-3: add logging commands
- phase-2.task-4: c65ea29  phase-2.task-4: add traced smoke tests
## NOTES
- phase-02/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? YES - `cargo test` passed 91 tests with 1 ignored, `cargo clippy -- -D warnings` passed, no `println!`/`eprintln!` remain, and JSON sink redaction is covered.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS
- Quality Findings:
  - LOW: `src-tauri/src/store/watcher.rs` still contains one `println!`. File was out of Phase 2's scope; non-blocking.
- Final Status: PASS
- Explanation: 91 tests + 1 ignored pass; clippy clean. MEDIUM Phase 1 finding fixed (record_debug). Redaction at sensitive sites (`log_body!` in expand/resolver, `FieldKind::ClipboardText` in inject/clipboard, `FieldKind::FormValue` in form/runner). Tauri commands shaped with inner helpers for testability.

## Squash Commit
- fb346ab  phase-2: rust tracing instrumentation + log IPC commands

## Decisions
- `get_logging_frontend_cfg` reads `LoggingConfig` from managed Tauri state (option 1 from the prompt — return effective config).
- `LoggingConfig` is added to managed Tauri state alongside `LogHandles`.

## Handoff
- Phase 3 (gemini, front): create `src/lib/logger.ts` (loglevel + redaction + ring), `await initFromConfig()` in `src/main.tsx`, migrate `console.*` in `routes/form`, `routes/settings/{index, PrefsPanel, SyncPanel}`. Frontend calls `invoke("get_logging_frontend_cfg")` (returns `{ level, modules, verbose_content? }` — verify the actual shape; if `verbose_content` is missing, derive from a separate field or skip it).
- Phase 4 will consume `invoke("get_log_ring", { sinceSeq })` returning `Vec<LogEntry>` with fields `seq, ts_unix_ms, level, target, message, fields, span_path`.
- Phase 2 carry-over (LOW): convert `store/watcher.rs` println in a later pass.
