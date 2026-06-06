## Original User Request
openmacro Phase 10 — Polish, installer, crash reporting. Signed-ready MSI + portable zip bundling, starter `default.yaml`, panic hook + crash dump writer, GitHub Actions release workflow. Both x64 and ARM64 targets.

## Phase
Final polish phase. Configure Tauri bundler for MSI + portable + ARM64; ship the starter snippet pack; install `std::panic::set_hook` writing dumps to `%APPDATA%/openmacro/crashes/`; wrap hook/form threads in `catch_unwind`; add `.github/workflows/release.yml`.

## Tasks
- task-1: **Bundler config** — Modify `F:/projects_new/textblaze/openmacro/src-tauri/tauri.conf.json`:
  - `bundle.targets`: `["msi", "nsis"]` (portable zip handled by GitHub Actions packaging step rather than bundler)
  - `bundle.windows.wix.skipWebviewInstall: false`
  - `bundle.publisher: "openmacro"`, `bundle.shortDescription: "Snippet expander"`, `bundle.longDescription` filled in.
  - Add `productVersion` aligned to `Cargo.toml` (0.1.0).
  - MSI installs per-user (`bundle.windows.wix.installScope: "perUser"`). Document in notes if Tauri's MSI scope is fixed and we must rely on NSIS instead — pick whichever supports per-user without admin and explain.
  - ARM64: Tauri 2 supports targeting ARM64 via `cargo tauri build --target aarch64-pc-windows-msvc`. Don't try to cross-build here; just confirm the config doesn't pin to x64.
- task-2: **Starter snippets + first-run seeding** — Create `F:/projects_new/textblaze/openmacro/snippets/default.yaml`:
  ```yaml
  version: 1
  snippets:
    - trigger: ";sig"
      replace: "Best,\n— $|$"
    - trigger: ";now"
      replace: "It is {{date}} at {{time}} right now."
    - trigger: ";log"
      replace: "console.log(\"$|$\");"
    - trigger: ";for"
      replace: "Hello {{name}}! Re: {{topic}}.\n\n$|$"
      vars:
        - name: name
          kind: form
          label: "Recipient"
          required: true
        - name: topic
          kind: form
          label: "Topic"
          required: true
    - trigger: ";head"
      replace: "// Current branch: {{branch}}"
      vars:
        - name: branch
          kind: shell
          cmd: ["git", "rev-parse", "--abbrev-ref", "HEAD"]
          timeout_ms: 2000
          confirm: false
  ```
  Modify `src-tauri/src/lib.rs` startup: after creating the snippets root, if the directory is empty, copy `default.yaml` (embedded via `include_str!`) into it. Don't overwrite if the user already has files.
- task-3: **Panic + crash dump** — Modify `src-tauri/src/lib.rs`:
  - `std::panic::set_hook(Box::new(|info| { ... }))` writes to `%APPDATA%/openmacro/crashes/<unix-ts>.log` containing: timestamp, thread name, location, payload (`info.payload().downcast_ref::<&str>()` + `String`), and a `std::backtrace::Backtrace::force_capture()`.
  - Hook thread (`hook::thread`) and form-runner background tasks wrapped in `std::panic::catch_unwind` so a panic does not kill the orchestrator. On caught panic: write the dump (reuse helper) and continue.
  - On next launch, scan crash dir for new files since the last `prefs.json` mtime and emit a tray notification "openmacro recovered from a crash — see crashes folder". Add `Prefs.last_crash_check: Option<u64>` field (serde default).
  - Unit test (`tests/panic_dump.rs`): trigger a panic inside a function that calls the dump-writer directly (don't actually unwind in test if it complicates jsdom-free environment); assert a dump file is created in a tempdir.
- task-4: **Release workflow** — Create `F:/projects_new/textblaze/.github/workflows/release.yml`:
  - Triggers: `push: { tags: ["v*.*.*"] }` and `workflow_dispatch`
  - Matrix: `{ target: [x86_64-pc-windows-msvc, aarch64-pc-windows-msvc] }` on `windows-latest`
  - Steps: checkout, setup-node, setup-pnpm, setup-rust w/ target, `pnpm install`, `pnpm tauri build --target ${{ matrix.target }}`
  - Upload MSI + NSIS artifacts via `softprops/action-gh-release`. Dry-run friendly (skip release upload if `GITHUB_REF_TYPE != tag`).
  - Document EV cert step as a comment (no real cert plumbing in CI).
  - Also create `F:/projects_new/textblaze/.github/workflows/ci.yml` if not already present, running `cargo test`, `cargo clippy`, `pnpm test`, `pnpm lint` on every PR — but only ADD if missing. Check first.

## Context
- Tauri 2 bundler is `tauri-cli`'s built-in WiX/NSIS. Don't try to write `installer/openmacro.wxs` by hand unless Tauri config can't express what spec needs.
- `include_str!` for the default snippet pack avoids needing the file to exist relative to the running exe.
- Panic hook: install BEFORE Tauri's `builder().run()`. Avoid heavy deps in the hook (no `tracing`); use raw `std::fs::write`.
- Don't try to actually run `cargo tauri build` in this phase if it requires a long compile — the goal is configuration correctness. Mention in notes if `pnpm tauri build` was skipped due to time.
- Clippy: `-D warnings`.

## Files
- Modify: `F:/projects_new/textblaze/openmacro/src-tauri/tauri.conf.json`
- Create: `F:/projects_new/textblaze/openmacro/snippets/default.yaml`
- Modify: `F:/projects_new/textblaze/openmacro/src-tauri/src/lib.rs` (seeding, panic hook, catch_unwind)
- Modify: `F:/projects_new/textblaze/openmacro/src-tauri/src/commands/prefs.rs` (`last_crash_check` field)
- Create: `F:/projects_new/textblaze/openmacro/src-tauri/tests/panic_dump.rs`
- Create: `F:/projects_new/textblaze/.github/workflows/release.yml`
- Create: `F:/projects_new/textblaze/.github/workflows/ci.yml` (only if missing)

## Done When
- `cargo test --all-features -- --test-threads=1` green (incl. panic_dump)
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- `pnpm lint` clean (no new TS code expected; should remain green)
- `tauri.conf.json` parses (Tauri schema valid) — `pnpm tauri info` exits 0 (or document deviation)
- `snippets/default.yaml` loads through the existing loader (cargo test covers via fixture if practical)
- Release workflow YAML is syntactically valid (verify with `actionlint` if available; otherwise a `yamllint`-style sanity)

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-10/`. TDD per `test-driven-development` for the panic-dump test.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
