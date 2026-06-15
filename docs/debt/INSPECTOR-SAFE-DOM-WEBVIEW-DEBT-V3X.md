# Inspector Safe DOM WebView Debt V3X

## Status

- Status: OPEN
- Date: 2026-06-15
- Branch: `stone-audit-v3x-controlled-demolition`
- Related task: `STONE-AUDIT-V3X-DAY02 | Inspector Safe DOM Rewrite Implementation`
- Related implementation commit: `66e8e41950b9f774a9187cd1eb049eef36015844`

## Debt

Day02 completed Node-level Inspector safe DOM rewrite validation, but did not run a real Tauri WebView click smoke.

This debt does not mean the Day02 Node smoke failed. It means browser/runtime proof is still separate and must not be marked as PASS.

## Verified In Day02

- `node --check src/interface/web/modules/inspector.js`: PASS
- `node --check src/interface/web/views/inspector-view.js`: PASS
- `node --check src/interface/web/controllers/inspector-controller.js`: PASS
- `node tests/frontend/day18_inspector_smoke.js`: PASS
- `node tests/frontend/day25_inspector_safety_smoke.js`: PASS
- `node tests/frontend/day35_inspector_rendering_safe_dom_smoke.js`: PASS
- `npm run test:security-gate`: KNOWN FAIL only
  - Remaining failures:
    - `src/interface/web/views/command-palette-view.js:36`
    - `src/interface/web/views/session-list-view.js:22`
    - `src/interface/web/views/session-list-view.js:26`
  - Inspector failures: 0

## Not Verified

- Real Tauri WebView Inspector opening.
- Real tab switching after safe DOM rewrite.
- Real Diff / Trace / Context Receipt visual rendering in desktop runtime.
- Real clickable Trace checkpoint restore/compare buttons in WebView.

## Required Closure Evidence

To close this debt, a future task must record:

1. App launch result.
2. Inspector visible result.
3. Diff tab visible and safe rendering result.
4. Trace tab visible and safe rendering result.
5. Context Receipt visible result.
6. White screen / crash / no response result.
7. Console error observation.
8. Commit SHA tested.

## Human Note

人话版：这次像是在厨房台面上把碗、勺子、食材都验了一遍，确认不会把脏东西直接倒进碗里；但还没把这套流程搬到真实餐厅里开火试一遍。这个债务就是提醒下一刀要做真实窗口验收。
