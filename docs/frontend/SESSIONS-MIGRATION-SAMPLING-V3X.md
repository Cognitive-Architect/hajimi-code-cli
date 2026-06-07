# STONE-AUDIT-V3X-CONTROLLED-DEMOLITION Day4-A

## Sessions Migration Sampling

Date: 2026-06-07

Branch: `stone-audit-v3x-controlled-demolition`

HEAD before: `f14f42cdfb9d702a5eb46e67fe9e6307e384fc62`

Scope: Sessions frontend domain sampling only.

Production code changes: NO

Commit: NO

Push: NO

Command Palette status: out of scope for this round. Day3-E is treated as DONE / PUSHED.

Known risk carried forward: `setupKeyboardShortcuts()` fallback coverage sampling is not a Day4-A gate.

Forbidden areas not touched:

- Provider / Keyring
- Shell
- Checkpoint restore / export / compare / replay
- CSP / withGlobalTauri
- Agent streaming
- Command Palette controller/view
- `src/interface/desktop/src/main.rs`

## Git Status Snapshot

Command:

```powershell
git status --short
```

Observed historical dirty files before this report was added:

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

This sampling round writes only:

- `docs/frontend/SESSIONS-MIGRATION-SAMPLING-V3X.md`

Production files modified: NO

## Plan Evidence

`docs/roadmap/Hajimi ToneFix/plan/new_plan.md` Day 4 names these target domains:

| Old area | New location |
| --- | --- |
| Sessions | `controllers/session-controller.js` + `views/session-list-view.js` |
| Settings | `controllers/settings-controller.js` + `views/settings-view.js` |
| Storage | `services/storage-service.js` |

Day4-A only samples Sessions. Settings and Storage are recorded as future adjacent domains, not moved here.

## Sessions Function Inventory

