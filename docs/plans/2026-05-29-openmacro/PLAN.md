# openmacro — Implementation Plan

- **Date:** 2026-05-29
- **Design:** [`../../2026-05-29-openmacro-design.md`](../../2026-05-29-openmacro-design.md)
- **PRD:** [`../../2026-05-29-openmacro-prd.md`](../../2026-05-29-openmacro-prd.md)
- **Status:** Ready to execute
- **Target:** Windows 10 (1809+) / 11, x64 + ARM64

## Conventions

- Repo root after fork: `F:/projects_new/textblaze/` (`openmacro/` workspace inside).
- Owners: `codex` = back-side Rust (`src-tauri/`). `gemini` = front-side React (`src/`). `coordinator` = scaffolding, wiring, docs.
- Per-phase artifacts land in `docs/plans/2026-05-29-openmacro/phase-NN/` (created lazily by the executor).
- All Rust work TDD-first per `test-driven-development`; bugs investigated per `systematic-debugging`.
- Every phase commits one-per-task during execution; coordinator squashes after Review PASS.

---

## Phase 1: Skeleton bootstrap

**Owner:** `coordinator`

**Goal:** A runnable Tauri 2 app bootstrapped from scratch (no upstream template fork), with tray icon, autostart toggle, single-instance lock, and an empty Settings webview. No expansion logic yet.

**Files:**
- Create: `openmacro/` (fresh scaffold via `pnpm create tauri-app` with React+TypeScript+Vite, or hand-rolled equivalent) — `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`, `vite.config.ts`, `src/main.tsx`, `src/routes/settings/index.tsx`.
- Modify: `src-tauri/src/lib.rs` (wire tray + single-instance), `src-tauri/tauri.conf.json` (app id `dev.openmacro`, product name, window defaults).
- Create: `src-tauri/src/engine/mod.rs`, `src-tauri/src/{store,matcher,hook,inject,expand,form,sync}/mod.rs` (empty modules with `//! TODO` headers, registered in `lib.rs`).

**Tasks:**
1. Bootstrap fresh Tauri 2 app at `openmacro/` (React + TS + Vite + pnpm); set product name `openmacro`, identifier `dev.openmacro`; verify `pnpm tauri dev` launches a window.
2. Add `tauri-plugin-autostart` + `tauri-plugin-single-instance`; named mutex `Global\openmacro-singleton`; second launch focuses existing tray.
3. Build tray icon + native menu (Pause/Resume placeholder, Reload snippets placeholder, Open snippets folder, Open settings, Quit). Left-click → settings window.
4. Create empty module skeleton under `src-tauri/src/` and register in `lib.rs` (compiles, no logic).

