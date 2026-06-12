# Phase 1 — Skeleton bootstrap

- Status: DONE
- Owner: codex
- Started: 2026-06-05
- Finished: 2026-06-05

## Route
- Reason: Back-side Rust scaffolding + plugin wiring (tray, single-instance, autostart) plus 7 module stubs. With the upstream template fork dropped, this is heavy back-side work belonging to Codex.
- Done When:
  - `cd openmacro && pnpm install` succeeds
  - `cd openmacro && pnpm tauri build --debug` succeeds
  - `pnpm tauri dev` opens a window + shows a tray icon (manual)
  - Second launch focuses first instance (manual)
  - Tray Quit exits cleanly; tray menu has Pause/Resume, Reload snippets, Open snippets folder, Open settings, Quit
  - `src-tauri/src/{engine,store,matcher,hook,inject,expand,form,sync}/mod.rs` exist with `//! TODO` headers and registered in `lib.rs`
  - Single-instance uses named mutex `Global\openmacro-singleton`
  - No `SetWindowsHookEx` calls yet
- Files:
  - Create: `openmacro/` (fresh Tauri 2 + React/TS/Vite scaffold) — `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`, `vite.config.ts`, `src/main.tsx`, `src/routes/settings/index.tsx`
  - Modify: `src-tauri/src/lib.rs` (tray + single-instance + autostart wiring), `src-tauri/tauri.conf.json` (app id `dev.openmacro`)
  - Create: `src-tauri/src/{engine,store,matcher,hook,inject,expand,form,sync}/mod.rs` (empty `//! TODO` modules registered in `lib.rs`)

## External Response
<!-- worker appends # EXTERNAL RESPONSE block here -->

# EXTERNAL RESPONSE
## META
- 1 / codex / n-a / 2026-06-05 / 2026-06-05 23:54:47 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Bootstrapped the `openmacro/` Tauri 2 React/TS/Vite app, added single-instance + autostart + tray wiring, and registered the eight empty Rust backend modules without adding expansion logic.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-01/notes.md` | Added per-task decisions, deviations, tradeoffs, assumptions, and evidence. |
| Create | `openmacro/{.gitignore,.vscode/extensions.json,index.html,package.json,pnpm-lock.yaml,tsconfig.json,tsconfig.node.json,vite.config.ts,README.md}` | Fresh pnpm Tauri scaffold and package lockfile. |
| Create | `openmacro/public/{tauri.svg,vite.svg}` | Default scaffold assets. |
| Create/Modify | `openmacro/src/{App.tsx,App.css,main.tsx,vite-env.d.ts,assets/react.svg}` | Reduced the default frontend to a minimal settings placeholder entrypoint. |
| Create | `openmacro/src/routes/settings/index.tsx` | Added the minimal settings route placeholder. |
| Create | `openmacro/src-tauri/{.gitignore,build.rs,capabilities/default.json,tauri.conf.json}` | Created Tauri app scaffold and configured product/identifier/window defaults. |
| Create | `openmacro/src-tauri/icons/*` | Kept the generated placeholder icon set for bundling and tray use. |
| Create/Modify | `openmacro/src-tauri/{Cargo.toml,Cargo.lock}` | Added Tauri tray/autostart dependencies and locked the Rust graph. |
| Create/Modify | `openmacro/src-tauri/src/{main.rs,lib.rs}` | Wired runtime startup, singleton handling, tray behavior, snippets-folder opener, and window hide/show behavior. |
| Create | `openmacro/src-tauri/src/{engine,store,matcher,hook,inject,expand,form,sync}/mod.rs` | Added empty backend module skeletons with `//! TODO` scope headers. |
| Create/Modify | `openmacro/src-tauri/vendor/tauri-plugin-single-instance/**/*` | Vendored and minimally patched the plugin to honor the required Windows mutex name. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-01/journal.md` | Appended this external response block. |
## COMMITS
- phase-1.task-1: fa8e7f2b62358a0888e5521d2c60aec527932f01  phase-1.task-1: bootstrap tauri app scaffold
- phase-1.task-2: 9110326d3d36eb7af90c1c17c5a1841005cb4b02  phase-1.task-2: add autostart and single-instance wiring
- phase-1.task-3: 0aa3f827a5aa7eb3b99daf96a2ba7e0abf5b37cd  phase-1.task-3: add tray menu and window controls
- phase-1.task-4: 88f6c02bd10ec30d0cdf520452431b8f4b043424  phase-1.task-4: add empty backend module skeleton
## NOTES
- phase-01/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? WITH_DEBT  — All compile/build/source checks passed, but direct visual tray/window observation could not be performed from the terminal-only worker session.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS_WITH_DEBT
- Quality Findings:
  | Severity | path:line | Problem | Fix |
  | --- | --- | --- | --- |
  | LOW | `src-tauri/src/lib.rs:92` | `.expect("default window icon")` panics if bundled icon missing — acceptable for placeholder bundled icon | Replace with proper error log when a custom tray icon ships in Phase 10 |
- Final Status: PASS_WITH_DEBT
- Explanation: All Done When criteria met (cargo check/clippy clean, tauri build succeeded, mutex constant + unit test present, 8 modules registered, no `SetWindowsHookEx`, second-launch focus verified via process-count test, identifier/productName/window defaults correct). Debt: (1) `tauri-plugin-single-instance` vendored + patched under `openmacro/src-tauri/vendor/` with a new `windows_mutex_name(...)` builder because the published plugin derives the mutex from the app identifier and cannot satisfy `Global\openmacro-singleton` literally; (2) no direct human-visual tray/window observation from the terminal worker session.
- Next: debt — record vendored plugin follow-up; user-side manual sanity launch of `pnpm tauri dev` recommended before Phase 2. Carry on to Phase 2.

## Squash Commit
- phase-1: skeleton bootstrap — Tauri 2 + React/TS/Vite app at `openmacro/`, tray + single-instance + autostart wiring, empty back-side module skeleton — `a76dcee`

## Decisions
- per-task decisions live in notes.md.

## Handoff
- Phase 2 (Snippet store + matcher, codex) consumes the empty `store/` and `matcher/` module stubs created here.
