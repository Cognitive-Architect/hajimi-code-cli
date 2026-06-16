# 派单示例：B-10/14 — ContextWindowManager 核心类型 + Assemble 逻辑（v3.0 格式）

> **本示例基于 ID-59 v3.0 通用增强版模板重写**
> **任务来源**：AGENT-PROMPT-CORE-001 Phase 4 Day 10
> **原始文档**：B-10-14-AGENT-PROMPT-CORE-001-ContextWindowManager-Core.md（旧模板）

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：ContextWindowManager 核心类型定义 + assemble 方法实现 + 模块注册
- **轰炸目标**：新建 `context_window_manager.rs`，定义 `ContextBlock`/`ContextPriority`/`ContentType`/`TokenAccount`/`OmittedBlock`，实现 `assemble` 方法，注册到 `lib.rs`
- **任务性质**：功能开发
- **输入基线**：完整技术背景（见模块2）
- **输出要求**：可执行产出 + 自动化质量闸门全通过 + 显式债务声明 + 结构化收卷
- **通用铁律**：
  1. **数据诚实**：所有测试数、warning 数必须来自真实 `cargo test` / `cargo clippy` 输出
  2. **零占位符**：禁止写"参考Day 9""见上文"，必须给出完整路径 + SHA
  3. **自动化优先**：刀刃表 16 项必须提供可执行验证命令
  4. **最小必要复杂度**：单函数 >50 行需解释理由，不为压行数硬拆函数
  5. **债务透明化**：如触发复杂度熔断必须声明 `DEBT-COMPLEXITY`

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 模块/文件/函数范围 | `src/intelligence/agent-core/context_window_manager.rs`（新建）<br>`src/intelligence/agent-core/lib.rs`（追加模块注册） | 必须 |
| 现状基线 | Phase 3 已完成 Reflector + Stop-Loss，需实现 Token 预算管理 | `grep -n "pub mod" src/intelligence/agent-core/lib.rs` | 必须 |
| 目标结果 | 1. 定义 5 个核心类型（ContextBlock/ContextPriority/ContentType/TokenAccount/OmittedBlock）<br>2. 实现 `assemble` 方法<br>3. 注册到 `lib.rs`<br>4. P0 overflow 返回 `Err` | `cargo check -p intelligence-agent-core` 0 errors | 必须 |
| 技术约束 | 1. `ContextWindowManager` 不依赖 LLM 客户端（仅接收 blocks）<br>2. P0 block overflow 返回 `Err(ContextError::Overflow(P0))` 而非静默省略<br>3. P1 尝试 compact，P2-P4 按优先级省略<br>4. `AssembledContext` 包含 included + omitted + total_tokens | 文字展开 | 必须 |
| 风险边界 | 禁止引入 `LlmClient`/`network`/`tokio` 依赖 | `grep "LlmClient\|network\|tokio" src/intelligence/agent-core/context_window_manager.rs` = 0 | 必须 |
| 测试基线 | 当前编译 / 测试状态 | `cargo check -p intelligence-agent-core` 0 errors<br>`cargo test -p intelligence-agent-core --lib` 107 passed | 必须 |
| 文档同步要求 | 无新增公开文档 | N/A | 按需 |
| 历史债务 / 相关缺陷 | 无 | N/A | 按需 |

### 探索补充栏
本任务为**已知解实现**，无需探索补充栏。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息
- **工单编号**：B-10/14
- **角色**：Engineer
- **目标**：ContextWindowManager 核心类型定义 + assemble 方法实现 + 模块注册
- **输入**：引用输入基线中"目标范围"与"现状基线"所有项
- **依赖关系**：依赖 Phase 3 完成（无并行依赖）

### 2）输出交付物
- **变更文件**：
  - `src/intelligence/agent-core/context_window_manager.rs`（新建）
  - `src/intelligence/agent-core/lib.rs`（追加模块注册）
- **核心修改点**：
  - 定义 `ContextBlock`/`ContextPriority`/`ContentType`/`TokenAccount`/`OmittedBlock`
  - 实现 `assemble` 方法（P0 优先，P1 compact，P2-P4 省略）
  - 定义 `ContextError`（Overflow 变体）
  - 注册模块到 `lib.rs`
- **必须包含**：
  - `grep -c "ContextBlock\|ContextPriority\|ContentType\|TokenAccount\|OmittedBlock"` ≥ 5
  - `grep -c "fn assemble"` ≥ 1
  - `grep -c "ContextError::Overflow"` ≥ 1
  - `grep -c "pub mod context_window_manager"` ≥ 1
- **禁止包含**：`rm`、`unsafe`、`panic!`、引入 LlmClient 依赖
- **交付证明**：`cargo check -p intelligence-agent-core` 0 errors + 刀刃表 16 项命令输出

