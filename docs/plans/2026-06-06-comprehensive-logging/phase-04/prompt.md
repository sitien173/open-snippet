## Original User Request
Add comprehensive developer-debugging logging to the openmacro Tauri app. Phases 1–3 are complete (Rust subsystem + IPC commands + frontend logger). Phase 4 adds the in-app `/logs` viewer route.

## Phase
Phase 4 — In-app log viewer. A new `/logs` React route that polls `invoke("get_log_ring", { sinceSeq })` (Rust ring) and reads `getRing()` (frontend ring), merges them, and renders a filterable, virtualized, pausable, savable log view. Reached via `Ctrl+Shift+L` hotkey and a link from the Settings route.

## Tasks
- task-1: Add `@tanstack/react-virtual` dep (only if not already present — verify via `package.json` before installing). Create `src/routes/logs/filter.ts` with pure functions: `mergeEntries(rustEntries: RustLogEntry[], frontEntries: FrontendLogEntry[]): UiEntry[]` (sort by `ts` ascending — Rust `ts_unix_ms` vs frontend `ts` (Date.now()) — both unix-ms), `applyFilter(rows, filter): UiEntry[]` (source: "all" | "rust" | "frontend"; minLevel: trace|debug|info|warn|error; moduleQuery: string substring on `target`/`module`; searchQuery: case-insensitive substring on `msg` plus stringified `fields`). Stable ordering on `ts` ties.
- task-2: Write `src/routes/logs/__tests__/filter.test.ts` covering `mergeEntries` time-sort with mixed-source input, `applyFilter` per-criterion + combined-criteria narrowing, empty-input edge cases, and the stable-tie ordering rule.
- task-3: Build `src/routes/logs/index.tsx` and `src/routes/logs/LogRow.tsx`. 500ms polling via `useEffect` + `setInterval` for Rust ring (track `lastSeq` in a `useRef`); merge with frontend ring on each tick. Virtualize via `@tanstack/react-virtual`. Controls: source dropdown, level dropdown, module text input, search text input, Pause/Resume button (toggles a paused state — interval pauses), Clear (clears local view, NOT the rings themselves — confirm by adding a comment), Copy (writes filtered view as JSON-lines to navigator.clipboard), Save (downloads as JSON-lines via a Blob + `URL.createObjectURL` + anchor click; default filename `openmacro-logs-<ISO>.jsonl`). Auto-scroll to bottom unless the user has scrolled up — detect via scroll position relative to scrollHeight. Container has `role="log"` and `aria-live="polite"`.
- task-4: Add Vitest coverage in `src/routes/logs/__tests__/index.test.tsx` (mock `invoke` via `window.__OPENMACRO_MOCK_INVOKE`; assert merge with frontend ring; assert pause stops polling; assert Save click constructs JSON-lines blob — spy on `URL.createObjectURL`). Add a11y test in `index.a11y.test.tsx` (axe scan, `role="log"`). Register `/logs` route in `src/main.tsx`; add a `Ctrl+Shift+L` global keydown handler (in `main.tsx` or in `App.tsx` — wherever makes sense) that navigates to `/logs`. Add a small "Logs" link in `src/routes/settings/index.tsx` pointing to `/logs`.

## Context