| Function / entry | Location | Current role | Calls / touches | Migration classification | Note |
| --- | --- | --- | --- | --- | --- |
| `init()` | `src/interface/web/app.js:51` | App startup | Calls `this.loadChatSessions()` | KEEP-FOR-NOW | Startup order still owned by app shell. |
| `getActiveSession()` | `src/interface/web/app.js:215` | Active session lookup | Reads `chatSessions`, `activeSessionId` | render-only helper candidate | Small helper, but tied to live shell title. |
| `getDisplaySessionTitle()` | `src/interface/web/app.js:219` | Session title fallback | Reads active session and chat messages | render-only helper candidate | Crosses Sessions and Chat display state. |
| `setupMoreMenus()` session branch | `src/interface/web/app.js:916` | More-menu binding | Binds `sessionMoreBtn` to `exportAllCheckpoints()` | KEEP-FOR-NOW | Export checkpoint action is out of scope. |
| `setupChat()` new session buttons | `src/interface/web/app.js:2410`, `src/interface/web/app.js:2414` | UI button entry | Binds `newChatBtn` and `newSessionBtn` to `newChatSession()` | DOM-only candidate | Can move only after controller owns button wiring. |
| `sendChatMessage()` | `src/interface/web/app.js:2443` | Chat submit flow | Calls `saveChatSessions()` and `renderSessionList()` after slash, backend, and demo responses | KEEP-FOR-NOW | Chat flow and provider/stream fallback stay out of Day4-A. |
| `handleAgentEvent()` | `src/interface/web/app.js:3172` | Agent event rendering | Calls `saveChatSessions()` and `renderSessionList()` on result/error | KEEP-FOR-NOW | Agent streaming path is forbidden. |
| `newChatSession()` wrapper | `src/interface/web/app.js:3584` | App wrapper | Delegates to `window.HajimiSessions.newChatSession(this)` | wrapper-with-fallback candidate | No inline fallback currently visible. |
| `loadChatSessions()` wrapper | `src/interface/web/app.js:3588` | App wrapper | Delegates to `window.HajimiSessions.loadChatSessions(this)` | wrapper-with-fallback candidate | No inline fallback currently visible. |
| `saveChatSessions()` wrapper | `src/interface/web/app.js:3592` | App wrapper | Delegates to `window.HajimiSessions.saveChatSessions(this)` | wrapper-with-fallback candidate | No inline fallback currently visible. |
| `switchSession(id)` wrapper | `src/interface/web/app.js:3596` | App wrapper | Delegates to `window.HajimiSessions.switchSession(this, id)` | wrapper-with-fallback candidate | No inline fallback currently visible. |
| `renderChatMessages()` wrapper | `src/interface/web/app.js:3600` | App wrapper | Delegates to `window.HajimiSessions.renderChatMessages(this)` | wrapper-with-fallback candidate | Crosses Sessions and Chat message rendering. |
| `renderSessionList()` wrapper | `src/interface/web/app.js:3604` | App wrapper | Delegates to `window.HajimiSessions.renderSessionList(this)` | wrapper-with-fallback candidate | Main render wrapper. |
| `setupKeyboardShortcuts()` Ctrl+Shift+C | `src/interface/web/app.js:5140`, `src/interface/web/app.js:5182` | Global shortcut | Calls `showSidebar('chat-sessions')` | KEEP-FOR-NOW | Explicitly kept as known risk, not a Day4-A gate. |
| `syncActiveSession(app)` | `src/interface/web/modules/sessions.js:10` | Copy chat messages into active session | Reads/writes `chatMessages`, `chatSessions`, `activeSessionId` | storage-coupled | Private helper. |
| `newChatSession(app)` | `src/interface/web/modules/sessions.js:28` | Create new session | Clears `aiChatMessages`, updates stats, saves and renders list | storage-coupled | Already extracted module logic. |
| `loadChatSessions(app)` | `src/interface/web/modules/sessions.js:56` | Load sessions | Reads localStorage key `hajimi_chat_sessions` | storage-coupled | Already extracted module logic. |
| `saveChatSessions(app)` | `src/interface/web/modules/sessions.js:81` | Save sessions | Writes localStorage key `hajimi_chat_sessions` | storage-coupled | Already extracted module logic. |
| `switchSession(app, id)` | `src/interface/web/modules/sessions.js:90` | Change active session | Re-renders chat/list and saves | storage-coupled | Already extracted module logic. |
| `renderChatMessages(app)` | `src/interface/web/modules/sessions.js:104` | Restore chat DOM from session | Uses `aiChatMessages`, `renderChatMessageFromSession`, `addChatMessage` | DOM-only / Chat-crossing | Crosses session restore and chat UI. |
| `formatSessionTime(value)` | `src/interface/web/modules/sessions.js:117` | Time label helper | No DOM or storage | render-only candidate | Safe to move with view. |
| `renderSessionList(app)` | `src/interface/web/modules/sessions.js:131` | Render list and bind item clicks | Uses `sessionList`, `.session-item`, `data-session` | DOM-only candidate | Best Day4-B view extraction candidate. |

## Sessions DOM Inventory

| DOM / selector | Location | Current owner | Evidence | Migration note |
| --- | --- | --- | --- | --- |
| `#chatSessionsPanel` | `src/interface/web/index.html:54` | Sidebar shell | `data-panel="chat-sessions"` | Keep sidebar shell outside first move. |
| `[data-panel="chat-sessions"]` | `src/interface/web/index.html:54` | Sidebar shell | Used by `showSidebar('chat-sessions')` | Crosses sidebar routing. |
| `#newSessionBtn` | `src/interface/web/index.html:58` | Sessions UI | Bound in `setupChat()` | Candidate for `session-controller.js`. |
| `#sessionMoreBtn` | `src/interface/web/index.html:59` | Sessions UI | Bound in `setupMoreMenus()` | Keep for now due checkpoint export action. |
| `.sidebar-search input[type=search]` | `src/interface/web/index.html:64` | Sidebar UI | No direct Sessions JS behavior found | UNKNOWN; possible future search/filter. |
| `#sessionList` | `src/interface/web/index.html:66` | Sessions list | Rendered by `HajimiSessions.renderSessionList()` | Candidate for `session-list-view.js`. |
| `.session-empty` | `src/interface/web/index.html:67`, `src/interface/web/modules/sessions.js:135` | Sessions list | Empty state text | Candidate for `session-list-view.js`. |
| `.session-item` | `src/interface/web/modules/sessions.js:140`, `src/interface/web/styles/sessions.css` | Sessions list | Rendered and clicked | Candidate for `session-list-view.js`. |
| `.session-item-main` | `src/interface/web/modules/sessions.js:141` | Sessions list | Rendered markup | Candidate for `session-list-view.js`. |
| `.session-title` | `src/interface/web/modules/sessions.js:142` | Sessions list | Escaped title render | Candidate for `session-list-view.js`. |
| `.session-preview` | `src/interface/web/modules/sessions.js:143` | Sessions list | Escaped preview render | Candidate for `session-list-view.js`. |
| `.session-time` | `src/interface/web/modules/sessions.js:145` | Sessions list | Formatted updated time | Candidate for `session-list-view.js`. |
| `[data-session]` | `src/interface/web/modules/sessions.js:140` | Sessions list | Click handler reads `el.dataset.session` | Candidate for `session-list-view.js`. |
| `#aiChatMessages` | `src/interface/web/modules/sessions.js:38`, `src/interface/web/modules/sessions.js:105` | Chat view | Cleared/restored by Sessions | KEEP-FOR-NOW until Chat split. |
| `#newChatBtn` | `src/interface/web/app.js:2410` | Chat toolbar | Calls `newChatSession()` | Candidate only if Session controller can own both chat/sidebar new-session buttons. |

