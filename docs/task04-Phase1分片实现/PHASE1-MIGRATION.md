# PHASE1-MIGRATION: 分片迁移与初始化文档

> **工单**: P1-04/05  
> **日期**: 2026-02-22  
> **状态**: ✅ 已完成

---

## 1. Schema 设计

### 1.1 核心表：chunks

```sql
CREATE TABLE chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simhash_hi BIGINT NOT NULL,          -- 路由键（高64bit）
    simhash_lo BIGINT NOT NULL,          -- 完整标识（低64bit）
    md5 BLOB NOT NULL,                   -- 数据哈希
    size INTEGER NOT NULL,               -- 数据大小
    storage_path TEXT,                   -- 外部存储路径
    storage_type INTEGER DEFAULT 0,      -- 0=内联, 1=外部
    inline_data BLOB,                    -- 小文件内联
    created_at INTEGER,                  -- 创建时间
    last_access INTEGER,                 -- LRU时间
    access_count INTEGER DEFAULT 0,      -- 访问计数
    tags TEXT                            -- JSON标签
);
```

### 1.2 索引

```sql
-- 核心索引：SimHash查询
CREATE INDEX idx_chunks_simhash_hi ON chunks(simhash_hi);

-- 唯一索引：防重复
CREATE UNIQUE INDEX idx_chunks_simhash_full ON chunks(simhash_hi, simhash_lo);

-- LRU索引
CREATE INDEX idx_chunks_lru ON chunks(last_access);
```

---

## 2. 迁移工具

### 2.1 CLI 用法

```bash
# 初始化16个分片
node src/storage/migrate.js --init

# 检查分片状态
node src/storage/migrate.js --check

# 删除所有分片（危险）
node src/storage/migrate.js --drop
```

### 2.2 API

```javascript
const { MigrationManager } = require('./src/storage/migrate');

const manager = new MigrationManager();

// 初始化所有分片
const results = await manager.initAllShards();
// → { created: [0,1,2...], existing: [], errors: [] }

// 检查状态
const statuses = await manager.checkShards();
// → [{ shardId, exists, version, needsUpgrade }]

// 升级
await manager.upgradeShards(1, 3);
```

---

## 3. 自测结果

### 3.1 MIG-001: 16个分片创建 ✅

```bash
node src/storage/migrate.js --init
# → 创建: 16个
```

### 3.2 MIG-002: Schema正确 ✅

每个分片包含：
- chunks 表
- chunk_vectors 表
- sync_peers 表
- 4个索引
- 2个触发器

### 3.3 MIG-003: shard_meta初始化 ✅

```sql
SELECT * FROM shard_meta;
-- version: 3
-- shard_id: {对应ID}
-- chunk_count: 0
```

### 3.4 MIG-004: 幂等性 ✅

重复执行`--init`不报错，已存在的分片跳过。

### 3.5 MIG-005: 版本升级 ✅

支持 v1→v2→v3 平滑升级。

---

## 4. P4检查表

| 检查点 | 覆盖 | 用例ID | 状态 |
|--------|------|--------|------|
| CF | ✅ | MIG-001,002 | 通过 |
| RG | ✅ | MIG-004 | 通过 |
| NG | ✅ | MIG-005 | 通过 |
| UX | ✅ | MIG-003 | 通过 |
| E2E | ✅ | MIG-001+002 | 通过 |
| High | ✅ | MIG-004 | 通过 |
| 字段完整性 | ✅ | 全部5项 | 通过 |
| 需求映射 | ✅ | P1-04 | 通过 |
| 执行结果 | ✅ | 全部通过 | 通过 |
| 范围边界 | ✅ | 仅迁移工具 | 通过 |

**P4检查**: 10/10 ✅

---

## 5. 交付物

| 文件 | 路径 | 说明 |
|------|------|------|
| Schema | `src/storage/schema.sql` | 分片内Schema |
| 迁移工具 | `src/storage/migrate.js` | MigrationManager |
| 迁移文档 | `docs/PHASE1-MIGRATION.md` | 本文档 |

---

**工单状态**: A级通过 ✅
