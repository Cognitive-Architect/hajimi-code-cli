# STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day4-D

## Remaining Frontend Domains Full Sampling

Date: 2026-06-07

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before: `4d1a957efd78284405623f95f7297d640d2a8dd6`

Production changes: NO

Commit: NO

Push: NO

## Git Status Snapshot

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

Old dirty files staged: NO

## Touched Files

- `docs/frontend/DAY4-REMAINING-FRONTEND-DOMAINS-SAMPLING-V3X.md` (this file)

Production files modified: NO

## Forbidden Areas Not Touched

- Provider / Keyring
- Shell
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- Command Palette controller/view
- `src/interface/desktop/src/main.rs`

---

## A. Sessions Remaining Inventory

### Sessions Remaining Function Table

| Function | Location | Role | Calls / Touches | Classification |
| --- | --- | --- | --- | --- |
| `syncActiveSession(app)` | `modules/sessions.js:26` | Copy chatMessages into active session object | Reads/writes `chatMessages`, `chatSessions`, `activeSessionId` | storage-coupled, private |
| `newChatSession(app)` | `modules/sessions.js:44` | Create new session, reset state, render | Calls `addChatMessage`, `updateTokenDisplay`, `saveChatSessions`, `renderSessionList`, `renderLiveShellState` | storage-coupled |
| `loadChatSessions(app)` | `modules/sessions.js:72` | Load from localStorage, restore latest | Calls `newChatSession`, `renderChatMessages`, `renderSessionList`, `renderLiveShellState` | storage-coupled |
| `saveChatSessions(app)` | `modules/sessions.js:97` | Persist to localStorage | Calls `syncActiveSession`, `localStorage.setItem` | storage-coupled |
| `switchSession(app, id)` | `modules/sessions.js:106` | Switch active session, re-render | Calls `syncActiveSession`, `renderChatMessages`, `updateTokenDisplay`, `renderSessionList`, `saveChatSessions`, `renderLiveShellState` | storage-coupled |
| `renderChatMessages(app)` | `modules/sessions.js:120` | Restore chat DOM from session messages | Uses `#aiChatMessages`, `renderChatMessageFromSession`, `addChatMessage` | DOM + Chat crossing |
| `renderSessionList(app)` | `modules/sessions.js:133` | Delegate to `HajimiSessionListView` | Calls `ensureSessionListView().renderSessionList(app)` | DONE (Day4-B/C) |
| `newChatSession()` wrapper | `app.js:3584` | Delegates to `HajimiSessions.newChatSession(this)` | — | thin wrapper |
| `loadChatSessions()` wrapper | `app.js:3588` | Delegates to `HajimiSessions.loadChatSessions(this)` | — | thin wrapper |
| `saveChatSessions()` wrapper | `app.js:3592` | Delegates to `HajimiSessions.saveChatSessions(this)` | — | thin wrapper |
| `switchSession(id)` wrapper | `app.js:3596` | Delegates to `HajimiSessions.switchSession(this, id)` | — | thin wrapper |
| `renderChatMessages()` wrapper | `app.js:3600` | Delegates to `HajimiSessions.renderChatMessages(this)` | — | thin wrapper |
| `renderSessionList()` wrapper | `app.js:3604` | Delegates to `HajimiSessions.renderSessionList(this)` | — | thin wrapper |
| `getActiveSession()` | `app.js:215` | Returns active session object | Reads `chatSessions`, `activeSessionId` | helper |
| `getDisplaySessionTitle()` | `app.js:219` | Returns display title | Reads active session + `chatMessages` | helper, Chat-crossing |

### Sessions DOM Table

| DOM / selector | Location | Owner | Note |
| --- | --- | --- | --- |
| `#chatSessionsPanel` | `index.html:54` | Sidebar shell | `data-panel="chat-sessions"` |
| `#newSessionBtn` | `index.html:58` | Session toolbar | Bound in `setupChat()` at `app.js:2414` |
| `#sessionMoreBtn` | `index.html:59` | Session toolbar | Bound in `setupMoreMenus()`, calls `exportAllCheckpoints()` |
| `#sessionList` | `index.html:66` | Session list | Rendered by `HajimiSessionListView` (DONE) |
| `#newChatBtn` | `index.html:318` | Chat toolbar | Bound in `setupChat()` at `app.js:2410` |
| `#aiChatMessages` | `index.html` | Chat view | Cleared/restored by Sessions |

### Sessions Storage Table

