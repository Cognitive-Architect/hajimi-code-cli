# Week 37 空间索引与批量计算架构

> **版本**: v1.0.0 | **日期**: 2026-04-10 | **关联**: DEBT-PERF-INSERT-W36, DEBT-HNSW-RECALL-W35

---

## 1. 问题分析

### O(N²) 瓶颈定量分析

当前实现（Line 303-304）每次插入执行全表扫描：

```rust
let mut stmt = tx.prepare("SELECT id, vector_json FROM hnsw_nodes WHERE level = ?1")?;
let existing: Vec<(String, String)> = stmt.query_map([&level], |row| {
    Ok((row.get(0)?, row.get(1)?))
})?.collect::<Result<Vec<_>, _>>()?;
```

**复杂度计算**：
- 500 条插入 × 平均 250 条现有节点 = **125K 次距离计算**
- 单次距离计算: 384 维欧氏距离 = 384 次浮点运算
- 总运算量: **48M 次浮点运算** + JSON 解析开销

**瓶颈定位**：单条 INSERT 触发 N 次距离计算（N=当前层节点数），无批量读取，无内存缓存。

---

## 2. 方案对比

### R-tree vs 网格 vs 批量计算 决策表

| 方案 | 适用维度 | 时间复杂度 | 空间复杂度 | 可行性 | 决策 |
|:---|:---:|:---:|:---:|:---:|:---:|
| R-tree | 2-5 维 | O(log N) | O(N) | ❌ 不可行 | 384 维高维诅咒，索引失效 |
| 网格索引 | 2-10 维 | O(1) 均摊 | O(k^d) | ❌ 不可行 | 384 维网格单元指数爆炸 |
| KD-Tree | <20 维 | O(log N) | O(N) | ❌ 不可行 | 高维下退化为线性扫描 |
| LSH | 任意维 | O(1) | O(N×L) | ⚠️ 复杂 | 哈希冲突影响 Recall |
| **批量计算** | 任意维 | O(N) 批处理 | O(N) | ✅ **推荐** | 内存计算 + LRU 缓存热点 |

**决策依据**：384 维向量的高维诅咒（dimension curse）使传统空间索引失效。R-tree 在 384 维下退化为线性扫描，网格索引面临单元数量爆炸（k^384 不可行）。批量计算是唯一兼顾性能与实现可行性的路径。

---

## 3. 技术约束

| 约束项 | 要求 | 影响 | 验证 |
|:---|:---|:---|:---|
| 零 unsafe | `#![deny(unsafe_code)]` | 禁用 SIMD intrinsics，纯 Rust 实现 | `grep -r "unsafe" src/memory/src/hnsw.rs` |
| SQLite 嵌入式 | 无外部服务依赖 | 拒绝 PostGIS/ElasticSearch/Milvus | 依赖检查 |
| 384 维 | `EMBEDDING_DIM = 384_usize` | R-tree vs 网格均不适用 | 编译期断言 |
| 冻结区域 | Line 1-245/270-344/442+ | API 签名不可变更，仅内部优化 | 代码审计 |
| Recall ≥90% | DEBT-HNSW-RECALL-W35 | 优化不得降低搜索质量 | 回归测试 |

---

## 4. 架构设计

### 4.1 批量计算 API

```rust
/// 批量距离计算核心 API - 纯内存计算，零 SQLite IO
pub fn batch_compute_distances(
    query: [f32; EMBEDDING_DIM],
    candidates: &[(String, [f32; EMBEDDING_DIM])],
) -> Vec<(String, f32)> {
    candidates.iter()
        .map(|(id, vec)| (id.clone(), euclidean_distance(query, *vec)))
        .collect()
}

/// 欧氏距离计算 - 循环展开友好
fn euclidean_distance(a: [f32; EMBEDDING_DIM], b: [f32; EMBEDDING_DIM]) -> f32 {
    let mut sum = 0.0_f32;
    for i in 0..EMBEDDING_DIM {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}
```

**SQLite 批量查询优化**：`WHERE id IN (...)` 单次查询替代 N 次往返，chunk_size=500 避免参数溢出（SQLite 限制 999 占位符）。

### 4.2 内存 LRU 缓存热点

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

/// 分层缓存热点层 - LayerCache 结构
pub struct LayerCache {
    /// 按层缓存向量数据，避免重复 SQLite 读取
    cache: LruCache<u8, Vec<(String, [f32; EMBEDDING_DIM])>>,
    capacity: usize,
}

