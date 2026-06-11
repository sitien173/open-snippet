# Design — Manual Trigger, Custom Prefixes, and Trigger-Bug Fixes

**Date:** 2026-06-12
**Status:** Confirmed design, ready for plan
**Scope:** `src-tauri/src/matcher/`, `src-tauri/src/store/`, `src-tauri/src/hook/`, settings UI in `src/routes/`

## Summary

Three related changes to openmacro's trigger pipeline:

1. **New feature** — manual confirm-to-expand mode (Tab/Enter), set as default.
2. **Refactor** — global configurable trigger prefix with per-snippet literal override.
3. **Bug fix** — investigate and resolve delay, missed-trigger after multiple Enters, double expansion, and app-specific injection failures.

Phases execute in the order: bugs first → prefix refactor → manual feature.

---

## 1. Manual trigger (new feature, default)

### Behavior

Two-step expansion:

1. User types a trigger (`;hello`) at a valid boundary. Matcher detects it but does **not** expand. Transitions to **armed** state holding `{ snippet_id, trigger_len_chars }`.
2. Next keystroke decides:
   - **Tab or Enter** → expand. Confirm key is consumed (swallowed). Trigger characters are deleted; snippet body injected as today.
   - **Any other key** (letter, space, backspace, arrow, click/focus change, IME composition start) → disarm silently. No expansion fires. User can re-type to re-arm.
   - Modifier-only key events do not disarm.

### State

Add an `ArmedTrigger { snippet_id: Arc<str>, trigger_len_chars: usize }` to matcher/hook state (exact owner decided during plan). All existing `Reset::*` reasons disarm.

### Settings

- New global setting: `expand_mode: "manual" | "auto"`.
- Default for new installs: `"manual"`.
- `"auto"` preserves today's boundary-triggered behavior; the armed-state code path is bypassed.
- No per-snippet override in v1.

### Confirm keys

Hardcoded Tab + Enter, both consumed. No newline preservation. No per-snippet/per-app override in v1.

### Edge cases

- Overlapping triggers: when `;hello` arms, typing `t` toward `;hellothere` disarms and the user must re-type. Standard wait-and-confirm semantics.
- Boundary rules in front of the trigger remain unchanged.

---

## 2. Custom prefixes (refactor)

### Decision

Global prefix setting with per-snippet literal override.

### Schema

- Global: `trigger_prefix: ";"` (default `";"` — backwards compatible).
- Per-snippet: optional `trigger_literal: true`. When set, `trigger` is used verbatim and the prefix is not prepended.

### Loader logic (`store/loader.rs`)

```
effective_trigger =
    if trigger_literal           → trigger
    else if trigger.starts_with(prefix) → trigger     // auto-detect, no migration
    else                         → prefix + trigger
```

Aho-Corasick is built from `effective_trigger`. Matcher code in `matcher/automaton.rs` is unchanged.

### Validation

- Prefix length 1–3, all punctuation/symbol class (no letters/digits/whitespace). Reject with clear error.
- Duplicate effective triggers after normalization → load error referencing the effective trigger.

### UI (`src/routes/`)

- Settings: prefix input with validation + live preview (`":" + "email" → ":email"`).
- Snippet editor: show effective trigger as read-only hint when `trigger_literal` is off.

### Hot reload

Existing `store/watcher` rebuild path handles it. No restart.

---

## 3. Trigger bugs — investigation plan

This is investigation, not a pre-committed fix. Hypotheses ranked by likelihood:

### A. Enter resets matcher state incorrectly (no trigger after several Enters)

Audit `hook/thread.rs`, `hook/winevent.rs` for VK_RETURN handling. Verify Enter is dispatched as a normal whitespace char, not `Reset::*`. Confirm `leading_boundary_state` after Enter still allows `boundary_allows_match`.

**Diagnostic:** trace logs around `MatchBuffer::push_char` / `reset_with`, DEBUG level.

### B. Re-entrant hook callbacks during injection (delay / double expansion)

Audit `LLKHF_INJECTED` / `LLMHF_INJECTED` filtering. If our synthesized keys re-enter the matcher, double expansion follows.

**Diagnostic:** log every matcher input with an "injected?" flag.

### C. Backpressure on the hook ring (late / missed)

`RING_CAPACITY` may drop events during heavy Enter mashing.

**Diagnostic:** add dropped-event counter in `hook/ring.rs`; expose via `/logs`.

### D. App-specific injection timing (works in Notepad, not in browser/chat)

Likely needs targeted paste fallback or inter-key delay per known HWND class.

**Diagnostic:** capture failing HWND class names; reproduce with `pnpm tauri dev`.

### Deliverable

`docs/plans/2026-06-12-trigger-bugs/findings.md` with confirmed hypotheses and a minimal fix scope. **No speculative fixes.**

---

## 4. Rollout, migration, tests

### Phase order

1. Bug investigation + fixes.
2. Custom prefix refactor.
3. Manual trigger feature.

### Migration

First launch after upgrade: missing `expand_mode` → write `"manual"` and surface a one-time notice. Users who want auto-expand flip the toggle in Settings.

### Tests

**Rust unit (`matcher/`):**
- armed-state transitions (arm on match, disarm per key class, disarm per `Reset::*`)
- Tab and Enter both confirm
- prefix normalization at load

**Rust integration (`src-tauri/tests/`):**
- `matcher_manual_mode.rs`
- `loader_prefix.rs`
- `matcher_enter_burst.rs` (regression for hypothesis A)

**Vitest:** settings UI for prefix and expand_mode, validation errors.

**Manual:** `pnpm tauri dev` against Notepad + a browser + a chat app each phase.

### Non-goals

- Per-snippet `expand_mode` override
- Visual inline hint overlay
- Multiple simultaneous prefixes
- macOS/Linux

---

## Open questions for the plan phase

- Settings storage location: extend existing config file, or add `snippets/_settings.yaml`?
- Owner of `ArmedTrigger` state: matcher vs. hook thread.
- Whether the bug-fix phase produces one PR or splits per confirmed hypothesis.
