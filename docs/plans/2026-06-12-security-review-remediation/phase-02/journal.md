# Phase 2 - Runtime Safety And Injection Ordering
- Status: REVIEWED
- Owner: codex
- Started: 2026-06-12T17:05:21+07:00
- Finished: 2026-06-12T21:10:00+07:00

## Route
- Reason: Backend/runtime safety phase touching Rust expansion ordering, clipboard injection, hook foreground events, and form runner behavior.
- Done When:
  - `cd src-tauri && cargo test form_focus shell_runner notepad_smoke -- --test-threads=1`
  - `cd src-tauri && cargo test --all-features -- --test-threads=1`
  - `cd src-tauri && OPENMACRO_E2E=1 cargo test notepad_smoke -- --ignored --test-threads=1`
- Files:
  - `src-tauri/src/engine/orchestrator.rs`
  - `src-tauri/src/expand/resolver.rs`
  - `src-tauri/src/inject/mod.rs`
  - `src-tauri/src/inject/clipboard.rs`
  - `src-tauri/src/inject/sendinput.rs`
  - `src-tauri/src/hook/mod.rs`
  - `src-tauri/src/hook/thread.rs`
  - `src-tauri/src/hook/winevent.rs`
  - `src-tauri/src/form/runner.rs`
  - `src-tauri/tests/form_focus.rs`
  - `src-tauri/tests/shell_runner.rs`
  - `src-tauri/tests/notepad_smoke.rs`

## External Response
# EXTERNAL RESPONSE
## META
- Phase 2 / codex / 019ebab2-7241-77a0-95d1-bea4e4c9de92 / 2026-06-12T17:05:21+07:00 / 2026-06-12T21:10:00+07:00 / docs/plans/2026-06-12-security-review-remediation
## SUMMARY
Implemented the Phase 2 backend/runtime safety fixes and reconstructed the required worker artifacts as task commits, notes, and journal evidence.
## FILES MODIFIED
| Action | Path | Change |
| Modify | src-tauri/src/engine/orchestrator.rs | Added form-side-effect regressions and deferred placeholder resolution for form snippets until submit. |
| Modify | src-tauri/src/form/runner.rs | Percent-encoded backend form route snippet IDs before opening `/form/...`. |
| Modify | src-tauri/src/inject/mod.rs | Reordered long-text injection, then fixed fallback logic so post-paste restore errors do not duplicate text via unicode fallback. |
| Modify | src-tauri/src/inject/clipboard.rs | Moved paste triggering into the clipboard backend, fixed the real system clipboard path to restore via separate open/close phases, and exposed post-paste restore failures without treating them as pre-paste failures. |
| Modify | src-tauri/src/hook/thread.rs | Registered real foreground-window WinEvent handling in the production hook thread. |
| Modify | src-tauri/src/hook/winevent.rs | Added foreground HWND tracking, denylist updates from real process basenames, and runtime regressions. |
| Modify | src-tauri/tests/notepad_smoke.rs | Extended the gated ignored smoke to exercise long replacement paste and clipboard restoration, with a longer bounded clipboard timeout. |
| Create | docs/plans/2026-06-12-security-review-remediation/phase-02/notes.md | Recorded per-task decisions, deviations, assumptions, and RED-to-GREEN evidence. |
| Create | docs/plans/2026-06-12-security-review-remediation/phase-02/journal.md | Recorded the Phase 2 route, ERP block, and handoff state. |
| Create | docs/plans/2026-06-12-security-review-remediation/phase-02/prompt.md | Tracked the Phase 2 worker prompt artifact used for the remediation route. |
| Modify | docs/plans/2026-06-12-security-review-remediation/.handover.md | Updated active-phase metadata, blockers, and next action with the reconstructed task chain. |
## COMMITS
- phase-2.task-1: 7b460a0  phase-2.task-1: add form side-effect regressions
- phase-2.task-2: 5b5e167  phase-2.task-2: defer form resolution until submit
- phase-2.task-3: a372339  phase-2.task-3: fix clipboard paste ordering
- phase-2.task-4: 010bb55  phase-2.task-4: wire foreground denylist resets
- phase-2.task-5: 9f731d3  phase-2.task-5: record worker artifacts
- phase-2.task-6: 4298650  phase-2.task-6: stabilize gated clipboard smoke
- phase-2.task-7: 789ca61  phase-2.task-7: fix paste restore fallback semantics
- phase-2.task-8: 846d3c2  phase-2.task-8: clean phase artifacts
- phase-2.task-9: 0fbe591  phase-2.task-9: complete artifact bookkeeping
- phase-2.task-10: a8b44a8  phase-2.task-10: finalize squash bookkeeping
## NOTES
- phase-02/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4, ## Task 5, ## Task 6, ## Task 7, ## Task 8, ## Task 9, ## Task 10)
## SPEC COMPLIANCE
- Meets Spec? YES - Targeted tests, full Rust tests, and the gated ignored Notepad smoke all pass when the real clipboard smoke runs serially.
## CLARIFICATIONS NEEDED
None
## NEXT
CONTINUE_SESSION

## Review
- Spec Status: PASS
- Quality Findings: No findings.
- Final Status: PASS

## Squash Commit
- phase-2: runtime safety and injection ordering

## Decisions
- Phase 2 routes directly to Codex because the work is backend/runtime-only and the plan already completed the high-impact cross-validation checkpoint before Phase 1.

## Handoff
- Current verification:
  - `rtk cargo test --test form_focus --test shell_runner --test notepad_smoke -- --test-threads=1` passed: `11 passed, 1 ignored`.
  - `rtk cargo test --all-features -- --test-threads=1` passed: `144 passed, 1 ignored`.
  - `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"` passed.
  - `rtk cmd /c "for /l %i in (1,1,5) do @echo RUN %i && set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1 || exit /b 1"` passed five consecutive serial reruns.
  - `rtk cargo test injector_does_not_fallback_to_unicode_after_post_paste_restore_failure -- --test-threads=1` passed.
  - RED evidence for task 7 was reproduced in a temporary parent worktree at `4298650`: the old-API regression failed because actions contained `Backspace`, `Paste(...)`, then Unicode fallback events.
  - `rtk cargo fmt --check` passed after final formatting.
- Notes: do not run the real clipboard smoke in parallel with other clipboard-touching tests; it is intended to run serially.

Next step: open Phase 3 Frontend Form Contract And Snippet Editor Correctness and route it to Gemini.
