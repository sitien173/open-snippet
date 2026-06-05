# Phase 4 — Resolvers + cursor token

- Status: IN_PROGRESS
- Owner: codex
- Started: 2026-06-06
- Finished: —

## Route
- Reason: Pure-logic back-side Rust (datetime, clipboard read, cursor math). Pure unit tests.
- Done When: `cargo test expand_resolver cursor_math` green; clippy clean; cursor math uses UTF-16 code units; `clipboard` var resolves before injector save; unknown placeholder fails closed.

## External Response
<!-- worker appends -->

## Review

## Squash Commit

## Decisions
- per-task in notes.md.

## Handoff
- Phase 5 (gemini, Settings UI) consumes `Snippet`/`VarDecl` shapes.