| Key | Location | R/W | Classification |
| --- | --- | --- | --- |
| `hajimi_chat_sessions` | `modules/sessions.js:4` | R: `loadChatSessions` / W: `saveChatSessions` | sessions |

### Sessions Cross-References (Keep-for-Now)

| Caller | Calls to Sessions | Risk |
| --- | --- | --- |
| `sendChatMessage()` `app.js:2485-2568` | `saveChatSessions()`, `renderSessionList()` (×3 branches: slash, backend, demo) | HIGH — Chat streaming path |
| `handleAgentEvent()` `app.js:3238-3258` | `saveChatSessions()`, `renderSessionList()` (×2: result, error) | HIGH — Agent streaming path |
| `setupChat()` `app.js:2410-2418` | Binds `#newChatBtn` → `newChatSession()`, `#newSessionBtn` → `newChatSession()` | Medium — button wiring |
| `setupMoreMenus()` `app.js:924` | Binds `#sessionMoreBtn` → `exportAllCheckpoints()` | FORBIDDEN — Checkpoint export |
| `setupKeyboardShortcuts()` `app.js:5182-5185` | `Ctrl+Shift+C` → `showSidebar('chat-sessions')` | shared shell shortcut |
| `init()` `app.js:68` | `loadChatSessions()` | bootstrap |

### Sessions Migration Candidates

| Candidate | Target | Readiness |
| --- | --- | --- |
| `#newSessionBtn` click binding | `controllers/session-controller.js` | CANDIDATE — needs button-wiring smoke |
| `#newChatBtn` click binding | `controllers/session-controller.js` | CANDIDATE — crosses Chat toolbar |
| `getActiveSession()` | `controllers/session-controller.js` or keep in app shell | CANDIDATE |
| `getDisplaySessionTitle()` | keep in app shell | Chat-crossing, used by renderLiveShellState |

### Sessions Keep-for-Now

| Item | Reason |
| --- | --- |
| `sendChatMessage()` save/render calls | Chat + Provider + streaming boundary |
| `handleAgentEvent()` save/render calls | Agent streaming boundary (FORBIDDEN) |
| `setupMoreMenus()` sessionMoreBtn | Calls `exportAllCheckpoints()` (Checkpoint FORBIDDEN) |
| `chatSessions` / `activeSessionId` state fields | Global state; move with app-state split |
| `renderChatMessages()` | Crosses Chat DOM (uses `addChatMessage`, `renderChatMessageFromSession`) |

---

## B. Settings Domain Inventory

### Settings Function Table

| Function | Location | Role | Classification |
| --- | --- | --- | --- |
| `loadSettings()` | `app.js:2026` | Load from localStorage, apply, bind events | settings-controller candidate |
| `saveSettings()` | `app.js:2040` | Persist to localStorage | settings-controller candidate |
| `applySettings()` | `app.js:2048` | Apply theme/fontSize/wordWrap/autoSave to DOM | settings-view candidate |
| `applyTheme(theme)` | `app.js:2070` | Set `data-theme` on root | settings-view candidate |
| `setupSystemThemeListener()` | `app.js:2081` | Listen for system theme change | settings-controller candidate |
| `bindSettingsEvents()` | `app.js:2090` | Bind change events on `#settingTheme/FontSize/WordWrap/AutoSave` | settings-controller candidate |
| `showSidebar(view)` | `app.js:464` | Delegates to `HajimiSettingsPanel.showSidebar(this, view)` | SHARED SHELL |
| `setupSettingsTabs()` | `app.js:468` | Delegates to `HajimiSettingsPanel.setupSettingsTabs(this)` | SHARED SHELL |
| `switchSettingsTab(tabId)` | `app.js:472` | Delegates to `HajimiSettingsPanel.switchSettingsTab(this, tabId)` | SHARED SHELL |
| `HajimiSettingsPanel.showSidebar()` | `modules/settings-panel.js:18` | Toggle activity/sidebar panels, load providers/git on tab switch | SHARED SHELL |
| `HajimiSettingsPanel.switchSettingsTab()` | `modules/settings-panel.js:43` | Toggle settings sub-tabs, load providers/mcp/checkpoints/audit | Provider-adjacent |

### Settings DOM Table

