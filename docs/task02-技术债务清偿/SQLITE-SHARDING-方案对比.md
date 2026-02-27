# DEBT-SQLITE-001: SQLite 分片架构方案对比

> **工单**: 03/04  
> **优先级**: P1-高  
> **状态**: ✅ 已完成  
> **日期**: 2026-02-22  
> **债务来源**: local-storage-v3-design.md Schema 设计未考虑分片

---

## 1. 问题陈述

### 1.1 当前设计风险

**原设计**: 单 SQLite 数据库 (`metadata.db`)

**潜在问题**:
- 100K+ 分片的元数据集中存储 → 单库文件过大
- 并发读写锁竞争 → 性能瓶颈
- 备份粒度粗 → 全量备份耗时

### 1.2 预研目标

对比 3 种分片方案，选定 **Phase 1 实施方案**。

---

## 2. 方案对比

### 2.1 方案 A: Hash 前缀水平分片 (推荐)

#### 设计

```
~/.hajimi/storage/v3/meta/
├── shard_00.db   # hash_prefix 00-0F
├── shard_01.db   # hash_prefix 10-1F
├── ...
└── shard_15.db   # hash_prefix F0-FF

共 16 个分片库，按 SimHash-64 前缀分配
```

#### 分片规则

```javascript
function getShardId(simhash_hi) {
  // 取高 8bit 作为分片键
  const prefix = Number((simhash_hi >> 56n) & 0xFFn);
  return prefix % 16;  // 0-15
}

function getShardPath(shardId) {
  return `meta/shard_${shardId.toString(16).padStart(2, '0')}.db`;
}
```

#### Schema (每分片)

```sql
-- 分片内独立 schema
CREATE TABLE chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simhash_hi BIGINT NOT NULL,
    simhash_lo BIGINT NOT NULL,
    md5 BLOB NOT NULL,
    size INTEGER NOT NULL,
    storage_path TEXT,
    created_at INTEGER
);

CREATE INDEX idx_simhash ON chunks(simhash_hi);

-- 分片元信息
CREATE TABLE shard_meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
INSERT INTO shard_meta VALUES ('shard_id', '00');
INSERT INTO shard_meta VALUES ('chunk_count', '0');
```

#### 优缺点

| 维度 | 评估 |
|------|------|
| **文件锁竞争** | ✅ 极低 - 16 个独立锁，天然并发 |
| **并发读写** | ✅ 优秀 - 可并行访问不同分片 |
| **备份复杂度** | ⚠️ 中等 - 16 个文件，需批量操作 |
| **实现工时** | ⚠️ 中等 - 需路由层 + 连接池 |
| **数据分布** | ✅ 均匀 - hash 随机性好 |
| **跨分片查询** | ⚠️ 需聚合 - 但 SimHash 查询天然单分片 |

#### 性能预估

```
100K 分片 / 16 = 每分片 ~6.25K 记录
单库大小: ~50MB (估算)
并发能力: 16× 单库能力
```

---

### 2.2 方案 B: 时间戳分片

#### 设计

```
~/.hajimi/storage/v3/meta/
├── 2026-02.db    # 2026年2月数据
├── 2026-03.db    # 2026年3月数据
└── current.db    # 当前写入 (软链接)
```

#### 分片规则

```javascript
function getShardByTimestamp(timestamp) {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  return `meta/${year}-${month}.db`;
}

// 写操作总是写入 current.db (软链接到当月)
// 读操作根据 chunk 时间戳路由
```

#### 优缺点

| 维度 | 评估 |
|------|------|
| **文件锁竞争** | ⚠️ 高 - 当前月库热点集中 |
| **并发读写** | ⚠️ 中等 - 历史库只读，当前库竞争 |
| **备份复杂度** | ✅ 简单 - 历史库 immutable，只备份当前 |
| **实现工时** | ✅ 低 - 简单时间路由 |
| **数据分布** | ⚠️ 不均 - 新旧数据不平衡 |
| **按时间查询** | ✅ 高效 - 自然分区裁剪 |
| **按 hash 查询** | ❌ 低效 - 需遍历所有库 |

