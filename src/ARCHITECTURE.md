# HAJIMI V3 架构文档

> **文档版本**: v1.0  
> **架构风格**: 本地优先 + P2P 同步 + 分层存储  
> **核心原则**: ZeroTUI（无 TUI 依赖）、零拷贝、最小侵入

---

## 🏛️ 系统架构总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Application Layer                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   CLI工具   │  │  REST API   │  │  MCP 协议   │  │   EVM 检测流水线    │ │
│  │  (cli/)     │  │   (api/)    │  │  (mcp/)     │  │   (adapters/evm/)   │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────┼────────────────────┼────────────┘
          │                │                │                    │
          └────────────────┴────────────────┴────────────────────┘
                                    │
┌───────────────────────────────────┼─────────────────────────────────────────┐
│                           Sync Engine Layer                                  │
│  ┌────────────────────────────────┼──────────────────────────────────────┐  │
│  │        P2P Synchronization     │  (p2p/)                              │  │
│  │  ┌──────────────┐  ┌───────────┴──────────┐  ┌──────────────────┐    │  │
│  │  │  CRDT Engine │  │   WebRTC Transport   │  │   Sync Manager   │    │  │
│  │  │  (Yjs)       │  │   (ICE/TURN/DTLS)   │  │   (Push/Pull)    │    │  │
│  │  └──────────────┘  └──────────────────────┘  └──────────────────┘    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────┼─────────────────────────────────────────┐
│                          Storage & Index Layer                               │
│  ┌──────────────────┬─────────────┴──────────────┬──────────────────────┐   │
│  │   Vector Index   │     KV Storage             │   Chunk Storage      │   │
│  │  (vector/)       │     (storage/)             │   (format/)          │   │
│  │  ┌────────────┐  │  ┌──────────────────────┐  │  ┌────────────────┐  │   │
│  │  │ HNSW (WASM)│  │  │ 16-Shard SQLite      │  │  │ .hctx Format   │  │   │
│  │  │ SimHash-64 │  │  │ SimHash Routing      │  │  │ BLAKE3 Verify  │  │   │
│  │  └────────────┘  │  │ WAL + ConnectionPool │  │  └────────────────┘  │   │
│  └──────────────────┘  └──────────────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────┼─────────────────────────────────────────┐
│                          AI & Memory Layer (Chimera)                         │
│  ┌────────────────────────────────┼──────────────────────────────────────┐  │
│  │    Chimera REPL (chimera/)     │  Codex Twist (crates/)               │  │
│  │  ┌──────────────────────────┐  │  ┌────────────────────────────────┐  │  │
│  │  │ ZeroTUI Event Loop       │  │  │ 5-Tier Memory Architecture     │  │  │
│  │  │ - Clock Abstraction      │  │  │ - Focus (LRU 4K)               │  │  │
│  │  │ - InputSource Trait      │  │  │ - Working (Sliding Window 32K) │  │  │
│  │  │ - Archive Writer         │  │  │ - Archive (mmap+zstd 1M)       │  │  │
│  │  │ - Codex Bridge           │  │  │ - RAG (HNSW 384-dim)           │  │  │
│  │  └──────────────────────────┘  │  └────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────┴─────────────────────────────────────────┐
│                         Runtime & Infrastructure Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────┐│
│  │   Worker    │  │    WASM     │  │   Security  │  │   Disk Management    ││
│  │  (worker/)  │  │   (wasm/)   │  │ (security/) │  │     (disk/)          ││
│  └─────────────┘  └─────────────┘  └─────────────┘  └──────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🎯 核心设计模式

### 1. ZeroTUI 架构
**核心思想**: 业务逻辑与 TUI 完全解耦

