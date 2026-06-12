# Comprehensive Logging — Implementation Plan

- Source design: `docs/plans/2026-06-06-comprehensive-logging-design.md`
- Created: 2026-06-06
- Project: openmacro (Tauri desktop — Rust backend + React/Vite frontend)

## Routing Summary

| Phase | Owner     | Side   | Reason                                                       |
|-------|-----------|--------|--------------------------------------------------------------|
| 1     | codex     | back   | Rust crate setup, subscriber, ring buffer, redaction, tests  |
| 2     | codex     | back   | `tracing` instrumentation + two new Tauri IPC commands       |
| 3     | gemini    | front  | TS logger module, init wiring, console.* migration, tests    |
| 4     | gemini    | front  | `/logs` React route, polling, virtualization, save, a11y     |
| 5     | coordinator | docs | Single short doc; trivial coordinator task                   |

Cross-Validation skipped — each phase has one clear side owner. Phase 2
crosses the IPC boundary but the contract is already pinned in §3 of the
design doc (`get_logging_frontend_cfg`, `get_log_ring`), so it stays
back-side only.

Discipline: every phase is feature work → **test-first**
(`test-driven-development`). Workers must write the failing test first,
record RED→GREEN, and capture the evidence in `notes.md`.

---

## Phase 1 — Rust logging core

**Owner:** `codex`

**Goal:** Stand up `log_init` with config schema, three-layer subscriber
(stdout + rotating JSON file + ring buffer), redaction utility, and
rotation pruning. No call-site changes yet.

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `tracing`, `tracing-subscriber`,
  `tracing-appender`; dev-dep `tracing-test`, `tempfile`)
- Modify: `src-tauri/src/main.rs` (call `log_init::init` first; keep
  `LogHandles` in app state)
- Modify: `src-tauri/src/store/` (add `LoggingConfig` struct + YAML
  (de)serialization with defaults)
- Modify: `src-tauri/src/lib.rs` or module root (declare `log_init`)
- Create: `src-tauri/src/log_init/mod.rs`
- Create: `src-tauri/src/log_init/ring.rs` (`RingBuffer`, `RingLayer`,
  `LogEntry`, monotonic `seq`)
- Create: `src-tauri/src/log_init/redact.rs` (`FieldKind`, `redact_str`,
  `log_body!` macro, `verbose_content()` `OnceLock`)
- Create: `src-tauri/src/log_init/rotation.rs` (`prune_old_logs`)
- Create: `src-tauri/tests/log_init_filter.rs`
- Create: `src-tauri/tests/log_init_redact.rs`
- Create: `src-tauri/tests/log_init_ring.rs`
- Create: `src-tauri/tests/log_init_rotation.rs`

**Tasks:**
1. Add deps + `LoggingConfig` schema in store with code-level defaults
   (`level=info`, `file.enabled=true`, `max_files=7`,
   `verbose_content=false`, empty `modules`, frontend block defaults).
2. Implement `RingBuffer` + `RingLayer` + `LogEntry` with capacity 2000,
   `Mutex<VecDeque<_>>`, atomic `seq`, and `slice_since(seq)` query.
3. Implement `redact::redact_str` + `FieldKind` + `log_body!` macro +
   `verbose_content()` toggle (env `OPENMACRO_LOG_VERBOSE=1` beats config).
4. Implement `init(cfg)` composing `EnvFilter` (env > config), pretty
   stdout layer, JSON daily-rotating file layer (via `tracing-appender`)
   with `WorkerGuard` returned in `LogHandles`, and `RingLayer`. Call
   `prune_old_logs(max_files)` once.

**Acceptance Criteria:**
- `cd src-tauri && cargo test` green (all four new test files pass).
- `cd src-tauri && cargo fmt --check` clean.
- `cd src-tauri && cargo clippy -- -D warnings` clean on new code.
- Running `pnpm tauri dev` produces a daily-rotating JSON file at
  `<app_log_dir>/openmacro.log.YYYY-MM-DD` and pretty stdout.
- `OPENMACRO_LOG_VERBOSE=1 cargo test verbose_unredacts` (or equivalent
  test name) demonstrates the toggle.

**Reviewer Checklist:**
- `EnvFilter` resolution order: `RUST_LOG` env > YAML config > defaults.
- Ring capacity exactly 2000; FIFO eviction; `seq` strictly monotonic.
- `prune_old_logs` deletes *only* `openmacro.log.*` files beyond
  `max_files` — does not touch other files in the log dir.
