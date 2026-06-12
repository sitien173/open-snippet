## Original User Request
openmacro Phase 7 — Form UI (front-side). `/form/<snippet-id>` route renders fields from snippet `vars` in declared order. Submit on Enter (Shift+Enter in textarea = newline), cancel on Esc. First focusable field auto-focused. Submit calls `form_submit`, cancel calls `form_cancel`.

## Phase
Build the React form route that the Phase 6 form runner opens. The window URL is `/form/<snippet-id>`; the route fetches the snippet (it can call `listSnippets()` and find by id), renders fields, and hands values back via `form_submit` / `form_cancel`.

## Tasks
- task-1: **`src/lib/form.ts`** — Typed wrapper. Exports:
  - `formSubmit(snippetId: string, values: Record<string,string>): Promise<void>`
  - `formCancel(snippetId: string): Promise<void>`
  - Reuses the same `__OPENMACRO_MOCK_INVOKE` test-stub pattern as `src/lib/snippets.ts`.
- task-2: **Field components** — Create:
  - `src/routes/form/fields/Text.tsx` — `<input type="text">` with label + required.
  - `src/routes/form/fields/Textarea.tsx` — `<textarea>` with Shift+Enter newline (default browser behavior, just don't preventDefault).
  - `src/routes/form/fields/Choice.tsx` — `<select>` over `options[]`; empty options → render an inline error message, not a blank dropdown.
  - `src/routes/form/fields/NumberField.tsx` — `<input type="number">` honoring `required`. Stores as string (form values are `Record<string,string>` per the back-end signature).
  Each field accepts `{decl: VarDecl, value: string, onChange: (v:string) => void, autoFocus?: boolean}`.
- task-3: **Route + window plumbing** — Create:
  - `src/routes/form/FieldRenderer.tsx` — switches on `decl.kind` to render Text / Textarea / Choice / NumberField. Other kinds (datetime, clipboard, cursor, shell, form) are NOT rendered (form runner only opens for `Form`-kind vars; the renderer should skip non-form kinds defensively).
  - `src/routes/form/index.tsx` — reads `:snippet-id` from `useParams`. On mount: `listSnippets()`, find by id, render fields in declared order. Manages a `values` state map. Enter submits unless target is a textarea (textarea uses Ctrl+Enter to submit per common convention OR don't submit on Enter inside textarea — pick textarea-safe default: Enter in non-textarea inputs submits; Enter in textarea inserts a newline; explicit Submit button always works). Esc cancels. First focusable field gets `autoFocus`.
  - On submit: validate required-non-empty for `required: true` declared fields; if invalid, focus the first invalid field and inline-error; if valid, call `formSubmit(id, values)`, then close the window via `getCurrentWindow().close()` from `@tauri-apps/api/window`.
  - On cancel: call `formCancel(id)`, then close window.
  - After layout: call `getCurrentWindow().setSize(new LogicalSize(400, ceil(content height)))` so the form auto-grows. Use a `ResizeObserver` on the form root.
  - Register the route in `src/main.tsx` BrowserRouter as `/form/:snippetId`.
- task-4: **Tests** — `src/routes/form/__tests__/Form.test.tsx`:
  - Renders each of the four field types from a synthetic snippet (use `__OPENMACRO_MOCK_INVOKE` to stub `list_snippets`).
  - Enter submits when focus in a `<input type="text">`; calls `form_submit` with the expected values.
  - Enter in a `<textarea>` inserts a newline; does NOT call `form_submit`.
  - Esc cancels; calls `form_cancel`; does NOT call `form_submit`.
  - Required-field validation: empty required text → submit blocked, error shown, `form_submit` not called.
  - Choice with empty options → renders an inline error message, not a blank dropdown.
  - `src/routes/form/__tests__/Form.a11y.test.tsx`: axe-clean.
  - TDD-first: write tests, watch them fail (no route exists yet), then implement; record RED→GREEN in `notes.md` task-4.

## Context
- `src/lib/snippets.ts` already exports `Snippet`/`VarDecl`/`listSnippets`. Reuse.
- The form runner (Phase 6) doesn't actually drive a real browser window in tests — Phase 7 tests use vitest + jsdom + the mock-invoke pattern. Manual smoke (`pnpm tauri dev` + `;for` snippet) is **out of scope** for this phase since it depends on the snippet store containing a real form snippet — note as follow-up.
- Plan dir: `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-07/`.
- Per-task commits: `phase-7.task-<M>: <summary>`.
- Use the existing testing infrastructure (vitest + @testing-library/react + axe-core) installed in Phase 5b. Do NOT modify `package.json` unless a missing dep blocks a test.

## Files
- Create: `F:/projects_new/textblaze/openmacro/src/lib/form.ts`
- Create: `F:/projects_new/textblaze/openmacro/src/routes/form/{index.tsx,FieldRenderer.tsx}`
- Create: `F:/projects_new/textblaze/openmacro/src/routes/form/fields/{Text.tsx,Textarea.tsx,Choice.tsx,NumberField.tsx}`
- Create: `F:/projects_new/textblaze/openmacro/src/routes/form/__tests__/{Form.test.tsx,Form.a11y.test.tsx}`
- Modify: `F:/projects_new/textblaze/openmacro/src/main.tsx` (add `/form/:snippetId` route)

## Done When
- `pnpm test` green (existing tests + new form tests + a11y test)
- `pnpm lint` clean
- `pnpm build` succeeds
- All IPC calls go through `src/lib/form.ts` (no direct `invoke` in components)
- Esc cancels; Enter submits (non-textarea); Shift+Enter in textarea inserts newline

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-07/`. TDD per `test-driven-development`.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
