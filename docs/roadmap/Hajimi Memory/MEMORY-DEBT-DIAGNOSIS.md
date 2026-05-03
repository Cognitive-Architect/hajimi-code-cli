# DEBT-MEMORY — Hajimi 记忆系统现状诊断与债务清单

> **诊断范围**: `src/intelligence/memory/` + `src/intelligence/agent-core/`
> **状态**: 架构完整，落地率低
> **诊断日期**: 2026-05-03
> **来源分析**: `docs/roadmap/Hajimi Memory/p0.md` 深度代码审计

---

<!-- MEMORY-REMEDIATION-2026-05-03: remediation initiated based on real code audit -->

## 记忆系统概览（理论 vs 现实）

| 层级 | 名称 | 设计目标 | 技术栈 | 理论性能 | 实际状态 | 关键文件 |
|:---|:---|:---|:---|:---:|:---:|:---|
| Hot | SessionMemory | 当前会话快速上下文 | LRU 4K tokens + HashMap | O(1) ~100ns | ✅ **工作** | `memory/src/session.rs` |
| Warm | AutoMemory | 项目级持久化记忆 | mmap + zstd 32K + JSONL | O(log n) ~1μs | ⚠️ **部分工作** | `memory/src/auto.rs` |
| Cold | DreamMemory | 向量语义检索 | HNSW 384-dim + ONNX Runtime | O(log n) ~5ms | ❌ **基本不可用** | `memory/src/dream.rs` |
| RAG | GraphMemory | 知识图谱节点召回 | SQLite + 实体关系 | O(log n) ~10ms | ❌ **空壳实现** | `memory/src/graph.rs` |
| Cloud | CloudMemory | E2EE 加密远程同步 | Age+X25519+Argon2id | 网络延迟 | ⚠️ **默认启用未集成** | `memory/src/cloud.rs` |

**生产环境实际情况**：`AgentLoopBuilder` 默认配置 `memory: Some(None), sync_gateway: Some(None)`——五层中仅 Session + Cloud 默认创建，**真正的多层级联检索 (retrieve_multi) 在生产代码中从未触发**。

---

## 跨会话记忆现状

**设计意图**：支持"上次进行到哪里了？"的上下文恢复效果（类似 Claude Code 的 `memory.md`）。

**实际状态**：框架存在，功能缺失。

| 组件 | 设计能力 | 实际状态 | 关键缺失 |
|:---|:---|:---|:---|
| CheckpointManager | 保存/恢复 Plan + Reflection + Blackboard | 仅内存 Vec，进程退出丢失 | 无 `restore_from_memory()` API |
| AutoMemory | JSONL 持久化到 `~/.hajimi/memory/{project_id}/` | 需显式启用 + 手动调用 `load()` | 无项目级自动恢复逻辑 |
| EpisodicMemory | 记录经验片段 (action/content/outcome) | 内存 VecDeque，MAX_EPISODES=1000 | 无跨会话查询接口 |
| ReflectionPersistence | 推送到 Dream/Graph，从 Session 恢复 | Dream/Graph 不可用，`load()` 有反序列化 bug | 无可靠的持久化-恢复链路 |
| Planner/Reflector | `save_to_graph()` / `load_from_graph()` | 实现为空 | LLM Bridge 未集成记忆加载 |

---

## 已知限制与债务清单（诚实声明）

### 债务 1: AutoMemory 生产默认不启用

- **位置**: `src/intelligence/memory/src/auto.rs:43-48`, `src/intelligence/memory/src/memory_gateway.rs:37-40`
- **状态**: ⚠️ **部分工作，需显式启用**
- **影响**: 项目级 JSONL 持久化仅在测试环境 `memory_sync_e2e.rs` 中启用，生产环境默认关闭，导致用户对话历史无法跨进程恢复
- **说明**: `AutoMemory` 实现完整（原子写入 `NamedTempFile + fs::rename`、`load()` 可从磁盘恢复），但 `MemoryGateway::new()` 不自动创建 AutoMemory 实例。需要显式调用 `enable_auto(project_id)`
- **验证**: `grep -n "enable_auto" src/intelligence/memory/src/memory_gateway.rs` → 存在但非默认调用

### 债务 2: DreamMemory HNSW 模块禁用

- **位置**: `src/intelligence/memory/src/dream.rs:67-72`, `src/intelligence/memory/src/lib.rs:9-10`
- **状态**: ❌ **基本不可用**
- **影响**: 384 维向量语义检索完全失效，无法通过语义相似度召回历史上下文
- **说明**: `lib.rs:9-10` 有注释 `# DEBT-HNSW-W34: HNSW模块临时禁用，Week 34重构`。`dream.rs:11-15` 的 `OnnxSession` 是空占位类型，`search` 方法不可用
- **验证**: `grep -n "DEBT-HNSW-W34\|HNSW" src/intelligence/memory/src/lib.rs src/intelligence/memory/src/dream.rs` → 匹配

### 债务 3: GraphMemory store()/recall() 空实现

- **位置**: `src/intelligence/memory/src/graph.rs:24-58`
- **状态**: ❌ **空壳实现**
- **影响**: 知识图谱节点召回完全无效，无法通过实体关系检索历史上下文
- **说明**: `graph.rs:37-38` 的 `store()` 和 `recall()` 方法返回空 `Vec`，仅作占位。需显式调用 `enable_graph()` 启用，但启用后也无实际功能
- **验证**: `sed -n '37,38p' src/intelligence/memory/src/graph.rs` → 返回空 Vec

### 债务 4: CheckpointManager 缺少从持久层恢复的 API

