## Original User Request

Complete `docs/plans/2026-06-12-manual-trigger-and-fixes`, routing failing work to another worker when needed.

## Phase

Phase 5: Manual trigger feature (frontend + migration).

## Route

# ROUTE
- Owner: Gemini
- Reason: Frontend settings UI and migration notice with Vitest coverage.
- Done When: `pnpm test`; `pnpm lint`; `pnpm build`; and if backend IPC is touched, `cd src-tauri && cargo test commands_roundtrip` plus `cd src-tauri && cargo fmt --check`.

## Tasks

- task-1: Write failing tests first for the expand-mode settings UI and migration notice. Cover Manual/Auto radio state, persistence payloads, preserving `trigger_prefix` when changing `expand_mode`, preserving `expand_mode` when saving `trigger_prefix`, upgrade notice shows only when the backend reports missing `expand_mode`, notice dismiss persists, and fresh install/default settings shows no notice.
- task-2: Update frontend bindings and preferences UI for `expand_mode`. Extend `src/lib/snippets.ts` with `ExpandMode = "manual" | "auto"` and `StoreSettings.expand_mode`. Add Manual/Auto radio controls in `PrefsPanel`, persist through existing `setStoreSettings`, and avoid resetting `expand_mode` when saving trigger prefix.
- task-3: Implement the one-time migration notice. It should be a non-blocking banner/toast in Settings, dismissible, and it must not block typing or settings changes. Dismiss should persist by writing the current settings with explicit `expand_mode: "manual"` when the backend reports the field was absent.
- task-4: If the frontend cannot reliably know field absence from the current IPC response, add the smallest backend IPC support needed to expose it. Prefer a get-settings DTO with a transient `expand_mode_missing` boolean over making the frontend parse files. Add focused Rust coverage if backend code changes.

## Backend Contract From Phase 4

- `StoreSettings` now has:
  - `trigger_prefix: string`
  - `expand_mode: "manual" | "auto"`
- Missing `expand_mode` defaults to Manual in backend deserialization.
- Existing `set_store_settings` rewrites `_settings.yaml` and triggers runtime reload.
- Existing Phase 3 prefix save currently sends only `trigger_prefix`; Phase 5 must fix the frontend so saving prefix preserves the loaded `expand_mode`.

## Migration Requirement

The design requires absence-based migration:
- Fresh install defaults to Manual and should not show the notice.
- Upgrade means `_settings.yaml` exists but lacks `expand_mode`; show the notice once.
- Dismiss persists, and subsequent launches should not show the notice.
- Detection must use absence of the `expand_mode` field, not version comparison and not frontend guesswork.

Because Phase 4 defaulting hides absence in the plain settings object, it is acceptable in this phase to add minimal backend IPC metadata such as `expand_mode_missing` to the `get_store_settings` response. Keep `set_store_settings` accepting the actual persisted settings shape.

## UI Requirements

- Add an "Expand mode" radio/segmented control to Preferences with choices:
  - Manual: snippets expand on Tab/Enter after typing the trigger.
  - Auto: snippets expand immediately when the trigger is typed.
- Keep visual style consistent with existing `PrefsPanel`; do not create a landing page or separate route.
- The migration notice copy should be concise, for example: "Snippets now expand on Tab/Enter. Change this in Settings."
- Use existing warning-card/badge/panel patterns where practical.
- No duplicate backend validation.

## Files Likely In Scope

- `F:/projects_new/textblaze/src/lib/snippets.ts`
- `F:/projects_new/textblaze/src/routes/settings/PrefsPanel.tsx`
- `F:/projects_new/textblaze/src/routes/settings/__tests__/PrefsPanel.test.tsx`
- `F:/projects_new/textblaze/src/routes/settings/index.tsx` only if parent state needs to track `expand_mode`
- `F:/projects_new/textblaze/src-tauri/src/commands/snippets.rs` only if exposing absence metadata
- `F:/projects_new/textblaze/src-tauri/tests/commands_roundtrip.rs` only if backend code changes

## Rules

Follow `F:/projects_new/textblaze/.agents/shared/worker-contract.md` and `F:/projects_new/textblaze/.agents/shared/erp.md`.

Feature work is test-first. Do not write production code until a failing test exists and RED has been observed. Append one `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-05/notes.md` after each task. Make one commit per task with subjects `phase-5.task-<M>: <summary>`. Append the required `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-05/journal.md`.

Use `coderecall` for semantic codebase search, `tgrep` for literal search, and do not use `rg`, `grep`, or web search for codebase lookup in this repository. Prefix shell commands with `rtk`.

## Response Format

Return the `# EXTERNAL RESPONSE` block, then the single completion line:

`Phase 5 completed. Journal: docs/plans/2026-06-12-manual-trigger-and-fixes/phase-05/journal.md.`
