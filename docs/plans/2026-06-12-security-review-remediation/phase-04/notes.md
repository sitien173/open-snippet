# Phase 4 — Decision Notes

## Task 1
- Decisions made: Added failing test for frontend privacy leak by spying on underlying console object to ensure redaction.
- Spec deviations: none
- Tradeoffs accepted: Mocked `console.log` directly in Vitest to observe the lowest-level output sent to the terminal.
- Assumptions: The `Logger` instance uses the global `console` object for fallback formatting.
- Follow-ups for human: none
- Test evidence: RED reproduced by temporarily changing `src/lib/logger.ts` back to `raw(msg, fields ?? "")` and running `rtk pnpm exec vitest run src/lib/__tests__/logger.test.ts -t "redacts sensitive keys in underlying console output by default" --reporter=basic`; the focused test failed because console output contained `{"secret":"do-not-print-me"}` and did not contain `<redacted len=15>`. GREEN verified after restoring the implementation.

## Task 2
- Decisions made: Updated `logger.ts` to redact payload objects automatically.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: none
- Follow-ups for human: none
- Test evidence: RED reproduced with the Task 1 console-output test by temporarily changing `src/lib/logger.ts` back to `raw(msg, fields ?? "")`; the focused test failed because raw sensitive fields reached console output. GREEN verified after restoring `raw(msg, entry.fields ?? "")`.

## Task 3
- Decisions made: Added inline validation in `SyncPanel` before calling `sync_test_connection` or `sync_init`.
- Spec deviations: none
- Tradeoffs accepted: Used a basic regex/URL check for HTTPS URL validation to reject malformed hosts early while delegating the full check to Rust.
- Assumptions: We don't want to duplicate the full URI parsing of the Rust backend, just prevent obviously malformed inputs from making IPC calls.
- Follow-ups for human: none
- Test evidence: RED reproduced by temporarily removing the HTTPS-mode `startsWith("https://")` guard and running `rtk pnpm exec vitest run src/routes/settings/__tests__/SyncPanel.test.tsx -t "non-https remote for https auth shows inline error" --reporter=basic`; the focused test failed because no `HTTPS is required for this authentication mode` message rendered and the mocked init path succeeded. GREEN verified after restoring the guard.

## Task 4
- Decisions made: Updated `persistPrefs` in `PrefsPanel.tsx` to keep a reference to `previous` state. Upon error, we call `setPrefsState(previous)` to roll back the optimistic UI update, avoiding misleading user feedback. Added corresponding test.
- Spec deviations: none
- Tradeoffs accepted: Optimistic update is still used to maintain UI snappiness; rollback may cause a slight flicker on error, which is an acceptable tradeoff for correct final state.
- Assumptions: `setPrefsState(previous)` safely triggers a re-render to the last known good state.
- Follow-ups for human: none
- Test evidence: RED reproduced by temporarily removing the `setPrefsState(previous)` rollback in `persistPrefs` and running `rtk pnpm exec vitest run src/routes/settings/__tests__/PrefsPanel.test.tsx -t "autostart save failure rolls back toggle and shows error" --reporter=basic`; the focused test failed with `AssertionError: expected true to be false`, proving the toggle stayed optimistically checked. GREEN verified after restoring the rollback.

## Task 6
- Decisions made: Added failing frontend test proving malformed HTTPS input (`https://`) does not reach IPC. Tightened HTTPS-mode validation in `SyncPanel.tsx` to require a parseable URL with a non-empty hostname.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: The browser's native `URL` constructor behaves similarly enough to the backend's URL parsing to act as an effective frontend guardrail.
- Follow-ups for human: none
- Test evidence: RED reproduced by running `rtk pnpm exec vitest run src/routes/settings/__tests__/SyncPanel.test.tsx` which failed on the missing inline error before the fix. GREEN verified after adding `new URL()` validation check.
