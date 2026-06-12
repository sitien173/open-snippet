## Original User Request

Complete `docs/plans/2026-06-12-manual-trigger-and-fixes`, routing failing work to another worker when needed. Phase 1 is complete; execute Phase 2 now.

## Phase

Phase 2: Custom prefix refactor (backend).

## Route

# ROUTE
- Owner: Codex
- Reason: Backend/store loader schema and Rust tests.
- Done When: `cd src-tauri && cargo test loader_prefix`; `cd src-tauri && cargo test`; `cd src-tauri && cargo fmt --check`; acceptance criteria below are met.

If Codex fails before implementation because of the known unsupported-model account issue, the coordinator will reroute under the user's instruction.

## Tasks

- task-1: Write failing Rust tests first in `F:/projects_new/textblaze/src-tauri/tests/loader_prefix.rs` for the Phase 2 behaviors: default prefix preserves existing literal `;foo` triggers, custom prefix applies to bare triggers, existing prefixed triggers are not double-prepended, `trigger_literal: true` uses the raw trigger, invalid prefixes are rejected, and duplicate effective triggers are rejected with an error that names the effective trigger. Record RED evidence.
- task-2: Implement the minimal store model and loader changes to pass the tests. Use a root-scoped settings file at `F:/projects_new/textblaze/<snippets-root>/_settings.yaml` with `trigger_prefix: ";"` default. The recursive snippet loader must not treat `_settings.yaml` as a snippet file. Keep `matcher/automaton.rs` unchanged.
- task-3: Wire persistence/IPC schema surfaces needed for the backend contract: add `trigger_literal` to loaded `Snippet`, snippet command DTOs, and YAML save/read paths so existing snippet saves do not drop the field. Ensure watcher reloads naturally when `_settings.yaml` changes. Add or update tests for save/load behavior if needed.

## Design Requirements

From `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes-design.md` and `PLAN.md`:

- Global setting: `trigger_prefix: ";"`, default `";"`.
- Per-snippet optional setting: `trigger_literal: true`.
- Effective trigger:
  - if `trigger_literal` is true, use `trigger` verbatim
  - else if `trigger.starts_with(prefix)`, use `trigger` unchanged
  - else use `prefix + trigger`
- Prefix validation: length 1-3 chars, punctuation/symbol only, no letters, digits, or whitespace. Reject invalid prefixes with a clear load error.
- Duplicate effective triggers after normalization are load errors and the error message must reference the effective trigger.
- Existing snippets with literal `;foo` triggers continue to work without edits.
- Changing `trigger_prefix` to `:` with a bare `trigger: email` yields `:email`.
- No matcher automaton changes.

## Files

- F:/projects_new/textblaze/src-tauri/src/store/model.rs
- F:/projects_new/textblaze/src-tauri/src/store/loader.rs
- F:/projects_new/textblaze/src-tauri/src/store/watcher.rs
- F:/projects_new/textblaze/src-tauri/src/store/mod.rs
- F:/projects_new/textblaze/src-tauri/src/commands/snippets.rs
- F:/projects_new/textblaze/src-tauri/tests/loader_prefix.rs
- F:/projects_new/textblaze/src-tauri/tests/store_yaml.rs

## Rules

Follow `F:/projects_new/textblaze/.agents/shared/worker-contract.md` and `F:/projects_new/textblaze/.agents/shared/erp.md`.

Feature/refactor work is test-first. Do not write production code until a failing test exists and RED has been observed. Append one `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-02/notes.md` after each task. Make one commit per task with subjects `phase-2.task-<M>: <summary>`. Append the required `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-02/journal.md`.

Use `coderecall` for semantic codebase search, `tgrep` for literal search, and do not use `rg`, `grep`, or web search for codebase lookup in this repository. Prefix shell commands with `rtk`.

## Response Format

Return the `# EXTERNAL RESPONSE` block, then the single completion line:

`Phase 2 completed. Journal: docs/plans/2026-06-12-manual-trigger-and-fixes/phase-02/journal.md.`
