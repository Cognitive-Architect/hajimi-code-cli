# Inspector Safe DOM Rewrite Debt V3X

Date: 2026-06-13

Branch: `stone-audit-v3x-controlled-demolition`

Baseline HEAD: `788c305481cb93efbdbf4ee7772210171e76607e`

Related task: `STONE-AUDIT-V3X-DAY01 Inspector Safe DOM Rewrite Sampling + Test Plan`

Related plan: `docs/frontend/INSPECTOR-SAFE-DOM-REWRITE-PLAN-V3X.md`

## Debt Summary

Inspector rendering still contains legacy `innerHTML` rendering in:

```text
src/interface/web/modules/inspector.js
```

The current security gate treats these Inspector module locations as accepted legacy warnings. However, Day6-D proved that moving the same rendering bodies into:

```text
src/interface/web/views/inspector-view.js
```

without a safe DOM rewrite creates new security-gate failures.

人话版：老房间里这批旧插座目前登记为“历史遗留风险”；但不能把旧插座原样搬到新房间。搬之前必须换成安全插座。

## Current Evidence

Validation run during Day01:

```text
node --check src/interface/web/modules/inspector.js
Result: PASS

node tests/frontend/day18_inspector_smoke.js
Result: PASS

node tests/frontend/day25_inspector_safety_smoke.js
Result: PASS

npm run test:security-gate
Result: KNOWN FAIL
Summary: findings 109; failures 3; warnings 106; allowlisted 106
Known failures:
- src/interface/web/views/command-palette-view.js:36
- src/interface/web/views/session-list-view.js:22
- src/interface/web/views/session-list-view.js:26
```

Inspector `innerHTML` lines sampled:

```text
src/interface/web/modules/inspector.js:148
src/interface/web/modules/inspector.js:153
src/interface/web/modules/inspector.js:165
src/interface/web/modules/inspector.js:167
src/interface/web/modules/inspector.js:179
src/interface/web/modules/inspector.js:184
src/interface/web/modules/inspector.js:200
src/interface/web/modules/inspector.js:242
src/interface/web/modules/inspector.js:254
src/interface/web/modules/inspector.js:261
src/interface/web/modules/inspector.js:339
src/interface/web/modules/inspector.js:367
```

## Not Completed

The following are intentionally not completed in Day01:

- No safe DOM rewrite implementation.
- No production code modification.
- No browser script wiring.
- No compatibility fallback removal.
- No real Tauri WebView proof.
- No security allowlist update.

## Required Clearance

This debt can only be cleared when all are true:

1. Diff preview, Trace inspector, and Context Receipt rendering use safe DOM construction instead of moved legacy `innerHTML` templates.
2. `node tests/frontend/day18_inspector_smoke.js` passes.
3. `node tests/frontend/day25_inspector_safety_smoke.js` passes.
4. A new or updated safe DOM smoke proves malicious strings render as text.
5. `npm run test:security-gate` has no new Inspector failures.
6. If WebView is claimed, a real Tauri WebView receipt exists.

Until then, status remains:

```text
STATUS: OPEN
```
