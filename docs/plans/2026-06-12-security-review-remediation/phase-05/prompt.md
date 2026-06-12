# Phase 5 Implementer Prompt

You are the Codex backend/infra worker for Phase 5 of `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/PLAN.md`.

Read first:
- `F:/projects_new/textblaze/.agents/BACKEND.md`
- `F:/projects_new/textblaze/.agents/shared/worker-contract.md`
- `F:/projects_new/textblaze/.agents/shared/erp.md`
- `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-05/journal.md`

Hard constraints:
- Use absolute paths in any durable notes.
- Prefix every shell command with `rtk`.
- Do not use `rg`, `grep`, `ag`, public web tools, or broad search. Use known file reads or `tgrep` if literal search is needed.
- Keep changes surgical and limited to the Phase 5 files unless a test proves a directly required adjacent file.
- Feature/bugfix work is test-first. For each task, record RED then GREEN evidence in `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-05/notes.md`.
- Make one commit per task, subject `phase-5.task-<N>: <summary>`.
- Do not commit `.agents`, `.ogrep`, `dist`, `target`, or unrelated files.

Phase goal:
Add verification and CI checks that would have caught the review regressions before manual review.

Tasks:
1. Add `pnpm build` and `cargo fmt --check` to CI.
2. Extend the gated Windows Notepad smoke test so it exercises actual trigger expansion, including one long replacement that uses the paste path.
3. Add or document targeted verification commands for security-boundary, form, injection, logging, and CI regressions.
4. Run the full integration matrix and record residual manual-only risks.

Acceptance criteria:
- CI fails on TypeScript build errors and Rust formatting drift.
- Gated smoke covers real Notepad input/output when `OPENMACRO_E2E=1` and skips cleanly otherwise.
- The remediation plan has no untested CRITICAL/HIGH findings left.
- Manual-only checks are explicitly listed with exact commands or steps.

Reviewer checklist:
- CI remains practical for pull requests and does not require GUI-only tests by default.
- E2E tests are gated and skipped cleanly when prerequisites are absent.
- Documentation points to commands that actually exist.
- No unrelated formatting churn is included.

Integration checks:
- `rtk pnpm install --frozen-lockfile`
- `rtk pnpm build`
- `rtk pnpm test`
- `rtk pnpm lint`
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo fmt --check"`
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo test --all-features -- --test-threads=1"`
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo clippy --all-targets --all-features -- -D warnings"`

Before returning:
- Append/update `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-05/journal.md` with the full `# EXTERNAL RESPONSE` block.
- Ensure `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-05/notes.md` has a `## Task <N>` block for every task with Decisions made, Spec deviations, Tradeoffs accepted, Assumptions, Follow-ups for human, and Test evidence.
- Return the full `# EXTERNAL RESPONSE` block and the final line: `Phase 5 completed. Journal: docs/plans/2026-06-12-security-review-remediation/phase-05/journal.md.`
