# Plan — Manual Trigger, Custom Prefixes, and Trigger-Bug Fixes

**Design:** `docs/plans/2026-06-12-manual-trigger-and-fixes-design.md`
**Created:** 2026-06-12
**Status:** Ready to execute

---

## Phase 1: Trigger bug investigation and minimal fixes

**Owner:** `codex`

**Goal:** Identify which of the four hypotheses (Enter-reset, re-entrant injection, ring backpressure, app-specific timing) are real, then apply the smallest fix per confirmed hypothesis. No speculative changes.

**Files:**
- Modify: `src-tauri/src/hook/thread.rs`
- Modify: `src-tauri/src/hook/winevent.rs`
- Modify: `src-tauri/src/hook/ring.rs`
- Modify: `src-tauri/src/matcher/buffer.rs` (only if hypothesis A confirmed)
- Modify: `src-tauri/src/inject/` (only if hypothesis B or D confirmed)
- Create: `docs/plans/2026-06-12-manual-trigger-and-fixes/findings.md`
- Create: `src-tauri/tests/matcher_enter_burst.rs`

**Tasks:**
1. Add diagnostic logging at DEBUG around `MatchBuffer::push_char` / `reset_with`, hook injected-event flag, ring drop counter. Reproduce each symptom in `pnpm tauri dev` against Notepad + a browser + a chat app. Capture findings in `findings.md` with confirmed/rejected status per hypothesis.
2. For each **confirmed** hypothesis, write a regression test first (TDD), then the minimal fix. Write `matcher_enter_burst.rs` for hypothesis A regardless of confirmation status.
3. Remove temporary diagnostic logs that don't earn their keep; keep the ring drop counter exposed via `/logs`.

**Acceptance Criteria:**
- `findings.md` exists with verdict per hypothesis (A, B, C, D).
- All confirmed hypotheses have a regression test + fix; rejected ones are documented as rejected with evidence.
- `matcher_enter_burst.rs` passes (typing a trigger after 10 rapid Enters expands correctly).
- No regression in existing snippet expansion behavior.

**Reviewer Checklist:**
- Findings memo cites concrete evidence (log excerpts, HWND class names) per hypothesis.
- No fix is applied for an unconfirmed hypothesis.
- Tests fail before fix, pass after (verify by reverting fix and re-running).
- `LLKHF_INJECTED` filtering, if touched, is verified to drop only synthesized events from this app's injector.

**Integration Checks:**
- `cd src-tauri && cargo test`
- `cd src-tauri && cargo fmt --check`
- `pnpm tauri dev` smoke test: 5 triggers in Notepad after Enter-mashing, 5 in a browser, 5 in a chat app.

---

## Phase 2: Custom prefix refactor (backend)

**Owner:** `codex`

**Goal:** Add a configurable global trigger prefix with per-snippet literal override, applied at snippet load time. Matcher logic unchanged.

**Files:**
- Modify: `src-tauri/src/store/model.rs` (add `trigger_literal` field to snippet schema; add `trigger_prefix` to settings model)
- Modify: `src-tauri/src/store/loader.rs` (effective-trigger normalization, duplicate detection, validation)
- Modify: `src-tauri/src/store/watcher.rs` (rebuild on prefix change)
- Create: `src-tauri/tests/loader_prefix.rs`

**Tasks:**
1. Extend settings model with `trigger_prefix: String` (default `";"`) and validate (length 1–3, punctuation/symbol class only, no whitespace/letter/digit). Add `trigger_literal: bool` to `Snippet`.
2. In loader, compute `effective_trigger` per the design's auto-detect rule (don't double-prepend if trigger already starts with the prefix). Reject duplicate effective triggers with an error message that names the effective trigger.
3. Wire prefix change through `watcher` so the matcher rebuilds. Add unit tests in `loader_prefix.rs` covering: default prefix, custom prefix, literal override, auto-detect, validation errors, duplicate detection.

**Acceptance Criteria:**
- Existing snippets with literal `;foo` triggers continue to work without edits.
- Changing `trigger_prefix` to `:` while a snippet has bare `trigger: email` produces effective trigger `:email`.
- Invalid prefixes are rejected at load with a clear error.
- `loader_prefix.rs` covers all listed cases.

**Reviewer Checklist:**
- Auto-detect rule does not double-prepend.
- Duplicate-effective-trigger error message references the effective trigger, not the raw `trigger` field.
- No matcher (`automaton.rs`) code touched.
- Hot reload path tested (modify settings file, observe rebuild log).

**Integration Checks:**
- `cd src-tauri && cargo test loader_prefix`
- `cd src-tauri && cargo test`
- `cd src-tauri && cargo fmt --check`

---

## Phase 3: Custom prefix refactor (frontend)

**Owner:** `gemini`

**Goal:** Settings UI for the prefix; snippet editor hint showing effective trigger.

**Files:**
- Modify: `src/routes/` — settings route (path TBD; locate during execute)
- Modify: snippet editor component (locate during execute; likely `src/routes/.../SnippetEditor.tsx`)
- Modify: `src/lib/` Tauri command bindings if needed
- Create: nearby `__tests__/` Vitest files