- `WorkerGuard` held for app lifetime; no flush race on shutdown path.
- `redact_str` covers every `FieldKind` variant explicitly.
- No `panic!`/`unwrap()` on startup error paths reachable in release.
- `log_dir()` uses the same Tauri app dir helper used elsewhere in the
  store (resolve the open question from the design doc).

**Integration Checks:**
- `cd src-tauri && cargo test`
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo clippy -- -D warnings`
- `pnpm tauri dev` (smoke: app starts, file appears, terminal shows logs)

---

## Phase 2 — Rust instrumentation + IPC commands

**Owner:** `codex`

**Goal:** Adopt `tracing` across backend modules with `#[instrument]` on
entry points and `log_body!` at sensitive call sites. Add the two new
Tauri commands the frontend will consume.

**Files:**
- Modify: `src-tauri/src/commands/*.rs` (existing Tauri commands —
  `#[instrument(skip(...))]`, replace any `println!`/`eprintln!`)
- Modify: `src-tauri/src/matcher/**/*.rs` (instrument matcher passes;
  trigger names safe to log, candidate bodies via `log_body!`)
- Modify: `src-tauri/src/expand/**/*.rs` (instrument expansion;
  `body = %log_body!(...)`)
- Modify: `src-tauri/src/hook/**/*.rs` (instrument input hook events at
  debug level — no key content unless verbose)
- Modify: `src-tauri/src/inject/**/*.rs` (instrument inject ops;
  redact clipboard payloads)
- Modify: `src-tauri/src/sync/**/*.rs` (instrument sync ops; redact
  credentials/tokens; log remote URLs)
- Modify: `src-tauri/src/form/**/*.rs` (instrument form workflow steps;
  redact form values)
- Create: `src-tauri/src/commands/logging.rs` (new commands:
  `get_logging_frontend_cfg` returning `{level, modules, verbose_content}`;
  `get_log_ring(since_seq: u64) -> Vec<LogEntry>` reading from
  `LogHandles.ring`)
- Modify: `src-tauri/src/commands/mod.rs` + `main.rs` `invoke_handler!` to
  register the two new commands.
- Create: `src-tauri/tests/commands_logging.rs` (test both commands end
  to end with a primed ring buffer).
- Add `#[traced_test]` (via `tracing-test`) to one representative existing
  test in each of `matcher_basic.rs`, `sync_roundtrip.rs` (or nearest
  equivalents) — confirm spans emit without panicking.

**Tasks:**
1. Replace ad-hoc `println!`/`eprintln!` and add `#[instrument]` +
   structured `tracing::{info,debug,warn,error}!` calls module by module
   (commands → matcher → expand → hook → inject → sync → form).
2. Apply `log_body!` and explicit `FieldKind` at every sensitive site
   (snippet bodies, clipboard text, form values, credentials, tokens).
   Add `// SECURITY:` comment where redaction is load-bearing.
3. Implement `get_logging_frontend_cfg` + `get_log_ring(since_seq)`
   commands; wire into `invoke_handler!`; add `commands_logging.rs`
   integration tests.
4. Add `#[traced_test]` smoke tests; verify representative existing
   suites still pass.

**Acceptance Criteria:**
- `cd src-tauri && cargo test` green.
- `cd src-tauri && cargo fmt --check` clean.
- `cd src-tauri && cargo clippy -- -D warnings` clean.
- `get_log_ring(0)` returns all current entries; subsequent call with
  the last `seq` returns only newer entries.
- `get_logging_frontend_cfg` echoes the YAML `logging.frontend` block.
- Default-mode run: snippet body fields appear as `<redacted len=N>` in
  the JSON file; `OPENMACRO_LOG_VERBOSE=1` run: bodies appear in plain.

**Reviewer Checklist:**
- Every sensitive site uses `log_body!` or explicit `FieldKind` —
  grep for `body`, `clipboard`, `value`, `password`, `token`, `secret`
  in changed files; each match must route through redaction unless
  trivially non-sensitive.
- `#[instrument]` `skip` lists exclude large/secret args (no full
  snippet structs leaking through span fields).
- No `println!`/`eprintln!` remain in changed modules.
- Ring command pagination is correct (no off-by-one on `since_seq`).
- Commands return shapes the design doc specifies (TS will consume
  them in Phase 3 — keep field names stable).

