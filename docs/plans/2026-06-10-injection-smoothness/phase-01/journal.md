# Phase 1 — Unicode-by-default injection, batched SendInput, updated tests

- Status: DONE
- Owner: Codex
- Started: 2026-06-10
- Finished: 2026-06-11

## Route

- Reason: Back-side Rust/Win32 injection rewrite; single side, scope is `src-tauri/src/inject/**` plus the three orchestrator/inject tests. No cross-validation needed.
- Done When: see PLAN.md.
- Files: src-tauri/src/inject/mod.rs, src-tauri/src/inject/sendinput.rs, src-tauri/src/engine/orchestrator.rs (test assertions only).

## External Response

<!-- worker appends here -->

# EXTERNAL RESPONSE
## META
- Phase 1 / codex / codex-20260611-phase-01 / 2026-06-11 / 2026-06-11T11:56:45.7862681+07:00 / F:/projects_new/textblaze/docs/plans/2026-06-10-injection-smoothness
## SUMMARY
Raised injection to unicode-by-default, batched Win32 `SendInput` for replacements and paste chords, rewrote the stale unicode-flow tests, and finished with a fully green `openmacro` test and clippy run.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Modified | F:/projects_new/textblaze/src-tauri/src/inject/mod.rs | Raised the unicode threshold, shortened inject timing, added `send_batch`, and batched injector actions. |
| Modified | F:/projects_new/textblaze/src-tauri/src/inject/sendinput.rs | Removed per-step paste delays and added batched Win32 `SendInput` submission with 256-entry chunking. |
| Modified | F:/projects_new/textblaze/src-tauri/src/inject/clipboard.rs | Moved paste key emission responsibility out of clipboard backends so injector batching owns the action stream. |
| Modified | F:/projects_new/textblaze/src-tauri/src/engine/orchestrator.rs | Updated the two stale orchestrator tests to assert unicode actions instead of `Paste(...)`. |
| Modified | F:/projects_new/textblaze/src-tauri/tests/store_yaml.rs | Updated the shipped-default-snippets count to match the current checked-out fixture used by the fresh verification run. |
| Added | F:/projects_new/textblaze/docs/plans/2026-06-10-injection-smoothness/phase-01/notes.md | Recorded Task 1-3 decisions and RED→GREEN verification evidence. |
| Modified | F:/projects_new/textblaze/docs/plans/2026-06-10-injection-smoothness/phase-01/journal.md | Appended this external response block. |
## COMMITS
- phase-1.task-1: 3615a8a  phase-1.task-1: unicode-by-default injection with shorter sleeps
- phase-1.task-2: 8a75e83  phase-1.task-2: batch sendinput, remove per-step paste delays
- phase-1.task-3: 0457e01  phase-1.task-3: verify cargo test + clippy green
## NOTES
- phase-01/notes.md  (## Task 1, ## Task 2, ## Task 3)
## SPEC COMPLIANCE
- Meets Spec? YES  - The injection path, stale tests, full `cargo test -p openmacro`, and `cargo clippy --all-targets -- -D warnings` are green with fresh evidence recorded this turn.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review

- Spec Status: PASS
- Quality Findings:
  | Severity | path:line | Problem | Fix |
  | --- | --- | --- | --- |
  | MEDIUM | src-tauri/src/inject/sendinput.rs:49-183 | `send_windows_action` (single) duplicates the INPUT-building logic now in `send_windows_actions` (batch). Production goes through batch; the single path remains live for trait `send()` callers. | Follow-up: have `send` delegate to `send_batch(&[action])`. Not blocking. |
  | LOW | src-tauri/src/inject/mod.rs:128-153 | The "short replacement uses unicode path" sentinel `Err(InjectError)` is used as control flow rather than a real error — minor readability smell. | Optional follow-up: replace with a `Branch::Unicode \| Branch::Paste` enum. |
  | LOW | src-tauri/tests/store_yaml.rs (diff in 0457e01) | Gratuitous rustfmt-style line wrapping mixed into the same commit as the necessary `len() == 6 → 7` assertion update (`snippets/default.yaml` gained a snippet in `90a349a UI Redesign`). Violates surgical-changes discipline. | Noted; revertable cosmetically if desired. |
- Final Status: PASS_WITH_DEBT
- Explanation: Functional behaviour fully meets spec — fresh `cargo test -p openmacro` is 96 passed / 1 ignored / 0 failed; fresh `clippy --all-targets -D warnings` clean. The `len() → 7` change in `store_yaml.rs` is necessary to satisfy the "fully green" gate (the test was stale vs `snippets/default.yaml` after the UI Redesign commit) and is correct. Debt is: (a) duplicated single-action SendInput path and (b) cosmetic reformatting that should have been a separate commit.
- Next: squash phase-1.task-1..3 into a single `fix(inject):` commit; user runs manual smoothness check (Notepad / Chrome / VS Code) per Handoff.

## Squash Commit

Applied: `3377c1a fix(inject): EVKey-style unicode-by-default snippet injection`

## Squash Commit

<!-- after Review PASS -->

## Decisions

- 2026-06-10 (user): Hybrid threshold — unicode by default up to ~2 KB, clipboard fallback only above 4 KB.
- 2026-06-10 (user): Update the three stale tests as part of this phase, not as a follow-up.
- 2026-06-10 (coordinator): Keep a single 5 ms `PRE_INJECT_DELAY` so the trigger char is committed before backspaces fire. Drop `POST_BACKSPACE_DELAY` and `PASTE_STEP_DELAY` entirely. Batch SendInput to one call per replacement (chunked to ≤256 entries).

## Handoff

After PASS, user runs manual smoothness check: type `;sig` and any longer trigger (e.g. paragraph-length expansion via `;log` or similar) in Notepad / Chrome / VS Code and confirm the visible behaviour matches EVKey's feel — no clipboard flash for small/medium snippets, no perceptible delay before the replacement appears.
