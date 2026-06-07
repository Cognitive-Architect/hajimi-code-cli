# APPJS-DEMOLITION-SAMPLING-V3X

Task: STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day 3-A

Scope: app.js readonly sampling plus minimum app/controllers/services/views skeleton landing.

Commit: NO

Push: NO

## Baseline

| Item | Value |
| --- | --- |
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD | `9b112ff1ba5d1f0d298252c331baef6a51f1c55b` |
| Pull result | `Already up to date.` |
| app.js current line count | 5084 |
| app.js raw LF line count | 5619 |
| app.js modified this pass | NO |
| old dirty files staged | NO |

Line count note: the operative baseline remains the PowerShell count requested by the task, `5084`. A raw LF scan reports `5619`, and `rg` line pointers below follow the raw LF/rg numbering.

## Skeleton Landed

Created placeholder-only files. They are not wired into `index.html` or `app.js`.

### `src/interface/web/app/`

- `app-state.js`
- `bootstrap.js`
- `app-shell.js`
- `event-bus.js`

### `src/interface/web/controllers/`

- `chat-controller.js`
- `command-controller.js`
- `provider-controller.js`
- `settings-controller.js`
- `session-controller.js`
- `inspector-controller.js`
- `model-picker-controller.js`
- `workspace-controller.js`

### `src/interface/web/services/`

- `tauri-service.js`
- `storage-service.js`
- `provider-service.js`
- `chat-service.js`

### `src/interface/web/views/`

- `chat-view.js`
- `command-palette-view.js`
- `session-list-view.js`
- `settings-view.js`
- `inspector-view.js`
- `model-picker-view.js`
- `topbar-view.js`
- `dashboard-view.js`

Each skeleton file contains only a CommonJS-safe placeholder object so Node syntax checks can cover the new directories without changing browser behavior.

## Current app.js Shape

`src/interface/web/app.js` is still a single `window.app = { ... }` object, not a class. It has no `constructor`; state initialization happens through object literal fields near the top, and bootstrap happens through `init()` plus `app.init()` at the bottom.

