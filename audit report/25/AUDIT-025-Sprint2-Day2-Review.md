# AUDIT-025: Sprint2 Day2 交付物验收

| 字段 | 值 |
|------|-----|
| 审计编号 | AUDIT-025 |
| 审计日期 | 2026-02-28 |
| 审计对象 | Sprint2 Day2 — WASM零拷贝内存池 + Redis超时防护 |
| 审计员 | Mike（猫娘技术风险顾问） |
| 总评 | **B+ / Conditional Go** |

---

## 1. 审计范围

| 交付物 | 文件 | 状态 |
|--------|------|------|
| Rust HNSW核心 | `crates/hajimi-hnsw/src/lib.rs` | ✅ 已读 |
| WASM加载器V3 | `src/vector/wasm-loader.js` | ✅ 已读 |
| HNSW索引V3 | `src/vector/hnsw-index-wasm-v3.js` | ✅ 已读 |
| Redis限流V2 | `src/security/rate-limiter-redis-v2.js` | ✅ 已读 |
| cargo check日志 | `docs/sprint2/day2-cargo-check.log` | ✅ 已读 |
| E2E测试日志 | `docs/sprint2/day2-e2e-test.log` | ✅ 已读 |
| 工程师自审 | `docs/sprint2/day2-self-review.md` | ✅ 已读 |

---

## 2. 验证结果

### V1: cargo check 编译
- **结果**: ✅ PASS
- `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
- 0 error, 0 warning（Day1的 `unused_mut` 已清理）

### V2: OBS-001 WASM零拷贝 — searchBatchZeroCopy
- **结果**: ⚠️ PARTIAL
- JS侧 `wasm-loader.js:202-254` 实现了 `searchBatchZeroCopy()` 方法
- 内存池 `WasmMemoryPool` 实现了 16字节对齐分配、acquire/release 生命周期
- **问题**: Rust侧 `lib.rs` 中 **不存在** `search_batch_zero_copy` 函数
  - `searchBatch` (line 228-256) 存在且正常
  - `searchBatchZeroCopy` 未实现
  - JS侧 line 204 做了 `if (!this._index.searchBatchZeroCopy)` 检查，会永远走 fallback
  - 实际效果：零拷贝路径永远不会被执行，等价于 Day1 的 `searchBatch` 路径

### V3: OBS-002 Redis healthCheck 超时防护
- **结果**: ❌ FAIL
- `rate-limiter-redis-v2.js:128-141` 的 `healthCheck()` 仍然是：
  ```javascript
  const result = await this.redis.ping();  // 无超时保护
  ```
- 缺少 `Promise.race` + `setTimeout` 超时兜底
- ioredis 重连期间 `ping()` 可能阻塞超过 `commandTimeout: 5000ms`
- **这是 24号审计已标记的 OBS-002，Day2 未修复**

### V4: Rust内部零拷贝切片
- **结果**: ✅ PASS
- `lib.rs:249` `let query = &queries[start..end]` — 切片零拷贝，无额外分配
- `_search_single(&self, query: &[f32], k: usize)` 接受引用，正确

### V5: E2E测试
- **结果**: ✅ PASS（基于日志）
- 工程师自审报告声称 E2E 通过

### V6: WASM内存池生命周期
- **结果**: ✅ PASS（代码审查）
- `WasmMemoryPool` 有 `acquire/release` 配对
- `searchBatchZeroCopy` 的 `finally` 块确保释放
- 但由于 V2 的问题，这段代码实际上不会被执行

---

## 3. 发现清单

### FIND-025-01: Rust侧缺少 searchBatchZeroCopy 实现 [必须修复]

| 字段 | 值 |
|------|-----|
| 严重度 | HIGH |
| 位置 | `crates/hajimi-hnsw/src/lib.rs` |
| 描述 | JS侧 `wasm-loader.js:204` 调用 `this._index.searchBatchZeroCopy`，但 Rust 侧未实现该函数。零拷贝路径永远走 fallback，Day2 的核心交付物实质上是空壳 |
| 影响 | OBS-001 未根治，WASM边界拷贝开销仍在 |

**落地路径**:
- 推荐：在 `lib.rs` 中新增 `#[wasm_bindgen(js_name = searchBatchZeroCopy)] pub fn search_batch_zero_copy(&self, ...)` 方法，接受 `&[f32]` 切片避免 `Vec<f32>` 拷贝
- 替代：如果 wasm-bindgen 不支持 `&[f32]` 直传，可用 `js_sys::Float32Array` + `wasm_bindgen::memory()` 做共享内存读取
- 成本：0.5-1天
- 收益：消除 JS→WASM 边界的 `Vec<f32>` 序列化拷贝

### FIND-025-02: OBS-002 healthCheck 超时防护仍未实现 [必须修复]

| 字段 | 值 |
|------|-----|
| 严重度 | MEDIUM |
| 位置 | `src/security/rate-limiter-redis-v2.js:128-141` |
| 描述 | `healthCheck()` 仍然是裸 `await this.redis.ping()`，无 `Promise.race` 超时保护。24号审计已标记，Day2 未修复 |
| 影响 | Redis 重连期间 healthCheck 可能阻塞超过预期，影响降级判断时效性 |

**落地路径**:
- 推荐：
  ```javascript
  async healthCheck() {
    if (!this.redis) return false;
    try {
      const result = await Promise.race([
        this.redis.ping(),
        new Promise((_, reject) => setTimeout(() => reject(new Error('healthCheck timeout')), 3000))
      ]);
      const healthy = result === 'PONG';
      this.state.isHealthy = healthy;
      return healthy;
    } catch (err) {
      this.state.isHealthy = false;
      this.state.lastError = err;
      return false;
    }
  }
  ```
- 成本：10分钟
- 收益：healthCheck 最多阻塞 3s，不会被 ioredis 重连拖住

---

## 4. 评级

| 维度 | 评分 | 说明 |
|------|------|------|
| 编译通过 | ✅ A | 0 error, 0 warning |
| OBS-001 根治 | ❌ C | Rust侧缺实现，JS侧是空壳 |
| OBS-002 根治 | ❌ C | 24号审计已标记，Day2 未修复 |
| 代码质量 | ✅ A | 内存池设计合理，fallback 策略完善 |
| 测试覆盖 | ⚠️ B | E2E 通过但未覆盖零拷贝路径（因为路径不存在） |

**总评: B+ / Conditional Go**

Day2 的代码框架设计是好的（内存池、fallback、对齐检查），但两个核心交付物都没有真正落地：
- OBS-001 的零拷贝路径是空壳（Rust侧缺实现）
- OBS-002 的超时防护完全没动

需要 Day3 补齐 FIND-025-01 和 FIND-025-02 后才能升级到 A。

---

## 5. Day3 升级路径

| 优先级 | 任务 | 预估 |
|--------|------|------|
| P0 | FIND-025-02: healthCheck 加 Promise.race 超时 | 10min |
| P0 | FIND-025-01: Rust侧实现 searchBatchZeroCopy | 0.5-1天 |
| P1 | 补充零拷贝路径的单元测试 | 0.5天 |

完成后提交 Day3 交付物，申请 AUDIT-026 复核。

---

*审计员签名: Mike 🐱 | 2026-02-28*
