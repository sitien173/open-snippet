## Original User Request
Complete the 2026-06-12 security review remediation plan for `F:/projects_new/textblaze`. This phase closes backend trust-boundary findings from the security review.

## Phase
Phase 1: Backend Trust Boundaries. Harden Rust-side IPC, storage, credential, prefs, and autostart boundaries before any UI behavior changes.

## Tasks
- task-1: Add failing Rust tests for snippet path escape, `_settings.yaml` writes, non-YAML paths, and symlink/junction traversal policy.
- task-2: Implement canonical snippet path validation and loader symlink/root policy with deterministic, UI-usable errors.
- task-3: Add failing Rust tests for HTTPS PAT host/remote mismatch and credential callback URL mismatch, then enforce remote host validation before storing or returning credentials.
- task-4: Add failing Rust tests for safety-sensitive prefs and autostart behavior, then validate prefs and wire autostart enable/disable through the Tauri autostart plugin.

## Context
- Plan: `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/PLAN.md`
- Phase journal: `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-01/journal.md`
- Phase notes: `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-01/notes.md`
- Cross-validation is complete in the phase journal. Use the reconciled contracts there as the implementation guide.
- User constraint: every OpenMCP backend call must use `timeout_s=0`. If you invoke any backend through OpenMCP, preserve that constraint.

## Cross-Validation Contracts
- Treat Phase 1 as backend-only trust-boundary hardening. Add failing Rust regressions first, then enforce backend validation before filesystem, credential, prefs, or autostart side effects.
- `save_snippet_inner` must treat caller paths as untrusted and reject absolute paths, `..` escapes, sibling-prefix escapes, Windows separator/case tricks, drive/UNC/device paths, `_settings.yaml`, and non-YAML extensions before read/write.
- The loader must never read final canonical targets outside the snippets root. Choose and document the exact symlink/junction policy in implementation notes and tests.
- HTTPS PAT storage and callbacks must validate that the actual remote URL host matches the credential host. Host equality is required; exact remote matching is optional only if it fits the current credential model cleanly.
- Credential callbacks must fail closed on URL/host mismatch without returning or logging secret material.
- Validate safety-sensitive prefs on IPC and persisted-load paths, especially `max_expansion_len`.
- Autostart changes must call the Tauri autostart plugin and persist prefs only after enable/disable succeeds.
- Prefs/autostart also need RED-to-GREEN regression evidence.

## Files
- `F:/projects_new/textblaze/src-tauri/src/commands/snippets.rs`
- `F:/projects_new/textblaze/src-tauri/src/store/loader.rs`
- `F:/projects_new/textblaze/src-tauri/src/commands/sync.rs`
- `F:/projects_new/textblaze/src-tauri/src/sync/creds.rs`
- `F:/projects_new/textblaze/src-tauri/src/sync/mod.rs`
- `F:/projects_new/textblaze/src-tauri/src/commands/prefs.rs`
- `F:/projects_new/textblaze/src-tauri/src/lib.rs`
- `F:/projects_new/textblaze/src-tauri/tests/commands_roundtrip.rs`
- `F:/projects_new/textblaze/src-tauri/tests/sync_roundtrip.rs`
- `F:/projects_new/textblaze/src-tauri/tests/prefs_roundtrip.rs`
- `F:/projects_new/textblaze/src-tauri/tests/store_yaml.rs`

## Done When
- `save_snippet_inner` cannot read or write outside the snippets root via absolute paths, `..`, symlinks, junctions, sibling-prefix paths, Windows separator/case tricks, drive/UNC/device paths, or special settings files.
- Snippet save/load rejects non-YAML extension targets deterministically.
- HTTPS PAT credentials are never stored or supplied unless the backend-validated remote host matches the credential host.
- Credential callback URL/host mismatches fail closed and never return or log PAT values.
- Backend prefs validation rejects invalid safety-sensitive values, including invalid `max_expansion_len`, on relevant IPC and load paths.
- Autostart checkbox changes alter actual OS autostart registration and persist prefs only when that operation succeeds.
- Error messages are deterministic enough for tests and usable in the UI.
- Run, through `rtk`, and record fresh output for:
  - `cd src-tauri && cargo test commands_roundtrip sync_roundtrip prefs_roundtrip store_yaml -- --test-threads=1`
  - `cd src-tauri && cargo test --all-features -- --test-threads=1`
  - `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`

## Rules
Follow the contract in `F:/projects_new/textblaze/.agents/shared/worker-contract.md`: per-task workflow, test-first discipline, one commit per task with subject `phase-1.task-<M>: <summary>`, append a `## Task <M>` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-01/notes.md`, append the full `# EXTERNAL RESPONSE` block to `F:/projects_new/textblaze/docs/plans/2026-06-12-security-review-remediation/phase-01/journal.md`, and edit on disk.

Follow `F:/projects_new/textblaze/.agents/shared/erp.md` for the response format and `F:/projects_new/textblaze/.agents/BACKEND.md` for backend domain rules.

Follow the repository instructions supplied for this workspace: use `ogrep` for behavioral discovery, `tgrep` for literal searches, and prefix shell commands with `rtk`. Avoid broad, unrelated refactors or formatting churn.

If a requirement is impossible or ambiguous enough to risk the security contract, emit `CLARIFICATIONS NEEDED` and stop rather than guessing.

## Response Format
Respond per `F:/projects_new/textblaze/.agents/shared/erp.md`: return the `# EXTERNAL RESPONSE` block, then the single completion line:

`Phase 1 completed. Journal: docs/plans/2026-06-12-security-review-remediation/phase-01/journal.md.`
