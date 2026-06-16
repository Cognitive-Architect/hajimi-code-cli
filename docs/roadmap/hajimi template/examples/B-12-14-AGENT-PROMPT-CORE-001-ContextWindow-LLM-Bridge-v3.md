# 派单示例：B-12/14 — ContextWindowManager LLM Bridge 集成 + Feature-Gate（v3.0 格式）

> **本示例基于 ID-59 v3.0 通用增强版模板重写**
> **任务来源**：AGENT-PROMPT-CORE-001 Phase 4 Day 12
> **原始文档**：B-12-14-AGENT-PROMPT-CORE-001-ContextWindow-LLM-Bridge.md（旧模板）

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：Planner/Reflector LLM Bridge 集成 ContextWindowManager + feature-gate
- **轰炸目标**：修改 `llm/bridge.rs` 让 Planner/Reflector Bridge 使用 `ContextWindowManager::assemble`，添加 `HAJIMI_CONTEXT_WINDOW_ENABLED` feature-gate
- **任务性质**：功能开发 + 集成 + feature-gate
- **输入基线**：完整技术背景（见模块2）
- **输出要求**：可执行产出 + 自动化质量闸门全通过 + 显式债务声明 + 结构化收卷
- **通用铁律**：
  1. **数据诚实**：所有测试数、warning 数必须来自真实 `cargo test` / `cargo clippy` 输出
  2. **零占位符**：禁止写"参考Day 11""见上文"，必须给出完整路径 + SHA
  3. **自动化优先**：刀刃表 16 项必须提供可执行验证命令
  4. **最小必要复杂度**：单函数 >50 行需解释理由，不为压行数硬拆函数
  5. **债务透明化**：如触发复杂度熔断必须声明 `DEBT-COMPLEXITY`

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 模块/文件/函数范围 | `src/intelligence/agent-core/llm/bridge.rs:50-120`<br>`src/intelligence/agent-core/prompts/mod.rs` | 必须 |
| 现状基线 | Day 11 已完成 `ContextWindowManager` 的 `assemble` 和 `compact_block` | `grep -n "fn chat_and_collect" src/intelligence/agent-core/llm/bridge.rs` | 必须 |
| 目标结果 | 1. `PlannerLlmBridge::chat_and_collect` 调用 `ContextWindowManager::assemble`<br>2. `ReflectorLlmBridge::chat_and_collect` 同上<br>3. 添加 `is_context_window_enabled()` feature-gate 函数<br>4. feature-gate 关闭时 fallback 到简单 2-message 路径 | `cargo test -p intelligence-agent-core --lib` 全部通过 | 必须 |
| 技术约束 | 1. feature-gate 关闭时回到简单 2-message 路径<br>2. assemble 输出注入到 `stream_chat_with_context` 的 messages 参数<br>3. 单次 LLM 调用控制在 8K tokens 以内<br>4. P0 blocks 从不被静默省略 | 文字展开 | 必须 |
| 风险边界 | 禁止删除旧 `chat_and_collect` 逻辑（作为 fallback 保留） | `grep "legacy.*path\|simple.*path" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | 必须 |
| 测试基线 | 当前编译 / 测试状态 | `cargo check -p intelligence-agent-core` 0 errors<br>`cargo test -p intelligence-agent-core --lib` 107 passed | 必须 |
| 文档同步要求 | 无新增公开文档 | N/A | 按需 |
| 历史债务 / 相关缺陷 | 无 | N/A | 按需 |

### 探索补充栏
本任务为**已知解实现**，无需探索补充栏。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息
- **工单编号**：B-12/14
- **角色**：Engineer
- **目标**：Planner/Reflector LLM Bridge 集成 ContextWindowManager + feature-gate
- **输入**：引用输入基线中"目标范围"与"现状基线"所有项
- **依赖关系**：依赖 Day 11 完成（无并行依赖）

### 2）输出交付物
- **变更文件**：
  - `src/intelligence/agent-core/llm/bridge.rs`
  - `src/intelligence/agent-core/prompts/mod.rs`
- **核心修改点**：
  - `PlannerLlmBridge::chat_and_collect` 集成 `assemble`
  - `ReflectorLlmBridge::chat_and_collect` 集成 `assemble`
  - 添加 `is_context_window_enabled()` feature-gate 函数
  - 实现 feature-gate 关闭时的 fallback 逻辑
- **必须包含**：
  - `grep -c "ContextWindowManager\|assemble"` ≥ 1
  - `grep -c "is_context_window_enabled"` ≥ 1
  - `grep -c "simple 2-message\|legacy.*path"` ≥ 1
  - `grep -c "HAJIMI_CONTEXT_WINDOW_ENABLED"` ≥ 1
- **禁止包含**：`rm`、`unsafe`、删除旧 `chat_and_collect` 逻辑
- **交付证明**：`cargo test -p intelligence-agent-core --lib` 全部通过 + 刀刃表 16 项命令输出

### 3）规模与复杂度观察
- **推荐目标**：单函数保持单一职责，bridge 集成逻辑 ≤40 行
- **复杂度说明**：两个 bridge 需分别集成 `assemble`，feature-gate 分支需清晰，预计单函数 30-50 行。如超过 50 行将声明 `DEBT-COMPLEXITY-B12-001`
- **禁止行为**：为压行数硬拆函数、添加无意义中间层

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 编译通过 | `cargo check -p intelligence-agent-core` | 返工 |
| FMT | 格式检查通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p intelligence-agent-core -- -D warnings` | 返工或声明债务 |
| TEST | 单元测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工 |
| ARCH | 不违反分层与接口约束 | `grep "legacy.*path\|simple.*path" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | 返工 |
| REAL | 禁止假实现 | 所有测试真实执行，无 `#[ignore]` | 返工 |
| DOC | 无需新增文档 | N/A | - |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `PlannerLlmBridge::chat_and_collect` 调用 `ContextWindowManager::assemble` | `grep -c "assemble" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| FUNC | FUNC-002 | `ReflectorLlmBridge::chat_and_collect` 调用 `ContextWindowManager::assemble` | `grep -c "assemble" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| FUNC | FUNC-003 | `is_context_window_enabled()` 在 `prompts/mod.rs` 中定义 | `grep -c "fn is_context_window_enabled" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| FUNC | FUNC-004 | feature-gate 关闭时使用简单 2-message 路径 | `grep -c "is_context_window_enabled\|legacy\|simple" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| CONST | CONST-001 | `HAJIMI_CONTEXT_WINDOW_ENABLED` 默认值 `true` | `grep -c "HAJIMI_CONTEXT_WINDOW_ENABLED" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| CONST | CONST-002 | assemble 输出正确注入 `stream_chat_with_context` 的 `messages` | `grep -c "stream_chat_with_context\|messages" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| CONST | CONST-003 | P0 blocks 从不被静默省略（assemble 返回 Err 时 fallback） | `grep -c "Err.*Overflow\|fallback" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| CONST | CONST-004 | 单次 LLM 调用控制在 8K tokens 以内 | `grep -c "8000\|8192\|max_tokens" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| NEG | NEG-001 | feature-gate 关闭测试：`$env:HAJIMI_CONTEXT_WINDOW_ENABLED="false"` 后测试通过 | 手动执行并验证 | [ ] |
| NEG | NEG-002 | assemble 失败时 fallback 到简单 2-message | `grep -c "match.*assemble\|if let.*assemble\|Err" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| NEG | NEG-003 | 编译无 error | `cargo check -p intelligence-agent-core` 退出码 0 | [ ] |
| NEG | NEG-004 | 现有测试不破坏 | `cargo test -p intelligence-agent-core --lib` 107 passed | [ ] |
| UX | UX-001 | feature-gate 函数有 rustdoc | `grep -c "///.*context_window" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| UX | UX-002 | assemble 集成点有注释说明 | `grep -c "//.*assemble\|//.*ContextWindow" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| E2E | E2E-001 | workspace 编译通过 | `cargo check --workspace` 退出码 0 | [ ] |
| High | HIGH-001 | 不删除旧 `chat_and_collect` 逻辑（作为 fallback 保留） | `grep -c "legacy\|simple.*path\|fallback" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |

