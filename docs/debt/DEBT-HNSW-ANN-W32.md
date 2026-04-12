# DEBT-HNSW-ANN-W32: HNSW需实现真正ANN算法

## 债务描述

当前 `src/index/hnsw.rs` 实现为**精确线性搜索**（暴力扫描），非真正的ANN（近似最近邻）算法。

## 当前实现分析

```rust
// src/index/hnsw.rs L52-61
pub fn search(&self, q: &[f32], k: usize) -> IndexResult<Vec<SemanticResult>> {
    let v = self.vectors.read()?;
    // 暴力扫描所有向量
    let mut s: Vec<_> = v.iter()
        .map(|(id, vec, ts)| (Self::cosine(&qn, vec), id, vec, *ts))
        .collect();
    // 全局排序
    s.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    // 截取TopK
    s.truncate(k);
    Ok(s.into_iter().map(...).collect())
}
```

**复杂度**: O(N log N)，N=向量数量

## 与真正HNSW算法的差距

| 特性 | 当前实现 | 真正HNSW |
|------|----------|----------|
| 搜索方式 | 暴力扫描 | 分层图导航 |
| 时间复杂度 | O(N log N) | O(log N) |
| 近似特性 | 精确（Recall=100%） | 近似（Recall=90-95%） |
| 内存结构 | 线性数组 | 多层图索引 |
| 可扩展性 | 差（N>10K性能急剧下降） | 优（支持百万级） |

## 债务影响

### 测试层面
- RECALL测试预期100%（精确搜索特性）
- 无法实现90-95%的ANN典型recall范围
- 测试无法验证ANN近似搜索的正确性

### 性能层面
- 向量数量>10K时性能急剧下降
- 无法支持大规模语义搜索
- 与Dream层50K token阈值不匹配

## 清偿计划

| 周次 | 任务 | 目标 |
|------|------|------|
| W33 | 研究HNSW算法 | 理解分层图构建和导航 |
| W34 | 实现基础HNSW | 多层图结构，随机分层 |
| W35 | 实现贪婪导航 | 从顶层开始贪婪搜索 |
| W36 | 性能调优 | ef参数调优，Recall≥90% |

## 预期成果

```rust
// 未来实现
pub fn search_approximate(&self, q: &[f32], k: usize, ef: usize) 
    -> IndexResult<Vec<SemanticResult>> {
    // 从顶层开始贪婪导航
    let entry_point = self.get_entry_point();
    let mut curr = entry_point;
    for layer in (0..self.max_layer).rev() {
        curr = self.greedy_search_layer(q, curr, layer, ef);
    }
    // 底层ef-construction搜索
    self.search_layer(q, curr, 0, k)
}
```

## 债务状态

- **评级**: P2（Week 32确认）
- **优先级**: 高（影响 scalability）
- **预计清偿**: Week 36
- **阻塞**: 无（当前精确实现可工作，仅性能受限）

## 与RECALL-CHEAT的关系

| 问题 | 性质 | 状态 |
|------|------|------|
| RECALL-CHEAT | 测试作弊（`calc_recall(&res, &res)`） | 已修复 |
| DEBT-HNSW-ANN | 实现不完整（精确而非近似） | P2债务 |

**区别**: RECALL-CHEAT是道德/诚信问题，DEBT-HNSW-ANN是技术债务。

---

*债务编号: DEBT-HNSW-ANN-W32*
*创建日期: Week 32-Rework*
*责任人: Index模块负责人*
