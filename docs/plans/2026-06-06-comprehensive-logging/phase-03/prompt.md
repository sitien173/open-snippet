## Original User Request
Add comprehensive developer-debugging logging to the openmacro Tauri app. Phases 1–2 built the Rust subsystem and the two IPC commands the frontend will consume. This phase introduces the frontend logger.

## Phase
Phase 3 — Frontend logger module + `console.*` migration. Add a small `loglevel`-backed logger with redaction + an in-memory ring buffer, wire `await initFromConfig()` into `src/main.tsx`, migrate the four files using `console.*` to typed module loggers.

## Tasks
- task-1: Add deps `loglevel` and `loglevel-plugin-prefix` via `pnpm add`. Create `src/lib/logger.ts` per the design (see Context for exact contract and known-deltas from spec). Export `getLogger(module)`, `getRing()`, `initFromConfig()`, the `LogEntry` type, and a `redact(fields)` helper (or keep it module-private — your call).
- task-2: Write `src/lib/__tests__/logger.test.ts` (Vitest + jsdom) covering: `getLogger("x").debug("msg", fields)` pushes into the ring; ring is capped at 1000 and FIFO; redaction by key name (`body`, `content`, `clipboard`, `value`, `token`, `password`, `secret` — case-insensitive, full-string match — see Context for the boundary rule); `<redacted len=N>` uses code-point count for unicode (assert with a multi-byte string).
- task-3: Write `src/lib/__tests__/logger-init.test.ts` covering: `initFromConfig()` calls `invoke("get_logging_frontend_cfg")` and applies the returned `level` + per-module levels; `VITE_LOG_LEVEL` env wins; `VITE_LOG_VERBOSE === "1"` enables verbose (disables redaction); `invoke` rejection leaves logger working at default `info` (no app-boot crash).
- task-4: Wire `await initFromConfig()` into `src/main.tsx` before `<App />` mounts. Migrate all `console.{log,warn,error}` in `src/routes/form/index.tsx`, `src/routes/settings/index.tsx`, `src/routes/settings/PrefsPanel.tsx`, `src/routes/settings/SyncPanel.tsx` to `getLogger("<module>").{debug,info,warn,error}` calls. Use module names: `form`, `settings`, `settings.prefs`, `settings.sync`.

## Context

### Design references (read fully)
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging-design.md — section 3 (Frontend logger) is the authoritative source for the API shape.
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/PLAN.md — Phase 3 row (file list, acceptance, reviewer checklist).
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-02/journal.md — Phase 2 outcomes and the IPC shape you must consume.

### IPC shape (delta from design doc)
The design doc §3 sketch shows the frontend expecting `invoke<FrontendLogCfg>("get_logging_frontend_cfg")` returning `{ level, modules, verbose_content }`. The actual Rust command (Phase 2 result) returns only `{ level: string, modules: Record<string, string> }` — `verbose_content` lives on the parent `LoggingConfig` and is **not** in the frontend block.

Resolution for this phase:
- Type `FrontendLogCfg = { level: string; modules: Record<string, string> }`.
- `verbose_content` is sourced **only** from `import.meta.env.VITE_LOG_VERBOSE === "1"` on the frontend side (no IPC field). This is acceptable under the "opt-in verbose, dev only" decision recorded in the design doc.
- Document this decision in your task-1 notes block.

If a future phase wants to add `verbose_content` to the IPC payload, that's a Rust-side change; out of scope here.

### `LogEntry` type to declare in TS
```ts
export type LogEntry = {
  ts: number;                                // Date.now()
  level: "trace" | "debug" | "info" | "warn" | "error";
  module: string;
  msg: string;
  fields?: Record<string, unknown>;
};
```
This is the *frontend* `LogEntry` (separate from the Rust one). Phase 4's viewer will merge it with the Rust ring entries.

### Redaction regex — boundary rule
Use `^(body|content|clipboard|value|token|password|secret)$` with the `i` flag. The regex MUST be anchored so `bodyguard` does not match. Inline the regex in the module; no separate config.

For string values, redact as `<redacted len=${[...v].length}>` (code-point count). For non-string sensitive values, redact as `"<redacted>"`. The function shape from the design doc:

```ts
function redact(fields: Record<string, unknown>): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(fields)) {
    if (SENSITIVE_KEYS.test(k) && typeof v === "string") {
      out[k] = `<redacted len=${[...v].length}>`;
    } else if (SENSITIVE_KEYS.test(k)) {
      out[k] = "<redacted>";
    } else {
      out[k] = v;
    }
  }
  return out;
}
```

### `loglevel` method-factory wrap
Wrap `log.methodFactory` so every call (after the wrapper) pushes a `LogEntry` into the ring AND calls through to the original method (which handles level-based filtering for devtools). Wrap *before* calling `prefix.apply(log, …)`. Remember to call `log.setLevel(log.getLevel())` after the wrap to apply it to existing loggers.

