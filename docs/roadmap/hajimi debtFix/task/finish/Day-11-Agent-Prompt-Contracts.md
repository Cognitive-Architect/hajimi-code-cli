# Day 11 派单: Agent Prompt V2 契约文档化

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 11，处理 `DEBT-AGENT-PROMPT-001` 的 P2 质量增强部分。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Architect）
- **任务名称**: Agent Prompt V2 契约文档化
- **轰炸目标**: 在 `docs/agent-prompt-core/` 下补齐 Persona、Planner、Reflector、Executor、Tool Manifest 五份契约文档，并和当前 `agent-core` 实现对齐
- **任务性质**: 文档架构 + 质量增强
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 5 份契约文档 + 与当前代码事实一致 + 不修改核心运行逻辑
- **通用铁律**:
  1. 文档不得宣称未实现功能已实现
  2. 契约必须包含输入、输出、失败降级、feature-gate、证据字段
  3. 必须对齐当前默认开启的 Persona、Planner V1、Context Window、Reflector V1 等 feature-gate
  4. 不抢 P0/P1 安全优先级，不改核心逻辑
  5. 所有“建议后续”必须标注为 future / debt，不得混成已交付

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `DEBT-AGENT-PROMPT-001` 当前 `PARTIAL / P2` | 债务总表第 8.1 节 | 必须 |
| Feature gates | 当前 prompt gate 位置 | `src/intelligence/agent-core/prompts/mod.rs:6-51` | 必须 |
| Planner DTO | Planner V1 schema | `src/intelligence/agent-core/planner_dto.rs` | 必须 |
| Reflector DTO | Reflector V1 schema | `src/intelligence/agent-core/reflector_dto.rs` | 必须 |
| Tool Manifest | 工具 manifest schema | `src/intelligence/agent-core/tool_manifest.rs` | 必须 |
| Act/Executor | ToolCall/ActExecutor blackboard 协议 | `src/intelligence/agent-core/act_executor.rs` | 必须 |
| 输出目录 | 契约文档目录 | `docs/agent-prompt-core/` | 必须 |

### 探索补充栏

本任务不写运行时代码。若发现代码事实和债务总表不一致，只更新文档事实，不尝试在 Day 11 修代码。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-11/15
- **角色**: Architect
- **目标**: 形成 Agent Prompt V2 的可执行契约文档，为 Day 12 golden regression 提供标准
- **输入**: `prompts/mod.rs`, `planner_dto.rs`, `reflector_dto.rs`, `tool_manifest.rs`, `act_executor.rs`
- **依赖关系**: 建议在 Day 10 后执行；Day 12 依赖本日文档

### 2）输出交付物

