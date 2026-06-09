# Day6 Provider / Inspector / Dashboard / Tauri Invoke Sampling V3X

Task: `STONE-AUDIT-V3X-DAY6-A-PROVIDER-INSPECTOR-DASHBOARD-TAURI-INVOKE-SAMPLING`

Source issue: <https://github.com/Cognitive-Architect/hajimi-code-cli/issues/7>

This is a docs-only, read-only sampling pass. It does not move, edit, or execute Provider / Keyring / Shell / Checkpoint / Agent streaming logic.

## 1. Git Baseline

| Item | Result |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| Starting HEAD | `77584f6bbbbfffc916abe45edd0b6294be34268f` |
| `git status --short` | Pre-existing dirty docs/roadmap/native-smoke files remain in worktree; none staged for Day6-A before this document. |
| Production changes | NO |
| Old dirty files staged | NO |
| Commit scope | Day6-A sampling document only |
| Push | YES after commit |

Pre-existing dirty files observed include docs/debt modifications, roadmap deletions, roadmap untracked folders, and `src/interface/desktop/native-smoke.txt`. They were not staged for this task.

## 2. Forbidden Diff Receipt

Command:

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/controllers src/interface/web/views src/interface/web/services src/interface/web/modules src/interface/web/index.html src/interface/web/style.css src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json
```

Result: no output.

| Receipt | Result |
|---|---|
| Forbidden production diff count | 0 |
| Forbidden areas touched | NO |
| `src/interface/web/app.js` modified | NO |
| `src/interface/web/controllers/` modified | NO |
| `src/interface/web/views/` modified | NO |
| `src/interface/web/services/` modified | NO |
| `src/interface/web/modules/` modified | NO |
| `src/interface/web/index.html` modified | NO |
| `src/interface/web/style.css` modified | NO |
| `src/interface/desktop/src/main.rs` modified | NO |
| `src/interface/desktop/tauri.conf.json` modified | NO |

## 3. Provider Inventory

### Provider-Related Functions

| Function / Path | Location | Role | Classification | Notes |
|---|---:|---|---|---|
| `loadProviders()` | `src/interface/web/app.js:3568` | Reads provider configs via `get_provider_configs`, updates `providerConfigs`, selects default active provider, renders model/provider UI. | RISKY | Read-only command, but feeds chat/model selection and Provider UI. |
| `renderModelButton()` | `src/interface/web/app.js:3583` | Updates selected model button text. | SAFE | UI-only candidate; already has Day5 model-picker coverage. |
| `setupModelPicker()` | `src/interface/web/app.js:3597` and `src/interface/web/controllers/model-picker-controller.js:3` | Binds model picker open/close/add events. | KEEP-FOR-NOW | Add flow calls `openProviderModal()`. |
| `openModelPicker()` / `closeModelPicker()` | `src/interface/web/app.js:3616`, `src/interface/web/app.js:3624`; delegated controller exists. | Opens/closes model picker modal. | SAFE | Already migrated to Day5 controller path; Provider edit/delete actions remain adjacent. |
| `renderModelPicker()` | `src/interface/web/app.js:3631` and `src/interface/web/views/model-picker-view.js:13` | Renders provider rows in model picker. | RISKY | Render is UI, but buttons can call `openProviderModal()` and `deleteProviderConfig()`. |
| `selectProvider(id)` | `src/interface/web/app.js:3684` | Sets `activeProviderId`, refreshes model UI. | RISKY | Low write to frontend state, but changes chat provider. |
| `renderProviderList()` | `src/interface/web/app.js:3696` | Renders provider settings list and edit/delete buttons. | RISKY | DOM render body is candidate only if edit/delete wiring is isolated first. |
| `setupProviderSettings()` | `src/interface/web/app.js:3733` | Binds provider modal, form, backup modal, API key toggle, long-context preset, probe controls. | KEEP-FOR-NOW | High coupling around Provider / Keyring / probe. |
| `openProviderModal(config)` | `src/interface/web/app.js:4004` | Initializes Provider form, fills config values, loads probe status via `get_probe_result`. | RISKY | Read/status side is possible later; key field and probe status coupling require extra smoke. |
| `updateCapabilityStatusDisplay(model, probe)` | `src/interface/web/app.js:4064` | Renders Provider probe/capability text. | SAFE-CANDIDATE | UI-only formatting if kept away from probe execution. |
| `loadLatestReceipt()` / `renderContextReceiptPanel()` / `setupReceiptPanel()` | `src/interface/web/app.js:4110`, `4121`, `4126` | Legacy wrappers to inspector module receipt behavior. | RISKY | Inspector-adjacent, not Provider migration target. |
| `closeProviderModal()` | `src/interface/web/app.js:4130` | Closes Provider modal and clears key field. | RISKY | Touches sensitive API-key field; keep near Provider boundary. |
| `exportProviderBackup(password)` | `src/interface/web/app.js:4172` | Calls `export_provider_backup`. | FORBIDDEN | Keyring/backup export boundary. |
| `importProviderBackup(password, filePath)` | `src/interface/web/app.js:4182` | Calls `import_provider_backup`. | FORBIDDEN | Keyring/backup import boundary. |
| `saveProviderConfig()` | `src/interface/web/app.js:4193` | Reads form/API key, calls add/update Provider config, clears key field. | FORBIDDEN | Provider write + OS keyring boundary. |
| `editProviderConfig(id)` | `src/interface/web/app.js:4252` | Finds provider and opens modal. | RISKY | Thin routing but enters Provider modal. |
| `deleteProviderConfig(id)` | `src/interface/web/app.js:4257` | Confirms and calls `delete_provider_config`. | FORBIDDEN | Provider delete + keyring cleanup boundary. |
| `loadAgentProviders()` | `src/interface/web/app.js:4341` | Reads agent-provider map, renders binding rows. | RISKY | Read path, but binding/unbinding is write. |
| `setupAgentProvider()` | `src/interface/web/app.js:4373` | Binds agent/provider map UI. | KEEP-FOR-NOW | Calls `set_agent_provider`. |
| `unbindAgentProvider(agentId)` | `src/interface/web/app.js:4395` | Calls `set_agent_provider` with null provider. | FORBIDDEN | Provider binding write. |
| `testProviderBtn` handler | `src/interface/web/app.js:5527` | Reads Provider form and calls `validate_provider`. | FORBIDDEN | Executes Provider validation; do not include in read-only migration. |
| `provider-service.js` | `src/interface/web/services/provider-service.js:1` | Skeleton only. | UNKNOWN | No production wiring yet. |
| `provider-controller.js` | `src/interface/web/controllers/provider-controller.js:1` | Skeleton only. | UNKNOWN | No production wiring yet. |

### Provider DOM Selectors / IDs

| DOM / Selector | Location | Role | Classification |
|---|---:|---|---|
| `[data-settings-panel="providers"]` | `src/interface/web/index.html:167` | Provider settings tab panel. | RISKY |
| `providerListTab` | `src/interface/web/index.html:178` | Provider list render target. | RISKY |
| `agentBindIdTab`, `agentBindProviderTab`, `agentBindBtnTab`, `agentProviderListTab` | `src/interface/web/index.html:186-192` | Agent-provider binding UI. | FORBIDDEN for write actions |
| `providerModal`, `providerModalTitle`, `providerModalClose`, `providerForm` | `src/interface/web/index.html:482-488` | Provider modal shell. | RISKY |
| `providerId`, `providerName`, `providerModalType`, `providerBaseUrl`, `providerModel` | `src/interface/web/index.html:489-523` | Provider identity/model form fields. | RISKY |
| `providerApiKey`, `providerModalToggleKey` | `src/interface/web/index.html:517-518` | API key field and visibility toggle. | FORBIDDEN / Keyring-adjacent |
| `providerSaveTarget`, `providerSaveTargetField` | `src/interface/web/index.html:525-527` | Global/workspace save target. | FORBIDDEN |
| `providerLongContextPreset`, `providerMaxContext`, `providerMaxOutput`, `providerReserveOutput`, `providerSafetyMargin`, `providerRetrievalBudget`, `providerLongContextMode` | `src/interface/web/index.html:534-568` | Long-context budget UI. | RISKY |
| `providerProbeLevelSelect`, `providerProbeStatusDetails`, `probeDetail*`, `providerCapabilityStatus` | `src/interface/web/index.html:574-592` | Probe/capability display and probe level. | KEEP-FOR-NOW |
| `testProviderBtn` | `src/interface/web/index.html:597` | Provider validation trigger. | FORBIDDEN |
| `backupModal`, `backupPassword`, `backupFilePath`, `backupTogglePassword` | `src/interface/web/index.html:604-620` | Provider backup import/export modal. | FORBIDDEN |
| `.provider-list`, `.provider-item`, `.provider-item-*` | `src/interface/web/styles/provider.css:2-48` | Provider visual rows. | SAFE for CSS only; no Day6-A change. |

### Provider State Fields

| Field | Location | Role | Classification |
|---|---:|---|---|
| `providerConfigs` | `src/interface/web/app.js:14` | In-memory provider config list. | RISKY |
| `activeProviderId` | `src/interface/web/app.js:15` | Selected provider for chat/model UI. | RISKY |
| `editingProviderId` | `src/interface/web/app.js:16` | Tracks modal edit mode. | RISKY |
| `currentWorkspace` | `src/interface/web/app.js:17` | Passed to provider config read/write commands. | RISKY |
| `activeContextProbeCancelled` | `src/interface/web/app.js:3875`, `3999` | Probe cancellation flag. | KEEP-FOR-NOW |
| Agent provider map local variables | `src/interface/web/app.js:4341-4398` | Agent-to-provider binding state from backend. | FORBIDDEN for writes |

### Provider Persistence / Storage

| Storage | Evidence | Classification | Notes |
|---|---|---|---|
| Provider config persistence | `get_provider_configs`, `add_provider_config`, `update_provider_config`, `delete_provider_config` | FORBIDDEN for writes | Backend/keyring-backed; no direct `localStorage` provider key found in frontend. |
| Probe receipt persistence | `get_probe_result`, `probe_provider_context_capacity` | KEEP-FOR-NOW | Probe can execute provider/network path; read-only display may be separable later. |
| Provider backup persistence | `export_provider_backup`, `import_provider_backup` | FORBIDDEN | Encrypted backup + key material boundary. |
| Local storage | `rg localStorage` found layout/stats/MCP/extensions/settings/sessions, not provider config. | SAFE note | Provider config is not a localStorage-only frontend slice. |

### Provider Tauri Invoke Commands

| Command | Caller | Classification | Notes |
|---|---|---|---|
| `get_provider_configs` | `app.js:3572` | RISKY | Read-only but seeds Provider/chat/model UI. |
| `get_probe_result` | `app.js:4040` | RISKY | Read-only probe status; modal/key context. |
| `probe_provider_context_capacity` | `app.js:3905`, `3939` | FORBIDDEN | Runs capacity probe. |
| `add_provider_config` / `update_provider_config` | dynamic command in `saveProviderConfig()` at `app.js:4239-4240` | FORBIDDEN | Provider write + keyring. |
| `delete_provider_config` | `app.js:4261` | FORBIDDEN | Provider delete + keyring cleanup. |
| `validate_provider` | `app.js:5557` | FORBIDDEN | Provider validation execution. |
| `export_provider_backup` / `import_provider_backup` | `app.js:4175`, `4185` | FORBIDDEN | Backup/key material boundary. |
| `get_agent_providers` | `app.js:4344` | RISKY | Read-only map. |
| `set_agent_provider` | `app.js:4382`, `4398` | FORBIDDEN | Writes agent-provider binding. |

### Provider Cross-References

| Cross-reference | Evidence | Risk |
|---|---|---|
| Model picker reads Provider state | `model-picker-view.js:6`, `18`, `24`, `61` | Medium: UI render depends on `providerConfigs` and `activeProviderId`. |
| Model picker actions enter Provider modal/delete | `model-picker-view.js:61-67`, `model-picker-controller.js:13-14` | High: edit/delete crosses Provider boundary. |
| Chat send uses active Provider | `app.js:2442-2455`, `app.js:2609-2622` | High: streaming/model execution boundary. |
| Settings tab loads providers | `day19_settings_smoke.js:117-128`, `app.js:96`, `app.js:941` | Medium: settings navigation reads Provider data. |
| Slash `/providers` lists Provider names | `app.js:2560-2562`, `slash-command-catalog.js:7` | Low/Medium: read-only display. |
| Context receipt displays provider/model | `inspector.js:298-316` | Medium: Provider data rendered in Inspector receipt. |

### Provider Smoke Coverage And Gaps

| Smoke / Doc | Coverage | Result / Classification |
|---|---|---|
| `tests/frontend/day19_settings_smoke.js` | Settings tab switching calls `loadProviders()` and `loadAgentProviders()`. | PASS previously and present in regression set; indirect only. |
| `tests/frontend/day34_model_picker_smoke.js` | Model picker render/open/close/action wiring. | PASS; does not prove Provider save/delete safety. |
| `docs/frontend/FRONTEND-SMOKE-COVERAGE-GAP-V2E.md` | Marks provider list/modal/backup/probe gaps. | Provider list GAP; modal/probe WEBVIEW-ONLY. |
| `docs/frontend/PROVIDER-CONFIG-V1.5-SLICE-PLAN.md` | Prior Provider sampling. | Confirms Provider save/delete/probe/keyring high risk. |

### Provider Migration Candidates

| Candidate | Recommendation |
|---|---|
| `updateCapabilityStatusDisplay()` pure formatting | Batch 2 RISKY after targeted tests; must not invoke probe. |
| Provider list read-only render without edit/delete wiring | Batch 2 RISKY; needs safe DOM smoke and explicit no-delete/no-save assertions. |
| Provider modal open/close/field fill only | Batch 2 RISKY; API key field and probe status make this sensitive. |
| Provider save/delete/backup/probe/validate/keyring | FORBIDDEN / KEEP-FOR-NOW. |
| Agent provider binding write paths | FORBIDDEN / KEEP-FOR-NOW. |

## 4. Inspector Inventory

### Inspector Functions

| Function | Location | Role | Classification |
|---|---:|---|---|
| `init(app)` | `src/interface/web/modules/inspector.js:4` | Binds inspector tabs and close button. | SAFE |
| `showInspectorTab(app, tabId)` | `inspector.js:22` | Toggles tabs/panels, triggers diff/trace render. | SAFE-CANDIDATE |
| `withInspectorGuard(app, label, renderFn)` | `inspector.js:37` | Guard wrapper for render failures. | SAFE |
| `safeUpdateTaskDetails()`, `safeRenderContextFiles()`, `safeRenderModelInfo()`, `safeRenderInspectorDiffPreview()`, `safeRenderTraceInspector()` | `inspector.js:45-62` | Thin guarded wrappers. | SAFE-CANDIDATE |
| `openDiffPreview(app, file)` | `inspector.js:65` | Opens inspector and diff tab, stores `currentDiffFile`. | RISKY |
| `updateTaskDetails(app, statusText)` | `inspector.js:72` | Updates status and session stats. | SAFE |
| `renderTaskSteps(app)` | `inspector.js:82` | Updates session stats. | SAFE |
| `renderEditSummary(app)` | `inspector.js:86` | Renders edit payload summary via innerHTML. | RISKY |
| `renderContextFiles(app)` | `inspector.js:103` | Renders chat context file names. | RISKY |
| `renderModelInfo(app)` | `inspector.js:117` | Renders active provider/model. | RISKY |
| `renderInspectorDiffPreview(app)` | `inspector.js:134` | Renders diff preview and old diff button. | RISKY |
| `renderTraceInspector(app)` | `inspector.js:191` | Renders trace list and checkpoint restore/compare buttons. | KEEP-FOR-NOW |
| `loadLatestReceipt(app)` | `inspector.js:266` | Calls `get_latest_receipt`. | RISKY |
| `renderContextReceiptPanel(app, receipt)` | `inspector.js:276` | Renders Context Receipt. | RISKY |
| `setupReceiptPanel(app)` | `inspector.js:334` | Binds refresh button and loads receipt. | RISKY |
| `inspector-controller.js`, `inspector-view.js` | `src/interface/web/controllers/inspector-controller.js:1`, `views/inspector-view.js:1` | Skeleton only. | UNKNOWN |

### Inspector DOM Selectors / IDs

| DOM / Selector | Location | Role |
|---|---:|---|
| `rightInspector` | `index.html:374` | Inspector shell. |
| `inspectorCloseBtn` | `index.html:377` | Close button. |
| `.inspector-tab`, `[data-inspector-tab]` | `index.html:379-383` | Tab controls. |
| `.inspector-panel`, `[data-inspector-panel]` | `index.html:387-438` | Panel targets. |
| `inspectorTaskStatus`, `inspectorContextFiles`, `inspectorModelInfo`, `inspectorTaskSteps`, `inspectorOperationSummary`, `inspectorEditSummary` | `index.html:390-420` | Task/detail cards. |
| `inspectorDiffContent` | `index.html:426` | Diff panel target. |
| `inspectorTraceContent` | `index.html:432` | Trace panel target. |
| `contextReceiptTab`, `contextReceiptPanel`, `refreshReceiptBtn`, `contextReceiptBody` | `index.html:383`, `438-444` | Context receipt tab/panel. |

### Inspector State Fields

| State | Evidence | Role | Classification |
|---|---|---|---|
| `traceEvents` | `app.js:38`, `thinking-ui.js:266-288`, `inspector.js:195-200` | Agent trace rendering source. | KEEP-FOR-NOW due Agent streaming/checkpoint buttons. |
| `currentEditPayload` | `app.js:5215-5223`, `inspector.js:89-184` | Diff/edit summary source. | RISKY |
| `currentDiffFile` | `app.js:5223`, `inspector.js:66`, `138-147` | Diff fallback file path. | RISKY |
| `chatContextFiles` | `app.js:28`, `inspector.js:106-111` | Context file display. | RISKY |
| `providerConfigs`, `activeProviderId` | `app.js:14-15`, `inspector.js:120-128` | Model info display. | RISKY |

### Inspector Rendering / Security Paths

| Path | Evidence | Security note |
|---|---|---|
| Edit summary | `inspector.js:90`, `95` | Uses innerHTML but escapes summary. |
| Context files | `inspector.js:107`, `109-111` | Uses innerHTML and `app.escapeHtml(name)`. |
| Model info | `inspector.js:121`, `126-128` | Uses innerHTML and escapes provider/model text. |
| Diff preview | `inspector.js:142`, `151-184` | Uses innerHTML; escapes summary/file/line text. |
| Trace inspector | `inspector.js:196`, `203-243` | Uses innerHTML; escapes step/iteration/details, but restore/compare buttons call checkpoint functions. |
| Context receipt | `inspector.js:281`, `304-330` | Uses innerHTML; escapes provider/model/role/mode/omitted block fields. |

Security-gate allowlist currently accepts `src/interface/web/modules/inspector.js` innerHTML as legacy extracted inspector rendering debt. `day25_inspector_safety_smoke.js` verifies malicious diff/trace/receipt strings are escaped and not executable.

### Inspector Smoke Coverage

| Smoke | Result | Coverage |
|---|---|---|
| `tests/frontend/day18_inspector_smoke.js` | Existing smoke available | Tab switch, close button, diff fallback, receipt refresh. |
| `tests/frontend/day25_inspector_safety_smoke.js` | PASS (4 scenarios) | Diff/Trace/Context Receipt empty/mock data and malicious HTML escaping. |
| `npm run test:security-gate` | KNOWN FAIL | Inspector innerHTML entries are allowlisted warnings, not current failures. |

### Inspector Migration Candidates

| Candidate | Recommendation |
|---|---|
| Tab/close binding (`init`, `showInspectorTab`) | Batch 1 SAFE after node smoke. |
| Guard wrappers | Batch 1 SAFE. |
| Task status/session stats text-only render | Batch 1 SAFE. |
| Diff/Context Receipt rendering | Batch 2 RISKY after preserving day25 security assertions. |
| Trace checkpoint restore/compare buttons | KEEP-FOR-NOW; crosses checkpoint restore/compare boundary. |
| `loadLatestReceipt()` invoke path | Batch 2 RISKY; read-only IPC but depends on Tauri receipt command. |

## 5. Resource Dashboard Inventory

### Dashboard Functions

| Function | Location | Role | Classification |
|---|---:|---|---|
| `setupResourceDashboard(app)` | `src/interface/web/modules/resource-dashboard.js:15` | Calls `app.updateMetrics()` immediately, registers 3000ms interval. | SAFE |
| `updateMetrics(app)` | `resource-dashboard.js:20` | If Tauri unavailable, sets N/A; otherwise calls `get_resource_metrics` and updates three DOM nodes. | SAFE |
| `setMetric(id, value)` | `resource-dashboard.js:10` | Updates `textContent`. | SAFE |
| `setupResourceDashboard()` wrapper | `src/interface/web/app.js:4998` | Delegates to `window.HajimiResourceDashboard`. | SAFE |
| `updateMetrics()` wrapper | `src/interface/web/app.js:5431` | Delegates to `window.HajimiResourceDashboard`. | SAFE |
| `dashboard-view.js` | `src/interface/web/views/dashboard-view.js:1` | Skeleton only. | UNKNOWN |

### Dashboard DOM / State

| DOM / State | Location | Role | Classification |
|---|---:|---|---|
| `metricIterationTab` | `src/interface/web/index.html:266` | Iteration metric display. | SAFE |
| `metricBlackboardTab` | `src/interface/web/index.html:270` | Blackboard metric display. | SAFE |
| `metricEditCountTab` | `src/interface/web/index.html:274` | Edit count metric display. | SAFE |
| `metricsInterval` | `resource-dashboard.js:17` | Interval handle stored on app. | SAFE |

### Dashboard Tauri Invoke Commands

| Command | Caller | Classification | Notes |
|---|---|---|---|
| `get_resource_metrics` | `resource-dashboard.js:28` | SAFE | Read-only metric call. |

### Dashboard Smoke Coverage

| Smoke / Doc | Result | Coverage |
|---|---|---|
| `tests/frontend/day24_resource_dashboard_smoke.js` | PASS (8 scenarios) | Immediate refresh, 3000ms interval, no-Tauri N/A, success update, missing nodes no throw, invoke failure no throw, and only `get_resource_metrics` call. |
| `docs/frontend/DOM-CONTRACT.md` | PASS for metric IDs | Exact DOM IDs covered. |
| `docs/frontend/CSS-SPLIT-WEBVIEW-SMOKE-V3X.md` | PASS for dashboard visible | WebView visual note exists from CSS split phase. |

### Dashboard Migration Candidates

| Candidate | Recommendation |
|---|---|
| Move module body toward `views/dashboard-view.js` or controller/view pair | Batch 1 SAFE, because current module is already small and read-only. |
| Keep `get_resource_metrics` as explicit command, not generalized IPC | Batch 1 SAFE with current command-specific smoke. |
| Expand metrics beyond current three DOM IDs | UNKNOWN; requires new backend contract sampling. |

## 6. Tauri Invoke / IPC Map

Frontend invoke helpers:

| Helper | Location | Role |
|---|---:|---|
| `getTauriBridge()` | `src/interface/web/app.js:144` | Reads Tauri bridge/global. |
| `isTauriAvailable()` | `src/interface/web/app.js:150` | Checks bridge availability. |
| `getTauriInvoke()` | `src/interface/web/app.js:154` | Returns invoke function source. |
| `invokeTauri(command, args)` | `src/interface/web/app.js:158` | App-level async invoke wrapper. |
| `getTauriChannel()` | `src/interface/web/app.js:162` | Channel constructor source. |
| `HajimiTauriBridge.invoke()` | `src/interface/web/modules/tauri-bridge.js:21` | Browser module bridge wrapper. |

### Day6 Domain IPC Commands

| Command name | Caller file | Domain | Error handling | Classification | Notes |
|---|---|---|---|---|---|
| `get_provider_configs` | `app.js:3572` | Provider / Keyring | catch fallback to `[]` and toast | RISKY | Read-only but provider model source. |
| `get_probe_result` | `app.js:4040` | Provider / probe | `.catch()` fallback status | RISKY | Read-only status. |
| `probe_provider_context_capacity` | `app.js:3905`, `3939` | Provider / probe | interval/catch updates probe UI | FORBIDDEN | Executes probe. |
| `add_provider_config` / `update_provider_config` | `app.js:4239-4240` | Provider / Keyring | try/catch toast, clears key | FORBIDDEN | Writes provider config/key. |
| `delete_provider_config` | `app.js:4261` | Provider / Keyring | confirm + try/catch | FORBIDDEN | Delete path. |
| `validate_provider` | `app.js:5557` | Provider / test | try/catch toast | FORBIDDEN | Runs validation. |
| `export_provider_backup` / `import_provider_backup` | `app.js:4175`, `4185` | Provider / backup | try/catch toast | FORBIDDEN | Key material backup. |
| `get_agent_providers` | `app.js:4344` | Provider / agent binding | try/catch | RISKY | Read-only map. |
| `set_agent_provider` | `app.js:4382`, `4398` | Provider / agent binding | try/catch | FORBIDDEN | Binding write. |
| `get_resource_metrics` | `resource-dashboard.js:28` | Dashboard / metrics | catch no-throw | SAFE | Read-only; day24 covered. |
| `get_latest_receipt` | `inspector.js:269` | Inspector / receipt | catch fallback render null | RISKY | Read-only; receipt rendering has security path. |

### Other Frontend IPC Commands By Boundary

| Category | Commands / Evidence | Classification |
|---|---|---|
| Shell | `execute_tool` via `app.js:172`, `run_agent_command` at `app.js:5337` | FORBIDDEN |
| Checkpoint | `list_checkpoints`, `restore_checkpoint`, `export_checkpoint`, `compare_checkpoints` at `app.js:4860-4991` | FORBIDDEN |
| Agent streaming | `run_agent_task`, `stream_chat`, `record_stream_diagnostic`, `subscribe_agent_trace` at `app.js:2829`, `3104`, `3367`; `thinking-ui.js:287` | FORBIDDEN / KEEP-FOR-NOW |
| Audit | `get_audit_logs` in `modules/audit-log.js:40` | SAFE read-only; day23 PASS |
| Workspace/files | `get_current_workspace`, `list_dir`, `create_dir`, `rename_path`, `delete_path`, `read_file`, `write_file`, `apply_edits` | OUT OF DAY6-A; many are write/high-risk |
| Chat/context tools | `optimize_context`, `list_tools`, `read_file`, `stream_chat` | KEEP-FOR-NOW |
| Profile/MCP/approval/edit history | `list_profiles`, `set_active_profile`, `create_profile`, `delete_profile`, MCP calls, `resolve_agent_approval`, `get_edit_history` | OUT OF DAY6-A / RISKY |

Backend command registration evidence was found in `src/interface/desktop/src/main.rs:3678-3732` for Provider, Dashboard, Inspector receipt, Audit, Agent trace, Checkpoint, and metrics commands. No backend file was modified.

## 7. Security Boundary Table

| Boundary | Day6-A Status |
|---|---|
| Provider save/delete/probe logic | Not modified; FORBIDDEN / KEEP-FOR-NOW |
| Keyring | Not modified; FORBIDDEN |
| Shell execution | Not modified; FORBIDDEN |
| Checkpoint restore/export/compare/replay | Not modified; FORBIDDEN |
| CSP | Not modified; FORBIDDEN |
| withGlobalTauri | Not modified; FORBIDDEN |
| Agent streaming | Not modified; FORBIDDEN |
| `src/interface/desktop/src/main.rs` | Not modified |
| `src/interface/desktop/tauri.conf.json` | Not modified |

## 8. Smoke Coverage Matrix

| Area | Existing smoke / command | Result | Coverage note |
|---|---|---|---|
| Audit log | `node tests/frontend/day23_audit_log_smoke.js` | PASS (9 scenarios) | Read-only audit load/render, escaping. |
| Resource dashboard | `node tests/frontend/day24_resource_dashboard_smoke.js` | PASS (8 scenarios) | Exact dashboard metric DOM and only `get_resource_metrics`. |
| Inspector safety | `node tests/frontend/day25_inspector_safety_smoke.js` | PASS (4 scenarios) | Tab switching, diff/trace/receipt escaping. |
| Inspector legacy module | `tests/frontend/day18_inspector_smoke.js` | Existing coverage | Diff fallback and receipt refresh path. |
| Settings/provider tab | `tests/frontend/day19_settings_smoke.js` | Existing coverage | Indirect: provider tab calls loaders; no save/delete/probe. |
| Model picker | `tests/frontend/day34_model_picker_smoke.js` | Existing coverage | Render/open/close/action wiring; not Provider write safety. |
| Security gate | `npm run test:security-gate` | KNOWN FAIL | 3 failures only: `command-palette-view.js:36`, `session-list-view.js:22`, `session-list-view.js:26`; 106 allowlisted warnings. |

No Node smoke is claimed as a full WebView or production-readiness proof in this sampling report.

## 9. Day6 Migration Recommendation

### Batch 1 SAFE Candidates

| Candidate | Why |
|---|---|
| Resource dashboard view/controller cleanup around `setupResourceDashboard` and `updateMetrics` | Already isolated in `modules/resource-dashboard.js`, read-only, day24 proves only `get_resource_metrics`. |
| Inspector tab/close binding and guard wrappers | Mostly DOM toggle/guard logic; day25 covers tab switching and empty states. |
| Inspector text-only task status/session stats wrappers | Low write surface; avoid trace checkpoint buttons and receipt invoke in first cut. |

### Batch 2 RISKY Candidates After Extra Smoke

| Candidate | Required extra evidence |
|---|---|
| Inspector diff and Context Receipt rendering split | Preserve day25 malicious HTML assertions and security-gate status. |
| Inspector `loadLatestReceipt()` read-only IPC | Add command-specific smoke for `get_latest_receipt` error and empty receipt behavior. |
| Provider `updateCapabilityStatusDisplay()` pure formatter | Add pure formatting smoke; do not execute `probe_provider_context_capacity`. |
| Provider list read-only render without edit/delete handlers | Add DOM smoke proving no save/delete/probe/keyring execution. |
| Provider modal open/close/field-fill only | Add WebView/Node receipt that does not touch save/test/delete/probe and handles API-key placeholder safely. |

### FORBIDDEN / KEEP-FOR-NOW

| Area | Reason |
|---|---|
| `saveProviderConfig()` / add/update Provider | Writes Provider config and API key/keyring material. |
| `deleteProviderConfig()` | Deletes Provider config and key material. |
| `probe_provider_context_capacity` path | Can execute provider/network probe and uses cancellation/timer state. |
| `validate_provider` / `testProviderBtn` | Executes Provider validation. |
| Provider backup import/export | Key material boundary. |
| Agent provider binding writes | Updates provider mapping. |
| Shell execution | Explicit forbidden boundary. |
| Checkpoint restore/export/compare/replay | Explicit forbidden boundary. |
| Agent streaming / `streamChat` / `handleAgentEvent` | Explicit forbidden boundary. |
| Generalized Tauri invoke service extraction | Map first; do not blindly centralize every IPC call. |

### Suggested Day6-B Task

Extract or re-home Resource Dashboard into a small `dashboard-view`/controller slice, preserving current `HajimiResourceDashboard` public API and day24 smoke. Do not generalize IPC beyond `get_resource_metrics`.

### Suggested Day6-C Task

Inspector tab shell extraction only: `init`, `showInspectorTab`, guard wrappers, and text-only status wrappers. Keep trace checkpoint restore/compare and Provider/model receipt rendering out until a separate security-rendering pass.

### Provider Recommendation

Do not start Provider migration yet. First add a read-only Provider DOM smoke that opens/closes the Provider modal, checks list rendering, and asserts no save/delete/probe/validate/keyring command can fire during the smoke.
