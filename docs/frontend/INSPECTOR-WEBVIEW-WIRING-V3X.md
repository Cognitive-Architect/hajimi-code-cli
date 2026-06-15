# STONE-AUDIT-V3X-DAY03 Inspector WebView Wiring Receipt

## Scope

- Task: `STONE-AUDIT-V3X-DAY03 | Inspector Browser Wiring + Compatibility Fallback Removal`
- Date: 2026-06-15
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `fa8ba44e37d6f1cf3a794d695f8114d62b6e83c3`
- Modified production file: `src/interface/web/index.html`
- Fallback removal: NO

## Summary

`src/interface/web/index.html` now loads the Inspector shell view and controller before `modules/inspector.js` and before `app.js`.

Human note: Day02 fixed the Inspector parts; Day03 connected those parts into the browser script line-up. The old backup wire inside `modules/inspector.js` was not removed because the real WebView proof was blocked.

## Script Order

Observed script order:

```text
src/interface/web/index.html:643 views/inspector-view.js
src/interface/web/index.html:644 controllers/inspector-controller.js
src/interface/web/index.html:645 modules/inspector.js
src/interface/web/index.html:658 app.js
```

The `index.html` diff only adds these two Inspector script tags:

```text
<script defer src="views/inspector-view.js"></script>
<script defer src="controllers/inspector-controller.js"></script>
```

## Node Validation

| Command | Result |
|---|---|
| `node --check src/interface/web/modules/inspector.js` | PASS |
| `node --check src/interface/web/views/inspector-view.js` | PASS |
| `node --check src/interface/web/controllers/inspector-controller.js` | PASS |
| `node tests/frontend/day18_inspector_smoke.js` | PASS |
| `node tests/frontend/day25_inspector_safety_smoke.js` | PASS |
| `node tests/frontend/day35_inspector_rendering_safe_dom_smoke.js` | PASS |

## Security Gate

Command:

```powershell
npm run test:security-gate
```

Result:

- Overall status: KNOWN FAIL
- Findings: `97`
- Failures: `3`
- Warnings: `94`
- Allowlisted: `94`
- Inspector failures: `0`

Known failures remaining out of Day03 scope:

1. `src/interface/web/views/command-palette-view.js:36`
2. `src/interface/web/views/session-list-view.js:22`
3. `src/interface/web/views/session-list-view.js:26`

No security allowlist was changed.

## WebView Smoke Attempt

Attempted launch path:

1. Started static frontend server:
   - command: `npx serve . -p 3456`
   - working directory: `F:\hajimi-code-cli\src\interface\web`
   - result: `http://localhost:3456` returned `200`
2. Started Tauri dev with WebView2 remote debugging attempt:
   - command: `cargo tauri dev`
   - working directory: `F:\hajimi-code-cli\src\interface\desktop`
   - env attempt: `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`
   - result: cargo build completed and launched `F:\hajimi-code-cli\target\debug\hajimi-desktop.exe`

Observed process/window evidence:

| Check | Result |
|---|---|
| Tauri build | PASS |
| `hajimi-desktop.exe` process | PASS |
| process responding | PASS |
| MainWindowTitle | `Hajimi 工具执行确认` |
| WebView2 processes | OBSERVED |
| WebView2 DevTools port `9222` | BLOCKED: not listening |
| `window.HajimiInspectorView` | BLOCKED: no DevTools target |
| `window.HajimiInspectorController` | BLOCKED: no DevTools target |
| Inspector visible/clickable | BLOCKED: confirmation window / no DOM access |
| white screen | NOT OBSERVED |
| crash | NO |
| no response | NO |
| command-specific error | NOT OBSERVED |

The WebView smoke is therefore **BLOCKED**, not PASS.

## Fallback Decision

`src/interface/web/modules/inspector.js` still contains:

- `compatInspectorView`
- `compatInspectorController`
- `getInspectorController()`

Decision:

- fallback removed: NO
- reason: true WebView Inspector globals and UI smoke were blocked
- fallback used in WebView: UNKNOWN

No fallback was removed in Day03 because the task requires real WebView proof before cutting the backup path.

## Forbidden Diff

Command:

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/modules/resource-dashboard.js src/interface/desktop src/engine/tool-system
```

Result: PASS, no output.

High-risk areas not touched:

- `src/interface/web/app.js`
- Provider / Keyring
- Shell execution
- Checkpoint restore/export/compare/replay
- Agent streaming
- CSP
- withGlobalTauri
- Desktop/Rust backend
- Dashboard / Provider migration

## Process Cleanup

Started test processes were stopped after the WebView attempt:

- static frontend server on `3456`: stopped
- Tauri dev / `hajimi-desktop.exe`: stopped
- WebView2 child processes from this attempt: stopped

## Result

Day03 result: PARTIAL / BLOCKED

- Browser script wiring: PASS
- Node Inspector smoke: PASS
- Security-gate no-new-failure: PASS against known-fail baseline
- WebView Inspector smoke: BLOCKED
- Compatibility fallback removal: NOT DONE

Next recommended step: run a dedicated Day03-B WebView smoke after resolving the startup confirmation / WebView2 DevTools access issue, then remove only the proven Inspector compatibility fallback.
