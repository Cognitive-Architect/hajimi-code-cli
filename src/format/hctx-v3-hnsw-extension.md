# HCTX v3 HNSW 扩展规范

> 版本: 1.0  
> 日期: 2026-02-25  
> 状态: Phase 2 实现

---

## 1. 概述

本文档描述 HCTX (Hajimi Chunk) v3 格式的 HNSW 扩展，用于在现有 Chunk 文件中嵌入 HNSW 向量索引元数据。

**设计目标：**
- 向后兼容：v3 无 HNSW → v3 有 HNSW 平滑迁移
- 最小侵入：不改变现有 128 字节 header 结构
- 可选存储：HNSW 数据存储在 metadata JSON 或独立文件

---

## 2. 存储方案

### 2.1 方案对比

| 方案 | 优点 | 缺点 | 选用 |
|:---|:---|:---|:---:|
| **独立文件** (.hnsw) | 灵活、索引可重建 | 多文件管理复杂 | ✅ 主方案 |
| **嵌入 Metadata** | 单文件、原子性 | Metadata 膨胀 | 备选 |
| **SQLite 存储** | 事务、查询 | 依赖存储层 | Phase 3 |

### 2.2 独立文件方案（主方案）

```
~/.hajimi/storage/v3/
├── meta/
│   ├── shard_00.db      # SQLite 元数据
│   ├── ...
│   └── shard_15.db
├── chunks/              # Chunk 文件
│   └── ...
└── hnsw/                # HNSW 索引（新增）
    ├── hnsw-index-0.json    # 分片 0 索引
    ├── hnsw-wal-0.log       # 分片 0 WAL
    ├── ...
    ├── hnsw-index-15.json
    └── backups/             # 备份目录
```

---

## 3. HNSW 索引文件格式

### 3.1 JSON 格式 (.json)

```json
{
  "version": 1,
  "shardId": 0,
  "timestamp": 1708888888888,
  "metadata": {
    "vectorCount": 100000,
    "encodingMethod": "hadamard",
    "outputDimension": 128,
    "distanceMetric": "l2",
    "hnswConfig": {
      "M": 16,
      "efConstruction": 200,
      "efSearch": 64
    }
  },
  "checksum": "sha256:abcd1234...",
  "index": {
    "config": { ... },
    "maxLevel": 5,
    "entryPoint": 12345,
    "elementCount": 100000,
    "nodes": [
      {
        "id": 0,
        "vector": [0.1, 0.2, ...],
        "level": 3,
        "connections": [[1, 2], [3, 4], [5], [], []],
        "deleted": false
      }
    ]
  }
}
```

### 3.2 字段说明

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `version` | number | 文件格式版本 |
| `shardId` | number | 所属分片 ID |
| `timestamp` | number | 保存时间戳 |
| `metadata` | object | 索引元数据 |
| `checksum` | string | SHA256 校验和 |
| `index` | object | HNSW 索引数据 |

### 3.3 Node 结构

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `id` | number | 节点 ID（对应文档 ID）|
| `vector` | array/bigint | 向量数据（Float32Array 或 SimHash）|
| `level` | number | 节点所在最高层 |
| `connections` | array | 每层连接 [level][neighborIds] |
| `deleted` | boolean | 软删除标记 |

---

## 4. Chunk Metadata 扩展

在 Chunk 的 metadata JSON 中，可选添加 HNSW 相关字段：

```json
{
  "size": 1024,
  "ctime": 1708888888888,
  "simhash": "a1b2c3d4e5f6...",
  "hnsw": {
    "indexed": true,
    "vectorId": 12345,
    "shardId": 3
  }
}
```

**字段说明：**

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `indexed` | boolean | 是否已加入 HNSW 索引 |
| `vectorId` | number | 在 HNSW 中的 ID |
| `shardId` | number | 所属分片 |

---

## 5. WAL 日志格式

### 5.1 日志文件 (.log)

每行一条 JSON 记录：

```json
{"seq":1,"time":1708888888888,"op":"insert","data":{"id":0,"vector":{"type":"Float32Array","value":[0.1,0.2,...]}}}
{"seq":2,"time":1708888888889,"op":"delete","data":{"id":5}}
```

### 5.2 操作类型

| 操作 | 说明 |
|:---|:---|
| `insert` | 插入新向量 |
| `delete` | 删除向量 |

---

## 6. 兼容性

### 6.1 v3 无 HNSW → v3 有 HNSW

1. 检测到无 HNSW 索引时，自动初始化空索引
2. 已有 Chunk 文件逐步加入索引（后台任务）
3. 检索时优先使用 HNSW，缺失的文档回退到 LSH

### 6.2 降级场景

| 场景 | 行为 |
|:---|:---|
| HNSW 文件损坏 | 自动重建索引（从 Chunk 重新导入）|
| HNSW 版本不兼容 | 清除并重建 |
| 内存不足 | 降级到 LSH，保留 HNSW 文件 |

---

## 7. 性能考虑

### 7.1 加载优化

- 使用 `LazyShardLoader` 按需加载分片
- 预加载相邻分片
- JSON 解析使用流式处理（大文件）

### 7.2 写入优化

- 批量写入 + WAL
- 异步刷盘
- 原子替换（write + rename）

### 7.3 存储大小估算

| 参数 | 值 |
|:---|:---|
| 100K 向量 × 128 维 | ~50MB JSON |
| 压缩后 | ~15MB |
| 平均每个向量 | ~500 bytes |

---

## 8. 安全与校验

### 8.1 数据完整性

- SHA256 校验和
- WAL 保证原子性
- 定期备份

### 8.2 异常处理

| 异常 | 处理 |
|:---|:---|
| 校验失败 | 尝试备份恢复 |
| WAL 损坏 | 跳过损坏条目 |
| 磁盘满 | 降级到 LSH，告警 |

---

## 9. API 参考

### 9.1 保存索引

```javascript
const { HNSWPersistence } = require('./hnsw-persistence');

const persistence = new HNSWPersistence({
  basePath: '~/.hajimi/storage/v3/hnsw',
  shardId: 0
});

await persistence.save(hnswIndex, { vectorCount: 100000 });
```

### 9.2 加载索引

```javascript
const result = await persistence.load();
if (result) {
  const { index, metadata } = result;
  // use index...
}
```

### 9.3 WAL 记录

```javascript
await persistence.logInsert(id, vector);
await persistence.logDelete(id);
await persistence.flush();  // 强制刷盘
```

---

## 10. 变更日志

| 版本 | 日期 | 变更 |
|:---|:---|:---|
| 1.0 | 2026-02-25 | 初始版本，Phase 2 实现 |

---

> **关联文档：**
> - `../vector/hnsw-persistence.js` - 持久化实现
> - `../storage/chunk.js` - Chunk 存储
