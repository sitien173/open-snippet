# Phase 1 — Decision Notes

## Task 1
- Decisions made: We bounded diagnostics by creating `matcher_enter_burst.rs` for hypothesis A, and added `DROPPED_EVENTS` metric for hypothesis C.
- Spec deviations: none
- Tradeoffs accepted: Used unit tests to verify Aho-Corasick matching of `\r` instead of end-to-end integration tests due to the complexity of the hook lifecycle.
- Assumptions: A text-expander user might reasonably type up to 100 Enters before a trigger.
- Follow-ups for human: none
- Test evidence: `matcher_enter_burst.rs` passes natively. `\r` correctly counts as whitespace. We confirm Hypothesis B manually by reading code and behavior of `SendInput` asynchronously vs synchronous `SUSPEND.store(false)`.

## Task 2
- Decisions made: Removed `SUSPEND.load` entirely from the hook's ignore logic. `LLKHF_INJECTED_FLAG` is now unconditionally ignored, fixing the race condition where `SUSPEND` toggles too quickly.
- Spec deviations: none
- Tradeoffs accepted: This disables tracking of injected text from other automation tools. Since Textblaze is a text expander, expanding on injected text is an anti-pattern (it leads to loops), so we accept this.
- Assumptions: none
- Follow-ups for human: none
- Test evidence: Added `should_ignore_event_logic` test, confirming that injected events are ignored regardless of the `SUSPEND` value. RED -> GREEN observed when changing `should_ignore_event`.

## Task 3
- Decisions made: Dropped the `DROPPED_EVENTS` metric added in Task 1 because Hypothesis C was rejected and the metric adds minimal value.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: none
- Follow-ups for human: none
- Test evidence: none

## Task 4
- Decisions made: Implement `INJECTED_MARKER` (defined as `0x5442_4C5A`) in `sendinput.rs` and verify it in the keyboard hook. This ensures we only ignore our own injected events. We also reverted the formatting-only changes in `orchestrator.rs` and `inject/mod.rs` as they were out of scope.
- Spec deviations: Reverted out-of-scope formatting-only files (`orchestrator.rs` and `inject/mod.rs`), which causes global `cargo fmt --check` to fail but keeps files untouched.
- Tradeoffs accepted: Unmarked injected events from other apps are no longer ignored, focusing strictly on resolving re-entrancy of this app's own injected inputs.
- Assumptions: none
- Follow-ups for human: none
- Test evidence:
  - Updated `matcher_enter_burst.rs` to simulate 10 Enters to match acceptance criteria.
  - Updated `findings.md` to match the actual test behavior (10 Enters instead of 100) and explicitly documented the GUI smoke testing limitation in a headless environment.
  - Added new assertions in `should_ignore_event_logic` test to verify marked/unmarked/non-injected events ignore behavior.
  - Cleaned up unused import (`HKL`) and unused inner constant (`LLKHF_INJECTED_FLAG`) to resolve compiler warnings.
  - All tests passed.

## Task 5
- Decisions made: Apply global `cargo fmt` to the `src-tauri` directory so that `cargo fmt --check` integration check passes, retaining formatting-only changes in `orchestrator.rs` and `inject/mod.rs` as required by the phase integration check. Corrected the stale Hypothesis C wording in `findings.md` to match the final state after the removal of the `DROPPED_EVENTS` metric.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: none
- Follow-ups for human: none
- Test evidence: `cd src-tauri && cargo fmt --check` passes successfully. All cargo tests pass.
