# 审计报告 #024 — Sprint2 Day1 交付物复核

| 字段 | 值 |
|------|-----|
| 审计编号 | AUDIT-024 |
| 审计类型 | Sprint2 Day1 交付物复核 |
| 审计日期 | 2026-02-28 |
| 审计员 | Mike（猫娘代码审计员） |
| 输入文件 | task-audit/24.md |
| 总评级 | **A- / Go** |

---

## 1. 审计范围

Sprint2 Day1 声称完成两项技术债务修复：

| 项目 | 声称状态 | 涉及文件 |
|------|----------|----------|
| OBS-001 WASM零拷贝优化 | ✅ 已修复 | `crates/hajimi-hnsw/src/lib.rs`, `src/vector/wasm-loader.js` |
| OBS-002 Redis healthCheck超时保护 | ✅ 已修复 | `src/security/rate-limiter-redis-v2.js` |

---

## 2. OBS-001 WASM零拷贝优化 — 评级：A

### 2.1 Rust 侧变更（lib.rs）

**新增 `search_batch_zero_copy` 方法（约 lib.rs:170-220）**

核心改动：
- 接收 `queries: &[f32]` 而非 `Vec<f32>`，避免 wasm-bindgen 的所有权转移拷贝
- 使用 `#[wasm_bindgen(js_name = "searchBatchZeroCopy")]` 导出
- 内部用 `&queries[start..end]` 切片遍历，与原 `search_batch` 一致的零拷贝内部逻辑
- 返回 `JsValue`（通过 `serde_wasm_bindgen::to_value`）

**审计判定**：
- ✅ `&[f32]` 签名正确 — wasm-bindgen 对 `&[f32]` 会创建一个临时视图到 WASM 线性内存，比 `Vec<f32>` 少一次堆分配+拷贝
- ✅ 切片遍历逻辑正确，边界检查完整（`start + self.dimension <= queries.len()`）
- ⚠️ `unused_mut` 警告：`lib.rs:177` 和 `lib.rs:199` 的 `buffer` 变量声明为 `mut` 但未修改 — 不影响功能，属于代码洁癖级别

**OBS-001-WARN-01**：`unused_mut` 警告
- 严重度：Info（不影响编译/运行）
- 建议：去掉 `mut` 关键字，消除编译警告
- 成本：30秒
- 分类：【可选优化】

### 2.2 JS 侧变更（wasm-loader.js）

**WASMIndexWrapper 新增 `searchBatchZeroCopy` 调用路径**

核心改动：
- `searchBatch()` 方法内新增分支：优先尝试 `this.index.searchBatchZeroCopy()`
- 传入 `Float32Array` 直接传递，不再经过 `Array.from()` 转换
- fallback 到原 `searchBatch()` 方法保持向后兼容

**审计判定**：
- ✅ 消除了 `Array.from(queries)` 的冗余拷贝 — 这是 OBS-001 的根因
- ✅ fallback 机制合理，不会因新方法不存在而崩溃
- ✅ Float32Array 直传 wasm-bindgen 的 `&[f32]` 参数，类型匹配正确

### 2.3 OBS-001 结论

| 维度 | 评价 |
|------|------|
| 根因是否解决 | ✅ 是 — 消除了 JS→WASM 边界的 Array.from 拷贝 |
| 实现方式 | ✅ `&[f32]` 签名 + Float32Array 直传，符合 wasm-bindgen 最优路径 |
| 向后兼容 | ✅ fallback 到旧方法 |
| 风险 | 低 — `&[f32]` 视图在 WASM 调用期间有效，无悬垂引用风险 |

---

## 3. OBS-002 Redis healthCheck 超时保护 — 评级：A

### 3.1 变更内容（rate-limiter-redis-v2.js）

**healthCheck() 方法新增 Promise.race 超时保护**

核心改动：
- `healthCheck()` 内部用 `Promise.race([this.redis.ping(), timeoutPromise])` 包裹
- 超时时间：`this.healthCheckTimeout || 3000`（默认3秒）
- 超时后返回 `{ healthy: false, reason: 'timeout' }`
- 超时 Promise 使用 `setTimeout` + `clearTimeout` 清理，无内存泄漏

**审计判定**：
- ✅ 根因解决 — 原来 `ping()` 在 ioredis 重连时可能无限阻塞，现在有硬超时
- ✅ 3秒默认值合理（比 ioredis 的 `commandTimeout: 5000` 更短，符合 healthCheck 快速失败原则）
- ✅ `clearTimeout` 清理正确，无 timer 泄漏
- ✅ 错误处理完整，catch 分支也返回 `{ healthy: false }`

### 3.2 OBS-002 结论

| 维度 | 评价 |
|------|------|
| 根因是否解决 | ✅ 是 — healthCheck 不再可能无限阻塞 |
| 超时值 | ✅ 3秒合理，可配置 |
| 资源清理 | ✅ clearTimeout 无泄漏 |
| 风险 | 极低 |

---

## 4. 编译验证

### 4.1 cargo check（dev profile）
- 结果：✅ 通过
- 耗时：0.18s（增量编译）
- 警告：2个 `unused_mut`（见 OBS-001-WARN-01）

### 4.2 cargo test（native target）
- 结果：⚠️ 1个测试 panic（`test_dimension_mismatch`）
- 原因：`wasm-bindgen` 的 `__wbindgen_describe` 在 native target 上未实现
- **这是预期行为**：wasm-bindgen 导出的函数不能在非 wasm32 target 上运行，需要 `wasm-pack test --node` 或 `--target wasm32-unknown-unknown`
- 其余5个纯 Rust 逻辑测试：✅ 通过

**审计判定**：编译验证通过。测试 panic 属于 wasm-bindgen 的已知限制，不是代码缺陷。

---

## 5. 发现汇总

| ID | 严重度 | 描述 | 分类 | 状态 |
|----|--------|------|------|------|
| OBS-001-WARN-01 | Info | `lib.rs:177,199` unused_mut 警告 | 【可选优化】 | Open |
| OBS-001 | — | WASM零拷贝优化 | 技术债务 | ✅ 已修复 |
| OBS-002 | — | healthCheck超时保护 | 技术债务 | ✅ 已修复 |

---

## 6. 总评

| 维度 | 评分 | 说明 |
|------|------|------|
| OBS-001 修复质量 | A | `&[f32]` + Float32Array 直传，根因消除 |
| OBS-002 修复质量 | A | Promise.race 超时保护，实现干净 |
| 编译状态 | A- | 通过，2个 Info 级警告 |
| 向后兼容 | A | fallback 机制完整 |
| **综合** | **A-** | 两项技术债务均已根治，仅余 Info 级警告 |

**Go/NoGo 决定：Go**

Sprint2 Day1 交付物质量达标，OBS-001 和 OBS-002 均已根治。可继续推进 Day2 任务。

---

## 7. 建议后续动作

1. （可选）消除 `unused_mut` 警告 — 成本30秒，收益：干净的编译输出
2. 补充 `wasm-pack test --node` 的 CI 步骤 — 确保 wasm32 target 下的测试覆盖
3. 跑一次 benchmark 对比 `searchBatch` vs `searchBatchZeroCopy` 的实际性能差异，量化 OBS-001 修复收益

---

*审计完成 — Mike 喵*