| Area | Evidence pointer | Current responsibility | Suggested ownership |
| --- | --- | --- | --- |
| state object | `app.js:5` | tabs, sidebar, provider, chat, session, trace, extensions, settings state | `app/app-state.js` |
| init/bootstrap | `app.js:51-118`, `app.js:5618` | calls all setup/load/render functions and builds command list | `app/bootstrap.js` + `app/app-shell.js` |
| Tauri bridge helpers | `app.js:144-175` | `getTauriBridge`, `isTauriAvailable`, `getTauriInvoke`, `invokeTauri`, tool execution helper | `services/tauri-service.js` |
| shell command helper | `app.js:184-205` | shell tool name, shell quoting, governed command execution | HIGH RISK: leave in app.js until shell boundary task |
| live shell/topbar render | `app.js:206-451` | topbar, live shell status, sidebar summaries, inspector summary render | `views/topbar-view.js`, `views/inspector-view.js` |
| activity/settings/sidebar wrappers | `app.js:454-475` | activity bar click routing and wrappers to `HajimiSettingsPanel` | `controllers/settings-controller.js` + `views/settings-view.js` |
| inspector wrappers | `app.js:492-557` | wrappers to `HajimiInspector` module | `controllers/inspector-controller.js` + `views/inspector-view.js` |
| search/git/file tree/editor | `app.js:561-1604` | search, git panel, file tree, tabs, editor and file operations | `controllers/workspace-controller.js`; split later |
| terminal/output/trace/layout/settings | `app.js:1648-2132` | terminal, output, trace tabs, layout persistence, settings storage | mixed; split later, shell/terminal is high risk |
| chat context/token budget | `app.js:2133-2340` | context files, token stats, auto compact prompt building | `services/chat-service.js` + `controllers/chat-controller.js` |
| chat setup | `app.js:2341-2423` | input event binding, slash palette integration, send trigger | `controllers/chat-controller.js` |
| slash command catalog fallback | `app.js:2424-2442` | inline fallback when catalog module missing | keep near chat until compatibility can be proven |
| sendChatMessage | `app.js:2443-2571` | message submit orchestration, provider check, session save, stream entry | `controllers/chat-controller.js` but HIGH RISK |
| handleChatCommand | `app.js:2572-2876` | slash command execution branches | HIGH RISK: leave in app.js until slash branch smoke |
| streaming helpers | `app.js:2877-3108` | thinking/stream parse and assistant turn rendering helpers | `services/chat-service.js` + `views/chat-view.js`, but move after stream tests |
| invokeAgentTask | `app.js:3101-3171` | Agent task start and channel setup | HIGH RISK: Agent streaming boundary |
| handleAgentEvent | `app.js:3172-3274` | Agent streaming event handling, trace updates, session render | HIGH RISK: Agent streaming boundary |
| streamChat | `app.js:3275-3457` | model stream channel, diagnostics, DOM updates, final content | HIGH RISK: Agent/model streaming boundary |
| chat render helpers | `app.js:3458-3579` | demo response, chat message DOM, thinking blocks, operation summary | `views/chat-view.js` |
| sessions wrappers | `app.js:3584-3610` | wrappers to `HajimiSessions` module | `controllers/session-controller.js` + `views/session-list-view.js` |
| providers/model picker | `app.js:3611-3760` | provider loading, active model button, model picker UI | `controllers/model-picker-controller.js`, `views/model-picker-view.js`, `services/provider-service.js` |
| setupProviderSettings | `app.js:3761-4031` | provider modal events, backup events, key toggle, capacity probe handler | HIGH RISK: Provider/Keyring/probe boundary |
| openProviderModal | `app.js:4032-4091` | modal field population and probe status read | HIGH RISK: Provider/Keyring/probe boundary |
| provider save/delete/backup | `app.js:4158-4298` | backup import/export, save/update/delete provider config | HIGH RISK: Provider/Keyring boundary |
| profile/agent provider/MCP/extensions | `app.js:4299-4616` | profile management, agent provider binding, MCP, extension list | split later; profile/provider/MCP need separate sampling |
| LSP/audit/governance/checkpoint | `app.js:4617-5025` | LSP tooltip, audit wrappers, governance approval, checkpoint restore/export/compare/replay | governance/checkpoint HIGH RISK |
| resource dashboard wrapper | `app.js:5026-5029`, `app.js:5481-5485` | wrappers to `HajimiResourceDashboard` | `controllers/dashboard-controller.js` + `views/dashboard-view.js` |
| command palette | `app.js:5107-5181` | palette open/close/filter/render/selection | `controllers/command-controller.js` + `views/command-palette-view.js` |
| keyboard shortcuts | `app.js:5182-5238` | global shortcuts to palette/sidebar | `controllers/command-controller.js` or `app/app-shell.js` |
| status/edit history/replay/timeline | `app.js:5239-5480` | status bar, inline edit panel, edit history, replay timeline | split later; edit apply path needs separate sampling |
| status indicator/spinner/zombie buttons | `app.js:5486-5617` | elapsed formatting, spinner, shimmer, status indicator, zombie provider test button | `views/topbar-view.js` + provider risk review |

## Required Hotspot Sampling

| Function / area | Evidence pointer | Suggested owner | Risk |
| --- | --- | --- | --- |
| constructor | N/A | N/A | `app.js` is object literal, no constructor found |
| init / bootstrap | `app.js:51-118`, `app.js:5618` | `app/bootstrap.js`, `app/app-shell.js` | Medium; high fan-out |
| handleChatCommand(text) | `app.js:2572-2876` | later `controllers/chat-controller.js` | High; slash command execution branches |
| sendChatMessage() | `app.js:2443-2571` | later `controllers/chat-controller.js` | High; message state, streaming, provider checks |
| streamChat(provider, prompt, config, messages) | `app.js:3275-3457` | later `services/chat-service.js` | High; Agent/model streaming path |
| setupProviderSettings() | `app.js:3761-4031` | later `controllers/provider-controller.js` | High; Provider/Keyring/probe UI |
| setupKeyboardShortcuts() | `app.js:5182-5238` | `controllers/command-controller.js` or `app/app-shell.js` | Medium; global shortcuts |
| openProviderModal(config) | `app.js:4032-4091` | later `views/model-picker-view.js` + `controllers/provider-controller.js` | High; Provider/probe status |
| handleAgentEvent(turn, event, statusHistory) | `app.js:3172-3274` | later `controllers/chat-controller.js` or `services/chat-service.js` | High; Agent streaming |
| Command Palette calls | `app.js:5107-5181` | `controllers/command-controller.js`, `views/command-palette-view.js` | Low/Medium; existing node and WebView smoke exists |
| Sessions calls | `app.js:3584-3610` | `controllers/session-controller.js`, `views/session-list-view.js` | Low; already wrapper to module |
| Settings calls | `app.js:464-475` | `controllers/settings-controller.js`, `views/settings-view.js` | Low; already wrapper to module |
| Inspector calls | `app.js:492-557`, `app.js:4137-4155` | `controllers/inspector-controller.js`, `views/inspector-view.js` | Low/Medium; already wrapper to module |
| Dashboard calls | `app.js:5026-5029`, `app.js:5481-5485` | `views/dashboard-view.js` | Low; already wrapper to module |

