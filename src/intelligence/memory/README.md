# Memory 模块

HAJIMI IDE 的 5 级记忆系统（Session / Auto / Dream / Graph / Cloud），为 Agent Core 提供跨层级记忆存储、检索与同步能力。

## 职责

- **Session Memory（Hot）**：`SessionMemory` 基于 HashMap + VecDeque LRU 队列实现，最大 4000 tokens，O(1) 访问；`estimate_tokens` 按字符数估算；自动 LRU 驱逐保证容量上限；支持 `get_mut` 访问计数
- **Auto Memory（Warm）**：自动归档层，基于 SQLite + mmap + zstd 压缩，32K 容量，O(log n) 访问延迟
- **Dream Memory（Cold）**：向量语义层，HNSW 384-dim 向量索引，支持嵌入向量搜索与 `EMBEDDING_DIM` 维度校验
- **Graph Memory（RAG）**：知识图谱层，节点召回与关系遍历
- **Cloud Memory**：云端同步层，支持 E2EE 加密（Argon2id + age）
- **SyncMemoryGateway**：跨层统一网关接口，定义 `MemoryTier`（Session → Auto → Dream → Graph → Cloud）与 `fallback_order` 级联检索；`retrieve_multi` 按优先级从多层检索并打分排序；`sync_with_blackboard` 双向同步 Blackboard 快照；`tier_health` 健康检查
- **线程安全**：`SyncGatewayHandle = Arc<tokio::sync::Mutex<dyn SyncMemoryGateway>>`，供 AgentLoop 并发使用

## 测试

运行记忆模块全部测试：

```bash
cargo test -p memory
```

测试覆盖 Session 命中/未命中、空查询、Auto 层未命中、Dream 层可用性、Graph 空召回、Cloud 不可用降级、多层 fallback 检索、事件推送、Blackboard 同步、健康检查、访问计数累积。

## 依赖

```toml
[dependencies]
async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true, features = ["serde"] }
tokio = { workspace = true, features = ["time", "sync", "fs", "rt", "macros"] }
rusqlite = { workspace = true, features = ["bundled", "chrono"] }
age = { workspace = true, features = ["armor"] }
argon2 = { workspace = true }
zeroize = { workspace = true }
subtle = { workspace = true }
ort = { workspace = true, optional = true }
ndarray = { workspace = true, optional = true }
lru = { workspace = true }
jieba-rs = { workspace = true }
```

Feature `onnx` 启用 ONNX Runtime 嵌入支持。

## 关键文件

| 文件 | 说明 |
|------|------|
| `src/session.rs` | `SessionMemory`：LRU 4000 tokens 热点记忆 |
| `src/sync_gateway.rs` | `SyncMemoryGateway` trait、`MemoryTier`、`GatewayEvent`、`BlackboardSnapshot`、`TierHealth` |
| `src/memory_gateway.rs` | `MemoryGateway`：5 层统一实现与路由 |
| `src/dream.rs` | Dream Memory：HNSW 向量语义检索 |
| `src/graph.rs` | Graph Memory：知识图谱节点与关系 |
| `src/types.rs` | `MemoryEntry`、`MemoryLayerId` 等共享类型 |

## 5 级记忆架构

```
Hot   → Session  → LRU 4K tokens          → O(1)   ~100ns
Warm  → Auto     → mmap + zstd 32K        → O(log n) ~1μs
Cold  → Dream    → HNSW 384-dim           → O(log n) ~5ms
RAG   → Graph    → 知识图谱节点召回        → O(log n) ~10ms
Cloud → Cloud    → E2EE 加密远程同步       → 网络延迟
```
