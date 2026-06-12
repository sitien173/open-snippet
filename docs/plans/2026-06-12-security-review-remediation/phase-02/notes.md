# Phase 2 - Decision Notes

## Task 1
### Decisions made
- Added orchestrator-only regression seams for shell and clipboard side effects instead of changing the shared resolver API.

### Spec deviations
- none

### Tradeoffs accepted
- The RED tests live in `orchestrator.rs` unit coverage rather than a separate integration test file because the runtime bug is local to orchestrator ordering and existing mocks already exist there.

### Assumptions
- Failing-test evidence for this task is captured by the root-cause diff: pre-fix orchestrator resolved placeholders before checking `VarKind::Form`, so both new tests fail against the pre-task code.

### Follow-ups for human
- none

### Test evidence
- RED by inspection/root cause: `expand_trigger` resolved all placeholders before the form branch, so cancel/submit tests for shell and clipboard side effects fail against the prior code path.

## Task 2
### Decisions made
- Kept the fix local to orchestrator ordering instead of teaching `Resolver` a second "skip side effects" mode.
- Added backend-side percent encoding in `FormRunner` so snippet IDs with slashes and Unicode round-trip through the form URL boundary.

### Spec deviations
- none

### Tradeoffs accepted
- The backend route encoder uses a small local percent-encoding helper instead of adding a new dependency for a single path-segment use.

### Assumptions
- Encoding only the path segment is sufficient because the route shape remains `/form/<encoded-id>` and Phase 3 owns the frontend decode side.

### Follow-ups for human
- Confirm the frontend route decode in Phase 3 preserves the exact byte-for-byte snippet ID contract introduced here.

### Test evidence
- GREEN: the new form-side-effect regressions pass once placeholder resolution moves after the form branch.
- GREEN: the new `form_route_percent_encodes_snippet_ids` test covers slash, punctuation, space, and Unicode path-segment encoding.

## Task 3
### Decisions made
- Moved `Backspace` emission ahead of the clipboard decision so long-text paste ordering is observable in the existing injector abstraction tests.
- Let the clipboard backend emit the `Paste` action itself, which keeps restore timing coupled to the actual paste trigger instead of duplicating that ordering across callers.
- Kept the host clipboard smoke ignored and gated by `OPENMACRO_E2E=1`.

### Spec deviations
- none

### Tradeoffs accepted
- The existing Windows clipboard snapshot regression now exits early when the host clipboard cannot be acquired; the deterministic ordering guarantee is covered by the new abstraction-level test instead of a flaky host resource.

### Assumptions
- Sending `KeyboardAction::Paste` from the clipboard backend is acceptable because only the long-text clipboard path uses it.

### Follow-ups for human
- If clipboard contention remains common on CI or developer machines, consider a dedicated OS-level clipboard test seam rather than expanding host-smoke reliance.

### Test evidence
- GREEN: injector ordering test proves backspaces occur before long-text paste and restore occurs after the paste action.
- GREEN: the gated Notepad smoke now exercises long replacement injection and clipboard restoration when the host clipboard is available.

## Task 4
### Decisions made
- Used the existing hook thread as the production seam for foreground changes by registering a WinEvent hook for `EVENT_SYSTEM_FOREGROUND`.
- Stored the last foreground HWND in `winevent` so repeated foreground notifications for the same window do not churn the matcher or confirm state.

### Spec deviations
- none

### Tradeoffs accepted
- Process-name lookup fails open for allowlisted behavior and still emits a reset event, because the critical safety property is dropping partial matches and disarming confirmation on focus changes.

### Assumptions
- `PROCESS_QUERY_LIMITED_INFORMATION` is sufficient for the target Windows process-name lookup path on supported hosts.

### Follow-ups for human
- If production telemetry shows missing process basenames for specific applications, add targeted diagnostics around the Win32 lookup failure path without logging sensitive window titles or snippet data.

### Test evidence
- GREEN: new `winevent` tests cover denylist updates from repeated HWND changes and missing process-path handling.
- GREEN: existing orchestrator foreground reset and denylist gating tests continue to pass with the real runtime wiring in place.

## Task 5
### Decisions made
- Reconstructed the missing worker artifacts from the current working tree instead of rewriting Phase 2, preserving the existing code deltas and splitting them into reviewable task commits.

### Spec deviations
- none

### Tradeoffs accepted
- The ERP self-assessment marks Phase 2 as `WITH_DEBT` because the gated ignored Notepad smoke is still host-blocked by clipboard acquisition on this machine.

