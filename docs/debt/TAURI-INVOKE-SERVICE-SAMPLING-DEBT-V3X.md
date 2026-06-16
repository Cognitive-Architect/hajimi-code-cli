# Tauri Invoke Service Sampling Debt V3X

日期：2026-06-16

分支：`stone-audit-v3x-controlled-demolition`

关联任务：`STONE-AUDIT-V3X-DAY06 Tauri Invoke Service Sampling`

关联报告：`docs/frontend/TAURI-INVOKE-SERVICE-SAMPLING-V3X.md`

关联提交：`375141352c1c30741fc613bc20f3814471d4827e`

## Debt Summary

Day06 完成了 docs-only invoke 调用点采样，但没有实现 `services/tauri-service.js`，没有迁移任何 invoke 调用，也没有执行真实 Tauri WebView 点击验证。

人话版：这次只把“前台递给后厨的小票”盘点清楚，没有换收银台，也没有真的让后厨按每张小票做一遍菜。

## Open Debt Items

| Debt | Status | Evidence | Required follow-up |
|---|---|---|---|
| WebView smoke not run | OPEN | `TAURI-INVOKE-SERVICE-SAMPLING-V3X.md` records `WebView: NOT RUN`. | Separate WebView smoke after any tauri-service wiring. |
| `services/tauri-service.js` remains skeleton | OPEN | `src/interface/web/services/tauri-service.js` still only exports `tauriServiceSkeleton`. | Day07 service contract implementation + Node smoke. |
| UNKNOWN invoke classifications | OPEN | `optimize_context`, dynamic governance `cmd`, checkpoint dry-run, provider validation remain UNKNOWN / boundary-tagged. | Separate per-domain sampling before migration. |
| Security gate known baseline fail | OPEN | `npm run test:security-gate` reported 97 findings, 3 failures, 94 warnings, 94 allowlisted. | Separate safe-DOM debt task for command palette/session list view. |
| Provider / Keyring / Checkpoint / Shell / Agent streaming not migrated | INTENTIONAL | Day06 forbidden boundary. | Dedicated future tasks only. |

## Explicit Non-Claims

- This debt note does not claim WebView PASS.
- This debt note does not claim security-gate PASS.
- This debt note does not clear Provider / Keyring / Checkpoint / Shell / Agent streaming risk.
- This debt note does not prove `tauri-service.js` is ready for production wiring.

## Stop Rule For Next Agent

If the next task needs to move Provider / Keyring / Checkpoint / Shell / Agent streaming together with the first `tauri-service.js` contract slice, stop and split the task. The first implementation slice should only prove the service wrapper contract and readonly candidates.
