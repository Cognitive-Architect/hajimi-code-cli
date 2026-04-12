# HAJIMI V3 源代码索引

> **文档版本**: v1.0  
> **最后更新**: 2026-04-02  
> **代码总行数**: ~4,726行自有代码（不含依赖）  

---

## 📁 目录总览

```
src/
├── adapters/           # 外部工具适配器（EVM/MCP）
├── api/                # REST API 接口
├── bench/              # 性能基准测试
├── chimera/            # Chimera REPL 引擎（Rust）⭐
├── cli/                # 命令行工具
├── crates/             # Rust Crates（Codex Twist/HNSW/EVM）⭐
├── disk/               # 磁盘管理与存储优化
├── format/             # 数据格式定义（HCTX/HNSW）
├── mcp/                # Model Context Protocol 实现
├── middleware/         # Express/Koa 中间件
├── migration/          # 数据库迁移工具
├── p2p/                # WebRTC P2P 同步核心 ⭐
├── security/           # 安全与限流控制
├── storage/            # 存储层（16分片SQLite）⭐
├── sync/               # 同步管理
├── test/               # 单元测试辅助
├── tests/              # 测试套件（已移入src）⭐
├── scripts/            # 脚本工具（已移入src）⭐
├── utils/              # 通用工具
├── vector/             # 向量索引（HNSW/SimHash）⭐
├── wasm/               # WASM 运行时与桥接
└── worker/             # Worker 线程池
```

---

## 🎯 核心子系统详解

### 1. chimera/ - Chimera REPL 引擎
**技术栈**: Rust (Edition 2024)  
**代码规模**: ~787行  
**状态**: CH-01~10 已完成，CH-11 待启动

| 文件 | 行数 | 功能描述 |
|------|------|----------|
| `src/lib.rs` | 70 | 核心引擎入口，已冻结 |
| `src/archive_writer.rs` | 97 | .hctx 归档格式 + BLAKE3 校验 |
| `src/codex_bridge.rs` | 112 | Codex MemoryGateway FFI 桥接 |
| `src/state.rs` | 89 | ReplState 状态机数据结构 |
| `src/traits.rs` | 76 | ReplEngineCore trait 定义 |
| `src/session.rs` | 73 | 会话状态管理 |
| `src/engine.rs` | 61 | 异步事件循环 |
| `src/repl.rs` | 61 | ZeroTUI 主循环 |
| `src/io.rs` | 59 | I/O 抽象层（Stdin/Mock）|
| `src/event.rs` | 48 | 事件通道定义 |
| `src/clock.rs` | 38 | 时钟抽象（SystemTime/Mock）|

**关键特性**:
- ZeroTUI 架构：纯业务逻辑，无 TUI 依赖
- Trait 化设计：Clock/InputSource/ReplEngineCore
- 零拷贝 FFI：通过 codex_bridge 与 hajimi-codex-twist 交互

---

### 2. crates/ - Rust Crates

#### 2.1 hajimi-codex-twist/ (~2,858行)
**定位**: OpenAI Codex 的本地优先移植版  
**核心概念**: Thread（对话容器）+ Turn（单次交互）+ Approval（审批系统）

| 模块 | 文件 | 功能 |
|------|------|------|
| 核心 | `lib.rs` | 模块导出与便捷函数 |
| 对话 | `thread.rs` | Thread/ThreadConfig/ThreadId 定义 |
| 交互 | `turn.rs` | Turn/TurnStatus/ToolCall 定义 |
| 存储 | `storage.rs` | HCTX 本地存储适配 |
| 审批 | `approval.rs` | 5级审批策略（Ask/Dangerous/Once/Auto/Deny）|
| FFI | `ffi.rs` | napi-rs Node.js 绑定 |
| 内存 | `memory/` | 5级内存架构（Focus/Working/Archive/RAG/Gateway）|
| 分层 | `tiered/` | Hot/Warm/Cold/Archive 分层存储 |

**关键改造**:
- 云端 JSON → LCR 本地 .hctx 存储
- OAuth → 用户自填 API Key
- 轻量级：<600行核心代码

#### 2.2 hajimi-hnsw/ (~632行)
**定位**: HNSW 向量索引的 Rust/WASM 实现  
**性能目标**: 比 JS 快 5 倍

| 文件 | 功能 |
|------|------|
| `lib.rs` | HNSWIndex 实现（分层导航图+贪心搜索）|

**API 功能**:
- `insert(id, vector)` - 插入向量
- `search(query, k)` - K近邻搜索
- `searchBatch()` - 批量搜索（RISK-02 修复）
- `searchBatchZeroCopy()` - 零拷贝批量搜索
- `insert_batch()` - 批量插入

