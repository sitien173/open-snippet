# Phase 10 — Decision Notes

## Task 1
### Decisions made (not in spec)
- Kept the Tauri bundle target set to `["msi", "nsis"]` and left portable zip creation to the release workflow, matching the phase note that zip packaging belongs in CI rather than the bundler config.

### Spec deviations
- The current pinned Tauri build stack rejects `productVersion`, `bundle.windows.wix.installScope`, and `bundle.windows.wix.skipWebviewInstall` as unknown fields, so the final config stays schema-valid with the supported top-level `version` plus generic bundle metadata and relies on NSIS/Tauri defaults rather than unsupported explicit WiX options.

### Tradeoffs accepted
- The config-only task does not try to prove ARM64 output locally; it only ensures the Tauri config is architecture-neutral and does not pin the build to x64.

### Assumptions
- `bundle.windows.wix.installScope: "perUser"` is supported by the current Tauri 2 WiX schema; if `tauri info` later rejects it, that would be recorded in the final notes as a schema/tooling deviation.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- Config-only task initially, but later `cargo test` passes surfaced schema failures: `productVersion`, `bundle.windows.wix.installScope`, and `bundle.windows.wix.skipWebviewInstall` are all unknown to the pinned Tauri build stack. The config was corrected back to a schema-valid bundle section and the version/installer/WebView deviations were recorded here.

## Task 2
### Decisions made (not in spec)
- Embedded the shipped snippet pack with `include_str!` and seeded it only when the snippets directory has no files, so first-run setup stays deterministic and never overwrites existing user YAML.

### Spec deviations
- none

### Tradeoffs accepted
- The seeding check treats an existing file as the signal that the user already owns the snippets directory contents; empty subdirectories alone do not block seeding.

### Assumptions
- Shipping the exact sample YAML under `openmacro/snippets/default.yaml` is sufficient for both runtime seeding and loader-based fixture coverage.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- Added `seeds_default_yaml_only_when_directory_is_empty` in `src-tauri/src/lib.rs` and `shipped_default_yaml_loads_without_errors` in `src-tauri/tests/store_yaml.rs`; focused `cargo test --test store_yaml shipped_default_yaml_loads_without_errors -- --exact --nocapture` passed after wiring the embedded file and loader fixture.

## Task 3
### Decisions made (not in spec)
- Added a dedicated `crash` module so the panic hook, caught background panics, and launch-time recovery scan all share the same raw-`std` dump writer.
- Used the new `Prefs.last_crash_check` as the primary checkpoint and fell back to `prefs.json` mtime only when upgrading an older prefs file that lacks the field.

### Spec deviations
- The panic hook and the explicit `catch_unwind` handlers both write dumps, but the hook thread/form task recovery path does not attempt automatic task restart; it records the crash and lets the app continue running.

### Tradeoffs accepted
- The recovery scan keys off timestamp-named dump files rather than file metadata so the crash check remains simple and deterministic.
- The form background task uses `FutureExt::catch_unwind` on the spawned future to contain task panics without changing the existing orchestrator control flow.

### Assumptions
- `%APPDATA%/openmacro/crashes` maps to `dirs::config_dir()/openmacro/crashes` on Windows in the current Tauri desktop environment.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- Added `src-tauri/tests/panic_dump.rs` first, then the first focused `cargo test --test panic_dump write_dump_to_dir_creates_crash_log -- --exact --nocapture` run failed on crash-helper compile issues (`Option<u64>` inference and a temporary-path lifetime bug); after fixing those, the focused panic-dump test and the shipped-default loader test both passed.

## Task 4
### Decisions made (not in spec)
- Added a Windows-only PR `ci.yml` because the repo had no existing workflow and the Rust/Tauri checks depend on the Windows target surface.
- Packaged the portable artifact in CI by zipping the built `openmacro.exe` after `pnpm tauri build`, keeping `tauri.conf.json` limited to `msi` and `nsis` bundle targets as the phase requested.

### Spec deviations
- The final `tauri.conf.json` intentionally omits `productVersion`, `bundle.windows.wix.installScope`, and `bundle.windows.wix.skipWebviewInstall` because the pinned Tauri schema rejects them; the release workflow therefore ships the supported bundle metadata plus portable zip packaging without unsupported WiX-only knobs.

### Tradeoffs accepted
- The release workflow uploads the portable zip alongside MSI and NSIS assets even though the explicit upload bullet only named MSI and NSIS, because the phase brief requires portable packaging and the zip is produced in the same build output tree.

### Assumptions
- `openmacro.exe` is emitted at `src-tauri/target/<target>/release/openmacro.exe` for both Windows targets when `pnpm tauri build --target ...` completes.

### Follow-ups for human
- none

### Test evidence (RED→GREEN, or root cause for a fix)
- Workflow YAML was added after confirming `.github/workflows/ci.yml` was missing. Final syntax and toolability are verified in the phase-level check pass (`pnpm tauri info` plus `actionlint` if available, otherwise command-availability deviation recorded).
