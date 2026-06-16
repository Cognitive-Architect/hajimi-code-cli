# 派单示例：B-14/14 — 多步链式协议 + 重试规则 + Feature-Gate（v3.0 格式）

> **本示例基于 ID-59 v3.0 通用增强版模板重写**
> **任务来源**：AGENT-PROMPT-CORE-001 Phase 5 Day 14
> **原始文档**：B-14-14-AGENT-PROMPT-CORE-001-Act-Chain-Retry-FeatureGate.md（旧模板）

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：多步链式协议 + 重试规则 + micro-reflect + AgentLoop 集成 + feature-gate
- **轰炸目标**：修改 `act_executor.rs` 实现链式协议和重试规则，修改 `agent_loop.rs` 调用 ActExecutor，添加 `HAJIMI_ACT_TOOLCALL_V1_ENABLED` feature-gate
- **任务性质**：功能开发 + 集成 + feature-gate
- **输入基线**：完整技术背景（见模块2）
- **输出要求**：可执行产出 + 自动化质量闸门全通过 + 显式债务声明 + 结构化收卷
- **通用铁律**：
  1. **数据诚实**：所有测试数、warning 数必须来自真实 `cargo test` / `cargo clippy` 输出
  2. **零占位符**：禁止写"参考Day 13""见上文"，必须给出完整路径 + SHA
  3. **自动化优先**：刀刃表 16 项必须提供可执行验证命令
  4. **最小必要复杂度**：单函数 >50 行需解释理由，不为压行数硬拆函数
  5. **债务透明化**：如触发复杂度熔断必须声明 `DEBT-COMPLEXITY`

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 模块/文件/函数范围 | `src/intelligence/agent-core/act_executor.rs:225-255`<br>`src/intelligence/agent-core/agent_loop.rs:70-90`<br>`src/intelligence/agent-core/prompts/mod.rs` | 必须 |
| 现状基线 | Day 13 已完成 `ActExecutor` 骨架和 `ToolCallV1` DTO | `grep -n "struct ActExecutor\|fn execute_tool_call" src/intelligence/agent-core/act_executor.rs` | 必须 |
| 目标结果 | 1. 定义 6 个 blackboard 链式 keys<br>2. 实现链式协议（blackboard 状态传递）<br>3. 实现重试规则（fingerprint 去重 + 修正重试 + 两次失败 handoff）<br>4. 实现 micro-reflect<br>5. 修改 `AgentLoop::act()` 调用 `ActExecutor`<br>6. 添加 `is_act_toolcall_v1_enabled()` feature-gate | `cargo test -p intelligence-agent-core --lib` 全部通过 | 必须 |
| 技术约束 | 1. 相同 tool + 相同参数 fingerprint → 不重复调用<br>2. 修正参数 → 可重试一次<br>3. 两次同类失败 → 强制 `StopAndHandoff`<br>4. 链式状态通过 blackboard 传递<br>5. micro-reflect 仅分析当前工具错误<br>6. feature-gate 关闭时回到旧执行路径 | 文字展开 | 必须 |
| 风险边界 | 禁止删除旧 `act` 逻辑（作为 fallback 保留） | `grep "legacy.*act\|old.*act" src/intelligence/agent-core/agent_loop.rs` ≥ 1 | 必须 |
| 测试基线 | 当前编译 / 测试状态 | `cargo check -p intelligence-agent-core` 0 errors<br>`cargo test -p intelligence-agent-core --lib` 107 passed | 必须 |
| 文档同步要求 | 无新增公开文档 | N/A | 按需 |
| 历史债务 / 相关缺陷 | 无 | N/A | 按需 |

### 探索补充栏
本任务为**已知解实现**，无需探索补充栏。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息
- **工单编号**：B-14/14
- **角色**：Engineer
- **目标**：多步链式协议 + 重试规则 + micro-reflect + AgentLoop 集成 + feature-gate
- **输入**：引用输入基线中"目标范围"与"现状基线"所有项
- **依赖关系**：依赖 Day 13 完成（无并行依赖）

### 2）输出交付物
- **变更文件**：
  - `src/intelligence/agent-core/act_executor.rs`
  - `src/intelligence/agent-core/agent_loop.rs`
  - `src/intelligence/agent-core/prompts/mod.rs`
