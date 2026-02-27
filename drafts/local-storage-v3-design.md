# Hajimi 本地存储架构 v3.0 设计文档

> **状态**: 草案 v0.1  
> **日期**: 2026-02-22  
> **任务来源**: task/task01.md

---

## 1. 设计目标

| 目标 | 说明 |
|------|------|
| **纯本地优先** | 彻底放弃 Helia/IPFS 依赖，零网络启动 |
| **海量向量支持** | 支持 100K+ 向量分片存储，查询性能不 degrading |
| **轻量同步** | WebRTC 局域网 P2P 或 文件导出，无需中心服务器 |
| **Termux 友好** | Android 13 + Termux 环境原生支持 |

---

## 2. 核心架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Hajimi Local Storage v3.0                 │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────┐   │
│  │   API Layer │   │  Index Core │   │  Sync Manager   │   │
│  │             │   │             │   │                 │   │
│  │ • store()   │   │ • LSH Tree  │   │ • WebRTC P2P    │   │
│  │ • query()   │   │ • HNSW Graph│   │ • File Export   │   │
│  │ • delete()  │   │ • BF Filter │   │ • Delta Sync    │   │
│  └──────┬──────┘   └──────┬──────┘   └────────┬────────┘   │
│         │                 │                    │            │
│         └─────────────────┼────────────────────┘            │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                 Storage Engine                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │   │
│  │  │  MetaDB  │  │ ChunkDB  │  │   VectorDB       │  │   │
│  │  │ (SQLite) │  │ (Files)  │  │ (Binary Flat)    │  │   │
│  │  └──────────┘  └──────────┘  └──────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 存储分层

```
数据类型          存储方式              理由
─────────────────────────────────────────────────────────
元数据            SQLite              结构化查询、事务支持
小文件(<1MB)      SQLite BLOB         减少 inode 开销
大文件分片        文件系统            mmap 友好、流式读写
向量数据          二进制 Flat 文件     顺序读取、SIMD 优化
索引              内存 + 持久化快照    快速重建、定期 checkpoint
```

---

## 3. 存储格式设计

### 3.1 目录结构

```
~/.hajimi/storage/v3/
├── config.json              # 存储配置
├── meta/
│   └── metadata.db          # SQLite: chunks, tags, refs
├── vectors/
│   ├── index.hnsw           # HNSW 图索引
│   ├── vectors.bin          # 原始向量数据 (flat)
│   └── lsh/
│       └── lsh_tables.bin   # LSH 哈希表
├── chunks/
│   ├── 00/                  # 按 hash 前缀分片
│   ├── 01/
│   ├── ...
│   └── ff/
│       └── <chunk_hash>     # 实际分片文件
├── cache/
│   └── hot/                 # LRU 热缓存
└── sync/
    └── manifest.json        # 同步清单
```

### 3.2 Chunk 存储格式

```
┌──────────────────────────────────────────────────────┐
│                  Chunk File Format                   │
├──────────────────────────────────────────────────────┤
│  Magic (4B)        │ 0x48414A49 "HAJI"               │
│  Version (1B)      │ 0x03                            │
│  Flags (1B)        │ compression, encryption         │
│  Hash Type (1B)    │ 0x02=MD5, 0x03=SHA256, etc     │
│  Reserved (1B)     │ ─                               │
├──────────────────────────────────────────────────────┤
│  Original Size (8B)│ 原始数据大小                    │
│  Stored Size (8B)  │ 实际存储大小                    │
│  Chunk Hash (32B)  │ 分片哈希 (级联哈希结果)         │
│  Parent Hash (32B) │ 父级引用 (用于版本控制)         │
├──────────────────────────────────────────────────────┤
│  Vector Count (4B) │ 关联向量数量                    │
│  Vector IDs (N×8B) │ 指向 vectors.bin 的偏移         │
├──────────────────────────────────────────────────────┤
│  Payload (N bytes) │ 实际数据 (可能压缩)             │
├──────────────────────────────────────────────────────┤
│  CRC32 (4B)        │ 完整性校验                      │
└──────────────────────────────────────────────────────┘
```

