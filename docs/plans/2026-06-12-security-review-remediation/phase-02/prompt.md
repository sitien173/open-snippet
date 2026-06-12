## Original User Request
Complete the 2026-06-12 security review remediation plan for the openmacro Tauri app. This phase implements the backend/runtime safety findings from that plan.

## Phase
Phase 2: Runtime Safety And Injection Ordering - make expansion side effects, clipboard paste, form submission, and foreground denylisting match the intended runtime safety model.

## Tasks
- task-1: Add failing tests proving form snippets do not execute shell or clipboard vars before submit/cancel, and execute side-effectful resolvers only once after successful submit.
- task-2: Detect form-required snippets before placeholder resolution, open the form first, and resolve placeholders once after submitted values are available.
- task-3: Fix clipboard paste ordering so paste happens before clipboard restore, with abstraction-level tests and gated Windows smoke coverage for long replacements.
- task-4: Wire real foreground-window change detection into production denylist state, reset the matcher buffer, and disarm manual confirmation on foreground changes.

## Context
- The remediation plan is `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/PLAN.md`.
- Phase 1 is complete and squashed. Its journal is `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-01/journal.md`.
- Phase 1 chose conservative backend validation; keep Phase 2 equally focused and avoid frontend changes.
- Current handover says Phase 2 is the active phase and Codex should own it.
- The project AGENTS rules require every shell command to be prefixed with `rtk`.
- Use the repo's existing Rust test seams where possible. Add seams only when needed to prove ordering or side-effect behavior.
- Treat this as bugfix/security remediation: identify root-cause evidence and use RED-to-GREEN tests before implementation.

## Files
- `F:/projects_new/textblaze/src-tauri/src/engine/orchestrator.rs`
- `F:/projects_new/textblaze/src-tauri/src/expand/resolver.rs`
- `F:/projects_new/textblaze/src-tauri/src/inject/mod.rs`
- `F:/projects_new/textblaze/src-tauri/src/inject/clipboard.rs`
- `F:/projects_new/textblaze/src-tauri/src/inject/sendinput.rs`
- `F:/projects_new/textblaze/src-tauri/src/hook/mod.rs`
- `F:/projects_new/textblaze/src-tauri/src/hook/thread.rs`
- `F:/projects_new/textblaze/src-tauri/src/hook/winevent.rs`
- `F:/projects_new/textblaze/src-tauri/src/form/runner.rs`
- `F:/projects_new/textblaze/src-tauri/tests/form_focus.rs`
- `F:/projects_new/textblaze/src-tauri/tests/shell_runner.rs`
- `F:/projects_new/textblaze/src-tauri/tests/notepad_smoke.rs`

## Done When
- Form cancel has no shell, clipboard, injection, or focus side effect beyond closing the form.
- Shell vars in form snippets run once per successful submit when shell consent permits.
- Long replacement paste sends the snippet text, not the prior clipboard text, and restores the original clipboard afterward.
- Password-manager denylist blocks expansions in listed processes during real runtime operation.
- Snippet IDs in form window URLs are encoded on the backend side.
- `cd F:/projects_new/textblaze/src-tauri && rtk cargo test form_focus shell_runner notepad_smoke -- --test-threads=1`
- `cd F:/projects_new/textblaze/src-tauri && rtk cargo test --all-features -- --test-threads=1`
- Address the gated smoke check: `cd F:/projects_new/textblaze/src-tauri && OPENMACRO_E2E=1 rtk cargo test notepad_smoke -- --ignored --test-threads=1`. If it cannot run in the worker environment, record the exact reason and keep the test gated/skipped cleanly by default.

## Rules
Follow the contract in `F:/projects_new/textblaze/.agents/shared/worker-contract.md` - per-task workflow (test-first -> one commit per task `phase-2.task-<M>: ...` -> append a `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-02/notes.md` -> append the `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-02/journal.md`) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear -> CLARIFICATIONS NEEDED + stop).

Also follow the backend domain-rules file: `F:/projects_new/textblaze/.agents/BACKEND.md`. Hard rules in that file override conflicting guidance in this spec; surface the conflict via CLARIFICATIONS NEEDED before deviating.

## Response Format
Respond per `F:/projects_new/textblaze/.agents/shared/erp.md` - return the `# EXTERNAL RESPONSE` block, then the single completion line.

## Same-phase fix
Reuse the cached `SESSION_ID`. Send `FIX:` plus only the delta files and delta context. The fix still gets its own task commit and, if it changes a decision, an appended notes block.
