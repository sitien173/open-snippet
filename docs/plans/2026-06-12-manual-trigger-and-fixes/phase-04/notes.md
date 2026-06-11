# Phase 4 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Split the backend coverage into `matcher_manual_mode.rs` for manual-vs-auto confirmation flow and `matcher_armed_dismiss.rs` for dismissal/reset behavior so the acceptance criteria stay isolated.
- Extended `commands_roundtrip.rs` for the `expand_mode` default/persistence assertions instead of creating a third integration file just for settings serialization.
### Spec deviations
- none
### Tradeoffs accepted
- Used compile-failing RED tests for the missing backend contracts (`ExpandMode`, `ConfirmKey`, new reset causes, and manual-runtime APIs) because those types did not exist yet.
### Assumptions
- The new manual-trigger runtime should be exercised through `Orchestrator` integration tests rather than lower-level matcher-only tests because the feature spans hook events, runtime state, and injection behavior.
### Follow-ups for human
- none
### Test evidence (RED->GREEN, or root cause for a fix)
- RED: `rtk cargo test --test matcher_manual_mode` failed with unresolved `ConfirmKey`, unresolved `ExpandMode`, missing manual-runtime API surface, and missing `HookEvent::Confirm`.
- RED: `rtk cargo test --test matcher_armed_dismiss` failed with the same missing confirm/mode contracts plus missing reset causes for arrow/home/end/page keys.
- RED: `rtk cargo test --test commands_roundtrip` failed because `StoreSettings` did not yet contain `expand_mode`.

## Task 2
### Decisions made (not in spec)
- Kept `expand_mode` on the existing root `StoreSettings` document and threaded it through `LoadResult`/`SnapshotInner` so the watcher snapshot remains the single runtime reload input.
### Spec deviations
- none
### Tradeoffs accepted
- Left the final orchestrator-side consumption of `SnapshotInner.settings` for Task 3 because the runtime consumer seam and the armed/manual behavior live in the same file. Splitting that into a standalone intermediate commit would have left the runtime half-converted.
### Assumptions
- Phase 5 can continue to use the existing `get_store_settings` / `set_store_settings` IPC surface because serde now covers `expand_mode` without a new command shape.
### Follow-ups for human
- none
### Test evidence (RED->GREEN, or root cause for a fix)
- GREEN: `rtk cargo test --test commands_roundtrip` passed with 6 tests after adding `ExpandMode`, default `Manual`, lowercase serde, and watcher snapshot settings propagation.

## Task 3
### Decisions made (not in spec)
- Factored the old immediate-injection branch into `expand_trigger(...)` so Manual confirm and Auto immediate expansion share one resolver/injector path instead of duplicating form, shell, and placeholder handling.
- Added an explicit `armed` runtime record of `{ snippet_id, trigger_len_chars }` so confirm expansion can reuse the matched snippet and exact backspace count without re-running matcher state.
### Spec deviations
- none
### Tradeoffs accepted
- Updated pre-existing orchestrator tests to call `set_expand_mode(ExpandMode::Auto)` where they were asserting immediate expansion. That keeps their original intent explicit under the new Manual default.
### Assumptions
- Manual mode should discard the armed match before processing the next normal character, so the next key is delivered to the foreground app and the matcher starts fresh from that character.
### Follow-ups for human
- none
### Test evidence (RED->GREEN, or root cause for a fix)
- GREEN: `rtk cargo test --test matcher_manual_mode` passed with 4 tests covering arm-without-inject, Tab confirm, Enter confirm, and Auto immediate expansion.
- GREEN: the first full-suite rerun failed in existing orchestrator tests that still assumed immediate expansion under the new default; setting those tests to explicit `Auto` restored their original contract.

## Task 4
### Decisions made (not in spec)
- Implemented confirm consumption as a small pure hook decision step (`decide_keydown_event`) so Tab/Enter swallowing, navigation resets, and modifier-only pass-through are unit-testable without the full Win32 hook loop.
- Reused the existing shared global-state guard for the new integration tests by exposing the same lock/reset helper through `hook::winevent::test_sync`, and reset the confirm-armed flag inside that guard as well.
### Spec deviations
- none
### Tradeoffs accepted
- Exposed the existing test synchronization helper as a hidden public module so integration tests can share the same mutex as unit tests. This changes no runtime behavior but avoids a second unsynchronized lock around the same globals.
### Assumptions
- Confirm-arm state is process-global at the hook boundary, so tests that drive injector/orchestrator paths need to serialize on the same guard as other `SUSPEND`-touching tests.
### Follow-ups for human
- GUI/manual smoke confirmation was not possible in the worker environment; behavior is covered by hook decision tests plus Rust integration tests.
### Test evidence (RED->GREEN, or root cause for a fix)
- GREEN: `rtk cargo test --test matcher_armed_dismiss` passed with 3 tests covering disarm on normal char, Backspace, and every reset cause.
- GREEN: `rtk cargo test` passed with `119 passed, 1 ignored` after the new matcher tests adopted the shared global-state guard, which removed the remaining full-suite race on shared hook/inject globals.

## Task 5
### Decisions made (not in spec)
- Kept the fix entirely inside the hook boundary by adding a small dispatch helper that owns two concerns only: clearing the hook-side armed flag for disarming events, and returning whether a confirm key should actually be swallowed based on queue success.
### Spec deviations
- none
### Tradeoffs accepted
- Cleared `CONFIRM_ARMED` before queueing disarming events. That means a dropped disarming event still clears the hook-side swallow flag immediately, which matches the requested “clear synchronously at the hook boundary” behavior and avoids a stale swallow window.
### Assumptions
- If a confirm event cannot be queued, the correct fallback is to forward Tab/Enter and leave runtime reconciliation to later input, rather than swallowing a user key with no event delivered to the orchestrator.
### Follow-ups for human
- none
### Test evidence (RED->GREEN, or root cause for a fix)
- RED: `rtk cargo test hook::thread::tests` failed with `unresolved import super::handle_keydown_decision` after adding reproducer tests for hook-side disarm-before-Tab and swallow-only-on-success behavior.
- GREEN: `rtk cargo test hook::thread::tests` passed with `11 passed, 113 filtered out` after the hook dispatch helper started clearing `CONFIRM_ARMED` for translated chars / Backspace / reset events and only swallowing confirm keys when `producer.push(...)` succeeded.

## Task 6
### Decisions made (not in spec)
- Extended the existing hook-side disarm predicate rather than adding a second confirm-specific branch, so confirm handling still flows through the same dispatch helper and queue-success semantics as the Task 5 fix.
### Spec deviations
- none
### Tradeoffs accepted
- Clearing `CONFIRM_ARMED` before queueing `HookEvent::Confirm(_)` means even a forwarded confirm key clears stale armed state immediately. That matches the “next keystroke decides” design and avoids repeat swallowing after a dropped or forwarded confirm.
### Assumptions
- Once a confirm key is seen at the hook boundary, the hook-side armed latch should be consumed regardless of whether the confirm event is swallowed or forwarded.
### Follow-ups for human
- none
### Test evidence (RED->GREEN, or root cause for a fix)
- RED: `rtk cargo test hook::thread::tests` failed with 3 regressions: `successful_confirm_queue_clears_armed_immediately`, `failed_confirm_queue_clears_armed_and_does_not_swallow`, and `later_tab_decision_sees_unarmed_after_successful_confirm`, all because `is_confirm_armed()` stayed true after confirm dispatch.
- GREEN: `rtk cargo test hook::thread::tests` passed with `14 passed, 113 filtered out` after `HookEvent::Confirm(_)` started clearing the hook-side armed flag in the dispatch helper.
