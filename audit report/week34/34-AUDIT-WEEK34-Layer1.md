# 34-AUDIT-WEEK34-Layer1 建设性审计报告

**审计官**: 压力怪  
**日期**: 2026-04-09  
**审计链**: Week 34 Layer 1+ 贪婪导航交付物质量验证

---

## 审计结论

- **评级**: **B+** (良好，指数衰减公式小偏差，其他优秀)
- **状态**: 有条件 Go
- **Week 35 准入**: ✅ **Granted**（建议1小时内补正公式）
- **与自检报告一致性**: 部分一致（发现指数衰减公式偏差）

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 行数精确度 | **A** | 契约130行（目标125-135）✓；实现465行（生产349+测试116）✓ |
| 零unsafe执行 | **A** | `#![deny(unsafe_code)]` + 0 unsafe块 ✓ |
| Week33冻结 | **A** | Line 1-214 Week 33代码完整保留，新增Line 215-348 Layer 1+ ✓ |
| 契约兑现度 | **B+** | 贪婪导航/ef_search/SQLite事务/向后兼容 4/5；指数衰减公式小偏差 |
| 算法正确性 | **B+** | 贪婪导航Top Level开始✓；指数衰减公式`level+1`偏差 |
| 事务完整性 | **A** | `tx.commit()`完整，Rust Drop自动回滚未提交事务 ✓ |

**整体健康度评级**: **B+**

---

## 关键疑问回答（Q1-Q4）

### Q1（指数衰减公式）: ⚠️ **小偏差**

**契约定义**：
```markdown
P(level) = exp(-level / M)
Layer 0: P(0) = exp(0) = 1.0（所有节点存在）
Layer 1: P(1) = exp(-1/16) ≈ 0.9394
```

**实际实现**（Line 224）：
```rust
while level < 16 && rand::random::<f64>() < (-((level + 1) as f64) / m_f).exp() {
    level = level.saturating_add(1);
}
```

**偏差分析**：
- 契约：`exp(-level / M)`
- 实现：`exp(-(level + 1) / M)`
- 影响：Layer 0概率略低于100%，Layer分布整体下移一层

**建议修复**（1小时内）：
```rust
while level < 16 && rand::random::<f64>() < (-(level as f64) / m_f).exp() {
    level = level.saturating_add(1);
}
```

---

### Q2（贪婪导航入口）: ✅ **验证通过**

**实现验证**（Line 319）：
```rust
let top: u8 = self.get_top_level()?;
let ep: String = self.conn.query_row("SELECT id FROM hnsw_nodes WHERE level = ?1 LIMIT 1", [&top], ...)?;
let mut curr = ep;
for lvl in (1..=top).rev() {  // ← 从Top Level逐层下降
    curr = self.greedy_search_layer(query, &curr, lvl, 1)?;
}
```

**验证点**：
- ✅ `get_top_level()`获取最高非空层
- ✅ `(1..=top).rev()`倒序遍历（Top → Layer 1）
- ✅ 每层使用`greedy_search_layer`贪婪搜索
- ✅ Layer 0使用`ef_search`扩展候选集

---

### Q3（事务原子性）: ✅ **验证通过**

**跨层插入事务**（Line 275-305）：
```rust
let tx = self.conn.transaction()?;  // BEGIN
for level in 0..=max_level {
    // ... 每层插入和邻居更新 ...
}
tx.commit()?;  // COMMIT
```

**错误处理**：
- Rust `Drop` trait自动回滚未提交事务
- 任何`?`提前返回都会导致`tx`被drop，触发ROLLBACK
- ✅ 无需显式`tx.rollback()`调用

---

### Q4（ef_search有效性）: ✅ **验证通过**

**参数使用**（Line 339）：
```rust
for nid in nids.into_iter().take(ef_search) {  // ef_search控制候选集大小
    if !visited.contains(&nid) { candidates.push(nid); }
}
```

