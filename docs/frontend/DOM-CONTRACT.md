# Frontend DOM Contract

> Task: `STONE-AUDIT-V2A-DOM-CONTRACT-SMOKE`
> Date: 2026-06-04
> Scope: read-only scan of `src/interface/web/index.html`, `app.js`, `modules/*.js`, `style.css`, and `tests/frontend/*.js`
> Smoke: `node tests/frontend/day26_dom_contract_smoke.js`

## 1. Result

`PASS / READONLY CONTRACT BASELINE RECORDED`

No production file was modified. This document records the current DOM binding facts and known gaps only.

Out of scope and untouched:

- `src/interface/web/index.html`
- `src/interface/web/app.js`
- `src/interface/web/style.css`
- `src/interface/web/modules/*.js`
- Provider / Keyring
- Shell execution
- Checkpoint restore / export / compare / replay
- Agent streaming
- CSP / `withGlobalTauri`

## 2. Status Rules

| Status | Meaning |
|---|---|
| `PASS` | DOM exists in `index.html`, JS references it, and an existing smoke mentions it. |
| `MISSING` | JS references the DOM ID, but `index.html` does not currently contain it. |
| `ORPHAN` | `index.html` contains the DOM ID, but the static JS scan did not find a literal reference. |
| `UNCOVERED` | DOM exists and JS references it, but no existing frontend smoke mentions it. |
| `UNKNOWN` | Static scan cannot reliably classify it, usually because the selector is dynamic. |

Plain language: this is a shelf-label check. `MISSING` means the shopping list asks for an item that is not on the shelf. `ORPHAN` means an item is on the shelf but no list asks for it. `UNCOVERED` means it is used, but there is no test receipt for it yet.

## 3. Scan Summary

Command evidence: `node tests/frontend/day26_dom_contract_smoke.js`

```text
day26 dom contract smoke: PASS
html ids: 162
js id refs: 177
css id refs: 1
PASS ids: 33
MISSING ids: 46 (known: 46, unexpected: 0)
ORPHAN ids: 31
UNCOVERED ids: 98
HTML classes: 162
JS class refs: 73
CSS class refs: 393
HTML data attrs: 7
JS data refs: 27
```

The smoke excludes its own file from coverage detection, so `day26_dom_contract_smoke.js` cannot make a selector look covered merely by naming it.

## 4. Key Contract Table

| DOM ID / Selector | Area | index.html | JS reference | CSS reference | Smoke coverage | Status | Notes |
|---|---|---:|---|---|---|---|---|
| `aiChatInput` | Chat composer | line 348 | `app.js:2342` | class based | `day21_slash_palette_app_integration_smoke.js` | `PASS` | Slash selection fills input without auto-send for high-risk commands. |
| `aiChatSendBtn` | Chat composer | line 349 | `app.js:2343` | class based | `day21_slash_palette_app_integration_smoke.js` | `PASS` | Send button is included in slash app integration smoke. |
| `slashPalette` | Chat composer | line 346 | `app.js:2344` | class based | `day16_slash_palette_smoke.js`, `day21_slash_palette_app_integration_smoke.js` | `PASS` | Existing low-risk slash palette contract. |
| `commandPalette` | Command palette | line 455 | `app.js:5106` | class based | none found | `UNCOVERED` | Command catalog is smoked, but modal DOM itself is not directly covered. |
| `commandInput` | Command palette | line 457 | `app.js:5107` | class based | none found | `UNCOVERED` | Same DOM coverage gap as command palette. |
| `commandList` | Command palette | line 458 | `app.js:5108` | class based | none found | `UNCOVERED` | Same DOM coverage gap as command palette. |
| `auditLogBodyTab` | Audit log | line 294 | `modules/audit-log.js:42` | class based | `day23_audit_log_smoke.js` | `PASS` | Read-only render path is covered. |
| `refreshAuditBtnTab` | Audit log | line 290 | `modules/audit-log.js:76` | class based | `day23_audit_log_smoke.js` | `PASS` | Refresh click binding is covered. |
| `metricIterationTab` | Resource dashboard | line 266 | `modules/resource-dashboard.js:22` | class based | `day24_resource_dashboard_smoke.js` | `PASS` | Detected through `setMetric(...)` wrapper. |
| `metricBlackboardTab` | Resource dashboard | line 270 | `modules/resource-dashboard.js:23` | class based | `day24_resource_dashboard_smoke.js` | `PASS` | Detected through `setMetric(...)` wrapper. |
| `metricEditCountTab` | Resource dashboard | line 274 | `modules/resource-dashboard.js:24` | class based | `day24_resource_dashboard_smoke.js` | `PASS` | Detected through `setMetric(...)` wrapper. |
| `rightInspector` | Inspector | line 374 | `modules/inspector.js:16` | class based | `day18_inspector_smoke.js`, `day25_inspector_safety_smoke.js` | `PASS` | Inspector open/close path covered. |
| `inspectorTraceContent` | Inspector trace | line 432 | `modules/inspector.js:192` | class based | `day18_inspector_smoke.js`, `day25_inspector_safety_smoke.js` | `PASS` | Trace rendering path covered. |
| `contextReceiptBody` | Inspector receipt | line 444 | `modules/inspector.js:277` | class based | `day18_inspector_smoke.js`, `day25_inspector_safety_smoke.js` | `PASS` | Context receipt rendering path covered. |
| `fileTree` | Workspace explorer | line 114 | `modules/workspace.js:109` | class based | `day13_workspace_modules_smoke.js` | `PASS` | Workspace tree render path covered. |
| `sessionList` | Sessions sidebar | line 66 | `modules/sessions.js:132` | class based | none found | `UNCOVERED` | Sessions module has smoke, but this exact DOM ID is not mentioned. |
| `providerListTab` | Provider settings | line 178 | `app.js:3723` | class based | none found | `UNCOVERED` | Provider config remains out of extraction scope; record only. |
| `tracePanel` | Legacy trace / replay | missing | `app.js:5367`, `modules/thinking-ui.js:775` | class based | `day14_sessions_thinking_modules_smoke.js` | `MISSING` | Known missing baseline; do not fix in this task. |
| `sessionReplayBar` | Replay bar | missing | `modules/thinking-ui.js:761` | `style.css` | none found | `MISSING` | Also the only non-color CSS ID selector found by the scan. |
| `settingsPanel` | Settings sidebar | line 120 | no literal JS ID ref found | class/data-panel based | `day19_settings_smoke.js` | `ORPHAN` | Likely reached via `data-panel="settings"` rather than ID lookup. |

