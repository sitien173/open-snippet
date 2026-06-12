# Phase 4 Implementer Prompt

You are the Gemini frontend worker for Phase 4 of `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/PLAN.md`.

Read first:
- `F:/projects_new/textblaze/.agents/FRONTEND.md`
- `F:/projects_new/textblaze/.agents/shared/worker-contract.md`
- `F:/projects_new/textblaze/.agents/shared/erp.md`
- `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-04/journal.md`

Hard constraints:
- Use absolute paths in any durable notes.
- Prefix shell commands with `rtk`.
- Do not use `rg`, `grep`, `ag`, public web tools, or broad search. Use known file reads or `tgrep` if literal search is needed.
- Keep changes surgical and limited to the Phase 4 files unless tests prove a directly required adjacent file.
- Feature/bugfix work is test-first. For each task, record RED then GREEN evidence in `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-04/notes.md`.
- Make one commit per task, subject `phase-4.task-<N>: <summary>`.
- Do not commit `.agents`, `.ogrep`, `dist`, or unrelated files.

Phase goal:
Prevent frontend privacy leaks and make the sync/settings UI fail earlier and more clearly.

Tasks:
1. Add failing tests that spy on console forwarding and prove sensitive fields are redacted outside explicit verbose mode.
2. Pass redacted fields to the underlying logger/console by default while preserving verbose-mode diagnostics.
3. Reject malformed HTTPS remotes in `SyncPanel` before invoking backend commands, without weakening backend validation.
4. Ensure settings save errors, especially autostart failures from Phase 1, are surfaced without leaving misleading optimistic UI state.

Acceptance criteria:
- Sensitive frontend log fields are redacted in both ring entries and console output by default.
- Verbose mode behavior is explicit and covered by tests.
- Invalid HTTPS remotes show inline validation and do not call `sync_test_connection` or `sync_init`.
- Autostart save failures are visible and do not display "Saved".

Reviewer checklist:
- Redaction does not mutate caller-owned field objects unexpectedly.
- Sync UI validation handles SSH mode separately from HTTPS mode.
- Error badges are accessible and preserve existing visual patterns.
- Tests do not assert implementation details beyond public behavior.

Integration checks:
- `rtk pnpm test -- src/lib src/routes/settings`
- `rtk pnpm lint`
- `rtk pnpm build`

Before returning:
- Append/update `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-04/journal.md` with the full `# EXTERNAL RESPONSE` block.
- Ensure `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-04/notes.md` has a `## Task <N>` block for every task with Decisions made, Spec deviations, Tradeoffs accepted, Assumptions, Follow-ups for human, and Test evidence.
- Return the full `# EXTERNAL RESPONSE` block and the final line: `Phase 4 completed. Journal: docs/plans/2026-06-12-security-review-remediation/phase-04/journal.md.`

## Same-phase FIX: Task 6

Coordinator review found a blocking gap in Task 3. Current `src/routes/settings/SyncPanel.tsx` only checks `remote.trim().toLowerCase().startsWith("https://")`; a malformed value such as `https://` still reaches `sync_init` / `sync_test_connection` with auth host `"unknown"`.

Do only this fix:
1. Add a failing frontend test proving malformed HTTPS input, for example `https://`, shows inline validation and does not call `sync_test_connection` or `sync_init`.
2. Tighten HTTPS-mode validation so it requires a parseable HTTPS URL with a non-empty host before IPC.
3. Preserve SSH-mode behavior and existing successful HTTPS behavior.
4. Append `## Task 6` to `phase-04/notes.md` with RED and GREEN evidence.
5. Update `phase-04/journal.md` by appending/replacing the external response so `## COMMITS` includes all Phase 4 task hashes including Task 6.
6. Commit only intended Phase 4 files with subject `phase-4.task-6: validate malformed https remotes`.

Run:
- `rtk pnpm exec vitest run src/routes/settings/__tests__/SyncPanel.test.tsx --reporter=dot`
- `rtk pnpm exec vitest run src/lib src/routes/settings --reporter=dot`
- `rtk pnpm lint`
- `rtk pnpm build`
