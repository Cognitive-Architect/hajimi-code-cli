# V3X-POST-01｜True Tauri WebView Full Regression Receipt

## 0. Scope

- Task: `V3X-POST-01 True Tauri WebView Full Regression Receipt`
- Date: `2026-06-17`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `cbee2b238f3e38a213254aa5f405037de2e9369d`
- Mode: receipt-only / no production fix
- Production changes: `NO`
- Git history changed before commit: `NO`

Human summary: this receipt opened the real desktop app and checked what could be safely observed. It does not repair failed or blocked paths.

## 1. Required Command Receipt

| Command | Result | Notes |
|---|---|---|
| `git branch --show-current` | `PASS` | `stone-audit-v3x-controlled-demolition` |
| `git rev-parse HEAD` | `PASS` | `cbee2b238f3e38a213254aa5f405037de2e9369d` |
| `git status --short` | `PASS` | clean before doc creation |
| `npm run test:security-gate` | `PASS` | 97 findings, 0 failures, 97 warnings, 97 allowlisted |
| `cargo check --workspace` | `PASS` | 29 warnings observed |
| `git diff --name-only -- src Cargo.toml Cargo.lock package.json src/interface/desktop/tauri.conf.json` | `PASS` | no output |
| `git diff --cached --check` | `PASS` | no output before doc stage |

## 2. WebView Environment

| Field | Value |
|---|---|
| OS context | Windows desktop |
| Tauri CLI | `tauri-cli 2.10.1` |
| App start mode | dev mode |
| Frontend server command | `python -m http.server 3456 --directory src/interface/web` |
| Tauri command | `cargo tauri dev` from `src/interface/desktop` |
| Tauri devUrl | `http://localhost:3456` |
| App process observed | `hajimi-desktop.exe` |
| App window title observed | `Hajimi` |
| WebView engine process observed | `msedgewebview2.exe` |
| Process cleanup after smoke | `DONE` |

Startup logs:

```text
cargo tauri dev: Finished dev profile and ran F:\hajimi-code-cli\target\debug\hajimi-desktop.exe
frontend server: GET /, /app.js, modules/*.js, views/*.js, controllers/*.js, styles/*.css returned HTTP 200
```

Screenshots/logs were stored in `%TEMP%` for local observation and were not committed as binary artifacts:

```text
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-screen-hajimi.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-screen-max.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-command-palette.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-settings-cn-filter.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-settings-keyboard-open.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-model-picker.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-font-restored.png
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-tauri-20260617-123811.log
C:\Users\22129\AppData\Local\Temp\v3x-post-webview-server-20260617-123811.log
```

## 3. Checked Paths Table

| Path | Result | Evidence | Notes |
|---|---|---|---|
| App launch | `PASS` | `hajimi-desktop.exe` window titled `Hajimi`; main UI visible | real Tauri WebView launched |
| White screen | `NO` | screenshot showed populated UI | no blank WebView |
| Crash | `NO` | process stayed alive during smoke | stopped manually after receipt |
| Console critical error | `UNKNOWN / NOT OBSERVED IN PROCESS LOGS` | no DevTools console attached; Tauri/server logs showed startup and HTTP 200 loads | cannot claim console-clean |
| Slash Palette | `BLOCKED / INCONCLUSIVE` | typed `/` attempt did not show candidate list | selected `/compact` session did not expose chat input in visible area |
| Command Palette | `PASS` | `Ctrl+Shift+P` opened palette; candidates visible; Chinese filter `设置` worked; settings command opened panel | does not prove every command action |
| Session List | `PARTIAL` | session list visible and existing session selected | new session click was inconclusive due coordinate/tool instability |
| Settings | `PASS` | settings panel opened; general tab, theme, font size, word wrap visible | accidental font-size focus/change restored to `14` |
| Chat basic path | `BLOCKED` | chat input not visible/reachable in selected session during this pass | no provider/model success claimed |
| Model Picker | `PASS` | topbar model picker opened; `deepseek / deepseek-v4-pro` option visible | safe open/cancel observation only |
| Inspector | `BLOCKED` | not safely opened in this pass | no DevTools/global observation; no fallback deletion |
| Dashboard | `BLOCKED` | not safely opened in this pass | no dashboard PASS claim |
| Provider read-only tab | `BLOCKED / INCONCLUSIVE` | settings tabs visible; provider/model tab attempt hit general font control due coordinate scaling | forbidden provider command calls remained `0` |