**Acceptance Criteria:**
- `pnpm tauri dev` opens a window and shows a tray icon.
- Second launch focuses first instance.
- `Quit` from tray exits cleanly.
- Autostart toggle (UI stub OK) writes `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value when enabled.

**Reviewer Checklist:**
- Single-instance mutex name matches design (`Global\openmacro-singleton`).
- No keyboard hook installed yet (verify with Process Explorer / search for `SetWindowsHookEx`).
- Tray icon survives webview hide.

**Integration Checks:**
- `cd openmacro && pnpm install`
- `cd openmacro && pnpm tauri build --debug`

---

## Phase 2: Snippet store + matcher

**Owner:** `codex`

**Goal:** YAML snippet store with file watcher and Aho-Corasick matcher operating on an in-memory rolling buffer. No OS hook, no injection — driven by unit tests.

**Files:**
- Create: `src-tauri/src/store/{mod.rs,loader.rs,watcher.rs,model.rs}`.
- Create: `src-tauri/src/matcher/{mod.rs,buffer.rs,automaton.rs,boundary.rs}`.
- Create: `src-tauri/tests/store_yaml.rs`, `src-tauri/tests/matcher_basic.rs`.
- Modify: `src-tauri/Cargo.toml` (add `serde`, `serde_yaml`, `aho-corasick`, `notify`, `tokio`).

**Tasks:**
1. Define `Snippet` model + YAML schema parser; per-file isolation (broken file logged + badged, others load). Handle `version: 1`, `trigger`, `replace`, `vars[]`.
2. Recursive loader for `%APPDATA%\openmacro\snippets\*.yaml`; debounced (200 ms) `notify` watcher rebuilds in-memory index on change.
3. `MatchBuffer` — 64-char rolling buffer; backspace pop; reset events (focus change hook stubbed, arrow/Home/End/PageUp/Down, IME, Caps).
4. Aho-Corasick automaton over all triggers; word-boundary rule (start / whitespace / punctuation); greedy = longest match wins.

**Acceptance Criteria:**
- `cargo test -p openmacro store_yaml` passes including malformed-file isolation case.
- `cargo test -p openmacro matcher_basic` passes covering: longest-match wins, word-boundary required, backspace pop, buffer reset on stub focus event.
- File watcher reload latency p95 < 250 ms in test.
- Module unit-line coverage ≥ 80 % (record output of `cargo llvm-cov --html`).

**Reviewer Checklist:**
- No I/O or allocation in matcher hot path beyond the AC automaton (no `Vec::push` in match resolution path).
- Loader never panics on bad YAML — returns `Result` + structured error.
- `version` mismatch surfaces a clear error, not a silent skip.

**Integration Checks:**
- `cd openmacro/src-tauri && cargo test --all-features`
- `cd openmacro/src-tauri && cargo clippy -- -D warnings`

---

## Phase 3: Keyboard hook + injection

**Owner:** `codex`

**Goal:** `WH_KEYBOARD_LL` thread, SPSC channel into matcher, clipboard-paste injector with `SendInput` fallback, pause hotkey, password-manager denylist, foreground change reset. End-to-end static expansion in Notepad.

**Files:**
- Create: `src-tauri/src/hook/{mod.rs,thread.rs,ring.rs,winevent.rs}`.
- Create: `src-tauri/src/inject/{mod.rs,clipboard.rs,sendinput.rs}`.
- Create: `src-tauri/src/engine/orchestrator.rs` (owns hook→match→inject pipeline).
- Create: `src-tauri/tests/notepad_smoke.rs` (gated `#[cfg(windows)]`, opt-in via `OPENMACRO_E2E=1`).
- Modify: `src-tauri/Cargo.toml` (add `windows` crate with `Win32_UI_Input_KeyboardAndMouse`, `Win32_System_DataExchange`, `Win32_UI_WindowsAndMessaging`, `Win32_UI_Accessibility`).

**Tasks:**
1. Dedicated OS thread: `SetWindowsHookEx(WH_KEYBOARD_LL)` + `GetMessage` pump. Callback translates `VK`+mods via `ToUnicodeEx`, pushes to lock-free SPSC ring (`crossbeam::queue::ArrayQueue` or `rtrb`). No alloc, no logging in callback. Detect IME / dead keys / Caps and signal reset.
2. `WinEventHook EVENT_SYSTEM_FOREGROUND` → buffer reset + denylist check (1Password / KeePass / Bitwarden / LastPass / `consent.exe`). While denylisted process is foreground, buffer never accumulates.
3. Injector: suspend matcher → `SendInput` Backspace × trigger len → clipboard paste (save all formats, set CF_UNICODETEXT, Ctrl+V, restore after 10 ms settle); fallback per-char `SendInput` if clipboard locked or payload > 4 KB.
4. Global pause hotkey `Ctrl+Alt+Pause` via `RegisterHotKey`; toggles matcher flag without unloading hook. Wire tray Pause/Resume to same flag. Max expansion length cap (32 KB default, configurable).

**Acceptance Criteria:**
- Unit tests for ring buffer (SPSC ordering, overflow drop) and denylist matcher pass.
- Smoke test (`OPENMACRO_E2E=1 cargo test notepad_smoke`) spawns Notepad, types `;sig\n`, asserts inserted text via UI Automation, restores clipboard.
- Pause hotkey toggles in < 50 ms; resume restores expansion without app restart.
- Foreground change to `consent.exe` clears buffer (logged) — verified by hook-thread unit test with mocked WinEvent.

