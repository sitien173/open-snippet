## Original User Request
openmacro Phase 5a — Rust IPC command surface for the Settings UI. Adds `src-tauri/src/commands/` and registers Tauri commands consumed by the front-end. UI work is split off into Phase 5b (gemini).

## Phase
Create the back-side IPC surface that the Settings UI will call: list snippets, save a snippet (round-trips YAML to its source file), reload the snippet store, read/write user prefs. Wire commands into the Tauri builder. Add unit/integration tests for the YAML round-trip + prefs persistence. No UI work in this phase.

## Tasks
- task-1: **`commands::snippets`** — Create `F:/projects_new/textblaze/openmacro/src-tauri/src/commands/{mod.rs,snippets.rs,prefs.rs}`. In `snippets.rs`, expose Tauri commands:
  - `list_snippets() -> Vec<SnippetDto>` — DTO with `{id, trigger, replace, vars, source_file, file_relative}`. Pulls from a `tauri::State` holding the loaded store + load errors.
  - `save_snippet(payload: SaveSnippetDto) -> Result<(), String>` — payload `{source_file: PathBuf, trigger, replace, vars}`. Reads the YAML file, replaces the snippet matching original `trigger` (or appends if `original_trigger` is None), serializes back via `serde_yaml`, writes atomically (`tempfile::NamedTempFile::persist`). Trigger collision within same file → return `Err("trigger collision: <t>")`.
  - `reload_snippets() -> Result<ReloadResult, String>` — forces a watcher reload and returns `{loaded: usize, errors: Vec<LoadErrorDto>}`.
  - `list_load_errors() -> Vec<LoadErrorDto>` — exposes broken-file errors for the red badges. `LoadErrorDto = {path: String, message: String}`.
  Use `serde::Serialize`/`Deserialize` on all DTOs. Add a `Store` `tauri::State` (RwLock around `Vec<Snippet>` + `Vec<LoadError>`) populated at startup from `store::loader::load_from_root`.
- task-2: **`commands::prefs`** — `Prefs` struct `{paused: bool, autostart: bool, max_expansion_len: usize, shell_consent: bool}` (serde, default impl). Persist to `%APPDATA%/openmacro/prefs.json` using `dirs::config_dir()` (add `dirs = "5"` if absent). Commands:
  - `get_prefs() -> Prefs`
  - `set_prefs(prefs: Prefs) -> Result<(), String>` — writes atomically; flips `engine::orchestrator::set_paused(prefs.paused)`.
  Use `tauri::State<RwLock<Prefs>>`. Skip the autostart plugin call here (Phase 1 already wired the plugin — just persist the bool; Phase 5b will call the plugin on change).
- task-3: **Wire into builder** — In `src-tauri/src/lib.rs`: `manage` both states, register commands in `.invoke_handler(tauri::generate_handler![...])`. Load snippets from `snippets_root()` (`%APPDATA%/openmacro/snippets`) on startup; create dir if missing. On startup, read `prefs.json` if it exists, else write defaults. Spawn a tokio task that drives `store::watcher::watch_root` and updates the shared store state on every reload event (debounced 200 ms per Phase 2).
- task-4: **Tests** — `src-tauri/tests/commands_roundtrip.rs`: end-to-end save → reload → list returns the new trigger. Use `tempfile::tempdir()` as the snippets root, override via a `OPENMACRO_SNIPPETS_ROOT` env var the production code reads at startup (gate behind `cfg(test)` or always-read with fallback). `src-tauri/tests/prefs_roundtrip.rs`: write prefs → read prefs returns same struct. Use a temp `OPENMACRO_PREFS_PATH` env override identically. **TDD-first**: write the failing test, watch it fail, then implement; record RED→GREEN in notes.

## Context
- Phase 4 left `engine::orchestrator::set_paused` in place — wire `set_prefs` to it.
- Snippet ID format is `<rel-path>::<trigger>` (Phase 2 decision); preserve when generating DTOs.
- Atomic file write: prefer `tempfile::NamedTempFile::new_in(parent).persist(target)` to avoid partial writes.
- Add deps in `Cargo.toml` as needed: `tempfile = "3"` (likely dev-dep already; promote if needed), `dirs = "5"`. `tokio` already present.
- Clippy: still `-D warnings`.

## Files
- `F:/projects_new/textblaze/openmacro/src-tauri/src/commands/{mod.rs,snippets.rs,prefs.rs}` (create)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/lib.rs` (modify — register commands + state + watcher task)
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml` (modify if deps needed)
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/commands_roundtrip.rs` (create)
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/prefs_roundtrip.rs` (create)

## Done When
- `cargo test --all-features` green
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- Save → reload → list returns the new trigger from a temp snippets dir (integration test)
- Prefs round-trip persists across reads (integration test)
- `tauri::generate_handler!` registers all five commands; `tauri::State` for store + prefs is `manage`d

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-05/`. TDD per `test-driven-development` — failing test first for every task that adds behavior.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
