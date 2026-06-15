# STONE-AUDIT-V3X-DAY04 Dashboard Browser Wiring + Compatibility Fallback Removal

Date: 2026-06-15

## Scope

Wire the Resource Dashboard view/controller into the browser script path and remove the proven compatibility fallback from `modules/resource-dashboard.js`.

Human summary: the dashboard now uses the new display/controller files directly. The old backup renderer inside the module was removed after Node and WebView proof.

## Branch / Baseline

- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `f7b97bc4352bf0f4ebc0496cc7912263cc72d7a7`
- `git pull --ff-only origin stone-audit-v3x-controlled-demolition`: already up to date

## Modified Files

- `src/interface/web/index.html`
- `src/interface/web/modules/resource-dashboard.js`
- `tests/frontend/day24_resource_dashboard_smoke.js`
- `docs/frontend/DASHBOARD-WEBVIEW-WIRING-V3X.md`

## Script Order

Verified by:

```powershell
rg -n "dashboard-view|dashboard-controller|resource-dashboard|app\.js" src/interface/web/index.html
```

Observed order:

```text
641:  <script defer src="views/dashboard-view.js"></script>
642:  <script defer src="controllers/dashboard-controller.js"></script>
643:  <script defer src="modules/resource-dashboard.js"></script>
660:  <script defer src="app.js"></script>
```

## Fallback Removal

- Removed full inline `compatView` fallback: YES
- Removed full inline `compatController` fallback: YES
- Kept stable public API `window.HajimiResourceDashboard`: YES
- Kept minimal missing-controller guard: YES
- Fallback used after WebView reload: NO

The module now delegates to `window.HajimiDashboardController` when it exists. If the controller script is missing, `setupResourceDashboard(app)` and `updateMetrics(app)` safely no-op instead of recreating the old DOM render body.

## Node Checks

```text
node --check src/interface/web/modules/resource-dashboard.js: PASS
node --check src/interface/web/views/dashboard-view.js: PASS
node --check src/interface/web/controllers/dashboard-controller.js: PASS
node --check tests/frontend/day24_resource_dashboard_smoke.js: PASS
```

## Smoke Results

```text
node tests/frontend/day24_resource_dashboard_smoke.js
day24 resource dashboard smoke: PASS (controller path + missing-controller guard)
```

day24 now proves:

- setup still refreshes immediately.
- setup still registers a 3000ms interval.
- Tauri unavailable still renders `N/A` through the controller/view path.
- `get_resource_metrics` success updates `metricIterationTab`, `metricBlackboardTab`, and `metricEditCountTab`.
- missing DOM nodes do not throw.
- `get_resource_metrics` failure does not throw.
- checkpoint/provider/agent/shell functions are not touched.
- missing dashboard controller no-ops without invoking backend or rendering the old fallback.

## Security Gate

```text
npm run test:security-gate
Security Audit Gate V1 summary
findings: 97
failures: 3
warnings: 94
allowlisted: 94
```

Result: FAIL, known pre-existing baseline. No new Dashboard failure was introduced.

Existing failures:

```text
src/interface/web/views/command-palette-view.js:36
src/interface/web/views/session-list-view.js:22
src/interface/web/views/session-list-view.js:26
```

## WebView Receipt

Startup command shape:

```powershell
npx serve . -p 3456
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 cargo tauri dev
```

Observed via WebView2 CDP target:

```text
title: Hajimi Code
url: http://localhost:3456/
readyState: complete
```

Dashboard globals after reload:

```text
window.HajimiDashboardView.renderMetrics: function
window.HajimiDashboardController.updateMetrics: function
window.HajimiResourceDashboard.updateMetrics: function
window.app.setupResourceDashboard: function
window.app.updateMetrics: function
window.app.metricsInterval: true
```

Dashboard visibility after `window.app.showSidebar('settings')` and `window.app.switchSettingsTab('audit')`:

```text
metricIterationTab exists: true, visible: true, text: 0
metricBlackboardTab exists: true, visible: true, text: 0
metricEditCountTab exists: true, visible: true, text: 0
whiteScreen: false
fallbackBodyPresent: false
```

WebView result:

```text
app launch: PASS
Dashboard visible: PASS
metrics visible: PASS
globals observed: YES
fallback used: NO
white screen: NO
crash: NO
no response: NO
command-specific error: NO
```

## Forbidden Diff

Verified by:

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/style.css src/interface/web/styles src/interface/desktop src/engine/tool-system
```

Result:

```text
no output
```

Forbidden areas not touched:

- `src/interface/web/app.js`
- `src/interface/web/style.css`
- `src/interface/web/styles/**`
- `src/interface/desktop/**`
- `src/engine/tool-system/**`
- Provider / Keyring
- Checkpoint
- Shell
- Agent streaming

## Old Dirty Files

Old dirty files remain in the working tree and were not staged for this task.

## Production Change Summary

- `index.html`: added two Dashboard script tags before `modules/resource-dashboard.js`.
- `modules/resource-dashboard.js`: removed inline DOM/controller compatibility fallback and kept only public API delegation/guard.
- `day24_resource_dashboard_smoke.js`: updated the final scenario from old fallback rendering to missing-controller safe no-op.

## Rollback

Rollback point:

```text
f7b97bc4352bf0f4ebc0496cc7912263cc72d7a7
```
