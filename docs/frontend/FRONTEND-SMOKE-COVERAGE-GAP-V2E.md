# STONE-AUDIT-V2E-FRONTEND-SMOKE-COVERAGE-GAP-SAMPLING

Date: 2026-06-06
Repo: Cognitive-Architect/hajimi-code-cli
Branch: feature/toolfix-deepseek-schema
Base HEAD: 3da2acb8db20826338fa8a62648e52da2a1554ed

## Scope

Readonly sampling only. This report reviews existing evidence from:

- `docs/frontend/DOM-CONTRACT.md`
- `tests/frontend/*.js`
- `src/interface/web/modules/*.js`
- Targeted readonly references in `src/interface/web/index.html` and `src/interface/web/app.js`

No production code was modified. No smoke was added. No fix was performed.

## Baseline Evidence

`docs/frontend/DOM-CONTRACT.md` records the current DOM contract baseline:

```text
day26 dom contract smoke: PASS
html ids: 162
js id refs: 177
css id refs: 1
PASS ids: 33
MISSING ids: 46 (known: 46, unexpected: 0)
ORPHAN ids: 31
UNCOVERED ids: 98
```

Plain language: `UNCOVERED` means the page element exists and code uses it, but the current frontend smoke files do not mention it directly. It is like a shelf item that exists and is used, but no inspection receipt names it.

## Coverage Gap Fact Table