**Reviewer Checklist:**
- Hook callback function does **no** allocation (`#[no_mangle]` audited; no `Box::new`, `format!`, `Vec::push`).
- Clipboard save/restore covers all formats present at trigger time, not just `CF_UNICODETEXT`.
- Pause flag is atomic (`AtomicBool`); no `Mutex` in keystroke hot path.
- `unsafe` blocks scoped narrowly; each has a one-line justification comment.

**Integration Checks:**
- `cd openmacro/src-tauri && cargo test`
- `cd openmacro/src-tauri && cargo clippy -- -D warnings`
- (Manual / CI Windows runner) `OPENMACRO_E2E=1 cargo test notepad_smoke`

---

## Phase 4: Placeholder resolvers + cursor token

**Owner:** `codex`

**Goal:** `{{date}}`, `{{time}}`, `{{clipboard}}`, `{{cursor}}` resolvers; `$|$` cursor positioning; full end-to-end static + variable expansion (form + shell deferred).

**Files:**
- Create: `src-tauri/src/expand/{mod.rs,resolver.rs,cursor.rs,datetime.rs,clipboard_var.rs}`.
- Create: `src-tauri/tests/expand_resolver.rs`, `src-tauri/tests/cursor_math.rs`.
- Modify: `src-tauri/src/engine/orchestrator.rs` (call resolver before inject; compute caret delta).

**Tasks:**
1. Placeholder resolver: parse `{{name}}` against snippet `vars`; unknown name → loud tray notification + skip injection. Support `datetime` with optional strftime `format:`.
2. `clipboard` var = current clipboard text at expansion time (empty if non-text). Distinct from injector's clipboard save/restore — read happens **before** save.
3. `$|$` cursor token: validate at parse time (0 or 1 occurrence; ≥2 → snippet flagged broken). Strip before injection; record (chars-after-token) for caret math.
4. Orchestrator: after paste settle, send `Left × chars_after_token` via `SendInput`. Handle case where placeholder expansion changes char count.

**Acceptance Criteria:**
- `cargo test expand_resolver cursor_math` covers all four var types, unknown-name failure, 2× `$|$` validation, and caret math under multi-byte UTF-8.
- Snippet `";now" → "It is {{date:%Y-%m-%d}} right now."` expands correctly in Notepad smoke test.
- Snippet `";log" → "console.log(\"$|$\");"` lands caret between quotes.

**Reviewer Checklist:**
- Caret math counts **UTF-16 code units** (Win32 `SendInput` semantics), not bytes or chars.
- `clipboard` var resolution happens before clipboard save, never sees injector's own scratch payload.
- Unknown placeholder fails closed (no partial injection).

**Integration Checks:**
- `cd openmacro/src-tauri && cargo test`
- `cd openmacro/src-tauri && cargo llvm-cov --summary-only` — confirm ≥ 80 % on `expand/`, `matcher/`, `store/`.

---

## Phase 5: Settings UI

**Owner:** `gemini`

**Goal:** Functional Settings webview — snippet list with broken-file badges, snippet editor (trigger + body + vars panel), pause toggle, autostart toggle, sync config panel (form only, agent comes Phase 8), reload-snippets button. Saves round-trip to YAML.

**Files:**
- Create: `src/routes/settings/{index.tsx,SnippetList.tsx,SnippetEditor.tsx,VarsPanel.tsx,SyncPanel.tsx,PrefsPanel.tsx}`.
- Create: `src/lib/snippets.ts` (typed wrappers over `tauri-specta` bindings).
- Modify: `src-tauri/src/commands/mod.rs` — expose `list_snippets`, `save_snippet`, `reload_snippets`, `get_prefs`, `set_prefs`.
- Create: `src/routes/settings/__tests__/SnippetEditor.test.tsx` (Vitest).

**Tasks:**
1. Snippet list view: groups by file, broken-file red badge with hover tooltip showing parse error from store.
2. Snippet editor: trigger input (validation: ≤32 chars, non-empty), Monaco-style body editor with `$|$` highlighter, vars panel (add/remove rows with type dropdown `text|textarea|choice|number|datetime|clipboard|cursor|shell|form`).
3. Prefs panel: pause toggle, autostart toggle, max expansion length, shell-consent display (read-only here; flip from confirm dialog only).
4. Save writes YAML to the snippet's source file via Rust command; file watcher reload event refreshes the list.