---

## 【模块3-B】地狱红线（10项）
1. 零占位符违规 → 返工
2. 验证造假（声称已验证但无命令输出） → 返工
3. 编译 / 测试失败 → 返工
4. 假实现 / mock 成功 → 返工
5. 架构约束违反（删除旧 chat_and_collect 逻辑） → 返工
6. 新增 warning 未申报 → 返工
7. 范围失控 → 返工
8. Git 历史不完整 → 返工
9. 复杂度超标且未声明 `DEBT-COMPLEXITY` → 返工
10. 探索任务伪装确定性完成 → 返工

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | Planner/Reflector Bridge 都集成 assemble 了吗？ | [ ] | CF-B12-001 | |
| 约束与回归用例（RG） | feature-gate 关闭时回到 2-message 路径吗？ | [ ] | RG-B12-001 | |
| 负面路径用例（NG） | assemble 失败 fallback 了吗？feature-gate 关闭测试了吗？ | [ ] | NG-B12-001 | |
| 用户体验用例（UX） | 集成点注释清晰吗？ | [ ] | UX-B12-001 | |
| 端到端关键路径（E2E） | `cargo test --lib` 通过吗？ | [ ] | E2E-B12-001 | |
| 高风险场景（High） | P0 blocks 不会被静默省略吗？ | [ ] | High-B12-001 | |
| 字段完整性 | 每条用例前置/预期/实际/风险等级 | [ ] | — | |
| 需求映射 | 用例映射到 Day 12 任务清单 | [ ] | — | |
| 自测执行 | 完整跑过一轮自测 | [ ] | — | |
| 范围边界与债务 | 未覆盖项明确标注 | [ ] | — | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## ✅ 工单 B-12/14 完成并提交

