# Phase 5 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Built the snippet command layer around a `SnippetStoreState` root path plus `RwLock` snapshot so the command helpers are testable without constructing a Tauri app.
- Kept the save/reload helpers synchronous for now; Task 3 can add the watcher-backed updater around the same state shape without changing the command tests.

### Spec deviations
- none

### Tradeoffs accepted
- `reload_snippets_inner` currently reloads directly from disk instead of waiting on a watcher event, which keeps the command deterministic in tests while still leaving room for watcher-triggered updates in Task 3.

### Assumptions
- `SaveSnippetDto` includes `original_trigger: Option<String>` because the task text requires replace-vs-append behavior keyed off that field even though the initial payload bullet omitted it.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test commands_roundtrip` failed because `openmacro_lib::commands` and the snippet IPC types/functions did not exist.
- GREEN: `cargo test --test commands_roundtrip` passed after adding the snippet state, DTOs, atomic YAML save, reload, and list/load-error helpers.

## Task 2
### Decisions made (not in spec)
- Implemented prefs around a `PrefsState { path, RwLock<Prefs> }` so command helpers and startup loading can share the same atomic read/write path.
- Set the default `max_expansion_len` to the current orchestrator default (`32_768`) to keep persisted prefs aligned with the runtime cap already in code.

### Spec deviations
- none

### Tradeoffs accepted
- `load_prefs_state` eagerly writes the default file when no prefs file exists, which adds an immediate disk side effect but keeps startup and tests deterministic.

### Assumptions
- The `%APPDATA%/openmacro/prefs.json` requirement maps to `dirs::config_dir()/openmacro/prefs.json` on Windows, which matches the spec’s instruction to use `dirs::config_dir()`.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test prefs_roundtrip` failed because the prefs command module, state, env-path override, and `Prefs` type did not exist.
- GREEN: `cargo test --test prefs_roundtrip --test commands_roundtrip` passed after adding atomic prefs read/write, the `OPENMACRO_PREFS_PATH` override, and `engine::set_paused` wiring.

## Task 3
### Decisions made (not in spec)
- Reused `SnippetStoreState` as the single managed snippet state and added an optional embedded watcher handle so the same state can serve command reads, explicit reloads, and background watcher updates.
- Registered all six actual commands in `generate_handler!` even though the phase text said “five commands”; the listed API surface is `list_snippets`, `save_snippet`, `reload_snippets`, `list_load_errors`, `get_prefs`, and `set_prefs`.

### Spec deviations
- none

### Tradeoffs accepted
- `reload_snippets_inner` still performs a direct reload after nudging the watcher, which means the shared state is refreshed immediately and then may be refreshed again by the background watcher task. That duplicate work is cheap and keeps command results deterministic.

### Assumptions
- `OPENMACRO_SNIPPETS_ROOT` should affect both startup loading and the tray’s “Open snippets folder” action, since both are logically addressing the same snippets root.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --lib snippets_root_prefers_env_override` failed because startup had no `snippets_root()` helper and therefore no env override path.
- GREEN: `cargo test --lib snippets_root_prefers_env_override && cargo test --test commands_roundtrip --test prefs_roundtrip` passed after adding `snippets_root()`, `generate_handler!` registration, managed snippet/prefs state, and the watcher-driven snapshot updater.

## Task 4
### Decisions made (not in spec)
- Moved snippet-root resolution into `commands::snippets::snippets_root()` so the integration test and startup path use the exact same env-override logic.
- Kept the integration tests at the helper/function layer instead of booting a full Tauri app; this still exercises the YAML/prefs persistence contracts while avoiding slow UI/runtime setup in Rust tests.

### Spec deviations
- none

### Tradeoffs accepted
- The final fresh `cargo test --all-features` required a `cargo clean` after the workspace ran out of disk space in `target/`; that made the final verification slower but restored a clean pass instead of leaving stale evidence.

### Assumptions
- The final task’s required evidence is satisfied by the integration tests already added in earlier tasks, with Task 4 tightening them to match the startup env-override requirement exactly.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `cargo test --test commands_roundtrip` failed after switching the test to `load_snippet_store_state()` because the snippet command module did not yet expose the startup-style env-root helper.
- GREEN: `cargo test --all-features` passed with `45 passed, 1 ignored (10 suites, 4.14s)` after adding the shared snippet-root helper and rerunning from a cleaned target directory.
- GREEN: `cargo clippy --all-targets --all-features -- -D warnings` passed clean after removing the dead startup-path helper and simplifying the duplicated append branch in `save_snippet_inner`.

## Task 5
### Decisions made (not in spec)
- Configured ESLint with typescript and react/react-hooks plugins matching installed devDependencies.

### Spec deviations
- none

### Tradeoffs accepted
- none

### Assumptions
- Used pnpm as the package manager because `pnpm-lock.yaml` is present in the openmacro folder.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- none (Tooling setup task)

## Task 6
### Decisions made (not in spec)
- Implemented a `safeInvoke` utility that checks for `window.__OPENMACRO_MOCK_INVOKE` to intercept command invocations, allowing testing without Tauri runtime environment.

### Spec deviations
- none

### Tradeoffs accepted
- none

### Assumptions
- The `SaveSnippetDto` and `Prefs` schemas map exactly to the corresponding Rust structures in Phase 5a.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- none (TypeScript compiler type check passed clean using `tsc --noEmit`)

## Task 7
### Decisions made (not in spec)
- Created a sidebar navigation layout in `Settings` route containing "Snippets", "Preferences", and "Sync & Diagnostics" tabs.
- Added active error handling and status monitoring on preferences updates and manual reloads.

### Spec deviations
- none

### Tradeoffs accepted
- none

### Assumptions
- none

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- none (UI components assembly phase; verified by tests in Task 8)

## Task 8
### Decisions made (not in spec)
- Modified `safeInvoke` wrapper inside `src/lib/snippets.ts` to only pass arguments to `__OPENMACRO_MOCK_INVOKE` if defined, resolving vitest's strict argument-matching expectation failures.
- Fixed React-unused ESLint errors and unhandled any warnings across all settings component files.

### Spec deviations
- none

### Tradeoffs accepted
- none

### Assumptions
- none

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `pnpm test` failed 6/7 tests because `SnippetEditor` lacked inputs, trigger constraints, and variable mutation rows.
- GREEN: `pnpm test` passed 7/7 tests (including accessibility checks). `pnpm lint` and `pnpm build` are fully clean.
