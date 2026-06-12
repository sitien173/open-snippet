# Phase 1 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Kept frontend routing minimal by rendering `src/routes/settings/index.tsx` directly from `App.tsx` instead of adding a router dependency for a single placeholder screen.

### Spec deviations
- none

### Tradeoffs accepted
- Left the generated Vite/Tauri support files in place where they did not conflict with the phase scope to avoid unnecessary scaffold churn.

### Assumptions
- Manual `pnpm tauri dev` window verification will be recorded under Task 3, because the final phase behavior requires the main window to start hidden and be surfaced from the tray.

### Follow-ups for human
- none

### Test evidence
- scaffolding, integration-check covered: `pnpm install`
- scaffolding, integration-check covered: `pnpm build`

## Task 2
### Decisions made (not in spec)
- Vendored `tauri-plugin-single-instance` under `openmacro/src-tauri/vendor/` and added a Windows-only `windows_mutex_name(...)` builder hook because the published plugin hardcodes the mutex name from the app identifier and cannot otherwise satisfy `Global\openmacro-singleton`.

### Spec deviations
- none

### Tradeoffs accepted
- Kept the local plugin patch minimal and Windows-scoped instead of replacing the plugin, so the app still uses `tauri-plugin-single-instance` as required.

### Assumptions
- Showing and focusing the main settings window on second launch satisfies the Phase 1 requirement that a secondary instance focuses the existing app.

### Follow-ups for human
- none

### Test evidence
- RED: initial `cargo test single_instance_mutex_name_matches_spec` compile surfaced the local-plugin lifetime error in `vendor/tauri-plugin-single-instance/src/platform_impl/windows.rs`; fixed by moving the setup closure capture.
- GREEN: `cargo check --message-format short`

## Task 3
### Decisions made (not in spec)
- Intercepted close requests on the main window and hid the webview instead of allowing the app to exit, so the tray remains alive after the settings window is dismissed.

### Spec deviations
- none

### Tradeoffs accepted
- Placeholder `Pause/Resume` and `Reload snippets` handlers are explicit no-ops for this phase instead of introducing premature state plumbing.

### Assumptions
- Using the bundled default window icon as the tray icon satisfies the placeholder-icon requirement for Phase 1.

### Follow-ups for human
- none

### Test evidence
- integration-check covered: `cargo check --message-format short`
- runtime sanity: `pnpm tauri dev` reached `Running target\debug\openmacro.exe`
- automated singleton sanity: launching `src-tauri/target/debug/openmacro.exe` twice yielded `ProcessCount = 1`, `SecondExited = True`, `SecondExitCode = 0`
- manual visual tray/window inspection could not be directly observed from the terminal-only worker session

## Task 4
### Decisions made (not in spec)
- Registered the empty back-side modules directly in `lib.rs` rather than introducing an intermediate module tree, because each module is only a placeholder this phase.

### Spec deviations
- none

### Tradeoffs accepted
- The placeholder modules intentionally contain only `//! TODO` scope headers so Phase 2+ owners can shape their internals without undoing scaffolding abstractions.

### Assumptions
- The incremental-compilation "Access is denied" warnings emitted during `tauri build` are environment-specific Rust tempdir cleanup warnings, not source warnings; `cargo check` stayed warning-free and `cargo clippy -- -D warnings` passed.

### Follow-ups for human
- none

### Test evidence
- integration-check covered: `cargo check --message-format short`
- integration-check covered: `cargo clippy -- -D warnings`
- integration-check covered: `pnpm tauri build --debug`
- source check covered: `tgrep -w "SetWindowsHookEx" F:/projects_new/textblaze/openmacro/src-tauri/src -l` returned no matches