**Acceptance Criteria:**
- `pnpm test src/routes/settings` (Vitest) passes: editor validation, vars panel add/remove, save-then-reload reflects new trigger.
- Keyboard-navigable (Tab order audited); screen-reader labels on every form field (axe-core clean on `/settings`).
- WCAG AA contrast verified by axe-core; no critical violations.
- Pause toggle in UI flips backend pause flag within 100 ms (manual verify).

**Reviewer Checklist:**
- No business logic in components — all snippet mutation goes through `src/lib/snippets.ts`.
- Editor never silently drops `vars` entries on save (round-trip test).
- Trigger collision with existing snippet → blocking inline error, not toast.

**Integration Checks:**
- `cd openmacro && pnpm test`
- `cd openmacro && pnpm lint`
- `cd openmacro && pnpm tauri dev` — manual: edit `;sig`, save, expand in Notepad.

---

## Phase 6: Form runner (back-side)

**Owner:** `codex`

**Goal:** Form state machine in Rust: capture foreground HWND at trigger, open form window with snippet-id payload, await values, restore HWND, run injection. Cancel = clean state.

**Files:**
- Create: `src-tauri/src/form/{mod.rs,runner.rs,focus.rs}`.
- Modify: `src-tauri/src/engine/orchestrator.rs` (branch to form runner when any `type: form` var exists).
- Modify: `src-tauri/src/commands/mod.rs` — `form_submit(snippet_id, values)`, `form_cancel(snippet_id)`.
- Create: `src-tauri/tests/form_focus.rs`.

**Tasks:**
1. On trigger with form vars: pause matcher, capture `GetForegroundWindow()` HWND, **do not** send backspaces yet, open form window via `WindowBuilder` (decorations false, always on top, skip taskbar) at active monitor centre.
2. Form state machine: `Pending → Submitted(values) | Cancelled`. Submit reactivates captured HWND (`SetForegroundWindow` + `AllowSetForegroundWindow` workaround if needed), then runs standard backspace + resolve + inject.
3. Cancel path: no backspace, no injection, buffer cleared, literal trigger remains as user typed.
4. Multi-monitor: form window positions on monitor of captured HWND (via `MonitorFromWindow`).

**Acceptance Criteria:**
- `cargo test form_focus` validates HWND capture/restore against a spawned Notepad on each connected monitor.
- Submit path inserts values at correct placeholders; caret lands at `$|$`.
- Cancel path leaves the literal trigger characters in the focused window untouched.
- Form window survives focus-restore race (no orphaned topmost windows on cancel).

**Reviewer Checklist:**
- `AllowSetForegroundWindow`/`AttachThreadInput` dance handled; SetForegroundWindow failure logs + falls back to user-clicks-target.
- Form runner is fully `async` (Tokio), never blocks the orchestrator thread.
- Multi-form re-entrancy: a second trigger while a form is open is **rejected** (logged), not queued.

**Integration Checks:**
- `cd openmacro/src-tauri && cargo test`

---

## Phase 7: Form UI (front-side)

**Owner:** `gemini`

**Goal:** `/form/<snippet-id>` route rendering fields from snippet `vars` in declared order, with submit-on-Enter, cancel-on-Esc, and accessible markup. Mounted in the repurposed Quick Pane window.

**Files:**
- Create: `src/routes/form/{index.tsx,FieldRenderer.tsx,fields/{Text.tsx,Textarea.tsx,Choice.tsx,NumberField.tsx}}`.
- Create: `src/lib/form.ts` (typed wrapper for `form_submit` / `form_cancel`).
- Create: `src/routes/form/__tests__/Form.test.tsx`.

**Tasks:**
1. Route fetches snippet by id, renders fields in declared order. Field types: `text`, `textarea`, `choice` (options dropdown), `number`. `label`, `default`, `required` honored.
2. Enter submits (Shift+Enter inserts newline in textarea). Esc cancels. First focusable field auto-focused on mount.
3. Submit → `form_submit(id, values)` → window closes. Cancel → `form_cancel(id)` → window closes.
4. Window auto-grow: form width 400 px, height fits content (call `appWindow.setSize` after layout).