#### 适用场景

- 日志型数据 (时间范围查询为主)
- 冷热数据分离需求强烈
- **不适用**: Hajimi 以 SimHash 查询为主

---

### 2.3 方案 C: 单库 + WAL 优化 (基准对照)

#### 设计

保持原设计，优化配置:

```javascript
const SQLITE_CONFIG = {
  // WAL 模式优化
  journal_mode: 'WAL',
  synchronous: 'NORMAL',
  cache_size: -64000,        // 64MB 缓存
  temp_store: 'MEMORY',
  mmap_size: 268435456,      // 256MB mmap
  
  // 连接池
  maxConnections: 4,
  
  // 自动维护
  auto_vacuum: 'INCREMENTAL'
};
```

#### 优缺点

| 维度 | 评估 |
|------|------|
| **文件锁竞争** | ⚠️ 高 - 单锁瓶颈 |
| **并发读写** | ⚠️ WAL 可缓解读，写仍串行 |
| **备份复杂度** | ✅ 最低 - 单文件复制 |
| **实现工时** | ✅ 最低 - 无需改动 |
| **数据分布** | ✅ N/A - 单库无分布问题 |
| **长期扩展** | ❌ 差 - 100万+ 记录后性能下降 |

---

## 3. 详细对比矩阵

| 维度 | 权重 | 方案 A (Hash) | 方案 B (时间) | 方案 C (单库) |
|------|------|---------------|---------------|---------------|
| **文件锁竞争** | 20% | 9/10 (16锁) | 4/10 (热点) | 3/10 (单锁) |
| **并发读写性能** | 20% | 9/10 | 5/10 | 4/10 |
| **按 Hash 查询** | 15% | 10/10 | 3/10 | 9/10 |
| **备份复杂度** | 15% | 6/10 | 8/10 | 10/10 |
| **实现工时** | 15% | 6/10 | 7/10 | 10/10 |
| **长期扩展性** | 10% | 9/10 | 6/10 | 4/10 |
| **数据分布均匀** | 5% | 10/10 | 5/10 | N/A |
| **总分** | 100% | **8.25** | **5.35** | **6.35** |

### 3.1 对比结论

| 方案 | 总分 | 推荐场景 |
|------|------|----------|
| **A (Hash)** | **8.25** | ✅ ** Hajimi 首选** - SimHash 查询友好 |
| C (单库) | 6.35 | 原型/MVP 阶段 |
| B (时间) | 5.35 | 日志系统，非 Hajimi 场景 |

---

## 4. 推荐方案: 方案 A (Hash 前缀分片)

### 4.1 推荐理由

1. **查询模式匹配**: Hajimi 以 SimHash 查询为主，Hash 分片保证单次查询只访问一个分片
2. **并发性能**: 16 个独立锁，支持真并行读写
3. **扩展性**: 未来可扩展至 64/256 分片
4. **数据均匀**: SimHash 均匀分布，避免热点

### 4.2 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Router Layer                             │
│              (根据 SimHash 路由到分片)                       │
└─────────────────────────────────────────────────────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
     ┌────────────┐  ┌────────────┐  ┌────────────┐
     │ shard_00.db│  │ shard_01.db│  │ shard_0F.db│
     │ (6.25K)    │  │ (6.25K)    │  │ (6.25K)    │
     ├────────────┤  ├────────────┤  ├────────────┤
     │ chunks     │  │ chunks     │  │ chunks     │
     │ vectors    │  │ vectors    │  │ vectors    │
     │ tags       │  │ tags       │  │ tags       │
     └────────────┘  └────────────┘  └────────────┘
```

### 4.3 连接池设计

```javascript
class ShardConnectionPool {
  constructor(shardCount = 16) {
    this.pools = Array(shardCount).fill(null).map(() => ({
      read: [],   // 读连接池
      write: null // 独占写连接
    }));
  }
  
