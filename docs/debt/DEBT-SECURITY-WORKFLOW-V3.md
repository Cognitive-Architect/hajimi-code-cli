# DEBT-SECURITY-WORKFLOW-V3

> Status: INTEGRATED FIX PLANNER / DRY-RUN ONLY  
> Updated: 2026-05-23  
> Work items: B-17/11 Security Fix Policy + Dry-run Patch Planner.  

---

## 1. 活跃技术约束与技术债务 (Active Technical Debt)

| 债务 ID (Debt ID) | 当前状态 | 说明与债务细节 |
|:---|:---:|:---|
| **DEBT-FIX-APPLY-B17-11** | `PENDING / DRY-RUN ONLY` | `SecurityFixPlanner` 仅作为一个纯虚拟/内存中的 dry-run planner。当前绝对禁止执行文件物理写入。任何代码修改必须走 dry-run 生成虚拟 patch 补丁，未来真正的应用通道（Apply Channel）必须无缝接入现有的 `EditApplier` 治理层。 |
| **HAJIMI_SECURITY_FIX_ENABLED** | `DEFAULT OFF` | `HAJIMI_SECURITY_FIX_ENABLED` 环境变量门禁默认未开启。即使人为强制设定为 `true`，为了绝对代码安全，系统在修复规划时也依旧强制遵循 dry-run 与 human review 拦截红线。 |
| **PENDING-WEBVIEW-SMOKE** | `PENDING` | 真机 WebView 安全面板与按钮交互测试仍处于挂起/债务状态。目前在本地已通过 DOM 沙箱仿真烟雾测试（`tests/frontend/day17_security_workflow_smoke.js`）完全验证。 |

---

## 2. 安全合规自检与断言证明

1. **零文件直接写操作**: `src/intelligence/agent-core/security_fix.rs` 中绝对没有调用任何 `fs::write`、`write_all`、`remove_file` 或 `rename` 等直接操作文件系统物理存储的 API，完全杜绝了静默代码篡改漏洞。
2. **高危漏洞强拦截**: 所有 Severity 为 High 与 Critical 的漏洞其生成的 `PatchPlan` 都硬性装载了 `human_review_required: true` 与 `dry_run: true`，从数据源头上确保安全策略合规性。
3. **两翼契约完备**: 每一个 `PatchPlan` 都生成了完整的 `validation_commands` 与 `rollback_plan`。
4. **编译与单元测试合规**: 
   - `cargo check -p intelligence-agent-core` 通过，0 编译错误。
   - `cargo test -p intelligence-agent-core security_fix` 3 个漏洞严重级与干跑策略单元测试 100% 通过。
   - `cargo test -p intelligence-agent-core security_workflow_fix_finding_runs_planner` 100% 通过。
