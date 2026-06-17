# APPJS Second-Pass Sampling Debt｜V4X

## Scope

This debt note records unresolved or unverified items found during `STONE-AUDIT-V4X-DAY2：app.js Second-Pass Sampling`.

- Branch sampled: `stone-audit-v3x-controlled-demolition`
- HEAD sampled: `c818ba28cb866013094ad3a724f1372961deb9ed`
- Production code changed: NO
- WebView run: NOT RUN
- Primary report: `F:\hajimi-code-cli\docs\frontend\APPJS-SECOND-PASS-SAMPLING-V4X.md`

## Debt Items

| Debt ID | Item | Status | Why it remains debt | Required next evidence |
|---|---|---|---|---|
| APPJS-V4X-DAY2-DEBT-001 | `chat-controller.js` browser/release loading | UNKNOWN | Controller exists, but Day2 did not run browser/release script-order validation. | Day3 Node + WebView receipt proving controller is loaded before `app.js` setup fallback is considered removable. |
| APPJS-V4X-DAY2-DEBT-002 | `app/app-state.js` and `app/bootstrap.js` activation | UNKNOWN | Files are skeleton-only and not proven production-wired. | Day6 init-order sampling and WebView launch receipt. |
| APPJS-V4X-DAY2-DEBT-003 | `setupKeyboardShortcuts()` fallback removal | KEEP-FOR-NOW | V3X intentionally kept the keyboard fallback; removing it could break global shortcuts. | Dedicated keyboard shortcut smoke and real WebView regression. |
| APPJS-V4X-DAY2-DEBT-004 | Provider readonly replacement of `renderProviderList` | UNKNOWN | Read-only modules and day36 smoke exist, but Day2 did not perform WebView readonly verification. | Day5 Provider readonly WebView receipt with forbidden provider command calls = 0. |
| APPJS-V4X-DAY2-DEBT-005 | `app.js <=1200` hard target | NOT CLAIMED | Remaining paths include Chat streaming, Provider/keyring, Checkpoint, Shell/tool execution, Agent streaming, governance, and MCP. | Day7 second-pass closure after staged cuts and WebView receipts. |

## Forbidden Until Separate Approval

- Provider save/delete/test/probe/validate/backup/keyring
- Checkpoint restore/export/replay/compare
- Shell execution or shell allowlist policy
- Agent streaming and agent event lifecycle
- CSP / withGlobalTauri / Tauri bridge semantics
- Repo history rewrite

## Next Action

Execute `F:\hajimi-code-cli\docs\roadmap\Hajimi ToneFix\task\task03.md` only after reading the Day2 sampling report. Day3 should stay focused on Chat basic path wiring/receipt and must not move `streamChat`, Provider write/keyring, Agent streaming, Shell, or Checkpoint logic.
