# STONE-AUDIT-V3X Plan Reality Rebase

Date: 2026-06-12

Source issue: https://github.com/Cognitive-Architect/hajimi-code-cli/issues/9

Task: audit the current V3X Controlled Demolition state against `docs/roadmap/Hajimi ToneFix/plan/new_plan.md`, then regenerate a practical daily plan from repository evidence.

Scope: docs-only reality rebase. This report does not modify production code and does not repair any finding.

## 1. Git Baseline

Commands run:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
git ls-remote origin stone-audit-v3x-controlled-demolition
git pull --ff-only origin stone-audit-v3x-controlled-demolition
git log --oneline -n 20
```

Results:

| Item | Value |
| --- | --- |
| Branch | `stone-audit-v3x-controlled-demolition` |
| Start HEAD | `71a7e2e164a95e7183089753bacc986686937ccb` |
| Remote branch HEAD | `71a7e2e164a95e7183089753bacc986686937ccb` |
| Pull result | `Already up to date.` |
| Report only | YES |
| Production changes | NO |
| Old dirty files staged | NO |

Current `git status --short` included only pre-existing dirty files and this report after creation. The old dirty file inventory was not staged:

```text
 M docs/debt/DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN.md
 M docs/debt/INDEX.md
 M docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md
 D "docs/roadmap/Hajimi Agent/debt/DEBT-DAY-07-CHECKPOINT-DIFF-UI.md"
 D "docs/roadmap/Hajimi Agent/plan/AGENT-UI-INTEGRATION-SAMPLING-NOTES.md"
 D "docs/roadmap/Hajimi LLM/plan/AGENT-LLM-NATIVE-DESIGN.md"
 D "docs/roadmap/Hajimi LLM/plan/LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-01/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-02/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-03/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-04/snapshot.md"
