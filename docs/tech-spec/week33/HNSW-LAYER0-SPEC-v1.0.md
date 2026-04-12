# HNSW Layer 0 技术契约规范 v1.0

**文档编号**: SPEC-HNSW-L0-W33 | **版本**: v1.0 | **日期**: 2026-04-09 | **范围**: Week 33 - Layer 0奠基（禁止涉及Layer 1+）

---

## 1. HNSW概述

分层可导航小世界图（Hierarchical Navigable Small World, HNSW）是基于图结构的近似最近邻（ANN）搜索算法。底层为全连接基础层（ground layer），上层为稀疏路由层。本文档定义Week 33范围：仅实现Layer 0全连接基础层，Layer 1+路由层为Week 34内容。

---

## 2. Layer 0定义

Layer 0（ground layer）是HNSW的全连接基础层（ground layer），包含索引中的所有节点：**全连接性**-所有向量节点必须存在于Layer 0；**邻居上限**-每个节点最多M=16个双向邻居连接；**基础导航**-在Layer 0上执行贪婪搜索完成最终近邻定位；**存储特性**-使用SQLite持久化节点与邻接关系。**禁忌**: 本文档不涉及Layer 1/2/3+实现细节，多层路由逻辑归属Week 34。

---

## 3. 技术参数

| 参数 | 符号 | 值 | 说明 |
|------|------|-----|------|
| 嵌入维度 | `EMBEDDING_DIM` | 384 | f32向量维度，固定384维 |
| 最大邻居数 | `M` | 16 | 每个节点最大双向连接数 |
| 构建搜索深度 | `ef_construction` | 200 | 插入时候选邻居池大小 |
| 距离度量 | `distance_fn` | Euclidean | 欧氏距离计算 |

```rust
pub const EMBEDDING_DIM: usize = 384;
pub const M: usize = 16;
pub const EF_CONSTRUCTION: usize = 200;
```

---

## 4. 节点存储格式

**SQLite表结构DDL**:
```sql
CREATE TABLE hnsw_nodes (
    id INTEGER PRIMARY KEY, level INTEGER NOT NULL DEFAULT 0,
    vector_json TEXT NOT NULL, neighbors_json TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_hnsw_level ON hnsw_nodes(level);
```

**字段约束**: `id`为节点唯一标识符；`level`固定为0；`vector_json`为384维f32数组的JSON序列化；`neighbors_json`为邻居节点ID列表，长度范围`[0, M]`。

---

## 5. 邻居列表序列化

**JSON格式协议**: 邻居列表采用紧凑JSON数组格式存储：`[42, 101, 237, 15, 88]`（长度 ∈ [0, M]）

**序列化规范**: **类型**-`Vec<usize>` -> JSON数组；**最大长度**-M = 16；**排序**-按插入时距离升序；**空值**-`[]`表示孤立节点

**邻居选择算法**:
```rust
fn select_neighbors(candidates: Vec<Neighbor>, m: usize) -> Vec<usize> {
    candidates.iter().take(m).map(|n| n.id).collect()
}
```

---

## 6. 距离度量

**欧氏距离（Euclidean Distance）公式**: 对于384维向量 a, b ∈ R^384：d(a, b) = sqrt(Σ(i=1 to 384) (a_i - b_i)^2)

**Rust实现**:
```rust
fn euclidean_distance(a: &[f32; EMBEDDING_DIM], b: &[f32; EMBEDDING_DIM]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt()
}
```
**约束**: 零向量距离定义为`f32::INFINITY`，避免除零错误。

---

## 7. 接口契约

```rust
pub fn search_ann(query: &[f32; EMBEDDING_DIM], ef: usize, k: usize) -> Result<Vec<Neighbor>, HnswError>;
pub fn insert(id: usize, vector: &[f32; EMBEDDING_DIM]) -> Result<(), HnswError>;
```
**副作用**: 计算与现有Layer 0所有节点的距离；选择最多M个最近邻居建立双向连接；更新SQLite中neighbors_json字段。

---

## 8. 向后兼容

保留精确搜索作为Fallback机制：
```rust
pub mod exact {
    pub fn search(query: &[f32; EMBEDDING_DIM], k: usize) -> Result<Vec<Neighbor>, HnswError>;
}
```
**使用场景**: 单元测试基准验证；Layer 0节点数 < 100时的性能退化回退；ANN搜索结果校验采样。

---

## 9. 错误处理

| 错误码 | 场景 | 处理策略 |
|--------|------|----------|
| `EmptyTable` | hnsw_nodes表无记录 | 返回空结果Vec::new() |
| `DimensionMismatch` | vector_json长度 != 384 | 返回Error，记录warn日志 |
| `ZeroVector` | 输入向量全零 | 距离设为f32::INFINITY |
| `NeighborOverflow` | 邻居数 > M | 截断至M，记录debug日志 |

---

## 10. 复杂度分析

**插入复杂度**: O(N*D)，N=Layer 0节点数, D=384。当前Layer 0为线性扫描，计算与所有现有节点的欧氏距离。

**查询复杂度**: O(N*D)。Week 33精确扫描，Week 34引入多层路由优化。

**DEBT声明**: 当前Layer 0为精确搜索，DEBT-HNSW-ANN-W32要求在Week 34实现真正ANN路由。

---

**文档结束**
