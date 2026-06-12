# Phase 01 Prompt - Investigate Snippet Not Working After Manual Trigger Plan

Project: F:/projects_new/textblaze

Follow these instructions:
- Read F:/projects_new/textblaze/.agents/shared/worker-contract.md and F:/projects_new/textblaze/.agents/shared/erp.md if present.
- Follow systematic debugging: no fix before root-cause evidence.
- If a fix is needed, follow TDD: add a focused failing regression test, confirm RED, apply the minimal production change, confirm GREEN.
- Use the repo instructions from F:/projects_new/textblaze/AGENTS.md as provided in the session: use `rtk` for commands and use `coderecall` / `tgrep`, not `rg` or grep.

User symptom:
- After completing plan `docs/plans/2026-06-12-manual-trigger-and-fixes`, snippets are not working correctly in `pnpm tauri dev`.
- Console log for typing a trigger shows pairs of typed chars and then injection events entering the hook:
  - `;`, `;`, `n`, `n`, `o`, `o`
  - on `w`: `queued hook event kind="char"`, then `queued hook event kind="backspace"`, then `translated key vk_code=231 ... outcome="none"`
  - another `w` shows the same pattern
  - Enter is translated/queued twice

Relevant prior-plan facts:
- Phase 1 added `src-tauri/src/inject/sendinput.rs::INJECTED_MARKER` and changed `src-tauri/src/hook/thread.rs::should_ignore_event` to ignore only injected events carrying that marker.
- Phase 4 made manual mode the default and added hook-side `CONFIRM_ARMED` state for Tab/Enter confirmation.
- Phase 4 follow-ups fixed stale hook armed-latch races, but GUI smoke was not run from the CLI environment.

Initial coordinator observations:
- `src-tauri/src/inject/sendinput.rs` tags SendInput backspace, Unicode, paste, and caret actions with `INJECTED_MARKER`.
- `src-tauri/src/hook/thread.rs::should_ignore_event` currently ignores only `(flags & LLKHF_INJECTED_FLAG) != 0 && dw_extra_info == INJECTED_MARKER`.
- The user's log showing queued backspace and `vk_code=231` immediately after expansion suggests app-injected input may still be reaching the hook/matcher boundary, or `dwExtraInfo`/flags are not what the filter expects at runtime.
- The apparent duplicate real chars need explanation, but do not assume that duplication alone is the root cause because expansion appears to start on `w`.

Scope:
- Primary files: F:/projects_new/textblaze/src-tauri/src/hook/thread.rs, F:/projects_new/textblaze/src-tauri/src/inject/sendinput.rs, F:/projects_new/textblaze/src-tauri/src/inject/mod.rs, F:/projects_new/textblaze/src-tauri/src/engine/orchestrator.rs.
- Tests likely near: F:/projects_new/textblaze/src-tauri/src/hook/thread.rs tests and/or F:/projects_new/textblaze/src-tauri/tests/matcher_manual_mode.rs.
- Do not change frontend files unless evidence proves the bug is frontend-side.

Tasks:
1. Identify the exact failing boundary with evidence. Prefer a focused unit/integration test for the hook decision/filtering behavior; if runtime-only evidence is needed, add minimal diagnostic logging that is safe to keep or remove it before final.
2. If code changes are needed, add a failing regression first and then the minimal fix. Keep changes surgical.
3. Run focused Rust tests and the relevant broader Rust check. Record RED/GREEN evidence.

Return format:
- Use the worker response format from the shared ERP contract if present.
- Include root-cause evidence, files modified, tests run with exit status, and any manual smoke limitation.
