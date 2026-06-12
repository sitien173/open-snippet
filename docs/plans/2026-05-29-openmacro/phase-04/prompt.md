## Original User Request
openmacro Phase 4 — `{{date}}`, `{{time}}`, `{{clipboard}}`, `{{cursor}}` resolvers + `$|$` cursor positioning. Wire into orchestrator before injection.

## Phase
Implement `expand/` and extend `engine::orchestrator` so static `snippet.replace` strings are run through a resolver that substitutes `{{name}}` placeholders and computes a caret offset for `$|$`.

## Tasks
- task-1: **`expand::resolver`** — Parser for `{{name}}` and `{{name:format-arg}}` (single optional arg). Resolves names against the snippet's declared `vars` first; built-in fallbacks: `date`, `time`, `datetime` (chrono `Local::now()`, strftime format from arg, defaults `%Y-%m-%d` / `%H:%M:%S` / `%Y-%m-%d %H:%M:%S`); `clipboard` (calls `ClipboardReader::read_text() -> Option<String>`, empty string if non-text). Trait-injected reader so unit tests don't touch real clipboard. Unknown name → `ResolveError::UnknownPlaceholder { name }`. Returns `Resolved { text, cursor_chars_after_token: Option<usize> }`.
- task-2: **`expand::cursor`** — `$|$` token handling. Parse-time validation: count occurrences; 0 = `cursor_chars_after_token = None`; 1 = strip and record (UTF-16 code units **after** the token in the resolved final string); ≥2 = `LoadError::TooManyCursorTokens` (this means Phase 2 `loader` must add this validation — yes, modify it). Wire it via a new validation pass in `loader::validate_snippet`. Strip token before returning the resolved text. `tests/cursor_math.rs` covers UTF-16 math for multi-byte chars (emoji, CJK).
- task-3: **`expand::datetime` + `expand::clipboard_var`** — Concrete resolver helpers used by task-1. Add `chrono = "0.4"` to Cargo.toml with `default-features = false, features = ["clock", "std"]`. For the clipboard var the reader trait must read **before** the injector saves clipboard (called in orchestrator before `Injector::inject`). Document the ordering in notes.md task-3.
- task-4: **Orchestrator integration** — Extend `engine::orchestrator`: on `MatchHit`, call `Resolver::resolve(snippet, &mut clipboard_reader)`, on `Err(UnknownPlaceholder)` emit a tray notification via `AppHandle` (use a `NotifySink` trait so unit tests inject a fake) and skip injection; on `Ok(Resolved { text, cursor })`, build `InjectPlan { backspaces, text, caret_left: cursor.unwrap_or(0) }` and after settle, send `caret_left` × `VK_LEFT` via `SendInput`. Add a `Resolved` round-trip test that exercises `;now` and `;log` snippets end-to-end with mocked sinks.

## Context
- Phase 3 left `engine::orchestrator` with a static-text inject path. This phase replaces that single path with the resolver pipeline.
- Phase 2 `loader::validate_snippet` does not exist yet; add it and call it from `loader::load_dir` per file. Update the existing `store_yaml.rs` tests if they accidentally include `$|$` (most don't).
- Clippy: still `-D warnings`.
- Coverage target ≥ 80% on `expand/` + `matcher/` + `store/`. If `cargo-llvm-cov` still missing, record deviation in notes.

## Files
- `F:/projects_new/textblaze/openmacro/src-tauri/src/expand/{mod.rs,resolver.rs,cursor.rs,datetime.rs,clipboard_var.rs}`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/engine/orchestrator.rs` (extend)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/store/loader.rs` (add `validate_snippet`)
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/expand_resolver.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/cursor_math.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml`

## Done When
- `cargo test --all-features` green
- `cargo clippy --all-targets -- -D warnings` clean
- `tests/expand_resolver.rs` covers: all four var kinds, unknown-name failure, strftime arg, missing clipboard
- `tests/cursor_math.rs` covers: 0/1/2-occurrence validation, UTF-16 math for an emoji-containing replace string, ASCII-only baseline
- `notes.md` task-3 explains clipboard read ordering relative to injector save

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-04/`.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
