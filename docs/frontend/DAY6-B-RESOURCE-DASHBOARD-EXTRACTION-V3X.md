# DAY6-B Resource Dashboard Extraction V3X

## Scope

- Task: `STONE-AUDIT-V3X Day6-B Resource Dashboard Small Extraction`
- Source issue: `https://github.com/Cognitive-Architect/hajimi-code-cli/issues/8`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `8477c2cfa020932ebf5faee414b1ddc5d313b38c`
- Date: `2026-06-12`

## Goal

Re-home the read-only Resource Dashboard metrics path into a small view/controller slice while preserving the existing public browser API:

- `window.HajimiResourceDashboard.setupResourceDashboard(app)`
- `window.HajimiResourceDashboard.updateMetrics(app)`

No backend metrics contract was changed. The only backend command still used by this path is:

- `get_resource_metrics`

## Files Changed

- `src/interface/web/modules/resource-dashboard.js`
- `src/interface/web/views/dashboard-view.js`
- `src/interface/web/controllers/dashboard-controller.js`
- `tests/frontend/day24_resource_dashboard_smoke.js`
- `docs/frontend/DAY6-B-RESOURCE-DASHBOARD-EXTRACTION-V3X.md`

## Implementation Notes

### View

`src/interface/web/views/dashboard-view.js` now exposes:

- `window.HajimiDashboardView.setMetric(id, value)`
- `window.HajimiDashboardView.renderMetrics(metrics)`
- `window.HajimiDashboardView.renderUnavailable()`

The view only writes Resource Dashboard DOM text for these fixed IDs:

- `metricIterationTab`
- `metricBlackboardTab`
- `metricEditCountTab`

### Controller

`src/interface/web/controllers/dashboard-controller.js` now exposes:

- `window.HajimiDashboardController.setupResourceDashboard(app)`
- `window.HajimiDashboardController.updateMetrics(app)`

The controller keeps the existing behavior:

- refresh immediately through `app.updateMetrics()`
- register a `3000ms` interval
- use `app.invokeTauri('get_resource_metrics')`
- safely ignore metrics invoke failure
- render `N/A` when Tauri is unavailable

### Compatibility

`src/interface/web/modules/resource-dashboard.js` remains the public compatibility API.

It prefers `window.HajimiDashboardController` when available. Because this batch did not modify `src/interface/web/index.html`, it also retains a minimal local compatibility fallback so the existing browser script order cannot break the live app.

This fallback is Dashboard-only and does not introduce a generalized Tauri invoke service.

## Preserved Contract

- Existing `window.HajimiResourceDashboard` API remains present.
- Existing app wrappers in `src/interface/web/app.js` were not modified.
- DOM IDs were not changed.
- CSS was not changed.
- `get_resource_metrics` remains the only Tauri command used by the Resource Dashboard path.
- No Provider, Inspector, Shell, Checkpoint, Agent streaming, `main.rs`, or `tauri.conf.json` files were modified.

## Automated Checks

```text
node --check src/interface/web/modules/resource-dashboard.js
Result: PASS

node --check src/interface/web/views/dashboard-view.js
Result: PASS

node --check src/interface/web/controllers/dashboard-controller.js
Result: PASS

node --check tests/frontend/day24_resource_dashboard_smoke.js
Result: PASS

node tests/frontend/day24_resource_dashboard_smoke.js
Result: PASS
Output: day24 resource dashboard smoke: PASS (9 scenarios)

npm run test:security-gate
Result: KNOWN FAIL
Summary: findings 109; failures 3; warnings 106; allowlisted 106
Allowed legacy failures:
- src/interface/web/views/command-palette-view.js:36
- src/interface/web/views/session-list-view.js:22
- src/interface/web/views/session-list-view.js:26
New Dashboard failure: NO
```

## Forbidden Boundary Receipt

Command:

```text
git diff --name-only -- src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json src/interface/web/modules/inspector.js src/interface/web/controllers/inspector-controller.js src/interface/web/views/inspector-view.js src/interface/web/controllers/provider-controller.js src/interface/web/views/provider-view.js src/interface/web/services/provider-service.js src/engine/tool-system/src/shell.rs
```

Result:

```text
<no output>
```

Boundary summary:

- Production changes outside target: NO
- Forbidden diff count: `0`
- Provider save/delete/probe/keyring touched: NO
- Shell / Checkpoint / Agent streaming touched: NO
- `main.rs` / `tauri.conf.json` touched: NO
- Generalized Tauri invoke service added: NO
- Old dirty files staged at doc write time: NO

## Decisions

- DECISION-001: Keep `window.HajimiResourceDashboard` as the stable public API for app.js compatibility.
- DECISION-002: Land `HajimiDashboardView` and `HajimiDashboardController` as narrow Dashboard-only globals.
- DECISION-003: Retain a minimal compatibility fallback inside `resource-dashboard.js` because this batch did not include browser script-order wiring in `index.html`.

## Debt

- DEBT-COMPAT-DAY6-B: `resource-dashboard.js` still has a minimal Dashboard-only fallback until a future browser wiring pass explicitly loads `views/dashboard-view.js` and `controllers/dashboard-controller.js` before `modules/resource-dashboard.js`.
- DEBT-SECURITY-GATE-LEGACY: security-gate remains KNOWN FAIL for the three pre-existing unverified DOM HTML findings listed above.

## Next

Next batch:

- `Day6-C Inspector Shell Extraction`
