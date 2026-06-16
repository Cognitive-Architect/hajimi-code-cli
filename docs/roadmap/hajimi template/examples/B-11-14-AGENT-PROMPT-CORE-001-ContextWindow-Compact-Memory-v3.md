# 派单示例：B-11/14 — Block 压缩规则 + MemoryRetriever 集成（v3.0 格式）

> **本示例基于 ID-59 v3.0 通用增强版模板重写**
> **任务来源**：AGENT-PROMPT-CORE-001 Phase 4 Day 11
> **原始文档**：B-11-14-AGENT-PROMPT-CORE-001-ContextWindow-Compact-Memory.md（旧模板）

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：ContextWindowManager compact/estimate 实现 + MemoryRetriever 集成
- **轰炸目标**：修改 `context_window_manager.rs` 实现 `compact_block` 和 `estimate_tokens`；修改 `memory_retriever.rs` 注入记忆 blocks
- **任务性质**：功能开发 + 集成
- **输入基线**：完整技术背景（见模块2）
- **输出要求**：可执行产出 + 自动化质量闸门全通过 + 显式债务声明 + 结构化收卷
- **通用铁律**：
  1. **数据诚实**：所有测试数、warning 数必须来自真实 `cargo test` / `cargo clippy` 输出
  2. **零占位符**：禁止写"参考Day 10""见上文"，必须给出完整路径 + SHA
  3. **自动化优先**：刀刃表 16 项必须提供可执行验证命令
  4. **最小必要复杂度**：单函数 >50 行需解释理由，不为压行数硬拆函数
  5. **债务透明化**：如触发复杂度熔断必须声明 `DEBT-COMPLEXITY`

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 模块/文件/函数范围 | `src/intelligence/agent-core/context_window_manager.rs:185-215`<br>`src/intelligence/agent-core/memory_retriever.rs:105-135` | 必须 |
| 现状基线 | Day 10 已完成 `ContextWindowManager` 骨架和 `assemble` 方法 | `grep -n "fn assemble\|fn compact_block" src/intelligence/agent-core/context_window_manager.rs` | 必须 |
| 目标结果 | 1. 实现 `compact_block`（按 ContentType 差异化压缩）<br>2. 实现 `estimate_tokens`（启发式 fallback）<br>3. 实现 `MemoryRetriever::retrieve_for_context`<br>4. Focus/Working/Archive Memory 分别返回 P1/P2/P3 blocks | `cargo test -p intelligence-agent-core --lib` 全部通过 | 必须 |
| 技术约束 | 1. `compact_block` 按 ContentType 差异化压缩<br>2. `estimate_tokens` 优先调用 `LlmClient::count_tokens`，fallback 到启发式<br>3. Focus Memory 始终 P1，Working Memory P2 摘要，Archive Memory P3 筛选<br>4. 不修改 `ContextWindowManager` 结构体定义 | 文字展开 | 必须 |
| 风险边界 | 禁止修改 `ContextWindowManager` 结构体定义 | `grep -n "struct ContextWindowManager" src/intelligence/agent-core/context_window_manager.rs` 行号与基线一致 | 必须 |
| 测试基线 | 当前编译 / 测试状态 | `cargo check -p intelligence-agent-core` 0 errors<br>`cargo test -p intelligence-agent-core --lib` 107 passed | 必须 |
| 文档同步要求 | 无新增公开文档 | N/A | 按需 |
| 历史债务 / 相关缺陷 | 无 | N/A | 按需 |

### 探索补充栏
本任务为**已知解实现**，无需探索补充栏。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息
- **工单编号**：B-11/14
- **角色**：Engineer
- **目标**：ContextWindowManager compact/estimate 实现 + MemoryRetriever 集成
- **输入**：引用输入基线中"目标范围"与"现状基线"所有项
- **依赖关系**：依赖 Day 10 完成（无并行依赖）

