## Original User Request
openmacro Phase 6 — Form runner (back-side). Rust form state machine: capture foreground HWND, open form window, await values, restore focus, run injection. Cancel = clean state. Multi-monitor positioning.

## Phase
Implement the Rust form runner that activates when a triggered snippet has any `VarKind::Form` declared var (or — more precisely — when any var that is not auto-resolvable; per spec the trigger is "any `type: form` var" which in our `VarKind` enum is `VarKind::Form`). The runner pauses the matcher, captures the foreground HWND, opens the form window (Tauri `WebviewWindowBuilder`) carrying the snippet id, awaits a submit/cancel from the front-end via two new commands, then either (submit) restores focus + sends backspaces + injects the resolved text with values bound; or (cancel) leaves the literal trigger characters in place.

## Tasks
- task-1: **`form::focus`** — `capture_foreground() -> Option<HWND>` and `restore_foreground(hwnd: HWND) -> Result<(), FocusError>`. Restore must handle the `SetForegroundWindow` constraints: call `AllowSetForegroundWindow(ASFW_ANY)` first; on failure, use the `AttachThreadInput` dance (`GetWindowThreadProcessId`, `AttachThreadInput(target, self, true)`, `SetForegroundWindow`, then detach). Log + return `FocusError::Refused` if both paths fail. Wrap raw `HWND` in a `Send`-safe newtype (`#[repr(transparent)] pub struct ForegroundWindow(pub isize)`) so it crosses tokio task boundaries.
- task-2: **`form::runner`** — State machine `FormState = Pending { hwnd: ForegroundWindow, snippet_id: Arc<str> } | Submitted { values: BTreeMap<String,String> } | Cancelled`. Public API:
  - `FormRunner::new(app: AppHandle) -> Self`
  - `async fn run(&self, snippet: &Snippet, hwnd: ForegroundWindow) -> Result<FormOutcome, FormError>` — opens the form window at `/form/<snippet_id>`, parks on a `tokio::sync::oneshot::Receiver<FormState>` until submit or cancel.
  - `fn submit(&self, snippet_id: &str, values: BTreeMap<String,String>)` — fires the oneshot.
  - `fn cancel(&self, snippet_id: &str)` — fires the oneshot with Cancelled.
  - **Re-entrancy**: a second `run` call while a form is open returns `FormError::AlreadyOpen` (no queueing). Use a `Mutex<Option<...>>` for the in-flight form, never block the orchestrator thread (caller awaits the future).
  - **Multi-monitor**: compute target monitor via `MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)`; position the form window centred on that monitor by getting its work area via `GetMonitorInfoW`. Window: `decorations(false)`, `always_on_top(true)`, `skip_taskbar(true)`, `inner_size(400, 240)` (height auto-grown by front-end in Phase 7).
- task-3: **Orchestrator branch** — In `engine::orchestrator::handle_event`, after `Resolver::resolve` succeeds and **before** injecting, check `snippet.vars.iter().any(|v| v.kind == VarKind::Form)`. If true:
  1. capture HWND
  2. `tokio::spawn` (orchestrator gets a `tokio::runtime::Handle` field) the `FormRunner::run` call
  3. on `FormOutcome::Submitted(values)`: re-resolve the snippet's `replace` with `values` overlay on the resolver's name lookup (form vars resolve to their submitted text); restore HWND; inject as before
  4. on `FormOutcome::Cancelled`: do **nothing** — buffer was reset on initial match, literal trigger characters are still in the focused window (since we never sent backspaces before opening the form, per spec)
  5. Add `FormVars` overlay to the resolver: extend `Resolver::resolve` to take an optional `&BTreeMap<String,String>` and check it before the `vars.iter().find(...)` fallback chain.
  Inject path stays the same. **Important**: do not send backspaces until after the form submits — spec is explicit that the literal trigger remains if cancelled.
- task-4: **Commands + tests** — Add to `src-tauri/src/commands/mod.rs`: `form_submit(snippet_id, values: BTreeMap<String,String>) -> Result<(), String>`, `form_cancel(snippet_id) -> Result<(), String>`. They forward to a `tauri::State<Arc<FormRunner>>`. Register in `lib.rs::generate_handler!`. Add `tests/form_focus.rs`:
  - Mock the focus surface behind a `FocusBackend` trait so the test doesn't need real HWNDs.
  - `cargo test form_focus` covers: HWND capture returns the registered value; restore is called with the captured value on submit; restore is NOT called on cancel (literal trigger preserved); re-entrancy rejects second run with `AlreadyOpen`; form-vars overlay produces correct text when piped through the resolver. **TDD-first**: tests fail before implementation, record RED→GREEN in `notes.md`.

## Context
- `VarKind::Form` already exists in `store::model`.
- `engine::orchestrator` currently does inject directly on `Resolver::resolve` success — refactor to branch.
- Tokio runtime: orchestrator currently has no runtime handle. Add `runtime: tokio::runtime::Handle` as a constructor parameter (default in tests can use `Handle::current()` inside a `#[tokio::test]`).
- `windows` crate already in Cargo.toml. Add features as needed (`Win32_Graphics_Gdi` for `MonitorFromWindow` / `GetMonitorInfoW`).
- Window URL: Tauri 2 uses `WebviewUrl::App(format!("/form/{snippet_id}").into())`. The actual front-end at `/form/<id>` is Phase 7 — for Phase 6 a stub HTML page is fine (workers must verify the URL resolves; Phase 7 will replace with a real route). If the route doesn't exist yet, the test path doesn't need a real window — gate window creation behind a `WindowSink` trait that the test stubs.
- Clippy: `-D warnings` still enforced.

## Files
- `F:/projects_new/textblaze/openmacro/src-tauri/src/form/{mod.rs,runner.rs,focus.rs}` (mod.rs exists as stub, fill in)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/engine/orchestrator.rs` (modify)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/expand/resolver.rs` (extend `Resolver::resolve` with form-vars overlay)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/commands/{mod.rs,form.rs}` (add form.rs)
- `F:/projects_new/textblaze/openmacro/src-tauri/src/lib.rs` (register form commands + manage `Arc<FormRunner>`)
- `F:/projects_new/textblaze/openmacro/src-tauri/Cargo.toml` (window features if needed)
- `F:/projects_new/textblaze/openmacro/src-tauri/tests/form_focus.rs` (create)

## Done When
- `cargo test --all-features` green (including new `form_focus` test)
- `cargo clippy --all-targets --all-features -- -D warnings` clean
- Re-entrancy rejected with `AlreadyOpen` (tested)
- Cancel path injects nothing and does not call restore (tested)
- Form-vars overlay produces correct expanded text (tested via resolver unit test)

## Rules
Contract: `F:/projects_new/textblaze/.agents/shared/worker-contract.md`. Notes/journal under `F:/projects_new/textblaze/docs/plans/2026-05-29-openmacro/phase-06/`. TDD per `test-driven-development` — failing test first for every behavior-adding task.

## Response Format
`F:/projects_new/textblaze/.agents/shared/erp.md`.
