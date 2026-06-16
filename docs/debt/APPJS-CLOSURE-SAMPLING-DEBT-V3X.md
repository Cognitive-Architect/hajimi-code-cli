# app.js Closure Sampling Debt V3X

日期：2026-06-16

关联任务：`STONE-AUDIT-V3X-DAY07 app.js Closure Sampling`

关联报告：`docs/frontend/APPJS-CLOSURE-SAMPLING-V3X.md`

分支：`stone-audit-v3x-controlled-demolition`

HEAD before：`f800fcad6fc2d6346616c07c4fb7ebf8e8e8c8c4`

## Debt Summary

Day07 completed a docs-only closure sampling pass. It did not delete `app.js` code. The sampling found no new app.js fallback block with enough evidence for Day08 deletion.

人话版：今天把杂物间又盘了一遍，发现有些箱子确实已经搬完了，但剩下的大箱子要么没实机确认，要么里面可能有煤气罐，不能直接扔。

## Open Debt Items

| Debt | Status | Evidence | Required follow-up |
|---|---|---|---|
| Security gate known fail | OPEN | `npm run test:security-gate` still reports 3 failures in `command-palette-view.js` and `session-list-view.js`. | Separate safe-DOM rewrite/allowlist decision. |
| Inspector WebView blocked | OPEN | `INSPECTOR-WEBVIEW-WIRING-V3X.md` records WebView BLOCKED and fallback removal NOT DONE. | Dedicated Inspector WebView retry before fallback removal. |
| Day5-BE WebView deferred | OPEN | `DAY5-BE-HANDOFF-V3X.md` and closure docs record WebView visual validation deferred for Feedback/Markdown/Topbar/Chat/Model Picker. | Real WebView receipt before deleting app.js fallbacks in those domains. |
| Day08 no new deletion target | INTENTIONAL | `APPJS-CLOSURE-SAMPLING-V3X.md` recommends verification-only. | Do not force deletion without new evidence. |
| Provider/Shell/Checkpoint/Agent streaming kept | INTENTIONAL | Day06/Day07 classify these as high-risk boundaries. | Separate domain-specific tasks only. |

## Explicit Non-Claims

- This debt note does not claim `app.js` closure is complete.
- This debt note does not claim `app.js <= 1200` lines.
- This debt note does not claim WebView PASS for Day5-BE domains.
- This debt note does not clear security-gate failures.

## Stop Rule For Next Agent

If the next task tries to delete app.js fallbacks without matching Node + WebView receipt, stop and write a sampling/update report instead.
