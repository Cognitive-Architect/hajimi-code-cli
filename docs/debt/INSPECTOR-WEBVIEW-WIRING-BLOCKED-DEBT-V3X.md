# Inspector WebView Wiring Blocked Debt V3X

## Status

- Status: OPEN
- Date: 2026-06-15
- Branch: `stone-audit-v3x-controlled-demolition`
- Related task: `STONE-AUDIT-V3X-DAY03 | Inspector Browser Wiring + Compatibility Fallback Removal`
- Related baseline HEAD: `fa8ba44e37d6f1cf3a794d695f8114d62b6e83c3`

## Debt

Day03 wired `views/inspector-view.js` and `controllers/inspector-controller.js` into `src/interface/web/index.html`, but true Inspector WebView smoke was blocked.

Because the WebView proof was blocked, `src/interface/web/modules/inspector.js` compatibility fallback was intentionally kept.

## Verified

- `index.html` loads:
  - `views/inspector-view.js`
  - `controllers/inspector-controller.js`
  - before `modules/inspector.js`
  - before `app.js`
- Inspector Node smoke passed:
  - `day18`
  - `day25`
  - `day35`
- `npm run test:security-gate` did not add Inspector failures.
- Tauri dev compiled and started `hajimi-desktop.exe`.
- Desktop process was responding.

## Blocker

The app launched into a native window titled:

```text
Hajimi 工具执行确认
```

WebView2 remote debugging was attempted through:

```text
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
```

But port `9222` was not listening, so DevTools could not read:

- `window.HajimiInspectorView`
- `window.HajimiInspectorController`
- Inspector tab DOM
- Diff / Trace / Context Receipt DOM

## Not Verified

- WebView `window.HajimiInspectorView exists`
- WebView `window.HajimiInspectorController exists`
- Inspector visible/clickable in the desktop WebView
- Diff tab rendering in the desktop WebView
- Trace tab rendering in the desktop WebView
- Context Receipt rendering in the desktop WebView
- Whether `modules/inspector.js` fallback is unused in the real WebView path

## Required Closure Evidence

To close this debt, a later task must provide:

1. App launch with main UI visible, not blocked by confirmation dialog.
2. WebView console or DevTools proof for:
   - `window.HajimiInspectorView`
   - `window.HajimiInspectorController`
   - `window.HajimiInspector`
3. Inspector open / tab switch observation.
4. Diff / Trace / Context Receipt visible observation.
5. No white screen / crash / no response.
6. Explicit fallback decision after proof.

## Human Note

人话版：新线已经接进配电箱了，台面测试也都过了；但真实车上试跑时，车门口弹了个确认门卫，而且我们没拿到车内监控画面。所以今天不剪旧备用线，先把卡点贴条记录，等下一刀专门处理。