### 3.3 Vector 存储格式 (Flat Binary)

```
vectors.bin:
┌────────────────────────────────────────────────────────┐
│ Header (256B)                                          │
│   - Magic: "HajimiVec"                                 │
│   - Version: 3                                         │
│   - Dim: 768 (向量维度)                                │
│   - Count: 100000 (向量总数)                           │
│   - Quantization: float32/float16/int8                 │
├────────────────────────────────────────────────────────┤
│ Index Section                                          │
│   - id → offset 映射表 (用于 O(1) 随机访问)            │
├────────────────────────────────────────────────────────┤
│ Vector Data (连续存储)                                 │
│   ┌─────────────┬─────────────┬─────────────┐         │
│   │  Vector #0  │  Vector #1  │  Vector #N  │         │
│   │  768×4B     │  768×4B     │  768×4B     │         │
│   └─────────────┴─────────────┴─────────────┘         │
└────────────────────────────────────────────────────────┘
```

---

## 4. 索引设计 (100K+ 向量支持)

### 4.1 双层索引策略

```
┌─────────────────────────────────────────────────────────────┐
│                     查询路径                                │
│                                                             │
│   Input Query (768d vector)                                 │
│        │                                                    │
│        ▼                                                    │
│   ┌─────────────┐   快速过滤    ┌─────────────┐            │
│   │ LSH (24bit) │ ───────────▶ │  Candidate  │ ~500个      │
│   │  Hamming    │   < 3ms       │   Set       │             │
│   └─────────────┘               └──────┬──────┘            │
│                                        │                    │
│                                        ▼                    │
│   ┌─────────────┐   精确排序    ┌─────────────┐            │
│   │ HNSW Graph  │ ◀─────────── │  Re-rank    │ Top-10      │
│   │  Cosine     │   ~10ms       │  Cosine     │             │
│   └─────────────┘               └─────────────┘            │
│                                                             │
│   Total: ~15ms for 100K vectors                             │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 LSH 参数 (SimHash 复用)

```javascript
// LSH 配置 - 复用现有 SimHash-64 成果
const LSH_CONFIG = {
  hashFunction: 'simhash-64',     // 已有实现
  numTables: 8,                   // 8 个哈希表
    numHashes: 8,                   // 每个表 8 个哈希函数
  bucketWidth: 4,                 // 桶宽度 (汉明距离 < 4)
  
  // 级联哈希用于去重
  dedupHash: 'simhash+md5',       // 已有 CASCADE 方案
};

// 预期性能
// - 100K 向量 → 8 个表 × ~1.3K 桶 =  manageable
// - 假阳性率: < 0.1% (级联哈希过滤)
```

### 4.3 HNSW 参数

```javascript
const HNSW_CONFIG = {
  M: 16,              // 每个节点最大连接数
  efConstruction: 200, // 构建时搜索深度
  efSearch: 64,       // 查询时搜索深度
  maxLevel: 4,        // 最大层数 (log_16(100K) ≈ 4)
  
  // 内存估算: 100K × 16 × 4B × 2 ≈ 12.8MB (邻居索引)
};
```

---

## 5. 轻量级同步方案

### 5.1 方案 A: WebRTC 局域网 P2P

```
┌─────────────┐                      ┌─────────────┐
│  Device A   │ ◀────WebRTC────▶    │  Device B   │
│  (Source)   │    DataChannel       │  (Target)   │
└──────┬──────┘                      └──────┬──────┘
       │                                    │
       ▼                                    ▼
