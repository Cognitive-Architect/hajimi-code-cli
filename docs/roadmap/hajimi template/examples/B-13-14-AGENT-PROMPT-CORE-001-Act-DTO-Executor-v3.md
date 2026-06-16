# 派单示例：B-13/14 — Act DTO + ActExecutor 骨架 + 工具调用执行（v3.0 格式）

> **本示例基于 ID-59 v3.0 通用增强版模板重写**
> **任务来源**：AGENT-PROMPT-CORE-001 Phase 5 Day 13
> **原始文档**：B-13-14-AGENT-PROMPT-CORE-001-Act-DTO-Executor.md（旧模板）

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：Act DTO 定义 + ActExecutor 骨架 + 工具调用执行 + governance 路由
- **轰炸目标**：新建 `act_dto.rs` 和 `act_executor.rs`，定义 `ToolCallV1` DTO 和 `ActExecutor` 执行逻辑，注册到 `lib.rs`
- **任务性质**：功能开发 + 骨架搭建
- **输入基线**：完整技术背景（见模块2）
- **输出要求**：可执行产出 + 自动化质量闸门全通过 + 显式债务声明 + 结构化收卷
- **通用铁律**：
  1. **数据诚实**：所有测试数、warning 数必须来自真实 `cargo test` / `cargo clippy` 输出
  2. **零占位符**：禁止写"参考Day 12""见上文"，必须给出完整路径 + SHA
  3. **自动化优先**：刀刃表 16 项必须提供可执行验证命令
  4. **最小必要复杂度**：单函数 >50 行需解释理由，不为压行数硬拆函数
  5. **债务透明化**：如触发复杂度熔断必须声明 `DEBT-COMPLEXITY`

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 模块/文件/函数范围 | `src/intelligence/agent-core/act_dto.rs`（新建）<br>`src/intelligence/agent-core/act_executor.rs`（新建）<br>`src/intelligence/agent-core/lib.rs`（追加模块注册） | 必须 |
| 现状基线 | Phase 4 已完成 ContextWindowManager，需实现工具自主选择和执行 | `grep -n "pub mod" src/intelligence/agent-core/lib.rs` | 必须 |
| 目标结果 | 1. 定义 `ActionType`/`ToolCallV1`/`ActDecision` DTO<br>2. 实现 `ActExecutor` 骨架和 `ActLlmBridge`<br>3. 实现 `execute_tool_call` 验证和执行逻辑<br>4. 注册到 `lib.rs` | `cargo check -p intelligence-agent-core` 0 errors | 必须 |
| 技术约束 | 1. `ToolCallV1` 参数为有效 JSON<br>2. `ActExecutor` 验证 tool 存在于 registry<br>3. Critical risk 工具路由到 governance<br>4. 工具不存在时返回 `ToolError`（不 panic）<br>5. 每次只选一个工具 | 文字展开 | 必须 |
| 风险边界 | 禁止直接调用未验证的工具、禁止 `panic!` | `grep "panic!" src/intelligence/agent-core/act_executor.rs` = 0 | 必须 |
| 测试基线 | 当前编译 / 测试状态 | `cargo check -p intelligence-agent-core` 0 errors<br>`cargo test -p intelligence-agent-core --lib` 107 passed | 必须 |
| 文档同步要求 | 无新增公开文档 | N/A | 按需 |
| 历史债务 / 相关缺陷 | 无 | N/A | 按需 |

### 探索补充栏
本任务为**已知解实现**，无需探索补充栏。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息
- **工单编号**：B-13/14
- **角色**：Engineer
- **目标**：Act DTO 定义 + ActExecutor 骨架 + 工具调用执行 + governance 路由
- **输入**：引用输入基线中"目标范围"与"现状基线"所有项
- **依赖关系**：依赖 Phase 4 完成（无并行依赖）

### 2）输出交付物
- **变更文件**：
  - `src/intelligence/agent-core/act_dto.rs`（新建）
  - `src/intelligence/agent-core/act_executor.rs`（新建）
  - `src/intelligence/agent-core/lib.rs`（追加模块注册）
- **核心修改点**：
  - 定义 `ActionType`/`ToolCallV1`/`ActDecision`
  - 实现 `ActExecutor` 和 `ActLlmBridge` 骨架
  - 实现 `execute_tool_call`（验证 → governance 路由 → 执行）
  - 注册模块到 `lib.rs`
