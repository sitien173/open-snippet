## Original User Request
openmacro Phase 8 — Shell snippets. Argv-only runner with mandatory timeout, restricted env (PATH allow-list), first-run consent, per-snippet `confirm`, kill-tree on timeout. Disabled until user accepts consent.

## Phase
Wire `VarKind::Shell` into the resolver: when a shell var is encountered, execute the snippet's declared `cmd` (argv array) via a sandboxed runner; substitute stdout (trimmed UTF-8) as the var value. Loader rejects non-array `cmd` and missing/over-large `timeout_ms`. Consent persists in `Prefs.shell_consent` (already added in Phase 5a).

## Tasks
- task-1: **Loader validation** — Extend `src-tauri/src/store/loader.rs`:
  - Snippet YAML may declare per-var `cmd: [argv]` and `timeout_ms: u64` when `kind: shell`. Add fields to `VarDecl` (`#[serde(default)] cmd: Vec<String>`, `#[serde(default)] timeout_ms: Option<u64>`, `#[serde(default)] confirm: bool`) so existing snippets without these fields still load.
  - `validate_snippet` (already added in Phase 4) gains:
    - For any `var.kind == Shell`: reject `LoadError::ShellCmdNotArray` if `cmd.is_empty()` (since serde would deserialize a string as a parse error already; ensure the type system makes a string `cmd` impossible by using `Vec<String>` only). Reject `LoadError::ShellTimeoutInvalid` if `timeout_ms` is `None` or `> 10_000`.
  - Add corresponding `LoadError` variants + `message()` arms.
  - Tests in `src-tauri/tests/store_yaml.rs`: a YAML with `cmd: "echo hi"` (string) parses as `LoadError::Parse`; a YAML with `cmd: [echo, hi]` and no timeout fails `ShellTimeoutInvalid`; a valid shell var loads.
- task-2: **`expand::shell`** — Create `src-tauri/src/expand/shell.rs`:
  - Trait `ShellBackend { fn run(&self, args: &[String], cwd: &Path, timeout: Duration) -> Result<String, ShellError>; }`. Production impl `TokioShellBackend` uses `tokio::process::Command`:
    - `Command::new(&args[0]).args(&args[1..])`
    - `.current_dir(cwd)`
    - `.env_clear()` then `.env("PATH", std::env::var("PATH").unwrap_or_default())`
    - `.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())`
    - On Windows: create a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` and assign the child PID; on timeout, drop the Job Object handle to kill the entire process tree. Use the existing `windows` crate (features: `Win32_System_JobObjects`, `Win32_System_Threading`).
    - `tokio::time::timeout(duration, child.wait_with_output()).await`
    - Stdout decoded as UTF-8 (`String::from_utf8`); non-UTF-8 → `ShellError::NonUtf8`. Trim trailing whitespace.
    - Returns `Ok(trimmed)` or `Err(ShellError::{Timeout|NonUtf8|Spawn(_)|Exit(code)})`.
  - Unit tests (`#[tokio::test]`): timeout kills (use `cmd /c timeout 5` on Windows but skip if Windows-only; for cross-test use `tokio::process::Command::new("sleep").arg("5")` gated `#[cfg(unix)]` — or use a mocked `ShellBackend` and only sanity-test `TokioShellBackend` behind `#[cfg(windows)]` with `cmd /c ver`).
- task-3: **Resolver integration + consent gate** — In `expand::resolver`:
  - Resolver constructor takes optional `&dyn ShellBackend` (default: `NoopShellBackend` that returns `ShellError::Disabled`).
  - When a `vars` entry has `kind: Shell`: if `Prefs.shell_consent == false` → `ResolveError::ShellDisabled { name }`; else if `var.confirm == true` → emit a `NotifySink::confirm_shell(snippet_id, name, args)` call that returns `bool` (sync); else run directly. For Phase 8, `confirm` notification is implemented as a `NotifySink` method `confirm_shell` returning bool (real Tauri impl will be wired in a follow-up using `tauri-plugin-dialog`; the trait is enough for tests).
  - Orchestrator: pass current `Prefs.shell_consent` (read via `tauri::State<RwLock<Prefs>>`) and the `Arc<dyn ShellBackend>` into `Resolver::resolve`.
- task-4: **Consent dialog + tests** —
  - Create `src/routes/settings/ShellConsentDialog.tsx`: one-shot dialog rendered by `<PrefsPanel>` when `prefs.shell_consent === false` AND `prefs` was loaded freshly (track via local state — first mount where we observe `shell_consent: false`). Accept button calls `setPrefs({...prefs, shell_consent: true})`; Decline closes (`shell_consent` stays false).
  - Vitest: `src/routes/settings/__tests__/ShellConsentDialog.test.tsx` — renders dialog when `shell_consent: false`, hides when `true`, accept fires `setPrefs` with `shell_consent: true`.
  - Rust integration test `src-tauri/tests/shell_runner.rs`: argv-only enforcement (via loader test reused), timeout kill (mocked `ShellBackend`), env strip allow-list (assert `Command` config — abstract behind a probe trait if reading the real env is awkward), non-UTF-8 stdout → `NonUtf8`. **TDD-first**: failing tests first.

## Context
- `Prefs.shell_consent` already exists from Phase 5a (`commands/prefs.rs`).
- Loader's `validate_snippet` was added in Phase 4 (cursor token validation).
- Resolver lives in `src-tauri/src/expand/resolver.rs`; current signature already takes a `ClipboardReader` and (Phase 6) an optional form-values overlay — add shell backend + prefs snapshot alongside.
- Job Object usage: `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (constant 0x00002000). Refer to existing windows-crate usage in `src-tauri/src/inject/sendinput.rs` for style.
- Spec says first-run consent fires on first load of any shell snippet. Simpler equivalent: fire when user is in Settings with `shell_consent: false` AND there exists at least one shell snippet (or any shell var). Front-end can call `listSnippets()` to detect. Acceptable.
- Clippy: `-D warnings`.

## Files
- `F:/projects_new/textblaze/openmacro/src-tauri/src/expand/shell.rs` (create)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/expand/{mod.rs,resolver.rs}` (modify — export shell, wire shell backend + consent gate)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/store/{loader.rs,model.rs}` (modify — VarDecl fields + LoadError variants + validation)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/engine/orchestrator.rs` (pass prefs + shell backend into resolver)
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml` (windows JobObjects feature if not present)
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/{store_yaml.rs,shell_runner.rs}` (modify/create)
- `F:/projects_new/textblaze/openmacro/src/routes/settings/ShellConsentDialog.tsx` (create)
- `F:/projects_new/textblaze/openmacro/src/routes/settings/PrefsPanel.tsx` (modify — mount dialog when needed)
- `F:/projects_new/textblaze/openmacro/src/routes/settings/__tests__/ShellConsentDialog.test.tsx` (create)

## Done When
- `cargo test --all-features` green
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- `pnpm test` green (existing 21 + 1 new shell-consent test minimum)
- `pnpm lint` clean
- Loader rejects string `cmd` and missing/excessive timeout
- Resolver returns `ShellDisabled` when `shell_consent: false`
- Timeout kills the entire process tree (Windows Job Object) — verified by code review of the runner; an actual kill-tree integration test is best-effort

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-08/`. TDD per `test-driven-development` — failing tests first for every behavior-adding task.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.

## Coordination Note
This phase is mostly back-side. Front-end ShellConsentDialog is a small companion. Codex owns the whole phase — the Rust touches are 80%+ of the work and the dialog is trivial vitest scope.
