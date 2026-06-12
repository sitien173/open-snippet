# Phase 5 - Decision Notes

## Task 1
### Decisions made
- Added the missing CI gates directly to `.github/workflows/ci.yml` instead of introducing new package script aliases.

### Spec deviations
- none

### Tradeoffs accepted
- The workflow continues to run on `windows-latest` so the Rust and frontend checks match the Windows-first runtime assumptions already used by the project.

### Assumptions
- CI gate coverage for TypeScript build failures and Rust formatting drift is sufficient without creating new wrapper scripts.

### Follow-ups for human
- none

### Test evidence
- RED by inspection: the existing CI workflow ran frontend tests/lint and Rust test/clippy, but it did not run `pnpm build` or `cargo fmt --check`.

## Task 2
### Decisions made
- Reworked the gated Windows smoke to drive a real `Orchestrator` against a real Notepad window instead of calling `Injector::inject` directly.
- Covered both the short unicode path and the long clipboard paste path in the same gated smoke.

### Spec deviations
- none

### Tradeoffs accepted
- The smoke remains `#[ignore]` and gated by `OPENMACRO_E2E=1` because it depends on a GUI desktop session and the real Windows clipboard.

### Assumptions
- Trigger-expansion coverage is meaningful only if the test types the trigger into Notepad, lets the orchestrator erase it, and then verifies actual Notepad output.

### Follow-ups for human
- Keep the gated smoke serial; do not run it in parallel with other clipboard-touching tests.

### Test evidence
- RED by inspection: the old smoke only called `Injector::inject` with a prepared `InjectPlan`, so it never exercised matcher/orchestrator trigger expansion.
- GREEN: `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"` passed after switching the smoke to a real orchestrator-driven trigger path and direct Notepad window-text readback.

## Task 3
### Decisions made
- Added a phase-local verification note instead of expanding general project docs so the remediation commands stay tied to this plan and phase.

### Spec deviations
- none

### Tradeoffs accepted
- The verification note references both Rust and frontend commands because Phase 5 is explicitly the CI/verification checkpoint across prior remediation phases.

### Assumptions
- Coordinators and later reviewers need exact rerun commands more than narrative explanation.

### Follow-ups for human
- none

### Test evidence
- RED by inspection: before this task, Phase 5 had no committed command list covering the security-boundary, runtime/injection, frontend privacy, CI parity, and gated Windows smoke checks together.

## Task 4
### Decisions made
- Fixed the all-target clippy gate by moving the existing `form::runner` unit test module below the helper functions instead of suppressing the lint.
- Recorded the gated Notepad smoke as the only manual-only verification item because it requires an interactive Windows desktop session.

### Spec deviations
- none

### Tradeoffs accepted
- `pnpm build` still reports the pre-existing CSS warning for `line-weight`; Phase 5 keeps that warning visible instead of folding an unrelated CSS cleanup into this backend/infra phase.

### Assumptions
- Phase 5 is complete when the full CI-equivalent matrix passes and the one GUI-only smoke is documented with its exact rerun command.

### Follow-ups for human
- Run the gated Notepad smoke only on a real Windows desktop session where clipboard ownership is stable.

### Test evidence
- GREEN: `rtk pnpm install --frozen-lockfile` passed.
- GREEN: `rtk pnpm build` passed; existing CSS warning for `line-weight` remains outside the Phase 5 file set.
- GREEN: `rtk pnpm test` passed: 13 files / 70 tests.
- GREEN: `rtk pnpm lint` passed.
- GREEN: `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo fmt --check"` passed after formatting the new smoke test.
- GREEN: `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo test --all-features -- --test-threads=1"` passed.
- GREEN: `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo clippy --all-targets --all-features -- -D warnings"` passed after moving the `form::runner` test module below non-test items.
- GREEN: `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"` passed.

## Task 5
### Decisions made
- Added a final artifact-only bookkeeping commit so the journal and handoff can pin Task 4 to its real hash without another self-referential amend cycle.

### Spec deviations
- none

### Tradeoffs accepted
- Artifact bookkeeping is tracked as a standalone follow-up commit because Git commit ids are not stable across amend operations.

### Assumptions
- The coordinator needs the durable artifacts to reference the finalized Task 4 commit id more than it needs a perfectly four-commit phase.

### Follow-ups for human
- none

### Test evidence
- RED by process: amending Task 4 to update its own hash changed the commit id again, so a separate artifact-only bookkeeping commit was required to make the recorded metadata stable.

## Task 6
### Decisions made
- Repaired the Phase 5 journal commit ledger so Task 5 includes its finalized commit hash.

### Spec deviations
- none

### Tradeoffs accepted
- No code or verification evidence changed; this was artifact-only bookkeeping.

### Assumptions
- Stable task commit hashes in `journal.md` are required before coordinator review can pass.

### Follow-ups for human
- none

### Test evidence
- RED by review: the Phase 5 journal listed `phase-5.task-5` without hash `5db4a67`.
- GREEN: `docs/plans/2026-06-12-security-review-remediation/phase-05/journal.md` now lists `phase-5.task-5: 5db4a67 finalize artifact bookkeeping`.

## Task 7
### Decisions made
- Bound the gated Notepad smoke to the launched instance by snapshotting visible top-level windows before spawn, then accepting only either the child PID's top-level window or a newly created foreground Notepad window that did not exist before spawn.

### Spec deviations
- none

### Tradeoffs accepted
- The fallback still uses foreground state, but only for a brand-new Notepad window created after this test starts. That keeps compatibility with hosts where `notepad` launches through a background process that does not own the visible editor window directly.

### Assumptions
- Pre-existing user Notepad windows are the actual correctness risk, so the fallback must reject any HWND already present before the test launches Notepad.

### Follow-ups for human
- If Windows Notepad launch behavior changes again, re-check whether the visible editor window still appears as a new top-level HWND even when the spawned child PID has no `MainWindowHandle`.

### Test evidence
- RED on this host: a PID-only binding failed because the spawned child process had no visible top-level window within the poll window, while a different Notepad process owned the interactive editor window.
- GREEN: `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo test --test notepad_smoke -- --test-threads=1"` passed.
- GREEN: `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo fmt --check"` passed.
- GREEN: `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo clippy --all-targets --all-features -- -D warnings"` passed.
- GREEN: `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"` passed after restricting the fallback to a newly created foreground Notepad window not present before spawn.