| Area | DOM / Module / Selector | Current evidence | Coverage classification | Evidence pointer | Suggested next evidence type | Boundary note |
| --- | --- | --- | --- | --- | --- | --- |
| DOM contract total | 98 UNCOVERED ids | `DOM-CONTRACT.md` states 98 IDs exist and are JS-referenced but not mentioned by frontend smoke files. | GAP | `docs/frontend/DOM-CONTRACT.md:42-50`, `docs/frontend/DOM-CONTRACT.md:188-199` | One readonly DOM smoke per focused area, not one giant test. | Record only; do not repair DOM or broaden production code. |
| Command palette DOM | `commandPalette`, `commandInput`, `commandList` | Command catalog is covered, but modal DOM is not directly smoked. | GAP | `docs/frontend/DOM-CONTRACT.md:67-69`, `src/interface/web/app.js:5107-5141`, `tests/frontend/day22_command_palette_catalog_smoke.js:156-195` | Node DOM smoke for open, filter, render, Escape/close, and click action wiring. | Do not rewrite `setupCommandPalette`, `showCommandPalette`, or command actions. |
| Command palette catalog | `modules/command-palette-catalog.js` | Catalog factory and command actions are covered. DOM shell is not. | INDIRECT | `tests/frontend/day22_command_palette_catalog_smoke.js:47-62`, `tests/frontend/day22_command_palette_catalog_smoke.js:156-195` | Pair existing catalog smoke with a separate DOM smoke. | Existing catalog PASS must not be counted as modal DOM PASS. |
| Sessions sidebar | `sessionList` / `modules/sessions.js` | Sessions module smoke exists, but `sessionList` exact DOM ID is not directly mentioned by smoke coverage. | NAMING-GAP | `docs/frontend/DOM-CONTRACT.md:79`, `src/interface/web/modules/sessions.js:131-145`, `tests/frontend/day14_sessions_thinking_modules_smoke.js:374-375` | Node DOM smoke that explicitly registers and asserts `sessionList` render behavior. | Keep storage/localStorage-only; do not add backend session behavior. |
| Provider settings list | `providerListTab` | DOM contract marks it UNCOVERED. App renders provider rows and edit/delete buttons. | GAP | `docs/frontend/DOM-CONTRACT.md:80`, `src/interface/web/index.html:167-178`, `src/interface/web/app.js:3725-3757` | Readonly DOM smoke for empty/list render only, if separately approved. | Do not test Keyring, save, delete, validate provider, or backup. |
| Provider modal fields | `providerModal`, `providerApiKey`, `providerMaxContext`, `testProviderBtn` | DOM contract groups Provider settings as UNCOVERED. App binds save/test/probe paths. | WEBVIEW-ONLY | `docs/frontend/DOM-CONTRACT.md:195`, `src/interface/web/index.html:482-598`, `src/interface/web/app.js:3762-4027`, `src/interface/web/app.js:5577-5612` | Separate manual/WebView readonly smoke for open/close/visibility only. | Do not submit save/test/delete; do not touch Provider / Keyring. |
| Backup/probe modal | `backupModal`, `backupPassword`, `probeDetailStatus`, `providerProbeStatusDetails` | DOM contract groups backup/probe as UNCOVERED and warns to keep separate from provider/keyring-sensitive work. | WEBVIEW-ONLY | `docs/frontend/DOM-CONTRACT.md:199`, `src/interface/web/index.html:583-620`, `src/interface/web/app.js:3842-3858`, `src/interface/web/app.js:3865-4027` | Separate observation-only WebView smoke for open/close labels and disabled states. | Do not run backup import/export or capacity probe. |
| Settings panel module | `modules/settings-panel.js`, `.settings-tab`, `.settings-tab-panel` | Settings panel behavior has a module smoke; DOM IDs inside provider panel are not covered by that smoke. | INDIRECT | `src/interface/web/modules/settings-panel.js:9-68`, `tests/frontend/day19_settings_smoke.js:73-128` | Keep existing settings smoke; add focused DOM smokes only for safe tabs. | Provider tab click calls provider loaders; avoid Provider/Keyring writes. |
| Security DOM module | `modules/security-dom.js`, `HajimiSecurityDom` | Escaping helpers are directly asserted across existing module smokes. File naming is indirect because coverage appears in several feature smokes. | NAMING-GAP | `src/interface/web/modules/security-dom.js:4-31`, `tests/frontend/day13_workspace_modules_smoke.js:118-124`, `tests/frontend/day17_thinking_ui_v2_security_smoke.js:82-88` | Optional tiny module-name smoke only if naming traceability matters. | Do not loosen escaping. |
| Slash command catalog | `modules/slash-command-catalog.js`, `HajimiSlashCommandCatalog` | App integration smoke mounts catalog and validates slash command triggers; catalog file name is covered through day21 rather than a standalone catalog-named test. | NAMING-GAP | `src/interface/web/modules/slash-command-catalog.js:4-24`, `tests/frontend/day21_slash_palette_app_integration_smoke.js:8`, `tests/frontend/day21_slash_palette_app_integration_smoke.js:123`, `tests/frontend/day21_slash_palette_app_integration_smoke.js:255-280` | Optional standalone module smoke only if file-name traceability is required. | Do not move command execution branches. |
| Slash palette DOM | `slashPalette`, `.slash-palette-*` | Slash palette and app integration are covered. | PASS | `docs/frontend/DOM-CONTRACT.md:66`, `tests/frontend/day16_slash_palette_smoke.js:214-317`, `tests/frontend/day21_slash_palette_app_integration_smoke.js:193-280` | No immediate gap for the known slash palette path. | Keep high-risk slash commands fill-only unless separately changed. |
| Tauri bridge module | `modules/tauri-bridge.js`, `HajimiTauri.Channel`, `HajimiTauri.listen` | Channel envelope and event listen bridge have focused tests. General real-WebView bridge availability remains environment-bound. | INDIRECT | `src/interface/web/modules/tauri-bridge.js:21-75`, `tests/frontend/day20_tauri_channel_envelope_regression.test.js:49-145`, `tests/frontend/day21_event_bridge.test.js:49-64` | Keep Node bridge tests plus separate WebView receipts for real event availability. | Do not change CSP / withGlobalTauri. |
| Handle chat command WebView | `/compact`, `/search`, `/chat dummy` | Real Tauri WebView smoke exists as a documentation receipt, not a Node smoke. | WEBVIEW-ONLY | `docs/frontend/HANDLE-CHAT-COMMAND-WEBVIEW-SMOKE.md:44-54`, `docs/frontend/HANDLE-CHAT-COMMAND-WEBVIEW-SMOKE.md:76-98` | Keep as WebView-only receipt; add Node smoke only for static/parser cases. | `/chat` did not call real Provider; Provider Keyring intentionally avoided. |
| Audit log | `auditLogBodyTab`, `refreshAuditBtnTab`, `modules/audit-log.js` | Readonly render and refresh behavior are covered. | PASS | `docs/frontend/DOM-CONTRACT.md:70-71`, `tests/frontend/day23_audit_log_smoke.js:171-236` | No immediate gap for readonly audit log path. | Do not add audit write/delete behavior. |
| Resource dashboard | `metricIterationTab`, `metricBlackboardTab`, `metricEditCountTab` | Readonly metrics refresh is covered and guarded against provider/checkpoint/agent/shell calls. | PASS | `docs/frontend/DOM-CONTRACT.md:72-74`, `tests/frontend/day24_resource_dashboard_smoke.js:69-87`, `tests/frontend/day24_resource_dashboard_smoke.js:131-165` | No immediate gap for three current metrics. | Do not touch checkpoint/provider/agent/shell. |
| Inspector major panels | `rightInspector`, `inspectorTraceContent`, `contextReceiptBody` | Major inspector panels and safety rendering are covered. Detail IDs are not exhaustive. | INDIRECT | `docs/frontend/DOM-CONTRACT.md:75-77`, `tests/frontend/day18_inspector_smoke.js:93-178`, `tests/frontend/day25_inspector_safety_smoke.js:98-271` | Only add detail-field smoke if a specific user path needs it. | Do not refactor Agent streaming or checkpoint replay. |
| Inspector detail fields | `inspectorTaskStatus`, `inspectorTaskSteps`, `inspectorOperationSummary` | DOM contract groups detail fields as UNCOVERED despite major panel coverage. | INDIRECT | `docs/frontend/DOM-CONTRACT.md:198` | Focused readonly render-state smoke if needed. | Do not claim full inspector detail coverage from panel-level tests. |
| Model picker | `modelPickerModal`, `modelPickerBody`, `modelPickerClose`, `modelPickerAddBtn` | DOM contract groups model picker as UNCOVERED candidate. | GAP | `docs/frontend/DOM-CONTRACT.md:196` | Readonly open/close/render smoke. | Avoid Provider save/delete/test. |
| Sidebar/status display | `topBarProject`, `topBarBranch`, `statusBranch`, `statusModel`, `statusTokens` | DOM contract groups status display as UNCOVERED. | GAP | `docs/frontend/DOM-CONTRACT.md:197` | Readonly display-state smoke. | Do not require real Git/model calls unless mocked. |

