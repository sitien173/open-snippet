## Original User Request
Build openmacro — a Windows text-expansion app on Tauri 2. Phase 1 bootstraps the app skeleton with no expansion logic. The user explicitly dropped any dependency on `dannysmith/tauri-template`; start from a clean `pnpm create tauri-app` scaffold.

## Phase
Bootstrap a runnable Tauri 2 + React/TS/Vite app at `openmacro/` inside this repo, wire tray + single-instance + autostart, and scaffold empty Rust modules. No keyboard hook, no matcher, no IPC commands yet.

## Tasks
- task-1: Bootstrap fresh Tauri 2 app at `F:/projects_new/textblaze/openmacro/` (React + TS + Vite + pnpm). Set product name `openmacro`, identifier `dev.openmacro`. Window defaults: not visible at startup, title `openmacro Settings`, 900×640, resizable. Verify `pnpm tauri dev` launches a window (do not commit `node_modules`, `dist`, `target` — add `.gitignore`).
- task-2: Add `tauri-plugin-autostart` and `tauri-plugin-single-instance`. Single-instance uses the named mutex `Global\openmacro-singleton`; a second launch focuses the existing tray (or shows the settings window). Autostart plugin wired but UI toggle deferred to Phase 5.
- task-3: Build tray icon + native menu using `tauri::tray::TrayIconBuilder` and `Menu::with_items`. Menu items in this order: `Pause/Resume` (placeholder, no-op for now), `Reload snippets` (placeholder), `Open snippets folder` (opens `%APPDATA%\openmacro\snippets\` via `tauri-plugin-opener`, creating the folder if missing), `Open settings` (shows the main window), `Quit` (exits cleanly). Left-click on tray icon shows the settings window. Tray icon must survive webview hide.
- task-4: Create empty back-side module skeleton: `src-tauri/src/{engine,store,matcher,hook,inject,expand,form,sync}/mod.rs`, each with a `//! TODO: <one-line scope>` header. Register all eight modules in `src-tauri/src/lib.rs` (`pub mod engine;` …). Crate must compile clean (`cargo check` + `cargo clippy -- -D warnings`).

## Context
Repo root: `F:/projects_new/textblaze/` is empty git repo (no commits, no `openmacro/` yet).

Plan: `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/PLAN.md` (read Phase 1 only).
Design (for terminology only — do not pre-implement later phases): `F:/projects_new/textblaze/docs/2026-05-29-openmacro-design.md`.
PRD: `F:/projects_new/textblaze/docs/2026-05-29-openmacro-prd.md`.

Target Tauri version: **Tauri 2.x** (latest stable on crates.io). Use `@tauri-apps/cli@^2`, `@tauri-apps/api@^2`. Bundler identifier `dev.openmacro`.

For TDD discipline this phase: most tasks are scaffolding without testable logic. Where a unit can be tested cheaply (e.g. `single_instance` mutex name constant, snippets-folder path helper), write a `#[cfg(test)]` test first. Otherwise rely on the integration checks below as completion evidence and record that under `Test evidence: scaffolding, integration-check covered` in notes.md.

## Files
- Create everything under `F:/projects_new/textblaze/openmacro/` (scaffold output).
- `F:/projects_new/textblaze/openmacro/.gitignore`
- `F:/projects_new/textblaze/openmacro/package.json`
- `F:/projects_new/textblaze/openmacro/vite.config.ts`
- `F:/projects_new/textblaze/openmacro/src/main.tsx`
- `F:/projects_new/textblaze/openmacro/src/routes/settings/index.tsx` (minimal placeholder route — single `<h1>openmacro settings</h1>`)
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml`
- `F:/projects_new/textblaze/openmacro/src-tauri/tauri.conf.json`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/main.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/lib.rs`
- `F:/projects_new/textblaze/openmacro/src-tauri/src/{engine,store,matcher,hook,inject,expand,form,sync}/mod.rs`
- Icon assets under `F:/projects_new/textblaze/openmacro/src-tauri/icons/` (Tauri default placeholder icons are fine)

## Done When
- `cd F:/projects_new/textblaze/openmacro && pnpm install` — exit 0
- `cd F:/projects_new/textblaze/openmacro/src-tauri && cargo check` — exit 0, no warnings
- `cd F:/projects_new/textblaze/openmacro/src-tauri && cargo clippy -- -D warnings` — exit 0
- `cd F:/projects_new/textblaze/openmacro && pnpm tauri build --debug` — exit 0 (full bundle is slow; if it exceeds a reasonable budget on this machine, `pnpm tauri build --debug --no-bundle` is acceptable evidence — record the substitution under "Spec deviations" in notes.md)
- Source includes a `pub const SINGLE_INSTANCE_MUTEX: &str = "Global\\openmacro-singleton";` (or equivalent) referenced where the plugin is initialized, plus a unit test asserting the literal value
- `tauri.conf.json` `identifier` == `dev.openmacro`, `productName` == `openmacro`
- Tray menu items present in source in the listed order
- Empty modules present and registered in `lib.rs`
- `grep -r "SetWindowsHookEx" F:/projects_new/textblaze/openmacro/src-tauri/src/` returns no hits
- Manual visual checks (window + tray icon + second-launch focus + Quit) recorded under "Test evidence" in notes.md task-3 block — describe what you observed when running `pnpm tauri dev`

## Rules

Follow the contract in `F:/projects_new/textblaze/.agents/shared/worker-contract.md` — per-task workflow (test-first where applicable → one commit per task `phase-1.task-<M>: …` → append a `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-01/notes.md` → append the `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-01/journal.md`) plus discipline (test-first, root-cause-first, evidence) and prompt discipline (edit on disk, no duplication, no redesign, unclear → CLARIFICATIONS NEEDED + stop).

This is a fresh repo with no commits — your `phase-1.task-1` commit is the initial commit on the master branch. No upstream to rebase against.

## Response Format

Respond per `F:/projects_new/textblaze/.agents/shared/erp.md` — return the `# EXTERNAL RESPONSE` block, then the single completion line.
