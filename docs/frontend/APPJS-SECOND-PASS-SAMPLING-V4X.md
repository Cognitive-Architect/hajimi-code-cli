# STONE-AUDIT-V4X-DAY2｜app.js Second-Pass Sampling

## 0. One-Line Result

V4X Day2 completed as docs-only/read-only sampling: `src/interface/web/app.js` remains 5568 lines, syntax-valid, and unchanged; the next safe direction is not a blind `<=1200` cut, but staged extraction around existing Chat, Provider-readonly, app-state/bootstrap, and closure batches while keeping Chat streaming, Provider write/keyring, Checkpoint, Shell/tool execution, Agent streaming, CSP, and withGlobalTauri in KEEP-FOR-NOW.

人话版：今天不是拆墙，是把大房子重新贴标签。哪些柜子可以先搬，哪些柜子连着煤气管，哪些还得先验收，都先写清楚。

## 1. Scope

- Task: `STONE-AUDIT-V4X-DAY2：app.js Second-Pass Sampling`
- Source work order: `F:\hajimi-code-cli\docs\roadmap\Hajimi ToneFix\task\task02.md`
- Source plan: `F:\hajimi-code-cli\docs\roadmap\Hajimi ToneFix\plan\STONE-AUDIT-V4X-POST-CLOSURE-FINISH_已更新_WebView真实验收_v0.2.md`
- Target file sampled: `F:\hajimi-code-cli\src\interface\web\app.js`
- Allowed write: this report only, plus debt note because UNKNOWN/high-risk areas remain.
- Production code changed: NO
- WebView run in this task: NOT RUN; this is static/read-only sampling only.

## 2. Git / Baseline Receipt

| Item | Result |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `c818ba28cb866013094ad3a724f1372961deb9ed` |
| HEAD after sampling before commit | `c818ba28cb866013094ad3a724f1372961deb9ed` |
| `git status --short` | untracked `.agents/`; untracked `docs/roadmap/Hajimi ToneFix/plan/STONE-AUDIT-V4X-POST-CLOSURE-FINISH_已更新_WebView真实验收_v0.2.md` |
| Old dirty files staged | NO at sampling time |
| `app.js` line count | `5568` |
| `style.css` line count | `21` |
| `main.rs` line count | `2782` |
| `node --check src/interface/web/app.js` | PASS |
| Forbidden production diff check | PASS, no output |
| `git diff --cached --check` before writing report | PASS |