**Integration Checks:**
- `cd src-tauri && cargo test`
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo clippy -- -D warnings`
- `pnpm tauri dev` → exercise a snippet expansion; confirm structured
  entries with spans appear in stdout and file.

---

## Phase 3 — Frontend logger module

**Owner:** `gemini`

**Goal:** Add `loglevel`-backed logger with redaction + in-memory ring,
wire init in `main.tsx`, migrate existing `console.*` sites.

**Files:**
- Modify: `package.json` + `pnpm-lock.yaml` (add `loglevel`,
  `loglevel-plugin-prefix`)
- Create: `src/lib/logger.ts` (ring buffer, redaction, `getLogger`,
  `getRing`, `initFromConfig`)
- Create: `src/lib/__tests__/logger.test.ts`
- Create: `src/lib/__tests__/logger-init.test.ts`
- Modify: `src/main.tsx` (await `initFromConfig()` before mounting)
- Modify: `src/routes/form/index.tsx` (replace `console.*` with
  `getLogger("form")` calls)
- Modify: `src/routes/settings/index.tsx` (→ `getLogger("settings")`)
- Modify: `src/routes/settings/PrefsPanel.tsx` (→ `getLogger("settings.prefs")`)
- Modify: `src/routes/settings/SyncPanel.tsx` (→ `getLogger("settings.sync")`)

**Tasks:**
1. Add deps; implement `src/lib/logger.ts` per design doc §3 including
   ring (cap 1000, FIFO), `SENSITIVE_KEYS` regex redaction, `loglevel`
   method-factory wrap, and `initFromConfig` that calls
   `invoke("get_logging_frontend_cfg")` and applies levels.
2. Write `logger.test.ts` (Vitest + jsdom) covering: ring writes, cap +
   FIFO, redaction by key, `verbose_content=true` disables redaction,
   `<redacted len=N>` uses code-point count for unicode.
3. Write `logger-init.test.ts` mocking `invoke` and `import.meta.env`:
   `VITE_LOG_LEVEL` wins; per-module levels applied; verbose flag
   respected.
4. Wire `await initFromConfig()` into `src/main.tsx` ahead of `<App />`;
   migrate the four `console.*` files to typed loggers.

**Acceptance Criteria:**
- `pnpm test` green.
- `pnpm lint` clean.
- `pnpm build` (which runs `tsc`) clean.
- No remaining `console.log`/`console.error`/`console.warn` in the four
  migrated files (`console.assert` allowed if it was already there).
- Manual: `pnpm tauri dev` → devtools console shows prefixed entries;
  in-memory ring populates (verify via temporary `window.__getRing` in
  dev only — remove before commit).

**Reviewer Checklist:**
- Ring cap precisely 1000; shifts oldest exactly when length exceeds cap.
- Redaction regex matches `body|content|clipboard|value|token|password|secret`
  case-insensitively at full-string boundaries (no partial matches
  like `bodyguard`).
- `initFromConfig` failure mode: if `invoke` rejects, logger still works
  at sensible default level (info) — don't crash app boot.
- Method-factory chain preserves original `loglevel` behavior (don't
  break level filtering).
- Module names align with what Phase 4's viewer will filter on.

**Integration Checks:**
- `pnpm test`
- `pnpm lint`
- `pnpm build`
- `pnpm tauri dev` (smoke: app boots; devtools shows formatted logs)

---

## Phase 4 — In-app log viewer route

**Owner:** `gemini`

**Goal:** `/logs` React route showing merged Rust + frontend rings with
filtering, search, pause, copy, save (JSON-lines), virtualization, and
a11y. Hotkey `Ctrl+Shift+L`.

**Files:**
- Modify: route registration (the table that registers `form` and
  `settings` routes — locate during execution and add `logs`)
- Modify: settings route or top-bar — add a link/button to `/logs`
- Modify: `package.json` (add `@tanstack/react-virtual` only if not
  already a dep — verify first)
- Create: `src/routes/logs/index.tsx`
- Create: `src/routes/logs/LogRow.tsx`
- Create: `src/routes/logs/filter.ts` (pure filter/merge helpers)
- Create: `src/routes/logs/__tests__/index.test.tsx`
- Create: `src/routes/logs/__tests__/index.a11y.test.tsx`
- Create: `src/routes/logs/__tests__/filter.test.ts`

**Tasks:**
1. Implement merge + filter pure helpers in `filter.ts` (merge Rust
   entries with frontend `getRing()` by timestamp; filters: source,
   min-level, module substring, search string; preserve insertion
   order on ties).
2. Build `LogsRoute` with 500ms polling of `invoke("get_log_ring",
   { sinceSeq })` + frontend `getRing()`, virtualized list via
   `@tanstack/react-virtual`, auto-scroll to bottom unless user
   scrolled up, pause/clear/copy controls.
3. Implement Save action using Tauri `save` dialog +
   `@tauri-apps/plugin-fs` `writeTextFile` writing JSON-lines.
   Re-redact entries on save unless an explicit "include sensitive
   content" checkbox is set in the dialog.
4. Wire `Ctrl+Shift+L` via existing `useHotkeys` pattern; write Vitest
   coverage (`index.test.tsx`, `filter.test.ts`) and axe test
   (`index.a11y.test.tsx`) — assert `role="log"` container, keyboard
   row nav, screen-reader-friendly timestamps.

**Acceptance Criteria:**
- `pnpm test` green (all three new test files).
- `pnpm lint` clean.
- `pnpm build` clean.
- Manual: `pnpm tauri dev`, navigate to `/logs`, trigger a snippet
  expansion; both Rust and frontend entries appear in real time;
  pause stops updates; filters narrow the view; save produces a
  `.jsonl` file that re-opens cleanly (`jq -c . < file` succeeds).
- Save with "include sensitive content" unchecked re-redacts even when
  app is running with `verbose_content=true`.

**Reviewer Checklist:**
- Polling interval not started until route mounts; cleared on unmount
  (no leaked intervals).
- `lastSeq` advances only when `rust.length > 0`; first poll uses 0.
- Virtualization handles 3000+ rows without dropping frames in dev.
- Auto-scroll suppressed when `scrollTop` isn't at bottom — verify
  with a unit test if feasible.
- Save dialog default filename includes ISO date for uniqueness.
- A11y: container has `role="log"` + `aria-live="polite"` (not
  `assertive` — would be noisy).

**Integration Checks:**
- `pnpm test`
- `pnpm lint`
- `pnpm build`
- `pnpm tauri dev` (manual viewer smoke)

---

## Phase 5 — Logging docs

**Owner:** `coordinator`

**Goal:** Short developer-facing doc explaining env vars, config schema,
redaction contract, and how to attach logs to a bug report.

**Files:**
- Create: `docs/logging.md`
- Modify: `README.md` (one-line link to `docs/logging.md` under a
  "Debugging" section if not already present)

**Tasks:**
1. Write `docs/logging.md` covering: env vars (`RUST_LOG`,
   `VITE_LOG_LEVEL`, `OPENMACRO_LOG_VERBOSE`, `VITE_LOG_VERBOSE`),
   YAML `logging` block schema with one example, sinks (stdout, file
   path, devtools, `/logs` route + hotkey), redaction contract +
   "include sensitive content" toggle on save, bug-report checklist.
2. Add link from `README.md`.

**Acceptance Criteria:**
- Doc renders cleanly on GitHub (no broken cross-links).
- All referenced env vars and config keys match those actually
  implemented in Phases 1–4 (verify by grep during review).

**Reviewer Checklist:**
- No mention of unimplemented features (no audit trail, no runtime UI
  level control, no IPC bridge of frontend → Rust file).
- Bug-report checklist tells users to use the viewer's redacted-save
  path by default.

**Integration Checks:**
- `pnpm lint` (markdownlint if configured, otherwise N/A)
- Manual: render `docs/logging.md` in an editor preview.

---

## Open Questions Carried From Design

These need answers during execution, not now:

1. Exact Tauri app dir API used by the existing store — `app_data_dir`
   vs `app_log_dir`. Phase 1 resolves by reading current store init code.
2. Whether `@tanstack/react-virtual` is already a dep. Phase 4 verifies
   before adding.

## Squash Strategy

After each phase's Review = PASS, the coordinator squashes that phase's
per-task commits into one with subject:

- `phase-1: rust logging core`
- `phase-2: rust tracing instrumentation + log IPC commands`
- `phase-3: frontend logger module + console migration`
- `phase-4: in-app log viewer route`
- `phase-5: logging docs`

Per-task commit hashes are review artifacts only and disappear after the
squash, in line with the repo's existing `phase-N:` subject convention.
