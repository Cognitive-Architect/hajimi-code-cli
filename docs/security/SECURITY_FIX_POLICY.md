# Hajimi IDE Security Fix Policy

> **Policy Version**: v1.0.0  
> **Status**: APPROVED / DRY-RUN ONLY  
> **Updated**: 2026-05-23  

---

## 1. 核心安全红线 (Core Safety Boundaries)

Hajimi IDE 作为一个本地优先的 AI 智能体 IDE，在执行安全修复（Security Fix）时必须严格遵守以下安全红线。任何违反安全红线的设计与代码实现都将被自动质量门禁直接打回：

1. **默认 Dry-run 机制**: 所有 `/security fix` 以及界面上的“生成修复计划”交互均默认为 `dry-run: true`。本日及当前架构阶段不真正执行文件物理修改。
2. **严禁越过审批层 (No Silent Bypass)**: 任何阶段的修复应用都绝不允许通过 `fs::write` 或类似的 API 直接裸写修改工作区中的业务代码。任何形式的代码变更应用必须走系统统一的 `EditApplier`、`governance`、`approval` 与工作区路径白名单（workspace path constraints）约束。
3. **高危漏洞强人工拦截 (Human Review Required for High/Critical)**: 
   - 对于 **High** 和 **Critical** 风险级别的漏洞，系统禁止生成任何可自动应用或静默部署的代码补丁。
   - Planner 只生成建议性的 dry-run plan，且该 PatchPlan 中的 `human_review_required` 属性必须强制硬编码为 `true`。
   - 所有高危漏洞补丁必须由人类开发者在 Diff 视图中进行逐行人工审查与签名确认后，方可交互式决定接受或拒绝。
4. **低风险同样受控 (Controlled Low Risk)**: 即便漏洞等级为 Low 或 Medium，系统也同样默认执行 dry-run，提供清晰的修复建议、可回滚策略以及复测命令，不得进行任何默认的后台静默自动修复。
5. **双翼齐全原则 (Validation & Rollback Requirements)**: 任何被生成或交付的 `PatchPlan` 都必须强制携带：
   - **复测命令 (`validation_commands`)**: 用于校验漏洞在应用补丁后是否切实被修复（如：`npm run test:security-gate` 或特定回归测试命令）。
   - **回退方案 (`rollback_plan`)**: 清晰的指令指导开发者如果在应用补丁后发生逻辑中断或编译崩溃时如何瞬间回滚至安全初始状态（例如：`git checkout -- <file>`）。

---

## 2. 漏洞修复风险策略矩阵 (Security Fix Risk Policy Matrix)

系统基于 `FindingSeverity` 对发现的安全漏洞划分等级，并定义相匹配的响应策略：

| 严重等级 (Severity) | 映射风险等级 (RiskLevel) | 默认 Dry-run | 强制人工确认 (Human Review) | 变更实施通道 (Apply Channel) | 必备交付项 |
|:---|:---|:---:|:---:|:---|:---|
| **Critical** | Critical | **Yes** | **Yes** | 必须由 `EditApplier` 拦截生成 UI Diff 供人工签名 | `validation_commands`, `rollback_plan`, 逐行说明 |
| **High** | High | **Yes** | **Yes** | 必须由 `EditApplier` 拦截生成 UI Diff 供人工签名 | `validation_commands`, `rollback_plan`, 逐行说明 |
| **Medium** | Medium | **Yes** | **Yes** | 交互式 Diff 审批后应用 | `validation_commands`, `rollback_plan` |
| **Low** | Low | **Yes** | **Yes** | 交互式 Diff 审批后应用 | `validation_commands`, `rollback_plan` |

---

## 3. 自动化等级与机制 (Automation Level)

当前系统的自动化修复能力定义为 **Level 1 (Suggested Advisory Only)**：
* **不进行任何静默代码修改**。
* **规划器 (Planner)**: `SecurityFixPlanner` 依据漏洞类型、所在文件、行号以及建议措施（Recommendation），在内存中动态组装并交付富文本的 `PatchPlan` DTO，该 DTO 包括需要修复的源文件、开始行、结束行，以及计划插入/替换的虚拟 edits。
* **回归测试自动装载**: 规划器会智能抓取发现漏洞时附带的 `regression_test`，自动将其升级为 PatchPlan 中的第一顺位 `validation_commands`，以实现自动化复测的无缝契约交付。

---

## 4. 技术审查与合规

本 Policy 全面落地于 Rust 后端智能层（Intelligence Layer）的 `src/intelligence/agent-core/security_fix.rs` 与 `src/intelligence/agent-core/security_workflow.rs`，确保了以下技术合规点：
- **不向上依赖 (Strict Layering)**: 本修复决策层零依赖 Interface 层（Tauri desktop/Web 交互），纯粹通过 DTO 响应 slash 契约。
- **配置门禁支持**: 引入环境变量 `HAJIMI_SECURITY_FIX_ENABLED`，默认禁用；无论该门禁开启与否，安全红线与强制 dry-run 机制都硬性生效以提供最大的底线防御。