**Acceptance Criteria:**
- `pnpm test src/routes/form` passes: each field type rendered, Enter submits, Shift+Enter newline in textarea, Esc cancels, required-field validation blocks submit.
- axe-core clean (labels, focus order, ARIA roles).
- Visual check: window opens centred on monitor of focused app; closes on submit/cancel.

**Reviewer Checklist:**
- No direct IPC from field components — all goes through `src/lib/form.ts`.
- `choice` empty `options` array shows clear error, not blank dropdown.
- `default` values respected; `required: false` allows empty submit.

**Integration Checks:**
- `cd openmacro && pnpm test`
- `cd openmacro && pnpm tauri dev` — manual: trigger `;for` form snippet, fill, submit.

---

## Phase 8: Shell snippets

**Owner:** `codex`

**Goal:** Argv-only shell snippet runner with mandatory timeout, first-run consent, per-snippet `confirm`, restricted env. Disabled until user enables.

**Files:**
- Create: `src-tauri/src/expand/shell.rs`.
- Create: `src-tauri/src/commands/consent.rs`.
- Create: `src/routes/settings/ShellConsentDialog.tsx` (one-shot dialog).
- Create: `src-tauri/tests/shell_runner.rs`.
- Modify: `src-tauri/src/store/loader.rs` — refuse non-array `cmd`.

**Tasks:**
1. Loader: reject snippet if `cmd` is a string (must be argv array). Reject if `timeout_ms` absent or > 10 000.
2. Runner: `Command::new(args[0]).args(&args[1..])`; cwd = snippet file's parent; env stripped to `PATH` only; UTF-8 stdout trimmed; timeout via `tokio::time::timeout`; on timeout → kill + tray notification.
3. First-run consent: first time **any** snippet of `type: shell` is loaded, prefs flag missing → fire one-shot Tauri dialog. Until accepted, shell snippets load but never execute (resolution errors with "shell disabled").
4. Per-snippet `confirm: true` → yes/no toast before each run (`tauri-plugin-dialog`).

**Acceptance Criteria:**
- `cargo test shell_runner` covers: argv-only enforcement, timeout kill, stripped env, non-UTF-8 rejection.
- First-run consent flow exercised in `pnpm test` for the React dialog component.
- With consent off, `;head` snippet expansion fails with the literal trigger left intact + tray notification.

**Reviewer Checklist:**
- No `cmd.exe /c`, no `sh -c`, no shell metacharacter expansion anywhere in the runner.
- Env strip is allow-list (`PATH` only), not deny-list.
- Timeout kills the **entire** process tree (use `Job Object` on Windows, not just `Child::kill`).

**Integration Checks:**
- `cd openmacro/src-tauri && cargo test`
- `cd openmacro && pnpm test`

---

## Phase 9: Git-backed sync

**Owner:** `codex`

**Goal:** `sync/` subfolder as a git working tree. Periodic + on-change pull-rebase-push. Conflicts → `sync/.conflicts/<ts>/` + tray notification. HTTPS PAT in Windows Credential Manager; SSH via `ssh-agent`.

**Files:**
- Create: `src-tauri/src/sync/{mod.rs,agent.rs,git.rs,creds.rs,conflicts.rs}`.
- Create: `src-tauri/tests/sync_roundtrip.rs`, `src-tauri/tests/sync_conflict.rs`.
- Modify: `src-tauri/Cargo.toml` (`git2`, `wincred` / `windows-credentials`).
- Modify: `src/routes/settings/SyncPanel.tsx` — wire to backend commands.

