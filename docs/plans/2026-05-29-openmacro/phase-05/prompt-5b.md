## Original User Request
openmacro Phase 5b — React Settings UI on top of the Phase 5a IPC surface. Snippet list (with broken-file badges), snippet editor with vars panel, prefs panel, sync panel placeholder, vitest coverage, axe-core clean. SyncPanel is a non-wired form for now (real wiring in Phase 9).

## Phase
Build the Settings webview at `/settings`. Components consume the Tauri commands added in Phase 5a (`list_snippets`, `save_snippet`, `reload_snippets`, `list_load_errors`, `get_prefs`, `set_prefs`). All command calls go through a single typed wrapper module — components never call `invoke` directly. Add Vitest + axe-core. UI must be keyboard-navigable, axe-clean.

## Tasks
- task-1: **Tooling setup** — Add to `F:/projects_new/textblaze/openmacro/package.json`:
  - deps: `react-router-dom@^6`, no Monaco (use plain `<textarea>` with monospace + `$|$` highlight via a simple span overlay or just a styled textarea — Monaco is out-of-scope).
  - devDeps: `vitest@^2`, `@testing-library/react@^16`, `@testing-library/user-event@^14`, `@testing-library/jest-dom@^6`, `jsdom@^25`, `@axe-core/react@^4`, `axe-core@^4`, `eslint`, `eslint-plugin-react`, `eslint-plugin-react-hooks`, `@typescript-eslint/parser`, `@typescript-eslint/eslint-plugin`.
  - scripts: `"test": "vitest run"`, `"test:watch": "vitest"`, `"lint": "eslint src --max-warnings=0"`.
  - Create `F:/projects_new/textblaze/openmacro/vitest.config.ts` (environment: `jsdom`, setupFiles: `./src/test-setup.ts`).
  - Create `F:/projects_new/textblaze/openmacro/src/test-setup.ts` (`import '@testing-library/jest-dom'`).
  - Create `F:/projects_new/textblaze/openmacro/.eslintrc.cjs` (typescript + react-hooks).
  - Modify `F:/projects_new/textblaze/openmacro/tsconfig.json` to include `vitest/globals` types.
  - Run `pnpm install` (or `npm install` — match whatever lockfile already exists; if neither, use `pnpm`).
- task-2: **`src/lib/snippets.ts`** — Typed wrapper over `@tauri-apps/api/core` `invoke`. Export:
  - `type VarKind = "text"|"textarea"|"choice"|"number"|"datetime"|"clipboard"|"cursor"|"shell"|"form"`
  - `type VarDecl = {name:string; kind:VarKind; label?:string; default?:string; required?:boolean; options?:string[]; format?:string}`
  - `type Snippet = {id:string; trigger:string; replace:string; vars:VarDecl[]; source_file:string; file_relative:string}`
  - `type LoadErrorDto = {path:string; message:string}`
  - `type Prefs = {paused:boolean; autostart:boolean; max_expansion_len:number; shell_consent:boolean}`
  - Functions: `listSnippets()`, `listLoadErrors()`, `saveSnippet(payload)`, `reloadSnippets()`, `getPrefs()`, `setPrefs(prefs)`. Each is `invoke<T>("command_name", args)`.
  - Add a `MockedInvoke` symbol pattern so tests can stub: detect `window.__OPENMACRO_MOCK_INVOKE` and call it if set; otherwise call the real `invoke`. This avoids fighting the Tauri runtime in vitest.
