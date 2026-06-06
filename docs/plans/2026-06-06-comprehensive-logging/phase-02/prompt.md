## Original User Request
Continue rolling out comprehensive developer-debugging logging across the openmacro Tauri backend. Phase 1 built the subsystem (`log_init`); this phase instruments the actual code and exposes the two Tauri commands the frontend will consume in Phase 3.

## Phase
Phase 2 — Rust instrumentation + IPC commands. Convert backend modules from any ad-hoc `println!`/`eprintln!` to `tracing::{info,debug,warn,error}!` with `#[instrument]` on entry points; apply `log_body!` (and explicit `FieldKind`) at every sensitive site; add `get_logging_frontend_cfg` and `get_log_ring(since_seq)` Tauri commands; smoke-test instrumentation via `tracing-test::traced_test` on representative existing tests. Also fix the carry-over MEDIUM finding from Phase 1 (see Context).

## Tasks
- task-1: Fix Phase 1 MEDIUM finding in `src-tauri/src/log_init/ring.rs` (`JsonFieldVisitor::record_debug` strips outer quotes from `format!("{value:?}")` and can mangle escaped quotes). Replace with a non-stripping path; add a unit test covering a string containing embedded `"` / backslashes.
- task-2: Instrument backend modules with `tracing` + `#[instrument(skip(...))]`. Pass the modules in this order so failures land in narrow diffs: `commands/*` → `matcher/*` → `expand/*` → `hook/*` → `inject/*` → `sync/*` → `form/*`. Replace any `println!`/`eprintln!` in changed files with appropriate `tracing` calls. Apply `log_body!` / explicit `FieldKind` at every sensitive site (snippet bodies, clipboard text, form values, credentials, tokens, secrets) — add a `// SECURITY:` comment where redaction is load-bearing.
- task-3: Add `src-tauri/src/commands/logging.rs` with two Tauri commands: `get_logging_frontend_cfg() -> LoggingFrontendConfig` (read the frontend block from current config) and `get_log_ring(since_seq: u64) -> Vec<LogEntry>` (read from `LogHandles.ring`). Register both in the `invoke_handler!` macro in `lib.rs`. Add `src-tauri/tests/commands_logging.rs` covering: ring delta correctness; default-config frontend cfg shape; seq monotonicity through the command boundary.
- task-4: Add `#[tracing_test::traced_test]` to one representative existing test in each of `src-tauri/tests/matcher_basic.rs` and `src-tauri/tests/sync_roundtrip.rs` (or nearest equivalent if those exact names have drifted). Use `logs_contain(...)` assertions only if a specific instrumentation point is in scope of that test; otherwise the `#[traced_test]` annotation alone is sufficient evidence that spans/events emit without panic. Ensure the full suite still passes.

## Context

### Design references (read fully)
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging-design.md — sections 2 (Rust layers) and 5 (Redaction) are authoritative.
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/PLAN.md — Phase 2 row.
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-01/journal.md — Phase 1 review (the MEDIUM finding you must fix in task-1; the runtime-init debt to consider).
- F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-01/notes.md — task-4 notes capture the `log_dir()` decision and the runtime-init `LoggingConfig::default()` debt.

### Phase 1 outcomes you can rely on
- Crate-level `log_init` module: `init(&LoggingConfig) -> LogHandles { _file_guard, ring: Arc<RingBuffer> }`. Called from `src-tauri/src/lib.rs` at startup.
- `LogHandles` is managed in Tauri app state; retrieve it via the standard `tauri::State<LogHandles>` argument in your new commands.
- `RingBuffer::slice_since(since_seq)` returns entries with `seq > since_seq` in insertion order; `LogEntry` is already `serde::Serialize`.
- `log_init::redact::FieldKind` variants: `SnippetBody`, `ClipboardText`, `FormValue`, `Credential`, `Token`, `TriggerName`, `SnippetId`, `Path`.
- `log_body!(expr)` redacts as `SnippetBody` (uses verbose flag); for non-body sensitive types, call `redact_str(s, FieldKind::X)` explicitly.
- `tracing-test` (v0.2+) and `tempfile` are already dev-deps. No new deps required for this phase except possibly `tracing-test` if not yet present — verify.

### Runtime-init carry-over
Phase 1 wires `init(&LoggingConfig::default())` because no app-wide YAML store object exists at boot. This is fine for `get_log_ring` (it reads the ring, not the config). For `get_logging_frontend_cfg`, you have two acceptable choices — pick one and document in notes:
1. Return the current effective config (whatever was passed to `init`). Add `LoggingConfig` to managed state and have the command read from there. Simplest and consistent.
2. Re-read the persisted YAML on each call. More accurate to disk state but currently no such file exists.

Pick option 1 unless you have a strong reason; document in notes.