### 3）规模与复杂度观察
- **推荐目标**：单函数保持单一职责，核心逻辑函数 ≤60 行
- **复杂度说明**：`assemble` 方法需处理 5 级优先级 + 4 种 ContentType，预计单函数 50-70 行。如超过 70 行将声明 `DEBT-COMPLEXITY-B10-001`
- **禁止行为**：为压行数硬拆函数、添加无意义中间层

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 编译通过 | `cargo check -p intelligence-agent-core` | 返工 |
| FMT | 格式检查通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p intelligence-agent-core -- -D warnings` | 返工或声明债务 |
| TEST | 单元测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工 |
| ARCH | 不违反分层与接口约束 | `grep "LlmClient\|network\|tokio" src/intelligence/agent-core/context_window_manager.rs` = 0 | 返工 |
| REAL | 禁止假实现 | 所有测试真实执行，无 `#[ignore]` | 返工 |
| DOC | 无需新增文档 | N/A | - |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `ContextBlock` 包含 6 个字段 | `grep -c "token_estimate\|truncatable" src/intelligence/agent-core/context_window_manager.rs` ≥ 2 | [ ] |
| FUNC | FUNC-002 | `ContextPriority` enum 为 P0-P4 五级 | `grep -c "P0,\|P1,\|P2,\|P3,\|P4" src/intelligence/agent-core/context_window_manager.rs` ≥ 5 | [ ] |
| FUNC | FUNC-003 | `ContentType` enum 覆盖 4 种类型 | `grep -c "SystemPrompt,\|Json,\|Text,\|Markdown" src/intelligence/agent-core/context_window_manager.rs` ≥ 4 | [ ] |
| FUNC | FUNC-004 | `assemble` 方法返回 `Result<AssembledContext, ContextError>` | `grep -c "fn assemble" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| CONST | CONST-001 | P0 overflow 返回 `Err(ContextError::Overflow(P0))` | `grep -c "Overflow.*P0\|ContextError::Overflow" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| CONST | CONST-002 | P1 尝试 compact，P2-P4 按优先级省略 | `grep -c "compact\|P1\|P2\|P3\|P4" src/intelligence/agent-core/context_window_manager.rs` ≥ 5 | [ ] |
| CONST | CONST-003 | `AssembledContext` 包含 3 个字段 | `grep -c "struct AssembledContext" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| CONST | CONST-004 | `lib.rs` 注册模块 | `grep -c "pub mod context_window_manager" src/intelligence/agent-core/lib.rs` ≥ 1 | [ ] |
| NEG | NEG-001 | P0 overflow 不静默省略 | `grep -c "return Err\|Overflow" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| NEG | NEG-002 | 空 blocks 输入返回空结果（不 panic） | `grep -c "fn test_empty\|fn test_" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| NEG | NEG-003 | 编译无 error | `cargo check -p intelligence-agent-core` 退出码 0 | [ ] |
| NEG | NEG-004 | 无 `unsafe` | `grep -c "unsafe" src/intelligence/agent-core/context_window_manager.rs` = 0 | [ ] |
| UX | UX-001 | 每个 pub 类型有 rustdoc | `grep -c "^///" src/intelligence/agent-core/context_window_manager.rs` ≥ 10 | [ ] |
| UX | UX-002 | `assemble` 方法有详细注释 | `grep -c "///.*assemble" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| E2E | E2E-001 | workspace 编译通过 | `cargo check --workspace` 退出码 0 | [ ] |
| High | HIGH-001 | 零外部依赖（LlmClient/网络） | `grep -c "LlmClient\|network\|tokio" src/intelligence/agent-core/context_window_manager.rs` = 0 | [ ] |

---

## 【模块3-B】地狱红线（10项）
1. 零占位符违规 → 返工
2. 验证造假（声称已验证但无命令输出） → 返工
3. 编译 / 测试失败 → 返工
4. 假实现 / mock 成功 → 返工
5. 架构约束违反（引入 LlmClient 依赖） → 返工
6. 新增 warning 未申报 → 返工
7. 范围失控 → 返工
8. Git 历史不完整 → 返工
9. 复杂度超标且未声明 `DEBT-COMPLEXITY` → 返工
10. 探索任务伪装确定性完成 → 返工

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | ContextBlock/ContextPriority/ContentType 定义完整吗？ | [ ] | CF-B10-001 | |
| 约束与回归用例（RG） | assemble 逻辑正确吗？P0 overflow 返回 Err 吗？ | [ ] | RG-B10-001 | |
| 负面路径用例（NG） | 空输入处理了吗？无 unsafe 吗？ | [ ] | NG-B10-001 | |
| 用户体验用例（UX） | rustdoc 覆盖每个 pub 类型吗？ | [ ] | UX-B10-001 | |
| 端到端关键路径（E2E） | `cargo check --workspace` 通过吗？ | [ ] | E2E-B10-001 | |
| 高风险场景（High） | 零外部依赖（LlmClient/网络）吗？ | [ ] | High-B10-001 | |
| 字段完整性 | 每条用例前置/预期/实际/风险等级 | [ ] | — | |
| 需求映射 | 用例映射到 Day 10 任务清单 | [ ] | — | |
| 自测执行 | 完整跑过一轮自测 | [ ] | — | |
| 范围边界与债务 | 未覆盖项明确标注 | [ ] | — | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## ✅ 工单 B-10/14 完成并提交