## 4. PASS / FAIL / BLOCKED Summary

| Area | Status |
|---|---|
| App launch | `PASS` |
| Main UI visible | `PASS` |
| White screen | `NO` |
| Crash | `NO` |
| Console critical error | `UNKNOWN / NOT OBSERVED IN PROCESS LOGS` |
| Slash Palette | `BLOCKED / INCONCLUSIVE` |
| Command Palette | `PASS` |
| Session List | `PARTIAL` |
| Settings | `PASS` |
| Chat basic path | `BLOCKED` |
| Model Picker | `PASS` |
| Inspector | `BLOCKED` |
| Dashboard | `BLOCKED` |
| Provider readonly | `BLOCKED / INCONCLUSIVE` |
| WebView overall | `PARTIAL / NOT FULL PASS` |

## 5. Forbidden Actions Not Touched

| Forbidden action | Touched |
|---|---|
| Provider save | `NO` |
| Provider delete | `NO` |
| Provider probe / validate | `NO` |
| Provider backup/import/export | `NO` |
| Keyring write/delete | `NO` |
| Shell execution | `NO` |
| Checkpoint restore/export/compare/replay | `NO` |
| Agent streaming full path | `NO` |
| Git history rewrite | `NO` |
| Production source edit | `NO` |

Provider forbidden command calls observed/executed: `0`.

## 6. Blocker Classification

| Blocker | Classification | Impact |
|---|---|---|
| No attached WebView console/devtools | observation limit | console critical error cannot be certified as `NO` |
| Chat input not visible/reachable in selected session | UI state / testability limit | Slash Palette and chat basic path remain blocked |
| Coordinate scaling / click offset | desktop automation limit | Provider tab, Inspector, Dashboard not safely clicked |
| High-risk Provider actions nearby | safety boundary | stopped before save/delete/probe risk |

## 7. Interpretation

This receipt improves Day14's `WebView BLOCKED / NOT RUN` state to:

```text
WebView overall: PARTIAL / NOT FULL PASS
```

What is now proven:

- Real Tauri WebView launches.
- The main UI renders and is not white-screened.
- Static frontend assets and split CSS/modules load through `localhost:3456`.
- Command Palette can open and execute a safe settings navigation.
- Settings general panel renders.
- Model Picker opens and shows the current model.
- Session list is visible.

What is not proven:

- Full Slash Palette interaction in a visible chat input.
- Chat submit path.
- Inspector full panel behavior.
- Dashboard full panel behavior.
- Provider read-only list/tab behavior.
- WebView console is free of critical errors.

## 8. Next Recommendation

Open a second, narrower WebView task with either manual human clicking or a reliable desktop automation tool configured for DPI-aware coordinates.

Recommended target:

```text
V3X-POST-02｜WebView blocked path follow-up: Slash / Chat / Inspector / Dashboard / Provider readonly
```

Do not combine that task with production fixes. If a failure is found, record it first and open a separate fix task.

## 9. Final Receipt

| Field | Value |
|---|---|
| Document path | `docs/frontend/V3X-POST-WEBVIEW-FULL-REGRESSION.md` |
| Commit planned | `docs(frontend): add v3x post webview full regression receipt` |
| HEAD before | `cbee2b238f3e38a213254aa5f405037de2e9369d` |
| HEAD after | to be filled after commit |
| Production changes | `NO` |
| Old dirty files staged | `NO` |
| WebView overall | `PARTIAL / NOT FULL PASS` |
