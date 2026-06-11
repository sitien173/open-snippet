# Phase 1 — Layout-aware translate_key + TRACE diagnostics

- Status: DONE
- Owner: Codex
- Started: 2026-06-10
- Finished: 2026-06-10

## Route

- Reason: Back-side Rust/Win32 hook bug; single side, low blast radius — no cross-validation needed.
- Done When: see PLAN.md.
- Files: src-tauri/src/hook/thread.rs (+ possibly src-tauri/src/hook/mod.rs or a new translate.rs for the pure helper).

## External Response

<!-- worker appends here -->

# EXTERNAL RESPONSE
## META
- Phase 1 / codex / codex-20260610-phase-01 / 2026-06-10 / 2026-06-10T17:21:07.4115171+07:00 / F:/projects_new/textblaze/docs/plans/2026-06-10-hook-layout-fix
## SUMMARY
Extracted a unit-tested `translate_with_layout` helper and switched hook key translation to the foreground thread HKL with read-only `ToUnicodeEx` TRACE diagnostics.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Modified | F:/projects_new/textblaze/src-tauri/src/hook/thread.rs | Added `TranslateOutcome` and `translate_with_layout`, then resolved translation HKL from the foreground window thread with TRACE logging. |
| Added | F:/projects_new/textblaze/docs/plans/2026-06-10-hook-layout-fix/phase-01/notes.md | Recorded Task 1-3 decisions and verification evidence. |
| Modified | F:/projects_new/textblaze/docs/plans/2026-06-10-hook-layout-fix/phase-01/journal.md | Appended this external response block. |
## COMMITS
- phase-1.task-1: 2bb4e49  phase-1.task-1: extract translate_with_layout helper
- phase-1.task-2: e51412c  phase-1.task-2: use foreground-thread layout for key translation
## NOTES
- phase-01/notes.md  (## Task 1, ## Task 2, ## Task 3)
## SPEC COMPLIANCE
- Meets Spec? NO  - Code changes and clippy are complete, but fresh `cargo test -p openmacro --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml` failed in pre-existing dirty files outside this phase.
## CLARIFICATIONS NEEDED
None
## NEXT
CONTINUE_SESSION

## Review

- Spec Status: PASS
- Quality Findings:
  | Severity | path:line | Problem | Fix |
  | --- | --- | --- | --- |
  | LOW | src-tauri/src/hook/thread.rs:155 | Unit tests `LoadKeyboardLayoutW` into the test thread without unloading; harmless but globally observable | Noted, no action this phase |
- Final Status: PASS
- Explanation: Codex's "Meets Spec? NO" claim about pre-existing failures verified: 3 failing tests (`engine::orchestrator::tests::resolved_round_trip_for_now_and_log_snippets`, `engine::orchestrator::tests::shell_snippet_injects_backend_output_when_consent_enabled`, `inject::tests::injector_sends_backspace_then_text_with_mocked_sink`) fail identically on parent commit `266e5e4` — they're stale expectations from the prior unicode/paste threshold split, not introduced by this phase. The phase's own tests are GREEN and `clippy --all-targets -D warnings` is clean, both with fresh evidence this turn.
- Next: squash phase-1.task-1 + phase-1.task-2 into a single `fix(hook):` commit; user runs manual repro with TRACE logging enabled (see Handoff).

## Squash Commit

Applied: `1e10588 fix(hook): translate keys with foreground window layout`

## Squash Commit

<!-- after Review PASS -->

## Decisions

- 2026-06-10 (coordinator): chose foreground-thread HKL with fallback to hook-thread HKL, drop on failure. Aggressive scan-code fallback deferred until TRACE logs from real reproduction show whether it's needed.
- 2026-06-10 (coordinator): use `wFlags = 2` on ToUnicodeEx to leave kernel dead-key buffer untouched.

## Handoff

After PASS, user must do a manual reproduction with TRACE logging enabled in Notepad, Chrome address bar, and VS Code editor. Coordinator will produce the manual-verify checklist as part of Gate 3.
