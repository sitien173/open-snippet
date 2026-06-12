# Phase 2 — Snippet store + matcher

- Status: DONE
- Owner: codex
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Back-side Rust (YAML load, file watch, Aho-Corasick matcher). Pure logic, unit-testable. Codex's lane.
- Done When:
  - `cd openmacro/src-tauri && cargo test --all-features` passes (including `store_yaml` + `matcher_basic` test files)
  - `cd openmacro/src-tauri && cargo clippy -- -D warnings` clean
  - Malformed YAML files isolated (others still load, badge surfaced via store error API)
  - File watcher reload latency p95 < 250 ms (recorded in a watcher test)
  - Aho-Corasick matcher: longest-match-wins, word-boundary required, backspace pop, buffer reset
- Files:
  - Create: `src-tauri/src/store/{loader.rs,watcher.rs,model.rs}` + replace `store/mod.rs`
  - Create: `src-tauri/src/matcher/{buffer.rs,automaton.rs,boundary.rs}` + replace `matcher/mod.rs`
  - Create: `src-tauri/tests/store_yaml.rs`, `src-tauri/tests/matcher_basic.rs`
  - Modify: `src-tauri/Cargo.toml` (serde, serde_yaml, aho-corasick, notify, tokio)

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 2 / codex / n-a / 2026-06-06 / 2026-06-06 01:11:45 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented the Phase 2 YAML snippet loader, debounced watcher store, rolling match buffer, and allocation-audited Aho-Corasick matcher with unit and integration tests.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create | `docs/plans/2026-05-29-openmacro/phase-02/notes.md` | Added per-task decisions, deviations, and RED→GREEN evidence. |
| Modify | `openmacro/src-tauri/Cargo.toml` | Added `aho-corasick`, `notify`, `serde_yaml`, `tokio`, and `tempfile` dependencies. |
| Modify | `openmacro/src-tauri/Cargo.lock` | Locked the new Phase 2 dependency graph. |
| Create/Modify | `openmacro/src-tauri/src/store/{mod.rs,model.rs,loader.rs,watcher.rs}` | Replaced the store stubs with the YAML model, recursive loader, and debounced watcher store. |
| Create/Modify | `openmacro/src-tauri/src/matcher/{mod.rs,boundary.rs,buffer.rs,automaton.rs}` | Replaced the matcher stubs with boundary helpers, rolling buffer logic, and the AC matcher. |
| Create | `openmacro/src-tauri/tests/store_yaml.rs` | Added loader/schema integration tests for valid files, bad versions, isolation, vars, and duplicate triggers. |
| Create | `openmacro/src-tauri/tests/matcher_basic.rs` | Added buffer, boundary, reset, longest-match, boundary rule, backspace, reset, and UTF-8 matcher tests. |
| Modify | `docs/plans/2026-05-29-openmacro/phase-02/journal.md` | Appended this external response block. |
## COMMITS
- phase-2.task-1: 23b2f3fe46165c8e80f471c97c79b79a9af256c8  phase-2.task-1: implement YAML snippet loader
- phase-2.task-2: 5ab8ca8ddded6b29ffbce61c4cf52aaf181e5c78  phase-2.task-2: add debounced store watcher
- phase-2.task-3: 8d0ddc0dfac440f3cc56bff378808154b908c46c  phase-2.task-3: add match buffer and boundary logic
- phase-2.task-4: fe76df076b68374ea3742c5e25af6982fecaf4dc  phase-2.task-4: add aho-corasick matcher
## NOTES
- phase-02/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? WITH_DEBT  — All required tests and clippy checks passed, but `cargo llvm-cov` is not installed so the coverage target is recorded as a tool-availability deviation.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS_WITH_DEBT
- Quality Findings: No findings (scoped to changed files; tests cover all required cases incl. UTF-8 + boundary + duplicate trigger isolation).
- Final Status: PASS_WITH_DEBT
- Explanation: 9 matcher tests + 6 store tests green; clippy clean. Debt: `cargo-llvm-cov` not installed → coverage % not measured (worker recorded as deviation; manual test-set review covers cases listed in spec).
- Next: Phase 3.

## Squash Commit
- phase-2: snippet store + matcher — `24a7926`

## Decisions
- per-task decisions live in notes.md.

## Handoff
- Phase 3 (hook + injection) consumes `MatchBuffer` and `Snippet` types exposed from this phase.