## 5. Data Attribute Contract

| Data attribute | index.html | JS reference | Status | Notes |
|---|---:|---|---|---|
| `data-view` | line 38 | found | `PASS` | Activity bar view switching. |
| `data-panel` | line 54 | found | `PASS` | Sidebar panel switching. |
| `data-tab` | line 128 | found | `PASS` | Settings tabs. |
| `data-settings-panel` | line 136 | found | `PASS` | Settings tab panels. |
| `data-inspector-tab` | line 380 | found | `PASS` | Inspector tabs. |
| `data-inspector-panel` | line 387 | found | `UNKNOWN` | Present in HTML; dynamic panel switching makes static attribution conservative. |
| `data-theme` | line 2 | `UNKNOWN` | `UNKNOWN` | Theme state is on `<html>` and may be controlled outside direct query patterns. |

## 6. Known MISSING IDs

All missing IDs below are known baseline findings. The smoke passes because no unexpected missing DOM ID was found. No fix was applied.

| DOM ID | JS reference | CSS reference | Smoke coverage | Status |
|---|---|---|---|---|
| `acceptAllEditsBtn` | `app.js:5260` | none | none | `MISSING` |
| `aiChatModelSelect` | `app.js:2446` | none | none | `MISSING` |
| `bottomPanel` | `app.js:1613` | none | none | `MISSING` |
| `breadcrumbBar` | `app.js:1116` | none | none | `MISSING` |
| `checkpointCompareResultTab` | `app.js:4964` | none | none | `MISSING` |
| `closeEditPanelBtn` | `app.js:5262` | none | none | `MISSING` |
| `closePanelBtn` | `app.js:1612` | none | none | `MISSING` |
| `contextMenu` | `app.js:984` | none | none | `MISSING` |
| `editHistoryPanel` | `app.js:5368` | none | none | `MISSING` |
| `editorArea` | `app.js:1098` | none | none | `MISSING` |
| `extensionsList` | `app.js:4539` | none | none | `MISSING` |
| `gitBadge` | `app.js:690` | none | none | `MISSING` |
| `gitCommitActionBtn` | `app.js:658` | none | none | `MISSING` |
| `gitCommitBtn` | `app.js:5613` | none | none | `MISSING` |
| `gitCommitInput` | `app.js:660` | none | none | `MISSING` |
| `gitDiffClose` | `app.js:661` | none | none | `MISSING` |
| `gitDiffContent` | `app.js:739` | none | none | `MISSING` |
| `gitDiffFileName` | `app.js:738` | none | none | `MISSING` |
| `gitDiffView` | `app.js:737` | none | none | `MISSING` |
| `gitFileList` | `app.js:689` | none | none | `MISSING` |
| `gitRefreshBtn` | `app.js:659` | none | none | `MISSING` |
| `inlineEditHunks` | `app.js:5282` | none | none | `MISSING` |
| `inlineEditPanel` | `app.js:5280` | none | none | `MISSING` |
| `inlineEditSummary` | `app.js:5281` | none | none | `MISSING` |
| `inspectorOldDiffBtn` | `modules/inspector.js:146` | none | `day18`, `day25` mention | `MISSING` |
| `lspTooltip` | `app.js:4709` | none | none | `MISSING` |
| `maximizePanelBtn` | `app.js:1616` | none | none | `MISSING` |
| `outputContent` | `app.js:1910` | none | none | `MISSING` |
| `pauseTraceBtn` | `modules/thinking-ui.js:352` | none | none | `MISSING` |
| `problemsContent` | `app.js:1781` | none | none | `MISSING` |
| `rejectAllEditsBtn` | `app.js:5261` | none | none | `MISSING` |
| `replayCloseBtn` | `app.js:5377` | none | none | `MISSING` |
| `replayNextBtn` | `app.js:5376` | none | none | `MISSING` |
| `replayPrevBtn` | `app.js:5375` | none | none | `MISSING` |
| `replayStatus` | `modules/thinking-ui.js:801` | none | none | `MISSING` |
| `searchCaseSensitive` | `app.js:585` | none | none | `MISSING` |
| `searchInput` | `app.js:562` | none | none | `MISSING` |
| `searchRegex` | `app.js:586` | none | none | `MISSING` |
| `searchResults` | `app.js:574` | none | none | `MISSING` |
| `searchWholeWord` | `app.js:587` | none | none | `MISSING` |
| `sessionReplayBar` | `modules/thinking-ui.js:761` | `style.css` | none | `MISSING` |
| `settingAutoSave` | `app.js:2066` | none | none | `MISSING` |
| `statusCursor` | `app.js:5251` | none | none | `MISSING` |
| `tabBar` | `app.js:1019` | none | none | `MISSING` |
| `terminalContent` | `app.js:1649` | none | none | `MISSING` |
| `tracePanel` | `app.js:5367` | none | `day14` mention | `MISSING` |