impl LayerCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            capacity,
        }
    }
    
    /// 获取层数据 - 缓存未命中时从 SQLite 加载
    pub fn get_layer(&mut self, level: u8) -> Option<&Vec<(String, [f32; EMBEDDING_DIM])>> {
        self.cache.get(&level)
    }
    
    /// 更新层数据 - 插入后同步更新
    pub fn put_layer(&mut self, level: u8, data: Vec<(String, [f32; EMBEDDING_DIM])>) {
        self.cache.put(level, data);
    }
}
```

**缓存策略**：Top Level（高层）节点数量少（<100），常驻内存；热点层 LRU 淘汰；内存上限约 15MB（10K 节点 × 384 维 × 4B）。

---

## 5. SQLite 优化

### 5.1 WHERE id IN (...) 批量查询

**优化原理**：
- 单次 SQLite 查询替代 N 次往返
- 减少解析锁持有时间
- 利用 SQLite 查询缓存

### 5.2 SpatiaLite 评估（不适合）

| 特性 | 评估 | 结论 |
|:---|:---|:---|
| RTREE 模块 | SQLite 原生支持 | 技术可行 |
| 384 维支持 | RTREE 最大 5 维 | ❌ 不可行 |
| 维度诅咒 | 5 维以上索引效率指数下降 | ❌ 不适用 |
| 嵌入式部署 | 需编译扩展，引入 C 依赖 | ❌ 违反约束 |
| **最终结论** | **SpatiaLite RTREE 不适合 384 维向量** | 放弃 |

---

## 6. 性能目标

| 指标 | 当前基线 | Week 37 目标 | 提升 | 验证方法 |
|:---|:---:|:---:|:---:|:---|
| 插入速度 | ~20 条/s | ≥100 条/s | **+5x** | 1000 条批量插入计时 |
| Recall@10 | 基准值 | ≥90% | 维持 | SIFT/GloVe 测试集 |
| 10K P99 查询 | >100 ms | <10 ms | **10x** | `search_ann_with_ef` 基准 |
| 内存占用 | N/A | <50 MB | 可接受 | RSS 监控 |

---

## 7. 风险评估

| 风险项 | 概率 | 影响 | 缓解措施 |
|:---|:---:|:---:|:---|
| 高维索引结构失效 | 已确认 | 高 | 放弃 R-tree，专注批量计算 |
| 缓存一致性 bug | 中 | 高 | 写操作同步更新缓存，事务回滚清空 |
| 内存膨胀 OOM | 低 | 中 | LRU 容量硬限制，RSS 监控告警 |
| API 向后兼容破坏 | 低 | 高 | 冻结区域保护，仅内部实现变更 |
| 批量查询参数溢出 | 低 | 中 | chunk_size=500 分批处理 |
| Recall 下降 | 中 | 高 | 优化前后对比测试，<90% 回滚 |

---

## 8. 向后兼容

### 8.1 冻结区域保护

```
Line 1-245:    冻结（数据结构/常量/错误定义）
Line 246-304:  解冻（insert_with_levels 优化区）
Line 305-344:  冻结（辅助函数 - search_layer/get_dist）
Line 345-441:  解冻（search_ann_with_ef 优化区）
Line 442+:     冻结（新增实现 - LayerCache/批量 API）
```

### 8.2 API 不变承诺

- `insert_with_levels(&mut self, id: &str, vector: [f32; EMBEDDING_DIM])` 签名冻结
- `search_ann_with_ef(&self, query, k, ef_search)` 签名冻结
- 仅内部实现引入 `batch_compute_distances` + `LayerCache`
- 零 public API 变更，用户无感知升级

---

## 9. 实施路线

| 阶段 | 任务 | 交付物 | 验收标准 |
|:---|:---|:---|:---|
| 1 | 批量查询实现 | `batch_fetch_candidates(level, ids)` | 单次查询替代 N 次 |
| 2 | 批量距离计算 | `batch_compute_distances` API | 零逐条 JSON 解析 |
| 3 | LRU 缓存层 | `LayerCache` 结构体 | 缓存命中率 ≥60% |
| 4 | 集成测试 | 回归测试报告 | Recall ≥90% |
| 5 | 性能调优 | 基准测试报告 | P99 <10ms @10K |

---

*DEBT-PERF-INSERT-W36 (P1) | DEBT-HNSW-RECALL-W35 (P1) | 零 unsafe 承诺 | 384 维高维诅咒*
