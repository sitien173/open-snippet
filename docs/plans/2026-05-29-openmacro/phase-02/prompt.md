## Original User Request
Continue openmacro Phase 2 — YAML snippet store + Aho-Corasick matcher operating on an in-memory rolling buffer. Pure logic, no OS hook, no injection yet. Driven by unit tests.

## Phase
Implement `store/` (YAML loading, file watcher) and `matcher/` (rolling buffer + AC automaton + word-boundary rule) under `openmacro/src-tauri/src/`. Replace the `//! TODO` stubs left by Phase 1.

## Tasks
- task-1: **`store::model` + `store::loader`** — Define `Snippet { id, trigger: String, replace: String, vars: Vec<VarDecl>, source_file: PathBuf }` and `VarDecl { name, kind: VarKind, label, default, required, options, format }`. YAML schema requires `version: 1` at file root, then a list of snippets. Per-file isolation: a malformed YAML file is captured as `LoadError { path, message }` while other files load. Return `LoadResult { snippets, errors }`. Recursive scan of a root dir for `*.yaml`/`*.yml`. Write `tests/store_yaml.rs` first (RED) covering: happy path, missing `version`, unknown `version`, malformed YAML isolation, vars round-trip (all kinds: text, textarea, choice, number, datetime, clipboard, cursor, shell, form).
- task-2: **`store::watcher`** — Wrap `notify::recommended_watcher` with a 200 ms debouncer. Public API: `Store::spawn(root: PathBuf) -> Store` returning a handle that exposes `subscribe() -> tokio::sync::watch::Receiver<Arc<SnapshotInner>>` and `reload_now()`. Each watch event triggers a full rescan; the new snapshot is published only after a quiet 200 ms. Unit test (gated `#[cfg(unix)]`-style guarded via `tempfile`, runs on Windows): create file → assert new snapshot delivered within 500 ms; measure latency between fs write and snapshot publish across 20 iterations, assert p95 < 250 ms. (Use `std::time::Instant`; record raw timings in test output.)
- task-3: **`matcher::buffer` + `matcher::boundary`** — `MatchBuffer` holds up to 64 chars in a `VecDeque<char>`. API: `push_char(c)`, `pop_char()` (backspace), `reset()`, `as_str() -> String`, `boundary_state()` (tracks the char immediately *before* the current potential match — start-of-buffer | whitespace | punctuation | other). Boundary rule: a trigger matches only when the preceding state is **start | whitespace | punctuation**. Reset events: arrow keys / Home / End / PageUp / PageDown / IME composition start / Caps lock toggle / focus change. Expose `Reset` enum for these — the hook (Phase 3) emits them; here, just take them as inputs in tests.
- task-4: **`matcher::automaton`** — Build an `aho_corasick::AhoCorasick` (leftmost-longest, case-sensitive) over the trigger set. API: `Matcher::rebuild(snippets: &[Snippet])`, `Matcher::on_char(&mut buffer, c) -> Option<MatchHit>` where `MatchHit { snippet_id, trigger_len_chars }`. Greedy = longest match wins. Hot path **must not allocate** beyond AC's internal state: no `Vec::push`, no `format!`, no `Box::new` in `on_char`. `tests/matcher_basic.rs` covers: longest-match wins (`;sig` vs `;signature`), word-boundary required (no match for `aa;sig`), backspace pop reverts a near-miss, full reset clears buffer, multi-byte UTF-8 trigger (`;π` say).

## Context
- Phase 1 left empty `src-tauri/src/{store,matcher}/mod.rs` files with `//! TODO` headers and registered them in `lib.rs`. Replace them.
- Crate layout: `openmacro/src-tauri/` is a single Tauri 2 binary crate. Library code lives in `src/lib.rs`; integration tests in `src-tauri/tests/`.
- Tokio is **not** yet in `Cargo.toml`. Add it with the `rt-multi-thread`, `macros`, `sync`, `time` features. Watcher uses `notify = "6"` (or latest 6.x). AC: `aho-corasick = "1"`. Serde: `serde = { version = "1", features = ["derive"] }`, `serde_yaml = "0.9"`.
- Snippet IDs: derive from `format!("{}::{}", source_file_relative_to_root, trigger)` so duplicate triggers across files don't collide silently; collision-within-a-file is a `LoadError`. Document the chosen scheme in `notes.md` task-1.
- Coverage target ≥ 80% line — record `cargo llvm-cov --summary-only` output under `notes.md` task-4 Test evidence. If `cargo-llvm-cov` is not installed and installing is heavy, skip and write a "Spec deviation: coverage tool unavailable — manual review of test set instead".

## Files
- `F:/projects_new/textblaze/openmacro/src-tauri/src/store/mod.rs` (replace stub — re-exports + types)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/store/{model.rs,loader.rs,watcher.rs}`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/matcher/mod.rs` (replace stub)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/matcher/{buffer.rs,boundary.rs,automaton.rs}`
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/store_yaml.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/matcher_basic.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml`

## Done When
- `cd F:/projects_new/textblaze/openmacro/src-tauri && cargo test --all-features` — all green, including `store_yaml` and `matcher_basic`
- `cd F:/projects_new/textblaze/openmacro/src-tauri && cargo clippy -- -D warnings` — exit 0
- Matcher `on_char` hot path audited (no allocation), document the audit in `notes.md` task-4 (just list the function and assert "no Vec::push / format! / Box::new on this path")
- Watcher p95 latency assertion present and passing in the watcher test (record raw timings in test stdout)
- Loader never panics on bad YAML — every error path returns `Result`/captured in `LoadError`

## Rules

Follow the contract in `F:/projects_new/textblaze/.agents/shared/worker-contract.md` — per-task workflow (test-first → one commit per task `phase-2.task-<M>: …` → append a `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-02/notes.md` → append the `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-02/journal.md`) plus discipline (test-first, root-cause-first, evidence) and prompt discipline.

## Response Format

Respond per `F:/projects_new/textblaze/.agents/shared/erp.md`.
