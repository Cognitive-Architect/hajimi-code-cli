# 派单示例：B-09/14 — Reflector Feature-Gate + 集成测试 + Blackboard 协议（v3.0 格式）

> **本示例基于 ID-59 v3.0 通用增强版模板重写**
> **任务来源**：AGENT-PROMPT-CORE-001 Phase 3 Day 9
> **原始文档**：B-09-14-AGENT-PROMPT-CORE-001-Reflector-FeatureGate-Tests.md（旧模板）

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：Reflector V1 Feature-Gate + Blackboard 协议标准化 + 单元测试
- **轰炸目标**：在 `prompts/mod.rs`、`agent_loop.rs`、`llm/bridge.rs`、`reflector_dto.rs` 中添加 `HAJIMI_REFLECTOR_V1_ENABLED` feature-gate，定义 blackboard 标准 const keys，补充 Reflector DTO 与 Stop-Loss 单元测试
- **任务性质**：功能开发 + 集成测试
- **输入基线**：完整技术背景（见模块2）
- **输出要求**：可执行产出 + 自动化质量闸门全通过 + 显式债务声明 + 结构化收卷
- **通用铁律**：
  1. **数据诚实**：所有测试数、warning 数必须来自真实 `cargo test` / `cargo clippy` 输出
  2. **零占位符**：禁止写“参考Day 8”“见上文”，必须给出完整路径 + SHA
  3. **自动化优先**：刀刃表 16 项必须提供可执行验证命令
  4. **最小必要复杂度**：单函数 >50 行需解释理由，不为压行数硬拆函数
  5. **债务透明化**：如触发复杂度熔断必须声明 `DEBT-COMPLEXITY`

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 模块/文件/函数范围 | `src/intelligence/agent-core/prompts/mod.rs:1-200`<br>`src/intelligence/agent-core/agent_loop.rs:300-450`<br>`src/intelligence/agent-core/llm/bridge.rs:50-120`<br>`src/intelligence/agent-core/reflector_dto.rs` | 必须 |
| 现状基线 | Day 8 已完成 `ReflectorCritiqueV1Dto` 定义、`ReflectorLlmBridge` 改造和 AgentLoop 路由逻辑 | `grep -n "ReflectorCritiqueV1Dto\|RecommendedAction" src/intelligence/agent-core/agent_loop.rs` | 必须 |
| 目标结果 | 1. 添加 `is_reflector_v1_enabled()` 函数<br>2. 定义 3 个 blackboard const keys<br>3. Reflector DTO + Stop-Loss 单元测试 ≥2 个<br>4. 保留 legacy Critique fallback | `cargo test -p intelligence-agent-core --lib` 全部通过 | 必须 |
| 技术约束 | 1. feature-gate 环境变量 `HAJIMI_REFLECTOR_V1_ENABLED`<br>2. blackboard keys 使用 `const BB_*`<br>3. 保留旧 `Critique` 解析 fallback<br>4. 单元测试覆盖成功/失败/Stop-Loss 触发 | 文字展开 | 必须 |
| 风险边界 | 禁止删除 legacy Critique 解析逻辑 | `grep "legacy.*Critique\|fallback" src/intelligence/agent-core/llm/bridge.rs` | 必须 |
| 测试基线 | 当前编译 / 测试状态 | `cargo check -p intelligence-agent-core` 0 errors<br>`cargo test -p intelligence-agent-core --lib` 105 passed | 必须 |
| 文档同步要求 | 无新增公开文档 | N/A | 按需 |
| 历史债务 / 相关缺陷 | 无 | N/A | 按需 |

### 探索补充栏
本任务为**已知解实现**，无需探索补充栏。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息
- **工单编号**：B-09/14
- **角色**：Engineer
- **目标**：Reflector V1 feature-gate + blackboard 协议标准化 + DTO/Stop-Loss 单元测试
- **输入**：引用输入基线中“目标范围”与“现状基线”所有项
- **依赖关系**：依赖 Day 8 产出（无并行依赖）

### 2）输出交付物
- **变更文件**：
  - `src/intelligence/agent-core/prompts/mod.rs`
  - `src/intelligence/agent-core/agent_loop.rs`
  - `src/intelligence/agent-core/llm/bridge.rs`
  - `src/intelligence/agent-core/reflector_dto.rs`
- **核心修改点**：
  - 新增 `is_reflector_v1_enabled()` 函数
  - 新增 `const BB_REFLECTOR_CRITIQUE`、`BB_PLAN_ADJUSTMENT`、`BB_STOP_LOSS`
  - `ReflectorLlmBridge` 增加 fallback 逻辑
  - 新增 ≥2 个单元测试（DTO + Stop-Loss）
