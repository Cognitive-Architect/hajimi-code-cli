# STONE-AUDIT-V4X-DAY6｜App State / Bootstrap Initial Split

## 0. One-Line Result

Day6 completed a controlled frontend split for the lowest-risk app entry surface: default app state now lives in `src/interface/web/app/app-state.js`, bootstrap/init sequencing now lives in `src/interface/web/app/bootstrap.js`, and `app.js` delegates startup after loading those helpers. Node smoke and security gate PASS; real WebView was NOT RUN in this task and is recorded as debt.

人话版：这次不是拆聊天、模型、Provider 那些大管子，而是把“默认摆设清单”和“开店顺序表”从 `app.js` 大本子里抽出来。小灶台测试过了，但还没让用户实机点窗口，所以不吹 WebView PASS。

## 1. Scope

- Task: `STONE-AUDIT-V4X-DAY6：App State / Bootstrap Initial Split`
- Source work order: `F:\hajimi-code-cli\docs\roadmap\Hajimi ToneFix\task\task06.md`
- Source plan: `F:\hajimi-code-cli\docs\roadmap\Hajimi ToneFix\plan\STONE-AUDIT-V4X-POST-CLOSURE-FINISH_已更新_WebView真实验收_v0.2.md`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `7bfb9124bd1389c3b8b3cc00670d95c79cac8d15`
- Production code changed: YES, limited to allowed Day6 frontend files.
- WebView run in this task: NOT RUN.

## 2. Modified Files

| File | Change |
|---|---|
| `F:\hajimi-code-cli\src\interface\web\app.js` | Removed inline default state block; replaced inline `init()` body with delegation to `window.HajimiAppBootstrap.runAppBootstrap(this)`; added startup loading for `app/app-state.js` and `app/bootstrap.js`. |
| `F:\hajimi-code-cli\src\interface\web\app\app-state.js` | Replaced V3X skeleton with `createDefaultSettings()`, `createDefaultExtensions()`, `createDefaultAppState()`, and `applyDefaultAppState(app)`. |
| `F:\hajimi-code-cli\src\interface\web\app\bootstrap.js` | Replaced V3X skeleton with `runAppBootstrap(app)`, `buildCommandCatalog(app)`, and fallback command catalog. |
| `F:\hajimi-code-cli\tests\frontend\day33_app_bootstrap_smoke.js` | Added Node smoke for state injection, bootstrap init ordering, command catalog preference/fallback, and low-risk Chat / Command Palette / Settings setup entry calls. |
| `F:\hajimi-code-cli\docs\frontend\APPJS-STATE-BOOTSTRAP-SPLIT-V4X.md` | This receipt. |
| `F:\hajimi-code-cli\docs\debt\APPJS-STATE-BOOTSTRAP-WEBVIEW-DEBT-V4X.md` | Debt note for real WebView NOT RUN / dynamic helper loading not manually verified. |

## 3. app.js Line Count

| Item | Count |
|---|---:|
| app.js before | `5568` |
| app.js after | `5490` |
| Delta | `-78` |
| app-state.js after | `77` |
| bootstrap.js after | `88` |

Command:

```powershell
(Get-Content src/interface/web/app.js).Count
(Get-Content src/interface/web/app/app-state.js).Count
(Get-Content src/interface/web/app/bootstrap.js).Count
```

## 4. Moved Helpers

| Helper | New location | Notes |
|---|---|---|
| Default settings/state fields | `src/interface/web/app/app-state.js` | Preserves defaults for tabs, sidebar, panel, provider/session/chat/token/MCP/trace/extension state. |
| Default extension catalog | `src/interface/web/app/app-state.js` | Preserves existing extension entries and labels. |
| Bootstrap init sequence | `src/interface/web/app/bootstrap.js` | Preserves the leading init order checked by Node smoke. |
| Command Palette fallback catalog | `src/interface/web/app/bootstrap.js` | Preserves fallback IDs while still preferring `window.HajimiCommandPaletteCatalog.createCommandPaletteCatalog(app)` when present. |

## 5. Init Order / Behavior Notes

| Item | Result |
|---|---|
| `app.init()` still exists | YES |
| Init sequence moved | YES, into `window.HajimiAppBootstrap.runAppBootstrap(app)` |
| Init order intentionally changed | NO |
| Command catalog semantic change | NO |
| Chat streaming changed | NO |
| Provider / Keyring changed | NO |
| Shell / Checkpoint / Agent streaming changed | NO |
| CSP / withGlobalTauri changed | NO |
| DOM ID/class changed | NO |

