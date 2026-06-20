# APPJS-STATE-BOOTSTRAP-WEBVIEW-DEBT-V4X

## 0. Summary

Day6 moved default app state and bootstrap sequencing out of `src/interface/web/app.js`, but this task only ran Node/static validation. Real Tauri WebView startup after dynamic loading of `app/app-state.js` and `app/bootstrap.js` remains NOT RUN.

人话版：纸面测试和小灶台测试过了，但还没打开真实桌面窗口确认这两个新文件一定能被应用读到。

## 1. Scope

- Related task: `STONE-AUDIT-V4X-DAY6：App State / Bootstrap Initial Split`
- Related receipt: `F:\hajimi-code-cli\docs\frontend\APPJS-STATE-BOOTSTRAP-SPLIT-V4X.md`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before task: `7bfb9124bd1389c3b8b3cc00670d95c79cac8d15`
- Production code changed in related task: YES, limited to `app.js` and `src/interface/web/app/*.js`.

## 2. Debt

| Item | Status | Required evidence |
|---|---|---|
| Release/dev WebView loads `src/interface/web/app/app-state.js` | UNKNOWN | Real Tauri WebView launch or console/network receipt showing `window.HajimiAppState` exists before app init. |
| Release/dev WebView loads `src/interface/web/app/bootstrap.js` | UNKNOWN | Real Tauri WebView launch or console/network receipt showing `window.HajimiAppBootstrap` exists before app init. |
| App launch after Day6 split | NOT RUN in this task | Manual/automated WebView smoke with app launch PASS, white screen NO, crash NO. |
| Command/Settings/Chat basic in real WebView after Day6 split | NOT RUN in this task | Focused WebView smoke; Node smoke alone is not enough. |

## 3. Validation Already Done

| Command | Result |
|---|---|
| `node --check src/interface/web/app.js` | PASS |
| `Get-ChildItem src/interface/web/app -Filter *.js \| ForEach-Object { node --check $_.FullName }` | PASS |
| `node tests/frontend/day33_app_bootstrap_smoke.js` | PASS |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS |
| `node tests/frontend/day29_session_list_dom_smoke.js` | PASS |
| `npm run test:security-gate` | PASS, failures `0`, warnings `97`, allowlisted `97` |

## 4. Next Required Evidence

Run a small real WebView receipt before expanding the split:

1. Launch desktop app.
2. Confirm app launch PASS, white screen NO, crash NO.
3. Confirm Command Palette opens.
4. Confirm Settings opens.
5. Confirm chat input is editable.
6. If console/devtools is available, confirm:
   - `window.HajimiAppState` exists.
   - `window.HajimiAppBootstrap` exists.
   - `window.app.commands` is populated after init.

## 5. Stop Rule

If real WebView shows white screen, crash, `HajimiAppState` missing, or `HajimiAppBootstrap` missing, stop and fix only the Day6 helper loading path. Do not continue app.js high-risk demolition until this receipt is closed.