  async query(simhash_hi, sql, params) {
    const shardId = this.getShardId(simhash_hi);
    const conn = this.pools[shardId].read.pop() || await this.createConnection(shardId);
    try {
      return await conn.query(sql, params);
    } finally {
      this.pools[shardId].read.push(conn);
    }
  }
  
  async write(sql, params) {
    // 广播写或根据参数路由
    const shardId = params.simhash_hi ? this.getShardId(params.simhash_hi) : 'all';
    // ...
  }
}
```

---

## 5. Phase 1 实施计划

### 5.1 任务拆分

```
Week 1: 基础设施
├── Day 1-2: ShardRouter 实现
│   └── 分片路由算法 + 连接管理
├── Day 3-4: Schema 迁移工具
│   └── 单库 → 多库迁移脚本
└── Day 5: 单元测试
    └── 路由准确性 + 连接池稳定性

Week 2: 集成与优化
├── Day 1-2: API 层适配
│   └── StorageV3 API 接入分片层
├── Day 3: 性能基准测试
│   └── 对比单库 vs 16分片性能
├── Day 4: 备份工具升级
│   └── 支持多库批量备份
└── Day 5: 文档 + 代码审查
```

### 5.2 验收标准

| 检查项 | 目标 | 验收方式 |
|--------|------|----------|
| 路由准确性 | 100% | 单元测试覆盖所有分片 |
| 并发性能 | >3× 单库 | 基准测试 (16 并发写) |
| 故障恢复 | 单分片损坏不影响其他 | 模拟故障测试 |
| 备份完整性 | 16 库数据一致性 | 校验和对比 |

---

## 6. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 分片不均 | 中 | 监控各分片大小，支持 rebalance |
| 跨分片事务 | 中 | 避免跨分片事务，使用 Saga 模式 |
| 连接数爆炸 | 中 | 每分片限制连接数，使用连接池 |
| 分片元数据同步 | 低 | 各分片独立，无需全局协调 |

---

## 7. 自测报告

### 7.1 SQL-001-DESIGN-001: 3 方案完整性 ✅

| 方案 | 描述 | 状态 |
|------|------|------|
| A | Hash 前缀分片 (16库) | ✅ |
| B | 时间戳分片 (按月) | ✅ |
| C | 单库 + WAL 优化 | ✅ |

### 7.2 SQL-001-COMP-001: 对比维度无遗漏 ✅

| 维度 | 已评估 |
|------|--------|
| 文件锁竞争 | ✅ |
| 并发读写性能 | ✅ |
| 备份复杂度 | ✅ |
| 实现工时 | ✅ |
| 长期扩展性 | ✅ |

### 7.3 SQL-001-DECISION-001: 明确推荐方案 ✅

- **推荐方案**: A (Hash 前缀分片)
- **理由**: 查询模式匹配 + 并发性能最优
- **总分**: 8.25/10 (领先单库 1.9 分)

---

## 8. 交付物清单

| 交付物 | 路径 | 状态 |
|--------|------|------|
| 方案对比文档 | `docs/SQLITE-SHARDING-方案对比.md` | ✅ |
| 推荐方案声明 | 方案 A (Hash 前缀分片) | ✅ |
| Phase 1 实施计划 | Week 1-2 任务拆分 | ✅ |
| 自测通过率 | 3/3 ✅ | ✅ |

---

## 9. 结论

| 检查项 | 结果 |
|--------|------|
| 3 方案完整性 | ✅ |
| 6 维度对比 | ✅ |
| 明确推荐方案 | ✅ 方案 A |
| Phase 1 计划 | ✅ |
| **自测通过率** | **3/3 ✅** |

> **债务清偿**: SQLite 分片架构已从"未考虑"升级为"方案 A 推荐 + 实施计划"。

---

**下一步**: 工单 03 自测全绿 (3/3 ✅)，可开工 **工单 04/04: DEBT-WEBRTC-001 + ROADMAP-001**。
