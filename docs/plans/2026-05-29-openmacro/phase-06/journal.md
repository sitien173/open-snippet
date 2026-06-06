# Phase 6 — Form runner (back-side)

- Status: IN_PROGRESS
- Owner: codex
- Started: 2026-06-06
- Finished: —

## Route
- Reason: Pure back-side Rust (Win32 focus dance, tokio state machine, orchestrator branch). No UI.
- Done When: cargo test green incl. form_focus; clippy clean; cancel preserves literal trigger; re-entrancy rejected; form-vars overlay works through resolver.

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 6 / codex / n-a / 2026-06-06 / 2026-06-06 12:33:15 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented the Phase 6 back-side form runner: Win32 foreground capture/restore helpers, a single-flight async form state machine, orchestrator form branching with delayed submit injection, and Tauri IPC commands for form submit/cancel.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-06/notes.md` | Added per-task decisions, deviations, assumptions, and RED→GREEN evidence. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-06/journal.md` | Appended this external response block. |
| Create/Modify | `openmacro/src-tauri/src/form/{mod.rs,focus.rs,runner.rs}` | Replaced the form stubs with focus helpers, a testable focus backend, window sink abstractions, and the async single-open form runner. |
| Modify | `openmacro/src-tauri/src/expand/resolver.rs` | Added optional form-values overlay support ahead of snippet var lookup. |
| Modify | `openmacro/src-tauri/src/inject/{mod.rs,clipboard.rs}` | Tightened trait bounds so the delayed form-submit path can reuse the existing injector safely across spawned tasks. |
| Modify | `openmacro/src-tauri/src/engine/orchestrator.rs` | Added the `VarKind::Form` branch, captured-foreground handoff, async runner usage, delayed resolve+inject on submit, and cancel/no-restore behavior. |
| Create | `openmacro/src-tauri/src/commands/form.rs` | Added `form_submit` and `form_cancel` Tauri commands. |
| Modify | `openmacro/src-tauri/src/commands/mod.rs` | Registered the new form command module. |
| Modify | `openmacro/src-tauri/src/lib.rs` | Managed a shared `Arc<FormRunner>`, wired it into app startup, and registered the form IPC commands. |
| Create/Modify | `openmacro/src-tauri/tests/{form_focus.rs,cursor_math.rs,expand_resolver.rs}` | Added form focus/runner tests and updated resolver tests for the new overlay signature and UTF-16 cursor math fixture. |
## COMMITS
- phase-6.task-1: 77ea224dfc807e8170171da3ccb6c3ce9728c77c  phase-6.task-1: add foreground focus surface
- phase-6.task-2: bd40c05f1647ad82a036ba3c8a5e23fbf681b9bb  phase-6.task-2: add form runner state machine
- phase-6.task-3: 05c286d12f5227f665237d60f7d1395f70f3b769  phase-6.task-3: branch orchestrator for forms
- phase-6.task-4: 7c2bc3169653f9881248b95028c0c8fc92e5e3c3  phase-6.task-4: add form command wiring
## NOTES
- phase-06/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? YES  — `cargo test --all-features` passed with `51 passed, 1 ignored`, `cargo clippy --all-targets --all-features -- -D warnings` passed, re-entrancy rejection is covered, cancel injects nothing and skips restore, and form-value overlay resolution is tested.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review

## Squash Commit

## Decisions
- per-task in notes.md.

## Handoff
- Phase 7 (gemini) builds the `/form/<id>` route that consumes `form_submit` / `form_cancel`.
