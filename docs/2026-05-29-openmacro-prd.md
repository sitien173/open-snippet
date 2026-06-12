# openmacro — Product Requirements Document

- **Author:** product lead
- **Date:** 2026-05-29
- **Status:** Draft v1, MVP scope locked
- **Related:** [`2026-05-29-openmacro-design.md`](./2026-05-29-openmacro-design.md) (technical design)

---

## 1. Summary

**openmacro** is a Windows desktop app that lets users define abbreviations which expand into longer text — plain prose, multi-line code, or dynamic templates filled in via a popup form — anywhere they type. Inspired by espanso, TextExpander, aText. Differentiator: a first-class **dynamic forms** feature for parameterized snippets, shipped on day one.

## 2. Problem statement

People retype the same content dozens of times a day across email, chat, IDEs, terminals, and browsers:
- Email signatures, canned replies, support macros.
- Boilerplate code (loops, license headers, log lines).
- Templated messages with one or two variable fields (e.g. "Hi {name}, your ticket {id} is …").
- Shell-output snippets (current git SHA, today's date).

Existing tools fall into two camps. Lightweight expanders (espanso, AHK) require text-file editing and lack good UX for forms. Polished tools (TextExpander) are subscription-only and weak on developer workflows (no shell-command snippets, no git-syncable plain-text storage). **openmacro targets the developer-leaning power user who wants TextExpander UX with espanso's openness.**

## 3. Goals & non-goals

### MVP goals

1. Type an abbreviation in any focused Windows app → silent inline replacement with the expanded text.
2. Support multi-line text, code blocks, a cursor-placement token, date/clipboard variables, shell-command output, and **dynamic forms** that prompt for field values before expansion.
3. Plain-text YAML storage the user can hand-edit, version-control, or share.
4. Cloud sync via a git remote of the user's choice (GitHub, Gitea, self-hosted).
5. Tray icon, autostart with Windows, pause/resume hotkey.
6. Be safe by default: never expand inside password managers or the UAC secure desktop; shell snippets disabled until explicitly enabled.

### Non-goals (MVP)

- macOS / Linux support.
- Per-application allow/deny rules (deferred to v1.1).
- Regex triggers (literal strings only).
- First-party hosted sync backend.
- Team / org features, sharing, marketplace.
- Mobile companion.

### Success criteria

- A new user installs, defines their first snippet, and successfully expands it in Notepad within **5 minutes**.
- End-to-end expansion latency from final keypress to inserted text **< 50 ms** for static snippets, **< 150 ms** including clipboard-paste settle.
- Crash-free sessions ≥ **99.5 %** in beta.
- **< 1 %** of expansions land in the wrong window or mangle the user's typing in week-1 dogfood.

## 4. Target users & personas

- **Developer Dana.** Lives in IDE + terminal + Slack. Wants `;log` → `console.log("$|$");`, `;sha` → current git HEAD, `;pr` → templated PR description form. Comfortable editing YAML, has a GitHub account, wants snippets synced across her two machines via git.
- **Support agent Sam.** Lives in helpdesk + email. Wants `;refund` → a multi-paragraph reply with a form prompting for order ID and amount. Doesn't know what YAML is — uses the settings UI exclusively.

Both personas are served by the same MVP; Dana exercises the developer-leaning features (shell snippets, git sync, cursor token), Sam exercises the forms-first UX.

## 5. User stories

### P0 — must ship in MVP

1. As a user, I can define a snippet that expands a plain-text abbreviation into static multi-line text in any focused Windows app.
2. As a user, I can place a `$|$` token in my snippet so my cursor lands at a marked position after expansion.
3. As a user, I can insert `{{date}}`, `{{time}}`, `{{clipboard}}`, and `{{cursor}}` variables that resolve at expansion time.
4. As a user, I can define a snippet whose expansion is the stdout of a shell command (e.g. `git rev-parse HEAD`), with a hard timeout and first-run consent.
5. As a user, I can define a snippet with form fields; typing the trigger opens a popup, I fill the fields, and on submit the values are inserted into the snippet at the right placeholders.
6. As a user, I can pause and resume expansions globally via a hotkey or tray menu.
7. As a user, I can have openmacro start with Windows.
8. As a user, I can sync my snippets across machines by pointing the app at a git remote I own.
9. As a user, I can be confident the app does not record or expand inside known password managers or UAC prompts.
10. As a user, I can edit my snippets in either the in-app UI or by hand-editing YAML files; both stay in sync.

### P1 — nice to have, fits if cheap

11. As a user, I can see a clear error badge in the settings UI when a YAML file fails to parse.
12. As a user, I can browse a starter pack of example snippets that ship with the app.
13. As a user, I can choose between text / textarea / dropdown (`choice`) / number field types in forms.

### P2 — explicitly deferred

14. Per-app allow/deny rules.
15. Regex triggers.
16. Cross-platform (macOS, Linux).
17. First-party hosted sync.

## 6. Functional requirements

### 6.1 Trigger & expansion

- Triggers are literal strings, case-sensitive, max 32 chars.
- Triggers fire on a word boundary: start of input, whitespace, or punctuation immediately precedes the trigger; the printable char following the trigger fires the match.
- Longest matching trigger wins on ambiguity (`;sig` vs `;signature`).
- Backspace pops one char from the typed buffer.
- Buffer is reset on: focus change, mouse click, arrow keys, Home/End/PageUp/Down, IME composition, Caps Lock toggle, dead key.

### 6.2 Snippet content

- Plain UTF-8 text, any length up to 32 KB (configurable cap).
- Newlines preserved.
- Exactly one `$|$` cursor token per snippet (zero allowed; two or more = validation error).
- Placeholders `{{name}}` reference entries in the snippet's `vars` array.

### 6.3 Variables

| Type | Behavior |
|---|---|
| `datetime` | Resolved at expansion. Optional `format:` strftime string. |
| `clipboard` | Current clipboard text at expansion time. Empty if non-text. |
| `cursor` | Synonym for `$|$` available as a named var. |
| `shell` | argv array spawned with timeout. stdout trimmed, UTF-8. Disabled until user opts in. |
| `form` | Prompts user via popup form before expansion. |

### 6.4 Forms

- Popup window opens when any form var exists in the triggered snippet.
- Fields rendered in declared order.
- Field types: `text`, `textarea`, `choice` (with `options: []`), `number`. Optional `label`, `default`, `required` (default true).
- Submit on Enter (Shift+Enter inserts newline in textarea); Cancel on Esc.
- On cancel: no backspacing, no insertion, literal trigger remains as the user typed it.
- On submit: previously focused window is reactivated, then standard expansion proceeds.

### 6.5 Storage

- Default folder: `%APPDATA%\openmacro\snippets\`.
- All `.yaml` files loaded recursively; folders are organizational only.
- File watcher reloads on change (debounced 200 ms).
- Broken file: logged, badged in UI, other files keep working.
- Settings UI edits write back to the same YAML files (round-trip preserving comments is a stretch goal; baseline is "edits via UI may reorder keys").

### 6.6 Sync

- User opts in by configuring a git remote URL + auth (HTTPS PAT stored in Windows Credential Manager, or SSH via local `ssh-agent`).
- Only the `sync/` subfolder is pushed; rest of `snippets/` stays local.
- Sync runs every 60 s and on debounced local change.
- Conflicts: rebase abort, files copied to `sync/.conflicts/<timestamp>/`, tray notification with one-click "open conflicts folder."

### 6.7 OS integration

- System tray icon always present while running.
- Tray menu: Pause/Resume · Reload snippets · Open snippets folder · Open settings · Quit.
- Global pause hotkey: default `Ctrl+Alt+Pause`, user-configurable.
- Autostart toggle in settings; implemented via `HKCU\…\Run`.
- Single-instance lock; launching a second instance focuses the existing tray.

### 6.8 Safety

- Hardcoded denylist of foreground processes that suppress buffer accumulation: 1Password, KeePass, Bitwarden, LastPass, Windows UAC consent UI.
- Max expansion length cap: 32 KB default, configurable up to 1 MB.
- Shell snippets disabled at install. First time a snippet of `type: shell` is loaded, a one-shot dialog asks for consent. Opt-out at any time in settings disables all shell snippets immediately.
- Per-snippet `confirm: true` to require yes/no toast before each shell run.

## 7. Non-functional requirements

| Area | Requirement |
|---|---|
| Latency | < 50 ms keypress-to-injection for static snippets; < 150 ms with clipboard paste settle. |
| Memory | Idle RSS < 80 MB with webview hidden, < 200 MB with settings open. |
| CPU | < 1 % average on idle hook (modern desktop). |
| Reliability | No crash from a malformed YAML file. No crash from a misbehaving snippet (timeout-protected). |
| Security | No outbound network traffic except git sync. PATs never written to disk in plaintext. Shell snippets sandboxed by argv-only invocation, mandatory timeout, restricted env. |
| Privacy | Keystroke buffer never persisted to disk. No telemetry in MVP (opt-in crash reporting in v1.1). |
| Compatibility | Windows 10 1809+ and Windows 11. Both x64 and ARM64 builds. |
| Accessibility | Settings UI: keyboard-navigable, screen-reader labels on form fields, WCAG AA contrast. |
| Distribution | Signed MSI installer (post-MVP can add MSIX/Store). Portable zip build for power users. |

## 8. UX flows

### 8.1 First-run

1. User runs installer → app launches, tray icon appears, balloon: "openmacro is running. Click here to add your first snippet."
2. Settings window opens to a starter snippet (`;hello` → "Hello, world!").
3. User edits → saves → app shows a "Try it now" prompt with a textarea where they can test the expansion locally.

### 8.2 Defining a snippet (UI path)

1. Settings → New snippet → choose template (Static / With form / With shell command).
2. Editor with trigger field, body (Monaco-style), and a variables panel.
3. Save writes YAML to the chosen file (default `personal.yaml`).

### 8.3 Defining a snippet (YAML path)

1. Tray → Open snippets folder → edit YAML in user's editor.
2. App's file watcher reloads within 200 ms; new trigger is live without restart.

### 8.4 Expansion (static)

1. User types `;sig` in any app.
2. App backspaces 4 chars, pastes the signature, restores prior clipboard. Total < 150 ms.

### 8.5 Expansion (form)

1. User types `;pr` in any app.
2. Popup opens centered on the current monitor, labeled "PR description."
3. User fills "Title" and "Ticket #", presses Enter.
4. Popup closes, previously focused window is reactivated, backspaces and paste run, caret lands at `$|$`.

### 8.6 Sync setup

1. Settings → Sync → "Connect a git remote."
2. User pastes URL, picks auth (HTTPS PAT or SSH agent).
3. App clones into `%APPDATA%\openmacro\sync\`, surfaces success.
4. First push happens on next change or within 60 s.

## 9. Out of scope (with rationale)

- **Per-app rules** — valuable but adds significant UI + foreground-process detection complexity; defer to v1.1 once core expansion is proven.
- **Regex triggers** — power-user feature, niche; literal triggers cover 95 % of use cases and make matching cheap.
- **Cross-platform** — Windows-only buys focus and lets us use the best-fit Win32 APIs; Tauri stack leaves the door open.
- **Hosted sync** — git-backed sync delivers the user value without operating a backend.

## 10. Risks & open questions

| Risk | Mitigation |
|---|---|
| **SmartScreen / AV flags** the binary because of global keyboard hook + SendInput. | Buy EV code-signing cert before public release (~$300/yr). Pre-submit to Microsoft Defender + major AV vendors. Document the SmartScreen click-through path for early users. |
| **Exclusive-fullscreen games / anti-cheat** filter SendInput. | Acceptable MVP gap. Document in release notes. |
| **Clipboard-paste injection** mangles formatted clipboards in apps that do rich paste. | Save + restore the full clipboard contents (all formats), not just text. Document edge cases. |
| **Git sync conflict UX** is confusing for non-developer users. | Sam-persona path: settings UI offers cloud-drive piggyback option in v1.1 for users who don't want git. |
| **Form popup steals focus on wrong monitor / wrong window.** | Capture HWND at trigger time, restore explicitly. Multi-monitor unit-tested. |

## 11. Release plan

| Milestone | Scope | Target |
|---|---|---|
| **M1 — Internal alpha** | Phases 1–4 of the design (skeleton, storage+matcher, hook+injection, placeholders). Author can dogfood static snippets + date/clipboard vars. | Week 6 |
| **M2 — Closed beta** | Phases 5–7 (settings UI, forms, shell snippets). ~20 invited testers. | Week 10 |
| **M3 — Public beta** | Phase 8 (git sync) + crash reporting + signed installer. | Week 14 |
| **M4 — 1.0** | Phase 9 polish, signed MSI on website, basic docs site, no critical open bugs for 2 weeks. | Week 18 |

Phase numbering matches the design doc's milestone list.

## 12. Metrics

Tracked locally only (no telemetry until 1.1 opt-in):

- Expansions per day (per user, local count).
- Form-snippet usage share.
- Sync conflict rate.

Public-facing metrics post-1.0:

- 1.0 download count.
- Crash-free session % from opt-in reports.
- Time-to-first-expansion from install (from anonymized opt-in event).

## 13. Open decisions to confirm before plan execution

1. ~~**Webview UI framework**~~ — **Resolved 2026-05-29:** React 19 + shadcn/ui v4 + Tailwind v4, inherited from the [`dannysmith/tauri-template`](https://github.com/dannysmith/tauri-template) starter. Auto-updates, native menus, prefs system, theme, logging, crash recovery, and i18n come along for the ride.
2. **Installer format** — MSI vs MSIX vs both. Recommend MSI for 1.0, MSIX in 1.1 if Store distribution is targeted.
3. **Telemetry stance for 1.1** — opt-in crash reporting only, or include anonymous usage counts? Recommend crash-only; revisit after public beta feedback.
4. **i18n scope at 1.0** — template ships i18n+RTL. Recommend keeping English-only strings at 1.0 but leaving the i18n plumbing intact so community translations can land later without refactor.
