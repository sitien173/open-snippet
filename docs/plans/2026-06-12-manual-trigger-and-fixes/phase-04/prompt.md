## Original User Request

Complete `docs/plans/2026-06-12-manual-trigger-and-fixes`, routing failing work to another worker when needed.

## Phase

Phase 4: Manual trigger feature (backend).

## Route

# ROUTE
- Owner: Codex
- Reason: Backend Rust feature touching matcher/orchestrator/hook/settings and Rust integration coverage.
- Done When: `cd src-tauri && cargo test matcher_manual_mode matcher_armed_dismiss`; `cd src-tauri && cargo test`; `cd src-tauri && cargo fmt --check`; acceptance criteria below are met.

## Tasks

- task-1: Write failing Rust tests first for manual mode settings/defaults and the manual armed-state behavior. Cover default `ExpandMode::Manual`, persisted `expand_mode: auto`, arm-on-match without immediate injection, Tab confirm, Enter confirm, and Auto mode unchanged. Record RED evidence.
- task-2: Add `ExpandMode { Auto, Manual }` to the store settings model with default `Manual`, serde lowercase, and persistence through existing `get_store_settings` / `set_store_settings`. Plumb loaded store settings into the runtime so the orchestrator can see the current expand mode after `_settings.yaml` reloads.
- task-3: Implement manual armed state in the runtime path. On a full trigger match in Manual mode, store the matched snippet id and trigger length and do not expand. On confirm, expand with the existing resolver/injector behavior. Auto mode must bypass this path and behave like today.
- task-4: Implement dismissal and hook confirm-key behavior. Any normal char, whitespace, Backspace, arrow/navigation reset, focus reset, IME/composition reset, or caps reset must disarm. Modifier-only key events should continue to produce no hook event and should not disarm. Tab/Enter confirmations must be consumed by the hook when armed, then delivered to the orchestrator as explicit confirm events.

## Important Architecture Constraint

The current low-level keyboard hook always forwards events with `CallNextHookEx`, while the orchestrator processes events later from a ring buffer. Returning `true` from `Orchestrator::handle_event` is not enough to consume Tab/Enter in the foreground app.

Implement consumption at the hook boundary. A minimal acceptable shape is:
- Add explicit hook events such as `HookEvent::Confirm(ConfirmKey)` for Tab/Enter.
- Expose a small hook/shared armed flag or equivalent state set by the orchestrator when manual mode arms and cleared on confirm/dismiss/reset.
- In the Win32 hook callback, when a Tab/Enter keydown arrives and the armed flag is set, push the confirm event and return a non-forwarding result so the key is swallowed. When not armed, preserve existing behavior: Enter should still translate to `HookEvent::Char('\r')`, and Tab should be forwarded normally unless you introduce a no-op event with tests.
- Add focused unit tests around the pure decision logic for confirm consumption; do not rely only on end-to-end manual testing.

## Backend Contract From Earlier Phases

- Global store settings live in `snippets/_settings.yaml` and are represented by `StoreSettings` in `src-tauri/src/store/model.rs`.
- Existing store settings currently contain `trigger_prefix`; extend this model rather than adding a second settings file.
- `Snippet.trigger` is the effective matcher trigger; `Snippet.raw_trigger` is the editable YAML trigger.
- Existing `get_store_settings` and `set_store_settings` commands are the IPC surface to extend for Phase 5.
- `set_store_settings` must keep triggering backend reload so runtime expand mode changes take effect without restart.

## Design Requirements

- Manual mode is the default for new installs and missing `expand_mode`.
- Auto mode preserves current immediate expansion behavior.
- Manual mode behavior:
  - Typing a full trigger arms but does not inject.
  - Next Tab or Enter confirms, expands, and is consumed.
  - Any other non-modifier key class disarms silently and the app receives that key.
  - All existing reset reasons disarm.
  - IME composition start disarms.
  - Modifier-only key events do not disarm.
- No per-snippet expand mode.
- Do not duplicate snippet resolution/injection logic unnecessarily; share the existing injection flow where practical.

## Files Likely In Scope

- `F:/projects_new/textblaze/src-tauri/src/store/model.rs`
- `F:/projects_new/textblaze/src-tauri/src/store/loader.rs`
- `F:/projects_new/textblaze/src-tauri/src/store/watcher.rs`
- `F:/projects_new/textblaze/src-tauri/src/commands/snippets.rs`
- `F:/projects_new/textblaze/src-tauri/src/hook/mod.rs`
- `F:/projects_new/textblaze/src-tauri/src/hook/thread.rs`
- `F:/projects_new/textblaze/src-tauri/src/hook/ring.rs`
- `F:/projects_new/textblaze/src-tauri/src/engine/orchestrator.rs`
- `F:/projects_new/textblaze/src-tauri/tests/matcher_manual_mode.rs`
- `F:/projects_new/textblaze/src-tauri/tests/matcher_armed_dismiss.rs`

## Review Expectations

- Tests must prove Manual is default, Auto is unchanged, Tab and Enter confirm, and confirm keys are consumed at the hook decision point.
- Existing orchestrator tests that expect immediate expansion should explicitly choose Auto mode if the new default would otherwise change their intent.
- Review the reset map: arrow keys, Home, End, PageUp, PageDown, IME/composition, caps, and foreground changes should all clear armed state.
- Keep changes surgical. Do not add frontend UI in this phase.
- If a GUI/manual smoke test is not possible in the worker environment, state that limitation in notes; do not claim it was done.

## Rules

Follow `F:/projects_new/textblaze/.agents/shared/worker-contract.md` and `F:/projects_new/textblaze/.agents/shared/erp.md`.

Feature work is test-first. Do not write production code until a failing test exists and RED has been observed. Append one `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-04/notes.md` after each task. Make one commit per task with subjects `phase-4.task-<M>: <summary>`. Append the required `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-04/journal.md`.

Use `coderecall` for semantic codebase search, `tgrep` for literal search, and do not use `rg`, `grep`, or web search for codebase lookup in this repository. Prefix shell commands with `rtk`.

## Response Format

Return the `# EXTERNAL RESPONSE` block, then the single completion line:

`Phase 4 completed. Journal: docs/plans/2026-06-12-manual-trigger-and-fixes/phase-04/journal.md.`
