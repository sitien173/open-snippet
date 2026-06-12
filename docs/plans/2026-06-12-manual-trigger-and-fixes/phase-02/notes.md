# Phase 2 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Added a dedicated `src-tauri/tests/loader_prefix.rs` integration test file so prefix behavior stays isolated from existing YAML coverage.
### Spec deviations
- none
### Tradeoffs accepted
- Focused the RED run on `cargo test --test loader_prefix` instead of the full Rust suite to isolate failing behavior before implementation.
### Assumptions
- The effective trigger should continue to define snippet ids, so the tests assert normalized ids where the effective trigger changes.
### Follow-ups for human
- none
### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `rtk cargo test --test loader_prefix` failed in five cases.
- Root cause evidence:
- `_settings.yaml` is parsed as a snippet file and fails with `missing field 'snippets'`.
- Bare triggers are left unprefixed under custom settings.
- Duplicate detection does not reject cross-file duplicate effective triggers like `:email`.

## Task 2
### Decisions made (not in spec)
- Added a root-scoped `StoreSettings` model with a default `trigger_prefix` of `;` so loader normalization can stay backend-local.
### Spec deviations
- none
### Tradeoffs accepted
- Duplicate effective triggers are removed from the loaded snapshot rather than keeping an arbitrary winner.
### Assumptions
- A `DuplicateTrigger` load error is the correct existing error shape for normalized trigger collisions as long as the effective trigger appears in the message.
### Follow-ups for human
- none
### Test evidence (RED→GREEN, or root cause for a fix)
- GREEN: `rtk cargo test --test loader_prefix` passed with 6 tests after the loader changes.
- Root cause fixed:
- `_settings.yaml` is now loaded once as root settings and skipped during recursive snippet file collection.
- Effective trigger normalization now happens before snippet ids and duplicate checks are finalized.

## Task 3
### Decisions made (not in spec)
- Marked `SaveSnippetDto.trigger_literal` with `#[serde(default)]` so older callers deserialize as `false` instead of failing hard.
### Spec deviations
- none
### Tradeoffs accepted
- Kept the persistence regression in `commands_roundtrip.rs` instead of introducing a separate backend-contract test file.
### Assumptions
- The backend list API should expose `trigger_literal` directly on `SnippetDto` because the loaded runtime `Snippet` now carries it.
### Follow-ups for human
- none
### Test evidence (RED→GREEN, or root cause for a fix)
- RED: `rtk cargo test --test commands_roundtrip` failed to compile because `SaveSnippetDto` and `SnippetDto` were missing `trigger_literal`.
- GREEN: `rtk cargo test --test commands_roundtrip` passed with 3 tests after wiring the field through DTOs and YAML save/read paths.

## Task 4
### Decisions made (not in spec)
- Preserved two trigger forms in the backend model: `Snippet.raw_trigger` for YAML/editor identity and `Snippet.trigger` for the effective matcher trigger.
- Added minimal `get_store_settings` / `set_store_settings` backend commands because Phase 3 had no existing IPC path to persist `snippets/_settings.yaml`.
- Added `SnippetDto.effective_trigger` while keeping `SnippetDto.trigger` as the raw editable trigger for editor compatibility.
### Spec deviations
- none
### Tradeoffs accepted
- Added `SaveSnippetDto.original_trigger_literal` to disambiguate raw-trigger row replacement when two YAML rows share the same raw trigger but differ by `trigger_literal`.
- Updated the shared `src/lib/snippets.ts` DTO shim in the same task so Phase 3 can consume the new backend contract without a second backend follow-up.
### Assumptions
- The matcher should continue reading `Snippet.trigger` as the effective trigger, so preserving the raw trigger must not alter matcher rebuild behavior.
- Phase 3 can call the new store-settings IPC instead of writing `_settings.yaml` directly from the frontend.
### Follow-ups for human
- none
### Test evidence (RED→GREEN, or root cause for a fix)
- Root cause evidence:
- `load_from_root` normalized the YAML trigger into `Snippet.trigger`, so `list_snippets_inner` exposed the effective trigger as the editable DTO trigger.
- `save_snippet_inner` matched `SaveSnippetDto.original_trigger` against the raw YAML `SnippetDocument.trigger`, so a listed `:email` could not match the stored `email` row and appended instead of replacing.
- RED: `rtk cargo test --test commands_roundtrip` failed to compile with missing `SnippetDto.effective_trigger`, missing `SaveSnippetDto.original_trigger_literal`, and missing store-settings helper functions.
- GREEN: `rtk cargo test --test commands_roundtrip` passed with 6 tests after preserving raw/effective triggers separately and adding the settings helpers.
- GREEN: `rtk cargo test --test loader_prefix` passed with 6 tests after the contract fix, confirming the original Phase 2 loader behaviors still hold.
- Verification: `rtk cargo check` passed with no warnings, `rtk cargo fmt --check` passed, and `rtk cargo test` passed with 108 tests and 1 ignored after the warning cleanup.

## Task 5
### Decisions made (not in spec)
- Reused `hook::winevent::test_sync::global_state_guard()` as the single shared test lock for both hook state and inject `SUSPEND` state.
### Spec deviations
- none
### Tradeoffs accepted
- Kept the fix entirely under `#[cfg(test)]` so runtime injection/orchestrator behavior stays unchanged.
- Added a focused guard-contract test instead of trying to force a deterministic cross-test race.
### Assumptions
- The isolated shell test and the earlier full-suite orchestrator flake share the same root cause: another test leaves `SUSPEND` true while orchestrator input handling runs.
### Follow-ups for human
- none
### Test evidence (REDâ†’GREEN, or root cause for a fix)
- Root cause confirmed in code:
- `Injector::inject` sets `crate::inject::SUSPEND` true/false in `src-tauri/src/inject/mod.rs`.
- `Orchestrator::handle_event` returns early when `SUSPEND.load(Ordering::Relaxed)` is true in `src-tauri/src/engine/orchestrator.rs`.
- Orchestrator tests already serialize through `hook::winevent::test_sync::global_state_guard()`, but injector tests previously mutated `SUSPEND` without taking that same lock.
- RED: `rtk cargo test --lib global_state_guard_resets_suspend_flag -- --nocapture` failed before the fix because the shared test guard reset `DENYLISTED` only and left `SUSPEND` unchanged.
- GREEN: `rtk cargo test --lib global_state_guard_resets_suspend_flag` passed with `1 passed, 33 filtered out`.
- GREEN: `rtk cargo test shell_snippet_injects_backend_output_when_consent_enabled -- --nocapture` passed with `1 passed, 109 filtered out`.
- GREEN: `rtk cargo test` passed with `109 passed, 1 ignored`.
- GREEN: `rtk cargo fmt --check` passed.

## Task 6
### Decisions made (not in spec)
- Coordinator updated existing frontend test fixtures to include the new required `Snippet.effective_trigger` and `Snippet.trigger_literal` fields introduced by the Phase 2 backend/frontend binding contract.
### Spec deviations
- none
### Tradeoffs accepted
- Kept this as a fixture-only change instead of routing another worker pass because runtime frontend behavior was not changed.
### Assumptions
- Fixture effective triggers should match raw triggers until Phase 3 adds custom-prefix UI coverage.
### Follow-ups for human
- none
### Test evidence (RED->GREEN, or root cause for a fix)
- RED: `rtk pnpm build` failed because four existing test fixtures were missing `effective_trigger` and `trigger_literal`.
- GREEN: `rtk pnpm build` passed after fixture updates.
- GREEN: `rtk pnpm test` passed with 12 files / 50 tests.
- GREEN: `rtk pnpm lint` passed with zero warnings.
