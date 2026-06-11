# Hook Layout Fix — 2026-06-10

## Background

User reports snippets fire when typed into terminals (Windows Terminal, conhost) but **not** in Notepad, browser address/search boxes, or the VS Code editor. Reproduces even with EVKey/IMEs disabled.

Investigation (coordinator, 2026-06-10) found two defects in the WH_KEYBOARD_LL translation path at `src-tauri/src/hook/thread.rs:165-196` (`translate_key`):

1. `ToUnicodeEx(..., GetKeyboardLayout(0))` uses the **hook thread's** active layout, not the foreground window's input thread layout. On multi-language Windows installs the two HKLs can differ per-window, causing `ToUnicodeEx` to return `0` and silently drop the keystroke before it reaches the matcher.
2. `ToUnicodeEx` is called with `wFlags=0`, which mutates the kernel-side dead-key buffer on success. Apps that maintain their own composition state (Chromium-based browsers, VS Code, Electron apps) can have keys dropped or doubled as a result. Passing `wFlags=2` ("do not change kernel state") avoids the side effect — documented since Windows 10 1607.

There is no diagnostic logging in the hot path, so we are theorising. Phase adds TRACE-level logging that records `vkCode`, `scanCode`, the resolved HKL, and the translated codepoint so the user can confirm the diagnosis from a real log when they reproduce the bug in Notepad/Chrome/VS Code.

## Out of scope

- Injection smoothness (clipboard path, sleeps in `inject/mod.rs` and `sendinput.rs`). Tracked separately after this phase lands and the user confirms triggers fire correctly in all target apps.
- Adding real foreground tracking (`apply_foreground_change` currently has no production caller). Separate hygiene task.

## Phase 1 — Layout-aware translate_key + TRACE diagnostics

Owner: Codex. See `phase-01/prompt.md`.

Done When:
- `translate_key` resolves HKL from `GetForegroundWindow → GetWindowThreadProcessId → GetKeyboardLayout(tid)`. Fallback to `GetKeyboardLayout(0)` only when `tid == 0`.
- `ToUnicodeEx` is invoked with `wFlags = 2` (no kernel-state mutation).
- TRACE-level structured log at the call site records `vk_code`, `scan_code`, `hkl` (hex), and outcome (`char`, `dead`, `none`).
- Pure helper extracted (e.g. `translate_with_layout(vk, scan, hkl) -> TranslateOutcome`) so it can be unit-tested. The OS-touching shim stays in `thread.rs`.
- Unit tests cover: ASCII letter, ASCII letter with Shift, dead-key returns `None` without side-effect on caller's `key_state`, layout=0 returns `None` cleanly.
- `cargo test -p <crate>` and `cargo clippy --all-targets -- -D warnings` pass.
