# Phase 3 — Keyboard hook + injection

- Status: DONE
- Owner: codex
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Windows-specific Rust system code (`WH_KEYBOARD_LL`, `SendInput`, clipboard, hotkey, WinEvents). Pure back-side.
- Done When: cargo test green (excluding `#[ignore]`/E2E); clippy clean; hook callback alloc-free; pause toggled via `AtomicBool`; clipboard save/restore covers all formats; SPSC ring drop semantics tested; denylist + foreground reset tested with mocked WinEvent.

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 3 / codex / n-a / 2026-06-06 / 2026-06-06 01:46:24 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented Phase 3’s hook-event transport, denylist foreground reset logic, clipboard/sendinput injector surface, pause-aware orchestrator logic, and the ignored Windows smoke-test scaffold.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-03/notes.md` | Added per-task decisions, deviations, and RED→GREEN evidence. |
| Modify | `openmacro/src-tauri/Cargo.toml` | Added `rtrb` and the Windows API feature set needed for hook and injection work. |
| Modify | `openmacro/src-tauri/Cargo.lock` | Locked the new Phase 3 dependency graph. |
| Create/Modify | `openmacro/src-tauri/src/hook/{mod.rs,ring.rs,thread.rs,winevent.rs}` | Replaced hook stubs with event types, SPSC transport, the Win32 hook-thread scaffold, and denylist/foreground helpers. |
| Create/Modify | `openmacro/src-tauri/src/inject/{mod.rs,clipboard.rs,sendinput.rs}` | Replaced injector stubs with clipboard snapshot/restore helpers, mocked and Windows keyboard sinks, and injection planning. |
| Create/Modify | `openmacro/src-tauri/src/engine/{mod.rs,orchestrator.rs}` | Added pause state, pure orchestration logic, and runtime setup scaffolding. |
| Modify | `openmacro/src-tauri/src/lib.rs` | Wired tray Pause/Resume to the engine pause atomic and registered the engine handle during app setup. |
| Create | `openmacro/src-tauri/tests/notepad_smoke.rs` | Added the ignored, `OPENMACRO_E2E=1`-gated Windows smoke-test scaffold. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-03/journal.md` | Appended this external response block. |
## COMMITS
- phase-3.task-1: cc2246e4dada09422f7d6eb7e83a8a3b831000a9  phase-3.task-1: add hook ring and thread scaffolding
- phase-3.task-2: f6ad557926b3c0daada1e03af139706c583c84f9  phase-3.task-2: add denylist foreground handling
- phase-3.task-3: de59e659da62d721a9383bb62ecd13a68bec2e16  phase-3.task-3: add injector and clipboard path
- phase-3.task-4: be877f08273370b948bfe846191c9c2669937d49  phase-3.task-4: wire orchestrator pause and smoke scaffold
## NOTES
- phase-03/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? WITH_DEBT  — The unit tests and `clippy` pass, but the actual long-lived Win32 hook/runtime ownership and the interactive Notepad E2E run remain scaffolded rather than fully exercised in this worker session.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS_WITH_DEBT
- Quality Findings: No findings (scoped to changed files).
- Final Status: PASS_WITH_DEBT
- Explanation: Cargo test + clippy clean. Debt: long-lived Win32 hook ownership + Notepad E2E remain scaffolded (worker can't drive a foreground UI session); E2E test compiles and is `#[ignore]`-gated.
- Next: Phase 4.

## Squash Commit
- phase-3: keyboard hook + injection — squashed

## Decisions
- per-task in notes.md.

## Handoff
- Phase 4 plugs the resolver between match and inject in the orchestrator.