- **必须包含**：
  - `grep -c "ActionType\|ToolCallV1\|ActDecision"` ≥ 3
  - `grep -c "struct ActExecutor"` ≥ 1
  - `grep -c "fn execute_tool_call"` ≥ 1
  - `grep -c "governance_required\|RiskLevel::Critical"` ≥ 1
  - `grep -c "pub mod act_dto\|pub mod act_executor"` ≥ 2
- **禁止包含**：`rm`、`unsafe`、`panic!`、`unwrap()`（业务逻辑）
- **交付证明**：`cargo check -p intelligence-agent-core` 0 errors + 刀刃表 16 项命令输出

### 3）规模与复杂度观察
- **推荐目标**：单函数保持单一职责，`execute_tool_call` ≤50 行
- **复杂度说明**：`ToolCallV1` DTO 字段较多（12 个），`execute_tool_call` 需处理验证 + governance 路由，预计单函数 40-60 行。如超过 60 行将声明 `DEBT-COMPLEXITY-B13-001`
- **禁止行为**：为压行数硬拆函数、添加无意义中间层

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 编译通过 | `cargo check -p intelligence-agent-core` | 返工 |
| FMT | 格式检查通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p intelligence-agent-core -- -D warnings` | 返工或声明债务 |
| TEST | 单元测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工 |
| ARCH | 不违反分层与接口约束 | `grep "panic!" src/intelligence/agent-core/act_executor.rs` = 0 | 返工 |
| REAL | 禁止假实现 | 所有测试真实执行，无 `#[ignore]` | 返工 |
| DOC | 无需新增文档 | N/A | - |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `ActionType` enum 包含 4 个变体 | `grep -c "CallTool,\|CannotAct,\|AskUser,\|StopAndHandoff" src/intelligence/agent-core/act_dto.rs` ≥ 4 | [ ] |
| FUNC | FUNC-002 | `ToolCallV1` 包含 12 个字段 | `grep -c "idempotency_key\|next_step_hint\|fallback_tool" src/intelligence/agent-core/act_dto.rs` ≥ 3 | [ ] |
| FUNC | FUNC-003 | `ActDecision` enum 包含 4 个变体 | `grep -c "enum ActDecision" src/intelligence/agent-core/act_dto.rs` ≥ 1 | [ ] |
| FUNC | FUNC-004 | `ActExecutor::execute_tool_call` 验证 tool 存在于 registry 再执行 | `grep -c "tool_registry.*get\|Tool not found" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| CONST | CONST-001 | `ActExecutor` 验证 parameters 为有效 JSON | `grep -c "parameters.*json\|serde_json" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| CONST | CONST-002 | Critical risk 工具路由到 governance | `grep -c "governance_required\|Critical\|AgentGovernance" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| CONST | CONST-003 | 工具不存在时返回 `ToolError`（不 panic） | `grep -c "ToolError\|ok_or" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| CONST | CONST-004 | `lib.rs` 注册 `act_dto` 和 `act_executor` 模块 | `grep -c "pub mod act_dto" src/intelligence/agent-core/lib.rs` ≥ 1 && `grep -c "pub mod act_executor" src/intelligence/agent-core/lib.rs` ≥ 1 | [ ] |
| NEG | NEG-001 | 工具执行失败时返回错误（不 panic） | `grep -c "await.*execute\|Result.*ToolError" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| NEG | NEG-002 | `ActLlmBridge` prompt 要求返回 ONLY valid JSON | `grep -c "ONLY valid JSON\|ToolCallV1" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| NEG | NEG-003 | 编译无 error | `cargo check -p intelligence-agent-core` 退出码 0 | [ ] |
| NEG | NEG-004 | 无 `unsafe` | `grep -c "unsafe" src/intelligence/agent-core/act_dto.rs src/intelligence/agent-core/act_executor.rs` = 0 | [ ] |
| UX | UX-001 | 每个 pub 类型有 rustdoc | `grep -c "^///" src/intelligence/agent-core/act_dto.rs` ≥ 5 | [ ] |
| UX | UX-002 | `execute_tool_call` 有详细 rustdoc | `grep -c "///.*execute_tool_call" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| E2E | E2E-001 | workspace 编译通过 | `cargo check --workspace` 退出码 0 | [ ] |
| High | HIGH-001 | 工具不存在时不 panic | `grep -c "panic!" src/intelligence/agent-core/act_executor.rs` = 0 | [ ] |

---

## 【模块3-B】地狱红线（10项）
1. 零占位符违规 → 返工
2. 验证造假（声称已验证但无命令输出） → 返工
3. 编译 / 测试失败 → 返工
4. 假实现 / mock 成功 → 返工
5. 架构约束违反（直接调用未验证的工具） → 返工
6. 新增 warning 未申报 → 返工
7. 范围失控 → 返工
8. Git 历史不完整 → 返工
9. 复杂度超标且未声明 `DEBT-COMPLEXITY` → 返工
10. 探索任务伪装确定性完成 → 返工

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | Act DTO 字段完整吗？ActExecutor 能执行工具吗？ | [ ] | CF-B13-001 | |
| 约束与回归用例（RG） | governance 路由对 Critical 生效吗？ | [ ] | RG-B13-001 | |
| 负面路径用例（NG） | 工具不存在时返回错误了吗？无 panic 吗？ | [ ] | NG-B13-001 | |
| 用户体验用例（UX） | rustdoc 覆盖每个 pub 类型吗？ | [ ] | UX-B13-001 | |
| 端到端关键路径（E2E） | `cargo check --workspace` 通过吗？ | [ ] | E2E-B13-001 | |
| 高风险场景（High） | 工具不存在时不 panic 吗？ | [ ] | High-B13-001 | |
| 字段完整性 | 每条用例前置/预期/实际/风险等级 | [ ] | — | |
| 需求映射 | 用例映射到 Day 13 任务清单 | [ ] | — | |
| 自测执行 | 完整跑过一轮自测 | [ ] | — | |
| 范围边界与债务 | 未覆盖项明确标注 | [ ] | — | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## ✅ 工单 B-13/14 完成并提交

### 提交信息
- Commit: `feat(intelligence/agent-core): add Act DTO, ActExecutor skeleton, and tool call execution`
- 分支: `feature/AGENT-PROMPT-CORE-001-phase5`
- 变更文件:
  - `src/intelligence/agent-core/act_dto.rs`（新建）
  - `src/intelligence/agent-core/act_executor.rs`（新建）
  - `src/intelligence/agent-core/lib.rs`（追加模块注册）

### 本轮目标与实际结果
- 目标: Act DTO 定义 + ActExecutor 骨架 + 工具调用执行 + governance 路由
- 实际完成: 全部 3 个文件修改完成，16 项刀刃表通过
- 未完成/不在范围: 链式协议仅骨架（Day 14 填充实现）

### 关键决策记录
- DECISION-001: `ActLlmBridge` prompt 要求返回 ONLY valid JSON —— 避免 LLM 返回非 JSON 格式导致解析失败
- DECISION-002: `execute_tool_call` 验证 tool 存在后再执行 —— 防止调用未注册的工具导致 panic

### 自动化质量检查报告
```bash
# BUILD
cargo check -p intelligence-agent-core
# 结果摘要: 0 errors

