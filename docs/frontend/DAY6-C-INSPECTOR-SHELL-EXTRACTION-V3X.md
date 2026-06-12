# DAY6-C Inspector Shell Extraction V3X

## Scope

- Task: `STONE-AUDIT-V3X Day6-C Inspector Shell Extraction`
- Source issue: `https://github.com/Cognitive-Architect/hajimi-code-cli/issues/8`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `c3585896d7da765b85b94af86a462dd1711d0b79`
- Date: `2026-06-12`

## Goal

Extract only the Inspector shell path into a narrow view/controller slice:

- tab click binding
- close button binding
- active tab / panel toggling
- guard wrappers
- text-only status wrapper flow

This batch explicitly did not move:

- diff rendering body
- trace rendering body
- Context Receipt rendering body
- `get_latest_receipt` invoke path
- checkpoint restore / compare behavior

## Files Changed

- `src/interface/web/modules/inspector.js`
- `src/interface/web/views/inspector-view.js`
- `src/interface/web/controllers/inspector-controller.js`
- `tests/frontend/day18_inspector_smoke.js`
- `tests/frontend/day25_inspector_safety_smoke.js`
- `docs/frontend/DAY6-C-INSPECTOR-SHELL-EXTRACTION-V3X.md`

## Implementation Notes

### View

`src/interface/web/views/inspector-view.js` now exposes:

- `window.HajimiInspectorView.bindTabs(onTabSelected)`
- `window.HajimiInspectorView.bindClose(onClose)`
- `window.HajimiInspectorView.setVisible(visible)`
- `window.HajimiInspectorView.setActiveTab(tabId, onActivePanel)`

The view only handles Inspector shell DOM behavior:

- `.inspector-tab`
- `.inspector-panel`
- `rightInspector`
- `inspectorCloseBtn`

### Controller

`src/interface/web/controllers/inspector-controller.js` now exposes:

- `window.HajimiInspectorController.init(app)`
- `window.HajimiInspectorController.showInspectorTab(app, tabId)`
- `window.HajimiInspectorController.withInspectorGuard(app, label, renderFn)`
- `window.HajimiInspectorController.safeUpdateTaskDetails(app, statusText)`
- `window.HajimiInspectorController.safeRenderContextFiles(app)`
- `window.HajimiInspectorController.safeRenderModelInfo(app)`
- `window.HajimiInspectorController.safeRenderInspectorDiffPreview(app)`
- `window.HajimiInspectorController.safeRenderTraceInspector(app)`
- `window.HajimiInspectorController.openDiffPreview(app, file)`
- `window.HajimiInspectorController.updateTaskDetails(app, statusText)`
- `window.HajimiInspectorController.renderTaskSteps(app)`

### Compatibility

`src/interface/web/modules/inspector.js` remains the public compatibility API through `window.HajimiInspector`.

It prefers `window.HajimiInspectorController` when available. Because this batch did not modify `src/interface/web/index.html`, it also retains a minimal shell-only compatibility fallback so the existing browser script order cannot break the live app.

Rendering-heavy methods remain in `inspector.js`:

- `renderEditSummary(app)`
- `renderContextFiles(app)`
- `renderModelInfo(app)`
- `renderInspectorDiffPreview(app)`
- `renderDiffPreview(app)`
- `renderTraceInspector(app)`
- `loadLatestReceipt(app)`
- `renderContextReceiptPanel(app, receipt)`
- `setupReceiptPanel(app)`

## Preserved Contract

- Existing `window.HajimiInspector` API remains present.
- Existing app wrappers in `src/interface/web/app.js` were not modified.
- DOM IDs and CSS selectors were not changed.
- Diff / trace / receipt rendering behavior was not rewritten.
- `get_latest_receipt` invoke path was not moved.
- Checkpoint restore / compare behavior was not modified.
- No Provider, Shell, Agent streaming, `main.rs`, or `tauri.conf.json` files were modified.

## Automated Checks

```text
node --check src/interface/web/modules/inspector.js
Result: PASS

node --check src/interface/web/views/inspector-view.js
Result: PASS

node --check src/interface/web/controllers/inspector-controller.js
Result: PASS

node --check tests/frontend/day18_inspector_smoke.js
Result: PASS

node --check tests/frontend/day25_inspector_safety_smoke.js
Result: PASS

node tests/frontend/day18_inspector_smoke.js
Result: PASS
Output: day18 inspector module smoke: PASS

node tests/frontend/day25_inspector_safety_smoke.js
Result: PASS
Output: day25 inspector safety smoke: PASS (4 scenarios)

npm run test:security-gate
Result: KNOWN FAIL
Summary: findings 109; failures 3; warnings 106; allowlisted 106
Allowed legacy failures:
- src/interface/web/views/command-palette-view.js:36
- src/interface/web/views/session-list-view.js:22
- src/interface/web/views/session-list-view.js:26
New Inspector shell failure: NO
```

## Forbidden Boundary Receipt

Command:

```text
git diff --name-only -- src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs src/interface/web/controllers/provider-controller.js src/interface/web/views/provider-view.js src/interface/web/services/provider-service.js src/interface/web/modules/resource-dashboard.js src/interface/web/controllers/dashboard-controller.js src/interface/web/views/dashboard-view.js
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

- DECISION-001: Keep `window.HajimiInspector` as the stable public API for app.js compatibility.
- DECISION-002: Land `HajimiInspectorView` and `HajimiInspectorController` as narrow shell-only globals.
- DECISION-003: Keep rendering-heavy diff / trace / Context Receipt bodies in `inspector.js` for Day6-D.
- DECISION-004: Retain a minimal shell-only compatibility fallback inside `inspector.js` because this batch did not include browser script-order wiring in `index.html`.

## Debt

- DEBT-COMPAT-DAY6-C: `inspector.js` still has a minimal shell-only fallback until a future browser wiring pass explicitly loads `views/inspector-view.js` and `controllers/inspector-controller.js` before `modules/inspector.js`.
- DEBT-RENDERING-DAY6-D: diff / trace / Context Receipt rendering and `get_latest_receipt` remain in `inspector.js` and must be handled by Day6-D.
- DEBT-SECURITY-GATE-LEGACY: security-gate remains KNOWN FAIL for the three pre-existing unverified DOM HTML findings listed above.

## Next

Next batch:

- `Day6-D Inspector Rendering / Receipt Guarded Extraction`
