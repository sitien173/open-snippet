# Phase 1 - Backend Trust Boundaries
- Status: REVIEWED
- Owner: codex
- Started: 2026-06-12T14:17:45+07:00
- Finished: 2026-06-12T15:30:03+07:00

## Route
- Reason: Backend/security-boundary phase touching Rust IPC validation, storage, credentials, prefs, and autostart behavior.
- Done When:
  - `cd src-tauri && cargo test commands_roundtrip sync_roundtrip prefs_roundtrip store_yaml -- --test-threads=1`
  - `cd src-tauri && cargo test --all-features -- --test-threads=1`
  - `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`
- Files:
  - `src-tauri/src/commands/snippets.rs`
  - `src-tauri/src/store/loader.rs`
  - `src-tauri/src/commands/sync.rs`
  - `src-tauri/src/sync/creds.rs`
  - `src-tauri/src/sync/mod.rs`
  - `src-tauri/src/commands/prefs.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/tests/commands_roundtrip.rs`
  - `src-tauri/tests/sync_roundtrip.rs`
  - `src-tauri/tests/prefs_roundtrip.rs`
  - `src-tauri/tests/store_yaml.rs`

## Cross-Validation
- Status: COMPLETE
- Codex: completed after user-approved retry with `timeout_s=0`. Session: `019ebab2-7241-77a0-95d1-bea4e4c9de92`.
- Gemini: completed. Session: `c1eb1c51-2957-489a-9e5d-d426d397eae7`.
- Agreement:
  - Phase 1 is backend-only trust-boundary hardening. Add failing Rust regressions first, then enforce backend validation before filesystem, credential, prefs, or autostart side effects.
  - `save_snippet_inner` must treat caller paths as untrusted and reject absolute paths, `..` escapes, sibling-prefix escapes, Windows separator/case tricks, drive/UNC/device paths, `_settings.yaml`, and non-YAML extensions before read/write.
  - The loader must never read final canonical targets outside the snippets root. Phase 1 should choose and document the exact symlink/junction policy.
  - HTTPS PAT storage and callbacks must validate that the actual remote URL host matches the credential host. Host equality is the required minimum; exact remote matching is optional only if it fits the existing credential model cleanly.
  - Credential callbacks must fail closed on URL/host mismatch without returning or logging secret material.
  - Safety-sensitive prefs need backend validation on IPC and persisted-load paths, especially `max_expansion_len`.
  - Autostart changes must call the Tauri autostart plugin and persist prefs only after enable/disable succeeds.
  - Prefs/autostart also need RED-to-GREEN regression evidence even though the Phase 1 task list only calls out failing tests explicitly for path and credential work.
- Divergences:
  - PAT scope: Codex raised exact remote matching; Gemini and the plan require host matching. Resolution: enforce host equality as the required contract, add exact remote checks only if they do not distort the current credential model.
  - Symlink policy: both agents noted the policy is underspecified. Resolution: Codex must make the policy explicit in implementation and tests, with the invariant that resolved targets outside the snippets root are always rejected.
  - `max_expansion_len`: exact numeric bounds are unspecified. Resolution: Codex should derive conservative bounds from existing defaults/tests and record the chosen bounds in notes.
- Next owner: Codex.

## External Response
# EXTERNAL RESPONSE
## META
- Phase 1 / Owner codex / SessionID 019ebab2-7241-77a0-95d1-bea4e4c9de92 / Started 2026-06-12T14:17:45+07:00 / Finished 2026-06-12T15:30:03+07:00 / Plan dir docs/plans/2026-06-12-security-review-remediation
## SUMMARY
Phase 1 hardened backend snippet path, loader symlink/root, HTTPS PAT, prefs bounds, and autostart trust boundaries with regression coverage.
## FILES MODIFIED
| Action | Path | Change |
| Modify | src-tauri/src/commands/snippets.rs | Validate snippet save paths under the snippets root before read/write. |
| Modify | src-tauri/src/store/loader.rs | Reject symlinked or root-escaping snippet files during recursive load. |
| Modify | src-tauri/src/commands/sync.rs | Validate HTTPS PAT remote/auth pairing before credential storage or backend init/test. |
| Modify | src-tauri/src/sync/creds.rs | Add HTTPS host validation and callback URL mismatch rejection before secret lookup. |
| Modify | src-tauri/src/sync/mod.rs | Re-export sync credential validation helpers for command/tests. |
| Modify | src-tauri/src/commands/prefs.rs | Validate prefs bounds and apply autostart through the Tauri plugin before persistence. |
| Modify | src-tauri/src/engine/orchestrator.rs | Add a focused clippy allow on an existing helper required by the all-target clippy gate. |
| Modify | src-tauri/tests/commands_roundtrip.rs | Add snippet save path regressions and clean existing clippy test warnings. |
| Modify | src-tauri/tests/store_yaml.rs | Add loader symlink escape regression. |
| Modify | src-tauri/tests/sync_roundtrip.rs | Add HTTPS PAT host and callback mismatch regressions. |
| Modify | src-tauri/tests/prefs_roundtrip.rs | Add prefs bounds and autostart apply-before-persist regressions. |
| Modify | src-tauri/tests/matcher_manual_mode.rs | Mark intentional guard field for clippy all-target gate. |
| Modify | src-tauri/tests/matcher_armed_dismiss.rs | Mark intentional guard field for clippy all-target gate. |
| Create | docs/plans/2026-06-12-security-review-remediation/phase-01/notes.md | Record task decisions, deviations, and RED-to-GREEN evidence. |
## COMMITS
- phase-1.task-1: 72e8930  phase-1.task-1: add snippet path boundary tests
- phase-1.task-2: 0d78994  phase-1.task-2: validate snippet storage paths
- phase-1.task-3: 0d57320  phase-1.task-3: validate HTTPS PAT hosts
- phase-1.task-4: 21c338e  phase-1.task-4: validate prefs and autostart
## NOTES
- phase-01/notes.md (## Task 1, ## Task 2, ## Task 3, ## Task 4, ## Integration Check Fixes)
## SPEC COMPLIANCE
- Meets Spec? YES - Phase 1 acceptance criteria and integration checks pass.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS
- Quality Findings: No findings
- Final Status: PASS
- Explanation: Required Phase 1 tests and clippy gate pass; extra lint-only files were touched only to satisfy the plan's all-target clippy check.
- Next: squash task commits into `phase-1: backend trust boundaries`, then advance to Phase 2.

## Squash Commit
- phase-1: backend trust boundaries

## Decisions
- Phase 1 requires a narrow cross-validation checkpoint before Codex implementation because it changes credential and filesystem trust boundaries.

## Handoff
Phase 1 passed review and was squashed into `phase-1: backend trust boundaries`.
RED-to-GREEN evidence and implementation decisions are recorded in `phase-01/notes.md`.

Passing checks:
- `rtk cargo test --test commands_roundtrip --test sync_roundtrip --test prefs_roundtrip --test store_yaml -- --test-threads=1` passed 31 tests.
- `rtk cargo test --all-features -- --test-threads=1` passed 137 tests, 1 ignored.
- `rtk cargo clippy --all-targets --all-features -- -D warnings` passed with no issues.

Next step: open Phase 2 Runtime Safety And Injection Ordering and route it to Codex.