**Tasks:**
1. `SyncBackend` trait (so B/C variants can slot in later); implement `GitBackend`. Tokio task: 60 s tick + debounced file-change trigger.
2. Cycle: `git fetch` → if dirty `git commit -am "openmacro: local"` → `git pull --rebase=merges` → on conflict, abort rebase, copy conflicted files to `sync/.conflicts/<unix-ts>/`, emit tray notification with one-click "open conflicts folder" → `git push`.
3. Credential storage: PAT in Windows Credential Manager keyed `openmacro/sync/<remote-host>`. SSH path uses local `ssh-agent` via libssh2 / git2 callbacks. PATs never written to disk in plaintext.
4. Settings sync panel: enter URL, pick auth (HTTPS PAT / SSH), test connection (`git ls-remote`), save → first clone into `%APPDATA%\openmacro\sync\`.

**Acceptance Criteria:**
- `cargo test sync_roundtrip` — two app instances pointed at a `tempdir` bare repo converge on a change.
- `cargo test sync_conflict` — concurrent edit on both sides produces `sync/.conflicts/<ts>/` with the conflicting files and emits one notification.
- Credentials never appear in logs, in error messages, or on disk outside Credential Manager (grep test).
- `git push` failure (offline, bad creds) → tray notification, retry on next tick, no crash.

**Reviewer Checklist:**
- `SyncBackend` trait surface is minimal (`init`, `tick`, `status`); no git-isms leak to callers.
- Conflict timestamp directory format is monotonic + sortable.
- Sync agent never touches files outside `sync/`.
- Credential read pulls fresh from Credential Manager — never cached in process for longer than one cycle.

**Integration Checks:**
- `cd openmacro/src-tauri && cargo test`
- `cd openmacro/src-tauri && cargo clippy -- -D warnings`

---

## Phase 10: Polish, installer, crash reporting

**Owner:** `codex`

**Goal:** Signed (or signing-ready) MSI installer, portable zip, in-process panic handler + crash dump writer, starter-pack `default.yaml`, release pipeline. Ready for M3 public-beta cut.

**Files:**
- Modify: `openmacro/src-tauri/tauri.conf.json` — bundler config, MSI metadata, both x64 + ARM64 targets.
- Create: `openmacro/installer/openmacro.wxs` (WiX template) or use Tauri's built-in WiX.
- Create: `openmacro/snippets/default.yaml` (starter pack: `;sig`, `;now`, `;log`, `;for`, `;head`).
- Create: `.github/workflows/release.yml` (Windows runner: build x64 + ARM64 → sign → release artifact).
- Modify: `src-tauri/src/lib.rs` — install `std::panic::set_hook` + `catch_unwind` around hook/form threads, write dumps to `%APPDATA%\openmacro\crashes\`.

**Tasks:**
1. Bundler config for MSI + portable zip; both x64 and ARM64.
2. Starter `default.yaml` ships in `snippets/` on first run (copy if folder empty).
3. Implement panic handler + crash dump writer (`std::panic::set_hook` → `%APPDATA%\openmacro\crashes\<ts>.log`); verify panic in form runner writes a dump file and surfaces tray notification on next launch.
4. CI release pipeline on tag push: build, sign with placeholder cert (real EV cert documented), publish to GitHub Releases.

**Acceptance Criteria:**
- `pnpm tauri build` produces signed (or signing-ready) MSI + portable zip on both x64 and ARM64.
- Fresh install on clean VM → tray icon → settings → `;sig` works end-to-end.
- Forced panic recoverable: app restarts, tray reappears, dump file present.
- CI release workflow green on a dry-run tag.

**Reviewer Checklist:**
- MSI installs without admin (per-user) and registers autostart under `HKCU`.
- Uninstall removes binaries but **preserves** `%APPDATA%\openmacro\` (verified in clean VM).
- Release workflow does not commit secrets or PATs to artifacts.
- Default snippet `;head` is documented as requiring shell-consent before it works.

**Integration Checks:**
- `cd openmacro && pnpm tauri build`
- `cd openmacro/src-tauri && cargo test --release`
- Manual: install MSI on Windows 11 VM; uninstall; verify clean state.

---

## Out-of-plan (deferred)

- Per-app allow/deny rules (PRD §3 non-goal; v1.1).
- Regex triggers (PRD §3 non-goal).
- macOS/Linux ports.
- Hosted sync backend.
- Opt-in telemetry (PRD §13, decide post-beta).
