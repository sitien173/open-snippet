# Security Review Remediation Plan

Created: 2026-06-12
Status: DONE
Source: 2026-06-12 code review findings

## Superpowers Route

# ROUTE
- Owner: Coordinator for planning; implementation phases route to Codex or Gemini by side.
- Reason: The remediation spans backend security boundaries, Windows runtime behavior, frontend form contracts, and CI. High-impact security boundaries require a cross-validation checkpoint before implementation starts.
- Done When: Each phase has RED-to-GREEN regression evidence, phase notes, journal output, review, and the listed integration checks pass.

## Assumptions

- Tauri commands are a security boundary. The Rust backend must validate IPC payloads even when the current React UI normally sends valid values.
- The app remains Windows-first for runtime behavior, keyboard hooks, clipboard injection, foreground process denylisting, and autostart.
- YAML compatibility matters. Existing `kind: form` snippets in `snippets/default.yaml` and user files must continue to work or be migrated explicitly.
- No production behavior should rely on browser-only checks for security-sensitive inputs such as filesystem paths, Git remotes, credential hosts, or shell execution.
- Every bugfix phase follows `systematic-debugging` and starts with a failing regression test before implementation.

## Design Plan

### 1. Trust Boundaries

Backend commands should normalize and validate all external inputs before side effects:

- Snippet file paths must canonicalize under the configured snippets root, reject `_settings.yaml`, reject non-YAML extensions, and avoid symlink or junction escapes.
- Git HTTPS PAT auth must derive or verify the credential host against the actual remote URL. The credential callback must reject URL/host mismatches before returning a secret.
- PATs should be stored only after backend validation has confirmed the remote/auth pairing is coherent.
- Preferences should validate safety-sensitive bounds, especially `max_expansion_len`, and autostart changes should call the Tauri autostart plugin rather than only persisting JSON.

### 2. Runtime Side Effects

Expansion must not run side-effectful resolvers until the app is committed to expansion:

- If a snippet contains interactive form vars, open the form before resolving placeholders.
- Shell, clipboard, datetime, and form values should resolve once after submit.
- Cancelling a form must not execute shell commands, read clipboard placeholders, or alter the target app.
- Unknown placeholders in form snippets should surface after submit without silent failure.

### 3. Injection And Focus Safety

Clipboard and denylist behavior should match the security intent:

- For long replacements, set clipboard text, send paste, wait for paste completion, then restore the original clipboard.
- Regression tests should prove paste ordering at the abstraction boundary, and a gated Windows smoke should cover real Notepad behavior.
- The password-manager denylist must be driven by real foreground-window changes, not only test helpers.
- Foreground changes should reset the matcher buffer and disarm manual confirmation.

### 4. Frontend Contract Alignment

The UI should reflect the same snippet model the backend loads:

- `kind: form` vars should render as interactive form fields or be migrated to a clarified model. The shipped `snippets/default.yaml` shape is the compatibility baseline.
- Form window URLs should safely carry snippet IDs containing nested file paths, slashes, punctuation, or Unicode.
- Snippet creation/editing should validate global effective-trigger collisions, matching loader behavior.
- Frontend sync validation should reject malformed HTTPS remotes before invoking backend commands, while backend validation remains authoritative.

### 5. Logging And Verification

Privacy and CI should prevent the same class of regressions:

- Frontend logger redaction must apply to both in-memory ring entries and browser console forwarding unless verbose logging is explicitly enabled.
- CI should run type/build checks and Rust formatting checks, not only tests/lint.
- Regression tests should cover the exact review findings, not only adjacent happy paths.

## Remediation Matrix

| Finding | Phase |
| --- | --- |
| PAT can be sent to wrong Git remote | Phase 1 |
| `save_snippet` trusts caller path | Phase 1 |
| Autostart persisted but not applied | Phase 1 |
| Loader symlink/root escape risk | Phase 1 |
| Clipboard paste restores too early | Phase 2 |
| Denylist flag never updated in production | Phase 2 |
| Form side effects run before submit | Phase 2 |
| `kind: form` fields render blank | Phase 3 |
| Form route ID not encoded/decoded safely | Phase 2 and Phase 3 |
| UI duplicate trigger check differs from loader | Phase 3 |
| Frontend logger leaks raw fields to console | Phase 4 |
| CI misses build/fmt checks | Phase 5 |

## Implementation Plan

### Phase 1: Backend Trust Boundaries

**Owner:** `codex`

**Goal:** Close backend IPC and credential trust-boundary gaps before touching UI behavior.

