# Repository Guidelines

## Project Structure & Module Organization

This Tauri desktop app is named `openmacro`. The React/Vite frontend lives in `src/`, with helpers in `src/lib/`, route UI in `src/routes/`, and frontend tests in nearby `__tests__/` directories. Global styles are in `src/App.css` and `src/colors_and_type.css`.

The Rust backend lives in `src-tauri/`. Key modules include `commands/` for Tauri IPC, `matcher/` for triggers, `expand/` for snippet expansion, `store/` for YAML storage, `hook/` and `inject/` for Windows input, `form/` for workflows, and `sync/` for Git-backed sync. Rust integration tests are in `src-tauri/tests/`. Docs are in `docs/`; sample snippets are in `snippets/default.yaml`.

## Build, Test, and Development Commands

Use pnpm for frontend commands:

- `pnpm install` installs dependencies from `pnpm-lock.yaml`.
- `pnpm dev` starts the Vite frontend.
- `pnpm tauri dev` runs the full Tauri desktop app locally.
- `pnpm build` type-checks with `tsc` and builds the frontend.
- `pnpm test` runs Vitest once; `pnpm test:watch` watches.
- `pnpm lint` runs ESLint against `src/`.
- `cd src-tauri && cargo test` runs the Rust test suite.
- `cd src-tauri && cargo fmt` formats Rust code before review.

## Coding Style & Naming Conventions

Frontend code uses TypeScript, React, and ES modules. Keep component filenames in PascalCase, such as `SnippetEditor.tsx`, and helper modules in lower camel or domain names, such as `snippets.ts`. Place route-specific components under the owning route folder. Follow existing two-space indentation in TypeScript and CSS.

Rust follows `rustfmt`, snake_case modules, and focused modules grouped by backend capability. Keep Tauri command handlers in `src-tauri/src/commands/`.

## Testing Guidelines

Vitest uses `jsdom` with setup in `src/test-setup.ts`. Name frontend tests `*.test.ts` or `*.test.tsx`; accessibility tests use `*.a11y.test.tsx`. Keep tests near the component or helper they cover.

Rust integration tests are named by behavior in `src-tauri/tests/`, for example `matcher_basic.rs` or `sync_roundtrip.rs`. Add tests when changing matcher behavior, snippet expansion, IPC contracts, sync, or forms.

## Commit & Pull Request Guidelines

Recent commits use short subjects with prefixes such as `phase-7:` and `chore:`. Keep subjects specific, for example `chore: update snippet docs` or `phase-8: add shell consent tests`.

Pull requests should describe the change, list test commands run, link related issues or plan docs, and include screenshots for visible UI changes.

## Security & Configuration Tips

Do not commit local credentials, sync remotes, or OS-specific secrets. Treat shell snippet execution, clipboard injection, keyboard hooks, and credential storage as security-sensitive areas; changes there need explicit tests and careful review.
