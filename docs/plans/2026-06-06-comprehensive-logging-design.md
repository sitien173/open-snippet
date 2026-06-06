# Comprehensive Logging — Design

- Date: 2026-06-06
- Status: Confirmed, ready for implementation planning
- Scope: openmacro (Tauri desktop app — Rust backend + React/Vite frontend)

## Goals & Non-Goals

**Primary purpose:** developer debugging — verbose tracing, module-level
filtering, dev-friendly output.

**In scope:**
- Rust backend logging via `tracing` with stdout + rotating JSON file +
  in-memory ring sink.
- Frontend logging via `loglevel` with devtools console + in-memory ring.
- Filtering controlled by env var (`RUST_LOG`, `VITE_LOG_LEVEL`) with
  config-file fallback in the existing YAML store.
- In-app log viewer route showing both rings side by side.
- Opt-in verbose content logging; redacted by default.

**Out of scope:**
- User-facing troubleshooting UI beyond the log viewer.
- Audit/security trail semantics (tamper-evidence, separate file).
- Unified pipeline — frontend logs are NOT forwarded over IPC into the
  Rust file. Stacks stay separate.
- Runtime UI control of log levels — config changes require restart.

## Requirements Summary

| Concern        | Decision                                                       |
|----------------|----------------------------------------------------------------|
| Purpose        | Developer debugging                                            |
| Layers         | Rust + frontend, separate stacks                               |
| Sinks          | stdout/stderr, rotating file, devtools console, in-app viewer  |
| Filter control | Env var + config file (no runtime UI)                          |
| Redaction      | Opt-in verbose (redact by default, `OPENMACRO_LOG_VERBOSE=1`)  |

## 1. Config Schema & Filtering

Add a `logging` block to the YAML store handled by `src-tauri/src/store/`:

```yaml
logging:
  level: info                    # global default: trace|debug|info|warn|error
  modules:                       # per-module overrides (Rust target names)
    openmacro::matcher: debug
    openmacro::sync: trace
  file:
    enabled: true
    max_files: 7                 # daily rotation, keep N days
  verbose_content: false         # if true, disables redaction
  frontend:
    level: info
    modules:                     # route/component tags
      settings: debug
```

**Rust resolution order:**
1. `RUST_LOG` env var if set → wins entirely (standard `EnvFilter`).
2. Else build `EnvFilter` directives from `logging.level` + `logging.modules`.
3. `OPENMACRO_LOG_VERBOSE=1` forces `verbose_content=true` regardless of config.

**Frontend resolution:**
1. `VITE_LOG_LEVEL` build-time env if set.
2. Else fetch `logging.frontend` via Tauri IPC on app start; apply to root
   `loglevel` logger + per-module children.
3. Live edits require restart.

**Loading:** new `src-tauri/src/log_init.rs` exposes
`init(cfg: &LoggingConfig) -> LogHandles`, called from `main.rs` before
any other subsystem. Returns handles kept in Tauri app state so the file
appender's worker guard stays alive for the process lifetime.

**Defaults** when no `logging` block exists: `level=info`, `file.enabled=true`,
`max_files=7`, `verbose_content=false`.

## 2. Rust Logging Layers

Dependencies in `src-tauri/Cargo.toml`:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
tracing-appender = "0.2"
```

Three layers behind one `EnvFilter`:

```rust
pub struct LogHandles {
    pub _file_guard: tracing_appender::non_blocking::WorkerGuard,
    pub ring: Arc<RingBuffer>,
}

pub fn init(cfg: &LoggingConfig) -> LogHandles {
    let filter = build_env_filter(cfg);

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_ansi(true)
        .compact();

    let file_appender = tracing_appender::rolling::daily(log_dir(), "openmacro.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_current_span(true);

    let ring = Arc::new(RingBuffer::new(2000));
    let ring_layer = RingLayer::new(ring.clone());

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer.with_filter(file_filter(cfg)))
        .with(ring_layer)
        .init();

    LogHandles { _file_guard: guard, ring }
}
```

**`RingLayer`** implements `tracing_subscriber::Layer`. Each event serializes
`{seq, ts, level, target, fields, span_path}` into a `LogEntry` pushed into
`Mutex<VecDeque<LogEntry>>` capped at 2000 (drops oldest). `seq` is a monotonic
`AtomicU64` so the frontend viewer can poll deltas.

**Rotation cleanup:** `tracing-appender::rolling::daily` does not prune.
Add `prune_old_logs(max_files)` invoked once at startup to delete files
beyond the configured retention.

**Call sites:** existing modules adopt `tracing::{info, debug, warn, error,
instrument}`. No `log` crate dep needed — `tracing` macros are compatible.
Wrap entry points (Tauri commands, matcher passes, sync ops) in
`#[instrument(skip(large_args))]` so nested logs inherit context like
`command=expand_snippet trigger="…"`.