**Files:**
- Modify: `src-tauri/src/commands/snippets.rs`
- Modify: `src-tauri/src/store/loader.rs`
- Modify: `src-tauri/src/commands/sync.rs`
- Modify: `src-tauri/src/sync/creds.rs`
- Modify: `src-tauri/src/sync/mod.rs`
- Modify: `src-tauri/src/commands/prefs.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify/Create: `src-tauri/tests/commands_roundtrip.rs`
- Modify/Create: `src-tauri/tests/sync_roundtrip.rs`
- Modify/Create: `src-tauri/tests/prefs_roundtrip.rs`
- Modify/Create: `src-tauri/tests/store_yaml.rs`

**Tasks:**
1. Add failing Rust tests for snippet path escape, `_settings.yaml` writes, non-YAML paths, and symlink/junction traversal policy.
2. Implement canonical snippet path validation and loader symlink policy with errors that are clear enough for the settings UI.
3. Add failing Rust tests for HTTPS PAT host/remote mismatch and callback URL mismatch, then enforce remote host validation before storing or returning credentials.
4. Apply backend validation for safety-sensitive prefs and wire autostart enable/disable through the Tauri autostart plugin.

**Acceptance Criteria:**
- `save_snippet_inner` cannot read or write outside the snippets root via absolute paths, `..`, symlinks, junctions, or special settings files.
- HTTPS PAT credentials are never stored or supplied unless the backend-validated remote host matches the credential host.
- Autostart checkbox changes alter actual OS autostart registration and persist prefs only when that operation succeeds.
- Error messages are deterministic enough for tests and usable in the UI.

**Reviewer Checklist:**
- No frontend-only validation is treated as sufficient for security.
- Secret material is not logged in new tests or implementation.
- Path validation handles Windows separators and case-insensitive root comparisons.
- `OPENMACRO_*` test overrides remain usable without weakening production validation.

**Integration Checks:**
- `cd src-tauri && cargo test commands_roundtrip sync_roundtrip prefs_roundtrip store_yaml -- --test-threads=1`
- `cd src-tauri && cargo test --all-features -- --test-threads=1`
- `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`

### Phase 2: Runtime Safety And Injection Ordering

**Owner:** `codex`

**Goal:** Make expansion side effects, clipboard paste, and foreground denylisting match the intended runtime safety model.

**Files:**
- Modify: `src-tauri/src/engine/orchestrator.rs`
- Modify: `src-tauri/src/expand/resolver.rs`
- Modify: `src-tauri/src/inject/mod.rs`
- Modify: `src-tauri/src/inject/clipboard.rs`
- Modify: `src-tauri/src/inject/sendinput.rs`
- Modify: `src-tauri/src/hook/mod.rs`
- Modify: `src-tauri/src/hook/thread.rs`
- Modify: `src-tauri/src/hook/winevent.rs`
- Modify: `src-tauri/src/form/runner.rs`
- Modify/Create: `src-tauri/tests/form_focus.rs`
- Modify/Create: `src-tauri/tests/shell_runner.rs`
- Modify/Create: `src-tauri/tests/notepad_smoke.rs`

**Tasks:**
1. Add failing tests proving form snippets do not execute shell or clipboard vars before submit and execute them only once after submit.
2. Detect form-required snippets before placeholder resolution, then resolve once after submitted values are available.
3. Fix clipboard paste ordering so the paste keystroke happens before clipboard restore; add abstraction-level tests and extend gated Windows smoke coverage for long replacements.
4. Wire foreground-window change detection into production runtime, update the denylist flag from real process basenames, and reset/disarm on foreground changes.

**Acceptance Criteria:**
- Form cancel has no shell, clipboard, injection, or focus side effect beyond closing the form.
- Shell vars in form snippets run once per successful submit when shell consent permits.
- Long replacement paste sends the snippet text, not the prior clipboard text, and restores the original clipboard afterward.
- Password-manager denylist blocks expansions in listed processes during real runtime operation.
- Snippet IDs in form window URLs are encoded on the backend side.

**Reviewer Checklist:**
- Runtime locking does not hold the injector mutex across user waits longer than necessary.
- Windows-only code is behind `#[cfg(windows)]` and testable non-Windows fallbacks remain sane.
- Denylist process-name lookup handles missing process paths without panics.
- Paste restore happens even when paste fails, where practical.

**Integration Checks:**
- `cd src-tauri && cargo test form_focus shell_runner notepad_smoke -- --test-threads=1`
- `cd src-tauri && cargo test --all-features -- --test-threads=1`
- Gated manual check: `OPENMACRO_E2E=1 cargo test notepad_smoke -- --ignored --test-threads=1`

### Phase 3: Frontend Form Contract And Snippet Editor Correctness

**Owner:** `gemini`

**Goal:** Align the React UI with the backend snippet model and close user-visible form/editor regressions.

**Files:**
- Modify: `src/routes/form/index.tsx`
- Modify: `src/routes/form/FieldRenderer.tsx`
- Modify: `src/routes/form/fields/Text.tsx`
- Modify: `src/routes/form/fields/Textarea.tsx`
- Modify: `src/routes/form/fields/Choice.tsx`
- Modify: `src/routes/form/fields/NumberField.tsx`
- Modify: `src/routes/form/__tests__/Form.test.tsx`
- Modify: `src/routes/form/__tests__/Form.a11y.test.tsx`
- Modify: `src/routes/settings/SnippetEditor.tsx`
- Modify: `src/routes/settings/__tests__/SnippetEditor.test.tsx`
- Modify: `src/lib/snippets.ts`

