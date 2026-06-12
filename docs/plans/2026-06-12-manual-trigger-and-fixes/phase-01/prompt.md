## Original User Request

Complete `docs/plans/2026-06-12-manual-trigger-and-fixes`, routing failing work to another worker when needed. The design requires bug investigation first, then prefix and manual-trigger phases.

## Phase

Phase 1: investigate trigger bugs and apply only minimal fixes for confirmed hypotheses.

Codex dispatch failed before implementation because the configured model was unsupported for this account. This phase is now rerouted to Gemini under the user's instruction to route failed work to another worker. Keep the backend/system scope unchanged.

## Tasks

- task-1: Add bounded diagnostics for hypotheses A-D, gather concrete evidence, and write `findings.md` with confirmed/rejected status for each hypothesis.
- task-2: For each confirmed hypothesis, write a failing regression test first, then the minimal fix. Create `matcher_enter_burst.rs` for hypothesis A regardless of confirmation status.
- task-3: Remove temporary diagnostics that do not earn their keep; keep a useful ring drop counter exposed through the existing logs path if hypothesis C or implementation evidence supports it.

## Context

Hypotheses from the confirmed design:

- A. Enter resets matcher state incorrectly, causing no trigger after several Enters. Audit `hook/thread.rs`, `hook/winevent.rs`, and `matcher/buffer.rs`.
- B. Re-entrant hook callbacks during injection cause delay or double expansion. Audit injected-event filtering such as `LLKHF_INJECTED` / `LLMHF_INJECTED`.
- C. Hook ring backpressure drops events during heavy Enter input. Audit `hook/ring.rs`.
- D. App-specific injection timing fails in browsers or chat apps. Audit `src-tauri/src/inject/`.

The plan explicitly says this is investigation first, not speculative fixing. Do not apply a fix unless the findings memo records evidence that confirms the relevant hypothesis.

## Files

- F:/projects_new/textblaze/src-tauri/src/hook/thread.rs
- F:/projects_new/textblaze/src-tauri/src/hook/winevent.rs
- F:/projects_new/textblaze/src-tauri/src/hook/ring.rs
- F:/projects_new/textblaze/src-tauri/src/matcher/buffer.rs
- F:/projects_new/textblaze/src-tauri/src/inject/
- F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/findings.md
- F:/projects_new/textblaze/src-tauri/tests/matcher_enter_burst.rs

## Done When

- `findings.md` exists and includes a concrete verdict for hypotheses A, B, C, and D.
- Every confirmed hypothesis has a regression test and minimal fix.
- Rejected hypotheses are documented as rejected with evidence.
- `matcher_enter_burst.rs` passes and verifies typing a trigger after 10 rapid Enter inputs still expands correctly.
- Existing snippet expansion behavior is not regressed.
- Fresh integration checks pass: `cd src-tauri && cargo test`; `cd src-tauri && cargo fmt --check`.
- If manual GUI smoke testing is not possible in the worker environment, document that limitation explicitly in `findings.md` and use the strongest available code/test evidence instead.

## Rules

Follow the contract in `F:/projects_new/textblaze/.agents/shared/worker-contract.md` - per-task workflow (test-first -> one commit per task `phase-1.task-<M>: ...` -> append a `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-01/notes.md` -> append the `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-manual-trigger-and-fixes/phase-01/journal.md`) plus the discipline rules (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear -> CLARIFICATIONS NEEDED + stop).

Use `coderecall` for semantic codebase search, `tgrep` for literal search, and never use `rg`, `grep`, or web search for codebase lookup in this repository. Prefix shell commands with `rtk`.

## Response Format

Respond per `F:/projects_new/textblaze/.agents/shared/erp.md` - return the `# EXTERNAL RESPONSE` block, then the single completion line. Use owner `gemini` in the response metadata.

## Same-phase fix

Reuse the cached `SESSION_ID`. Send `FIX:` plus only the delta files and delta context. The fix still gets its own task commit and, if it changes a decision, an appended `notes.md` block.
