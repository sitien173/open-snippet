# Phase 5 Verification Commands

## Backend security boundary regressions
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo test --test commands_roundtrip --test sync_roundtrip --test prefs_roundtrip --test store_yaml -- --test-threads=1"`

## Runtime, form, and injection regressions
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo test --test form_focus --test shell_runner --test notepad_smoke -- --test-threads=1"`

## Frontend privacy and settings regressions
- `rtk pnpm exec vitest run src/lib src/routes/settings --reporter=dot`

## CI parity checks
- `rtk pnpm install --frozen-lockfile`
- `rtk pnpm build`
- `rtk pnpm test`
- `rtk pnpm lint`
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo fmt --check"`
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo test --all-features -- --test-threads=1"`
- `rtk powershell -NoProfile -Command "Set-Location src-tauri; cargo clippy --all-targets --all-features -- -D warnings"`

## Manual-only gated Windows smoke
- `rtk cmd /c "set OPENMACRO_E2E=1&& cargo test --test notepad_smoke -- --ignored --test-threads=1"`
