# Phase 2 - Decision Notes

## Task 1
### Decisions made (not in spec)
- Implemented snippet IDs as `"{relative_path_with_forward_slashes}::{trigger}"`, normalizing path separators to `/` so IDs are stable across Windows path display differences.
- Exposed `load_from_root(root: &Path) -> io::Result<LoadResult>` as the task-1 entrypoint and treated unreadable directories/files as I/O failures while YAML/schema problems are captured as per-file `LoadError`s.

### Spec deviations
- none

### Tradeoffs accepted
- `VarDecl.label`, `default`, and `format` are optional fields, with `required` defaulting to `false` and `options` defaulting to an empty list, which keeps deserialization minimal while matching the current schema needs.

### Assumptions
- The YAML root shape is `version: 1` plus a `snippets:` sequence; files missing the `snippets` key are treated as malformed YAML/schema input for that file.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test --test store_yaml` failed with unresolved `store::{load_from_root, LoadResult, VarKind}` imports because the Phase 1 stubs exposed no loader/model API.
- GREEN: `cargo test --test store_yaml`

## Task 2
### Decisions made (not in spec)
- Backed `Store` with a dedicated worker thread plus a `tokio::sync::watch` snapshot channel, using `notify` only for filesystem edge detection and a standard-library quiet-period loop for the 200 ms debounce.

### Spec deviations
- none

### Tradeoffs accepted
- `Store::spawn` returns `notify::Result<Store>` instead of a bare `Store` so watcher initialization failures remain explicit instead of panicking.

### Assumptions
- Publishing an initial revision-0 snapshot on spawn is acceptable and useful for subscribers, with later filesystem or manual reloads incrementing the revision counter.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test watcher_latency` failed with unresolved `Store` and `SnapshotInner` symbols while the watcher API still only existed as tests.
- GREEN: `cargo test watcher -- --nocapture`

## Task 3
### Decisions made (not in spec)
- Tracked two boundary notions in `MatchBuffer`: the char before the buffer start (for later exact matcher checks) and the char before the current trailing candidate (to satisfy the Task 3 boundary-state API cleanly).

### Spec deviations
- none

### Tradeoffs accepted
- Kept `reset_with(Reset)` as a thin wrapper over `reset()` in this phase, because the reset reason is only a typed signal until the hook layer arrives in Phase 3.

### Assumptions
- `boundary_char_state(0)` should report the char immediately preceding the current trailing candidate, which is what the Task 3 buffer tests exercise directly.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test --test matcher_basic` failed with unresolved `matcher::{BoundaryState, MatchBuffer, Reset}` imports because the matcher module still only exposed the Phase 1 stub.
- GREEN: `cargo test --test matcher_basic`

## Task 4
### Decisions made (not in spec)
- Stored matcher pattern metadata as `Arc<str>` snippet IDs so `Matcher::on_char` can return `MatchHit` without allocating on the hot path.

### Spec deviations
- coverage tool unavailable - manual review of test set instead (`cargo llvm-cov --summary-only` failed because `cargo llvm-cov` is not installed).

### Tradeoffs accepted
- Kept the hot-path haystack assembly in a reusable preallocated `String` plus a fixed `[usize; 65]` byte-offset table, which avoids per-keystroke allocation while keeping the suffix/boundary logic simple to audit.

### Assumptions
- The Phase 2 hot-path audit requirement is satisfied by `Matcher::on_char`: no `Vec::push`, no `format!`, and no `Box::new` appear on that path; the only per-keystroke mutations are `MatchBuffer::push_char`, `String::clear`, `String::push` into preallocated storage, and fixed-array writes.

### Follow-ups for human
- none

### Test evidence
- RED: `cargo test longest_match_wins` failed with unresolved `matcher::Matcher` because `automaton.rs` was still only a placeholder.
- RED: `cargo clippy -- -D warnings` flagged the range-loop, `loop`/`match`, and `&PathBuf` issues fixed in `matcher::automaton` and `store::watcher`.
- GREEN: `cargo test --all-features`
- GREEN: `cargo clippy -- -D warnings`
- Spec deviation evidence: `cargo llvm-cov --summary-only` -> `error: no such command: llvm-cov`
