# Debt: STONE-AUDIT-V3X-DAY08 app.js fallback removal blocked

## Summary

- Date: 2026-06-16
- Branch: `stone-audit-v3x-controlled-demolition`
- Baseline HEAD: `fde3a5e110bea802016938f772478de62143c714`
- Task: `STONE-AUDIT-V3X-DAY08`
- Status: `BLOCKED / NO-OP`
- Production code modified: `NO`

## Why This Is Debt

The Day08 work order requested proven app.js fallback removal and expected
`app.js` line count to decrease. The required input document,
`docs/frontend/APPJS-CLOSURE-SAMPLING-V3X.md`, says there is no new `app.js`
fallback block with enough Node + WebView evidence to delete safely in Day08.

The only `SAFE-TO-REMOVE` items listed by Day07 were already completed before
this task:

1. Command Palette six pure fallback bodies.
2. Resource Dashboard module compatibility fallback.
3. Session List View compatibility fallback.

## Current Evidence

| Evidence | Result |
|---|---|
| app.js line count before | `5568` |
| app.js line count after | `5568` |
| Production diff | None |
| Node smoke | PASS for affected historical seams |
| Security gate | FAIL on known baseline: 3 existing high findings |
| WebView | Not run for Day08 because no affected production UI changed |

## Known Security Gate Baseline

The current `npm run test:security-gate` result is not clear:

- Findings: `97`
- Failures: `3`
- Warnings: `94`

Known failures:

1. `src/interface/web/views/command-palette-view.js:36`
2. `src/interface/web/views/session-list-view.js:22`
3. `src/interface/web/views/session-list-view.js:26`

This Day08 task does not fix or clear those findings.

## Stop Rule Applied

No fallback was removed because removing anything beyond Day07 completed items
would violate the Day08 rule: "不删 Day07 未列入 SAFE 的内容".

## Recommended Next Step

Before deleting any remaining fallback from `app.js`, create a separate receipt
with real WebView evidence for the specific domain, then run the relevant Node
smoke and security gate comparison.

Candidate follow-up areas only after separate proof:

- Feedback / Error toast fallback
- Markdown service fallback
- Topbar fallback
- Chat view/controller fallback
- Model Picker fallback
- Settings / Audit wrapper proof
