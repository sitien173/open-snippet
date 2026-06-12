# Phase 10 — Polish, installer, crash reporting

- Status: DONE
- Owner: codex
- Started: 2026-06-06
- Finished: 2026-06-06

## Route
- Reason: Final polish — bundler config + starter snippets + panic hook + CI workflow. All back-side + config.
- Done When: cargo tests green incl. panic_dump; clippy clean; pnpm lint clean; tauri.conf.json valid; release workflow YAML valid; default.yaml loads.

## External Response
<!-- worker appends -->

# EXTERNAL RESPONSE
## META
- 10 / codex / n-a / 2026-06-06 / 2026-06-06 14:36:12 +07:00 / docs/plans/2026-05-29-openmacro
## SUMMARY
Implemented Phase 10 polish: schema-valid Windows bundling metadata, first-run starter snippet seeding, panic/crash dump reporting with recovery notification, and Windows CI/release workflows with portable zip packaging.
## FILES MODIFIED
| Action | Path | Change |
| --- | --- | --- |
| Create/Modify | `docs/plans/2026-05-29-openmacro/phase-10/{notes.md,journal.md}` | Added per-task decisions/evidence and appended this external response block. |
| Modify | `openmacro/src-tauri/tauri.conf.json` | Finalized the schema-valid bundle targets and Windows bundle metadata required for release packaging. |
| Create | `openmacro/snippets/default.yaml` | Added the shipped starter snippet pack used for first-run seeding and loader coverage. |
| Modify | `openmacro/src-tauri/src/{lib.rs,commands/prefs.rs,engine/orchestrator.rs,hook/thread.rs}` | Added snippet seeding, crash-recovery notification, prefs checkpointing, and panic containment in the hook/form background paths. |
| Create | `openmacro/src-tauri/src/crash.rs` | Added the shared panic hook and crash dump writer/report scanner. |
| Modify | `openmacro/src-tauri/{Cargo.toml,Cargo.lock}` | Added `futures-util` for async panic containment and updated the lockfile. |
| Create/Modify | `openmacro/src-tauri/tests/{store_yaml.rs,panic_dump.rs,prefs_roundtrip.rs}` | Added coverage for shipped defaults and panic dump creation, and updated prefs round-trip coverage for the new crash checkpoint field. |
| Create | `.github/workflows/{release.yml,ci.yml}` | Added Windows release and PR verification workflows, including portable zip packaging in CI. |
## COMMITS
- phase-10.task-1: 830bda6d7b9e82b5b8d6e7a755e0ab52a018ccc0  phase-10.task-1: configure windows bundling
- phase-10.task-2: f51308c0980bac6b636e9711578b6d8cc4c8d0a9  phase-10.task-2: seed starter snippets
- phase-10.task-3: e5d200fbe8235137b0ed6f8b5f619bad6c2266b1  phase-10.task-3: add panic dump reporting
- phase-10.task-4: 30a0b494660ae939e2be357e62da4388dc0b12c9  phase-10.task-4: add release workflows
## NOTES
- phase-10/notes.md  (## Task 1, ## Task 2, ## Task 3, ## Task 4)
## SPEC COMPLIANCE
- Meets Spec? YES  — `cargo test --all-features -- --test-threads=1`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm lint`, and `pnpm tauri info` all passed; `actionlint` was unavailable, so workflow syntax sanity was verified with `pnpm dlx js-yaml` instead.
## CLARIFICATIONS NEEDED
None
## NEXT
TASK_COMPLETE

## Review
- Spec Status: PASS — `cargo test --all-features -- --test-threads=1` (all suites green incl. panic_dump, shipped_default_yaml_loads_without_errors), `cargo clippy --all-targets --all-features -- -D warnings` clean, `pnpm lint` clean. Workflow YAML, tauri.conf.json, default.yaml all present and valid.
- Quality Findings: No findings.
- Final Status: PASS
- Explanation: All Done When checks pass with fresh evidence; spec satisfied as described.
- Next: done

## Squash Commit
- phase-10: polish, installer config, panic dumps, release workflows  (5730d8a)

## Decisions
- per-task in notes.md.

## Handoff
- Plan complete. Out-of-plan items in PLAN.md "Out-of-plan (deferred)" section.
