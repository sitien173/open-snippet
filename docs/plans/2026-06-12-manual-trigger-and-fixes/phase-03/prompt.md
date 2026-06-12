## Original User Request

Complete `docs/plans/2026-06-12-manual-trigger-and-fixes`, routing failing work to another worker when needed.

## Phase

Phase 3: Custom prefix refactor (frontend).

## Route

# ROUTE
- Owner: Gemini
- Reason: Frontend settings UI, snippet editor controls, and Vitest coverage.
- Done When: `pnpm test`; `pnpm lint`; `pnpm build`; acceptance criteria below are met.

## Tasks

- task-1: Write failing Vitest coverage first for the settings prefix UI and snippet editor contract. Cover prefix preview/persistence, invalid-prefix backend error display, read-only effective trigger hint when `trigger_literal` is false, and literal toggle save payload behavior. Record RED evidence.
- task-2: Add a trigger-prefix control to the Settings preferences view using the Phase 2 IPC wrappers `getStoreSettings` and `setStoreSettings` from `F:/projects_new/textblaze/src/lib/snippets.ts`. Load current settings, persist edits, show save/error state, and preview a bare trigger like `email` using the current prefix. Let backend validation be the source of truth; do not duplicate backend validation beyond lightweight empty UI handling if already local style requires it.
- task-3: Update `SnippetEditor` so it keeps raw trigger editing, exposes a `trigger_literal` checkbox/toggle, sends `trigger_literal` and `original_trigger_literal`, and shows the read-only effective trigger hint when literal mode is off. Existing save/reload flow must continue to work.

## Backend Contract From Phase 2

- `Snippet.trigger` is the raw YAML/editable trigger.
- `Snippet.effective_trigger` is the matcher trigger after global-prefix normalization.
- `Snippet.trigger_literal` is the per-snippet literal override.
- `SaveSnippetDto.original_trigger_literal` disambiguates updates.
- `getStoreSettings()` returns `{ trigger_prefix: string }`.
- `setStoreSettings(settings)` persists `_settings.yaml` and triggers backend reload.

## Design Requirements

From the confirmed design and plan:

- Settings: prefix input with validation + live preview (`":" + "email" -> ":email"`).
- Invalid prefix shows the backend's error message inline; save is blocked or immediately rolls back to editable error state.
- Snippet editor shows effective trigger as a read-only hint when `trigger_literal` is off.
- Snippet editor exposes a toggle for `trigger_literal`; when on, the preview is hidden and save uses raw trigger.
- Changing prefix in Settings persists and updates the editor preview without restart.
- No business-logic duplication in frontend; backend validation remains authoritative.

## Files

- F:/projects_new/textblaze/src/routes/settings/index.tsx
- F:/projects_new/textblaze/src/routes/settings/PrefsPanel.tsx
- F:/projects_new/textblaze/src/routes/settings/SnippetEditor.tsx
- F:/projects_new/textblaze/src/routes/settings/SnippetList.tsx
- F:/projects_new/textblaze/src/routes/settings/__tests__/
- F:/projects_new/textblaze/src/lib/snippets.ts

## Rules

Follow `F:/projects_new/textblaze/.agents/shared/worker-contract.md` and `F:/projects_new/textblaze/.agents/shared/erp.md`.

Feature/refactor work is test-first. Do not write production code until a failing test exists and RED has been observed. Append one `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-03/notes.md` after each task. Make one commit per task with subjects `phase-3.task-<M>: <summary>`. Append the required `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-03/journal.md`.

Use `coderecall` for semantic codebase search, `tgrep` for literal search, and do not use `rg`, `grep`, or web search for codebase lookup in this repository. Prefix shell commands with `rtk`.

## Response Format

Return the `# EXTERNAL RESPONSE` block, then the single completion line:

`Phase 3 completed. Journal: docs/plans/2026-06-12-manual-trigger-and-fixes/phase-03/journal.md.`