**边界处理**：
- ✅ `ef_search`通过`take(ef_search)`限制邻居扩展数量
- ⚠️ 建议添加`ef_search >= k`断言（可选改进）

---

## 验证结果（V1-V6）

| 验证 ID | 结果 | 证据 |
|:---|:---:|:---|
| V1-契约行数 | ✅ **PASS** | 130行（目标125-135） |
| V2-生产代码 | ✅ **PASS** | 349行生产代码（Line 1-348） |
| V3-零unsafe | ✅ **PASS** | 1处（deny属性），0 unsafe块 |
| V4-unwrap检查 | ⚠️ **PASS** | 生产代码1处（Line 71 Ord trait），可接受 |
| V5-指数衰减 | ⚠️ **DEVIATION** | 5处相关，但公式`level+1`偏差 |
| V6-贪婪导航 | ✅ **PASS** | 4处（get_top_level/rev/greedy_search_layer） |

---

## 发现的问题

### 问题1: 指数衰减公式小偏差（B+级关键原因）

**位置**: `src/memory/src/hnsw.rs` Line 224

**当前代码**：
```rust
(-((level + 1) as f64) / m_f).exp()  // exp(-(level+1)/M)
```

**期望代码**：
```rust
(-(level as f64) / m_f).exp()  // exp(-level/M)
```

**影响**: Layer 0概率略低于100%，但实际影响微小（约6%节点可能缺少Layer 0）

**修复时间**: 1小时内（删除`+ 1`）

---

## 问题与建议

### 短期（立即处理 - 1小时内）

1. **修复指数衰减公式**: Line 224删除`+ 1`，确保`exp(-level/M)`数学正确

### 中期（Week 35 前）

2. **添加ef_search边界断言**（可选）：
```rust
assert!(ef_search >= k, "ef_search must be >= k");
```

3. **统计验证**: 插入1000节点，验证层级分布是否符合理论期望

### 长期（Phase 4 收官）

4. **Week 35 目标**: Recall 90-95%验证，参数调优（M/ef_search）
5. **性能基准**: 与Layer 0扫描对比，验证O(log N)复杂度

---

## 压力怪评语

🥁 **"无聊"**（B+级 - 指数衰减公式小偏差，1小时补正）

> 行数130和349都精准，零unsafe执行到位，Week 33代码完整冻结。
>
> 贪婪导航从Top Level开始写了（Line 319 `for lvl in (1..=top).rev()`），不是Layer 0假导航。
>
> 跨层事务用了（tx.transaction/commit），Rust Drop自动回滚，事务完整性ok。
>
> ef_search参数控制了候选集（Line 339 `take(ef_search)`），功能完整。
>
> **但是**：指数衰减公式写了`level + 1`，应该是`level`。
>
> 契约说`exp(-level/M)`，代码写了`exp(-(level+1)/M)`，Layer 0概率不是100%。
>
> 这是1分钟能修的小问题，不返工，B+级，修完升A-。
>
> Week 35 Recall验证见。

---

## Week 35 准入决定

- **准入状态**: ✅ **Granted**（建议1小时内修复指数衰减公式）
- **准入条件**: 修复`random_level()`中`level + 1`为`level`
- **Week 35 目标**: 
  1. Recall 90-95%验证（100条数据集测试）
  2. 与Layer 0扫描性能对比（验证O(log N)）
  3. 参数调优（M/ef_search最优值）
- **预期评级**: Week 35 Recall达标后目标 **A-级**

---

## 归档建议

- **审计报告**: `audit report/week34/34-AUDIT-WEEK34-Layer1.md` ✅
- **关联债务**: 
  - DEBT-HNSW-ANN-W32: **Layer 1+完成，待Week 35 Recall验证**
- **Week 34 最终评级**: **B+级**（修复后升A-）
- **Week 35 准入状态**: **Granted**

---

*审计链闭环: Week 33(B+) → Week 34(B+/Layer 1+) → Week 35(Recall验证/A-目标)*

☝️🐍♾️⚖️🔍
