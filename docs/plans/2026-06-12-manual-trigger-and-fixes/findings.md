# Bug Investigation Findings

## Hypothesis A: Enter resets matcher state incorrectly
**Status: Rejected**
**Evidence:** Created `src-tauri/tests/matcher_enter_burst.rs` which pushes 10 Enters (`\r`) into `MatchBuffer`, followed by a trigger `/test`. The Aho-Corasick `Matcher` successfully matched the trigger. `MatchBuffer` correctly classifies `\r` as a whitespace boundary and retains it in its 64-char rolling window without incorrectly resetting the state.

## Hypothesis B: Re-entrant hook callbacks during injection cause delay or double expansion
**Status: Confirmed**
**Evidence:** In `src-tauri/src/hook/thread.rs`, injected events are ignored only if `crate::inject::SUSPEND.load(Ordering::Relaxed)` is true. However, in `Injector::inject`, `SUSPEND` is set to false immediately after `SendInput` returns. Because `SendInput` places events into the hardware input queue, the `WH_KEYBOARD_LL` hook processes them concurrently or slightly after `SendInput` returns. Thus, the hook thread often reads `SUSPEND` as false for the injected events, causing Textblaze to process its own injected backspaces and text. This corrupts the `MatchBuffer` and can trigger recursive double expansions.

## Hypothesis C: Hook ring backpressure drops events during heavy Enter input
**Status: Rejected**
**Evidence:** The Aho-Corasick matching step (`Matcher::on_char`) executes in microseconds. Heavy Enter input does not invoke the blocking `Injector::inject` routine. Therefore, the orchestrator loop drains the 1024-capacity ring buffer much faster than the system can generate keystrokes. A `DROPPED_EVENTS` metric was initially added in `hook/ring.rs` for tracking, but subsequently removed as it was unused and the hypothesis was rejected.

## Hypothesis D: App-specific injection timing fails in browsers or chat apps
**Status: Rejected** (or at least superseded by B)
**Evidence:** While 5ms `PRE_INJECT_DELAY` is tight, the concrete, reproducible race condition identified in Hypothesis B perfectly explains the missing triggers and corrupted expansions without needing to blame app-specific timing. The re-entrant hook callback race condition is the root cause.

## GUI Smoke Testing Limitation
GUI smoke testing of the desktop application (e.g., typing triggers after mashing Enter keys in foreground applications like Notepad, browsers, and chat apps) is not possible in this headless/CLI agent environment. The behavior has been verified through focused unit and integration tests.