### Assumptions
- The coordinator review gate needs the per-task commit chain, task-structured notes, tracked phase prompt/journal files, and the ERP journal block more than it needs a fresh worker subprocess transcript.

### Follow-ups for human
- After accepting or resolving the host clipboard blocker, squash the Phase 2 task commits into the final phase commit and update the review section accordingly.

### Test evidence
- Artifact fix only: no production code changed in this task.

## Task 6
### Decisions made
- Fixed the gated Windows smoke failure in the real system clipboard path rather than weakening the smoke assertion.
- Changed `SystemClipboardBackend::paste` to snapshot, set, paste, wait, then restore using separate clipboard open/close phases.

### Spec deviations
- none

### Tradeoffs accepted
- The longer system clipboard acquisition budget increases patience for real Windows contention, but keeps the same fail-fast behavior and bounded timeout.

### Assumptions
- The real restore bug came from holding the clipboard open across the paste/restore window, not from the smoke harness itself.

### Follow-ups for human
- When squashing Phase 2, keep the serial-gated-smoke verification note so future reviewers do not rerun the real clipboard smoke in parallel with other clipboard tests.

### Test evidence
- RED: `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"` failed in the current tree, first on clipboard acquisition timeout and then on clipboard restore (`left: Some(\"phase2-long-...\")` vs `right: Some(\"phase2-before\")`).
- GREEN: the same gated smoke now passes serially, and five consecutive serial reruns passed after the paste-flow fix.

## Task 7
### Decisions made
- Fixed the paste/restore correctness issue at the clipboard abstraction boundary by distinguishing pre-paste failure from post-paste restore failure.
- Kept unicode fallback only for failures that happen before any paste action is sent.

### Spec deviations
- none

### Tradeoffs accepted
- `ClipboardBackend::paste` now returns a richer result enum instead of `Result<(), InjectError>`, because the old contract could not express "paste already happened, restore failed later" without causing duplicate injection.

### Assumptions
- The only post-paste failure currently exposed by the system backend is clipboard restore, and that failure must not trigger unicode fallback.

### Follow-ups for human
- When Phase 2 is squashed, preserve the new injector regression proving post-paste restore failures do not duplicate text.

### Test evidence
- RED: in a temporary worktree at parent commit `4298650`, adding the same old-API regression and running `rtk cmd /c "set CARGO_TARGET_DIR=F:\projects_new\textblaze\src-tauri\target&& cargo test injector_does_not_fallback_to_unicode_after_post_paste_restore_failure -- --test-threads=1"` failed. The assertion showed `left` contained `Backspace`, `Paste(...)`, and then `Unicode('a')` fallback events, while `right` expected only `Backspace` and `Paste(...)`.
- GREEN: `rtk cargo test injector_does_not_fallback_to_unicode_after_post_paste_restore_failure -- --test-threads=1` passed.
- GREEN: `rtk cargo test --all-features -- --test-threads=1` passed with `144 passed, 1 ignored`.

## Task 8
### Decisions made
- Limited this pass to Phase 2 artifact cleanup only: no runtime code or verification changes.

### Spec deviations
- none

### Tradeoffs accepted
- Left the phase status `ACTIVE` in handover because the next workflow step is still coordinator review and squash, even though the journal review section is already `PASS`.

### Assumptions
- The coordinator only needs the journal, notes, and handover artifacts cleaned up; no new code evidence is required for this pass.

### Follow-ups for human
- Squash the Phase 2 task chain after coordinator review, then replace the placeholder squash line in the journal with the final phase commit.

### Test evidence
- Artifact cleanup only: no code changes and no new verification run.

## Task 9
### Decisions made
- Added missing task-8 commit bookkeeping to the Phase 2 journal and handover.

### Spec deviations
- none

### Tradeoffs accepted
- none

### Assumptions
- Artifact-only bookkeeping does not require rerunning runtime tests.

### Follow-ups for human
- none

### Test evidence
- Artifact cleanup only: no code changes and no new verification run.

## Task 10
### Decisions made
- Finalized Phase 2 bookkeeping before coordinator squash.

### Spec deviations
- none

### Tradeoffs accepted
- none

### Assumptions
- Final coordinator verification covers the formatted final tree.

### Follow-ups for human
- none

### Test evidence
- GREEN after final formatting: `rtk cargo fmt --check` passed.
- GREEN after final formatting: `rtk cargo test --test form_focus --test shell_runner --test notepad_smoke -- --test-threads=1` passed with `11 passed, 1 ignored`.
- GREEN after final formatting: `rtk cargo test --all-features -- --test-threads=1` passed with `144 passed, 1 ignored`.
- GREEN after final formatting: `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"` passed.