# FMT
cargo fmt -- --check
# 结果摘要: 无格式问题

# LINT
cargo clippy -p intelligence-agent-core -- -D warnings
# 结果摘要: 0 warnings

# TEST
cargo test -p intelligence-agent-core --lib
# 结果摘要: 107 passed（原105 + 新增2）
```

### 刀刃表摘要
| 类别 | 覆盖数 | 关键证据 |
|:---|:---:|:---|
| FUNC | 4/4 | 命令输出已验证 |
| CONST | 4/4 | 命令输出已验证 |
| NEG | 4/4 | 命令输出已验证 |
| UX | 2/2 | 命令输出已验证 |
| E2E | 1/1 | 命令输出已验证 |
| High | 1/1 | 命令输出已验证 |

### P4检查表摘要
| 检查点 | 状态 | 备注 |
|:---|:---:|:---|
| CF | [x] | |
| RG | [x] | |
| NG | [x] | |
| UX | [x] | |
| E2E | [x] | |
| High | [x] | |

### 规模与复杂度说明
- 关键函数: `ActExecutor::execute_tool_call`、`ActLlmBridge::llm_decide`
- 是否存在复杂度例外: 无
- 若有，说明: N/A

### 债务声明
- DEBT-COMPLEXITY-B13-001: 无（未触发）
- DEBT-TEST-B13-001: 无
- DEBT-SCOPE-B13-001: 链式协议仅骨架，Day 14 填充实现

### 风险与回滚点
- 主要风险: `engine_tool_system::ToolRegistry` trait 不在公共接口
- 回滚方式: `git revert <commit>` 或使用具体类型 wrapper
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | `engine_tool_system::ToolRegistry` trait 不在公共接口 | 暂停实现，使用具体类型或 wrapper | 延期 |
| QUALITY-001 | 自动化闸门连续 2 次不通过 | 停止堆代码，先修复质量 | 返工 |
| COMPLEXITY-001 | `execute_tool_call` 单函数 >60 行且连续 2 次返工 | 允许带 `DEBT-COMPLEXITY` 交付 | 记录债务 |
| SCOPE-001 | 误实现链式协议完整逻辑 | 回退到骨架，标记 TODO | 返工 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **AGENT-PROMPT-CORE-001 Phase 5 Day 13** 通用高压任务！

### 技术背景
Phase 4 已完成 ContextWindowManager。Phase 5 目标是实现工具自主选择和执行。需新建 `act_dto.rs` 定义 `ToolCallV1` DTO，新建 `act_executor.rs` 实现 `ActExecutor` 和 `ActLlmBridge`，并注册到 `lib.rs`。

### 关键约束
- `ActionType` 4 变体：`CallTool`, `CannotAct`, `AskUser`, `StopAndHandoff`
- `ToolCallV1` 字段：schema_version, action_type, tool_name, parameters, reason, expected_output, expected_evidence, fallback_tool, governance_required, risk_level, idempotency_key, next_step_hint
- `ActDecision` 4 变体：`ToolCall(ToolCallV1)`, `CannotAct { reason }`, `AskUser { reason }`, `StopAndHandoff { reason }`
- `ActExecutor` 字段：`tool_registry: Arc<dyn ToolRegistry>`, `governance: Arc<dyn AgentGovernance>`
- `execute_tool_call` 流程：验证 tool 存在 → 验证 parameters 为 JSON → governance 路由（Critical）→ 执行 tool → 返回 ToolOutput
- 工具不存在时返回 `ToolError`（不 panic）
- `ActLlmBridge` prompt 要求返回 ONLY valid JSON matching ToolCallV1
- 每次只选一个工具
- 零 `unsafe`，零业务逻辑 `unwrap()`
- 现有 107 个单元测试必须全部通过

### 质量红线
- 10 项地狱红线生效
- 刀刃表 16 项必须命令化验证
- 禁止使用绝对行数限制

### 工单并行矩阵
- B-13/14 Engineer：Act DTO + ActExecutor 骨架 + 工具调用执行

### 验收铁律
- `cargo check -p intelligence-agent-core` 0 errors
- `cargo test -p intelligence-agent-core --lib` 107 passed（不破坏现有测试）
- `grep "ToolCallV1" src/intelligence/agent-core/act_dto.rs` ≥ 1
- `grep "ActExecutor" src/intelligence/agent-core/act_executor.rs` ≥ 1
- `grep "fn execute_tool_call" src/intelligence/agent-core/act_executor.rs` ≥ 1
- `grep "governance_required\|RiskLevel::Critical" src/intelligence/agent-core/act_executor.rs` ≥ 1
- `grep "pub mod act_dto" src/intelligence/agent-core/lib.rs` ≥ 1
- `grep "pub mod act_executor" src/intelligence/agent-core/lib.rs` ≥ 1
- `grep -c "panic!" src/intelligence/agent-core/act_executor.rs` = 0
- `cargo check --workspace` 0 errors

### 收卷要求
- 必须附自动化质量检查摘要
- 必须附刀刃表摘要
- 必须诚实声明债务

Ouroboros 闭环启动，**AGENT-PROMPT-CORE-001 Day 13**，执行！ ☝️🐍♾️🔥

---

## 【模块8】通用验证命令库（Rust 适用）

```bash
git rev-parse HEAD
git branch --show-current
cargo check -p intelligence-agent-core
cargo fmt -- --check
cargo clippy -p intelligence-agent-core -- -D warnings
cargo test -p intelligence-agent-core --lib
cargo test -p intelligence-agent-core --lib -- --nocapture
```

---

**本示例已完全按照 ID-59 v3.0 通用增强版模板格式重写，可直接复制使用。**