## 3. Frontend Logger

New module `src/lib/logger.ts`. Deps: `loglevel` (~1KB) +
`loglevel-plugin-prefix`.

```ts
import log from "loglevel";
import prefix from "loglevel-plugin-prefix";
import { invoke } from "@tauri-apps/api/core";

export type LogEntry = {
  ts: number;
  level: "trace" | "debug" | "info" | "warn" | "error";
  module: string;
  msg: string;
  fields?: Record<string, unknown>;
};

const RING_CAP = 1000;
const ring: LogEntry[] = [];
let verboseContent = false;

prefix.reg(log);
prefix.apply(log, {
  format: (level, name, ts) =>
    `[${ts.toISOString()}] ${level.toUpperCase()} ${name ?? "app"}:`,
});

const origFactory = log.methodFactory;
log.methodFactory = (methodName, logLevel, loggerName) => {
  const raw = origFactory(methodName, logLevel, loggerName);
  return (msg: string, fields?: Record<string, unknown>) => {
    ring.push({
      ts: Date.now(),
      level: methodName as LogEntry["level"],
      module: String(loggerName ?? "app"),
      msg,
      fields: fields && (verboseContent ? fields : redact(fields)),
    });
    if (ring.length > RING_CAP) ring.shift();
    raw(msg, fields ?? "");
  };
};

export function getLogger(module: string) { return log.getLogger(module); }
export function getRing(): LogEntry[] { return ring.slice(); }

export async function initFromConfig() {
  const cfg = await invoke<FrontendLogCfg>("get_logging_frontend_cfg");
  verboseContent = cfg.verbose_content || import.meta.env.VITE_LOG_VERBOSE === "1";
  const envLevel = import.meta.env.VITE_LOG_LEVEL as log.LogLevelDesc | undefined;
  log.setLevel(envLevel ?? cfg.level);
  for (const [mod, lvl] of Object.entries(cfg.modules ?? {})) {
    log.getLogger(mod).setLevel(lvl as log.LogLevelDesc);
  }
}
```

`src/main.tsx` awaits `initFromConfig()` before mounting `<App />`. Existing
`console.*` call sites in `src/routes/form/index.tsx`,
`src/routes/settings/index.tsx`, `src/routes/settings/PrefsPanel.tsx`,
`src/routes/settings/SyncPanel.tsx` migrate to
`const log = getLogger("settings.prefs"); log.debug("saved", { keys })`.

New Tauri command `get_logging_frontend_cfg` returns
`{ level, modules, verbose_content }` from the store.

## 4. In-App Log Viewer

New route `src/routes/logs/index.tsx`. Reachable from Settings or via
`Ctrl+Shift+L` hotkey.

```
┌─────────────────────────────────────────────────────┐
│ [Source: Rust ▾] [Level: ≥debug ▾] [Module: …]      │
│ [Search: ___________]  [Pause] [Clear] [Copy] [Save]│
├─────────────────────────────────────────────────────┤
│  12:03:41.220 DEBUG matcher    trigger matched ";h" │
│  12:03:41.221 INFO  expand     expanded 14 chars    │
└─────────────────────────────────────────────────────┘
```

**Data sources:**
- Rust: new Tauri command `get_log_ring(since_seq: u64) -> Vec<LogEntry>`,
  polled at 500ms (no event bridge — matches the "no runtime UI" stance).
- Frontend: `getRing()` from `src/lib/logger.ts`, read on the same tick.

```tsx
function LogsRoute() {
  const [entries, setEntries] = useState<UiEntry[]>([]);
  const [filter, setFilter] = useState<Filter>(defaults);
  const lastSeq = useRef(0);
  useInterval(async () => {
    const rust = await invoke<LogEntry[]>("get_log_ring",
                   { sinceSeq: lastSeq.current });
    if (rust.length) lastSeq.current = rust.at(-1)!.seq;
    setEntries(merge(entries, rust, getRing()));
  }, filter.paused ? null : 500);
  const view = useMemo(() => applyFilter(entries, filter), [entries, filter]);
  return <VirtualList rows={view} renderRow={LogRow} />;
}
```

**Virtualization:** `@tanstack/react-virtual` if not already a dep.
Auto-scroll to bottom unless user scrolls up.

**Save:** Tauri `save` dialog → `writeTextFile` as JSON-lines to match the
on-disk Rust file format.

