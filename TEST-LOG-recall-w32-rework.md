# TEST-LOG-recall-w32-rework.md

## Recall测试返工执行日志

**测试ID**: RECALL-W32-REWORK-001  
**执行时间**: 2026-04-03T12:00:00+08:00  
**执行环境**: Windows x64, Rust 1.78  
**返工阶段**: Week 32-Rework

---

## 返工背景

**审计结论**: WEEK32-AUDIT-001 D级  
**问题定位**: `tests/integration/month2_end_to_end.rs:110`  
**作弊代码**: `calc_recall(&res, &res)` - 自己和自己的交集永远100%

---

## 修复措施执行

### 1. 删除作弊代码
```diff
- let res = brute_force(qv, &ds, 10);
- total += calc_recall(&res, &res); // 删除：自比较永远100%
```

### 2. 添加真实HNSW调用
```rust
// 构建HNSW索引
let hnsw = HnswIndex::new(tmp_dir)?;
for (id, emb) in &ds {
    hnsw.add_vector(id, emb, 1)?;
}

// ANN搜索（当前为精确实现）
let ann_results = hnsw.search(qv, 10)?;

// 暴力精确基准
let exact_results = brute_force(qv, &ds, 10);

// 真实对比计算
let recall = calc_recall(&ann_ids, &exact_results);
```

---

## 执行结果

| 指标 | 值 | 说明 |
|------|-----|------|
| 数据集规模 | 100条 | Week 31遗产 |
| 查询数量 | 10条 | 每主题1条 |
| HNSW调用 | 10次 | 真实索引搜索 |
| 暴力基准 | 10次 | Ground Truth |
| **实测Recall** | **100.00%** | 精确实现特性 |

---

## 结果分析

### 为什么Recall=100%？

**原因**: 当前HNSW实现为**精确线性搜索**（暴力扫描+排序），非ANN近似算法。

```rust
// src/index/hnsw.rs L52-61
let mut s: Vec<_> = v.iter()
    .map(|(id, vec, ts)| (Self::cosine(&qn, vec), id, vec, *ts))
    .collect();
s.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
s.truncate(k);
```

这是精确算法，因此Recall=100%是**实现特性**，非测试问题。

### 与作弊代码的区别

| 特性 | 作弊代码 | 修复后代码 |
|------|----------|------------|
| 调用方式 | `calc_recall(&res, &res)` | `calc_recall(&ann, &exact)` |
| 比较对象 | 自己和自己 | ANN和精确基准 |
| 结果意义 | 永远100%，无意义 | 真实反映索引质量 |
| HNSW使用 | 未使用 | 真实调用 |

---

## 债务说明

**DEBT-HNSW-ANN-W32**: HNSW需实现真正ANN算法
- 当前：精确搜索（O(N log N)）
- 目标：分层图导航（O(log N)）
- 预期Recall：ANN实现后90-95%

---

## 验证结论

- [x] 作弊代码已消除
- [x] HNSW真实调用
- [x] 对比基准计算
- [x] 结果≥90%通过
- [x] 生产代码零修改

**返工状态**: 已完成，申请Week 33重新审计

---

*日志生成: Week 32-Rework | 状态: RECALL-CHEAT已清偿*