### Design references (read fully)
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging-design.md — section 4 (In-app log viewer) is authoritative.
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/PLAN.md — Phase 4 row.
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-03/journal.md — Phase 3 outcomes (frontend logger you'll consume).

### IPC shape (Rust → TS for `get_log_ring`)
The Rust command returns:
```ts
type RustLogEntry = {
  seq: number;                          // monotonic u64
  ts_unix_ms: number;                   // i64
  level: "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";  // tracing::Level::as_str() uppercase
  target: string;
  message: string;
  fields: Record<string, unknown>;      // serde_json::Value::Object — could also be other JSON types in edge cases; treat as object
  span_path: string[];
};
```
Frontend logger ring (`getRing()` in `src/lib/logger.ts`) returns:
```ts
type FrontendLogEntry = {
  ts: number;                           // Date.now()
  level: "trace" | "debug" | "info" | "warn" | "error";  // lowercase
  module: string;
  msg: string;
  fields?: Record<string, unknown>;
};
```

Pick a normalized **UI row type**:
```ts
type UiEntry = {
  source: "rust" | "frontend";
  ts: number;
  level: "trace" | "debug" | "info" | "warn" | "error";
  module: string;        // rust.target or frontend.module
  msg: string;           // rust.message or frontend.msg
  fields?: Record<string, unknown>;
  // optional source-specific extras for display:
  seq?: number;          // rust only
  spanPath?: string[];   // rust only
};
```
Lowercase the Rust level when normalizing.

### IPC call
```ts
import { invoke } from "@tauri-apps/api/core";
const rust = await invoke<RustLogEntry[]>("get_log_ring", { sinceSeq: lastSeq.current });
```
**Important:** the Tauri argument name as deserialized on the Rust side is `sinceSeq` (camelCase from the function signature `since_seq`). Tauri v2 converts function parameters to camelCase by default in `invoke`. Verify by reading `src-tauri/src/commands/logging.rs` if uncertain. If the first call returns 0 entries despite emissions, try `since_seq: 0` (snake_case) instead — Phase 2's worker may have used `#[tauri::command(rename_all = "snake_case")]`. Document whichever works in your task-3 notes.

### Save / Copy implementation
- **Save:** No Tauri fs/dialog plugins are installed. Use the browser download path:
  ```ts
  const text = filteredView.map(e => JSON.stringify(e)).join("\n");
  const blob = new Blob([text], { type: "application/x-ndjson" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `openmacro-logs-${new Date().toISOString().replace(/[:.]/g, "-")}.jsonl`;
  a.click();
  URL.revokeObjectURL(url);
  ```
  This works inside Tauri's webview as well as in `pnpm dev`. **Skip** the "include sensitive content" checkbox concept from the design doc — frontend redaction already happened on ingest, so saved entries are already redacted. Document in notes.
- **Copy:** `navigator.clipboard.writeText(text)`.

### Test-mode injection
Phase 3 added `window.__OPENMACRO_MOCK_INVOKE`. Reuse the same convention in viewer tests:
```ts
beforeEach(() => {
  window.__OPENMACRO_MOCK_INVOKE = async (cmd, args) => {
    if (cmd === "get_log_ring") return mockedRustEntries;
    if (cmd === "get_logging_frontend_cfg") return { level: "info", modules: {} };
    return undefined;
  };
});
afterEach(() => { delete window.__OPENMACRO_MOCK_INVOKE; });
```
For tests of `<LogsRoute />` itself, you may need to import `safeInvoke` indirectly via the route — or extract `getLogRing(sinceSeq)` into a small helper in `filter.ts` or a new `ipc.ts` and import that from the route, so tests can replace the helper. **Pick what keeps tests simple.**

### Polling implementation
Use a `useEffect` with `setInterval(500)`. On each tick, if not paused: call `invoke<RustLogEntry[]>("get_log_ring", { sinceSeq: lastSeq.current })`, update `lastSeq` to max `seq` in the response (only if non-empty), and call `setRows(prev => [...prev, ...newRows.map(normalize)])`. Then merge `getRing()` from the frontend logger by detecting newly-arrived entries (track a `lastFrontendTs` ref).

Cap the in-component row state at ~5000 — drop oldest beyond that (rings are bounded too, but the merged history can grow if the user leaves the route mounted).

Clear interval on unmount.

### Auto-scroll detection
Track a ref to the scroll container. Before each row append, compute `wasAtBottom = scrollTop + clientHeight >= scrollHeight - 8`. After append + DOM update, if `wasAtBottom`, scroll to `scrollHeight`. If not, leave alone. (With `@tanstack/react-virtual`, scroll positions are on the wrapper element — read from `parentRef.current`.)

### Routing
Add the route in `src/main.tsx`:
```tsx
<Route path="/logs" element={<LogsRoute />} />
```
Hotkey: add a `useEffect` in `App` or in `main.tsx` that registers a `window` keydown listener for `Ctrl+Shift+L` (or `Meta+Shift+L` on mac if `event.metaKey`) and calls `window.location.hash = "/logs"` — wait, this app uses `BrowserRouter` (not Hash). Use `useNavigate()` from inside an `<AppShell />` component if it exists, or hand-roll via `window.history.pushState` + dispatching `popstate`. Cleanest: wrap routes in a small component that has access to `useNavigate`. Pick whichever requires fewer structural changes; document in notes.

A11y reminder: don't trap the hotkey when focus is inside an input/textarea — bail if `event.target` is an editable element.

### Settings link
Add a small unobtrusive link/button in `src/routes/settings/index.tsx` near the top or in an existing list — text "Logs" or "View logs". Use `<Link to="/logs">Logs</Link>` from `react-router-dom`.

### What NOT to do
- Do not modify any file under `src-tauri/`.
- Do not redesign the logger module from Phase 3.
- Do not add the Tauri fs/dialog plugins — browser download path is fine.
- Do not implement live level changes in the viewer; viewer is read-only per the design.
- Do not bump unrelated deps.

## Files
- F:/projects_new/textblaze/package.json
- F:/projects_new/textblaze/pnpm-lock.yaml
- F:/projects_new/textblaze/src/routes/logs/index.tsx                       (new)
- F:/projects_new/textblaze/src/routes/logs/LogRow.tsx                       (new)
- F:/projects_new/textblaze/src/routes/logs/filter.ts                        (new)
- F:/projects_new/textblaze/src/routes/logs/__tests__/index.test.tsx         (new)
- F:/projects_new/textblaze/src/routes/logs/__tests__/filter.test.ts         (new)
- F:/projects_new/textblaze/src/routes/logs/__tests__/index.a11y.test.tsx    (new)
- F:/projects_new/textblaze/src/main.tsx                                     (route + hotkey)
- F:/projects_new/textblaze/src/routes/settings/index.tsx                    (link to /logs)

## Done When
- `cd F:/projects_new/textblaze && pnpm test` — green; three new test files pass; existing 36 tests still pass.
- `cd F:/projects_new/textblaze && pnpm lint` — clean.
- `cd F:/projects_new/textblaze && pnpm build` — clean.
- `/logs` route renders without runtime errors when navigated via the Settings link or `Ctrl+Shift+L`.
- Save action produces a `.jsonl` file whose lines parse as JSON.
- Fresh output of `pnpm test`, `pnpm lint`, `pnpm build` is captured in your `# EXTERNAL RESPONSE`.

## Rules

Follow the contract in F:/projects_new/textblaze/.agents/shared/worker-contract.md — per-task workflow (test-first → one commit per task `phase-4.task-<M>: …` → append a `## Task <M>` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-04/notes.md → append the `# EXTERNAL RESPONSE` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-04/journal.md) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear → CLARIFICATIONS NEEDED + stop).

Additional phase-specific rules:
- **No backend edits.** `src-tauri/` is sealed.
- **One commit per task**, subject exactly `phase-4.task-<M>: <summary>`. Stage only your own changes.
- **`pnpm lint` after each task** and fix only your own lint errors before committing.

## Response Format

Respond per F:/projects_new/textblaze/.agents/shared/erp.md — return the `# EXTERNAL RESPONSE` block, then the single completion line.

## Same-phase fix

Reuse cached SESSION_ID `ae784fe5-ca22-4245-876e-e8c2cfa8e8ab`. Send `FIX:` + only the delta files / delta context.