#### 2.3 evm-bench-adapter/ (~233行)
**定位**: EVM 漏洞基准测试适配器  
**技术**: ethers.rs + Foundry/Anvil

| 文件 | 功能 |
|------|------|
| `lib.rs` | 模块导出 |
| `runner.rs` | 漏洞利用运行器 |
| `types.rs` | ExploitConfig/Vulnerability 类型 |
| `main.rs` | CLI 入口 |

---

### 3. p2p/ - WebRTC P2P 同步
**技术栈**: TypeScript + WebRTC DataChannel  
**协议**: ICEv2 (RFC 8445) + Yjs CRDT

| 文件 | 功能 |
|------|------|
| `signaling-server.js` | WebSocket 信令服务器 |
| `signaling-client.js` | WebRTC 客户端 |
| `datachannel-manager.js` | DataChannel 管理 |
| `crdt-engine.ts` | Yjs CRDT 引擎封装 |
| `sync-engine.ts` | 同步生命周期管理 |
| `yjs-adapter.ts` | Yjs 适配器 |
| `dvv-manager.ts` | Dotted Version Vector 实现 |
| `ice-manager.ts` | ICE 候选管理 |
| `turn-client.ts` | TURN 客户端（RFC 5766）|
| `bidirectional-sync*.ts` | 双向同步实现（v1-v4 演进）|

**关键协议**:
- 信令协议: JSON-RPC 2.0 over WebSocket
- 数据通道: 64KB 分片 + SHA256 校验
- 加密: AES-256-GCM
- 拥塞控制: 滑动窗口 + RTT 测量

---

### 4. storage/ - 存储层
**核心**: 16分片 SQLite + WAL 模式  
**路由**: SimHash-64 高 8bit → 分片 00-15

| 文件 | 功能 |
|------|------|
| `shard-router.js` | SimHash → 分片路由 |
| `chunk.js` | Chunk 数据模型 |
| `connection-pool.js` | SQLite 连接池 |
| `batch-writer-optimized.js` | 批量写入优化 |
| `migrate.js` | 数据库迁移 |
| `queue-db-interface.ts` | 持久化队列接口 |
| `leveldb-optimized.ts` | LevelDB 优化配置 |
| `rocksdb-adapter.ts` | RocksDB 适配器 |

**性能指标**:
- 写入: 9,569 ops/s（WAL 批量）
- 16分片: 单机支持 100K+ 向量

---

### 5. vector/ - 向量索引
**算法**: HNSW（图索引）+ SimHash-64（LSH）

| 文件 | 功能 |
|------|------|
| `hnsw-core.js` | JS 版 HNSW 实现 |
| `hnsw-index-wasm-v3.js` | WASM 版 HNSW 桥接（1.94x 加速）|
| `hnsw-index-hybrid.js` | JS+WASM 混合索引 |
| `hnsw-persistence.js` | HNSW 索引持久化 |
| `hnsw-memory-manager.js` | 内存管理 |
| `distance.js` | 距离计算（余弦/欧氏）|
| `encoder.js` | 向量编码器 |
| `simhash64.js` | SimHash-64 LSH 实现 |
| `hybrid-retriever.js` | 混合检索器 |
| `lazy-loader.js` | 懒加载管理 |
| `write-queue.js` | 写入队列 |
| `wal-checkpointer.js` | WAL 检查点 |

**性能加速**:
- WASM 查询: 1.94x
- WASM 构建: 7.7x

---

### 6. security/ - 安全与限流
**核心**: Token Bucket + SQLite 持久化 + 熔断器

| 文件 | 功能 |
|------|------|
| `rate-limiter-sqlite-luxury.js` | 豪华版限流（9,569 ops/s）|
| `rate-limiter-redis.js` | Redis 限流器 |
| `rate-limiter-redis-v2.js` | Redis 限流 v2 |
| `rate-limiter.js` | 基础限流器 |
| `rate-limiter-factory.js` | 限流器工厂 |
| `headers.js` | 安全响应头 |

---

### 7. mcp/ - Model Context Protocol
**规范**: MCP 2025-03-26  
**传输**: SSE / stdio

| 文件 | 功能 |
|------|------|
| `server.ts` | MCP 服务器实现（8,724行）|
| `adapters/ffi-bridge/tools-bridge.ts` | FFI 工具桥接 |
| `adapters/ffi-bridge/resources-bridge.ts` | FFI 资源桥接 |
| `adapters/mcp/message-adapter.ts` | 消息适配器 |
| `adapters/mcp/sse-transport.ts` | SSE 传输 |
| `adapters/mcp/stdio-transport.ts` | stdio 传输 |

