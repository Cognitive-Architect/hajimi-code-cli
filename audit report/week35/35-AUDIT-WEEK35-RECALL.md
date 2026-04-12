# 35-AUDIT-WEEK35-RECALL 建设性审计报告（部分成功/部分失败诊断版）

**审计官**: 压力怪  
**日期**: 2026-04-09  
**审计链**: Week 35 Recall验证失败诊断与Week 36治疗性返工授权

---

## 审计结论

- **评级**: **C级**（部分成功，关键失败，需治疗性返工）
- **状态**: 有条件返工（Week 36治疗性返工授权）
- **Week 36准入**: ✅ **Granted**（解冻生产代码授权）
- **成功部分**: Architect契约A级，测试框架完整
- **失败部分**: Recall@10 24.8%<90%，10K超时>95s

---

## 进度报告（分项评级）

### 成功部分

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 契约文档 | **A** | 144行（目标140±5），16/16刀刃表通过 ✓ |
| 测试框架 | **A-** | Ground Truth暴力搜索、Recall计算、ef_search调优完整 ✓ |
| 债务诚实 | **A** | 测试代码标记Week 35，失败结果诚实披露 ✓ |

### 失败部分

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| Recall达标 | **D** | 24.8%<90%，红线违反 |
| 性能达标 | **D** | 10K测试超时>95s，P99>10ms要求未满足 |
| 算法实现 | **C** | 贪婪导航简化，缺少启发式路由 |

**整体健康度评级**: **C级**（部分成功，关键失败，需治疗性返工）

---

## 关键疑问回答（Q1-Q4）

### Q1（Recall 24.8%根因）: ✅ **算法缺陷确认**

**测试结果复现**：
```
test test_recall_at_10 ... FAILED
Recall@10 = 24.80%
thread panicked: Recall@10 must be >= 90%, actual = 24.80%
```

**根因分析**：
1. **Entry Point选择非最优**（Line 314）：
   ```rust
   let ep: String = self.conn.query_row("SELECT id FROM hnsw_nodes WHERE level = ?1 LIMIT 1", ...)?;
   ```
   - 使用`LIMIT 1`随机选择，非距离最优

2. **贪婪搜索单层单点**（Line 319）：
   ```rust
   for lvl in (1..=top).rev() { curr = self.greedy_search_layer(query, &curr, lvl, 1)?; }
   ```
   - `ef=1`，每层仅保留1个最近邻
   - 无候选集扩展机制

3. **层间信息丢失**：
   - 上层结果未作为下层初始候选集
   - 每层独立搜索，未实现多层动态路由

---

### Q2（算法缺陷位置）: ✅ **定位完成**

**缺陷位置1：`search_ann_with_ef`（Line 310-347）**
```rust
// 问题：Entry Point随机选择
let ep: String = match self.conn.query_row("SELECT id FROM hnsw_nodes WHERE level = ?1 LIMIT 1", ...)?;

// 问题：ef=1，单层单点
for lvl in (1..=top).rev() { curr = self.greedy_search_layer(query, &curr, lvl, 1)?; }
```

**缺陷位置2：`greedy_search_layer`（Line 246-269）**
```rust
// 问题：仅返回单个最近邻，无ef候选集扩展
fn greedy_search_layer(&self, ..., ef: usize) -> Result<String, HnswError> {
    // ... 仅跟踪best_id，无候选集管理
    Ok(best_id)  // 返回String而非Vec<String>
}
```

**标准HNSW对比**：
| 特性 | 标准HNSW | 当前实现 |
|:---|:---|:---|
| Entry Point | 最近邻继承启发式 | 随机选择 |
| 候选集扩展 | 每层ef个候选动态扩展 | 单层单点 |
| 层间传递 | 上层候选集继承到下层 | 仅传递1个节点 |
| 剪枝策略 | 下界剪枝避免无效计算 | 无剪枝 |

---

### Q3（解冻必要性）: ✅ **必须解冻，范围限定**

**解冻必要性**：算法缺陷在生产代码核心（`search_ann_with_ef`/`greedy_search_layer`），无法通过调参修复。