┌─────────────┐                      ┌─────────────┐
│ 1. Generate │                      │ 1. Receive  │
│    Manifest │                      │    Manifest │
├─────────────┤                      ├─────────────┤
│ 2. Send     │                      │ 2. Compare  │
│    Delta    │◀────────────────────▶│    Local    │
├─────────────┤     chunk hashes     ├─────────────┤
│ 3. Stream   │                      │ 3. Request  │
│    Missing  │◀────────────────────▶│    Missing  │
│    Chunks   │                      │    Chunks   │
└─────────────┘                      └─────────────┘
```

**WebRTC 信令** (可选极简方案):
- 局域网 mDNS 发现: `_hajimi._tcp.local`
- 二维码交换 SDP (无服务器)
- 或手动输入 4 位 PIN 码配对

### 5.2 方案 B: 文件导出 (离线同步)

```
Export Format (.hajimi 文件):
┌────────────────────────────────────────────────────────┐
│ Header                                                 │
│   - Magic: "HajimiExport"                              │
│   - Version: 1                                         │
│   - Timestamp                                          │
│   - Device ID                                          │
├────────────────────────────────────────────────────────┤
│ Manifest (JSON)                                        │
│   - export_scope: "full" | "partial" | "query_result" │
│   - chunk_list: [{hash, size, vector_ids}]             │
│   - vector_range: [start_id, end_id]                   │
├────────────────────────────────────────────────────────┤
│ Data Sections                                          │
│   [1] Chunk Data (tar-like 打包)                       │
│   [2] Vector Snapshot (可选，大)                       │
│   [3] Index Diff (增量)                                │
├────────────────────────────────────────────────────────┤
│ Footer                                                 │
│   - SHA256 校验                                        │
└────────────────────────────────────────────────────────┘

// 使用场景
// - 导出到 U 盘/SD 卡
// - 通过 Telegram/邮件传输
// - 冷备份存档
```

### 5.3 Delta 算法

```javascript
// 基于级联哈希的增量检测
async function generateDelta(localDB, remoteManifest) {
  const delta = {
    added: [],      // 本地没有，远程有
    removed: [],    // 本地有，远程没有  
    modified: [],   // hash 不同 (同一个逻辑 id)
    unchanged: []   // 完全相同
  };
  
  // 使用两级 Bloom Filter 加速比对
  // Level 1: SimHash-64 (快速预过滤)
  // Level 2: MD5-128 (精确确认)
  
  for (const chunk of remoteManifest.chunks) {
    const local = await localDB.getBySimhash(chunk.simhash);
    if (!local) {
      delta.added.push(chunk);
    } else if (local.md5 !== chunk.md5) {
      delta.modified.push({local, remote: chunk});
    } else {
      delta.unchanged.push(chunk);
    }
  }
  
  return delta;
}
```

---

## 6. SQLite Schema

```sql
-- 分片元数据表
CREATE TABLE chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simhash_hi BIGINT NOT NULL,          -- SimHash 高 64bit
    simhash_lo BIGINT NOT NULL,          -- SimHash 低 64bit (预留)
    md5 BLOB NOT NULL,                   -- MD5-128 (级联合成)
    sha256 BLOB,                         -- SHA256-256 (未来)
    size INTEGER NOT NULL,
    storage_path TEXT,                   -- 文件系统路径 (NULL=内联)
    storage_type INTEGER DEFAULT 0,      -- 0=内联, 1=外部文件
    inline_data BLOB,                    -- 小文件内联存储
    created_at INTEGER,                  -- Unix timestamp
    access_count INTEGER DEFAULT 0,      -- 访问计数 (LRU)
    last_access INTEGER                  -- 最后访问时间
);

-- 向量关联表 (多对多)
CREATE TABLE chunk_vectors (
    chunk_id INTEGER REFERENCES chunks(id),
    vector_id INTEGER,
    vector_offset INTEGER,               -- 在 vectors.bin 中的偏移
    PRIMARY KEY (chunk_id, vector_id)
);

-- 标签/索引表
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    color TEXT,
    created_at INTEGER
);

CREATE TABLE chunk_tags (
    chunk_id INTEGER REFERENCES chunks(id),
    tag_id INTEGER REFERENCES tags(id),
    PRIMARY KEY (chunk_id, tag_id)
);