---

### 8. adapters/evm/ - EVM 适配器
**工具链**: Foundry + Slither + Docker

| 文件 | 功能 |
|------|------|
| `foundry-adapter.ts` | Foundry 工具链适配 |
| `slither-adapter.ts` | Slither 静态分析适配 |
| `docker-foundry-adapter.ts` | Docker 隔离运行 |
| `evm-pipeline.ts` | EVM 检测流水线 |
| `container-manager.ts` | 容器生命周期管理 |
| `patch-generator.ts` | 补丁生成器 |
| `verify-runner.ts` | 验证运行器 |
| `vuln-loader.ts` | 漏洞数据加载 |

---

### 9. disk/ - 磁盘管理
**特性**: ENOSPC 处理 + 内存映射 + 紧急模式

| 文件 | 功能 |
|------|------|
| `memory-mapped-store.js` | mmap 存储（Archive 层）|
| `enospc-handler.js` | 磁盘满处理 |
| `emergency-mode.js` | 紧急模式（只读降级）|
| `overflow-manager.js` | 溢出管理 |
| `overflow-manager-v2.js` | 溢出管理 v2 |
| `block-cache.js` | 块缓存 |

---

### 10. wasm/ - WASM 运行时
**技术**: wasm-bindgen + SharedArrayBuffer

| 文件 | 功能 |
|------|------|
| `loader.js` | WASM 加载器 |
| `runtime-loader.js` | 运行时加载器 |
| `hnsw-bridge.js` | HNSW WASM 桥接 |
| `wasm-memory-pool.js` | WASM 内存池 |
| `sab-allocator.ts` | SharedArrayBuffer 分配器 |
| `wasm-sab-bridge.ts` | SAB 桥接 |

---

### 11. worker/ - Worker 线程池
| 文件 | 功能 |
|------|------|
| `hnsw-worker.js` | HNSW Worker |
| `index-builder-bridge.js` | 索引构建桥接 |
| `worker-pool.js` | Worker 池管理 |

---

## 🧪 测试与脚本

### tests/ - 测试套件
| 目录 | 内容 |
|------|------|
| `unit/` | 单元测试 |
| `integration/` | 集成测试 |
| `e2e/` | 端到端测试 |
| `p2p/` | P2P 同步测试 |
| `rc/` | 发布候选测试 |
| `bench/` | 性能基准 |
| `mcp/` | MCP 测试 |

**关键测试**:
- `webrtc-handshake.e2e.js` - WebRTC 握手测试
- `datachannel-transfer.e2e.js` - DataChannel 传输测试
- `serial-transfer-100x.test.js` - 100次串行传输测试

### scripts/ - 脚本工具
| 文件 | 功能 |
|------|------|
| `install-wrtc.bat` | Windows wrtc 安装 |
| `install-wrtc.sh` | Linux/macOS wrtc 安装 |
| `run-real-e2e.sh` | 真实网络 E2E 测试 |
| `line-count.rs` | 代码行数统计 |

---

## 📊 代码统计

| 子系统 | 语言 | 行数 | 状态 |
|--------|------|------|------|
| chimera-repl | Rust | ~787 | CH-10 完成 |
| hajimi-codex-twist | Rust | ~2,858 | 稳定 |
| hajimi-hnsw | Rust | ~632 | 稳定 |
| evm-bench-adapter | Rust | ~233 | 稳定 |
| JS/TS 源码 | TS/JS | ~4,726 总计 | 活跃开发 |

---

## 🔗 关键依赖关系

```
chimera-repl
└── hajimi-codex-twist (Thread/Turn/MemoryGateway)

vector/
├── hajimi-hnsw (WASM HNSW)
└── hnsw-core.js (JS HNSW)

p2p/
├── Yjs (CRDT)
├── @koush/wrtc (WebRTC)
└── ws (WebSocket)

mcp/
└── @modelcontextprotocol/sdk

storage/
├── sql.js (SQLite)
├── level (LevelDB)
└── ioredis (Redis)
```

---

## 📝 如何阅读代码

1. **从入口开始**: `chimera/chimera-repl/src/lib.rs`（REPL 引擎）
2. **理解存储**: `storage/shard-router.js`（16分片路由）
3. **看 P2P 同步**: `p2p/sync-engine.ts`（数据同步核心）
4. **向量检索**: `vector/hnsw-index-wasm-v3.js`（WASM 加速）
5. **安全限流**: `security/rate-limiter-sqlite-luxury.js`（豪华版限流）

---

*本索引文档与代码同步维护，最后更新于 2026-04-02*
