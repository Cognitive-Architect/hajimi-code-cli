# APPJS Second-Pass Closure Debt - V4X

## 1. Summary

Day07 app.js second-pass closure did not remove wrappers or fallbacks. The task stayed in **LIMITED / NO-WEBVIEW** mode because Day06 real Tauri WebView validation remains `NOT RUN / DEFERRED`.

## 2. Current Status

| Item | Status |
|---|---|
| `app.js` current line count | `5490` |
| `app.js <=4000` | NOT MET |
| `app.js <=2500` | NOT MET |
| `app.js <=1200` | NOT MET |
| Removed wrappers | NONE |
| Production code changed | NO |
| Real WebView | `NOT RUN / DEFERRED` |
| Release validation | `NOT RUN / DEFERRED` |

## 3. Why No Wrapper / Fallback Was Removed

The Day02 sampling receipt lists possible wrapper/fallback candidates, but the current evidence is not enough to delete them safely:

- Slash catalog fallback requires script-load proof before deletion.
- Keyboard fallback needs dedicated shortcut coverage and WebView keyboard regression.
- Command Palette wrappers are already delegated but marked keep-stable.
- Chat view wrappers touch chat rendering and remain unsafe to change without broader chat/WebView evidence.
- Day06 app-state/bootstrap helper loading still lacks the real WebView receipt required by the deferred debt note.

Because the remaining candidates depend on browser or WebView behavior, deleting them in a Node-only environment would risk behavior loss.

## 4. Required Before Closure Can Remove More Code

Before deleting wrapper/fallback code that depends on runtime browser behavior, obtain one of these evidence sets:

1. Focused real Tauri WebView receipt proving app launch, no white screen, no crash, Command Palette opens, Settings opens, chat input is editable, and app-state/bootstrap globals load correctly.
2. Dedicated browser/WebView regression for the exact wrapper/fallback being removed.
3. A narrow Node smoke only if the target code path is fully Node-representable and does not depend on WebView script ordering, desktop integration, keyboard behavior, Provider/keyring, Shell, Checkpoint, or Agent streaming.

## 5. Forbidden Until Evidence Exists

- No fake WebView PASS.
- No release PASS claim.
- No deletion of fallback/wrapper code that requires WebView proof.
- No Provider save/delete/test/probe/validate/backup/keyring changes.
- No Checkpoint restore/export/replay/compare changes.
- No Shell execution, shell allowlist, CSP, or `withGlobalTauri` changes.
- No Agent streaming refactor.
- No repo history rewrite.

## 6. Next Candidate Work

Future app.js reduction should be split into small, separately verified tasks:

| Candidate | Current disposition |
|---|---|
| Slash catalog fallback | Candidate only after script-load proof. |
| Keyboard fallback | Candidate only after WebView keyboard regression. |
| Chat view render wrappers | Candidate only with chat render smoke plus WebView/manual coverage. |
| Token cumulative localStorage helpers | Candidate only after storage-key compatibility smoke. |
| Settings wrapper delegation | Candidate only after proving settings controller path in browser context. |

Until that evidence exists, Day07 closure remains docs-only and the app.js size targets remain open debt.