- **位置**: `src/intelligence/memory/src/checkpoint.rs:18-83`
- **状态**: ❌ **框架存在，功能缺失**
- **影响**: 检查点仅存储在 `Arc<RwLock<Vec<Checkpoint>>>` 内存中，进程退出丢失。即使注入 `MemoryGateway`，也没有 `restore_from_auto_memory()` 或 `restore_latest_from_disk()` 方法
- **说明**: `save()` 支持序列化到内存 Vec + 可选推送到 MemoryGateway，但 `restore_latest()` 只从内存恢复。没有项目级检查点恢复逻辑
- **验证**: `grep -n "restore" src/intelligence/memory/src/checkpoint.rs` → 仅 `restore_latest`，无持久化恢复

### 债务 5: AgentLoopBuilder 默认不注入 memory/sync_gateway

- **位置**: `src/intelligence/agent-core/src/agent_loop_builder.rs:46`, `src/intelligence/agent-core/src/agent_loop.rs:70-74`
- **状态**: ❌ **默认关闭，集成断裂**
- **影响**: `AgentLoop` 初始化时 `memory_retriever` 的 `sync_gateway` 和 `memory` 均为 `None`，导致 `retrieve_multi` 永不触发，`store()` 仅做检查点内存保存
- **说明**: `AgentLoopBuilder` 默认配置 `memory: Some(None), sync_gateway: Some(None)`。生产代码（如 `minimal_agent.rs:10-18`）只启用 Session + Cloud，Auto/Dream/Graph 全为 `None`
- **验证**: `grep -n "memory: Some(None)\|sync_gateway: Some(None)" src/intelligence/agent-core/src/agent_loop_builder.rs` → 匹配

### 债务 6: ReflectionPersistence load() 反序列化 bug

- **位置**: `src/intelligence/memory/src/reflection_persistence.rs:48-50`
- **状态**: ❌ **有 bug**
- **影响**: 无法从 SessionMemory 恢复历史 Reflection，跨会话经验积累断裂
- **说明**: `persist()` 尝试推送到 Dream（向量）和 Graph（知识图谱），但两者均不可用。`load()` 逻辑尝试从 Session 恢复，但反序列化格式错误
- **验证**: `sed -n '48,50p' src/intelligence/memory/src/reflection_persistence.rs` → 反序列化逻辑

### 债务 7: 无项目级记忆初始化与摘要生成机制

- **位置**: 全局缺失（无对应文件）
- **状态**: ❌ **完全未实现**
- **影响**: 无法实现"上次进行到哪里了？"的效果。没有 `load_project_memory(project_id)`、没有 `query_last_session(project_id)`、没有将历史 Reflection/Checkpoint 压缩为 LLM context 的摘要生成逻辑
- **说明**: 这是跨会话记忆的"最后一公里"——即使修复了 Auto/Dream/Graph，也需要一个顶层协调器来：①加载持久化记忆 ②恢复最近 Checkpoint ③生成摘要注入 Blackboard ④构建带记忆的 AgentLoop
- **验证**: `grep -r "load_project_memory\|query_last_session\|resume_project" src/intelligence/` → 0 匹配

---

## 验证证据

```powershell
# 五层记忆系统状态检查
grep -n "enable_auto\|enable_dream\|enable_graph" src/intelligence/memory/src/memory_gateway.rs
grep -n "DEBT-HNSW-W34\|HNSW" src/intelligence/memory/src/lib.rs src/intelligence/memory/src/dream.rs
sed -n '37,38p' src/intelligence/memory/src/graph.rs
grep -n "restore" src/intelligence/memory/src/checkpoint.rs
grep -n "memory: Some(None)\|sync_gateway: Some(None)" src/intelligence/agent-core/src/agent_loop_builder.rs
grep -r "load_project_memory\|query_last_session\|resume_project" src/intelligence/

# AgentLoop 默认配置检查
grep -n "memory_retriever\|sync_gateway" src/intelligence/agent-core/src/agent_loop.rs
grep -n "memory: Some(None)" src/intelligence/agent-core/src/agent_loop_builder.rs

# 编译与测试
cargo check --workspace                                 # 0 errors
cargo test -p codex-twist --test token_tracking_e2e     # 12 passed

# 记忆系统文件行数统计
wc -l src/intelligence/memory/src/*.rs src/intelligence/agent-core/src/agent_loop*.rs src/intelligence/agent-core/src/memory_retriever.rs src/intelligence/agent-core/src/planner.rs
```

---

## 清偿建议（优先级排序）

| 优先级 | 债务 | 最小修复路径 | 预计影响 |
|:---|:---|:---|:---|
| P0 | 债务 5 | 修改 `AgentLoopBuilder` 默认配置，注入 `MemoryGateway` 和 `SyncMemoryGateway` | 使五层级联检索在生产环境可用 |
| P0 | 债务 4 | 为 `CheckpointManager` 实现 `restore_from_auto_memory(project_id)` | 使检查点可跨进程恢复 |
| P1 | 债务 1 | 在 `MemoryGateway::new(project_id)` 中自动启用 `AutoMemory` | 使项目级持久化默认生效 |
| P1 | 债务 3 | 实现 `GraphMemory::store()` / `recall()` 的 SQLite 读写逻辑 | 使知识图谱召回可用 |
| P2 | 债务 6 | 修复 `ReflectionPersistence::load()` 反序列化逻辑 | 使经验片段可恢复 |
| P2 | 债务 2 | 恢复 HNSW 模块（解除 `DEBT-HNSW-W34`）或替换为轻量向量索引 | 使语义检索可用 |
| P3 | 债务 7 | 实现项目级记忆初始化协调器 + 记忆摘要生成 | 实现"上次到哪了？"效果 |

---

*本文件与代码同步维护。所有 metric 来自 `2026-05-03` 代码审计，零占位符，零估算。*