| DOM / selector | Location | Owner |
| --- | --- | --- |
| `#settingsPanel` | `index.html:120` | `data-panel="settings"` sidebar panel |
| `#settingsTabs` | `index.html:127` | Settings tab bar |
| `.settings-tab[data-tab="general"]` | `index.html:128` | General tab |
| `.settings-tab[data-tab="providers"]` | `index.html:129` | Providers tab |
| `.settings-tab[data-tab="mcp"]` | `index.html:130` | MCP tab |
| `.settings-tab[data-tab="governance"]` | `index.html:131` | Governance tab |
| `.settings-tab[data-tab="audit"]` | `index.html:132` | Audit tab |
| `[data-settings-panel="general"]` | `index.html:136` | General panel |
| `[data-settings-panel="providers"]` | `index.html:167` | Providers panel |
| `[data-settings-panel="mcp"]` | `index.html:198` | MCP panel |
| `[data-settings-panel="governance"]` | `index.html:218` | Governance panel |
| `[data-settings-panel="audit"]` | `index.html:259` | Audit panel |
| `#settingTheme` | `index.html:150` | Theme select |
| `#settingFontSize` | `index.html:158` | Font size input |
| `#settingWordWrap` | `index.html:162` | Word wrap checkbox |

### Settings Storage Table

| Key | Location | R/W | Classification |
| --- | --- | --- | --- |
| `hajimi.settings` | `app.js:2028/2042` | R: `loadSettings` / W: `saveSettings` | settings |

### Provider-Adjacent Risk Table

| Function / Path | Risk | Reason |
| --- | --- | --- |
| `switchSettingsTab('providers')` | HIGH | Calls `loadProviders()`, `loadAgentProviders()` |
| `switchSettingsTab('audit')` | HIGH | Calls `loadCheckpoints()`, `loadAuditLogs()` |
| `showSidebar('settings')` | MEDIUM | Calls `loadProviders()`, `loadAgentProviders()`, `loadMcpServers()` |
| `setupProviderSettings()` | FORBIDDEN | Provider/Keyring/probe boundary |
| `openProviderModal()` | FORBIDDEN | Provider/probe status |
| `saveProviderConfig/deleteProviderConfig` | FORBIDDEN | Provider/Keyring boundary |

### Settings Migration Candidates

| Candidate | Target | Readiness |
| --- | --- | --- |
| `loadSettings()` | `controllers/settings-controller.js` | SAFE — pure localStorage + DOM |
| `saveSettings()` | `controllers/settings-controller.js` | SAFE — pure localStorage |
| `applySettings()` | `views/settings-view.js` | SAFE — DOM-only |
| `applyTheme(theme)` | `views/settings-view.js` | SAFE — DOM-only |
| `setupSystemThemeListener()` | `controllers/settings-controller.js` | SAFE |
| `bindSettingsEvents()` | `controllers/settings-controller.js` | SAFE — DOM event binding |

### Settings Keep-for-Now

| Item | Reason |
| --- | --- |
| `showSidebar()` / `setupSettingsTabs()` / `switchSettingsTab()` | Shared shell — multi-domain routing |
| `HajimiSettingsPanel` module | Shared shell — already extracted module |
| Provider / Governance / Audit sub-tab logic | Provider/Checkpoint FORBIDDEN boundaries |

---

## C. Storage Key Inventory

| Key | Domain | Read Location | Write Location | Service Candidate |
| --- | --- | --- | --- | --- |
| `hajimi_chat_sessions` | sessions | `modules/sessions.js:74` | `modules/sessions.js:100` | YES |
| `hajimi.settings` | settings | `app.js:2028` | `app.js:2042` | YES |
| `hajimi.layout` | layout | `app.js:2010` | `app.js:2002` | YES |
| `hajimi_cumulative_stats` | stats | `app.js:2218` | `app.js:2231` | YES |
| `hajimi.mcpServers` | mcp | `app.js:4520` | `app.js:4512` | LATER — MCP domain |
| `hajimi.installedExtensions` | extensions | `app.js:4601` | `app.js:4593` | LATER — Extensions domain |

### Candidate Storage Service API Draft

```text
StorageService {
  getSessions(): object[]
  setSessions(data: object[]): void
  getSettings(): object
  setSettings(data: object): void
  getLayout(): object
  setLayout(data: object): void
  getCumulativeStats(): object
  setCumulativeStats(data: object): void
}
```

### Keep-in-Domain List

| Key | Reason |
| --- | --- |
| `hajimi.mcpServers` | MCP domain not in Day4 scope |
| `hajimi.installedExtensions` | Extensions domain not in Day4 scope |

---

## D. Shared DOM / Sidebar / Routing

### Shared Shell Ownership Table

