## Original User Request
Add comprehensive developer-debugging logging to the openmacro Tauri app. This phase stands up the Rust logging core: config schema, three-layer `tracing` subscriber (stdout + rotating JSON file + ring buffer), redaction utility, rotation pruning, and tests. No call-site instrumentation in this phase.

## Phase
Phase 1 — Rust logging core. Bring up `log_init` as a working subsystem wired into `main.rs`, with all four integration tests passing. Do **not** convert existing modules to `tracing` yet; that's Phase 2.

## Tasks
- task-1: Add deps to `src-tauri/Cargo.toml` (`tracing`, `tracing-subscriber` with `env-filter`+`json`+`fmt`, `tracing-appender`; dev-deps `tracing-test`, `tempfile`) and define `LoggingConfig` in the existing store with serde defaults matching the design.
- task-2: Implement `log_init::ring` (`RingBuffer`, `RingLayer`, `LogEntry`, monotonic `seq`, `slice_since(seq)`, capacity 2000 FIFO) with `log_init_ring.rs` integration test.
- task-3: Implement `log_init::redact` (`FieldKind`, `redact_str`, `log_body!` macro, `verbose_content()` `OnceLock<AtomicBool>`; `OPENMACRO_LOG_VERBOSE=1` env wins over config) with `log_init_redact.rs` test.
- task-4: Implement `log_init::init(cfg) -> LogHandles` composing `EnvFilter` (env > config > defaults), pretty stdout layer, JSON daily-rotating file layer with `WorkerGuard`, and `RingLayer`; implement `log_init::rotation::prune_old_logs(max_files)`; wire from `main.rs` and keep `LogHandles` in Tauri app state. Add `log_init_filter.rs` + `log_init_rotation.rs` tests.

## Context

### Design references (read fully before editing)
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging-design.md — sections 1, 2, 5 are authoritative for this phase.
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/PLAN.md — Phase 1 row (files, acceptance criteria, reviewer checklist).

### Project conventions (from AGENTS.md)
- pnpm for frontend, cargo for backend.
- Rust uses `rustfmt`, snake_case modules, focused modules grouped by capability.
- Tauri command handlers live in `src-tauri/src/commands/` — but this phase adds **no** new commands (those are Phase 2).
- Integration tests live in `src-tauri/tests/`, named by behavior (`matcher_basic.rs`, `sync_roundtrip.rs`, …).

### Key design decisions (do not redesign — reflect these literally)
- Config schema (YAML, in the existing store):
  ```yaml
  logging:
    level: info
    modules:                       # target -> level
      openmacro::matcher: debug
    file:
      enabled: true
      max_files: 7
    verbose_content: false
    frontend:
      level: info
      modules: {}
  ```
- Defaults if `logging` block absent: `level=info`, `file.enabled=true`, `max_files=7`, `verbose_content=false`, empty `modules`, frontend block defaults.
- `EnvFilter` resolution: if `RUST_LOG` env is set, it wins entirely. Otherwise build directives from `logging.level` plus per-module overrides.
- `verbose_content()` reads a `OnceLock<AtomicBool>` set once at init. `OPENMACRO_LOG_VERBOSE=1` env beats the YAML flag.
- Ring layer: capacity 2000, `Mutex<VecDeque<LogEntry>>`, monotonic `AtomicU64` seq, `slice_since(since_seq) -> Vec<LogEntry>` returns entries with `seq > since_seq` in insertion order.
- `LogEntry`: `{ seq: u64, ts_unix_ms: i64, level: Level, target: String, message: String, fields: serde_json::Value, span_path: Vec<String> }` — `serde::Serialize` for the future Tauri command in Phase 2.
- File appender: `tracing_appender::rolling::daily(log_dir(), "openmacro.log")`, JSON layer with current span, non-blocking with `WorkerGuard` kept alive via `LogHandles` in Tauri state.
- `prune_old_logs(max_files)` deletes only files matching `openmacro.log.*` beyond the newest `max_files`; never touches other files in the log dir. Runs once at startup.
- `LogHandles { _file_guard: WorkerGuard, ring: Arc<RingBuffer> }`.

### Open question to resolve in this phase
The design carries this forward: confirm the exact Tauri app dir API used elsewhere — `app_data_dir` vs `app_log_dir`. Read `src-tauri/src/store/` first to see how the store resolves its directory and reuse the same helper. Document the choice in your `## Task <M>` notes block.

### Sensitive `FieldKind` variants (from design §5)
- `SnippetBody`, `ClipboardText`, `FormValue` → `<redacted len=N>` (use `s.chars().count()` for unicode-safety).
- `Credential`, `Token` → `<redacted>`.
- `TriggerName`, `SnippetId`, `Path` → unchanged.
- Verbose mode returns the raw string for the redacted variants.