```rust
// chimera/src/repl.rs
pub struct ChimeraRepl<C: Clock, I: InputSource, R: AsyncWrite + Unpin> {
    state: ReplState<C>,
    input: I,           // 注入的输入源
    output: Pin<Box<R>>, // 注入的输出目标
    // ...
}

// 可以搭配任何 I/O：Stdin/文件/Mock
impl<C: Clock, I: InputSource, R: AsyncWrite + Unpin> ChimeraRepl<C, I, R> {
    pub async fn run<H: EventHandler>(&mut self, handler: &mut H) -> ReplResult<()> {
        // 纯业务逻辑，无 TUI 依赖
    }
}
```

### 2. 本地优先存储
**层级**: Hot → Warm → Cold → Archive

```
Hot Tier:     Memory (LRU 4K tokens)     O(1) ~100ns
Warm Tier:    mmap + zstd (32K tokens)   O(log n) ~1μs
Cold Tier:    LevelDB (1M tokens)        O(log n) ~10ms
Archive Tier: .hctx File (unlimited)     O(log n) ~50ms
```

### 3. P2P 同步架构
**协议栈**:
```
Application:    Yjs CRDT (State Vector)
Transport:      WebRTC DataChannel ( unreliable )
Security:       DTLS 1.2 + AES-256-GCM
Connectivity:   ICEv2 (RFC 8445) + TURN (RFC 5766)
Signaling:      WebSocket + JSON-RPC 2.0
```

### 4. 16分片 SQLite
**路由算法**:
```javascript
// storage/shard-router.js
function route(key) {
    const hash = simhash64(key);        // 64-bit SimHash
    const shard = (hash >> 56) & 0x0F;  // 高 8bit → 00-15
    return `shard_${shard.toString(16).padStart(2, '0')}`;
}
```

---

## 🔌 关键接口定义

### 1. 存储接口
```typescript
// storage/queue-db-interface.ts
interface IQueueDb {
    getQueue(): Promise<SyncOperation[]>;
    saveQueue(queue: SyncOperation[]): Promise<void>;
    append(operation: SyncOperation): Promise<void>;
}
```

### 2. CRDT 引擎接口
```typescript
// p2p/crdt-engine.ts
interface ICrdtEngine {
    merge(local: Chunk, remote: Chunk): MergeResult;
    encodeState(chunk: Chunk): Uint8Array;
    decodeState(state: Uint8Array): Partial<Chunk>;
}
```

### 3. 限流器接口
```typescript
// security/rate-limiter.js
interface RateLimiter {
    check(key: string): Promise<RateLimitResult>;
    consume(key: string, tokens: number): Promise<boolean>;
    reset(key: string): Promise<void>;
}
```

### 4. Clock 抽象（Rust）
```rust
// chimera/src/clock.rs
pub trait Clock: Send + Sync + Clone + 'static {
    fn now_ms(&self) -> u64;
}

pub struct SystemTimeClock;
impl Clock for SystemTimeClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
```

---

## 🔄 数据流

### 1. Chunk 写入流程
```
┌─────────┐    ┌──────────────┐    ┌──────────────┐    ┌─────────────┐
│  Input  │───→│  SimHash-64  │───→│  Shard Router│───→│  SQLite WAL │
│  Chunk  │    │  (LSH)       │    │  (00-15)     │    │  (Async)    │
└─────────┘    └──────────────┘    └──────────────┘    └─────────────┘
                                                             │
                                                             ↓
┌─────────────┐    ┌──────────────┐    ┌──────────────┐    ┌─────────────┐
│  Complete   │←───│  HNSW Insert │←───│  Vector Enc  │←───│  Commit WAL │
│             │    │  (WASM)      │    │  (384-dim)   │    │             │
└─────────────┘    └──────────────┘    └──────────────┘    └─────────────┘
```

### 2. P2P 同步流程
```
Peer A                                    Peer B
  │                                         │
  │── WebSocket Signaling ─────────────────→│
  │   (offer/answer/ice-candidate)          │
  │                                         │
  │←──────── WebRTC Connection ────────────→│
  │   (ICE + DTLS handshake)                │
  │                                         │
  │── DataChannel Open ────────────────────→│
  │                                         │
  │── Yjs State Vector ────────────────────→│
  │   (encoded updates)                     │
  │                                         │
  │←─── Missing Updates ────────────────────│
  │                                         │
  │── CRDT Merge (YATA) ───────────────────→│
  │   (conflict-free)                       │
```