### 提交信息
- Commit: `feat(intelligence/agent-core): integrate ContextWindowManager into LLM Bridges with feature-gate`
- 分支: `feature/AGENT-PROMPT-CORE-001-phase4`
- 变更文件:
  - `src/intelligence/agent-core/llm/bridge.rs`
  - `src/intelligence/agent-core/prompts/mod.rs`

### 本轮目标与实际结果
- 目标: Planner/Reflector LLM Bridge 集成 ContextWindowManager + feature-gate
- 实际完成: 全部 2 个文件修改完成，16 项刀刃表通过
- 未完成/不在范围: 无

### 关键决策记录
- DECISION-001: `is_context_window_enabled()` 放在 `prompts/mod.rs` —— 复用已有 feature-gate 模式，保持一致性
- DECISION-002: assemble 失败时 fallback 到简单 2-message —— 保证系统可用性，避免因 token 预算问题导致 LLM 调用完全失败

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
- 关键函数: `PlannerLlmBridge::chat_and_collect`、`ReflectorLlmBridge::chat_and_collect`
- 是否存在复杂度例外: 无
- 若有，说明: N/A

### 债务声明
- DEBT-COMPLEXITY-B12-001: 无（未触发）
- DEBT-TEST-B12-001: 无

### 风险与回滚点
- 主要风险: `stream_chat_with_context` 的 `messages` 参数类型与 `AssembledContext` 不匹配
- 回滚方式: `git revert <commit>` 或删除 assemble 集成逻辑
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | `stream_chat_with_context` 的 `messages` 参数类型与 `AssembledContext` 不匹配 | 暂停实现，添加适配转换函数 | 延期 |
| QUALITY-001 | 自动化闸门连续 2 次不通过 | 停止堆代码，先修复质量 | 返工 |
| COMPLEXITY-001 | bridge 集成单函数 >50 行且连续 2 次返工 | 允许带 `DEBT-COMPLEXITY` 交付 | 记录债务 |
| PERF-001 | assemble 调用引入显著延迟 | 缓存 AssembledContext 结果 | 返工 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **AGENT-PROMPT-CORE-001 Phase 4 Day 12** 通用高压任务！

### 技术背景
Day 10-11 已完成 `ContextWindowManager` 核心类型、`assemble` 方法、`compact_block`、`estimate_tokens` 和 `MemoryRetriever` 集成。Phase 4 最后一步：让 `PlannerLlmBridge` 和 `ReflectorLlmBridge` 在调用 LLM 前使用 `ContextWindowManager::assemble` 组装上下文，添加 `HAJIMI_CONTEXT_WINDOW_ENABLED` feature-gate。

### 关键约束
- `PlannerLlmBridge::chat_and_collect` 在调用 `stream_chat_with_context` 前，用 `ContextWindowManager::assemble` 组装 system prompt + user prompt 为 `Vec<ChatMessage>`
- `ReflectorLlmBridge::chat_and_collect` 同上
- feature-gate 函数 `is_context_window_enabled()` 放在 `prompts/mod.rs`，复用已有模式
- 默认值 `true`，环境变量 `"false"` 时关闭
- feature-gate 关闭时回到简单 2-message 路径（system + user）
- `assemble` 返回 `Err` 时 fallback 到简单 2-message（不 panic）
- 单次 LLM 调用控制在 8K tokens 以内（通过 `ContextWindowManager` 的 max_tokens 配置）
- P0 blocks 从不被静默省略
- 现有 107 个单元测试必须全部通过

### 质量红线
- 10 项地狱红线生效
- 刀刃表 16 项必须命令化验证
- 禁止使用绝对行数限制

### 工单并行矩阵
- B-12/14 Engineer：ContextWindowManager LLM Bridge 集成 + Feature-Gate

### 验收铁律
- `cargo check -p intelligence-agent-core` 0 errors
- `cargo test -p intelligence-agent-core --lib` 全部通过
- `$env:HAJIMI_CONTEXT_WINDOW_ENABLED="false"; cargo test -p intelligence-agent-core --lib` 全部通过
- `grep "ContextWindowManager" src/intelligence/agent-core/llm/bridge.rs` ≥ 1
- `grep "assemble" src/intelligence/agent-core/llm/bridge.rs` ≥ 1
- `grep "is_context_window_enabled" src/intelligence/agent-core/prompts/mod.rs` ≥ 1
- `grep "HAJIMI_CONTEXT_WINDOW_ENABLED" src/intelligence/agent-core/prompts/mod.rs` ≥ 1
- `grep "stream_chat_with_context" src/intelligence/agent-core/llm/bridge.rs` ≥ 1
- `cargo check --workspace` 0 errors

### 收卷要求
- 必须附自动化质量检查摘要
- 必须附刀刃表摘要
- 必须诚实声明债务

Ouroboros 闭环启动，**AGENT-PROMPT-CORE-001 Day 12**，执行！ ☝️🐍♾️🔥

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