**建议解冻范围**：
| 范围 | 状态 | 说明 |
|:---|:---:|:---|
| Line 246-269 | **解冻** | `greedy_search_layer`需重构为返回候选集 |
| Line 310-347 | **解冻** | `search_ann_with_ef`需实现启发式路由 |
| Line 1-245 | **冻结** | Layer 0基础、数据结构、存储格式保留 |
| Line 348-465 | **冻结** | 原有测试代码保留 |
| Line 466+ | **test only** | Week 35/36测试代码可扩展 |

**向后兼容保证**：
- 公开API签名不变（`search_ann_with_ef`参数/返回值不变）
- 仅优化内部算法实现
- SQLite表结构不变

---

### Q4（Week 36返工方案）: ✅ **治疗性方案制定**

### 技术方案

| 修改项 | 当前缺陷 | 修复方案 | 预期Recall提升 |
|:---|:---|:---|:---:|
| **Entry Point选择** | `LIMIT 1`随机选择 | 实现`get_nearest_entry_point`：查询Top Level所有节点，返回距离最近者 | +10% |
| **候选集扩展** | 单层单点（ef=1） | `greedy_search_layer`返回`Vec<String>`（ef个候选），而非单个String | +25% |
| **层间信息传递** | 仅传递1个节点 | 上层候选集作为下层初始Entry Points集合 | +15% |
| **Layer 0扩展** | 仅Entry Point邻居 | 实现多源BFS：所有上层候选同时扩展 | +15% |

### 伪代码方案

```rust
// 修复后：search_ann_with_ef（Week 36目标）
pub fn search_ann_with_ef(&self, query: [f32; EMBEDDING_DIM], k: usize, ef_search: usize) -> Result<Vec<Neighbor>, HnswError> {
    let top = self.get_top_level()?;
    
    // 修复1：最优Entry Point选择（查询Top Level所有节点，选最近）
    let mut candidates = self.get_all_nodes_at_level(top)?;
    candidates.sort_by(|a, b| self.distance(query, a).partial_cmp(&self.distance(query, b)).unwrap());
    let top_candidates: Vec<String> = candidates.into_iter().take(ef_search).map(|n| n.id).collect();
    
    // 修复2：层间候选集传递
    let mut current_candidates = top_candidates;
    for lvl in (1..=top).rev() {
        // 修复3：每层扩展ef个候选
        let mut next_candidates: Vec<String> = Vec::new();
        for entry in &current_candidates {
            let layer_candidates = self.greedy_search_layer(query, entry, lvl, ef_search)?;
            next_candidates.extend(layer_candidates);
        }
        // 去重+按距离排序，保留top ef_search
        next_candidates = self.deduplicate_and_sort(query, next_candidates, ef_search)?;
        current_candidates = next_candidates;
    }
    
    // 修复4：Layer 0多源BFS扩展
    self.multi_source_search_layer0(query, &current_candidates, k, ef_search)
}

// 修复后：greedy_search_layer返回候选集
fn greedy_search_layer(&self, query: [f32; EMBEDDING_DIM], entry: &str, level: u8, ef: usize) -> Result<Vec<String>, HnswError> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut candidates: BinaryHeap<(f32, String)> = BinaryHeap::new();  // 最小堆按距离排序
    
    // 初始化候选集
    candidates.push((self.get_dist(entry, query)?, entry.to_string()));
    visited.insert(entry.to_string());
    
    while candidates.len() < ef {
        if let Some((dist, curr)) = candidates.pop() {
            for neighbor in self.get_neighbors(&curr, level)? {
                if visited.insert(neighbor.clone()) {
                    if let Ok(d) = self.get_dist(&neighbor, query) {
                        candidates.push((d, neighbor));
                    }
                }
            }
        } else { break; }
    }
    
    // 返回ef个最近候选
    Ok(candidates.into_vec().into_iter().map(|(_, id)| id).collect())
}
```

### 成功标准

| 指标 | 当前值 | Week 36目标 | 验证方法 |
|:---|:---:|:---:|:---|
| Recall@10 | 24.8% | **≥90%** | `cargo test test_recall_at_10` |
| Recall@50 | ~30% | **≥95%** | 新增测试验证 |
| 10K P99延迟 | >95s | **<100ms** | `cargo test test_10k_nodes_pressure` |
| 零unsafe/unwrap | 符合 | **保持** | `grep unsafe src/memory/src/hnsw.rs` |