## 6. Validation Results

| Command | Result | Summary |
|---|---|---|
| `node --check src/interface/web/app.js` | PASS | syntax valid |
| `Get-ChildItem src/interface/web/app -Filter *.js \| ForEach-Object { node --check $_.FullName }` | PASS | `app/*.js node --check: PASS` |
| `node --check tests/frontend/day33_app_bootstrap_smoke.js` | PASS | syntax valid |
| `node tests/frontend/day33_app_bootstrap_smoke.js` | PASS | `day33 app bootstrap smoke: PASS (state/bootstrap/init/catalog scenarios)` |
| `node tests/frontend/day28_command_palette_dom_smoke.js` | PASS | `day28 command palette DOM smoke: PASS (5 scenarios)` |
| `node tests/frontend/day29_session_list_dom_smoke.js` | PASS | `day29 sessionList DOM smoke: PASS (6 scenarios)` |
| `npm run test:security-gate` | PASS | `failures: 0`, `warnings: 97`, `allowlisted: 97` |
| `git diff --name-only -- src/interface/web/controllers src/interface/web/views src/interface/web/modules src/interface/web/styles src/interface/web/style.css src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json Cargo.toml Cargo.lock package.json` | PASS | no output |
| `git diff --cached --check` before staging | PASS | no output |

## 7. Smoke Coverage

| Area | Evidence | Result |
|---|---|---|
| App default state | `day33_app_bootstrap_smoke.js` checks injected tabs/settings/token/extensions defaults | PASS |
| Bootstrap init order | `day33_app_bootstrap_smoke.js` checks leading setup order and async workspace/loadFileTree path | PASS |
| Chat basic setup entry | `day33_app_bootstrap_smoke.js` checks `setupChat` is still called | PASS |
| Command Palette setup entry | `day33_app_bootstrap_smoke.js` plus day28 DOM smoke | PASS |
| Settings setup entry | `day33_app_bootstrap_smoke.js` checks `loadSettings` and `setupSettingsTabs` | PASS |
| Session list regression | day29 DOM smoke | PASS |
| Real Tauri WebView | not launched in this task | NOT RUN |

## 8. Forbidden Diff Receipt

Forbidden diff command produced no output:

```powershell
git diff --name-only -- src/interface/web/controllers src/interface/web/views src/interface/web/modules src/interface/web/styles src/interface/web/style.css src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs src/interface/desktop/tauri.conf.json Cargo.toml Cargo.lock package.json
```

Forbidden areas intentionally not touched:

- Provider / Keyring
- Shell execution
- Checkpoint restore/export/replay/compare
- Agent streaming
- CSP / withGlobalTauri
- Command Palette controller/view semantics
- Session List module/view semantics
- Rust backend / Tauri config

## 9. Git Status Summary Before Commit

Expected task changes:

- `M src/interface/web/app.js`
- `M src/interface/web/app/app-state.js`
- `M src/interface/web/app/bootstrap.js`
- `?? tests/frontend/day33_app_bootstrap_smoke.js`
- `?? docs/frontend/APPJS-STATE-BOOTSTRAP-SPLIT-V4X.md`
- `?? docs/debt/APPJS-STATE-BOOTSTRAP-WEBVIEW-DEBT-V4X.md`

Known old dirty files not staged:

- `?? .agents/`
- `?? docs/roadmap/Hajimi ToneFix/plan/STONE-AUDIT-V4X-POST-CLOSURE-FINISH_已更新_WebView真实验收_v0.2.md`

Old dirty files staged: NO.

## 10. Debt / UNKNOWN / NOT RUN

| Item | Status | Reason |
|---|---|---|
| Real WebView launch after dynamic app helper loading | NOT RUN | This task ran Node/static smoke only. |
| Dynamic loading of `app/app-state.js` and `app/bootstrap.js` in packaged release | UNKNOWN | Needs manual or automated Tauri WebView receipt. |
| app.js hard target `<=1200` | NOT MET | Current line count is `5490`; Day09 already retargeted hard split. |

Companion debt file:

- `F:\hajimi-code-cli\docs\debt\APPJS-STATE-BOOTSTRAP-WEBVIEW-DEBT-V4X.md`

## 11. Next Recommendation

Run the next day/task only after a focused WebView receipt confirms startup still loads `app/app-state.js` and `app/bootstrap.js` in the real desktop window.

人话版：下一步先开真实窗口看一眼“新抽屉能不能真的被柜子读到”，再继续拆别的。别一边没确认开店流程，一边又去搬厨房。
