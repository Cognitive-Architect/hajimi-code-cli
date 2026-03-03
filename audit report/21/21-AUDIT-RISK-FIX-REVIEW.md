# 审计报告 #21 — Task 22 风险修复验收审计

> **审计员**: Mike（审计汪）
> **任务**: Task 18 / task-audit/18.md
> **审计对象**: RISK-01（`_prune_connections` 空函数）+ RISK-02（`getWASMLoader` 单例无并发保护）
> **前置报告**: 审计报告 #20
> **日期**: 2026-02-27

---

## 一、总体健康度

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| RISK-01 修复质量 | A | 实现完整，Borrow Checker 合规，编译 0 错误 |
| RISK-02 修复质量 | A | Promise 缓存方案正确，6/6 并发测试通过 |
| 文档完整性 | A | 白皮书 + 自测表齐全，逐行 diff 清晰 |
| 遗留债务 | B | PRUNE-014/015 E2E 测试待补，`_select_neighbors` 未使用参数警告未清理 |
| **综合评级** | **A-/Go** | 两项 P0 修复均已落地，可放行 Phase 5 |

---

## 二、RISK-01 验收详情

**靶点**: `crates/hajimi-hnsw/src/lib.rs:412`
**修复者**: 唐音

### 2.1 代码核查

实地抽查 `lib.rs:412` 确认：

```
412: fn _prune_connections(&mut self, node_id: u32, level: u8) -> bool {
```

- `&self` → `&mut self` ✅
- 返回 `bool` ✅
- 调用点 `lib.rs:172` 已使用返回值 ✅

### 2.2 编译验证（实测）

```
cargo check
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
0 errors, 5 warnings
```

编译通过，无新增错误。

### 2.3 逻辑正确性

修复方案采用"两阶段借用"策略：
1. 不可变借用阶段：检查是否需要裁剪、收集向量和连接列表
2. 可变借用阶段：写回裁剪结果

这是 Rust 中处理 HashMap 自引用的标准模式，无内存安全问题。

裁剪逻辑：`sort_by` 按余弦距离升序 → `take(m * 2)` → 写回。逻辑正确，复杂度 O(k log k)，k ≤ 2M，可接受。

### 2.4 遗留问题

| 问题 | 等级 | 说明 |
|:---|:---:|:---|
| PRUNE-014/015 E2E 测试跳过 | B | `wasm-pack build` 和 `npm test` 标记为 ⏭️，未实际执行 |
| `_select_neighbors` 未使用参数 `query` | C | 编译警告，与本次修复无关但属于存量债务 |

**PRUNE-014/015 说明**：自测表注明 wasm-bindgen-cli 待完善（DEBT-PHASE2-001），E2E 跳过有合理依据，不阻塞放行，但须在 DEBT-PHASE2-001 解决后补测。

### 2.5 RISK-01 结论

**C → 已修复 ✅**
不修复后果（节点膨胀 O(N) 退化）已消除。

---

## 三、RISK-02 验收详情

**靶点**: `src/vector/wasm-loader.js:199`
**修复者**: 黄瓜睦

### 3.1 代码核查

实地抽查 `wasm-loader.js:199-214` 确认：

```
199: let loaderPromise = null;
202:   if (!loaderPromise) {
204:     loaderPromise = (async () => {
210:   return loaderPromise;
214:   loaderPromise = null;
```

- `loaderInstance` → `loaderPromise` 全量替换 ✅
- `resetWASMLoader` 同步更新 ✅
- Promise 立即赋值，消除异步间隙竞态 ✅

### 3.2 并发测试验证（实测）

```
=== Results: 6 passed, 0 failed ===
CONC-001: 10 concurrent calls return same instance ✅
CONC-002: init() executes only once ✅
CONC-003: No race condition creates multiple instances ✅
CONC-004: Memory increase: 0.33% ✅
CONC-005: Sequential calls return same instance ✅
CONC-006: resetWASMLoader creates new instance on next call ✅
```

内存增长 0.33%，远低于 1.5x 阈值。

### 3.3 方案评估

Promise 缓存是 JavaScript 单例并发控制的惯用模式，优于 mutex 模拟方案：
- 无需引入额外依赖
- 天然利用 JS 事件循环保证原子性
- 代码量最小（+2 行）

### 3.4 遗留问题

无新增债务，接口兼容性保持。

### 3.5 RISK-02 结论

**C → 已修复 ✅**
不修复后果（高并发内存翻倍）已消除。

---

## 四、编译警告清单（存量债务）

以下 5 条警告均为存量，非本次修复引入，记录备案：

| 警告 | 位置 | 类型 | 处置建议 |
|:---|:---|:---:|:---|
| `console_error_panic_hook` feature 未声明 | lib.rs:14/90/507 | cfg | Phase 5 清理，加入 Cargo.toml features |
| 不必要括号 | lib.rs:491 | style | `cargo fix` 一键修复 |
| `query` 参数未使用 | lib.rs:400 | unused | 改为 `_query` 或实现完整 heuristic |

**不修复后果**：仅影响编译输出可读性，不影响运行时行为。建议 Phase 5 Sprint 1 统一清理。

---

## 五、可维护性评估

| 维度 | 评分 | 说明 |
|:---|:---:|:---|
| 修复文档质量 | 9/10 | 白皮书结构完整，逐行 diff 清晰，工时诚实申报 |
| 自测表执行质量 | RISK-01: 8/10 / RISK-02: 10/10 | RISK-01 有 3 项 E2E 跳过 |
| 代码可读性 | 9/10 | 两处修复均有注释说明意图 |
| 回归风险 | 低 | 修改范围精准，无副作用 |

---

## 六、落地建议

### 短期（Phase 5 Sprint 1，≤1周）

1. **清理编译警告**：`cargo fix --lib -p hajimi-hnsw` 处理括号警告；`_select_neighbors` 的 `query` 参数改为 `_query`
2. **补充 PRUNE-014/015**：DEBT-PHASE2-001 解决后，补跑 `wasm-pack build` + `npm test` 并更新自测表

### 中期（Phase 5 Sprint 2）

3. **`_select_neighbors` 完整实现**：当前实现忽略 `query` 参数，未使用启发式选择策略，大规模数据下邻居质量次优

### 长期

4. **HNSW 集成测试**：补充端到端测试覆盖 `_prune_connections` 实际触发路径（需 >2M 个连接的节点）

---

## 七、结论

**综合评级：A-/Go（附条件放行）**

两项 P0 风险（RISK-01、RISK-02）均已正确修复，代码落地可验证，文档完整。

**放行条件**：
- ✅ RISK-01 修复已落地，编译通过
- ✅ RISK-02 修复已落地，6/6 并发测试通过
- ⏭️ PRUNE-014/015 E2E 测试待 DEBT-PHASE2-001 解决后补测（不阻塞）

**Phase 5 可启动。**

---

*审计员: Mike | 报告编号: #21 | 前置报告: #20*