### 2）输出交付物
- **变更文件**：
  - `src/intelligence/agent-core/context_window_manager.rs`
  - `src/intelligence/agent-core/memory_retriever.rs`
- **核心修改点**：
  - 实现 `compact_block`（SystemPrompt/Json/Text/Markdown 差异化压缩）
  - 实现 `estimate_tokens`（中文 0.9/char，英文 1.3/word）
  - 实现 `MemoryRetriever::retrieve_for_context`（Focus/Working/Archive → P1/P2/P3）
- **必须包含**：
  - `grep -c "fn compact_block"` ≥ 1
  - `grep -c "fn estimate_tokens"` ≥ 1
  - `grep -c "fn retrieve_for_context"` ≥ 1
  - `grep -c "ContextPriority::P1\|ContextPriority::P2\|ContextPriority::P3"` ≥ 3
- **禁止包含**：`rm`、`unsafe`、`panic!`、修改 ContextWindowManager 结构体
- **交付证明**：`cargo test -p intelligence-agent-core --lib` 全部通过 + 刀刃表 16 项命令输出

### 3）规模与复杂度观察
- **推荐目标**：单函数保持单一职责，`compact_block` 匹配分支 ≤40 行
- **复杂度说明**：`compact_block` 需处理 4 种 ContentType，`retrieve_for_context` 需处理 3 层记忆，预计单函数 30-50 行。如超过 50 行将声明 `DEBT-COMPLEXITY-B11-001`
- **禁止行为**：为压行数硬拆函数、添加无意义中间层

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 编译通过 | `cargo check -p intelligence-agent-core` | 返工 |
| FMT | 格式检查通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p intelligence-agent-core -- -D warnings` | 返工或声明债务 |
| TEST | 单元测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工 |
| ARCH | 不违反分层与接口约束 | `grep -n "struct ContextWindowManager" src/intelligence/agent-core/context_window_manager.rs` 行号与基线一致 | 返工 |
| REAL | 禁止假实现 | 所有测试真实执行，无 `#[ignore]` | 返工 |
| DOC | 无需新增文档 | N/A | - |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `compact_block` 按 ContentType 差异化压缩 | `grep -c "match.*content_type\|ContentType::" src/intelligence/agent-core/context_window_manager.rs` ≥ 3 | [ ] |
| FUNC | FUNC-002 | `estimate_tokens` 优先调用 `LlmClient::count_tokens`，fallback 启发式 | `grep -c "count_tokens\|LlmClient\|fallback" src/intelligence/agent-core/context_window_manager.rs` ≥ 2 | [ ] |
| FUNC | FUNC-003 | `MemoryRetriever::retrieve_for_context` 返回 `Vec<ContextBlock>` | `grep -c "fn retrieve_for_context" src/intelligence/agent-core/memory_retriever.rs` ≥ 1 | [ ] |
| FUNC | FUNC-004 | Focus Memory 始终作为 P1 block 返回 | `grep -c "Focus.*P1\|ContextPriority::P1" src/intelligence/agent-core/memory_retriever.rs` ≥ 1 | [ ] |
| CONST | CONST-001 | Working Memory 压缩为摘要，作为 P2 block 返回 | `grep -c "Working.*P2\|ContextPriority::P2" src/intelligence/agent-core/memory_retriever.rs` ≥ 1 | [ ] |
| CONST | CONST-002 | Archive Memory 仅返回 top-ranked 片段，作为 P3 block 返回 | `grep -c "Archive.*P3\|ContextPriority::P3" src/intelligence/agent-core/memory_retriever.rs` ≥ 1 | [ ] |
| CONST | CONST-003 | 启发式 fallback：中文 0.9/char，英文 1.3/word | `grep -c "0.9\|1.3" src/intelligence/agent-core/context_window_manager.rs` ≥ 2 | [ ] |
| CONST | CONST-004 | `ContextWindowManager` 结构体定义未修改 | `grep -n "struct ContextWindowManager" src/intelligence/agent-core/context_window_manager.rs` 行号与基线一致 | [ ] |
| NEG | NEG-001 | `LlmClient::count_tokens` 不存在时纯启发式工作 | `grep -c "estimate_tokens" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| NEG | NEG-002 | compact 失败时返回原 block（不 panic） | `grep -c "Some.*compact\|None" src/intelligence/agent-core/context_window_manager.rs` ≥ 1 | [ ] |
| NEG | NEG-003 | 编译无 error | `cargo check -p intelligence-agent-core` 退出码 0 | [ ] |
| NEG | NEG-004 | 现有测试不破坏 | `cargo test -p intelligence-agent-core --lib` 107 passed | [ ] |
| UX | UX-001 | 每个 compact 策略有注释说明 | `grep -c "//.*compact\|//.*SystemPrompt\|//.*ToolManifest" src/intelligence/agent-core/context_window_manager.rs` ≥ 3 | [ ] |
| UX | UX-002 | `retrieve_for_context` 有 rustdoc | `grep -c "///.*retrieve_for_context" src/intelligence/agent-core/memory_retriever.rs` ≥ 1 | [ ] |
| E2E | E2E-001 | workspace 编译通过 | `cargo check --workspace` 退出码 0 | [ ] |
| High | HIGH-001 | Focus Memory 即使预算紧张也始终包含 | `grep -c "Focus.*always\|P0\|P1.*force" src/intelligence/agent-core/memory_retriever.rs` ≥ 1 | [ ] |

---

## 【模块3-B】地狱红线（10项）
1. 零占位符违规 → 返工
2. 验证造假（声称已验证但无命令输出） → 返工
3. 编译 / 测试失败 → 返工
4. 假实现 / mock 成功 → 返工
5. 架构约束违反（修改 ContextWindowManager 结构体定义） → 返工
6. 新增 warning 未申报 → 返工
7. 范围失控 → 返工
8. Git 历史不完整 → 返工
9. 复杂度超标且未声明 `DEBT-COMPLEXITY` → 返工
10. 探索任务伪装确定性完成 → 返工

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | compact_block 覆盖所有 ContentType 吗？ | [ ] | CF-B11-001 | |
| 约束与回归用例（RG） | ContextWindowManager 结构体未修改吗？ | [ ] | RG-B11-001 | |
| 负面路径用例（NG） | LlmClient::count_tokens 缺失时 fallback 了吗？ | [ ] | NG-B11-001 | |
| 用户体验用例（UX） | compact 策略注释清晰吗？ | [ ] | UX-B11-001 | |
| 端到端关键路径（E2E） | `cargo test --lib` 通过吗？ | [ ] | E2E-B11-001 | |
| 高风险场景（High） | Focus Memory 始终包含吗？ | [ ] | High-B11-001 | |
| 字段完整性 | 每条用例前置/预期/实际/风险等级 | [ ] | — | |
| 需求映射 | 用例映射到 Day 11 任务清单 | [ ] | — | |
| 自测执行 | 完整跑过一轮自测 | [ ] | — | |
| 范围边界与债务 | 未覆盖项明确标注 | [ ] | — | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## ✅ 工单 B-11/14 完成并提交

### 提交信息
- Commit: `feat(intelligence/agent-core): implement block compact rules and MemoryRetriever integration`
- 分支: `feature/AGENT-PROMPT-CORE-001-phase4`
- 变更文件:
  - `src/intelligence/agent-core/context_window_manager.rs`
  - `src/intelligence/agent-core/memory_retriever.rs`

### 本轮目标与实际结果
- 目标: ContextWindowManager compact/estimate 实现 + MemoryRetriever 集成
- 实际完成: 全部 2 个文件修改完成，16 项刀刃表通过
- 未完成/不在范围: 无

### 关键决策记录
- DECISION-001: `estimate_tokens` 启发式系数硬编码（中文 0.9/char，英文 1.3/word）—— 避免引入复杂分词依赖，保持零外部依赖
- DECISION-002: Focus Memory 始终 P1 即使预算紧张 —— 符合用户体验优先原则，关键记忆不可省略

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
- 关键函数: `ContextWindowManager::compact_block`、`MemoryRetriever::retrieve_for_context`
- 是否存在复杂度例外: 无
- 若有，说明: N/A

### 债务声明
- DEBT-COMPLEXITY-B11-001: 无（未触发）
- DEBT-TEST-B11-001: 无

### 风险与回滚点
- 主要风险: 启发式 token 估算精度偏差 >30%
- 回滚方式: `git revert <commit>` 或调整启发式系数
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | `LlmClient` 无 `count_tokens` 方法 | 暂停实现，仅保留启发式 fallback | 延期 |
| QUALITY-001 | 自动化闸门连续 2 次不通过 | 停止堆代码，先修复质量 | 返工 |
| COMPLEXITY-001 | `compact_block` 单函数 >50 行且连续 2 次返工 | 允许带 `DEBT-COMPLEXITY` 交付 | 记录债务 |
| PERF-001 | `estimate_tokens` 启发式精度偏差 >30% | 调整系数或改用字符数 | 返工 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **AGENT-PROMPT-CORE-001 Phase 4 Day 11** 通用高压任务！

### 技术背景
Day 10 已完成 `ContextWindowManager` 核心类型和 `assemble` 方法。需实现 `compact_block` 和 `estimate_tokens`，并修改 `MemoryRetriever` 将记忆系统注入为 `ContextBlock`。

### 关键约束
- `compact_block` 按 `ContentType` 差异化压缩：
  - `SystemPrompt` → 精简版（保留核心指令）
  - `ToolManifest` → 截断描述，保留 name + parameters_schema
  - `Text`/`Markdown` → 保留前 N 字符摘要
- `estimate_tokens`：优先调用 `LlmClient::count_tokens`，若不存在则 fallback 到启发式（中文 0.9/char，英文 1.3/word）
- `MemoryRetriever::retrieve_for_context(goal, budget) → Vec<ContextBlock>`：
  - Focus Memory → 始终 P1 block（即使预算紧张）
  - Working Memory → 压缩为摘要，P2 block
  - Archive Memory → top-ranked 片段，P3 block
- `ContextWindowManager` 结构体定义不可修改
- 零 `unsafe`，零业务逻辑 `unwrap()`
- 现有 107 个单元测试必须全部通过

### 质量红线
- 10 项地狱红线生效
- 刀刃表 16 项必须命令化验证
- 禁止使用绝对行数限制

### 工单并行矩阵
- B-11/14 Engineer：Block 压缩规则 + MemoryRetriever 集成

### 验收铁律
- `cargo check -p intelligence-agent-core` 0 errors
- `cargo test -p intelligence-agent-core --lib` 107 passed（不破坏现有测试）
- `grep "fn compact_block" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "fn estimate_tokens" src/intelligence/agent-core/context_window_manager.rs` ≥ 1
- `grep "fn retrieve_for_context" src/intelligence/agent-core/memory_retriever.rs` ≥ 1
- `grep "ContextPriority::P1" src/intelligence/agent-core/memory_retriever.rs` ≥ 1
- `grep "ContextPriority::P2" src/intelligence/agent-core/memory_retriever.rs` ≥ 1
- `grep "ContextPriority::P3" src/intelligence/agent-core/memory_retriever.rs` ≥ 1
- `grep -c "unsafe" src/intelligence/agent-core/context_window_manager.rs src/intelligence/agent-core/memory_retriever.rs` = 0
- `cargo check --workspace` 0 errors

### 收卷要求
- 必须附自动化质量检查摘要
- 必须附刀刃表摘要
- 必须诚实声明债务

Ouroboros 闭环启动，**AGENT-PROMPT-CORE-001 Day 11**，执行！ ☝️🐍♾️🔥

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
