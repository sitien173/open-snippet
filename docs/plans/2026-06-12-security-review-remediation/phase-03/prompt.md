## Original User Request
Complete the 2026-06-12 security review remediation plan for the openmacro Tauri app. This phase implements the frontend form/editor contract findings from that plan.

## Phase
Phase 3: Frontend Form Contract And Snippet Editor Correctness - align the React form route and snippet editor with the backend snippet model and Phase 2 encoded form URL contract.

## Tasks
- task-1: Add failing form tests using `kind: "form"` vars matching `snippets/default.yaml`, then render and validate those fields correctly.
- task-2: Decode encoded form route IDs and handle snippet IDs containing slashes, nested paths, punctuation, and Unicode.
- task-3: Update snippet editor collision detection to compare global effective triggers using current prefix and `trigger_literal`, not only same-file raw triggers.
- task-4: Add regression tests for cross-file duplicate triggers, prefix-sensitive duplicates, and default YAML form rendering.

## Context
- The remediation plan is `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/PLAN.md`.
- Phase 2 is complete and squashed. Its journal is `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-02/journal.md`.
- Phase 2 changed the backend form route to percent-encode snippet IDs before opening `/form/<encoded-id>`. Phase 3 must decode that route on the frontend and preserve IDs containing slash, punctuation, nested paths, spaces, and Unicode.
- The shipped YAML compatibility baseline is `F:/projects_new/textblaze/snippets/default.yaml`, especially the default `;for` form snippet.
- Keep this frontend-only. Backend validation from Phases 1 and 2 remains authoritative; do not weaken it.
- This is bugfix/security remediation: use test-first workflow and record RED-to-GREEN evidence in notes.
- The project AGENTS rules require every shell command to be prefixed with `rtk`.

## Files
- `F:/projects_new/textblaze/src/routes/form/index.tsx`
- `F:/projects_new/textblaze/src/routes/form/FieldRenderer.tsx`
- `F:/projects_new/textblaze/src/routes/form/fields/Text.tsx`
- `F:/projects_new/textblaze/src/routes/form/fields/Textarea.tsx`
- `F:/projects_new/textblaze/src/routes/form/fields/Choice.tsx`
- `F:/projects_new/textblaze/src/routes/form/fields/NumberField.tsx`
- `F:/projects_new/textblaze/src/routes/form/__tests__/Form.test.tsx`
- `F:/projects_new/textblaze/src/routes/form/__tests__/Form.a11y.test.tsx`
- `F:/projects_new/textblaze/src/routes/settings/SnippetEditor.tsx`
- `F:/projects_new/textblaze/src/routes/settings/__tests__/SnippetEditor.test.tsx`
- `F:/projects_new/textblaze/src/lib/snippets.ts`

## Done When
- The shipped `;for` default snippet opens a usable form with Recipient and Topic fields.
- Required validation applies to `kind: form` fields and blocks empty submit.
- Nested file snippet IDs round-trip through the encoded form route.
- Creating or editing a snippet cannot produce a global effective-trigger collision that the loader later rejects.
- `cd F:/projects_new/textblaze && rtk pnpm test -- src/routes/form src/routes/settings`
- `cd F:/projects_new/textblaze && rtk pnpm lint`
- `cd F:/projects_new/textblaze && rtk pnpm build`

## Rules
Follow the contract in `F:/projects_new/textblaze/.agents/shared/worker-contract.md` - per-task workflow (test-first -> one commit per task `phase-3.task-<M>: ...` -> append a `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-03/notes.md` -> append the `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-03/journal.md`) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear -> CLARIFICATIONS NEEDED + stop).

Also follow the frontend domain-rules file: `F:/projects_new/textblaze/.agents/FRONTEND.md`. Hard rules in that file override conflicting guidance in this spec; surface the conflict via CLARIFICATIONS NEEDED before deviating.

## Response Format
Respond per `F:/projects_new/textblaze/.agents/shared/erp.md` - return the `# EXTERNAL RESPONSE` block, then the single completion line.

## Same-phase fix
Reuse the cached `SESSION_ID`. Send `FIX:` plus only the delta files and delta context. The fix still gets its own task commit and, if it changes a decision, an appended notes block.
