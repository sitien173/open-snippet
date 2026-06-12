# Phase 3 - Decision Notes

## Task 1
- Decisions made: Treat `kind: "form"` variables as text-like form fields in the frontend route and field renderer so shipped snippets render usable inputs.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: Existing text field behavior is the compatibility target for legacy `kind: "form"` variables.
- Follow-ups for human: none
- Test evidence: RED reproduced by temporarily removing `kind: "form"` handling, then running `rtk pnpm test -- src/routes/form/__tests__/Form.test.tsx -t "renders 'form' kind fields as text inputs and validates them" --reporter=basic`; failures included `Unable to find a label with the text of: Recipient`. GREEN verified after restore by the Phase 3 targeted suite.

## Task 2
- Decisions made: Added a regression test for URL-encoded snippet IDs containing slash and punctuation characters.
- Spec deviations: none
- Tradeoffs accepted: No production code change was needed because React Router `useParams` already decodes the encoded route segment used by the form route.
- Assumptions: React Router decoding behavior is stable for `%2F` and `%3B` encoded route parameters in supported runtimes.
- Follow-ups for human: none
- Test evidence: Test-only confirmation; `decodes URL-encoded snippet IDs with slashes and matches snippet` passes in the Phase 3 targeted suite. No pre-fix RED was observed for route decoding itself.

## Task 3
- Decisions made: Compute the current edited snippet's effective trigger using the active prefix and `trigger_literal`, then compare it against all snippets' `effective_trigger` values.
- Spec deviations: none
- Tradeoffs accepted: Collision detection relies on backend-provided `effective_trigger` for existing snippets.
- Assumptions: `list_snippets` continues to serialize `effective_trigger` for all snippets.
- Follow-ups for human: none
- Test evidence: RED reproduced by temporarily restoring the old same-file raw-trigger comparison, then running `rtk pnpm exec vitest run src/routes/settings/__tests__/SnippetEditor.test.tsx -t "global effective collision|prefix-sensitive duplicate" --reporter=basic`; the global effective collision test failed because `/trigger collision/i` was not rendered. GREEN verified after restore by the Phase 3 targeted suite.

## Task 4
- Decisions made: Added regression coverage for cross-file effective-trigger collisions and prefix-sensitive literal/non-literal duplicates.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: Prefix-sensitive duplicate behavior should match loader rejection semantics globally, not only within one source file.
- Follow-ups for human: none
- Test evidence: RED reproduced with the same temporary old collision logic as Task 3; both `trigger collision in another file DOES show error and disables save (global effective collision)` and `prefix-sensitive duplicate triggers (e.g. literal :test vs non-literal test with prefix :) collide` failed because `/trigger collision/i` was not rendered. GREEN verified after restore by the Phase 3 targeted suite.

## Task 5
- Decisions made: Cleaned a stale `FieldRenderer` comment and removed scratch comments from the new snippet collision tests.
- Spec deviations: none
- Tradeoffs accepted: none
- Assumptions: none
- Follow-ups for human: none
- Test evidence: Comment-only cleanup verified by the Phase 3 targeted suite, lint, and build checks.