?? docs/debt/DEBT-AGENT-CHINESE-I18N.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-THINKING-LEAK.md
?? docs/debt/DEBT-AGENT-LOOP-LLM-NO-OP.md
?? docs/debt/DEBT-AGENT-UI-INTEGRATION.md
?? docs/debt/DEBT-TAURI-CHANNEL-ENVELOPE-REGRESSION.md
?? docs/debt/STONE-AUDIT-V1.5-PACKAGE-SMOKE.md
?? "docs/roadmap/Hajimi AgentFix/"
?? "docs/roadmap/Hajimi ToneFix/"
?? "docs/roadmap/hajimi template.7z"
?? "docs/roadmap/hajimi template/"
?? src/interface/desktop/native-smoke.txt
```

## 2. File Size / Line Count Reality Check

Commands run with PowerShell equivalents of the issue's `wc -l` and `find ... -exec wc -l` requests.

### Main Target Files

| File | Current lines | Original hard target | Status |
| --- | ---: | ---: | --- |
| `src/interface/web/app.js` | 5047 | <= 1200 | NOT MET |
| `src/interface/web/style.css` | 20 | <= 120 | DONE |
| `src/interface/desktop/src/main.rs` | 4654 | <= 900 | NOT MET |

Day 0 branch baseline was recorded in `docs/debt/STONE-AUDIT-V3X-CONTROLLED-DEMOLITION.md` as:

| File | Day 0 lines | Current lines | Delta |
| --- | ---: | ---: | ---: |
| `src/interface/web/app.js` | 5084 | 5047 | -37 |
| `src/interface/web/style.css` | 3961 | 20 | -3941 |
| `src/interface/desktop/src/main.rs` | 4654 | 4654 | 0 |

### Frontend Controllers

| File | Lines |
| --- | ---: |
| `src/interface/web/controllers/chat-controller.js` | 93 |
| `src/interface/web/controllers/command-controller.js` | 96 |
| `src/interface/web/controllers/dashboard-controller.js` | 32 |
| `src/interface/web/controllers/inspector-controller.js` | 76 |
| `src/interface/web/controllers/model-picker-controller.js` | 35 |
| `src/interface/web/controllers/provider-controller.js` | 6 |
| `src/interface/web/controllers/session-controller.js` | 26 |
| `src/interface/web/controllers/settings-controller.js` | 122 |
| `src/interface/web/controllers/workspace-controller.js` | 6 |

### Frontend Views

| File | Lines |
| --- | ---: |
| `src/interface/web/views/chat-view.js` | 210 |
| `src/interface/web/views/command-palette-view.js` | 73 |
| `src/interface/web/views/dashboard-view.js` | 32 |
| `src/interface/web/views/feedback-view.js` | 21 |
| `src/interface/web/views/inspector-view.js` | 50 |
| `src/interface/web/views/model-picker-view.js` | 71 |
| `src/interface/web/views/session-list-view.js` | 51 |
| `src/interface/web/views/settings-view.js` | 43 |
| `src/interface/web/views/topbar-view.js` | 57 |

### Frontend Services

| File | Lines |
| --- | ---: |
| `src/interface/web/services/chat-service.js` | 6 |
| `src/interface/web/services/markdown-service.js` | 70 |
| `src/interface/web/services/provider-service.js` | 6 |
| `src/interface/web/services/storage-service.js` | 50 |
| `src/interface/web/services/tauri-service.js` | 6 |

### Frontend Modules

| File | Lines |
| --- | ---: |
| `src/interface/web/modules/audit-log.js` | 81 |
| `src/interface/web/modules/command-palette-catalog.js` | 41 |
| `src/interface/web/modules/inspector.js` | 379 |
| `src/interface/web/modules/resource-dashboard.js` | 61 |
| `src/interface/web/modules/security-dom.js` | 29 |
| `src/interface/web/modules/sessions.js` | 144 |
| `src/interface/web/modules/settings-panel.js` | 62 |
| `src/interface/web/modules/slash-command-catalog.js` | 23 |
| `src/interface/web/modules/slash-palette.js` | 241 |
| `src/interface/web/modules/tauri-bridge.js` | 67 |
| `src/interface/web/modules/thinking-ui.js` | 797 |
| `src/interface/web/modules/workspace.js` | 214 |

### Desktop Rust Files

| File | Lines |
| --- | ---: |
| `src/interface/desktop/src/audit.rs` | 91 |
| `src/interface/desktop/src/main.rs` | 4654 |

### CSS Split Files

`src/interface/web/styles` currently contains 16 CSS module files:

```text
base.css
chat.css
command-palette.css
dashboard.css
inspector.css
layout.css
modals.css
model-picker.css
provider.css
sessions.css
settings.css
sidebar.css
slash-palette.css
tokens.css
topbar.css
utilities.css
```

## 3. Original Day0-Day14 Completion Matrix

| Original day | Planned target | Current evidence | Status | Remaining work | Next action |
| --- | --- | --- | --- | --- | --- |
| Day 0 | Backup branch, demolition tag, baseline doc | Tag `stone-v3x-before-demolition` exists; `docs/debt/STONE-AUDIT-V3X-CONTROLLED-DEMOLITION.md` exists; commit `26f250fb` recorded baseline | DONE | None | Keep tag as rollback root |
| Day 1 | CSS full split | `style.css` is 20 lines; 16 `styles/*.css` files; `docs/frontend/CSS-SPLIT-V3X.md`; commit `9b112ff1` | DONE | None | Keep CSS regression in final suite |
| Day 2 | CSS WebView regression | `docs/frontend/CSS-SPLIT-WEBVIEW-SMOKE-V3X.md` records app launch/chat/sidebar/command/settings/inspector/dashboard PASS | DONE | None | Re-run in final full WebView |
| Day 3 | app.js skeleton and app/controllers/services/views | `app/`, `controllers/`, `services/`, `views/` exist; command palette migrated and wired; `app.js` still 5047 lines | PARTIAL | app shell is not yet <=1200; many wrappers/high-risk paths remain | Continue only after Day6 blocker is resolved |
| Day 4 | Command / Session / Settings frontend domains | Command Palette completed; sessions view/storage/button slices completed; settings pure/storage completed; Day4 closure exists | PARTIAL | Settings/Sessions true WebView receipt was deferred; sessions full controller not done | Re-run targeted WebView during app.js closure |
| Day 5 | Chat / Model Picker / UI Feedback | Day5-BE closure and Day5-F WebView receipt exist; feedback/markdown/topbar/chat-view/chat-controller/model-picker slices PASS | PARTIAL | `sendChatMessage`, `streamChat`, `handleAgentEvent`, `invokeAgentTask` remain in `app.js` | Keep streaming paths for later high-risk chat slice |
| Day 6 | Provider / Inspector / Dashboard / Tauri invoke frontend domains | Day6-A sampling done; Day6-B Dashboard DONE; Day6-C Inspector Shell DONE; Day6-D rendering BLOCKED; Day6-E/F not started | BLOCKED | Inspector rendering safe DOM rewrite blocks Provider continuation; Dashboard/Inspector browser wiring debt remains | Start Re-Day 6.1/6.2 safe DOM path |
| Day 7 | app.js closure + WebView regression, app.js <=1200 | Current `app.js` 5047 lines; no closure doc | BLOCKED | Requires Day6 unblocked and further high-risk path extraction | Do not begin until Day6-D blocker resolved |
| Day 8 | main.rs state/registry/startup/commands skeleton | `src/interface/desktop/src` only contains `audit.rs` and `main.rs`; `main.rs` 4654 lines | NOT STARTED | No state/registry/startup/commands skeleton | Start after app.js closure or explicit re-order approval |
| Day 9 | Low-risk Tauri commands | No command-domain files under `src/interface/desktop/src/commands` | NOT STARTED | Low-risk command modules not extracted | Needs Day8 skeleton first |
| Day 10 | High-risk Tauri commands | No provider/checkpoint/shell/agent command modules | NOT STARTED | Provider/Keyring/Shell/Checkpoint command movement not started | Needs Day8/9 plus cargo baseline |
| Day 11 | main.rs closure + desktop WebView regression, main.rs <=900 | `main.rs` 4654 lines; no closure doc | NOT STARTED | `main.rs` far above target; cargo/WebView closure not run | After Day8-10 |
| Day 12 | Repo volume surgery dry run | `git count-objects -vH`: `size-pack: 3.81 GiB`; tracked `models/*`; no V3X dry-run doc found | NOT STARTED | Dry run and migration plan still needed | Prepare docs-only dry run |
| Day 13 | Repo volume surgery execution if approved | No history rewrite, LFS migration, or pack reduction evidence | NOT STARTED | Requires explicit approval after Day12 | Do not execute without fresh backup/approval |
| Day 14 | Full regression + score closure | No final V3X closure or MasterMind v0.7/v0.12 update found | NOT STARTED | Needs frontend/desktop/repo phases complete | Final only after all phases |

## 4. Phase Completion Matrix

| Phase | Original target | Current evidence | Status | Percent estimate | Notes |
| --- | --- | --- | --- | ---: | --- |
| Phase A CSS split | Split `style.css` into `styles/*`, shell <=120 lines | `style.css` 20 lines; 16 CSS files; CSS WebView PASS | DONE | 100% | Meets hard target |
| Phase B Frontend app.js split | Move app.js domains into app/controllers/services/views and reach <=1200 lines | Many domains extracted; `app.js` still 5047 lines; Day6-D blocked | PARTIAL | 45% | Structure exists, but hard line target and high-risk paths remain |
| Phase C Desktop main.rs split | Split state/registry/startup/commands and reach <=900 lines | `main.rs` 4654 lines; no command directory | NOT STARTED | 0% | `audit.rs` exists, but main.rs target untouched |
| Phase D Repo volume surgery | Dry run and optional history cleanup; pack target <1 GiB if rewrite approved | `git count-objects -vH` still `size-pack: 3.81 GiB`; tracked `models/*` | NOT STARTED | 5% | Original problem reconfirmed only |
| Phase E Full regression + closure | Node/cargo/WebView/security closure and score update | Multiple Node/WebView receipts exist; no full cargo/final closure | PARTIAL | 20% | Useful receipts exist, but not final closure |

Overall new_plan completion estimate: about 35%.

Rationale: Phase A is complete, Phase B has meaningful domain extraction but misses the hard app.js line target, and Phases C/D/E are mostly not started.

## 5. Current Completed Inventory

| Area | Evidence | Current state |
| --- | --- | --- |
| CSS split / WebView docs | `docs/frontend/CSS-SPLIT-V3X.md`, `docs/frontend/CSS-SPLIT-WEBVIEW-SMOKE-V3X.md`, commit `9b112ff1` | DONE |
| app skeleton docs | `docs/frontend/APPJS-DEMOLITION-SAMPLING-V3X.md`, commit `fd0ca7f0`; files under `src/interface/web/app/`, `controllers/`, `services/`, `views/` | PARTIAL foundation complete |
| Command Palette split | `docs/frontend/COMMAND-PALETTE-DEMOLITION-V3X.md`, `COMMAND-PALETTE-WEBVIEW-WIRING-V3X.md`, `COMMAND-PALETTE-FALLBACK-REMOVAL-V3X.md`; commits `cf063a6c`, `160f9025`, `f14f42cd` | DONE for command palette |
| Session split | `SESSION-LIST-VIEW-EXTRACTION-V3X.md`, `SESSION-LIST-VIEW-WEBVIEW-WIRING-V3X.md`, `SESSION-BUTTON-WIRING-V3X.md`; commits `fe9e29b9`, `4d1a957e`, `6667d86e` | PARTIAL; core chat/session render remains |
| Settings split | `SETTINGS-PURE-FUNCTIONS-EXTRACTION-V3X.md`, `STORAGE-SERVICE-SETTINGS-ONLY-V3X.md`; commits `86576e94`, `7a4277c9` | DONE for settings pure/storage slice |
| Storage service split | `storage-service.js` has settings and sessions APIs; docs `STORAGE-SERVICE-SETTINGS-ONLY-V3X.md`, `STORAGE-SERVICE-SESSIONS-ONLY-V3X.md` | PARTIAL; other localStorage keys not moved |
| Chat split | `chat-view.js`, `chat-controller.js`, docs `DAY5-C-CHAT-RENDER-VIEW-V3X.md`, `DAY5-D-CHAT-CONTROLLER-WIRING-V3X.md`; commits `05fc2120`, `8be4955a` | PARTIAL; core streaming/send remains in `app.js` |
| Model picker split | `model-picker-view.js`, `model-picker-controller.js`, doc `DAY5-E-MODEL-PICKER-V3X.md`, commit `72357f59` | DONE for model picker UI slice |
| Topbar / UI feedback split | `feedback-view.js`, `markdown-service.js`, `topbar-view.js`; doc `DAY5-B-FEEDBACK-MARKDOWN-TOPBAR-V3X.md`, commit `6a1e8b8f` | DONE for Day5-B slice |
| Day5 WebView receipt | `docs/frontend/DAY5-F-WEBVIEW-VISUAL-RECEIPT-V3X.md`, commit `77584f6b` | PASS WITH KNOWN SECURITY-GATE FAIL |
| Dashboard extraction | `dashboard-view.js`, `dashboard-controller.js`, `resource-dashboard.js`; doc `DAY6-B-RESOURCE-DASHBOARD-EXTRACTION-V3X.md`, commit `c3585896` | DONE extraction; browser script wiring fallback remains |
| Inspector shell extraction | `inspector-view.js`, `inspector-controller.js`, `inspector.js`; doc `DAY6-C-INSPECTOR-SHELL-EXTRACTION-V3X.md`, commit `8672c81e` | DONE shell extraction; rendering remains in module |
| Inspector rendering blocked receipt | `docs/frontend/DAY6-D-INSPECTOR-RENDERING-RECEIPT-EXTRACTION-V3X.md`, commit `71a7e2e1` | BLOCKED |
| Provider sampling / migration status | `docs/frontend/DAY6-PROVIDER-INSPECTOR-DASHBOARD-TAURI-INVOKE-SAMPLING-V3X.md`, commit `8477c2cf`; `provider-controller.js` and `provider-service.js` are skeletons only | Sampling done; migration not started |
| Security-gate known fail state | `npm run test:security-gate`: findings 109, failures 3, warnings 106, allowlisted 106 | KNOWN FAIL only |

Current `index.html` script wiring evidence:

```text
services/storage-service.js
views/session-list-view.js
views/command-palette-view.js
controllers/command-controller.js
views/settings-view.js
controllers/settings-controller.js
views/chat-view.js
controllers/chat-controller.js
views/model-picker-view.js
controllers/model-picker-controller.js
views/feedback-view.js
services/markdown-service.js
views/topbar-view.js
```

Notably absent from current browser script wiring:

```text
views/dashboard-view.js
controllers/dashboard-controller.js
views/inspector-view.js
controllers/inspector-controller.js
services/tauri-service.js
controllers/provider-controller.js
services/provider-service.js
```

## 6. Current Incomplete / Blocked Inventory

### A. Must Unblock Before Provider

- Inspector rendering / Context Receipt migration is BLOCKED by security-gate new failures when legacy `innerHTML` is moved into `inspector-view.js`.
- Required next proof: a safe DOM rewrite or explicitly approved allowlist/debt path. Current recommendation is safe DOM rewrite, not allowlist expansion.
- Dashboard and Inspector shell extractions currently rely on compatibility fallback because browser script tags are not wired for their new controller/view files.

### B. Remaining Frontend app.js Closure Work

- `app.js` is 5047 lines, far above <=1200.
- High-risk paths still in `app.js` include:
  - `sendChatMessage()`
  - `handleChatCommand(text)`
  - `invokeAgentTask()`
  - `handleAgentEvent()`
  - `streamChat()`
  - Provider save/delete/backup/probe/validate/keyring paths
  - checkpoint restore/export/compare/replay paths
  - shell/agent command paths
  - profile/MCP/extensions/edit history/replay/timeline leftovers

### C. Remaining Provider Day6 Work

- Provider read-only smoke not started.
- Provider read-only UI thin slice not started.
- `provider-controller.js` and `provider-service.js` are Day3-A skeletons only.
- Provider save/delete/probe/backup/keyring remain forbidden unless a separate high-risk task explicitly allows them.

### D. Remaining Tauri Invoke Work

- `services/tauri-service.js` is still a Day3-A skeleton only.
- `app.js` still owns `getTauriBridge`, `isTauriAvailable`, `getTauriInvoke`, and `invokeTauri`.
- Multiple high-risk invokes remain in `app.js` for file writes, provider writes, checkpoint actions, shell/agent commands, and streaming.

### E. Remaining WebView / Regression Work

- CSS and Day5 WebView receipts exist.
- Command Palette WebView receipt exists.
- Settings/Sessions Day4 WebView was deferred.
- Dashboard/Inspector Day6 extractions do not yet have browser wiring WebView receipts for the new controller/view path.
- Final Day14 full WebView regression not started.

### F. Remaining main.rs Work

- `main.rs` is still 4654 lines.
- `src/interface/desktop/src/commands/` does not exist.
- `state.rs`, `registry.rs`, `startup.rs`, `error.rs` do not exist.
- No cargo check for a V3X main.rs split has been run because the split has not started.

### G. Remaining Repo Volume Work

- `git count-objects -vH` still reports `size-pack: 3.81 GiB`.
- `models/*` files are tracked:
  - `models/all-MiniLM-L6-v2.tar.gz`
  - `models/fast-all-MiniLM-L6-v2/model.onnx`
  - related tokenizer/config files
- No V3X `filter-repo --analyze` dry-run document was found.
- No LFS/release-asset execution was found.

### H. Remaining Final Closure / Scoring Work

- No V3X final closure document found.
- No MasterMind v0.7/v0.12 final score update found.
- Original hard targets are not met for `app.js`, `main.rs`, or repo pack size.

## 7. Revised Daily Plan

This revised plan does not blindly preserve the original day numbers. It starts from the current actual blocker: Day6-D Inspector rendering security-gate failure.

### Re-Day 6.1: Inspector Safe DOM Rewrite Sampling + Test Plan

- Objective: define the exact Inspector rendering paths that must stop using moved legacy `innerHTML` before any Provider work resumes.
- Allowed files:
  - `docs/frontend/INSPECTOR-SAFE-DOM-REWRITE-PLAN-V3X.md`
  - optional `tests/frontend/day35_inspector_rendering_safe_dom_smoke.js`
- Forbidden files / boundaries:
  - No production code changes
  - No Provider / Keyring / Shell / Checkpoint / Agent streaming / CSP / withGlobalTauri
- Required commands:
  - `node tests/frontend/day18_inspector_smoke.js`
  - `node tests/frontend/day25_inspector_safety_smoke.js`
  - `npm run test:security-gate`
  - `git diff --name-only -- src/interface/web app.js src/interface/desktop/src/main.rs`
- Exit criteria:
  - document identifies each target method and the required safe DOM evidence
  - old dirty files staged: NO
- Stop condition:
  - if the plan requires allowlist expansion as the first option, stop and record BLOCKED
- Expected commit message:
  - `docs(frontend): plan inspector safe dom rewrite`

### Re-Day 6.2: Inspector Safe DOM Rewrite Implementation

- Objective: migrate Inspector diff / Context Receipt rendering using safe DOM construction so `security-gate` does not gain new failures.
- Allowed files:
  - `src/interface/web/modules/inspector.js`
  - `src/interface/web/views/inspector-view.js`
  - `src/interface/web/controllers/inspector-controller.js`
  - `tests/frontend/day18_inspector_smoke.js`
  - `tests/frontend/day25_inspector_safety_smoke.js`
  - optional new day35 smoke
  - `docs/frontend/INSPECTOR-SAFE-DOM-REWRITE-V3X.md`
- Forbidden files / boundaries:
  - No Provider / Keyring
  - No Shell
  - No Checkpoint restore/export/compare/replay
  - No Agent streaming
  - No CSP / withGlobalTauri
  - No security allowlist shortcut unless separately approved
- Required commands:
  - `node --check src/interface/web/modules/inspector.js`
  - `node --check src/interface/web/views/inspector-view.js`
  - `node --check src/interface/web/controllers/inspector-controller.js`
  - `node tests/frontend/day18_inspector_smoke.js`
  - `node tests/frontend/day25_inspector_safety_smoke.js`
  - `npm run test:security-gate`
- Exit criteria:
  - day18/day25 PASS
  - security-gate remains only the known 3 failures or improves
  - no new Inspector view failures
- Stop condition:
  - any new security-gate failure outside the known 3
- Expected commit message:
  - `refactor(frontend): rewrite inspector rendering with safe dom`

### Re-Day 6.3: Inspector Browser Wiring + Compatibility Fallback Removal

- Objective: load `views/inspector-view.js` and `controllers/inspector-controller.js` in real browser order and remove only the now-proven shell/render compatibility fallback.
- Allowed files:
  - `src/interface/web/index.html`
  - `src/interface/web/modules/inspector.js`
  - `docs/frontend/INSPECTOR-WEBVIEW-WIRING-V3X.md`
- Forbidden files / boundaries:
  - No Provider / Shell / Checkpoint / Agent streaming
  - No app.js rewrite
- Required commands:
  - `node --check src/interface/web/modules/inspector.js`
  - `node tests/frontend/day18_inspector_smoke.js`
  - `node tests/frontend/day25_inspector_safety_smoke.js`
  - `npm run test:security-gate`
  - real Tauri WebView Inspector smoke
- Exit criteria:
  - `window.HajimiInspectorView` and `window.HajimiInspectorController` observed in WebView
  - Inspector tabs/diff/receipt render without white screen or new console error
- Stop condition:
  - WebView white screen/crash/no response or security-gate new failure
- Expected commit message:
  - `refactor(frontend): wire inspector view scripts`

### Re-Day 6.4: Dashboard Browser Wiring + Compatibility Fallback Removal

- Objective: load `views/dashboard-view.js` and `controllers/dashboard-controller.js` before `modules/resource-dashboard.js`, then remove only the Dashboard compatibility fallback.
- Allowed files:
  - `src/interface/web/index.html`
  - `src/interface/web/modules/resource-dashboard.js`
  - `tests/frontend/day24_resource_dashboard_smoke.js`
  - `docs/frontend/DASHBOARD-WEBVIEW-WIRING-V3X.md`
- Forbidden files / boundaries:
  - No Provider / Keyring
  - No Checkpoint / Shell / Agent streaming
  - No `get_resource_metrics` backend change
- Required commands:
  - `node --check src/interface/web/modules/resource-dashboard.js`
  - `node --check src/interface/web/views/dashboard-view.js`
  - `node --check src/interface/web/controllers/dashboard-controller.js`
  - `node tests/frontend/day24_resource_dashboard_smoke.js`
  - `npm run test:security-gate`
  - real Tauri WebView Dashboard smoke
- Exit criteria:
  - Dashboard metrics render through new controller/view path
  - known security-gate baseline unchanged
- Stop condition:
  - any checkpoint/provider/agent/shell touch or WebView failure
- Expected commit message:
  - `refactor(frontend): wire dashboard view scripts`

### Re-Day 6.5: Provider Read-only DOM Smoke

- Objective: prove Provider list/modal shell can be inspected read-only without save/delete/probe/keyring execution.
- Allowed files:
  - `tests/frontend/day36_provider_readonly_dom_smoke.js`
  - `docs/frontend/PROVIDER-READONLY-DOM-SMOKE-V3X.md`
- Forbidden files / boundaries:
  - No production code changes
  - No Provider save/delete/probe/validate/backup/keyring execution
  - No Shell / Checkpoint / Agent streaming
- Required commands:
  - `node tests/frontend/day19_settings_smoke.js`
  - `node tests/frontend/day34_model_picker_smoke.js`
  - `node tests/frontend/day36_provider_readonly_dom_smoke.js`
  - `npm run test:security-gate`
- Exit criteria:
  - Provider DOM shell/read-only render evidence exists
  - forbidden Provider commands not called
- Stop condition:
  - smoke needs real keyring/provider write to pass
- Expected commit message:
  - `test(frontend): add provider readonly dom smoke`

### Re-Day 6.6: Provider Read-only UI Thin Slice

- Objective: extract only read-only Provider display helpers after Re-Day 6.5 passes.
- Allowed files:
  - `src/interface/web/controllers/provider-controller.js`
  - `src/interface/web/services/provider-service.js`
  - optional `src/interface/web/views/provider-view.js`
  - provider read-only smoke/docs
- Forbidden files / boundaries:
  - No save/delete/probe/validate/backup/keyring logic movement unless the task explicitly narrows it
  - No backend Tauri command changes
- Required commands:
  - provider read-only smoke
  - `node --check` for touched files
  - `npm run test:security-gate`
  - forbidden diff checks for Shell/Checkpoint/Agent/CSP
- Exit criteria:
  - read-only Provider UI delegates to new module
  - all write/probe/key paths remain in legacy guarded area
- Stop condition:
  - any write command must move for the slice to work
- Expected commit message:
  - `refactor(frontend): extract provider readonly view`

### Re-Day 6.7: Tauri Invoke Service Sampling

- Objective: sample `invokeTauri` usage and identify read-only vs high-risk invoke groups before moving `services/tauri-service.js` beyond skeleton.
- Allowed files:
  - `docs/frontend/TAURI-INVOKE-SERVICE-SAMPLING-V3X.md`
- Forbidden files / boundaries:
  - No production code changes
  - No command execution beyond existing Node checks/security gate
- Required commands:
  - `rg -n "invokeTauri|getTauriInvoke|window.__TAURI__" src/interface/web tests/frontend docs/frontend`
  - `npm run test:security-gate`
- Exit criteria:
  - invoke migration candidates grouped by read-only, risky, forbidden
- Stop condition:
  - any plan requires changing `withGlobalTauri`, CSP, shell, checkpoint, keyring
- Expected commit message:
  - `docs(frontend): sample tauri invoke service migration`

### Re-Day 7.1: app.js Closure Sampling

- Objective: remeasure app.js and identify only proven wrappers/fallbacks that can be deleted after Re-Day 6 wiring.
- Allowed files:
  - `docs/frontend/APPJS-CLOSURE-SAMPLING-V3X.md`
- Forbidden files / boundaries:
  - No production code changes
- Required commands:
  - line counts
  - `rg` for wrappers/fallbacks
  - full frontend Node smoke set from Day14 subset
- Exit criteria:
  - deletion candidate table with rollback points
- Stop condition:
  - any candidate touches Provider writes, Shell, Checkpoint, or Agent streaming without a separate task
- Expected commit message:
  - `docs(frontend): sample appjs closure candidates`

### Re-Day 7.2: app.js Proven Fallback Removal Batches

- Objective: remove only proven compatibility fallbacks after browser wiring receipts exist.
- Allowed files:
  - `src/interface/web/app.js`
  - affected tests/docs only
- Forbidden files / boundaries:
  - No sendChatMessage/streamChat/handleAgentEvent changes unless specifically in scope
  - No Provider/Keyring/Shell/Checkpoint/Agent streaming semantics changes
- Required commands:
  - `node --check src/interface/web/app.js`
  - targeted smoke for each removed fallback
  - `npm run test:security-gate`
  - real WebView smoke for affected UI
- Exit criteria:
  - app.js line count decreases
  - no new security-gate failure
  - no WebView regression
- Stop condition:
  - app.js behavior differs in real WebView
- Expected commit message:
  - `refactor(frontend): remove proven appjs compatibility fallbacks`

### Re-Day 7.3: app.js High-risk Remainder Decision

- Objective: decide whether to proceed into chat streaming/provider/checkpoint/shell extraction or re-scope the hard `app.js <=1200` target.
- Allowed files:
  - `docs/debt/APPJS-HIGH-RISK-REMAINDER-V3X.md`
- Forbidden files / boundaries:
  - No production code changes
- Required commands:
  - line counts
  - `rg` sampling for high-risk functions
  - `npm run test:security-gate`
- Exit criteria:
  - explicit decision: continue extraction, defer, or revise hard target
- Stop condition:
  - no clear safe path to `app.js <=1200`
- Expected commit message:
  - `docs(debt): record appjs high risk remainder`

### Re-Day 8: main.rs Split Sampling + Command Map

- Objective: create a desktop command map and split plan before moving Rust code.
- Allowed files:
  - `docs/debt/MAINRS-SPLIT-SAMPLING-V3X.md`
  - optional `docs/debt/TAURI-COMMAND-MAP-V3X.md`
- Forbidden files / boundaries:
  - No Rust production changes
- Required commands:
  - `rg -n "#\\[tauri::command\\]|invoke_handler|manage\\(" src/interface/desktop/src/main.rs`
  - `cargo check -p hajimi-desktop` if available
- Exit criteria:
  - command grouping and rollback plan exists
- Stop condition:
  - cargo baseline cannot be established
- Expected commit message:
  - `docs(debt): sample mainrs split plan`

### Re-Day 9: main.rs State / Registry / Startup Skeleton

- Objective: create Rust module skeleton and move only low-risk initialization structure.
- Allowed files:
  - `src/interface/desktop/src/main.rs`
  - `src/interface/desktop/src/state.rs`
  - `src/interface/desktop/src/registry.rs`
  - `src/interface/desktop/src/startup.rs`
  - `src/interface/desktop/src/error.rs`
  - docs/tests as needed
- Forbidden files / boundaries:
  - No command semantic changes
  - No shell/provider/checkpoint behavior changes
- Required commands:
  - `cargo check -p hajimi-desktop`
  - `cargo check --workspace`
- Exit criteria:
  - cargo check PASS
  - main.rs line count decreases
- Stop condition:
  - command registry behavior changes or cargo fails
- Expected commit message:
  - `refactor(desktop): split tauri startup skeleton`

### Re-Day 10: Low-risk Tauri Commands

- Objective: move read-only/low-risk commands into `commands/*`.
- Allowed files:
  - `src/interface/desktop/src/commands/*.rs`
  - `src/interface/desktop/src/main.rs`
  - `src/interface/desktop/src/registry.rs`
- Forbidden files / boundaries:
  - No Provider keyring writes
  - No Shell policy changes
  - No Checkpoint restore/export/compare/replay semantic changes
- Required commands:
  - `cargo check -p hajimi-desktop`
  - targeted frontend smoke for audit/dashboard if applicable
- Exit criteria:
  - low-risk commands moved, cargo PASS
- Stop condition:
  - any command must be rewritten rather than moved
- Expected commit message:
  - `refactor(desktop): split low risk tauri commands`

### Re-Day 11: High-risk Tauri Commands

- Objective: move high-risk commands only after low-risk split proves registry stability.
- Allowed files:
  - desktop `commands/provider.rs`, `commands/checkpoint.rs`, `commands/shell.rs`, `commands/chat.rs`, `commands/governance.rs`
- Forbidden files / boundaries:
  - No policy loosening
  - No keyring semantic change
  - No checkpoint semantic change
  - No shell allowlist change
- Required commands:
  - `cargo check --workspace`
  - `npm run test:security-gate`
  - targeted desktop/native smoke if available
- Exit criteria:
  - high-risk commands moved with semantics preserved
  - main.rs approaches <=900
- Stop condition:
  - cargo check failure or any safety policy diff
- Expected commit message:
  - `refactor(desktop): split high risk tauri commands`

### Re-Day 12: Repo Volume Surgery Dry Run

- Objective: analyze tracked large files and produce a no-rewrite cleanup plan.
- Allowed files:
  - `docs/debt/REPO-VOLUME-SURGERY-DRY-RUN-V3X.md`
  - optional raw log under `docs/debt/`
- Forbidden files / boundaries:
  - No history rewrite
  - No large file deletion
  - No `git filter-repo` execution that mutates history
- Required commands:
  - `git count-objects -vH`
  - `git ls-files models target src/interface/desktop/target`
  - `git filter-repo --analyze` only if tool exists and analysis output is safe/approved
- Exit criteria:
  - exact cleanup candidates and risk plan documented
- Stop condition:
  - tool missing or command would mutate history
- Expected commit message:
  - `docs(debt): add repo volume surgery dry run`

### Re-Day 13: Repo Volume Surgery Execution

- Objective: execute only if Re-Day 12 is approved explicitly.
- Allowed files:
  - `.gitattributes`, `.gitignore`, docs, and approved tracked file movements
- Forbidden files / boundaries:
  - No execution without fresh backup branch/tag and written approval
- Required commands:
  - fresh clone validation
  - pack size validation
- Exit criteria:
  - pack size reduced or execution BLOCKED with rollback evidence
- Stop condition:
  - any clone/recovery uncertainty
- Expected commit message:
  - `chore(repo): migrate large artifacts out of git history`

### Re-Day 14: Full Regression + Score Closure

- Objective: close V3X with real evidence.
- Allowed files:
  - `docs/debt/STONE-AUDIT-V3X-CONTROLLED-DEMOLITION-CLOSURE.md`
  - MasterMind / score docs if present and explicitly allowed
- Forbidden files / boundaries:
  - No new production changes
- Required commands:
  - full frontend smoke set
  - `npm run test:security-gate`
  - `cargo check --workspace`
  - real Tauri WebView smoke
  - line counts
  - `git count-objects -vH`
- Exit criteria:
  - app.js/style.css/main.rs/repo-volume/security/WebView/cargo status recorded honestly
- Stop condition:
  - any required final proof cannot be run; mark BLOCKED, not CLEARED
- Expected commit message:
  - `docs(debt): close v3x controlled demolition`

## 8. Stop-loss Recommendations

| Risk | Stop rule | Only allowed next action |
| --- | --- | --- |
| security-gate new failures | If failures exceed the known 3 (`command-palette-view.js:36`, `session-list-view.js:22`, `session-list-view.js:26`) stop | rollback production attempt or commit docs-only BLOCKED receipt |
| Provider forbidden command invocation | If any smoke or migration calls save/delete/probe/validate/backup/keyring commands unintentionally stop | record command path and do not continue Provider slice |
| main.rs command registry breakage | If cargo check fails after moving registry/commands stop | rollback current batch or docs-only blocker |
| cargo check failure | If `cargo check -p hajimi-desktop` or workspace check fails and cannot be attributed to pre-existing baseline stop | no further Rust migration |
| WebView white screen / crash | If real Tauri WebView white screens, crashes, or becomes unresponsive stop | record FAIL and do not patch opportunistically |
| history rewrite risk | If backup branch/tag/fresh clone/recovery plan is missing stop | docs-only dry run |
| old dirty files staged | If unrelated dirty files appear in `git diff --cached --name-only` stop | unstage only those files or abort before commit |

## 9. Validation Commands Run For This Report

| Command | Result |
| --- | --- |
| `node --check src/interface/web/app.js` plus controllers/views/services loop | PASS, `node_check_failed=0` |
| `node tests/frontend/day18_inspector_smoke.js` | PASS |
| `node tests/frontend/day23_audit_log_smoke.js` | PASS, 9 scenarios |
| `node tests/frontend/day24_resource_dashboard_smoke.js` | PASS, 9 scenarios |
| `node tests/frontend/day25_inspector_safety_smoke.js` | PASS, 4 scenarios |
| `node tests/frontend/day34_model_picker_smoke.js` | PASS |
| `npm run test:security-gate` | KNOWN FAIL only: findings 109, failures 3, warnings 106, allowlisted 106 |
| `git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/style.css src/interface/web/modules src/interface/web/controllers src/interface/web/views src/interface/web/services src/interface/desktop/src/main.rs src/interface/desktop/src src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs` | no output before report creation |
| `git diff --cached --check` | PASS before report staging |
| `git diff --check` | PASS with only old dirty docs CRLF warnings before report creation |

Known security-gate failures:

```text
src/interface/web/views/command-palette-view.js:36
src/interface/web/views/session-list-view.js:22
src/interface/web/views/session-list-view.js:26
```

## 10. Final Short Answer

PLAN REALITY REBASE RESULT:

- committed: NO at report write time; this file is the commit object
- pushed: NO at report write time; push must happen after staging this report only
- branch: `stone-audit-v3x-controlled-demolition`
- start HEAD: `71a7e2e164a95e7183089753bacc986686937ccb`
- final HEAD: pending post-commit
- report file: `docs/debt/STONE-AUDIT-V3X-PLAN-REALITY-REBASE.md`
- production changes: NO
- old dirty files staged: NO
- overall new_plan completion estimate: about 35%
- current blocking item: Day6-D Inspector rendering / Context Receipt safe DOM rewrite
- next recommended issue: `STONE-AUDIT-V3X Re-Day 6.1: Inspector Safe DOM Rewrite Sampling + Test Plan`
