# Phase 4 — Resolvers + cursor token

- Status: DONE
- Owner: codex
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Pure-logic back-side Rust (datetime, clipboard read, cursor math). Pure unit tests.
- Done When: `cargo test expand_resolver cursor_math` green; clippy clean; cursor math uses UTF-16 code units; `clipboard` var resolves before injector save; unknown placeholder fails closed.

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 4 / codex / n-a / 2026-06-06 / 2026-06-06 10:24:36 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented Phase 4 placeholder resolution, cursor-token validation/UTF-16 caret math, and orchestrator injection wiring with notification-on-unknown behavior.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-04/notes.md` | Added per-task decisions, deviations, tradeoffs, assumptions, and RED→GREEN evidence. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-04/journal.md` | Appended this external response block. |
| Modify | `openmacro/src-tauri/Cargo.toml` | Added `chrono` and `tauri-plugin-notification` dependencies for resolver and notifier wiring. |
| Modify | `openmacro/src-tauri/Cargo.lock` | Locked the Phase 4 dependency graph updates. |
| Create/Modify | `openmacro/src-tauri/src/expand/{mod.rs,resolver.rs,cursor.rs,datetime.rs,clipboard_var.rs}` | Implemented placeholder parsing, cursor-token stripping, datetime formatting, clipboard reads, and exported resolver APIs. |
| Modify | `openmacro/src-tauri/src/store/loader.rs` | Added snippet validation and typed `LoadError::TooManyCursorTokens` handling. |
| Modify | `openmacro/src-tauri/src/hook/winevent.rs` | Added shared test-state locking for denylist-sensitive tests. |
| Create/Modify | `openmacro/src-tauri/src/inject/{mod.rs,clipboard.rs,sendinput.rs}` | Added readable clipboard backends, caret-left planning, and left-arrow `SendInput` support. |
| Modify | `openmacro/src-tauri/src/engine/orchestrator.rs` | Replaced static-text injection with resolver/notify/caret-aware orchestration and end-to-end unit tests. |
| Modify | `openmacro/src-tauri/src/lib.rs` | Registered the Tauri notification plugin. |
| Create | `openmacro/src-tauri/tests/expand_resolver.rs` | Added resolver integration tests for built-ins, declared var kinds, clipboard fallback, and unknown placeholders. |
| Create | `openmacro/src-tauri/tests/cursor_math.rs` | Added loader-validation and UTF-16 caret math tests for `$|$`. |
| Modify | `openmacro/src-tauri/tests/store_yaml.rs` | Updated loader assertions for the typed `LoadError` surface. |
## COMMITS
- phase-4.task-1: 2c96c385ae55e229d26a9b1a1be9ed9f2d2c65f5  phase-4.task-1: add placeholder resolver
- phase-4.task-2: 42d5f0155808811b55a6ba6a28ffec9f69d46989  phase-4.task-2: validate cursor token placement
- phase-4.task-3: 6c2d7dd719026887a0290490946d0d14a42f2dfe  phase-4.task-3: add datetime and clipboard var resolvers
- phase-4.task-4: 5200ed21bfe6a290985d304bfd3c2bc05a477737  phase-4.task-4: wire resolver into orchestrator
## NOTES
- phase-04/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? WITH_DEBT  — `cargo test --all-features` and `cargo clippy --all-targets -- -D warnings` passed, but the ≥80% coverage check could not be executed because `cargo-llvm-cov` is not installed in this environment.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS_WITH_DEBT
- Quality Findings:
  | Severity | path:line | Problem | Fix |
  | MEDIUM | engine/orchestrator.rs:133 | `Err(_)` arm swallows `ResolveError::UnterminatedPlaceholder` silently — no notification, no buffer reset; user gets no signal for malformed `{{...` snippets. | Reset buffer + notify (or surface a distinct sink call) when this is hit. Tracked as debt. |
  | LOW | expand/resolver.rs:11 | Field `cursor_chars_after_token` is named *chars* but holds UTF-16 code units (per cursor.rs:24). Misleading identifier. | Rename to `cursor_utf16_code_units_after_token` or similar; deferred to keep diff scoped. |
- Final Status: PASS_WITH_DEBT
- Explanation: Fresh `cargo test --all-features` (47 tests passed, 1 ignored E2E) and `cargo clippy --all-targets --all-features -- -D warnings` both clean. Spec Done-When met. Coverage gate not run (`cargo-llvm-cov` absent — pre-existing env debt). MEDIUM finding logged as carry-forward debt; not blocking Phase 5.
- Next: debt → coordinator backlog; proceed to Phase 5.

## Squash Commit
- b59e4ef  phase-4: resolvers + cursor token + orchestrator integration

## Decisions
- per-task in notes.md.
- Coordinator accepted `UnterminatedPlaceholder` silent-swallow as MEDIUM debt rather than block; worker contract did not specify a notify path for that branch.

## Handoff
- Phase 5 (gemini, Settings UI) consumes `Snippet`/`VarDecl` shapes.
- Carry-forward debt: (a) `cargo-llvm-cov` install + ≥80% coverage on expand/matcher/store; (b) UnterminatedPlaceholder UX path; (c) field naming clean-up in `Resolved`.