**Tasks:**
1. Add failing form tests using `kind: "form"` vars matching `snippets/default.yaml`, then render and validate those fields correctly.
2. Decode encoded form route IDs and handle snippet IDs containing slashes, nested paths, punctuation, and Unicode.
3. Update snippet editor collision detection to compare global effective triggers using current prefix and `trigger_literal`, not only same-file raw triggers.
4. Add regression tests for cross-file duplicate triggers, prefix-sensitive duplicates, and default YAML form rendering.

**Acceptance Criteria:**
- The shipped `;for` default snippet opens a usable form with Recipient and Topic fields.
- Required validation applies to `kind: form` fields and blocks empty submit.
- Nested file snippet IDs round-trip through the form route.
- Creating or editing a snippet cannot produce a global effective-trigger collision that the loader later rejects.

**Reviewer Checklist:**
- No mouse-only interaction is introduced.
- Labels, focus behavior, and a11y tests remain valid.
- Frontend type definitions match backend serialized data.
- Existing `text`, `textarea`, `choice`, and `number` form tests still pass.

**Integration Checks:**
- `pnpm test -- src/routes/form src/routes/settings`
- `pnpm lint`
- `pnpm build`

### Phase 4: Frontend Privacy And Sync UX Guardrails

**Owner:** `gemini`

**Goal:** Prevent frontend privacy leaks and make the sync/settings UI fail earlier and more clearly.

**Files:**
- Modify: `src/lib/logger.ts`
- Modify: `src/lib/__tests__/logger.test.ts`
- Modify: `src/lib/__tests__/logger-init.test.ts`
- Modify: `src/routes/settings/SyncPanel.tsx`
- Modify: `src/routes/settings/__tests__/SyncPanel.test.tsx`
- Modify: `src/routes/settings/PrefsPanel.tsx`
- Modify: `src/routes/settings/__tests__/PrefsPanel.test.tsx`

**Tasks:**
1. Add failing tests that spy on console forwarding and prove sensitive fields are redacted outside explicit verbose mode.
2. Pass redacted fields to the underlying logger/console by default while preserving verbose-mode diagnostics.
3. Reject malformed HTTPS remotes in `SyncPanel` before invoking backend commands, without weakening backend validation.
4. Ensure settings save errors, especially autostart failures from Phase 1, are surfaced without leaving misleading optimistic UI state.

**Acceptance Criteria:**
- Sensitive frontend log fields are redacted in both ring entries and console output by default.
- Verbose mode behavior is explicit and covered by tests.
- Invalid HTTPS remotes show inline validation and do not call `sync_test_connection` or `sync_init`.
- Autostart save failures are visible and do not display "Saved".

**Reviewer Checklist:**
- Redaction does not mutate caller-owned field objects unexpectedly.
- Sync UI validation handles SSH mode separately from HTTPS mode.
- Error badges are accessible and preserve existing visual patterns.
- Tests do not assert implementation details beyond the public behavior.

**Integration Checks:**
- `pnpm test -- src/lib src/routes/settings`
- `pnpm lint`
- `pnpm build`

### Phase 5: Verification Hardening And CI

**Owner:** `codex`

**Goal:** Add the checks that would have caught these regressions before review.

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `package.json` if script aliases are useful
- Modify/Create: `src-tauri/tests/notepad_smoke.rs`
- Modify/Create: `docs/logging.md` or a small verification note if test commands need documentation

**Tasks:**
1. Add `pnpm build` and `cargo fmt --check` to CI.
2. Extend the gated Windows smoke test so it exercises actual trigger expansion, including one long replacement that uses the paste path.
3. Add or document targeted verification commands for security-boundary, form, injection, logging, and CI regressions.
4. Run the full integration matrix and record residual manual-only risks.

**Acceptance Criteria:**
- CI fails on TypeScript build errors and Rust formatting drift.
- Gated smoke covers real Notepad input/output when `OPENMACRO_E2E=1`.
- The remediation plan has no untested CRITICAL/HIGH findings left.
- Manual-only checks are explicitly listed with exact commands or steps.

**Reviewer Checklist:**
- CI remains practical for pull requests and does not require GUI-only tests by default.
- E2E tests are gated and skipped cleanly when prerequisites are absent.
- Documentation points to commands that actually exist.
- No unrelated formatting churn is included.

**Integration Checks:**
- `pnpm install --frozen-lockfile`
- `pnpm build`
- `pnpm test`
- `pnpm lint`
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo test --all-features -- --test-threads=1`
- `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`

## Execution Notes

- Execute one phase at a time through `superpowers-ccg:executing-plans`.
- Before Phase 1 implementation, run a narrow cross-validation checkpoint on the credential/path/form contracts because these touch security boundaries.
- Workers must write RED-to-GREEN evidence into phase notes and journals.
- Do not pre-create `phase-NN/` directories. The executor creates them lazily.
- Keep unrelated repo changes, especially existing `.agents/` files, out of this plan unless the executor requires them for worker dispatch.