## Storage Touchpoints

| Storage / state | Location | Use | Classification | Note |
| --- | --- | --- | --- | --- |
| `chatSessions: []` | `src/interface/web/app.js:30` | App state array | storage-coupled | Likely remains state until app-state split. |
| `activeSessionId: null` | `src/interface/web/app.js:31` | Current session id | storage-coupled | Likely remains state until app-state split. |
| `STORAGE_KEY = 'hajimi_chat_sessions'` | `src/interface/web/modules/sessions.js:4` | LocalStorage key | storage-coupled | Candidate for future `storage-service.js`. |
| `localStorage.getItem(STORAGE_KEY)` | `src/interface/web/modules/sessions.js:58` | Load session list | storage-coupled | Node smoke already mocks it. |
| `localStorage.setItem(STORAGE_KEY, JSON.stringify(app.chatSessions))` | `src/interface/web/modules/sessions.js:84` | Persist session list | storage-coupled | Node smoke already mocks it. |
| `sessionStorage` | searched in app/modules/tests | No Sessions usage found | PASS | No sessionStorage dependency found in sampled paths. |
| Adjacent storage keys | `src/interface/web/app.js` | `hajimi.layout`, `hajimi.settings`, `hajimi_cumulative_stats`, `hajimi.mcpServers`, `hajimi.installedExtensions` | future Storage domain | Not moved in Day4-A. |

## Existing Smoke Coverage

| Smoke | Current evidence | Coverage | Gap |
| --- | --- | --- | --- |
| `tests/frontend/day14_sessions_thinking_modules_smoke.js` | PASS in this round | Covers `HajimiSessions` module loading, new/load/save/switch/render integration with mock DOM/storage | Broad module smoke, not a browser click proof. |
| `tests/frontend/day29_session_list_dom_smoke.js` | Existing file found | Covers exact `#sessionList`, empty state, localStorage-backed sessions, `.session-item`, `data-session`, escaping, click switch | Good Node DOM receipt. Not rerun in Day4-A unless promoted to Day4-B gate. |
| `tests/frontend/day22_command_palette_catalog_smoke.js` | Existing indirect evidence | Includes `chat.new`, `view.chat-sessions`, `session.export` command entries | Indirect only; not Sessions DOM behavior. |
| `tests/frontend/day26_dom_contract_smoke.js` | Existing DOM contract references `sessionList` | DOM contract receipt | Indirect only. |
| WebView sessionList smoke | Not performed in this round | UNKNOWN | Day4-A does not prove real WebView session click behavior. |

## Migration Candidates