- task-3: **Routes + components** — Create:
  - `src/routes/settings/index.tsx` — top-level layout: left nav with `Snippets | Prefs | Sync`, right panel renders the active tab. Wraps `<SnippetList />`, `<PrefsPanel />`, `<SyncPanel />`.
  - `src/routes/settings/SnippetList.tsx` — fetches via `listSnippets()` + `listLoadErrors()`. Groups by `file_relative`. Each row: trigger + first 60 chars of replace. Broken-file group renders with red badge and tooltip showing the load-error message. Click row → opens `<SnippetEditor>` in a modal/inline pane.
  - `src/routes/settings/SnippetEditor.tsx` — fields: trigger (≤32 chars, non-empty, inline error on collision), body (textarea monospace, shows `$|$` highlighted via simple regex-wrapping span), `<VarsPanel>`. Save button calls `saveSnippet`, then `reloadSnippets`, then refreshes list. Cancel discards.
  - `src/routes/settings/VarsPanel.tsx` — table of `VarDecl` rows; add row, remove row, dropdown for `kind`, text inputs for `label`/`default`, checkbox for `required`, optional `format`/`options` based on kind (`datetime` → format, `choice` → comma-separated options).
  - `src/routes/settings/PrefsPanel.tsx` — `getPrefs` on mount; toggles for `paused`, `autostart`, `shell_consent` (read-only display — note "flipped from confirm dialog only"), number input for `max_expansion_len` (1..=131072). Each change calls `setPrefs(full prefs)` (no debouncing required; spec doesn't demand it). Optimistic update with rollback on error.
  - `src/routes/settings/SyncPanel.tsx` — form fields (remote URL, auth type radio HTTPS/SSH, PAT text input). No backend wiring. A `<button disabled>` "Save (coming in Phase 9)".
  - Update `F:/projects_new/textblaze/openmacro/src/main.tsx` to mount `<BrowserRouter>` with `/settings` → `<Settings />` (and a redirect from `/` to `/settings`).
- task-4: **Tests** — `src/routes/settings/__tests__/SnippetEditor.test.tsx`:
  - Renders editor with a fixture snippet; typing into trigger updates state.
  - Empty trigger → inline error, save button disabled.
  - Trigger >32 chars → inline error, save disabled.
  - Trigger collision with another existing snippet in the same file → inline error, save disabled. (Other snippets passed in as a prop, no IPC call.)
  - Vars panel: add a row, change kind to `choice`, fill options, remove a row. Final state contains the right vars[] when save is clicked (intercept via `__OPENMACRO_MOCK_INVOKE`).
  - Save round-trip: clicking save calls `save_snippet` with expected payload; reloading the list reflects the new trigger.
  - Add `src/routes/settings/__tests__/SnippetEditor.a11y.test.tsx`: render editor, run `axe(container)`, expect no violations.
  - TDD-first: stub the components minimally to make tests *fail*, watch the failures, then implement. Record RED→GREEN in `phase-05/notes.md` task-4 block.

## Context
- The Tauri IPC commands live in Phase 5a (`commands_roundtrip.rs` + `prefs_roundtrip.rs` integration tests are green). Don't change Rust code in this phase. If you find a missing field, add it as a TODO comment in the TS wrapper and call out in notes.
- Snippet IDs are `<rel-path>::<trigger>`. UI doesn't need to render the ID — show file path + trigger.
- pnpm lockfile may not exist; if `package-lock.json` exists, use `npm`. If neither, default to `pnpm`. State your choice in notes.
- Plan dir: `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-05/`. Append to existing `journal.md` and `notes.md` — do **not** overwrite the Phase 5a content. Add a fresh `## Task 5`, `## Task 6`, etc. block per task (task numbering continues from Phase 5a's task-4, so start at task-5).
- Per-task commits: `phase-5.task-<M>: <summary>` (continuing the numbering: task-5 is tooling, task-6 is wrappers, task-7 is components, task-8 is tests).

## Files
- Modify: `F:/projects_new/textblaze/openmacro/package.json`, `F:/projects_new/textblaze/openmacro/tsconfig.json`
- Create: `F:/projects_new/textblaze/openmacro/vitest.config.ts`, `F:/projects_new/textblaze/openmacro/src/test-setup.ts`, `F:/projects_new/textblaze/openmacro/.eslintrc.cjs`
- Create: `F:/projects_new/textblaze/openmacro/src/lib/snippets.ts`
- Create: `F:/projects_new/textblaze/openmacro/src/routes/settings/{SnippetList.tsx,SnippetEditor.tsx,VarsPanel.tsx,PrefsPanel.tsx,SyncPanel.tsx}` (modify `index.tsx`)
- Create: `F:/projects_new/textblaze/openmacro/src/routes/settings/__tests__/{SnippetEditor.test.tsx,SnippetEditor.a11y.test.tsx}`
- Modify: `F:/projects_new/textblaze/openmacro/src/main.tsx`

## Done When
- `cd F:/projects_new/textblaze/openmacro && pnpm test` (or `npm test`) — Vitest green, including the axe a11y test
- `cd F:/projects_new/textblaze/openmacro && pnpm lint` (or `npm run lint`) — clean
- `pnpm build` (or `npm run build`) succeeds: `tsc && vite build`
- All snippet-mutation goes through `src/lib/snippets.ts` (no `invoke` calls in components)
- Trigger collision is a **blocking inline** error, not a toast

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-05/`. TDD per `test-driven-development` — failing test first for every component that has tests.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