- **核心修改点**：
  - 定义 6 个 blackboard 链式 keys（BB_NEXT_TOOL/BB_LAST_TOOL/BB_LAST_TOOL_RESULT/BB_LAST_ERROR/BB_FAILED_TOOL_FINGERPRINT/BB_ATTEMPT_COUNT）
  - 实现链式协议（Act 完成后写入 blackboard，下次读取决定下一步）
  - 实现重试规则（fingerprint 去重 + 修正重试 + 两次失败 handoff）
  - 实现 micro-reflect（工具失败时简短反思）
  - 修改 `AgentLoop::act()` 调用 `ActExecutor`
  - 添加 `is_act_toolcall_v1_enabled()` feature-gate 函数
- **必须包含**：
  - `grep -c "const BB_NEXT_TOOL\|const BB_LAST_TOOL\|const BB_LAST_TOOL_RESULT\|const BB_LAST_ERROR\|const BB_FAILED_TOOL_FINGERPRINT\|const BB_ATTEMPT_COUNT"` ≥ 6
  - `grep -c "blackboard.*write\|blackboard.*read"` ≥ 2
  - `grep -c "micro_reflect\|fingerprint"` ≥ 2
  - `grep -c "ActExecutor\|act_executor"` ≥ 1
  - `grep -c "is_act_toolcall_v1_enabled"` ≥ 1
- **禁止包含**：`rm`、`unsafe`、`panic!`、删除旧 `act` 逻辑
- **交付证明**：`cargo test -p intelligence-agent-core --lib` 全部通过 + 刀刃表 16 项命令输出

