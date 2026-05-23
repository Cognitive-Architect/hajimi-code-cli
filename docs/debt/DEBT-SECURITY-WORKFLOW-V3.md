# DEBT-SECURITY-WORKFLOW-V3

> **Status**: INTEGRATED FIX PLANNER + REVALIDATION / DRY-RUN ONLY
> **Updated**: 2026-05-23
> **Work items**: B-17/11 Security Fix Policy + B-17/12 Revalidation Receipt & Integration.

---

## 1. 活跃技术约束与技术债务 (Active Technical Debt)

| 债务 ID (Debt ID) | 当前状态 | 说明与债务细节 |
|:---|:---:|:---|
| **DEBT-FIX-APPLY-B17-11** | `PENDING / DRY-RUN ONLY` | `SecurityFixPlanner` 仅作为一个纯虚拟/内存中的 dry-run planner。当前禁止执行文件物理写入。任何代码修改必须走 dry-run 生成虚拟 patch 补丁，未来真正的应用通道（Apply Channel）必须无缝接入现有的 `EditApplier` 治理层。 |
| **DEBT-REVALIDATION-B17-12** | `PENDING / DRY-RUN ONLY` | 验证命令的本地执行仅执行安全网禁检查（allow-list）并输出 dry-run receipt 契约，本日不执行真实物理 Shell 命令运行。无 passing validation receipt 的漏洞发现不允许被标记为 `Fixed` / `revalidated`。 |
| **HAJIMI_SECURITY_FIX_ENABLED** | `DEFAULT OFF` | `HAJIMI_SECURITY_FIX_ENABLED` 环境变量门禁默认未开启。即使人为强制设定为 `true`，为了代码安全，系统在修复规划时也依旧遵循 dry-run 与 human review 拦截红线。 |
| **PENDING-WEBVIEW-SMOKE** | `PENDING` | 真机 WebView 安全面板与按钮交互测试仍处于挂起/债务状态。目前在本地已通过 DOM 沙箱仿真烟雾测试（`tests/frontend/day17_security_workflow_smoke.js`）进行验证。 |

---

## 2. 安全合规自检与断言证明

1. **零文件直接写操作**: `src/intelligence/agent-core/security_fix.rs` 中未发现调用 `fs::write`、`write_all`、`remove_file` 或 `rename` 等直接操作文件系统物理存储的 API，以避免非预期文件修改。
2. **高危漏洞强拦截**: 所有 Severity 为 High 与 Critical 的漏洞其生成的 `PatchPlan` 都装载了 `human_review_required: true` 与 `dry_run: true`，从数据源头上确保安全策略合规性。
3. **策略契约生成**: 每一个 `PatchPlan` 都生成了对应的 `validation_commands` 与 `rollback_plan`，以及对应的 `revalidation_receipt`。
4. **限制危险指令执行**: 本地复测指令被限制在安全白名单内（仅允许本地的 `npm run test:security-gate`, `cargo test`, `git diff` 等本地静态/环境自测命令）。任何带有 `curl`、`wget`、`rm`、`eval`、`bash -c`、`powershell`、`cmd` 等敏感参数特征的指令在规划期会被拦截并使凭证标记为 `Fail`。
5. **编译与单元测试合规**:
   - `cargo check -p intelligence-agent-core` 通过，0 编译错误。
   - `cargo test -p intelligence-agent-core security_fix` 5 个漏洞严重级、白名单网关、生命周期状态转换与干跑策略单元测试通过。
   - `cargo test -p intelligence-agent-core security_workflow` 9 个工作流生命周期、严重级分类单元测试通过。
