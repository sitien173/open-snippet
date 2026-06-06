# Phase 7 — Decision Notes

## Task 1
- Decisions made (not in spec): - none
- Spec deviations: - none
- Tradeoffs accepted: - none
- Assumptions: - none
- Follow-ups for human: - none
- Test evidence:
  - RED: Failed to resolve import `../form` in `src/lib/__tests__/form.test.ts`.
  - GREEN: `src/lib/__tests__/form.test.ts` passed successfully with `formSubmit` and `formCancel` calling `safeInvoke` with `"form_submit"` and `"form_cancel"`.

## Task 2
- Decisions made (not in spec): - none
- Spec deviations: - none
- Tradeoffs accepted: - none
- Assumptions: - none
- Follow-ups for human: - none
- Test evidence:
  - RED: Failed to resolve import `../fields/Text` in `src/routes/form/__tests__/Form.test.tsx`.
  - GREEN: `src/routes/form/__tests__/Form.test.tsx` passed successfully verifying the rendering and interaction of Text, Textarea, Choice, and NumberField components.

## Task 3
- Decisions made (not in spec): added `Form.css` for light/dark styling; added `noValidate` on the `<form>` so jsdom honors custom validation; hoisted `handleCancel` above the `useEffect` that references it; ResizeObserver mocked globally in `src/test-setup.ts`.
- Spec deviations: none material.
- Tradeoffs accepted: global ResizeObserver mock vs per-test injection.
- Assumptions: BrowserRouter remains the right router for the Tauri webview at this stage.
- Follow-ups for human: manual end-to-end smoke test via `pnpm tauri dev` once a form snippet exists in the store.
- Test evidence:
  - RED: `src/routes/form/__tests__/Form.test.tsx` failed (no `FormRoute` export, `/form/:snippetId` route missing).
  - GREEN: passes after `FieldRenderer.tsx`, `index.tsx`, and `main.tsx` wiring.

## Task 4
- Decisions made (not in spec): a11y test co-located as `Form.a11y.test.tsx`.
- Spec deviations: none.
- Tradeoffs accepted: axe runs on rendered FormRoute, not each field in isolation.
- Assumptions: same axe-core config as the Phase 5b SnippetEditor a11y test.
- Follow-ups for human: none.
- Test evidence:
  - RED: `Form.a11y.test.tsx` failed importing `FormRoute` before route wiring.
  - GREEN: axe reports zero violations; total suite 21/21 green.

## Coordinator note
- Gemini timed out before staging task-3/task-4 commits. The coordinator verified `pnpm test`, `pnpm lint`, `pnpm build` all green, then committed the staged work and authored the EXTERNAL RESPONSE block + this Task 3/4 notes section.

