# APPJS State / Bootstrap Real WebView Deferred Debt - V4X

## 1. Baseline / 基线

| Item | Value |
|---|---|
| Branch at recording time | `work` |
| HEAD at recording time | `93074676d5a5536efd8198fe3699e4d73460b56e` |
| Related commit | `93074676d5a5536efd8198fe3699e4d73460b56e` |
| Related issue | GitHub Issue #11: V4X-DAY06-FOLLOWUP App State / Bootstrap Real WebView Receipt Before Day7 |
| Follow-up issue | GitHub Issue #13: V4X-DAY06-DEFER |
| Existing debt note | `docs/debt/APPJS-STATE-BOOTSTRAP-WEBVIEW-DEBT-V4X.md` |
| Day06 split receipt | `docs/frontend/APPJS-STATE-BOOTSTRAP-SPLIT-V4X.md` |
| Related runtime file | `src/interface/web/app/app-state.js` |
| Related runtime file | `src/interface/web/app/bootstrap.js` |

This record does not replace the required real WebView receipt. It only records that the Day06 receipt is deferred because a usable desktop/WebView environment is not available in this execution context.

## 2. Debt status / 债务状态

| Item | Status |
|---|---|
| Real Tauri WebView receipt | `NOT RUN / DEFERRED` |
| App launch in real desktop window | `NOT RUN / DEFERRED` |
| White screen check | `NOT RUN / DEFERRED` |
| Crash check | `NOT RUN / DEFERRED` |
| `window.HajimiAppState` in real WebView | `UNKNOWN` |
| `window.HajimiAppBootstrap` in real WebView | `UNKNOWN` |
| `window.app.commands` in real WebView after init | `UNKNOWN` |

Reason: the current user/environment does not provide a usable desktop session for real Tauri release/dev WebView manual validation.

Important boundaries:

- This is **not** a WebView PASS.
- This is **not** full release validation.
- This does **not** prove that packaged/release dynamic loading of `app/app-state.js` and `app/bootstrap.js` works.
- The earlier Node/static validation remains useful but is not equivalent to a real Tauri WebView receipt.

## 3. Day07 allowance / Day07 放行边界

Day07 may proceed only in **LIMITED / NO-WEBVIEW** mode while this debt remains open.

Allowed Day07 work:

- Line-count measurement and docs-only reporting.
- Static checks that do not require a real desktop window.
- Node smoke checks that do not claim WebView coverage.
- Documentation closure that preserves the `NOT RUN / DEFERRED` status.

Required wording for any Day07 report until the real receipt is completed:

- WebView status must remain `NOT RUN / DEFERRED`.
- Day06 real WebView receipt must remain open debt.
- Any Day07 outcome must avoid wording such as `WebView PASS`, `release PASS`, or `desktop verified` unless a real Tauri WebView was actually launched and checked.

## 4. Forbidden carry-over / 禁止继承风险

The following actions remain forbidden while this debt is open:

- No fake WebView PASS.
- No Chat streaming changes.
- No Provider save/delete/test/keyring changes.
- No Shell changes.
- No Checkpoint restore/export/replay/compare changes.
- No Agent streaming changes.
- No CSP changes.
- No `withGlobalTauri` changes.
- No repo history rewrite.
- No removal of fallback/wrapper code that depends on real WebView evidence for safe deletion.

## 5. Future required validation / 后续必须补验

A future focused real WebView receipt must record these checks before this debt can be closed:

| Check | Required result before closure |
|---|---|
| Launch Tauri desktop app | PASS |
| White screen | NO |
| Crash | NO |
| Command Palette opens | PASS |
| Settings opens | PASS |
| Chat input editable | PASS |
| `window.HajimiAppState` exists | PASS or explicitly justified if console unavailable |
| `window.HajimiAppBootstrap` exists | PASS or explicitly justified if console unavailable |
| `window.app.commands` populated after init | PASS or explicitly justified if console unavailable |

If any launch, white-screen, crash, missing-helper, or missing-command-catalog issue appears, stop and fix only the Day06 helper loading path before continuing broader app.js demolition.