## First Safe Move Candidates

These are candidates for the next implementation slice, not moved in Day 3-A:

1. Command Palette DOM functions: `setupCommandPalette`, `showCommandPalette`, `hideCommandPalette`, `renderCommandList`, `navigateCommandList`, `executeSelectedCommand`.
   - Why: bounded DOM IDs, existing catalog smoke, Node DOM smoke, and WebView smoke.
   - Target: `controllers/command-controller.js` plus `views/command-palette-view.js`.

2. Settings/sidebar wrappers: `showSidebar`, `setupSettingsTabs`, `switchSettingsTab`.
   - Why: already delegate to `window.HajimiSettingsPanel`.
   - Target: `controllers/settings-controller.js`.

3. Sessions wrappers: `newChatSession`, `loadChatSessions`, `saveChatSessions`, `switchSession`, `renderChatMessages`, `renderSessionList`.
   - Why: already delegate to `window.HajimiSessions`.
   - Target: `controllers/session-controller.js` and `views/session-list-view.js`.

4. Resource dashboard wrappers: `setupResourceDashboard`, `updateMetrics`.
   - Why: already delegate to `window.HajimiResourceDashboard`.
   - Target: `views/dashboard-view.js` or a future `controllers/dashboard-controller.js`.

5. Inspector wrappers only: `setupInspector`, `showInspectorTab`, `safeRender*`, `render*` wrapper methods.
   - Why: already delegate to `window.HajimiInspector`.
   - Target: `controllers/inspector-controller.js` and `views/inspector-view.js`.

## Do Not Move Yet / High Risk

- Provider save/update/delete/import/export/probe: `setupProviderSettings`, `openProviderModal`, `saveProviderConfig`, `deleteProviderConfig`, backup functions.
- Provider/Keyring data semantics and API key handling.
- Shell helpers and terminal command execution.
- Checkpoint restore/export/compare/replay.
- Agent streaming and model streaming: `sendChatMessage`, `handleChatCommand`, `invokeAgentTask`, `handleAgentEvent`, `streamChat`.
- CSP / withGlobalTauri / Tauri bridge behavior.
- `main.rs` or backend Tauri commands.

## Day 3-A Non-Actions

- Did not modify `src/interface/web/app.js`.
- Did not modify `src/interface/web/index.html`.
- Did not modify `src/interface/web/modules/**`.
- Did not modify `src/interface/web/styles/**`.
- Did not modify `src/interface/desktop/src/main.rs`.
- Did not touch Provider / Keyring / Shell / Checkpoint / CSP / withGlobalTauri / Agent streaming behavior.
- Did not commit.
- Did not push.

## Validation Notes

| Command / Check | Result | Note |
| --- | --- | --- |
| `node --check src/interface/web/app.js` | PASS | no syntax output |
| `node --check src/interface/web/app/*.js` | Windows wildcard caveat | PowerShell invocation passed `*.js` literally to Node, causing `MODULE_NOT_FOUND`; expanded per-file check passed |
| `node --check src/interface/web/controllers/*.js` | Windows wildcard caveat | PowerShell invocation passed `*.js` literally to Node, causing `MODULE_NOT_FOUND`; expanded per-file check passed |
| `node --check src/interface/web/services/*.js` | Windows wildcard caveat | PowerShell invocation passed `*.js` literally to Node, causing `MODULE_NOT_FOUND`; expanded per-file check passed |
| `node --check src/interface/web/views/*.js` | Windows wildcard caveat | PowerShell invocation passed `*.js` literally to Node, causing `MODULE_NOT_FOUND`; expanded per-file check passed |
| Expanded app skeleton check | PASS | `app_expanded_check_failed=0` |
| Expanded controllers skeleton check | PASS | `controllers_expanded_check_failed=0` |
| Expanded services skeleton check | PASS | `services_expanded_check_failed=0` |
| Expanded views skeleton check | PASS | `views_expanded_check_failed=0` |

## Next Suggested Slice

Day 3-B should move only Command Palette functions first, because this area has the strongest existing evidence from catalog smoke, Node DOM smoke, and WebView smoke. Keep inline wrappers in `app.js` until equivalence is proven.