### Test guidance
- `log_init_filter.rs`: table-driven. Cases include: env unset + default config → `info`; env unset + per-module override → directive string contains override; `RUST_LOG=…` set → env wins (asserted by reading back the directive used). Use `serial_test` only if env-var mutation collides; otherwise spawn each case as its own `#[test]` with environment cleared via a small `with_env` helper.
- `log_init_redact.rs`: cover every `FieldKind`; verify `OPENMACRO_LOG_VERBOSE=1` toggles to plain; verify char-count uses code points not bytes (use a multi-byte UTF-8 string).
- `log_init_ring.rs`: push 2100 events, assert length 2000, newest preserved, oldest dropped; assert `seq` strictly increases; `slice_since(seq_at_index_500)` returns exactly the entries after that point.
- `log_init_rotation.rs`: in a `tempfile::TempDir`, create 7 fake `openmacro.log.YYYY-MM-DD` files plus an unrelated `random.txt`; call `prune_old_logs(3)`; assert exactly 3 newest log files remain *and* `random.txt` is untouched.

### main.rs wiring sketch
```rust
fn main() {
    let store = load_store().expect("load store");
    let cfg = store.logging.clone();
    let handles = openmacro::log_init::init(&cfg);
    tauri::Builder::default()
        .manage(handles)            // keep _file_guard alive
        .manage(store)
        // ... existing setup
        .run(...);
}
```
Exact existing names will differ — match what's already in `main.rs`. Do **not** rename existing items.

## Files
- F:/projects_new/textblaze/src-tauri/Cargo.toml
- F:/projects_new/textblaze/src-tauri/src/main.rs
- F:/projects_new/textblaze/src-tauri/src/lib.rs   (or whatever the existing module root is — declare `pub mod log_init;`)
- F:/projects_new/textblaze/src-tauri/src/store/   (add `LoggingConfig` + (de)serialization)
- F:/projects_new/textblaze/src-tauri/src/log_init/mod.rs   (new)
- F:/projects_new/textblaze/src-tauri/src/log_init/ring.rs   (new)
- F:/projects_new/textblaze/src-tauri/src/log_init/redact.rs   (new)
- F:/projects_new/textblaze/src-tauri/src/log_init/rotation.rs   (new)
- F:/projects_new/textblaze/src-tauri/tests/log_init_filter.rs   (new)
- F:/projects_new/textblaze/src-tauri/tests/log_init_redact.rs   (new)
- F:/projects_new/textblaze/src-tauri/tests/log_init_ring.rs   (new)
- F:/projects_new/textblaze/src-tauri/tests/log_init_rotation.rs   (new)

## Done When
- `cd F:/projects_new/textblaze/src-tauri && cargo test` — green; the four new test files all pass and no existing test regresses.
- `cd F:/projects_new/textblaze/src-tauri && cargo fmt --check` — clean.
- `cd F:/projects_new/textblaze/src-tauri && cargo clippy -- -D warnings` — clean.
- A run of the binary (cargo test exercising init is sufficient evidence; full `pnpm tauri dev` smoke is the coordinator's check) demonstrates: init does not panic with default config; with `OPENMACRO_LOG_VERBOSE=1` `verbose_content()` returns `true`.
- The four `Done When` command outputs are captured fresh in your `# EXTERNAL RESPONSE`.

## Rules

Follow the contract in F:/projects_new/textblaze/.agents/shared/worker-contract.md — per-task workflow (test-first → one commit per task `phase-1.task-<M>: …` → append a `## Task <M>` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-01/notes.md → append the `# EXTERNAL RESPONSE` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-01/journal.md) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear → CLARIFICATIONS NEEDED + stop).

Additional phase-specific rules:
- **No call-site changes.** Do not insert `tracing::*` calls into matcher/expand/hook/inject/sync/form/commands modules. That's Phase 2. The only existing modules you may edit are `Cargo.toml`, the store (to add `LoggingConfig`), `lib.rs`/module root (to declare `pub mod log_init;`), and `main.rs` (to call `log_init::init` and `manage(handles)`).
- **No new Tauri commands.** Those are Phase 2.
- **Reuse the store's existing app-dir helper** rather than picking a Tauri API freshly — record your choice in the task-4 notes block.
- **One commit per task**, subject exactly `phase-1.task-<M>: <summary>`. Stage only your own changes — `git status` must show no unrelated dirty files between commits (the tree was clean when this phase started).
- **`cargo fmt` after each task** before committing.

## Response Format

Respond per F:/projects_new/textblaze/.agents/shared/erp.md — return the `# EXTERNAL RESPONSE` block, then the single completion line.
