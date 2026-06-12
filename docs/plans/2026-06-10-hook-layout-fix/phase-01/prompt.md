# Phase 1 — Layout-aware translate_key + TRACE diagnostics

Project root: `F:/projects_new/textblaze`
Plan dir: `F:/projects_new/textblaze/docs/plans/2026-06-10-hook-layout-fix`
Worker contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`
ERP format: `F:/projects_new/textblaze/.agents/shared/erp.md`

Read both shared files before starting. The worker contract is binding (per-task commit, notes.md per task, journal External Response append, completion line).

## Context (read before coding)

`src-tauri/src/hook/thread.rs` installs a `WH_KEYBOARD_LL` hook. Inside the OS callback, `translate_key(vk_code, scan_code)` calls `ToUnicodeEx` with `GetKeyboardLayout(0)`. Two defects:

1. **Wrong layout.** `GetKeyboardLayout(0)` returns the **hook thread's** active layout, not the foreground window's input thread layout. Apps running on threads with a different active HKL (common on multi-language Windows installs) get keystrokes silently dropped because `ToUnicodeEx` returns `0`. This is the user's reported bug: triggers fire in terminals but not in Notepad / Chrome address bar / VS Code editor.
2. **Dead-key kernel-state mutation.** With `wFlags = 0`, a successful `ToUnicodeEx` call mutates the kernel dead-key buffer, which interferes with composition state in apps that maintain their own (Chromium-based, Electron). Passing `wFlags = 2` makes the call read-only with respect to kernel state.

Background context for "smoothness" (out of scope here, but informs how invasive to be): we want this hook to behave like EVKey — a global low-level hook that observes typing without disturbing the target app.

## Tasks

### Task 1 — Extract a testable translate helper

Refactor `translate_key` so the OS-touching shim stays in `thread.rs` and a **pure** function lives somewhere unit-testable. Suggested signature (adjust to fit the codebase):

```rust
pub(super) enum TranslateOutcome {
    Char(char),
    DeadKey,
    None,
}

pub(super) fn translate_with_layout(
    vk_code: u32,
    scan_code: u32,
    hkl: windows::Win32::UI::TextServices::HKL,
    key_state: &[u8; 256],
) -> TranslateOutcome;
```

`translate_with_layout` calls `ToUnicodeEx` with `wFlags = 2` (no kernel-state mutation) and returns:
- `Char(c)` when `ToUnicodeEx` returns `1`.
- `DeadKey` when it returns `-1`.
- `None` for `0` or any unexpected value.

The OS shim in `thread.rs::translate_key` (or a renamed `translate_for_foreground`) is responsible for:
- Snapshotting `GetKeyboardState` + per-modifier `GetKeyState` overrides (as today).
- Resolving the active HKL:
  ```text
  let hwnd = GetForegroundWindow();
  if hwnd.is_invalid() { hkl = GetKeyboardLayout(0); }
  else {
      let tid = GetWindowThreadProcessId(hwnd, None);
      hkl = if tid == 0 { GetKeyboardLayout(0) } else { GetKeyboardLayout(tid) };
  }
  ```
- Calling `translate_with_layout` and mapping its outcome to the existing `Option<char>` return.

Test-first: before writing the extraction, add unit tests for `translate_with_layout` covering:
- `VK_A` no modifiers → `Char('a')` (use the en-US HKL fetched via `LoadKeyboardLayoutW("00000409", KLF_NOTELLSHELL)` or `GetKeyboardLayout(0)`; if the test thread can't guarantee a layout, gate the test with `#[cfg(windows)]` and skip when the layout doesn't have ASCII letters).
- `VK_A` with Shift bit set in `key_state` → `Char('A')`.
- A `hkl` of `HKL(0)` returns `None` without panicking.

Test-evidence requirement in notes.md: paste the RED→GREEN `cargo test` output for the new tests.

Commit: `phase-1.task-1: extract translate_with_layout helper`

### Task 2 — Foreground-aware translate_key with TRACE logging

In `thread.rs`, update `translate_key` (or its renamed equivalent) to:
- Resolve HKL from the foreground window's input thread as described above.
- Emit a `tracing::trace!` event **inside the OS callback path** with fields: `vk_code` (decimal), `scan_code` (decimal), `hkl` (lowercase hex, e.g. `0x04090409`), and `outcome` (`"char"` / `"dead"` / `"none"`). For `Char`, also include the codepoint in `\u{XXXX}` form (do not log raw char content at INFO/DEBUG — TRACE only, scoped to this site). Keep the event short; this runs in the LL hook callback which has a ~300 ms timeout.
- Continue to call `CallNextHookEx` exactly as before — no change to control flow other than the new HKL source and the new wFlags.

No new test needed for this task beyond the helper's tests; the OS shim is exercised by integration via the helper. Commit: `phase-1.task-2: use foreground-thread layout for key translation`

### Task 3 — Verification

Run from `F:/projects_new/textblaze`:

```
cargo test -p <the src-tauri crate name from src-tauri/Cargo.toml>
cargo clippy --all-targets -- -D warnings
```

Both must pass with **fresh** output captured this turn. If `cargo test` cannot run on this machine (no Windows toolchain), say so under SPEC COMPLIANCE — do not fake the evidence.

Commit: `phase-1.task-3: verify cargo test + clippy green` only if there are file changes (lockfile, etc.); otherwise skip the commit and note "no files touched, verification only" in notes.md Task 3.

## Constraints

- Do not touch `src-tauri/src/inject/**` in this phase. Smoothness is a separate phase.
- Do not add real foreground tracking (`apply_foreground_change` integration). Separate phase.
- Keep TRACE logging strictly off by default; do not flip log levels in `log_init`.
- Absolute paths only when referring to files in your response.

## Done When (mirrors PLAN.md)

- `translate_with_layout` exists and is unit-tested.
- `translate_key` uses foreground-thread HKL with fallback, `wFlags = 2`, and TRACE logging.
- `cargo test` and `cargo clippy --all-targets -- -D warnings` are green with fresh evidence.
- Per-task commits, notes.md task blocks, journal External Response present.

If any of the above is unclear before coding, list questions under CLARIFICATIONS NEEDED and stop.