## WebView-Only Risk Notes

The following evidence must not be described as Node smoke coverage:

- `docs/frontend/HANDLE-CHAT-COMMAND-WEBVIEW-SMOKE.md` covers real WebView interaction for `/compact`, `/search`, and `/chat dummy`.
- Provider modal, backup modal, and probe controls are currently treated as WebView-only or future separated smoke areas because meaningful execution can cross Provider / Keyring or cost/probe boundaries.
- Real Tauri bridge event availability can differ from Node mocks; existing Node tests cover bridge shaping, not every desktop runtime condition.

## Current Test Inventory

Observed frontend test files:

```text
tests/frontend/agent_governance_approval_smoke.js
tests/frontend/agent_result_rendering_smoke.js
tests/frontend/agent_thinking_leak_smoke.js
tests/frontend/day13_workspace_modules_smoke.js
tests/frontend/day14_sessions_thinking_modules_smoke.js
tests/frontend/day16_slash_palette_smoke.js
tests/frontend/day17_thinking_ui_v2_security_smoke.js
tests/frontend/day18_inspector_smoke.js
tests/frontend/day19_settings_smoke.js
tests/frontend/day20_tauri_channel_envelope_regression.test.js
tests/frontend/day21_event_bridge.test.js
tests/frontend/day21_slash_palette_app_integration_smoke.js
tests/frontend/day22_command_palette_catalog_smoke.js
tests/frontend/day23_audit_log_smoke.js
tests/frontend/day24_resource_dashboard_smoke.js
tests/frontend/day25_inspector_safety_smoke.js
tests/frontend/day26_dom_contract_smoke.js
tests/frontend/day27_handle_chat_command_invoke_smoke.js
tests/frontend/static_web_server.js
tests/frontend/thinking_stream_parser.test.js
```

Observed frontend module files:

```text
src/interface/web/modules/audit-log.js
src/interface/web/modules/command-palette-catalog.js
src/interface/web/modules/inspector.js
src/interface/web/modules/resource-dashboard.js
src/interface/web/modules/security-dom.js
src/interface/web/modules/sessions.js
src/interface/web/modules/settings-panel.js
src/interface/web/modules/slash-command-catalog.js
src/interface/web/modules/slash-palette.js
src/interface/web/modules/tauri-bridge.js
src/interface/web/modules/thinking-ui.js
src/interface/web/modules/workspace.js
```

## Validation Log

`git status --short` before this report was staged:

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
?? "docs/roadmap/hajimi template/"
?? src/interface/desktop/native-smoke.txt
```

Production/frontend source diff check:

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/style.css src/interface/web/modules
```

Observed result before this report was written:

```text

```

Meaning: no production frontend source diff was present before writing this report.

`git diff --cached --check` before staging:

```text
PASS
```

## Next Candidates Only

1. Add a command palette DOM smoke for `commandPalette`, `commandInput`, and `commandList`.
2. Add a sessions DOM smoke that explicitly names `sessionList`.
3. Add a provider settings readonly DOM visibility smoke only if the task explicitly excludes Provider / Keyring save/delete/test/probe actions.
4. Add a model picker open/close/render smoke.
5. Keep WebView-only receipts separate from Node smoke coverage in future reports.
