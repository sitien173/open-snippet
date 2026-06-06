# Phase 8 — Shell snippets

- Status: IN_PROGRESS
- Owner: codex (includes small front-side ShellConsentDialog companion)
- Started: 2026-06-06
- Finished: —

## Route
- Reason: Mostly back-side (loader validation, sandboxed runner, Job Object kill-tree, resolver gate). Trivial companion dialog UI bundled rather than split.
- Done When: cargo test green incl. shell_runner; clippy clean; pnpm test green incl. ShellConsentDialog; loader rejects string `cmd`; resolver returns `ShellDisabled` when consent off; Job Object kill-tree wired.

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 8 / codex / n-a / 2026-06-06 / 2026-06-06 13:18:30 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented shell snippet support end-to-end: loader validation, sandboxed argv-only execution with timeout/job-object handling, consent-aware resolver/orchestrator wiring, and the one-shot Settings consent dialog.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-08/notes.md` | Added per-task decisions, deviations, assumptions, and RED→GREEN evidence. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-08/journal.md` | Appended this external response block. |
| Modify | `openmacro/src-tauri/Cargo.toml` | Added `tokio` process support and the Windows security/job-object feature set needed by the shell runner. |
| Modify | `openmacro/src-tauri/Cargo.lock` | Locked the Phase 8 dependency graph updates. |
| Modify | `openmacro/src-tauri/src/store/{model.rs,loader.rs}` | Added shell-var fields and loader validation/error reporting for `cmd` and `timeout_ms`. |
| Create/Modify | `openmacro/src-tauri/src/expand/{mod.rs,resolver.rs,shell.rs}` | Added the shell backend module, resolver consent/confirm handling, and shell-specific resolve errors/notify hooks. |
| Modify | `openmacro/src-tauri/src/engine/orchestrator.rs` | Threaded prefs + shell backend through expansion, added shell runtime tests, and kept form behavior intact. |
| Create/Modify | `openmacro/src-tauri/tests/{store_yaml.rs,shell_runner.rs,expand_resolver.rs,cursor_math.rs,form_focus.rs,commands_roundtrip.rs}` | Added shell runner coverage and updated existing resolver-facing tests for the new API and shell-valid fixtures. |
| Create | `openmacro/src/routes/settings/ShellConsentDialog.tsx` | Added the one-shot shell-consent modal component. |
| Modify | `openmacro/src/routes/settings/PrefsPanel.tsx` | Loaded shell-snippet presence on first prefs fetch and mounted the consent dialog when needed. |
| Create | `openmacro/src/routes/settings/__tests__/ShellConsentDialog.test.tsx` | Added the Vitest coverage for dialog visibility and acceptance behavior. |
## COMMITS
- phase-8.task-1: 71c00e9862158008d975a5cc5d62db8344917bb7  phase-8.task-1: validate shell loader fields
- phase-8.task-2: a2d1751686f8b318aad88afb9f04c4728da728b5  phase-8.task-2: add shell runner backend
- phase-8.task-3: fc4baa9c70d00d03d4e9e61748a1b1abc8f61f57  phase-8.task-3: wire shell consent into resolver
- phase-8.task-4: d23bed7f61bee1014859c5e42a6426f3fd0f47f5  phase-8.task-4: add shell consent dialog
## NOTES
- phase-08/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? YES  — `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm test`, and `pnpm lint` all passed; loader rejects invalid shell declarations, consent-disabled resolution returns `ShellDisabled`, and the Windows timeout path uses a kill-on-close Job Object as required.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review

## Squash Commit

## Decisions
- per-task in notes.md.

## Handoff
- Phase 9 (codex, git-backed sync) — independent.