### 3. 向量检索流程
```
Query Vector
     │
     ↓
┌─────────────────┐
│ SimHash-64 LSH  │──→ Candidate Buckets
└─────────────────┘
     │
     ↓
┌─────────────────┐
│ HNSW Search     │──→ Top-K Approximate
│ (WASM)          │    (ef=64, m=16)
└─────────────────┘
     │
     ↓
┌─────────────────┐
│ Exact Distance  │──→ Re-rank & Filter
│ (Cosine)        │
└─────────────────┘
```

---

## 🛡️ 安全架构

### 1. 限流策略
```
Token Bucket (SQLite Persistent)
├── Burst: 100 requests
├── Rate: 10 req/s
├── Window: 60s
└── Circuit Breaker:
    ├── Failure Threshold: 50%
    ├── Recovery Timeout: 30s
    └── Half-Open Requests: 5
```

### 2. 审批策略（Codex Twist）
```rust
enum ApprovalPolicy {
    AskBeforeExec,      // 每次询问
    AskForDangerous,    // 危险操作询问
    AskOnceThenAuto,    // 首次询问后自动
    FullAuto,           // 完全自动
    FullDeny,           // 完全拒绝
}
```

### 3. 数据完整性
- **Chunk**: MD5-128 校验
- **Archive**: BLAKE3 校验（.hctx 格式）
- **P2P**: SHA256 分片校验

---

## 📊 性能基准

| 操作 | 指标 | 实现 |
|------|------|------|
| SQLite 批量写入 | 9,569 ops/s | WAL + 16分片 |
| HNSW 查询 | 1.94x 加速 | WASM |
| HNSW 构建 | 7.7x 加速 | WASM |
| WebRTC 握手 | <5s | ICEv2 |
| P2P 传输 | 64KB/s-10MB/s | DataChannel |
| MemoryGateway | O(1) ~100ns | LRU Focus |

---

## 🧩 扩展点

### 1. 添加新的存储后端
```typescript
// 实现 IQueueDb 接口
class MyCustomStorage implements IQueueDb {
    async getQueue(): Promise<SyncOperation[]> { }
    async saveQueue(queue: SyncOperation[]): Promise<void> { }
}
```

### 2. 添加新的 Clock 实现
```rust
// 实现 Clock trait
pub struct MyCustomClock;
impl Clock for MyCustomClock {
    fn now_ms(&self) -> u64 { }
}
```

### 3. 添加新的 InputSource
```rust
// 实现 InputSource trait
pub struct FileInput;
impl InputSource for FileInput {
    async fn read_line(&mut self) -> io::Result<String> { }
}
```

---

## 📝 架构决策记录 (ADR)

### ADR-001: 16分片 SQLite
- **决策**: 使用 SimHash-64 高 8bit 路由到 16 个 SQLite 分片
- **原因**: 单机可支持 100K+ 向量，避免单库性能瓶颈
- **状态**: 已实施

### ADR-002: ZeroTUI
- **决策**: REPL 引擎完全无 TUI 依赖
- **原因**: 支持多种运行时环境（CLI/Server/嵌入式）
- **状态**: 已实施（chimera-repl）

### ADR-003: WASM HNSW
- **决策**: 核心向量算法用 Rust/WASM 实现
- **原因**: 比 JS 快 5 倍，内存安全
- **状态**: 已实施

### ADR-004: 5级内存架构
- **决策**: Focus/Working/Archive/RAG/Gateway 分层
- **原因**: 平衡延迟、容量和成本
- **状态**: 已实施（hajimi-codex-twist）

---

*本架构文档与代码同步维护，最后更新于 2026-04-02*