### 提交信息
- Commit: `feat(intelligence/agent-core): add ContextWindowManager core types and assemble logic`
- 分支: `feature/AGENT-PROMPT-CORE-001-phase4`
- 变更文件:
  - `src/intelligence/agent-core/context_window_manager.rs`（新建）
  - `src/intelligence/agent-core/lib.rs`（追加模块注册）

### 本轮目标与实际结果
- 目标: ContextWindowManager 核心类型定义 + assemble 方法实现 + 模块注册
- 实际完成: 全部 2 个文件修改完成，16 项刀刃表通过
- 未完成/不在范围: compact_block 仅骨架（Day 11 填充实现）

### 关键决策记录
- DECISION-001: `compact_block` 仅返回 `Option<ContextBlock>` 骨架 —— 避免 Day 10 过度实现，保持 Phase 4 增量交付
- DECISION-002: `ContextError` 定义在同一文件内 —— 避免引入新模块，保持单一文件职责清晰

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
- 关键函数: `ContextWindowManager::assemble`
- 是否存在复杂度例外: 无
- 若有，说明: N/A

### 债务声明
- DEBT-COMPLEXITY-B10-001: 无（未触发）
- DEBT-TEST-B10-001: 无
- DEBT-SCOPE-B10-001: `compact_block` 仅骨架，Day 11 填充实现

### 风险与回滚点
- 主要风险: `AssembledContext` 过大导致栈溢出
- 回滚方式: `git revert <commit>` 或删除 `context_window_manager.rs`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | `ContextError` 与现有 error 类型冲突 | 暂停实现，重命名或封装 | 返工 |
| QUALITY-001 | 自动化闸门连续 2 次不通过 | 停止堆代码，先修复质量 | 返工 |
| COMPLEXITY-001 | `assemble` 单函数 >70 行且连续 2 次返工 | 允许带 `DEBT-COMPLEXITY` 交付 | 记录债务 |
| SCOPE-001 | 误实现 `compact_block` 完整逻辑 | 回退到骨架，返回 `Option` | 返工 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **AGENT-PROMPT-CORE-001 Phase 4 Day 10** 通用高压任务！

### 技术背景
Phase 3 已完成 Reflector + Stop-Loss。Phase 4 目标是实现 Token 预算管理和上下文优先级组装。需新建 `context_window_manager.rs`，定义 `ContextBlock`、`ContextPriority`、`ContentType`、`TokenAccount`、`OmittedBlock`，实现 `assemble` 方法，并注册到 `lib.rs`。

### 关键约束
- `ContextBlock` 字段：name, priority, content_type, content, token_estimate, truncatable
- `ContextPriority` 5 级：P0, P1, P2, P3, P4
- `ContentType` 4 变体：SystemPrompt, Json, Text, Markdown
- `assemble` 方法：按优先级顺序处理 blocks，P0 无法容纳时返回 `Err(ContextError::Overflow(P0))`
- P1 尝试 `compact_block`（Day 11 实现），P2-P4 直接省略
- `AssembledContext` 包含：blocks（included）、omitted、total_tokens
- `ContextWindowManager` 不依赖 `LlmClient`、`network` 或 `tokio`（纯同步逻辑）
- `compact_block` 仅骨架（返回 `Option<ContextBlock>`），标记 `TODO: Phase 4 Day 11`
- 零 `unsafe`，零业务逻辑 `unwrap()`

### 质量红线
- 10 项地狱红线生效
- 刀刃表 16 项必须命令化验证
- 禁止使用绝对行数限制

### 工单并行矩阵
- B-10/14 Engineer：ContextWindowManager 核心类型 + Assemble 逻辑

### 验收铁律
- `cargo check -p intelligence-agent-core` 0 errors
- `grep "ContextBlock" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "ContextPriority" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "ContentType" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "fn assemble" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "ContextError::Overflow" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "pub mod context_window_manager" src/intelligence/agent-core/lib.rs` ≥ 1
- `grep -c "unsafe" src/intelligence/agent-core/context_window_manager.rs` = 0
- `cargo check --workspace` 0 errors

### 收卷要求
- 必须附自动化质量检查摘要
- 必须附刀刃表摘要
- 必须诚实声明债务

Ouroboros 闭环启动，**AGENT-PROMPT-CORE-001 Day 10**，执行！ ☝️🐍♾️🔥

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