### Sensitive sites — non-exhaustive starter list
Inspect modules before instrumenting; this list helps you grep for likely call sites:
- `matcher/*` — trigger names safe (`FieldKind::TriggerName`); candidate snippet bodies must use `log_body!`.
- `expand/*` — substituted body content via `log_body!`; resolver inputs that could contain user secrets via `FieldKind::FormValue`.
- `expand/shell.rs` (if exists) — command stdout/stderr must be `FormValue`-redacted (never log the raw process output unless verbose).
- `hook/*` — keystroke events at debug only; characters via `FieldKind::FormValue` or summarized counts.
- `inject/clipboard.rs` — clipboard payloads via `FieldKind::ClipboardText`.
- `sync/creds.rs`, `sync/git.rs` — credentials/tokens via `FieldKind::Credential`/`Token`; remote URLs are OK to log as `FieldKind::Path`.
- `form/runner.rs`, `form/focus.rs` — form values via `FieldKind::FormValue`.
- `commands/snippets.rs`, `commands/prefs.rs`, `commands/sync.rs` — `#[instrument(skip(state, payload))]` on each `#[tauri::command]`; log the operation name + a non-sensitive identifier.

### Existing module structure check
Before editing each module, run `tgrep -l "println!\\|eprintln!" src-tauri/src` to list current ad-hoc-log sites. Replace them in the same pass.

### What NOT to do
- Do not change the public IPC shape of any existing Tauri command — only annotate them with `#[instrument]` and replace ad-hoc logs.
- Do not refactor unrelated code.
- Do not bump or restructure dependencies beyond optionally adding `tracing-test` if absent.
- Do not modify the frontend (`src/`) in any way — Phase 3.
- Do not touch the 5-snippet vs 6-snippet test assertion fixed in Phase 1.

## Files
(Adjust if actual paths differ; do not invent new modules outside this list.)
- F:/projects_new/textblaze/src-tauri/src/log_init/ring.rs        (fix MEDIUM finding)
- F:/projects_new/textblaze/src-tauri/src/commands/logging.rs     (new)
- F:/projects_new/textblaze/src-tauri/src/commands/mod.rs         (register submodule)
- F:/projects_new/textblaze/src-tauri/src/lib.rs                  (invoke_handler! registration; manage LoggingConfig if you pick option 1)
- F:/projects_new/textblaze/src-tauri/src/commands/*.rs           (instrument each Tauri command)
- F:/projects_new/textblaze/src-tauri/src/matcher/**/*.rs         (instrument)
- F:/projects_new/textblaze/src-tauri/src/expand/**/*.rs          (instrument; log_body!)
- F:/projects_new/textblaze/src-tauri/src/hook/**/*.rs            (instrument)
- F:/projects_new/textblaze/src-tauri/src/inject/**/*.rs          (instrument; redact clipboard)
- F:/projects_new/textblaze/src-tauri/src/sync/**/*.rs            (instrument; redact creds/tokens)
- F:/projects_new/textblaze/src-tauri/src/form/**/*.rs            (instrument; redact form values)
- F:/projects_new/textblaze/src-tauri/Cargo.toml                  (add `tracing-test` dev-dep if missing)
- F:/projects_new/textblaze/src-tauri/tests/commands_logging.rs   (new)
- F:/projects_new/textblaze/src-tauri/tests/matcher_basic.rs      (add #[traced_test] to one test)
- F:/projects_new/textblaze/src-tauri/tests/sync_roundtrip.rs     (add #[traced_test] to one test)

## Done When
- `cd F:/projects_new/textblaze/src-tauri && cargo test` — green; all existing tests still pass and the new `commands_logging.rs` passes.
- `cd F:/projects_new/textblaze/src-tauri && cargo clippy -- -D warnings` — clean.
- New `commands_logging.rs` test demonstrates: (a) `get_log_ring(0)` returns N entries after N pushes; (b) follow-up call with the last `seq` returns only newly-added entries; (c) `get_logging_frontend_cfg` returns the expected shape.
- In a `cargo test` run, the JSON file in the configured log dir contains structured entries with `target`, `level`, `fields.body` redacted as `<redacted len=...>` (or absent) — proven via the new traced tests or by writing a small integration test.
- Phase 1 MEDIUM finding fixed and covered by a new unit test in `ring.rs` (or its sibling test file).

## Rules

Follow the contract in F:/projects_new/textblaze/.agents/shared/worker-contract.md — per-task workflow (test-first → one commit per task `phase-2.task-<M>: …` → append a `## Task <M>` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-02/notes.md → append the `# EXTERNAL RESPONSE` block to F:/projects_new/textblaze/docs/plans/2026-06-06-comprehensive-logging/phase-02/journal.md) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear → CLARIFICATIONS NEEDED + stop).

Additional phase-specific rules:
- **No frontend edits.** `src/` is Phase 3's territory.
- **Do not modify pre-existing fmt drift outside files you are already editing for this phase.** If a file you touch happens to have fmt issues on neighboring lines, leave them alone unless your edit lands on those lines.
- **`cargo fmt`** the files you create/modify before committing each task.
- **Stage only your own changes per commit.** The tree was clean at phase start.
- **One commit per task**, subject exactly `phase-2.task-<M>: <summary>`.

## Response Format

Respond per F:/projects_new/textblaze/.agents/shared/erp.md — return the `# EXTERNAL RESPONSE` block, then the single completion line.

## Same-phase fix

Reuse cached SESSION_ID `019e9c74-7ac4-7d20-a0e7-3ff8bff27bb3`. Send `FIX:` + only the delta files / delta context. The fix still gets its own task commit and (if it changes a decision) an appended `notes.md` block.
