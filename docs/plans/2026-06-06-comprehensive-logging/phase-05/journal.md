# Phase 5 — Logging docs

- Status: DONE
- Owner: coordinator
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Docs-only — short developer-facing doc covering env vars, YAML schema, redaction, sinks, viewer, bug-report checklist.
- Done When:
  - `docs/logging.md` exists and matches actual Phase 1–4 implementation (env vars, config keys, viewer behavior).
  - `README.md` links to `docs/logging.md` under a "Debugging" section.
- Files:
  - Create: docs/logging.md
  - Modify: README.md

## External Response
Coordinator phase — no worker dispatch. Wrote `docs/logging.md` after verifying env-var and config-key names against `src-tauri/src/log_init/`, `src-tauri/src/store/model.rs`, and `src/lib/logger.ts`. Added a "Debugging" section to `README.md` linking to the doc.

## Review
- Spec Status: PASS
- Quality Findings: No findings (docs-only).
- Final Status: PASS
- Explanation: All referenced env vars and config keys verified against implementation:
  - `RUST_LOG`, `OPENMACRO_LOG_VERBOSE`, `OPENMACRO_LOG_DIR` — `src-tauri/src/log_init/{mod.rs,redact.rs}`.
  - `VITE_LOG_LEVEL`, `VITE_LOG_VERBOSE` — `src/lib/logger.ts`.
  - YAML `logging.{level, modules, file.{enabled, max_files}, verbose_content, frontend.{level, modules}}` — `src-tauri/src/store/model.rs`.
  - `/logs` hotkey `Ctrl+Shift+L` / `⌘+Shift+L` — `src/main.tsx` HotkeyHandler.
  - No mention of unimplemented features (no runtime UI level control; no "include sensitive content" save checkbox — frontend redacts on ingest per phase-04 spec deviation).
- Next: done — plan complete, status DONE.

## Squash Commit
- phase-5: logging docs

## Decisions
- Documented the phase-04 spec deviation: Save action has no "include sensitive content" toggle because frontend redaction happens on ingest. Rust redaction is still controlled by `OPENMACRO_LOG_VERBOSE` / `verbose_content`.

## Handoff
Plan complete. No further phases.