**A11y:** `role="log"` container; keyboard row nav; sibling
`*.a11y.test.tsx`.

**Read-only:** changing levels requires editing config + restart.

## 5. Redaction

Two parallel utilities sharing field-name conventions.

**Rust** — `src-tauri/src/log_init/redact.rs`:

```rust
pub fn redact_str(s: &str, kind: FieldKind) -> String {
    match kind {
        FieldKind::SnippetBody | FieldKind::ClipboardText | FieldKind::FormValue =>
            format!("<redacted len={}>", s.chars().count()),
        FieldKind::Credential | FieldKind::Token =>
            "<redacted>".into(),
        FieldKind::TriggerName | FieldKind::SnippetId | FieldKind::Path =>
            s.to_string(),
    }
}

#[macro_export]
macro_rules! log_body {
    ($body:expr) => {
        if $crate::log_init::verbose_content() { $body.to_string() }
        else { $crate::log_init::redact::redact_str($body, FieldKind::SnippetBody) }
    };
}
```

Call sites are explicit:
```rust
tracing::debug!(trigger = %t.name, body = %log_body!(&snippet.body), "expanding");
```

`verbose_content()` reads a `OnceLock<AtomicBool>` set at init from
`OPENMACRO_LOG_VERBOSE` (env wins) or `logging.verbose_content`.

**Frontend** — key-name regex matching in `src/lib/logger.ts`:

```ts
const SENSITIVE_KEYS = /^(body|content|clipboard|value|token|password|secret)$/i;
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

Rationale: Rust uses explicit `FieldKind` because typed structs already
carry semantics; TS uses key-name regex (over-redact) because call sites
don't have type-driven kinds.

**Safety boundary on save:** the viewer's "Save to file" action re-applies
redaction to entries even if `verbose_content=true` at runtime, unless
the user explicitly checks "include sensitive content" in the save dialog.
Prevents accidental upload of verbose dev logs to bug trackers.

## 6. Testing

**Rust** (`src-tauri/tests/`):
- `log_init_filter.rs` — table-driven `EnvFilter` directive build; env wins.
- `log_init_redact.rs` — each `FieldKind`; `verbose_content()` toggle; env beats config.
- `log_init_ring.rs` — capacity-2000 FIFO, monotonic `seq`, `get_log_ring(since_seq)` slice correctness.
- `log_init_rotation.rs` — `prune_old_logs(3)` against temp dir with 7 fake files.
- Add `tracing-test` `#[traced_test]` to one matcher/sync/expand test each.

**Frontend** (`src/lib/__tests__/`, Vitest + jsdom):
- `logger.test.ts` — ring writes, cap+FIFO, redaction by key, verbose toggle.
- `logger-init.test.ts` — `invoke` mock, level application, `VITE_LOG_LEVEL` override.
- `routes/logs/index.test.tsx` — viewer merge, filters, pause, JSON-lines save.
- `routes/logs/index.a11y.test.tsx` — axe + `role="log"` + keyboard nav.

**Manual verification:**
1. `cd src-tauri && cargo test` green.
2. `pnpm test` green.
3. `pnpm lint` green.
4. `pnpm tauri dev`: pretty stdout, JSON file at `%APPDATA%/openmacro/logs/`,
   `/logs` route shows both rings, filters work, save produces JSON-lines.
5. `OPENMACRO_LOG_VERBOSE=1 pnpm tauri dev`: bodies unredacted; default run
   shows `<redacted len=N>`.

## 7. Rollout Phases

1. **Phase 1 — Rust core:** deps, `log_init.rs`, config schema in store,
   env filter, stdout + file layers, redaction utility, rotation prune. Tests.
2. **Phase 2 — Rust instrumentation:** `#[instrument]` on Tauri commands,
   `matcher`, `expand`, `hook`, `inject`, `sync`; `log_body!` at sensitive sites.
3. **Phase 3 — Frontend logger:** `loglevel` deps, `src/lib/logger.ts`,
   `get_logging_frontend_cfg` command, `main.tsx` wiring, migrate four `console.*` files.
4. **Phase 4 — In-app viewer:** `/logs` route, polling, virtualization,
   filter/search/save UI, a11y test, hotkey.
5. **Phase 5 — Docs:** `docs/logging.md` covering env vars, config, redaction
   contract, bug-report instructions.

## Open Questions

None at design-confirmation time. Carry forward to the implementation plan:

- Confirm `@tanstack/react-virtual` is already a dependency before assuming it.
- Confirm exact Tauri app data dir API used elsewhere (`app_data_dir` vs
  `app_log_dir`) to keep `log_dir()` consistent with the existing store path.