### `initFromConfig` shape
```ts
export async function initFromConfig() {
  let cfg: FrontendLogCfg = { level: "info", modules: {} };
  try {
    cfg = await invoke<FrontendLogCfg>("get_logging_frontend_cfg");
  } catch {
    // Tauri not available (e.g. pure web dev) or command failed — keep defaults.
  }
  verboseContent = import.meta.env.VITE_LOG_VERBOSE === "1";
  const envLevel = import.meta.env.VITE_LOG_LEVEL as log.LogLevelDesc | undefined;
  log.setLevel(envLevel ?? cfg.level as log.LogLevelDesc);
  for (const [mod, lvl] of Object.entries(cfg.modules)) {
    log.getLogger(mod).setLevel(lvl as log.LogLevelDesc);
  }
}
```

### Console.* migration
Locate every `console.{log,warn,error,info,debug}` in the four files (`grep -n` if needed; do not migrate `console.assert`). Convert to:
```ts
import { getLogger } from "../../lib/logger";   // adjust path
const log = getLogger("form");                    // or "settings", "settings.prefs", "settings.sync"
log.debug("saved", { keys });
```
Choose the log method by the original `console` call:
- `console.log` / `console.debug` → `log.debug`
- `console.info` → `log.info`
- `console.warn` → `log.warn`
- `console.error` → `log.error`

Preserve the message text. For object arguments, pass them as the `fields` second argument; if the original call passed a string plus multiple objects, merge them or pick the most useful one — judgment call, document in notes.

### Frontend file path conventions
- Tests live next to the source under `src/lib/__tests__/`.
- Frontend test setup is `src/test-setup.ts` (Vitest + jsdom). It is auto-loaded by vitest config — do not re-import it manually.
- TypeScript path alias: imports from `src/lib/...` use relative paths like `../../lib/logger` (the project does not appear to use a `@` alias — verify by reading the existing route files before adding imports).

### main.tsx wiring
```tsx
import { initFromConfig } from "./lib/logger";

async function main() {
  await initFromConfig();
  ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
}
main();
```
The current `main.tsx` may already use a top-level `createRoot(...)` call; wrap it in an async `main()` and call it. Do not block on init for more than ~1s; the try/catch in `initFromConfig` already prevents a hang.

### What NOT to do
- Do not modify any file under `src-tauri/`. Backend is done.
- Do not add an in-app log viewer route — that's Phase 4.
- Do not change `import.meta.env` types globally; if TS complains, add narrow types inline.
- Do not add Tauri command bindings beyond `get_logging_frontend_cfg`. (`get_log_ring` is Phase 4.)

## Files
- F:/projects_new/textblaze/package.json
- F:/projects_new/textblaze/pnpm-lock.yaml
- F:/projects_new/textblaze/src/lib/logger.ts                          (new)
- F:/projects_new/textblaze/src/lib/__tests__/logger.test.ts           (new)
- F:/projects_new/textblaze/src/lib/__tests__/logger-init.test.ts      (new)
- F:/projects_new/textblaze/src/main.tsx
- F:/projects_new/textblaze/src/routes/form/index.tsx
- F:/projects_new/textblaze/src/routes/settings/index.tsx
- F:/projects_new/textblaze/src/routes/settings/PrefsPanel.tsx
- F:/projects_new/textblaze/src/routes/settings/SyncPanel.tsx

## Done When
- `cd F:/projects_new/textblaze && pnpm test` — green; new `logger.test.ts` and `logger-init.test.ts` both pass; existing tests still pass.
- `cd F:/projects_new/textblaze && pnpm lint` — clean.
- `cd F:/projects_new/textblaze && pnpm build` — clean (`tsc` passes, vite build succeeds).
- `grep -rn "console\\.\\(log\\|info\\|warn\\|error\\|debug\\)" src/routes/form/index.tsx src/routes/settings/index.tsx src/routes/settings/PrefsPanel.tsx src/routes/settings/SyncPanel.tsx` returns no matches.
- Fresh output of all three commands is captured in your `# EXTERNAL RESPONSE`.

## Rules

Follow the contract in F:/projects_new/textblaze/.agents/shared/worker-contract.md — per-task workflow (test-first → one commit per task `phase-3.task-<M>: …` → append a `## Task <M>` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-03/notes.md → append the `# EXTERNAL RESPONSE` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-03/journal.md) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear → CLARIFICATIONS NEEDED + stop).

Additional phase-specific rules:
- **No backend edits.** `src-tauri/` is sealed for this phase.
- **No new routes.** `/logs` route is Phase 4.
- **One commit per task**, subject exactly `phase-3.task-<M>: <summary>`. Stage only your own changes; the tree was clean at phase start.
- **Run `pnpm lint` after each task** and fix lint errors in your own changes before committing (don't fix pre-existing lint issues in untouched files).

## Response Format

Respond per F:/projects_new/textblaze/.agents/shared/erp.md — return the `# EXTERNAL RESPONSE` block, then the single completion line.