| Entry | Owner | Callers | Classification |
| --- | --- | --- | --- |
| `showSidebar(view)` | `HajimiSettingsPanel` | `setupActivityBar`, `setupLiveShellControls`, `setupMoreMenus`, `setupChat`, command catalog, keyboard shortcuts, `openFolder`, many command palette entries | SHARED SHELL — do not move into single-domain controller |
| `setupActivityBar()` | app shell | Binds all `.activity-item[data-view]` clicks | SHARED SHELL |
| `toggleSidebar()` | app shell | `Ctrl+B` shortcut | SHARED SHELL |
| `switchSettingsTab(tabId)` | `HajimiSettingsPanel` | command catalog, more menus, settings-related commands | SHARED SHELL |

### Activity Bar → Sidebar Panel Map

| `data-view` | `data-panel` | Panel ID |
| --- | --- | --- |
| `chat-sessions` | `chat-sessions` | `#chatSessionsPanel` |
| `explorer` | `explorer` | `#explorerPanel` |
| `settings` | `settings` | `#settingsPanel` |

### Shortcut Known-Risk Table

| Shortcut | Action | Domain | Status |
| --- | --- | --- | --- |
| `Ctrl+Shift+P` | Command Palette | command-palette | DONE — delegated to controller |
| `Ctrl+Shift+C` | `showSidebar('chat-sessions')` | sessions / shared | shared shell |
| `Ctrl+Shift+S` | `showSidebar('settings')` | settings / shared | shared shell |
| `Ctrl+Shift+E` | `showSidebar('explorer')` | workspace / shared | shared shell |
| `Ctrl+Shift+F` | `showSidebar('search')` | search / shared | shared shell |
| `Ctrl+Shift+G` | `showSidebar('git')` | git / shared | shared shell |
| `Ctrl+Shift+A` | `showSidebar('agent-trace')` | agent / shared | shared shell |
| `Ctrl+Shift+X` | `showSidebar('extensions')` | extensions / shared | shared shell |
| `Ctrl+B` | `toggleSidebar()` | shared | shared shell |
| `Escape` | `hideCommandPalette()` | command-palette | DONE |

### Do-Not-Move-Yet Table

| Item | Reason |
| --- | --- |
| `showSidebar()` | Routes to 8+ panels; cannot be owned by a single domain controller |
| `setupActivityBar()` | Generic click dispatcher for all activity items |
| `toggleSidebar()` | Pure layout toggle |
| `setupKeyboardShortcuts()` inline fallback | Still kept for when controller modules are absent |
| `setupMoreMenus()` | Mixes sessions (`exportAllCheckpoints`) + settings + trace |

---

## E. Existing Smoke Coverage Matrix

| Smoke File | Coverage | Classification | Pre/Post Gate |
| --- | --- | --- | --- |
| `day14_sessions_thinking_modules_smoke.js` | `HajimiSessions` module loading, new/load/save/switch/render | DIRECT — sessions module | YES — must run before/after session migration |
| `day19_settings_smoke.js` | `HajimiSettingsPanel` tab switching, DOM contract | DIRECT — settings module | YES — must run before/after settings migration |
| `day29_session_list_dom_smoke.js` | `#sessionList`, empty state, items, escaping, click switch | DIRECT — session list view | YES |
| `day22_command_palette_catalog_smoke.js` | Command catalog including `chat.new`, `view.chat-sessions`, `session.export` | INDIRECT — sessions | NO |
| `day26_dom_contract_smoke.js` | DOM contract: `sessionList`, `settingsPanel`, `settingTheme` | INDIRECT — DOM structure | NO |
| `day28_command_palette_dom_smoke.js` | Command palette DOM rendering | INDIRECT | NO |
| `day30_command_palette_delegation_smoke.js` | Module delegation for command palette | INDIRECT | NO |
| WebView session click | Session item click, session switching in real browser | WEBVIEW-GAP — done in Day4-C | DONE |
| WebView settings visual | Settings panel visibility in real browser | WEBVIEW-GAP — needs receipt | NEEDED for settings migration |
| WebView session button | `#newSessionBtn` / `#newChatBtn` click creates session | WEBVIEW-GAP | NEEDED for session controller |
| `day13_workspace_modules_smoke.js` | Workspace module | UNRELATED | NO |
| `day16_slash_palette_smoke.js` | Slash palette | UNRELATED | NO |
| `day25_inspector_safety_smoke.js` | Inspector | UNRELATED | NO |
| `day23_audit_log_smoke.js` | Audit log | UNRELATED | NO |
| `day24_resource_dashboard_smoke.js` | Resource dashboard | UNRELATED | NO |