- **变更文件**:
  - `docs/agent-prompt-core/AGENT-PERSONA.md`
  - `docs/agent-prompt-core/PLANNER-PROMPT-CONTRACT.md`
  - `docs/agent-prompt-core/REFLECTOR-CONTRACT.md`
  - `docs/agent-prompt-core/EXECUTOR-CONTRACT.md`
  - `docs/agent-prompt-core/TOOL-MANIFEST-SCHEMA.md`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`，如状态同步需要
- **核心修改点**:
  - Persona: 角色边界、安全边界、不可伪造证据
  - Planner: `PlannerSubgoalPlanV1` 输入/输出/fallback/stop conditions
  - Reflector: `ReflectorCritiqueV1` root cause、recommended action、stop-loss
  - Executor: ToolCall、governance、blackboard keys、失败恢复
  - Tool Manifest: schema、过滤未知工具、权限字段
- **必须包含**:
  - 每份文档引用对应源码路径
  - feature-gate 默认行为与回滚方式
  - 与 Day 12 golden cases 的映射说明
- **禁止包含**:
  - 伪称 Prompt V2 已全量产品化
  - 删除或覆盖现有 `agent_persona.md`
  - 修改 `src/intelligence/agent-core` 运行逻辑
  - 写“见上文”作为输入基线
- **交付证明**:
  - `Get-ChildItem -LiteralPath docs/agent-prompt-core`
  - `rg -n "PlannerSubgoalPlanV1|ReflectorCritiqueV1|Tool Manifest|feature-gate|fallback" docs/agent-prompt-core`

### 3）规模与复杂度观察

- **推荐目标**: 每份文档聚焦契约，不写长篇愿景
- **复杂度说明**: 如果契约无法与当前代码对齐，声明 `DEBT-DOC-B11-001` 而不是写假契约
- **禁止行为**: 用营销式描述替代输入/输出 schema

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 本日不改代码 | `git diff -- src/intelligence/agent-core` 应为空或只含用户已有改动 | 返工 |
| FMT | Markdown 人工检查 | N/A，项目无 Markdown formatter | 说明原因 |
| LINT | 契约关键字可搜索 | `rg -n "fallback|feature-gate|schema_version|stop" docs/agent-prompt-core` | 返工 |
| TEST | agent-core 现有测试可跑或记录 | `cargo test -p intelligence-agent-core --lib` 或 N/A 原因 | 有条件 |
| ARCH | 不改分层代码 | `git diff -- src` | 返工 |
| REAL | 文档和源码事实一致 | `rg` 对照源码符号 | 返工 |
| DOC | 5 份文档存在 | `Get-ChildItem -LiteralPath docs/agent-prompt-core -Filter *.md` | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | Persona 契约存在 | `Test-Path docs/agent-prompt-core/AGENT-PERSONA.md` | [ ] |
| FUNC | FUNC-002 | Planner 契约存在 | `Test-Path docs/agent-prompt-core/PLANNER-PROMPT-CONTRACT.md` | [ ] |
| FUNC | FUNC-003 | Reflector 契约存在 | `Test-Path docs/agent-prompt-core/REFLECTOR-CONTRACT.md` | [ ] |
| FUNC | FUNC-004 | Executor 契约存在 | `Test-Path docs/agent-prompt-core/EXECUTOR-CONTRACT.md` | [ ] |
| CONST | CONST-001 | Tool Manifest schema 存在 | `Test-Path docs/agent-prompt-core/TOOL-MANIFEST-SCHEMA.md` | [ ] |
| CONST | CONST-002 | feature-gate 行为已写明 | `rg -n "HAJIMI_|feature-gate|默认" docs/agent-prompt-core` | [ ] |
| CONST | CONST-003 | fallback 行为已写明 | `rg -n "fallback|降级|legacy" docs/agent-prompt-core` | [ ] |
| CONST | CONST-004 | 代码未被修改 | `git diff -- src/intelligence/agent-core` | [ ] |
| NEG | NEG-001 | 未宣称未实现功能 | 人工检查“future/debt/已实现”区分 | [ ] |
| NEG | NEG-002 | 未删除 legacy 路径 | `git diff -- src/intelligence/agent-core` | [ ] |
| NEG | NEG-003 | 未跳过安全约束 | `rg -n "安全|evidence|证据|禁止" docs/agent-prompt-core` | [ ] |
| NEG | NEG-004 | 未写空泛模板 | `rg -n "TODO|TBD|见上文|参考ID" docs/agent-prompt-core` 应无偷懒占位 | [ ] |
| UX | UX-001 | 后续 agent 可读懂输入输出 | 每份文档有 Input/Output/Failure 段 | [ ] |
| UX | UX-002 | Day 12 映射明确 | `rg -n "golden|regression|Day 12" docs/agent-prompt-core` | [ ] |
| E2E | E2E-001 | agent-core 测试未破坏 | `cargo test -p intelligence-agent-core --lib` 或 N/A 原因 | [ ] |
| High | HIGH-001 | Prompt 债状态不被伪清偿 | 债务总表保持 `PARTIAL/P2` 或有证据推进 | [ ] |

---

## 【模块3-B】地狱红线

1. 文档没有源码路径，返工
2. 文档宣称未实现功能已实现，返工
3. 运行逻辑被修改，返工
4. 五份契约缺任一份，返工
5. feature-gate/fallback 未写，返工
6. Day 12 无法基于文档写 golden cases，返工
7. 出现 `TODO/TBD/见上文` 偷懒占位，返工
8. agent prompt 状态直接标 `CLEARED`，返工
9. 与 `planner_dto/reflector_dto/tool_manifest` 字段不一致，返工
10. 忽略安全/证据约束，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 五份契约是否齐全 | [ ] | FUNC-001~004, CONST-001 | |
| RG | 是否对齐当前代码事实 | [ ] | REAL 闸门 | |
| NG | 是否无占位/伪实现声明 | [ ] | NEG-001, NEG-004 | |
| UX | 后续 agent 是否能直接用 | [ ] | UX-001~002 | |
| E2E | 现有测试是否未破坏 | [ ] | E2E-001 | |
| High | 状态是否诚实 | [ ] | HIGH-001 | |
| 字段完整性 | 是否包含 input/output/failure | [ ] | 文档 | |
| 需求映射 | 是否映射 `DEBT-AGENT-PROMPT-001` | [ ] | 债务总表 8.1 | |
| 自测执行 | 是否跑过搜索/测试 | [ ] | 质量闸门 | |
| 范围边界与债务 | 未实现项是否声明 future | [ ] | 文档 | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-11/15 完成并提交

### 提交信息
- Commit: `docs(intelligence/agent-core): define prompt v2 contracts`
- 分支: `<实际分支>`
- 变更文件:
  - `docs/agent-prompt-core/AGENT-PERSONA.md`
  - `docs/agent-prompt-core/PLANNER-PROMPT-CONTRACT.md`
  - `docs/agent-prompt-core/REFLECTOR-CONTRACT.md`
  - `docs/agent-prompt-core/EXECUTOR-CONTRACT.md`
  - `docs/agent-prompt-core/TOOL-MANIFEST-SCHEMA.md`

### 本轮目标与实际结果
- 目标: Prompt V2 契约文档化
- 实际完成: `<列出五份文档和源码对齐点>`
- 未完成/不在范围: 运行时代码改造和 golden tests 属 Day 12+

### 自动化质量检查报告
- `Get-ChildItem docs/agent-prompt-core`: `<摘要>`
- `rg -n "PlannerSubgoalPlanV1|ReflectorCritiqueV1|Tool Manifest" docs/agent-prompt-core`: `<摘要>`
- `git diff -- src/intelligence/agent-core`: `<摘要>`

### 债务声明
- `DEBT-DOC-B11-001`: `<如有未对齐契约或 future 项>`

### 风险与回滚点
- 主要风险: 文档过度承诺
- 回滚方式: 回退 docs/agent-prompt-core 本日新增文档
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| DOC-001 | 代码事实和计划目标冲突 | 以代码事实为准，目标写 future | 防止文档造假 |
| QUALITY-001 | 五份文档过长但缺 schema | 停止扩写，补输入输出表 | 返工 |
| TEST-001 | agent-core 测试因环境无法跑 | 记录原因，本日以文档搜索为主 | 有条件交付 |
| ARCH-001 | 需要改代码才能满足契约 | 不改代码，登记 Day 12+ 债务 | 保持范围 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 11 Agent Prompt V2 契约文档化**。

### 关键约束
- 只写契约，不改运行逻辑
- 文档必须和当前代码事实一致
- 五份契约缺一不可
- Agent Prompt 债不直接清偿

### 验收铁律
- 5 份文档存在
- feature-gate/fallback/schema 都写明
- 无偷懒占位
- Day 12 可直接写 golden cases

闭环启动，Day 11，执行。