Commands used:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
node --check src/interface/web/app.js
git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/modules src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json
git diff --cached --check
(Get-Content src/interface/web/app.js).Count
(Get-Content src/interface/web/style.css).Count
(Get-Content src/interface/desktop/src/main.rs).Count
```

## 3. Existing Module Reality

The second-pass plan must account for already extracted modules and skeletons.

| Path | Current sampling note | Classification |
|---|---|---|
| `src/interface/web/controllers/chat-controller.js` | Already owns `setupChat()` DOM event wiring if loaded; calls back into `app.sendChatMessage()` and app helpers. | controller-only candidate already started |
| `src/interface/web/views/chat-view.js` | Already owns turn/message render helpers used by `renderChatMessageFromSession`, `updateTurnThinking`, `updateTurnResponse`. | render-only already started |
| `src/interface/web/controllers/provider-controller.js` | Read-only provider controller exists. | provider-readonly candidate |
| `src/interface/web/views/provider-view.js` | Read-only provider view exists and avoids edit/delete buttons. | provider-readonly candidate |
| `src/interface/web/services/provider-service.js` | Provider normalization/read-only metadata support exists. | provider-readonly candidate |
| `src/interface/web/app/app-state.js` | Skeleton only, no production wiring yet. | state/bootstrap candidate, not active |
| `src/interface/web/app/bootstrap.js` | Skeleton only, no production wiring yet. | state/bootstrap candidate, not active |
| `src/interface/web/modules/sessions.js` + `views/session-list-view.js` | Session domain already extracted and browser-wired in earlier V3X work. | do not reopen unless regression |
| `src/interface/web/controllers/command-controller.js` + `views/command-palette-view.js` | Command Palette already delegated; only keyboard fallback remains in `app.js`. | keep stable |

## 4. app.js Hotspot Inventory

Line ranges come from a static method boundary scan of `src/interface/web/app.js`.

| Area | Method / range | Current role | Calls out to | Risk class |
|---|---:|---|---|---|
| Tauri bridge | `getTauriBridge` 144-149, `isTauriAvailable` 150-153, `getTauriInvoke` 154-157, `invokeTauri` 158-161 | Core bridge helpers | `window.HajimiTauri`, `window.__TAURI__` | KEEP-FOR-NOW |
| Shell | `getShellToolName` 184-187, `quoteShellArg` 188-193, `runShellCommand` 194-205 | Shell/tool execution wrapper | `executeTool`, shell tools | KEEP-FOR-NOW |
| Topbar/live shell | `setupLiveShellControls` 206-214, `renderLiveShellState` 226-238, summary renderers 239-410 | DOM status summaries | `HajimiTopbarView`, direct DOM fallback | render-only candidate with fallback caution |
| Settings wrappers | `loadSettings` 2032-2035, `saveSettings` 2036-2039, `applySettings` 2040-2043 | Already delegates to settings controller | `HajimiSettingsController` | SAFE-TO-VERIFY |
| Chat context | `addChatContextFile` 2059-2066, `removeChatContextFile` 2067-2073, `clearChatContext` 2074-2087, `renderChatContext` 2088-2113 | Context list state + DOM render | direct DOM, `renderLiveShellState` | controller/view candidate |
| Token cumulative | `loadCumulativeFromLocalStorage` 2143-2154, `saveCumulativeToLocalStorage` 2155-2163 | localStorage token stats fallback | `localStorage` | storage-coupled candidate |
| Active provider config | `getActiveProviderConfig` 2199-2212 | Provider config normalization for chat | provider state | high-risk-adjacent |
| Chat setup shell | `setupChat` 2267-2356 | DOM event wiring and slash palette setup, but already delegates if `HajimiChatController` exists | `HajimiChatController`, `HajimiSlashPalette`, session buttons | controller-only candidate |
| Slash catalog fallback | `getSlashCommands` 2357-2375 | Catalog module fallback | `HajimiSlashCommandCatalog` | SAFE-TO-VERIFY; do not delete without script load proof |
| Chat send | `sendChatMessage` 2376-2504 | Main chat basic submit + slash command + provider dispatch + session persistence | `handleChatCommand`, `streamChat`, session save/render | high-risk split candidate |
| Slash command execution | `handleChatCommand` 2505-2809 | Executes `/agent`, `/tools`, `/providers`, `/tool`, `/chat`, `/mcp`, `/search`, `/git`, `/extensions`, `/compact` | Tauri invoke/tools/git/MCP/Agent/chat | KEEP-FOR-NOW except receipt-only |
| Chat view wrappers | `renderChatMessageFromSession` 2955-2988, `updateTurnThinking` 2989-3021, `updateTurnResponse` 3022-3054 | Mostly delegates to `HajimiChatView` | `HajimiChatView` | SAFE-TO-VERIFY |
| Agent task | `invokeAgentTask` 3055-3125 | Starts agent task, updates UI state | `invokeTauri('start_agent_task')`, events | KEEP-FOR-NOW |
| Agent event stream | `handleAgentEvent` 3126-3228 | Agent event rendering and completion handling | thinking UI, sessions, cumulative stats | KEEP-FOR-NOW |
| Chat streaming | `streamChat` 3229-3411 | Provider streaming, parser callbacks, token update | `invokeTauri('stream_chat')`, `HajimiThinkingUI` | KEEP-FOR-NOW |
| Sessions wrappers | `newChatSession` 3541-3544 through `renderSessionList` 3561-3567 | Already delegates to sessions module | `HajimiSessions` | keep stable |
| Provider list load | `loadProviders` 3568-3582 | Tauri load provider configs, render model/list/state | `invokeTauri('list_provider_configs')` | high-risk-adjacent |
| Model picker/list | `renderModelButton` 3583-3596, `setupModelPicker` 3597-3630, `renderModelPicker` 3631-3683, `selectProvider` 3684-3695 | Partly delegated to model picker modules | model picker controller/view | SAFE-TO-VERIFY, WebView needed |
| Provider read/write UI | `renderProviderList` 3696-3732, `setupProviderSettings` 3733-4003, `openProviderModal` 4004-4063 | Full provider modal/list/read/write/probe/backup bindings | provider write/probe/backup/keyring paths | split provider-readonly only |
| Provider backup/write | `exportProviderBackup` 4172-4181, `importProviderBackup` 4182-4192, `saveProviderConfig` 4193-4251, `deleteProviderConfig` 4257-4270 | High-risk write paths | keyring/config/backup commands | KEEP-FOR-NOW |
| Profiles / agent provider | `loadProfiles` 4271-4287, `setupProfileSettings` 4288-4340, `loadAgentProviders` 4341-4372, `setupAgentProvider` 4373-4394 | Profile and agent-provider bindings | Tauri profile/provider commands | KEEP-FOR-NOW |
| MCP | `setupMcpSettings` 4408-4415 through `loadMcpServers` 4490-4506 | MCP localStorage + Tauri init/invoke | `mcp_init`, `mcp_invoke` | high-risk-adjacent |
| Governance / approvals | `setupGovernance` 4698-4752, `showApprovalModal` 4753-4838, `invokeGovernance` 4839-4848 | Approval UI and governance commands | Tauri events/commands | KEEP-FOR-NOW |
| Checkpoints | `setupSessionBrowser` 4849-4854 through `exportAllCheckpoints` 4988-4997 | Restore/export/compare/replay/checkpoint UI | checkpoint Tauri commands | KEEP-FOR-NOW |
| Resource dashboard | `setupResourceDashboard` 4998-5001, `updateMetrics` 5431-5435 | Already delegates to dashboard module | `HajimiResourceDashboard` | keep stable |
| Command Palette wrappers | `setupCommandPalette` 5094-5103 through `executeSelectedCommand` 5120-5126 | Delegates to controller/view | `HajimiCommandController`, `HajimiCommandPaletteView` | keep stable |
| Keyboard fallback | `setupKeyboardShortcuts` 5127-5188 | Inline global keyboard fallback retained from V3X | direct DOM/app actions | keep-for-now known risk |
| Inline edit / trace replay | `setupInlineEditPanel` 5211-5230 through replay functions 5313-5430 | Edit accept/reject and replay UI | Tauri/apply edits, thinking UI | KEEP-FOR-NOW |
| Compatibility validator | tail block 5521-5567 | Extra provider validation/git bindings through `document.addEventListener('DOMContentLoaded')` | `validate_provider`, provider form DOM | KEEP-FOR-NOW; high-risk duplicate binding |

## 5. DOM / Storage Touchpoints Sampled

This is not a full DOM contract replacement; it is a Day2 hotspot receipt.

| Area | DOM IDs / selectors | Notes |
|---|---|---|
| Chat basic | `aiChatInput`, `aiChatSendBtn`, `slashPalette`, `modelSelectBtn`, `addContextBtn`, `clearContextBtn`, `editModeBtn`, `aiChatMessages`, `aiChatContext`, `aiChatContextList`, `.composer-hint`, `statusTokens` | Input wiring already has controller module, but `sendChatMessage` remains in app.js. |
| Chat turns | `.assistant-turn:last-child .assistant-response-content`, `statusIndicatorRow`, `statusHeader` | Render wrappers use `HajimiChatView`; state mutation remains app-owned. |
| Provider settings | `providerListTab`, `addProviderBtnTab`, `cancelProvider`, `saveProvider`, `providerModalClose`, `providerModal`, `providerApiKey`, `testContextCapacityBtn`, `testProviderBtn`, `backupModal` | Read-only modules exist, but app.js still binds write/probe/backup controls. |
| Model picker | `modelSelectBtn`, `modelPickerModal`, `modelPickerClose`, `modelPickerAddBtn`, `modelPickerBody`, `statusModel` | Mostly delegated to controller/view, but edit/delete actions can call Provider write paths. |
| Storage | `hajimi.layout`, `hajimi_cumulative_stats`, `hajimi.mcpServers`, `hajimi.installedExtensions`; sessions handled by `HajimiSessions.storageKey` in module | Storage split should be cautious because some keys are legacy contracts. |
| Checkpoints | `checkpointListTab`, `checkpointCompareResultTab`, checkpoint restore/export/replay/compare buttons | High-risk; do not move during app.js second pass unless separate sampling exists. |
| Shell / terminal | `terminalContent`, `.terminal-input`, panel selectors | Shell execution is policy-sensitive; not part of Day3-Day7 safe split. |

## 6. Candidate Batch Table

| Batch | Scope | Candidate files | Classification | Pre-removal / pre-move checks | Post checks | Recommendation |
|---|---|---|---|---|---|---|
| 2A | Chat setup DOM shell | `controllers/chat-controller.js`, maybe `tests/frontend/day33_chat_controller_smoke.js` | controller-only | Confirm `index.html` loads chat controller before `app.js`; run existing day33 smoke if present | `node --check app.js`; chat controller smoke; WebView chat input | SAFE-TO-VERIFY, because controller already mirrors `setupChat` |
| 2B | Chat view wrappers | `views/chat-view.js`, tests day32/day17 | render-only | Confirm wrappers all delegate to `HajimiChatView`; no streaming mutation moved | day32/day17 smoke; chat render WebView | SAFE-TO-VERIFY, no business move |
| 2C | Chat context list | new/existing chat view/controller | DOM-only + state-coupled | Sample `chatContextFiles` lifecycle; confirm context file add/remove smoke | Node smoke + WebView context list | CANDIDATE, but not first cut |
| 2D | Token cumulative localStorage | `services/storage-service.js` or new token service | storage-coupled | Verify `hajimi_cumulative_stats` legacy key and backend fallback | Node storage smoke | CANDIDATE after Chat basic |
| 2E | Provider readonly render | `services/provider-service.js`, `views/provider-view.js`, `controllers/provider-controller.js` | provider-readonly | Run day36 provider readonly smoke; verify no `invokeTauri` in readonly modules | WebView readonly receipt | CANDIDATE, readonly only |
| 2F | Provider write/probe/backup | app.js provider modal/write functions | high-risk-keep | Separate approval/sampling only | N/A | KEEP-FOR-NOW |
| 2G | App state/bootstrap skeleton activation | `app/app-state.js`, `app/bootstrap.js` | state/bootstrap | Confirm skeletons currently not browser-wired; sample init order | Node init smoke + WebView launch | CANDIDATE for Day6, not before Chat/Provider receipts |
| 2H | Command Palette wrappers | existing command controller/view | wrapper-only | Already done in V3X; do not reopen unless regression | day22/day28/day30 | KEEP STABLE |
| 2I | `setupKeyboardShortcuts` fallback | app.js | known risk | Needs separate coverage sampling because it controls global shortcuts | WebView keyboard regression | KEEP-FOR-NOW |
| 2J | Checkpoints | app.js checkpoint methods | checkpoint high-risk | Separate checkpoint-only readonly/write sampling | cargo/WebView + backend receipt | KEEP-FOR-NOW |
| 2K | Shell / terminal | app.js terminal/shell methods | shell high-risk | Separate Shell policy sampling | security-gate + shell tests | KEEP-FOR-NOW |
| 2L | Agent task/event/streaming | `invokeAgentTask`, `handleAgentEvent`, `streamChat` | Agent/streaming high-risk | Separate Agent streaming sampling | WebView + backend event receipt | KEEP-FOR-NOW |
| 2M | MCP localStorage + invoke | app.js MCP methods | storage + invoke high-risk | Separate MCP sampling; command may execute tools | Node/WebView safe list only | KEEP-FOR-NOW |

## 7. Suggested Day3-Day7 Execution Plan

| Day | Suggested focus | Allowed target | Keep out |
|---|---|---|---|
| Day3 | Chat basic path extraction | Verify/use `HajimiChatController` and `HajimiChatView`; reduce `setupChat` fallback if browser wiring is proven | `streamChat`, Provider semantics, Agent streaming |
| Day4 | Slash / Command / Chat WebView receipt | Real WebView receipt for `/`, chat input/submit, Command Palette regression | Provider/Checkpoint/Shell writes |
| Day5 | Provider readonly WebView closure | Use readonly provider modules and day36 evidence; record forbidden command calls = 0 | save/delete/test/probe/backup/keyring |
| Day6 | App state/bootstrap initial split | Activate or extend app-state/bootstrap only for pure state/init helpers with smoke | any execution branch |
| Day7 | app.js second-pass closure | Delete only proven old fallback/no-op wrappers and score line-count progress | hard `<=1200` claim without evidence |

## 8. KEEP-FOR-NOW Table

| Path / method | Why keep |
|---|---|
| `sendChatMessage` 2376-2504 | Central chat pipeline mixes input, slash commands, provider dispatch, session persistence, UI state, cumulative stats. Only split after Chat basic and WebView receipt. |
| `handleChatCommand` 2505-2809 | Executes high-impact slash commands including `/tool`, `/agent`, `/mcp`, `/git`, `/compact`; command semantics must not move in broad app.js slimming. |
| `streamChat` 3229-3411 | Provider streaming and thinking/response parser path; high-risk and already covered by specialized smoke. |
| `invokeAgentTask` / `handleAgentEvent` 3055-3228 | Agent event lifecycle; do not mix with UI slimming. |
| Provider write/probe/backup functions 3733-4251 and tail validator 5521-5567 | Touches keyring-adjacent save/update/delete/probe/backup/validate commands. |
| Checkpoint functions 4849-4997 | Restore/export/compare/replay are explicitly forbidden high-risk paths. |
| Shell/terminal functions 184-205 and 1654-1770 | Shell execution is security-policy-sensitive. |
| Governance approval functions 4698-4848 | Approval flow and Tauri event path; separate safety task only. |
| MCP invoke functions 4408-4506 | Can initialize/invoke external tools; not a generic storage split. |

## 9. UNKNOWN / Debt

| Item | Status | Required evidence before upgrade |
|---|---|---|
| Whether `chat-controller.js` is loaded in release path after latest dist sync | UNKNOWN in this task | `index.html` script order + release/WebView receipt |
| Whether `app/app-state.js` and `app/bootstrap.js` can be safely wired | UNKNOWN | Day6 sampling of init order and WebView launch |
| Whether `setupKeyboardShortcuts` fallback can be removed | UNKNOWN / known risk | dedicated shortcut coverage and WebView keyboard regression |
| Whether Provider readonly can fully replace `renderProviderList` | UNKNOWN | day36 Node PASS plus WebView readonly receipt with forbidden calls = 0 |
| Whether app.js can reach `<=1200` | UNKNOWN / not claimed | Day7 closure after evidence; current recommendation only targets staged reduction |

Debt companion: `F:\hajimi-code-cli\docs\debt\APPJS-SECOND-PASS-SAMPLING-DEBT-V4X.md`.

## 10. Validation Summary

| Check | Result |
|---|---|
| `git branch --show-current` | PASS: `stone-audit-v3x-controlled-demolition` |
| `git rev-parse HEAD` | PASS: `c818ba28cb866013094ad3a724f1372961deb9ed` |
| `git status --short` | PASS with old untracked files only before report write: `.agents/`, V4X plan markdown |
| `node --check src/interface/web/app.js` | PASS |
| Forbidden production diff | PASS, no output before report write |
| `git diff --cached --check` | PASS before report write |
| WebView | NOT RUN; not required for Day2 docs-only sampling |

## 11. Final Recommendation

Proceed to Day3 only as a small Chat basic wiring/verification slice:

1. Verify `controllers/chat-controller.js` is loaded in browser/release path before `app.js`.
2. Keep `sendChatMessage`, `handleChatCommand`, and `streamChat` semantics in `app.js`.
3. Use Node smoke plus real WebView chat input/submit receipt.
4. Stop immediately if the implementation requires Provider write/keyring, Shell, Checkpoint, Agent streaming, CSP, or withGlobalTauri changes.

人话版：下一步可以先搬“聊天输入框这张小桌子”，但别动“模型流式回复这根煤气管”，也别碰 Provider 保存钥匙那一柜子。