---

## F. Final Migration Recommendation

### Batch 1: Safest — Can Migrate Immediately

| Candidate | Target | Required Gates |
| --- | --- | --- |
| `loadSettings()` | `controllers/settings-controller.js` | `node --check`, day19 before/after |
| `saveSettings()` | `controllers/settings-controller.js` | day19 |
| `applySettings()` | `views/settings-view.js` | day19 |
| `applyTheme(theme)` | `views/settings-view.js` | day19 |
| `setupSystemThemeListener()` | `controllers/settings-controller.js` | day19 |
| `bindSettingsEvents()` | `controllers/settings-controller.js` | day19 |

### Batch 2: Needs Node Smoke After Migration

| Candidate | Target | Required Gates |
| --- | --- | --- |
| `#newSessionBtn` click binding | `controllers/session-controller.js` | day14, day29, new button-wiring smoke |
| `#newChatBtn` click binding | `controllers/session-controller.js` | day14, day29, new button-wiring smoke |
| `getActiveSession()` | `controllers/session-controller.js` | day14 |
| Storage keys into `services/storage-service.js` | `services/storage-service.js` | day14, day19 |

### Batch 3: Needs WebView Receipt After Migration

| Candidate | Target | Required Receipt |
| --- | --- | --- |
| Settings full controller/view wiring in `index.html` | `index.html` script tags | WebView: settings panel visible, theme change works, font size change works |
| Session controller wiring in `index.html` | `index.html` script tags | WebView: new session button creates session, chat btn creates session |

### Batch 4: Keep-for-Now

| Item | Reason |
| --- | --- |
| `showSidebar()` / `setupSettingsTabs()` / `switchSettingsTab()` | Shared shell — multi-domain |
| `setupActivityBar()` | Generic dispatcher |
| `toggleSidebar()` | Layout utility |
| `setupMoreMenus()` | Mixes checkpoint export |
| `setupKeyboardShortcuts()` inline fallback | Kept until all modules are wired |
| `chatSessions` / `activeSessionId` state fields | Global state — app-state split |
| `renderChatMessages()` | Chat DOM crossing |
| `getDisplaySessionTitle()` | Chat + session crossing |
| `sendChatMessage()` session save/render | Chat streaming path |
| `handleAgentEvent()` session save/render | Agent streaming path |
| `HajimiSettingsPanel` module | Already extracted shared shell |
| `hajimi.mcpServers` storage | MCP domain |
| `hajimi.installedExtensions` storage | Extensions domain |

### Forbidden — Must Not Touch in Day4

| Item | Reason |
| --- | --- |
| `setupProviderSettings()` | Provider/Keyring/probe |
| `openProviderModal()` | Provider/probe |
| `saveProviderConfig` / `deleteProviderConfig` | Provider/Keyring |
| `streamChat()` | Agent/model streaming |
| `invokeAgentTask()` | Agent streaming |
| `handleAgentEvent()` | Agent streaming |
| `sendChatMessage()` | Chat/Provider/streaming |
| `handleChatCommand()` | Slash command execution |
| `exportAllCheckpoints()` | Checkpoint |
| `setupGovernance()` | Governance |
| Shell helpers | Shell boundary |
| CSP / withGlobalTauri | Security boundary |
| `src/interface/desktop/src/main.rs` | Backend |

---

## Validation Results

| Check | Result |
| --- | --- |
| `git pull --ff-only` | Already up to date |
| `node --check src/interface/web/app.js` | PASS |
| `node --check src/interface/web/modules/sessions.js` | PASS |
| `node --check src/interface/web/views/session-list-view.js` | PASS |
| `node tests/frontend/day14_sessions_thinking_modules_smoke.js` | PASS |
| `node tests/frontend/day19_settings_smoke.js` | PASS |
| `node tests/frontend/day29_session_list_dom_smoke.js` | PASS (6 scenarios) |
| forbidden diff count | 0 |
| `git diff --cached --check` | PASS |
| old dirty files staged | NO |

## Rollback Point

```text
4d1a957efd78284405623f95f7297d640d2a8dd6
```

## Next Recommended Step

Day5-A: Migrate Settings functions (Batch 1) into `controllers/settings-controller.js` + `views/settings-view.js` with delegation wrappers in `app.js`. Run day19 before/after. Do not touch Provider/Keyring/Checkpoint/Shell/Agent.
