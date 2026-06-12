# Phase 5 — Decision Notes

## Task 1
- Decisions made: Created failing tests for `expand_mode` settings (manual/auto radio) and migration notice. Mocks returned standard preferences and mocked `get_store_settings` with and without `expand_mode_missing`.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: The migration notice banner will be placed in `PrefsPanel.tsx`.
- Follow-ups for human: none
- Test evidence: Run Vitest which reported 4 failed tests, showing correct RED behavior.

## Task 2
- Decisions made: Added `ExpandMode` and `expand_mode` to `StoreSettings` frontend type definition. Added radio button controls to `PrefsPanel.tsx` and implemented `handleExpandModeChange` to persist selections. Updated `persistPrefix` to preserve `expand_mode` when saving.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: none
- Follow-ups for human: none
- Test evidence: Run Vitest, confirming that radio selection, persistence, and prefix-saving tests are now passing (GREEN), while the migration notice test remains RED.

## Task 3
- Decisions made: Implemented a dismissible migration notice banner using Flowforge decorative styling in `PrefsPanel.tsx`. When dismissed, it writes an explicit `expand_mode: "manual"` to the backend. Adjusted and fixed legacy Vitest tests in `PrefsPanel.test.tsx` and `SnippetEditor.test.tsx` to align with the new schema payload.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: none
- Follow-ups for human: none
- Test evidence: Run Vitest, confirming all 63 frontend tests are now completely GREEN.

## Task 4
- Decisions made: Added a `StoreSettingsDto` response for `get_store_settings` with transient `expand_mode_missing` metadata while keeping `set_store_settings` on the persisted `StoreSettings` shape.
- Spec deviations: none
- Tradeoffs accepted: The backend detects absence by reading `_settings.yaml` as a YAML value after normal settings load succeeds, avoiding a second settings parser in the frontend.
- Assumptions: Missing `expand_mode` should only count as an upgrade when `_settings.yaml` exists; no settings file remains a fresh install and returns `expand_mode_missing: false`.
- Follow-ups for human: none
- Test evidence: `rtk cargo test --test commands_roundtrip` passed with 7 tests after the migration DTO implementation.
