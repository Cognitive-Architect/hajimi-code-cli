# HNSW Layer 1+ 技术契约规范 v1.0

**文档编号**: SPEC-HNSW-L1-W34 | **版本**: v1.0 | **日期**: 2026-04-03 | **范围**: Week 34 - 多层路由与贪婪导航

---

## 1. Layer 1+ 概述

分层可导航小世界图（HNSW）通过多层图结构实现O(log N)复杂度的近似最近邻（ANN）搜索。Layer 1+作为路由层（routing layers），节点按概率稀疏分布，上层提供长程跳跃，下层逐步精化。Week 34目标：实现指数衰减分层算法、多层图契约、贪婪导航协议。

---

## 2. 指数衰减算法

节点在Level层的存在概率由指数衰减函数决定：

```
P(level) = exp(-level / M)
其中 M = 16

Layer 0: P(0) = exp(0) = 1.0（所有节点存在）
Layer 1: P(1) = exp(-1/16) ≈ 0.9394
Layer 2: P(2) = exp(-2/16) ≈ 0.8825
Layer 3: P(3) = exp(-3/16) ≈ 0.8290
```

**存在判定**: 对每一层生成uniform随机数r∈[0,1)，若r < 1-P(level)则节点存在于该层。实际probability逐层递减形成金字塔结构。

---

## 3. 多层图结构

**层数计算**: 节点最大层数 L_max = ⌈-ln(uniform(0,1)) * M⌉，期望值E[L]≈ln(N)*M。

**跨层连接**: 每层独立维护邻居列表，节点在不同层拥有不同邻居集合。上层邻居用于快速跳转，下层邻居用于精细定位。

**存储扩展**: SQLite新增level字段索引，节点可存在于多个层级记录，通过id+level复合键唯一标识。

---

## 4. 贪婪导航协议

贪婪导航搜索（greedy navigate）从Top Level逐层下降：

```
1. 确定当前最高层 Top Level = max(level)
2. 从Entry Point开始（任意Top Level节点）
3. 在当前层执行贪婪搜索：遍历当前层邻居，选择距离query最近的节点
4. 找到局部最优后，下降到下一层，以该节点作为新Entry Point
5. 重复直到Layer 0，在Layer 0返回最近邻列表
```

**Entry Point选择**: 默认选择最高层任意节点；可维护全局Entry Point指针优化性能。

---

## 5. ef_search参数

**Enter Point Factor（ef_search）**控制搜索时动态候选集大小：

- **默认值**: ef_search = k（返回数量）
- **Recall调优**: 增大ef_search提高Recall（以速度换精度）
- **Layer 0扩展**: ef_search决定Layer 0贪婪搜索的候选池大小，实际比较节点数 = min(ef_search, Layer 0节点数)

**参数范围**: ef_search ∈ [k, 10k]，推荐默认值k，高精度场景使用2k~5k。

---

## 6. SQLite事务要求

跨层更新必须保证原子性：

```sql
BEGIN TRANSACTION;
INSERT INTO hnsw_nodes (id, level, vector_json, neighbors_json) VALUES (?, ?, ?, ?);
-- 对每一层重复插入
UPDATE hnsw_nodes SET neighbors_json = ? WHERE id = ? AND level = ?;
-- 更新反向连接
COMMIT;
```

**事务边界**: 单次插入涉及1+L_max次写操作，必须包裹在单一事务内，失败时ROLLBACK保证一致性。

---

## 7. 向后兼容

**保留Layer 0**: Week 33的281行生产代码完全冻结，不修改、不删除、不重构。

**新增函数**: Layer 1+功能通过新增函数实现：
- `insert_multilayer()` - 带分层的新插入接口
- `search_ann_layered()` - 多层贪婪搜索

**Fallback机制**: 当多层图为空或损坏时，自动降级到Layer 0精确搜索。

---

## 8. 错误处理

| 错误场景 | 检测条件 | 处理策略 |
|----------|----------|----------|
| 空高层 | Top Level无节点 | 直接下降到Layer 0搜索 |
| Entry Point失效 | 节点ID不存在 | 重新选择任意同层节点 |
| 层间不一致 | 高层节点缺失Layer 0记录 | 重建节点Layer 0连接 |
| 邻居循环 | 贪婪搜索陷入局部循环 | 设置最大迭代次数上限 |

---

## 9. 复杂度分析

| 操作 | Layer 0 | Layer 1+ | 优化比 |
|------|---------|----------|--------|
| 插入 | O(N*D) | O(logN * M * D) | N/logN倍 |
| 查询 | O(N*D) | O(logN * M * D) | N/logN倍 |
| 内存 | O(N*M) | O(N*M/(1-1/e)) | ~1.58倍 |

**D=384**, **M=16**。当N=1M时，Layer 0需1M次距离计算，Layer 1+仅需~320次。

---

## 10. 实现约束

- **零unsafe**: 禁止unsafe块，纯Safe Rust实现
- **384维固定**: EMBEDDING_DIM=384不可变
- **冻结Week 33**: Layer 0代码281行禁止任何修改
- **伪代码规范**: 本文档仅含算法伪代码，禁止可编译Rust代码

---

**文档结束**