-- 同步状态表
CREATE TABLE sync_peers (
    id INTEGER PRIMARY KEY,
    device_id TEXT UNIQUE NOT NULL,
    device_name TEXT,
    last_sync INTEGER,
    sync_count INTEGER DEFAULT 0
);

-- 索引优化
CREATE INDEX idx_chunks_simhash ON chunks(simhash_hi);
CREATE INDEX idx_chunks_md5 ON chunks(md5);
CREATE INDEX idx_chunks_access ON chunks(last_access); -- for LRU
```

---

## 7. API 设计

```typescript
// 核心接口
interface LocalStorageV3 {
  // 存储
  store(data: Buffer, options?: StoreOptions): Promise<ChunkRef>;
  storeStream(stream: ReadableStream): Promise<ChunkRef>;
  
  // 查询
  queryByVector(vector: Float32Array, k: number): Promise<QueryResult[]>;
  queryByHash(simhash: bigint): Promise<ChunkRef[]>;
  queryByTag(tag: string): Promise<ChunkRef[]>;
  
  // 检索
  get(id: string): Promise<Buffer>;
  getStream(id: string): Promise<ReadableStream>;
  
  // 管理
  delete(id: string): Promise<void>;
  gc(): Promise<GCReport>;           // 垃圾回收
  stats(): Promise<StorageStats>;
  
  // 同步
  sync: SyncManager;
}

interface SyncManager {
  // WebRTC P2P
  createOffer(): Promise<RTCSessionDescription>;
  acceptAnswer(answer: RTCSessionDescription): Promise<void>;
  
  // 文件导出
  export(options: ExportOptions): Promise<Blob>;
  import(data: Blob): Promise<ImportReport>;
  
  // 清单操作
  generateManifest(): Promise<Manifest>;
  compareManifest(remote: Manifest): Promise<Delta>;
  applyDelta(delta: Delta, source: SyncSource): Promise<void>;
}
```

---

## 8. 性能目标

| 指标 | 目标值 | 备注 |
|------|--------|------|
| 冷启动时间 | < 500ms | SQLite + 索引加载 |
| 存储 100K 向量 | < 400MB | float32, 768d |
| 向量查询 (k=10) | < 20ms | LSH + HNSW |
| 分片写入 | > 50MB/s | 批量写入优化 |
| 分片读取 | > 100MB/s | mmap + 缓存 |
| P2P 同步 | 依赖网络 | 局域网 ~50MB/s |
| 文件导出 | > 20MB/s | 压缩后 |

---

## 9. 实现路线图

```
Phase 1: 核心存储 (2周)
├── SQLite schema 实现
├── Chunk 文件格式
└── 基础 CRUD API

Phase 2: 向量索引 (2周)
├── Flat vector storage
├── LSH 索引 (复用 SimHash)
└── HNSW 图构建

Phase 3: 同步机制 (2周)
├── Manifest 生成/比对
├── 文件导出/导入
└── WebRTC P2P 基础

Phase 4: 优化 (1周)
├── 性能调优
├── 缓存策略
└── 压缩/加密

Total: ~7 周
```

---

## 10. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| SQLite 性能瓶颈 | 高 | 批量写入、WAL 模式、定期 VACUUM |
| HNSW 内存占用 | 中 | 分层存储、磁盘持久化、按需加载 |
| WebRTC 兼容性 | 中 | 降级到文件导出方案 |
| 100K+ 向量查询慢 | 高 | LSH 预过滤 + 量化降维 |
| 存储损坏 | 高 | CRC 校验、定期备份、repair 模式 |

---

## 11. 依赖清单

```json
{
  "core": {
    "better-sqlite3": "^9.0.0",      // SQLite 同步 API
    "hnswlib-node": "^2.0.0",        // HNSW 索引
  },
  "sync": {
    "simple-peer": "^9.11.1",        // WebRTC 封装 (可选)
  },
  "crypto": {
    " uses": "Node.js crypto module"  // MD5, SHA256, BLAKE3
  }
}
```

---

> **下一步**: 需要我提供详细实现代码，或者先评审这个设计？