---

## 验证结果（V1-V6）

| 验证 ID | 结果 | 证据 |
|:---|:---:|:---|
| V1-契约行数 | ✅ **PASS** | 144行（目标140±5） |
| V2-Recall复现 | ✅ **CONFIRMED** | Recall@10 = 24.80%（可复现失败） |
| V3-算法缺陷 | ✅ **LOCATED** | `greedy_search_layer`单层单点，`search_ann_with_ef` ef=1 |
| V4-生产冻结 | ⚠️ **需解冻** | Line 246-269, 310-347需修改 |
| V5-债务申报 | ✅ **PASS** | 测试代码标记Week 35，诚实披露失败 |
| V6-10K超时 | ✅ **CONFIRMED** | 测试运行>95s超时 |

---

## 问题与建议

### 短期（Week 36治疗性返工）

1. **解冻授权**：允许修改`search_ann_with_ef`（Line 310-347）和`greedy_search_layer`（Line 246-269）
2. **Entry Point优化**：实现`get_nearest_entry_point`，查询Top Level所有节点选最近
3. **候选集扩展**：`greedy_search_layer`返回`Vec<String>`（ef个候选）而非单个String
4. **层间传递**：上层候选集作为下层初始Entry Points

### 中期（Week 36验证）

5. **Recall验证**：目标Recall@10 ≥90%，Recall@50 ≥95%
6. **性能验证**：10K节点P99 <100ms（放宽标准，Week 37优化至<10ms）
7. **回归测试**：确保Layer 0向后兼容，原有测试通过

### 长期（Phase 4收官）

8. **DEBT-HNSW-RECALL-W35清偿**：Week 36验证通过后关闭债务
9. **参数调优文档**：记录最优ef_search/M配置
10. **性能基准报告**：对比ANN vs 暴力扫描性能提升

---

## 压力怪评语

🥁 **"哈？！"**（C级 - 算法有缺陷，Week 36返工授权）

> 契约144行写得漂亮，测试框架也搭得完整，Ground Truth暴力搜索、Recall计算都有。
>
> **但是** Recall@10才24.8%，离90%红线差得远。
>
> 算法缺陷找到了：`search_ann_with_ef`里`ef=1`，每层就贪心一个最近邻，信息丢光了。
>
> `greedy_search_layer`返回单个String，不是候选集，标准HNSW的ef扩展完全没有。
>
> Entry Point也是随机选`LIMIT 1`，连最近邻继承都没做。
>
> 这是算法实现简化过度，不是调参能修的。必须解冻Line 246-269和310-347，重构启发式路由。
>
> **C级评级，Week 36返工授权，解冻生产代码，目标Recall@10 ≥90%。**
>
> 测试框架保留，Architect契约A级资产保护，只改算法核心。

---

## Week 36准入决定

- **准入状态**: ✅ **Granted**（治疗性返工授权）
- **解冻范围**: 
  - ✅ 允许修改：`search_ann_with_ef`（Line 310-347）、`greedy_search_layer`（Line 246-269）
  - ❌ 冻结保留：Line 1-245（数据结构、存储格式）、Line 348-465（原有测试）
- **Week 36目标**: 
  1. 实现启发式Entry Point选择
  2. 实现候选集扩展（ef个候选）
  3. 实现层间信息传递
  4. Recall@10 ≥90%，10K P99 <100ms
- **债务状态**: DEBT-HNSW-RECALL-W35（P0，Week 36清偿）
- **预期评级**: Week 36成功后目标 **A-级**

---

## 归档建议

- **审计报告**: `audit report/week35/35-AUDIT-WEEK35-RECALL.md` ✅
- **成功资产**: 
  - `docs/tech-spec/week35/BENCHMARK-SPEC-v1.0.md`（A级，保留）
  - 测试框架（Ground Truth/Recall计算/ef调优，保留）
- **失败债务**: DEBT-HNSW-RECALL-W35（P0，Week 36清偿）
- **Week 36状态**: 治疗性返工授权，解冻Line 246-269/310-347

---

*审计链闭环: Week 34(A-) → Week 35(C级/部分失败) → 35号审计(治疗性方案) → Week 36(重构授权)*

☝️🐍♾️⚖️🔍
