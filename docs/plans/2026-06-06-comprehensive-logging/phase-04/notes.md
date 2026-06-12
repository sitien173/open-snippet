# Phase 4 — Decision Notes

## Task 1
- Decisions made: Created log filter utilities in `src/routes/logs/filter.ts` with TypeScript types for normalized entries. Added `@tanstack/react-virtual` to dependencies.
- Spec deviations: - none
- Tradeoffs accepted: - none
- Assumptions: - none
- Follow-ups for human: - none
- Test evidence: - none

## Task 2
- Decisions made: Wrote comprehensive unit tests for `mergeEntries` and `applyFilter` in `src/routes/logs/__tests__/filter.test.ts`. Verified time-sorting, stable-tie ordering, per-criterion narrowing, and edge cases.
- Spec deviations: - none
- Tradeoffs accepted: - none
- Assumptions: - none
- Follow-ups for human: - none
- Test evidence: Vitest test run showed all 9 new filter tests passed successfully.

## Task 3
- Decisions made: Implemented `LogsRoute` (in `src/routes/logs/index.tsx`) and `LogRow` (in `src/routes/logs/LogRow.tsx`). Configured 500ms polling with deduplication on frontend ring using a stable reference Set. Added pause/resume, clear (local state only), copy, and save as JSON-lines download. Virtualized using `@tanstack/react-virtual` with dynamic height measurement (`measureElement`) to support expandable detail views. Included auto-scroll to bottom behavior.
- Spec deviations: - none
- Tradeoffs accepted: - none
- Assumptions: Rust IPC arguments are camelCased by Tauri v2 default deserializer (so we pass `{ sinceSeq: ... }`).
- Follow-ups for human: - none
- Test evidence: - none

## Task 4
- Decisions made: Registered `/logs` route in `src/main.tsx`; added a `HotkeyHandler` wrapper component listening for `Ctrl+Shift+L` (`Meta+Shift+L` on macOS) that navigates to `/logs`, bailing if focus is on an input/textarea/contentEditable. Added a "Logs" `<Link to="/logs">` in `src/routes/settings/index.tsx`. Added `index.test.tsx` (route render + Save action JSON-lines blob) and `index.a11y.test.tsx` (axe scan + `role="log"`).
- Coordinator finish: Worker timed out mid-task before committing. Coordinator finalized the on-disk state with two trivial fixes: replaced one `as any` cast in `index.test.tsx:134` with `as unknown as HTMLAnchorElement`; removed unused `React` import from `src/routes/logs/index.tsx:1` (TS6133). All `Done When` checks pass.
- Spec deviations: - none
- Tradeoffs accepted: Browser-style Blob download for Save (no Tauri fs plugin) — works inside the Tauri webview and `pnpm dev` alike.
- Assumptions: - none
- Follow-ups for human: - none
- Test evidence: `pnpm test` → 12 files / 49 tests pass; `pnpm lint` clean; `pnpm build` clean.