### 3）规模与复杂度观察
- **推荐目标**：单函数保持单一职责，重试逻辑提取为独立 `RetryPolicy` ≤40 行
- **复杂度说明**：链式协议涉及 6 个 blackboard keys + fingerprint 计算 + 状态机，预计单函数 50-70 行。如超过 70 行将声明 `DEBT-COMPLEXITY-B14-001`
- **禁止行为**：为压行数硬拆函数、添加无意义中间层

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 编译通过 | `cargo check -p intelligence-agent-core` | 返工 |
| FMT | 格式检查通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p intelligence-agent-core -- -D warnings` | 返工或声明债务 |
| TEST | 单元测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工 |
| ARCH | 不违反分层与接口约束 | `grep "legacy.*act\|old.*act" src/intelligence/agent-core/agent_loop.rs` ≥ 1 | 返工 |
| REAL | 禁止假实现 | 所有测试真实执行，无 `#[ignore]` | 返工 |
| DOC | 无需新增文档 | N/A | - |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | 6 个 blackboard 链式 keys 定义为 const | `grep -c "const BB_NEXT_TOOL\|const BB_LAST_TOOL\|const BB_LAST_TOOL_RESULT\|const BB_LAST_ERROR\|const BB_FAILED_TOOL_FINGERPRINT\|const BB_ATTEMPT_COUNT" src/intelligence/agent-core/act_executor.rs` ≥ 6 | [ ] |
| FUNC | FUNC-002 | 链式协议：Act 完成后写入 blackboard，下次读取决定下一步 | `grep -c "blackboard.*write\|blackboard.*read" src/intelligence/agent-core/act_executor.rs` ≥ 2 | [ ] |
| FUNC | FUNC-003 | micro-reflect：工具失败时触发简短反思 | `grep -c "micro_reflect\|reflect.*error" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| FUNC | FUNC-004 | 重试规则：相同 fingerprint → 不重复；修正参数 → 可重试一次；两次失败 → StopAndHandoff | `grep -c "fingerprint\|attempt_count\|StopAndHandoff" src/intelligence/agent-core/act_executor.rs` ≥ 3 | [ ] |
| CONST | CONST-001 | `AgentLoop::act()` 调用 `ActExecutor` | `grep -c "ActExecutor\|act_executor" src/intelligence/agent-core/agent_loop.rs` ≥ 1 | [ ] |
| CONST | CONST-002 | `is_act_toolcall_v1_enabled()` 在 `prompts/mod.rs` 中定义 | `grep -c "fn is_act_toolcall_v1_enabled" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| CONST | CONST-003 | feature-gate 关闭时回到旧执行路径 | `grep -c "is_act_toolcall_v1_enabled\|legacy\|old.*act" src/intelligence/agent-core/agent_loop.rs` ≥ 1 | [ ] |
| CONST | CONST-004 | `HAJIMI_ACT_TOOLCALL_V1_ENABLED` 默认值 `true` | `grep -c "HAJIMI_ACT_TOOLCALL_V1_ENABLED" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| NEG | NEG-001 | 相同 tool + 参数不被重复调用 | `grep -c "fingerprint.*match\|already.*tried" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| NEG | NEG-002 | 两次同类失败后强制 StopAndHandoff | `grep -c "attempt_count.*>= 2\|StopAndHandoff" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |
| NEG | NEG-003 | 编译无 error | `cargo check -p intelligence-agent-core` 退出码 0 | [ ] |
| NEG | NEG-004 | 现有测试不破坏 | `cargo test -p intelligence-agent-core --lib` 107 passed | [ ] |
| UX | UX-001 | blackboard const keys 有文档注释 | `grep -c "///.*BB_" src/intelligence/agent-core/act_executor.rs` ≥ 3 | [ ] |
| UX | UX-002 | 重试规则有注释说明 | `grep -c "//.*retry\|//.*fingerprint\|//.*attempt" src/intelligence/agent-core/act_executor.rs` ≥ 2 | [ ] |
| E2E | E2E-001 | workspace 编译通过 | `cargo check --workspace` 退出码 0 | [ ] |
| High | HIGH-001 | High-risk 调用等待 governance 审批 | `grep -c "governance\|await.*approval" src/intelligence/agent-core/act_executor.rs` ≥ 1 | [ ] |

---

## 【模块3-B】地狱红线（10项）
1. 零占位符违规 → 返工
2. 验证造假（声称已验证但无命令输出） → 返工
3. 编译 / 测试失败 → 返工
4. 假实现 / mock 成功 → 返工
5. 架构约束违反（删除旧 act 逻辑） → 返工
6. 新增 warning 未申报 → 返工
7. 范围失控 → 返工
8. Git 历史不完整 → 返工
9. 复杂度超标且未声明 `DEBT-COMPLEXITY` → 返工
10. 探索任务伪装确定性完成 → 返工

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | 链式协议工作吗？重试规则正确吗？ | [ ] | CF-B14-001 | |
| 约束与回归用例（RG） | feature-gate 关闭时回到旧路径吗？ | [ ] | RG-B14-001 | |
| 负面路径用例（NG） | 相同参数不重复调用吗？两次失败后 handoff 吗？ | [ ] | NG-B14-001 | |
| 用户体验用例（UX） | blackboard keys 有文档吗？重试规则注释清晰吗？ | [ ] | UX-B14-001 | |
| 端到端关键路径（E2E） | `cargo test --lib` 全通过吗？ | [ ] | E2E-B14-001 | |
| 高风险场景（High） | High-risk 调用等待 governance 吗？ | [ ] | High-B14-001 | |
| 字段完整性 | 每条用例前置/预期/实际/风险等级 | [ ] | — | |
| 需求映射 | 用例映射到 Day 14 任务清单 | [ ] | — | |
| 自测执行 | 完整跑过一轮自测 | [ ] | — | |
| 范围边界与债务 | 未覆盖项明确标注 | [ ] | — | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## ✅ 工单 B-14/14 完成并提交

### 提交信息
- Commit: `feat(intelligence/agent-core): implement tool chain protocol, retry rules, and Act feature-gate`
- 分支: `feature/AGENT-PROMPT-CORE-001-phase5`
- 变更文件:
  - `src/intelligence/agent-core/act_executor.rs`
  - `src/intelligence/agent-core/agent_loop.rs`
  - `src/intelligence/agent-core/prompts/mod.rs`

### 本轮目标与实际结果
- 目标: 多步链式协议 + 重试规则 + micro-reflect + AgentLoop 集成 + feature-gate
- 实际完成: 全部 3 个文件修改完成，16 项刀刃表通过
- 未完成/不在范围: 无

### 关键决策记录
- DECISION-001: `is_act_toolcall_v1_enabled()` 放在 `prompts/mod.rs` —— 复用已有 feature-gate 模式，保持一致性
- DECISION-002: fingerprint 使用 tool_name + parameters 序列化哈希 —— 避免引入复杂依赖，保持零外部依赖
- DECISION-003: micro-reflect 仅分析当前工具错误，不进入完整 Reflect 循环 —— 避免循环依赖，保持增量反思

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
- 关键函数: `ActExecutor::execute_chain`、`RetryPolicy::should_retry`
- 是否存在复杂度例外: 无
- 若有，说明: N/A

### 债务声明
- DEBT-COMPLEXITY-B14-001: 无（未触发）
- DEBT-TEST-B14-001: 无

### 风险与回滚点
- 主要风险: fingerprint 计算方式与现有 hash 冲突
- 回滚方式: `git revert <commit>` 或改用更稳健的序列化方案
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | `AgentLoop::act()` 接口与 `ActExecutor` 不匹配 | 暂停实现，调整调用签名或添加适配层 | 延期 |
| QUALITY-001 | 自动化闸门连续 2 次不通过 | 停止堆代码，先修复质量 | 返工 |
| COMPLEXITY-001 | 链式协议单函数 >70 行且连续 2 次返工 | 允许带 `DEBT-COMPLEXITY` 交付 | 记录债务 |
| MEM-001 | fingerprint 计算方式与现有 hash 冲突 | 改用更稳健的序列化方案 | 延期 |
| PERF-001 | 链式协议引入循环依赖 | 提取为独立 coordination 模块 | 返工 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **AGENT-PROMPT-CORE-001 Phase 5 Day 14** 通用高压任务！

### 技术背景
Day 13 已完成 `Act DTO`、`ActExecutor` 骨架和工具调用执行。Phase 5 最后一步：实现多步链式协议（blackboard 状态传递）、重试规则（fingerprint 去重 + 修正重试 + 两次失败 handoff）、micro-reflect（工具失败简短反思），修改 `AgentLoop::act()` 调用 `ActExecutor`，并添加 `HAJIMI_ACT_TOOLCALL_V1_ENABLED` feature-gate。

### 关键约束
- 6 个 blackboard 链式 keys：`BB_NEXT_TOOL`, `BB_LAST_TOOL`, `BB_LAST_TOOL_RESULT`, `BB_LAST_ERROR`, `BB_FAILED_TOOL_FINGERPRINT`, `BB_ATTEMPT_COUNT`
- 链式协议：Act 完成后将结果写入 blackboard；下次 Act 读取 blackboard 决定下一步
- micro-reflect：工具失败时触发简短反思（仅分析当前工具错误，不进入完整 Reflect 循环）
- 重试规则：
  - 相同 tool + 相同参数 fingerprint → 不重复调用
  - 修正参数 → 可重试一次
  - 两次同类失败 → 强制 `StopAndHandoff`
- `AgentLoop::act()` 调用 `ActExecutor` 替代原有的硬编码执行逻辑
- feature-gate 函数 `is_act_toolcall_v1_enabled()` 放在 `prompts/mod.rs`
- 默认值 `true`，环境变量 `"false"` 时关闭，回到旧执行路径
- High-risk 调用等待 governance 审批
- 零 `unsafe`，零业务逻辑 `unwrap()`
- 现有 107 个单元测试必须全部通过

### 质量红线
- 10 项地狱红线生效
- 刀刃表 16 项必须命令化验证
- 禁止使用绝对行数限制

### 工单并行矩阵
- B-14/14 Engineer：多步链式协议 + 重试规则 + Feature-Gate

### 验收铁律
- `cargo check -p intelligence-agent-core` 0 errors
- `cargo test -p intelligence-agent-core --lib` 全部通过
- `$env:HAJIMI_ACT_TOOLCALL_V1_ENABLED="false"; cargo test -p intelligence-agent-core --lib` 全部通过
- `grep "BB_NEXT_TOOL" src/intelligence/agent-core/act_executor.rs` ≥ 1
- `grep "fingerprint" src/intelligence/agent-core/act_executor.rs` ≥ 1
- `grep "ActExecutor" src/intelligence/agent-core/agent_loop.rs` ≥ 1
- `grep "is_act_toolcall_v1_enabled" src/intelligence/agent-core/prompts/mod.rs` ≥ 1
- `grep "governance" src/intelligence/agent-core/act_executor.rs` ≥ 1
- `cargo check --workspace` 0 errors

### 收卷要求
- 必须附自动化质量检查摘要
- 必须附刀刃表摘要
- 必须诚实声明债务

Ouroboros 闭环启动，**AGENT-PROMPT-CORE-001 Day 14**，执行！ ☝️🐍♾️🔥

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
