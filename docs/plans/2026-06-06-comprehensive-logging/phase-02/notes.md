# Phase 2 - Decision Notes

## Task 1

### Decisions made (not in spec)
- Preserved `Debug` field rendering exactly as `format!("{value:?}")` instead of attempting to normalize string-looking values.

### Spec deviations
- none

### Tradeoffs accepted
- Debug string fields now include outer quotes and escapes; typed string fields recorded through `%` still store the plain string through `record_str`.

### Assumptions
- Future log viewers can display debug-rendered fields as-is when call sites intentionally use `?field`.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test --test log_init_ring ring_layer_preserves_debug_string_escapes` failed because the field value lost the outer debug quotes.
- GREEN: `rtk cargo test --test log_init_ring` passed 3 tests.

## Task 2

### Decisions made (not in spec)
- Logged typed hook characters through `FieldKind::FormValue` redaction; default logs expose only `<redacted len=1>`, while verbose mode may expose the character by explicit opt-in.
- Logged sync remotes/paths as operational metadata and skipped PAT/auth payloads in spans.

### Spec deviations
- none

### Tradeoffs accepted
- Instrumentation favors counts, identifiers, and redacted fields over verbose payload logging to keep security-sensitive paths conservative.

### Assumptions
- Trigger names, snippet IDs, file paths, executable basenames, branch names, and sync remote strings are acceptable developer-debug metadata under the design's safe field categories.

### Follow-ups for human
- none

### Test evidence
- RED: n/a for instrumentation-only changes; baseline checks showed no existing `println!`/`eprintln!` call sites to replace.
- GREEN: `rtk cargo test --no-run` compiled the instrumented backend.
- GREEN: `rtk tgrep -l "println!\\|eprintln!" src-tauri/src` returned no matches.
- GREEN: sensitive-term scan showed changed payload sites routed through `log_body!`, `redact_str`, skip fields, or count-only logging.
- GREEN: `rtk cargo test` passed 88 tests, 1 ignored.

## Task 3

### Decisions made (not in spec)
- Picked runtime-init option 1: manage the effective `LoggingConfig` passed to `log_init::init` and have `get_logging_frontend_cfg` read that managed state.
- Kept test coverage on command inner functions to validate command return shapes without needing a full Tauri harness.

### Spec deviations
- none

### Tradeoffs accepted
- `get_logging_frontend_cfg` returns the current effective frontend block; it does not re-read disk because the app has no persisted app-wide logging config file yet.

### Assumptions
- Phase 3 will consume `LoggingFrontendConfig` (`level`, `modules`) as the backend command shape specified in this phase.

### Follow-ups for human
- If the frontend needs `verbose_content` in the same command payload, extend the command contract before Phase 3 implementation.

### Test evidence
- RED: `rtk cargo test --test commands_logging` failed with unresolved import `openmacro_lib::commands::logging`.
- GREEN: `rtk cargo test --test commands_logging` passed 2 tests.

## Task 4

### Decisions made (not in spec)
- Added `#[test]` alongside `#[tracing_test::traced_test]`; in this project the traced macro alone did not register the sync roundtrip function as a test.

### Spec deviations
- none

### Tradeoffs accepted
- Used annotation-only traced smoke tests instead of `logs_contain` assertions to avoid coupling tests to exact log message text.

### Assumptions
- The selected matcher and sync tests are representative because they exercise instrumented matcher rebuild/match and sync init/tick/status paths.

### Follow-ups for human
- none

### Test evidence
- RED: initial `#[tracing_test::traced_test]`-only run reported `sync_roundtrip` as 0 tests and reduced matcher test count.
- GREEN: `rtk cargo test --test matcher_basic` passed 9 tests.
- GREEN: `rtk cargo test --test sync_roundtrip` passed 1 test.
- GREEN: `rtk cargo test --test commands_logging` passed 3 tests, including JSON file sink evidence for structured `target`/`level` and redacted `fields.body`.
