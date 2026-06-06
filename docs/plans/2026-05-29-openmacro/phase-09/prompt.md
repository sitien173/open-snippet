## Original User Request
openmacro Phase 9 — Git-backed sync. `sync/` working tree under `%APPDATA%/openmacro/sync/`. Periodic + on-change pull-rebase-push. Conflicts → `sync/.conflicts/<unix-ts>/` + tray notification. HTTPS PAT in Windows Credential Manager; SSH via `ssh-agent`.

## Phase
Add a `SyncBackend` trait + `GitBackend` impl. Tokio task that ticks every 60 s and on file-change events (subscribe to the existing `store::watcher`). PAT credentials via `wincred`. Test the round-trip + conflict path against tempdir bare repos. Wire `SyncPanel.tsx` to the new commands.

## Tasks
- task-1: **`sync::SyncBackend` trait + `GitBackend`** — Create `src-tauri/src/sync/{mod.rs,git.rs}`:
  - `trait SyncBackend: Send + Sync { fn init(&self, remote: &str, auth: AuthMode, local_dir: &Path) -> Result<()>; fn tick(&self) -> Result<TickReport>; fn status(&self) -> SyncStatus; }`
  - `enum AuthMode { HttpsPat { host: String, username: String }, Ssh }` — PAT value pulled at use-time from Credential Manager.
  - `GitBackend` uses `git2::Repository`. Cycle implementation (`tick`):
    1. `fetch` from remote
    2. if working tree dirty → commit all `"openmacro: local"`
    3. `pull --rebase=merges` equivalent: `rebase(fetch_head)` via git2
    4. on conflict → abort rebase, copy each conflicting file (working-tree + index versions) to `sync/.conflicts/<unix-ts>/`, return `TickReport::Conflict { dir }`; emit notification through `NotifySink::sync_conflict(path)` (extend the existing trait)
    5. `push`
  - Tokio driver in `sync::mod`: 60 s `tokio::time::interval` + a `tokio::sync::mpsc` debounced (500 ms) trigger from the file watcher.
- task-2: **Credentials** — `src-tauri/src/sync/creds.rs`:
  - Wrap `windows-credentials` (or `wincred`) to read/write/delete credentials keyed `openmacro/sync/<remote-host>`.
  - `git2` callbacks: `RemoteCallbacks::credentials(|url, username_from_url, allowed|...)` returns `Cred::userpass_plaintext(user, pat)` for HTTPS; `Cred::ssh_key_from_agent(user)` for SSH.
  - **Never** write PATs to any file, env var, or log line. Add a clippy comment: `// PATs in-process only, deleted from stack as soon as git2 consumes them`. (Cannot truly enforce in safe Rust without scrubbing memory; document the contract.)
  - Tests: write→read→delete via a `CredentialStore` trait (real impl + mock); assert PATs never appear in `tracing::field::display`-style structures (use a `Secret` newtype with `Debug = "<redacted>"`).
- task-3: **Commands + tests** — `src-tauri/src/commands/sync.rs`:
  - `sync_test_connection(remote: String, auth: AuthMode, pat: Option<String>) -> Result<(), String>` — `git2::Remote::ls` with provided creds; if PAT supplied, store it in cred manager first.
  - `sync_init(remote, auth, pat?) -> Result<()>` — clone into `%APPDATA%/openmacro/sync/` (or open if already present), store creds.
  - `sync_tick_now() -> Result<TickReportDto>` — manual trigger from UI.
  - `sync_status() -> Result<SyncStatusDto>` — last tick time, branch, ahead/behind.
  - Tests:
    - `src-tauri/tests/sync_roundtrip.rs`: two `tempdir` "client" repos cloned from a bare repo; client A edits + ticks; client B ticks; B sees A's change.
    - `src-tauri/tests/sync_conflict.rs`: both clients edit the same line; A pushes; B ticks → `sync/.conflicts/<ts>/` populated; notification trait recorded.
    - Mock the credential store so tests don't touch Windows Credential Manager. **TDD-first** for both.
- task-4: **Settings panel wiring** — Modify `src/routes/settings/SyncPanel.tsx`:
  - Add `src/lib/sync.ts` typed wrapper: `syncTestConnection`, `syncInit`, `syncTickNow`, `syncStatus`. Mock-invoke pattern as before.
  - Panel: remote URL field, auth radio (HTTPS PAT / SSH), PAT text input (only when HTTPS), Test Connection button (shows OK/Err inline), Save & Init button (calls `syncInit`), Sync Now button (calls `syncTickNow`), status row (last tick / ahead-behind).
  - Vitest: `SyncPanel.test.tsx` — test connection happy path, missing PAT for HTTPS shows error, sync-now displays tick result.

## Context
- Add `git2 = "0.20"` and `windows-credentials = "0.5"` (or whatever current Tauri 2 ecosystem prefers) to Cargo.toml. Note `git2` brings libgit2 — ok.
- Snippet root and sync root are siblings under `%APPDATA%/openmacro/`. Sync agent NEVER touches files outside its `sync_root` (assert in tests with a recorded write-set).
- Use `tracing` or `log` (already in Cargo via tauri) for log lines. Never log creds.
- Clippy: `-D warnings`. Skip on `git2`-related lint where unavoidable, but justify in notes.

## Files
- Create: `F:/projects_new/textblaze/openmacro/src-tauri/src/sync/{mod.rs,git.rs,creds.rs,conflicts.rs}` (the existing `sync/mod.rs` is a stub; fill in)
- Create: `F:/projects_new/textblaze/openmacro/src-tauri/src/commands/sync.rs`
- Modify: `F:/projects_new/textblaze/openmacro/src-tauri/src/commands/mod.rs`, `src/lib.rs`
- Modify: `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml`
- Create: `F:/projects_new/textblaze/openmacro/src-tauri/tests/{sync_roundtrip.rs,sync_conflict.rs}`
- Create: `F:/projects_new/textblaze/openmacro/src/lib/sync.ts`, `F:/projects_new/textblaze/openmacro/src/routes/settings/__tests__/SyncPanel.test.tsx`
- Modify: `F:/projects_new/textblaze/openmacro/src/routes/settings/SyncPanel.tsx`

## Done When
- `cargo test --all-features` green incl. `sync_roundtrip`, `sync_conflict`
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- `pnpm test` green incl. `SyncPanel.test.tsx`
- Conflict folder format `sync/.conflicts/<unix-ts>/` monotonic + sortable
- Sync agent does NOT touch files outside its root (asserted via mocked fs in a test)
- PAT never appears in any log message, error string, or persisted file (grep test)

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-09/`. TDD per `test-driven-development`.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
