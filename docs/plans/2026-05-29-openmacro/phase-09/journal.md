# Phase 9 — Git-backed sync

- Status: IN_PROGRESS
- Owner: codex (back-side dominant; trivial SyncPanel wiring bundled)
- Started: 2026-06-06
- Finished: —

## Route
- Reason: Mostly back-side (git2 cycle, credential manager, conflict folder, tokio driver). SyncPanel wiring is a small companion.
- Done When: cargo tests green incl. sync_roundtrip + sync_conflict; clippy clean; pnpm tests green incl. SyncPanel; conflict folder monotonic; PAT grep clean.

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 9 / codex / n-a / 2026-06-06 / 2026-06-06 14:08:25 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented git-backed sync with a `git2` backend, Windows Credential Manager PAT handling, periodic/on-change driver wiring, tested round-trip/conflict flows, and the Settings sync panel command surface.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-09/notes.md` | Added per-task decisions, deviations, assumptions, and RED→GREEN evidence. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-09/journal.md` | Appended this external response block. |
| Modify | `openmacro/src-tauri/Cargo.toml` | Added `git2` and the Windows credential-manager feature set required by sync. |
| Modify | `openmacro/src-tauri/Cargo.lock` | Locked the Phase 9 dependency graph updates. |
| Create/Modify | `openmacro/src-tauri/src/sync/{mod.rs,git.rs,creds.rs,conflicts.rs}` | Added the sync trait/backend, Windows credential store, conflict capture helpers, notification surface, and interval/file-change driver. |
| Create | `openmacro/src-tauri/src/commands/sync.rs` | Added sync test/init/tick/status commands plus testable inner helpers and DTO mapping. |
| Modify | `openmacro/src-tauri/src/commands/mod.rs` | Registered the sync command module. |
| Modify | `openmacro/src-tauri/src/lib.rs` | Registered sync commands, managed sync state, and spawned the watcher-driven sync driver. |
| Create | `openmacro/src-tauri/tests/{sync_roundtrip.rs,sync_conflict.rs}` | Added integration coverage for two-client sync propagation and conflict capture/notification. |
| Create | `openmacro/src/lib/sync.ts` | Added typed frontend wrappers for sync commands and DTOs. |
| Modify | `openmacro/src/routes/settings/SyncPanel.tsx` | Replaced the old reload-diagnostics panel with remote/auth/test/init/tick/status controls. |
| Create | `openmacro/src/routes/settings/__tests__/SyncPanel.test.tsx` | Added Vitest coverage for connection testing, HTTPS PAT validation, and sync-now feedback. |
## COMMITS
- phase-9.task-1: 1fd42b92e2bcfc8272f38b2961c90b1c354d1a5c  phase-9.task-1: add git sync backend scaffold
- phase-9.task-2: 8159785cd57594c5656a9145778939af64e7e61d  phase-9.task-2: add credential manager support
- phase-9.task-3: acb0add7c094ed4dab3ce11692cf7358d4a659d4  phase-9.task-3: add sync command surface
- phase-9.task-4: dd4503d324b818b5e9836edc22b176f4ff27f7aa  phase-9.task-4: wire sync settings panel
## NOTES
- phase-09/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? YES  — `cargo test --all-features -- --test-threads=1`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm test`, and `pnpm lint` all passed; sync round-trip/conflict coverage is in place, conflict captures land under `.conflicts/<unix-ts>/`, and PATs stay in-memory/Credential Manager only with redacted debug output.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review

## Squash Commit

## Decisions
- per-task in notes.md.

## Handoff
- Phase 10 (codex, polish + installer + crash reporting).