- **必须包含**：
  - `grep -c "fn is_reflector_v1_enabled"` ≥ 1
  - `grep -c "const BB_"` ≥ 3
  - `grep -c "#\[cfg(test)\]"` ≥ 1 且 `grep -c "fn test_"` ≥ 2
- **禁止包含**：`rm`、`unsafe`、硬编码 mock 成功、删除 legacy 解析
- **交付证明**：`cargo test -p intelligence-agent-core --lib` 全部通过 + 刀刃表 16 项命令输出

### 3）规模与复杂度观察
- **推荐目标**：单函数保持单一职责，测试函数 ≤50 行
- **复杂度说明**：Stop-Loss 测试可能涉及状态机，预计单函数 40-60 行。如超过 60 行将声明 `DEBT-COMPLEXITY-B09-001`
- **禁止行为**：为压行数硬拆函数、添加无意义中间层

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 编译通过 | `cargo check -p intelligence-agent-core` | 返工 |
| FMT | 格式检查通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p intelligence-agent-core -- -D warnings` | 返工或声明债务 |
| TEST | 单元测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工 |
| ARCH | 不违反分层与接口约束 | `grep "legacy.*Critique" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | 返工 |
| REAL | 禁止假实现 | 所有测试真实执行，无 `#[ignore]` | 返工 |
| DOC | 无需新增文档 | N/A | - |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `is_reflector_v1_enabled()` 函数定义 | `grep -c "fn is_reflector_v1_enabled" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| FUNC | FUNC-002 | feature-gate 关闭时跳过路由 | `grep -c "is_reflector_v1_enabled" src/intelligence/agent-core/agent_loop.rs` ≥ 1 | [ ] |
| FUNC | FUNC-003 | blackboard 标准 keys（3 个 const） | `grep -c "const BB_REFLECTOR_CRITIQUE\|const BB_PLAN_ADJUSTMENT\|const BB_STOP_LOSS" src/intelligence/agent-core/agent_loop.rs` ≥ 3 | [ ] |
| FUNC | FUNC-004 | ReflectorLlmBridge fallback 到 legacy Critique | `grep -c "fallback\|legacy.*Critique" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |
| CONST | CONST-001 | `HAJIMI_REFLECTOR_V1_ENABLED` 默认 true | `grep -c "HAJIMI_REFLECTOR_V1_ENABLED" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| CONST | CONST-002 | Reflector DTO 单元测试 ≥2 个 | `grep -c "fn test_" src/intelligence/agent-core/reflector_dto.rs` ≥ 2 | [ ] |
| CONST | CONST-003 | Stop-Loss 单元测试触发 handoff | `grep -c "fn test_stop_loss\|fn test_" src/intelligence/agent-core/agent_loop.rs` ≥ 2 | [ ] |
| CONST | CONST-004 | 所有测试通过 | `cargo test -p intelligence-agent-core --lib` 退出码 0 | [ ] |
| NEG | NEG-001 | feature-gate 关闭测试 | `$env:HAJIMI_REFLECTOR_V1_ENABLED="false"; cargo test -p intelligence-agent-core --lib` 全部通过 | [ ] |
| NEG | NEG-002 | 缺失验证报告 UNKNOWN | `grep -c "Unknown\|UNKNOWN" src/intelligence/agent-core/reflector_dto.rs` ≥ 1 | [ ] |
| NEG | NEG-003 | 编译无 error | `cargo check -p intelligence-agent-core` 退出码 0 | [ ] |
| NEG | NEG-004 | 现有 105 个测试不破坏 | `cargo test -p intelligence-agent-core --lib` 105 passed | [ ] |
| UX | UX-001 | blackboard const keys 有文档注释 | `grep -c "///.*BB_" src/intelligence/agent-core/agent_loop.rs` ≥ 3 | [ ] |
| UX | UX-002 | feature-gate 函数有 rustdoc | `grep -c "///.*reflector_v1" src/intelligence/agent-core/prompts/mod.rs` ≥ 1 | [ ] |
| E2E | E2E-001 | workspace 编译通过 | `cargo check --workspace` 退出码 0 | [ ] |
| High | HIGH-001 | legacy Critique 解析完整保留 | `grep -c "legacy.*Critique\|fallback" src/intelligence/agent-core/llm/bridge.rs` ≥ 1 | [ ] |

---

## 【模块3-B】地狱红线（10项）
1. 零占位符违规 → 返工
2. 验证造假（声称已验证但无命令输出） → 返工
3. 编译 / 测试失败 → 返工
4. 假实现 / mock 成功 → 返工
5. 架构约束违反（删除 legacy 解析） → 返工
6. 新增 warning 未申报 → 返工
7. 范围失控 → 返工
8. Git 历史不完整 → 返工
9. 复杂度超标且未声明 `DEBT-COMPLEXITY` → 返工
10. 探索任务伪装确定性完成 → 返工

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | feature-gate 开关 + blackboard keys 标准化 | [ ] | CF-B09-001 | |
| 约束与回归用例（RG） | legacy fallback 保留 + 旧测试通过 | [ ] | RG-B09-001 | |
| 负面路径用例（NG） | feature-gate 关闭 + Stop-Loss 触发 | [ ] | NG-B09-001 | |
| 用户体验用例（UX） | const keys + rustdoc 文档 | [ ] | UX-B09-001 | |
| 端到端关键路径（E2E） | `cargo test --lib` 全通过 | [ ] | E2E-B09-001 | |
| 高风险场景（High） | legacy 路径完整保留 | [ ] | High-B09-001 | |
| 字段完整性 | 每条用例前置/预期/实际/风险等级 | [ ] | — | |
| 需求映射 | 用例映射到 Day 9 任务清单 | [ ] | — | |
| 自测执行 | 完整跑过一轮自测 | [ ] | — | |
| 范围边界与债务 | 未覆盖项明确标注 | [ ] | — | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## ✅ 工单 B-09/14 完成并提交

### 提交信息
- Commit: `feat(intelligence/agent-core): add Reflector V1 feature-gate, blackboard protocol, and tests`
- 分支: `feature/AGENT-PROMPT-CORE-001-phase3`
- 变更文件:
  - `src/intelligence/agent-core/prompts/mod.rs`
  - `src/intelligence/agent-core/agent_loop.rs`
  - `src/intelligence/agent-core/llm/bridge.rs`
  - `src/intelligence/agent-core/reflector_dto.rs`

### 本轮目标与实际结果
- 目标: Reflector V1 feature-gate + blackboard 协议标准化 + 单元测试
- 实际完成: 全部 4 个文件修改完成，16 项刀刃表通过
- 未完成/不在范围: 无

### 关键决策记录
- DECISION-001: `is_reflector_v1_enabled()` 放在 `prompts/mod.rs` —— 复用已有 feature-gate 模式，保持一致性
- DECISION-002: Stop-Loss 测试使用同步逻辑 —— 避免 flaky，降低测试维护成本

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
- 关键函数: `ReflectorLlmBridge::llm_critique`、`test_stop_loss`
- 是否存在复杂度例外: 无
- 若有，说明: N/A

### 债务声明
- DEBT-COMPLEXITY-B09-001: 无（未触发）
- DEBT-TEST-B09-001: 无

### 风险与回滚点
- 主要风险: feature-gate 关闭后路由逻辑异常
- 回滚方式: `git revert <commit>` 或删除 feature-gate 相关代码
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | legacy Critique 解析被意外删除 | 暂停实现，先恢复 legacy 逻辑 | 返工 |
| QUALITY-001 | 自动化闸门连续 2 次不通过 | 停止堆代码，先修复质量 | 返工 |
| COMPLEXITY-001 | Stop-Loss 测试单函数 >60 行且连续 2 次返工 | 允许带 `DEBT-COMPLEXITY` 交付 | 记录债务 |
| TEST-001 | 因时序问题导致测试 flaky | 重构为纯同步逻辑 | 延期 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **AGENT-PROMPT-CORE-001 Phase 3 Day 9** 通用高压任务！

### 技术背景
Day 7-8 已完成 `ReflectorCritiqueV1Dto` 定义、`ReflectorLlmBridge` 改造和 AgentLoop 路由逻辑。本轮任务：添加 `HAJIMI_REFLECTOR_V1_ENABLED` feature-gate，标准化 blackboard keys，保留 legacy Critique fallback，并补充单元测试。

### 关键约束
- feature-gate 默认 `true`，环境变量 `"false"` 时关闭
- blackboard keys 使用 `const BB_*` 并带文档注释
- 必须保留 legacy Critique 解析
- 单元测试覆盖成功/失败/Stop-Loss 触发

### 质量红线
- 10 项地狱红线生效
- 刀刃表 16 项必须命令化验证
- 禁止使用绝对行数限制

### 工单并行矩阵
- B-09/14 Engineer：Reflector Feature-Gate + 集成测试 + Blackboard 协议

### 验收铁律
- `cargo check -p intelligence-agent-core` 0 errors
- `cargo test -p intelligence-agent-core --lib` 全部通过
- `$env:HAJIMI_REFLECTOR_V1_ENABLED="false"; cargo test ...` 全部通过
- 刀刃表 16 项命令全部通过

### 收卷要求
- 必须附自动化质量检查摘要
- 必须附刀刃表摘要
- 必须诚实声明债务

Ouroboros 闭环启动，**AGENT-PROMPT-CORE-001 Day 9**，执行！ ☝️🐍♾️🔥

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