**Tasks:**
1. Add "Trigger prefix" input in Settings with live validation and a preview line (`"email" → ":email"`). Surface backend validation errors inline.
2. In the snippet editor, render the effective trigger as a read-only hint when `trigger_literal` is off; expose a toggle for `trigger_literal`.
3. Add Vitest coverage for: validation errors, preview text, literal toggle behavior.

**Acceptance Criteria:**
- Changing the prefix in Settings persists and updates the editor's preview without restart.
- Invalid prefix shows the backend's error message inline; save is blocked.
- Snippet editor toggle for `trigger_literal` hides the preview and uses the raw trigger.

**Reviewer Checklist:**
- No business-logic duplication in frontend (validation rules live in backend; UI only displays errors).
- Vitest covers a11y (`*.a11y.test.tsx` where applicable).
- Existing snippet edit flow not regressed.

**Integration Checks:**
- `pnpm test`
- `pnpm lint`
- `pnpm tauri dev` — manual: change prefix, edit a snippet, verify hint updates.

---

## Phase 4: Manual trigger feature (backend)

**Owner:** `codex`

**Goal:** Implement armed-state expansion: on full trigger match, hold; on next keystroke, expand if Tab/Enter (consumed) or disarm.

**Files:**
- Modify: `src-tauri/src/matcher/automaton.rs` (return armed-hit or expand-hit per mode)
- Modify: `src-tauri/src/matcher/buffer.rs` if state lives here, or `src-tauri/src/hook/thread.rs` if state lives in hook (decide during execute; document choice in code comment)
- Modify: `src-tauri/src/store/model.rs` (add `expand_mode: ExpandMode` enum with `Manual` default to settings)
- Modify: `src-tauri/src/commands/` settings IPC
- Modify: `src-tauri/src/hook/thread.rs` — intercept Tab/Enter when armed and consume them
- Create: `src-tauri/tests/matcher_manual_mode.rs`
- Create: `src-tauri/tests/matcher_armed_dismiss.rs`

**Tasks:**
1. Add `ExpandMode { Auto, Manual }` to settings with default `Manual`. Plumb to matcher/hook layer.
2. Implement armed state: on match in Manual mode, store `ArmedTrigger`; do not expand. On next char, dispatch by class: Tab/Enter → expand + consume; any other key class (letter/space/backspace/arrow) or `Reset::*` → disarm. Modifier-only events are ignored.
3. Auto mode bypasses the armed path entirely — behavior identical to today.
4. Write tests for: arm-on-match, all disarm classes, Tab confirms, Enter confirms, both confirm keys consumed, Auto mode unchanged.

**Acceptance Criteria:**
- `matcher_manual_mode.rs` and `matcher_armed_dismiss.rs` pass.
- Manual mode: typing `;hello` then Tab expands; typing `;hello` then `x` does not expand and resumes scanning.
- Auto mode: existing tests still pass; no behavior change.
- Tab/Enter are not delivered to the foreground app when they confirm an expansion.

**Reviewer Checklist:**
- Armed state cleared on every `Reset::*` reason listed in `matcher/buffer.rs`.
- IME composition start disarms.
- Modifier-only key events do not disarm.
- Auto mode code path is provably unchanged (diff review).
- No re-entrant expansion (the injected snippet body doesn't re-trigger armed state).

**Integration Checks:**
- `cd src-tauri && cargo test matcher_manual_mode matcher_armed_dismiss`
- `cd src-tauri && cargo test`
- `cd src-tauri && cargo fmt --check`
- `pnpm tauri dev` — manual: trigger + Tab, trigger + Enter, trigger + other key, trigger + arrow.

---

## Phase 5: Manual trigger feature (frontend + migration)

**Owner:** `gemini`

**Goal:** Settings toggle for expand mode; first-launch migration notice for users upgrading from auto-only.

**Files:**
- Modify: `src/routes/` settings route
- Modify: `src/lib/` Tauri command bindings (settings get/set if needed)
- Create: one-time notice component (locate during execute)
- Create: Vitest coverage in nearby `__tests__/`

**Tasks:**
1. Add "Expand mode" radio (Manual / Auto) in Settings with helper text explaining Tab/Enter confirmation.
2. On first launch after upgrade (detect by absence of `expand_mode` in stored settings), surface a non-blocking toast/banner: "Snippets now expand on Tab/Enter. Change in Settings." Dismiss persists.
3. Vitest for: setting persistence, notice shows once and dismisses, radio reflects current value.

**Acceptance Criteria:**
- Fresh install defaults to Manual with no notice.
- Upgrade (settings file without `expand_mode`) shows notice once; subsequent launches don't.
- Switching to Auto persists and takes effect without restart.

**Reviewer Checklist:**
- Migration detection uses absence-of-field, not version comparison.
- Notice is dismissible and doesn't block input.
- No duplicate backend validation in UI.

**Integration Checks:**
- `pnpm test`
- `pnpm lint`
- `pnpm tauri dev` — manual: delete `expand_mode` from settings file, relaunch, observe notice; dismiss, relaunch, verify it doesn't return.

---

## Open questions (resolve during execution)

- Settings storage: extend existing config file vs. new `snippets/_settings.yaml`. Decide in Phase 2.
- Armed-state owner: matcher vs. hook thread. Decide in Phase 4 plan step.