| Candidate | Proposed target | Classification | Readiness | Required guard before moving |
| --- | --- | --- | --- | --- |
| `renderSessionList(app)` DOM render body | `src/interface/web/views/session-list-view.js` | DOM-only | SAFE-TO-MIGRATE-CANDIDATE | Run day14 + day29 before/after; add browser wiring only after Node pass. |
| `formatSessionTime(value)` | `src/interface/web/views/session-list-view.js` or helper inside view | render-only | SAFE-TO-MIGRATE-CANDIDATE | Keep output unchanged for today/yesterday/date cases. |
| `.session-item` click binding | `src/interface/web/views/session-list-view.js` | DOM-only | SAFE-TO-MIGRATE-CANDIDATE | Preserve `data-session` and `app.switchSession(id)` behavior. |
| `#newSessionBtn` binding | `src/interface/web/controllers/session-controller.js` | DOM-only | MIGRATE-AFTER-VIEW | Need controller setup smoke; do not touch `setupKeyboardShortcuts()`. |
| `#newChatBtn` binding | `src/interface/web/controllers/session-controller.js` | DOM-only / Chat-crossing | MIGRATE-AFTER-VIEW | Crosses Chat toolbar; move only if test fixture covers both buttons. |
| `newChatSession/loadChatSessions/saveChatSessions/switchSession` wrappers | `src/interface/web/controllers/session-controller.js` facade or app wrapper shell | wrapper-with-fallback | CANDIDATE | Keep app wrappers for compatibility until browser smoke proves controller exists. |
| `STORAGE_KEY`, localStorage get/set | `src/interface/web/services/storage-service.js` | storage-coupled | LATER-CANDIDATE | Do not mix with Settings/Storage bulk move in Day4-A. |

## Keep For Now

| Item | Reason | Risk |
| --- | --- | --- |
| `sendChatMessage()` save/render calls | Chat submit, provider, slash command, demo fallback all converge here | Moving now would cross Chat and Provider/streaming boundaries. |
| `handleAgentEvent()` save/render calls | Agent streaming result/error path | Forbidden Agent streaming path. |
| `#aiChatMessages` clearing/restoring | Chat view ownership is not split in Day4 | Wait for Chat domain day. |
| `setupMoreMenus()` `sessionMoreBtn` export action | Calls `exportAllCheckpoints()` | Checkpoint export is forbidden. |
| `setupKeyboardShortcuts()` Ctrl+Shift+C | Explicitly downgraded known risk | Keep fallback until shortcut coverage exists. |
| `showSidebar('chat-sessions')` routing | Sidebar shell is shared across Explorer/Settings/Trace/etc. | Do not split sidebar routing in Sessions-only pass. |
| Command catalog `view.chat-sessions`, `chat.new`, `session.export` | Command Palette already migrated and out of scope | Do not reopen Day3-F. |
| `chatSessions` and `activeSessionId` app state fields | Global state is still in app shell | Move only with app-state split. |

## Validation Log

```powershell
node --check src/interface/web/app.js
```

Result: PASS

```powershell
node --check src/interface/web/modules/sessions.js
```

Result: PASS

```powershell
node tests/frontend/day14_sessions_thinking_modules_smoke.js
```

Result: PASS

Output:

```text
day14 sessions/thinking modules smoke: PASS
```

```powershell
git diff --cached --check
```

Result: PASS, no staged diff output.

```powershell
git diff --name-only -- src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json src/interface/web/controllers/command-controller.js src/interface/web/views/command-palette-view.js
```

Result: PASS, no output.

Forbidden diff count: 0

## Day4-B Recommendation

Allow entering a narrow Day4-B Sessions cut only if it is limited to these candidates:

1. Create real `session-list-view.js` behavior from the current `renderSessionList(app)` body and `formatSessionTime(value)`.
2. Add a tiny `session-controller.js` only for `#newSessionBtn` and possibly `#newChatBtn` wiring if a Node smoke covers both.
3. Keep app.js wrappers as compatibility shell at first.
4. Do not move `sendChatMessage()`, `handleAgentEvent()`, `exportAllCheckpoints()`, `setupKeyboardShortcuts()`, or `showSidebar()`.

Required pre/post checks for Day4-B:

```powershell
node --check src/interface/web/app.js
node --check src/interface/web/modules/sessions.js
node --check src/interface/web/views/session-list-view.js
node --check src/interface/web/controllers/session-controller.js
node tests/frontend/day14_sessions_thinking_modules_smoke.js
node tests/frontend/day29_session_list_dom_smoke.js
git diff --name-only -- src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json src/interface/web/controllers/command-controller.js src/interface/web/views/command-palette-view.js
git diff --cached --check
```

Optional WebView receipt after Day4-B:

- App launch
- Session list visible
- New session button creates/activates a session
- Existing session item click switches active session
- No white screen / crash / no response

Rollback point for Day4-A sampling baseline:

```text
f14f42cdfb9d702a5eb46e67fe9e6307e384fc62
```
