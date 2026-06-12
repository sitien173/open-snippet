# Phase 3 — Decision Notes

## Task 1
- Decisions made:
  - Sourced `verboseContent` only from `import.meta.env.VITE_LOG_VERBOSE === "1"` on the frontend side because the Rust-side `get_logging_frontend_cfg` IPC payload does not contain `verbose_content`.
  - Configured `loglevel-plugin-prefix` with a custom `timestampFormatter` returning `date.toISOString()` to ensure the log entries formats properly without throwing format errors.
- Spec deviations:
  - none
- Tradeoffs accepted:
  - none
- Assumptions:
  - none
- Follow-ups for human:
  - none
- Test evidence:
  - none (covered by tests in Task 2)

## Task 2
- Decisions made:
  - Asserted prefix-inclusion in ring buffer matching the `Wrap *before*` requirement.
- Spec deviations:
  - none
- Tradeoffs accepted:
  - none
- Assumptions:
  - none
- Follow-ups for human:
  - none
- Test evidence:
  - `logger.test.ts` passes with 5 tests, covering ring writes, capped FIFO length of 1000, case-insensitive anchored redaction, and code-point counts for unicode.

## Task 3
- Decisions made:
  - Used logical OR `(envLevel || cfg.level)` instead of nullish coalescing to prevent logger initialization failure when `VITE_LOG_LEVEL` environment variable is defined as an empty string.
- Spec deviations:
  - none
- Tradeoffs accepted:
  - none
- Assumptions:
  - none
- Follow-ups for human:
  - none
- Test evidence:
  - `logger-init.test.ts` passes with 4 tests, covering configuration application, precedence of environment level over IPC, verbose mode toggle, and recovery/defaulting on IPC failure.

## Task 4
- Decisions made:
  - Mapped console.error logs to module specific loggers (`form`, `settings`, `settings.prefs`, `settings.sync`) with the second argument wrapping `err`/`error` inside a flat fields object: `{ error: err }`.
- Spec deviations:
  - none
- Tradeoffs accepted:
  - none
- Assumptions:
  - none
- Follow-ups for human:
  - none
- Test evidence:
  - Production build succeeds cleanly via `pnpm build`.
  - All existing and new tests pass cleanly with `pnpm test`.
  - No `console.{log,info,warn,error,debug}` usages remain in the 4 target files.