## 7. ORPHAN IDs

These IDs exist in `index.html`, but the static JS ID scan found no literal ID lookup. Some may be referenced through classes, data attributes, or layout-only CSS.

```text
activityBar
appFooter
backupForm
chatContainer
chatHeader
chatSessionsPanel
composer
composerAttachments
contextReceiptPanel
contextReceiptTab
explorerPanel
governanceStatusTab
inspectorContent
inspectorTabs
inspectorTaskDetail
mainArea
mainColumn
providerSaveTargetField
sessionMoreBtn
settingsMoreBtn
settingsPanel
settingsTabs
sidebarContent
statusBar
statusElapsed
statusEncoding
statusSpinner
statusSync
topBarNotifyBtn
windowTopBar
workspace
```

## 8. UNCOVERED Groups

There are 98 IDs that exist in HTML and are referenced by JS but are not mentioned by existing frontend smoke files. Important groups:

| Group | Examples | Notes |
|---|---|---|
| Command palette DOM | `commandPalette`, `commandInput`, `commandList` | Catalog is covered, modal DOM itself is not directly smoked. |
| Provider settings | `providerModal`, `providerListTab`, `providerApiKey`, `providerMaxContext`, `testProviderBtn` | Provider config is intentionally not touched in this task. |
| Model picker | `modelPickerModal`, `modelPickerBody`, `modelPickerClose`, `modelPickerAddBtn` | Candidate for future read-only smoke only. |
| Sidebar status | `topBarProject`, `topBarBranch`, `statusBranch`, `statusModel`, `statusTokens` | Mostly read-only display binding. |
| Inspector detail fields | `inspectorTaskStatus`, `inspectorTaskSteps`, `inspectorOperationSummary` | Inspector module smokes cover major panels, not every detail ID. |
| Backup/probe modal | `backupModal`, `backupPassword`, `probeDetailStatus`, `providerProbeStatusDetails` | Keep separate from provider/keyring-sensitive work. |

## 9. Validation

| Command | Result |
|---|---|
| `node tests/frontend/day26_dom_contract_smoke.js` | PASS |
| `git diff --check` | PASS; warnings only for old dirty docs plus CRLF notice for this doc |
| `git diff --cached --check` | PASS; empty cached diff before staging |
| `git status --short` | PASS for isolation; old dirty files remain, this task only adds/updates the two allowed files |

## 10. Next Candidates Only

No implementation was performed. Candidate follow-ups:

1. Add a command palette DOM smoke for `commandPalette`, `commandInput`, and `commandList`.
2. Add a sessions DOM smoke that explicitly mentions `sessionList`.
3. Create a separate provider-config read-only DOM smoke only if the scope avoids Provider/Keyring writes.
4. Split legacy MISSING IDs by feature area before any production fix is attempted.
