# Injection Smoothness — 2026-06-10

## Background

Issue 2 from the user: "the snippet of this app should be smooth like EVKey
macro does" (https://github.com/lamquangminh/EVKey). EVKey-class IMEs feel
smooth because they inject characters as `KEYEVENTF_UNICODE` directly via
`SendInput` in a single batch, with no clipboard round-trip and minimal
fixed sleeps.

Current pipeline (`src-tauri/src/inject/mod.rs`, `src-tauri/src/inject/sendinput.rs`):

- `PRE_INJECT_DELAY = 20 ms` (hard wait before any backspace)
- `POST_BACKSPACE_DELAY = 12 ms`
- `PASTE_STEP_DELAY = 10 ms` between each Ctrl/V key event (≈40 ms per paste chord)
- `UNICODE_DIRECT_THRESHOLD_CHARS = 24` — anything longer goes through
  clipboard paste, which causes the visible clipboard "flash" and an
  additional Ctrl+V keystroke that some apps render as a separate event
- Each unicode char is sent as a separate `SendInput` call instead of one batch

This plan switches to **unicode by default**, keeping clipboard fallback only
for very large replacements (>4 KB of text), and trims the inter-step sleeps
so behaviour matches EVKey's perceived smoothness.

## Out of scope

- Hook layout / translation changes — covered by `2026-06-10-hook-layout-fix`.
- Foreground tracking (`apply_foreground_change` still has no production
  caller) — tracked separately.
- New user-facing prefs for delays or thresholds — defaults only; we can
  expose knobs later if real apps need tuning.

## Phase 1 — Unicode-by-default injection, batched SendInput, updated tests

Owner: Codex. See `phase-01/prompt.md`.

Done When:
- `UNICODE_DIRECT_THRESHOLD_CHARS` raised so that everything ≤ ~2048 chars
  takes the unicode path. The clipboard branch only runs when the rendered
  text is strictly larger than the configured byte cap (≥ ~4 KB).
- `PRE_INJECT_DELAY` is reduced to **5 ms** (a single short settle, enough
  for the foreground app to commit the trigger char before backspaces fire).
- `POST_BACKSPACE_DELAY` is removed (set to 0); backspaces and the unicode
  payload go in the same SendInput batch where possible so the OS delivers
  them as one queue write.
- `PASTE_STEP_DELAY` is removed entirely from `sendinput.rs`. The paste
  branch still works correctly for the >4 KB case; the key events are
  delivered as a single SendInput batch instead of four separate calls
  with sleeps.
- Unicode injection batches all `KEYEVENTF_UNICODE` `INPUT` events for a
  given replacement into one `SendInput` call (subject to a sane chunk
  size, e.g. 256 events per batch, to keep stack arrays bounded). One
  batch per call instead of `chars().for_each(SendInput-of-one)`.
- The three pre-existing failing tests in `src-tauri/src/engine/orchestrator.rs`
  (`resolved_round_trip_for_now_and_log_snippets`,
  `shell_snippet_injects_backend_output_when_consent_enabled`) and
  `src-tauri/src/inject/mod.rs` (`injector_sends_backspace_then_text_with_mocked_sink`)
  are updated to assert the new unicode flow, then pass.
- A new unit test in `src-tauri/src/inject/mod.rs` covers a ~500-char
  replacement going through the unicode path (not Paste) with the new
  threshold.
- `cargo test -p openmacro --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml`
  is **fully green** (no pre-existing failures).
- `cargo clippy --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml --all-targets -- -D warnings`
  clean.
- Per-task commits, per-task notes.md blocks, journal External Response,
  completion line — per worker contract.
