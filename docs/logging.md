# Logging

openmacro emits structured developer-debugging logs from both the Rust
backend (`tracing`) and the React frontend (`loglevel`). This document
covers the env vars, config schema, redaction contract, sinks, and how
to attach logs to a bug report.

## Sinks

| Sink            | Source            | Default                                        |
|-----------------|-------------------|------------------------------------------------|
| Pretty stdout   | Rust              | Visible in the `pnpm tauri dev` terminal       |
| Rotating file   | Rust              | `<log_dir>/openmacro.log.YYYY-MM-DD` (daily)   |
| Ring buffer     | Rust              | 2000 most-recent entries, exposed via IPC      |
| DevTools console| Frontend          | Prefixed entries (`[ISO] LEVEL module:`)       |
| In-memory ring  | Frontend          | 1000 most-recent entries                       |
| `/logs` viewer  | Merged Rust + FE  | Hotkey `Ctrl+Shift+L` (or `⌘+Shift+L` on mac)  |

Default `log_dir` is `${config_dir}/openmacro/logs` (e.g. on Windows:
`%APPDATA%\openmacro\logs`). Override with `OPENMACRO_LOG_DIR`.

The file sink keeps the latest `max_files` rotated files (default `7`)
and prunes the rest on startup.

## Env vars

| Variable                 | Scope    | Effect                                                    |
|--------------------------|----------|-----------------------------------------------------------|
| `RUST_LOG`               | Rust     | `EnvFilter` directives; overrides YAML `logging.level`    |
| `OPENMACRO_LOG_VERBOSE`  | Rust     | `=1` disables body/secret redaction in Rust logs          |
| `OPENMACRO_LOG_DIR`      | Rust     | Overrides the file-sink directory                         |
| `VITE_LOG_LEVEL`         | Frontend | Overrides YAML `logging.frontend.level` at build/dev time |
| `VITE_LOG_VERBOSE`       | Frontend | `=1` disables redaction in frontend logs                  |

Resolution order (Rust): `RUST_LOG` env → YAML `logging` → code default
(`info`). Resolution order (frontend): `VITE_LOG_LEVEL` env → IPC
`get_logging_frontend_cfg` (YAML `logging.frontend`) → code default
(`info`).

## YAML config

Add a `logging` block to the app's YAML config to set persistent
defaults. All fields are optional; defaults are shown below.

```yaml
logging:
  level: info                  # trace | debug | info | warn | error
  modules:                     # per-target overrides for Rust
    matcher: debug
    sync: warn
  file:
    enabled: true
    max_files: 7               # rotated files retained
  verbose_content: false       # Rust: true disables redaction (env wins)
  frontend:
    level: info
    modules:
      form: debug              # per-module overrides for frontend
```

## Redaction

Sensitive fields are redacted to `<redacted len=N>` (N = unicode code
points) by default. Sensitive keys (case-insensitive, whole-string):
`body`, `content`, `clipboard`, `value`, `token`, `password`, `secret`.

- **Rust:** redaction is applied via the `log_body!` macro and explicit
  `FieldKind` at every sensitive call site. Set
  `OPENMACRO_LOG_VERBOSE=1` (or `logging.verbose_content: true`) to
  emit raw content — env wins.
- **Frontend:** the `getLogger(module)` wrapper redacts matching keys
  before they enter the in-memory ring or DevTools console. Set
  `VITE_LOG_VERBOSE=1` to disable.

Frontend entries are redacted **on ingest**, so anything visible in the
`/logs` viewer or saved via the Save button is already redacted (no
"include sensitive content" toggle on save).

## In-app viewer (`/logs`)

Open via the hotkey `Ctrl+Shift+L` (`⌘+Shift+L` on macOS) or the
"Logs" link in Settings. The viewer merges the Rust and frontend rings,
polls every 500ms, and supports:

- Filters: source, minimum level, module substring, free-text search.
- Pause/Resume (stops polling), Clear (local view only — does not touch
  the underlying rings).
- Copy: filtered view → clipboard as JSON-lines.
- Save: filtered view → `openmacro-logs-<ISO>.jsonl` browser download.

The container is `role="log"` with `aria-live="polite"`.

## Attaching logs to a bug report

1. Reproduce the issue with the app running (`pnpm tauri dev` or a
   release build).
2. Open `/logs` (`Ctrl+Shift+L`).
3. Optionally narrow with the module filter (e.g. `matcher`, `sync`).
4. Click **Save** to download the `.jsonl` file. Frontend entries are
   already redacted; Rust entries follow the active redaction mode
   (default redacted, `OPENMACRO_LOG_VERBOSE=1` raw).
5. Attach the `.jsonl` plus, if relevant, the rotated file from
   `<log_dir>/openmacro.log.<date>`.
6. Include: OS + version, app version, repro steps, and whether
   `OPENMACRO_LOG_VERBOSE` / `VITE_LOG_VERBOSE` were set.

Verify the `.jsonl` parses cleanly before sending:

```bash
jq -c . < openmacro-logs-<ISO>.jsonl > /dev/null
```
