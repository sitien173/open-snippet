# Phase 1 — Unicode-by-default injection, batched SendInput, updated tests

Project root: `F:/projects_new/textblaze`
Plan dir: `F:/projects_new/textblaze/docs/plans/2026-06-10-injection-smoothness`
Worker contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`
ERP format: `F:/projects_new/textblaze/.agents/shared/erp.md`

Read both shared files before starting. The worker contract is binding
(per-task commit `phase-1.task-<M>: <one-line>`, `notes.md` block per task,
journal External Response append, completion line).

## Context (read before coding)

User wants snippet expansion to feel like EVKey
(https://github.com/lamquangminh/EVKey): no visible clipboard flash, no
perceptible delay before the replacement appears, no per-character timing
artefacts.

Current pipeline:

- `src-tauri/src/inject/mod.rs`:
  - `PRE_INJECT_DELAY = 20 ms` (hard sleep before first backspace)
  - `POST_BACKSPACE_DELAY = 12 ms` (hard sleep between backspaces and replacement)
  - `UNICODE_DIRECT_THRESHOLD_CHARS = 24` — anything longer goes through
    clipboard paste (Ctrl+V chord), which is the main source of perceived
    "non-smoothness".
- `src-tauri/src/inject/sendinput.rs`:
  - `PASTE_STEP_DELAY = 10 ms` between each Ctrl/V keydown/up (≈40 ms per paste chord)
  - Each unicode character is sent via its own `SendInput` call (one `INPUT` pair at a time)

Three tests in this area currently fail on `master` (the failures predate
the hook-layout phase — confirmed by running them against parent commit
`266e5e4`). They assert the old `Paste`/timing behaviour and will need to
be rewritten to match the new unicode flow:

- `engine::orchestrator::tests::resolved_round_trip_for_now_and_log_snippets`
- `engine::orchestrator::tests::shell_snippet_injects_backend_output_when_consent_enabled`
- `inject::tests::injector_sends_backspace_then_text_with_mocked_sink`

## Design constants for this phase

| Constant | Old | New |
| --- | --- | --- |
| `PRE_INJECT_DELAY` | 20 ms | **5 ms** |
| `POST_BACKSPACE_DELAY` | 12 ms | **0 ms** (remove the sleep) |
| `PASTE_STEP_DELAY` | 10 ms | **removed** |
| `UNICODE_DIRECT_THRESHOLD_CHARS` | 24 | **2048** |
| Unicode SendInput batching | one INPUT per call | **single SendInput per replacement, chunked to ≤256 INPUT entries per call** |

Clipboard fallback still runs when `plan.text.len() > plan.max_clipboard_bytes`
(unchanged), which in practice means replacements > 4 KB. Inside the
paste path, the four key events (Ctrl down, V down, V up, Ctrl up) must
be sent as a **single** `SendInput` batch — no per-event sleeps.

## Tasks

### Task 1 — Constants + unicode-by-default threshold

**Test-first.** In `src-tauri/src/inject/mod.rs`, add a new failing test
that constructs an `InjectPlan` whose `text` is exactly 500 ASCII
characters and asserts that the `MockSink` records 500 `Unicode(...)`
actions (no `Paste`). Run it, watch it fail (RED), paste the RED line
into `notes.md` Task 1 evidence.

Then change `UNICODE_DIRECT_THRESHOLD_CHARS` to `2048` and reduce
`PRE_INJECT_DELAY` to `Duration::from_millis(5)` and `POST_BACKSPACE_DELAY`
to `Duration::from_millis(0)` (or remove the corresponding `thread::sleep`
call when the duration is zero — keep the constant if it makes the code
clearer, just don't sleep on 0). The new test goes GREEN.

Update `injector_sends_backspace_then_text_with_mocked_sink` so it now
asserts the unicode flow for the short `"ok"` replacement
(`Backspace, Backspace, Backspace, Unicode('o'), Unicode('k'),
LeftArrow, LeftArrow`). Run the file's tests, confirm green.

Commit: `phase-1.task-1: unicode-by-default injection with shorter sleeps`

### Task 2 — Batched SendInput for unicode + paste chord

In `src-tauri/src/inject/sendinput.rs`:

- Delete `PASTE_STEP_DELAY` and the `send_input` (single-with-sleep)
  helper. `send_inputs` already batches.
- Rewrite the `KeyboardAction::Paste(_)` branch so the four key events
  (Ctrl down, V down, V up, Ctrl up) are constructed in one `[INPUT; 4]`
  array and submitted via a single `send_inputs` call. No sleeps.

In `src-tauri/src/inject/mod.rs`, the unicode injection path currently
loops `self.sink.send(KeyboardAction::Unicode(ch))` per character. To
batch this efficiently while keeping the `KeyboardSink` trait honest,
add a new trait method:

```rust
pub trait KeyboardSink: Send + 'static {
    fn send(&mut self, action: KeyboardAction);
    fn send_batch(&mut self, actions: &[KeyboardAction]) {
        for action in actions {
            self.send(action.clone());
        }
    }
}
```

…and override `send_batch` in `WindowsKeyboardSink` to build a single
`Vec<INPUT>` (chunked to ≤256 entries per `SendInput` call) covering
backspaces + unicode codepoints in one pass. The default `send_batch`
impl preserves behaviour for the mock sink so the test assertions
stay simple.

The `Injector::inject` flow then becomes (roughly):

1. `SUSPEND.store(true)`.
2. `thread::sleep(PRE_INJECT_DELAY)` (5 ms).
3. Build a `Vec<KeyboardAction>` containing N backspaces + the text path
   (`Unicode(ch)` for each char) when the text fits the unicode threshold,
   or N backspaces + a single `Paste(text)` action otherwise.
4. `self.sink.send_batch(&actions)`.
5. Send `LeftArrow * caret_left` as a batch.
6. `SUSPEND.store(false)`.

No `POST_BACKSPACE_DELAY` sleep.

Update the two orchestrator tests
(`resolved_round_trip_for_now_and_log_snippets`,
`shell_snippet_injects_backend_output_when_consent_enabled`) so their
`assert_eq!(actions, ...)` matches the new unicode flow. The text
`"copied tail"` is 11 chars → unicode path; `"hello"` from the shell
snippet is 5 chars → unicode path; the `{{date:%Y}}` resolved year
is 4 chars → unicode path. Expected action sequences accordingly.

Commit: `phase-1.task-2: batch sendinput, remove per-step paste delays`

### Task 3 — Verification

From `F:/projects_new/textblaze`:

```
rtk cargo test -p openmacro --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml
rtk cargo clippy --manifest-path F:/projects_new/textblaze/src-tauri/Cargo.toml --all-targets -- -D warnings
```

Both must be **green** with fresh output captured this turn. Paste the
trailing summary lines into `notes.md` Task 3 evidence.

Commit only if there are file changes (lockfile etc.); otherwise note
"verification only, no files touched" in `notes.md` Task 3.

## Constraints

- Do not touch `src-tauri/src/hook/**`.
- Do not change the `Snippet` / `Prefs` schemas. No new user-facing settings.
- Keep the existing `MockClipboardBackend` / `TestClipboardBackend` types
  intact; tests still use them.
- Absolute paths in any user-facing report.

## Done When (mirrors PLAN.md)

- Constants updated (thresholds + sleeps).
- Unicode injection is batched into one `SendInput` per replacement
  (chunked to ≤256 entries per call).
- Paste chord goes through a single `SendInput`.
- 3 stale tests rewritten to assert the new flow; new 500-char unicode
  test added.
- `cargo test` and `cargo clippy --all-targets -- -D warnings` are fully
  green with fresh evidence.
- Per-task commits, per-task notes, journal External Response present.

If anything is unclear before coding, list questions under CLARIFICATIONS
NEEDED and stop.
