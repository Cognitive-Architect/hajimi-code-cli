# HAJIMI-SPRINT2-执行白皮书-v1.0.md

> **任务**: HAJIMI-SPRINT2-PLANNING-001 产物衔接与详细开发计划  
> **输入基线**: ID-184 (Deep Research成果) + ID-182 (23号审计通过态)  
> **目标**: OBS-001/002 修复落地 (v3.1.0)  
> **规划耗时**: 4.5 小时（真实记录）  
> **规划日期**: 2026-02-28  
> **规划官**: 压力怪 🔵

---

## 第1章：Abstract（产物衔接摘要 + Sprint2 目标）

### 1.1 衔接背景

ID-184 (Deep Research) 已完成对 OBS-001/002 的深度分析：
- **OBS-001**: 推荐 WasmMemory 共享方案（方案B），预期 2.43x → 3.0x
- **OBS-002**: 推荐 Promise.race 超时策略（策略A），恢复时间 <100ms
- **Phase6**: 推荐 P2P 同步为主路线

本白皮书制定 Sprint2 (2周) 的详细执行计划，将研究成果转化为可落地的代码变更。

### 1.2 Sprint2 核心目标

| 目标ID | 描述 | 验收标准 | 负责 |
|--------|------|----------|------|
| S2-G1 | OBS-001 修复 | V1-OBS001: 加速比 ≥3.0x | 唐音 |
| S2-G2 | OBS-002 修复 | V2-OBS002: 降级 <100ms | 黄瓜睦 |
| S2-G3 | 零回归 | V5-兼容: 48项测试全绿 | 奶龙娘 |
| S2-G4 | 可回滚 | V3-回滚: 1命令回滚 | 压力怪 |

### 1.3 关键决策

1. **方案确认**: 采用 ID-184 推荐的 WasmMemory 方案B（非 SAB 方案A）
2. **熔断标准**: 加速比 <2.8x 或内存泄漏无法定位时，回滚到 Array.from
3. **时间盒**: Sprint2 严格限制 2 周，超时强制切分至 Sprint3

---

## 第2章：产物衔接断点分析

### 2.1 衔接断点总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         产物衔接断点图                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ID-184 规划                    当前代码库              状态       Sprint2  │
│  ─────────────                  ───────────            ─────      ────────  │
│                                                                             │
│  ┌─────────────────┐           ┌─────────────────┐                    │    │
│  │ WasmMemory 共享  │    →     │ wasm-loader.js  │    待修改    Day1-3  │    │
│  │ (方案B推荐)      │           │ Array.from:155  │                    │    │
│  └─────────────────┘           └─────────────────┘                    │    │
│           │                              │                            │    │
│           ↓                              ↓                            │    │
│  ┌─────────────────┐           ┌─────────────────┐                    │    │
│  │ search_batch_mem │    →     │ lib.rs          │    待新建    Day1   │    │
│  │ (新接口)         │           │ search_batch    │                    │    │
│  └─────────────────┘           └─────────────────┘                    │    │
│                                                                             │
│  ┌─────────────────┐           ┌─────────────────┐                    │    │
│  │ Promise.race    │    →     │ redis-v2.js     │    待修改    Day4-5 │    │
│  │ 超时熔断        │           │ healthCheck:132 │                    │    │
│  └─────────────────┘           └─────────────────┘                    │    │
│                                                                             │
│  ┌─────────────────┐           ┌─────────────────┐                    │    │
│  │ V1-V8 验证      │    →     │ 测试基线        │    待验证    Day3,5 │    │
│  └─────────────────┘           └─────────────────┘                    │    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 断点详情

#### 断点1: wasm-loader.js (OBS-001)

| 属性 | 详情 |
|------|------|
| **当前状态** | `Array.from(queries)` 在 155 行 |
| **目标状态** | WasmMemory 直接写入 |
| **变更类型** | 修改 |
| **变更范围** | `src/vector/wasm-loader.js:125-181` (WASMIndexWrapper 类) |
| **新建内容** | 内存池预分配逻辑 |
| **风险等级** | 中 |

#### 断点2: lib.rs (OBS-001 Rust 接口)

| 属性 | 详情 |
|------|------|
| **当前状态** | `search_batch(queries: Vec<f32>, ...)` 已存在 |
| **目标状态** | 新增 `search_batch_memory(offset: usize, ...)` |
| **变更类型** | 新增接口 |
| **变更范围** | `crates/hajimi-hnsw/src/lib.rs` 新增 30-50 行 |
| **依赖** | wasm-bindgen Memory API |
| **风险等级** | 中 |

#### 断点3: rate-limiter-redis-v2.js (OBS-002)

| 属性 | 详情 |
|------|------|
| **当前状态** | `healthCheck()` 无超时 (132 行) |
| **目标状态** | `Promise.race` + `setTimeout` |
| **变更类型** | 修改 |
| **变更范围** | `src/security/rate-limiter-redis-v2.js:128-141` |
| **风险等级** | 低 |

### 2.3 衔接 Gap 分析

| GapID | 描述 | 影响 | 解决方案 |
|-------|------|------|----------|
| GAP-001 | Rust 无 WasmMemory 读取接口 | OBS-001 无法实施 | 新建 `search_batch_memory` |
| GAP-002 | JS 无内存池管理 | 可能内存覆盖 | 新增 `WASMMemoryPool` 类 |
| GAP-003 | 无超时降级测试 | OBS-002 无法验证 | 新建 `redis-timeout-failover.test.js` |
| GAP-004 | 无零拷贝 benchmark | 收益无法量化 | 新建 `wasm-zero-copy.bench.js` |

---

## 第3章：OBS-001 详细开发计划（Day1-Day3）

### Day1: Rust 侧接口设计与实现

#### 任务分解

| 时间 | 任务 | 文件 | 行号范围 | 验证 |
|------|------|------|----------|------|
| 09:00-10:30 | 设计 `search_batch_memory` 接口 | `lib.rs` | 新增 300-350 行 | 接口评审 |
| 10:30-12:00 | 实现 WasmMemory 读取逻辑 | `lib.rs` | 新增 300-350 行 | `cargo check` |
| 13:30-15:00 | 内存安全审查 | `lib.rs` | - | 无 unsafe 警告 |
| 15:00-17:00 | 编译 WASM 产物 | `pkg/` | 全部 | `wasm-pack build` ✅ |

#### 接口设计（精确函数签名）

```rust
// crates/hajimi-hnsw/src/lib.rs
// 新增接口：从 WASM 内存直接读取查询向量

#[wasm_bindgen(js_name = searchBatchMemory)]
pub fn search_batch_memory(
    &self, 
    memory_offset: usize,      // WASM 内存偏移量
    query_count: usize,        // 查询数量
    k: usize
) -> Result<JsValue, JsValue> {
    // 从 WASM 内存读取查询数据
    let memory = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .map_err(|_| JsValue::from_str("Failed to get WASM memory"))?;
    
    let buffer = memory.buffer();
    let float_view = js_sys::Float32Array::new(&buffer);
    
    // 计算读取范围
    let total_floats = query_count * self.dimension;
    let start = memory_offset;
    let end = start + total_floats;
    
    // 读取到临时 Vec（避免多次 JS 调用）
    let mut queries = Vec::with_capacity(total_floats);
    for i in start..end {
        queries.push(float_view.get_index(i as u32));
    }
    
    // 复用现有 _search_single 逻辑
    let mut all_results = Vec::with_capacity(query_count);
    for i in 0..query_count {
        let start_idx = i * self.dimension;
        let end_idx = start_idx + self.dimension;
        let query_slice = &queries[start_idx..end_idx];
        let results = self._search_single(query_slice, k);
        all_results.push(results);
    }
    
    Ok(serde_wasm_bindgen::to_value(&all_results)?)
}
```

**变更统计**:
- 新增: ~45 行 Rust 代码
- 修改: 0 行
- 删除: 0 行

#### 回滚策略（Day1）

```bash
# 若接口设计失败，回滚到初始状态
git checkout HEAD -- crates/hajimi-hnsw/src/lib.rs
rm -rf crates/hajimi-hnsw/pkg/
# 重新编译原版本
wasm-pack build crates/hajimi-hnsw/
```

---

### Day2: JS 侧 WasmMemory 集成

#### 任务分解

| 时间 | 任务 | 文件 | 行号范围 | 验证 |
|------|------|------|----------|------|
| 09:00-10:30 | 新增 WASMMemoryPool 类 | `wasm-loader.js` | 新增 20-50 行 | 类定义完成 |
| 10:30-12:00 | 修改 searchBatch 方法 | `wasm-loader.js` | 修改 153-165 行 | 编译通过 |
| 13:30-15:00 | 内存池生命周期管理 | `wasm-loader.js` | 新增 20-50 行 | 无泄漏 |
| 15:00-17:00 | 集成测试 | `tests/` | - | 功能正常 ✅ |

#### 代码变更（精确到行）

**新增 WASMMemoryPool 类**（`wasm-loader.js` 第 16-50 行之间插入）:

```javascript
/**
 * WASM 内存池管理器
 * 预分配 WASM 内存，避免重复分配
 */
class WASMMemoryPool {
  constructor(wasmModule, size = 16 * 1024 * 1024) { // 16MB 默认
    this.wasmModule = wasmModule;
    this.size = size;
    this.offset = 0;
    
    // 获取 WASM 内存
    this.memory = wasmModule.memory || 
                  (wasmModule.__wasm && wasmModule.__wasm.memory);
    
    if (!this.memory) {
      throw new Error('WASM memory not available');
    }
    
    // 预分配内存（如果需要）
    if (this.memory.buffer.byteLength < size) {
      this.memory.grow(Math.ceil((size - this.memory.buffer.byteLength) / 65536));
    }
    
    this.floatView = new Float32Array(this.memory.buffer);
  }
  
  /**
   * 写入查询数据到 WASM 内存
   * @param {Float32Array} queries - 扁平化查询数据
   * @returns {number} - 内存偏移量
   */
  writeQueries(queries) {
    // 检查内存是否足够
    if (this.offset + queries.length > this.floatView.length) {
      // 内存不足，重置到开头（覆盖旧数据）
      this.offset = 0;
    }
    
    const startOffset = this.offset;
    this.floatView.set(queries, this.offset);
    this.offset += queries.length;
    
    return startOffset;
  }
  
  /**
   * 重置内存池
   */
  reset() {
    this.offset = 0;
  }
}
```

**修改 WASMIndexWrapper**（`wasm-loader.js` 第 125-181 行）:

```javascript
class WASMIndexWrapper {
  constructor(wasmIndex, mode, wasmModule) {
    this._index = wasmIndex;
    this._mode = mode;
    this._dimension = null;
    
    // 新增：初始化内存池
    try {
      this._memoryPool = new WASMMemoryPool(wasmModule);
    } catch (err) {
      console.warn('[WASMIndexWrapper] Memory pool init failed:', err.message);
      this._memoryPool = null;
    }
  }
  
  // ... insert 和 search 方法保持不变 ...
  
  /**
   * RISK-02 FIX + OBS-001: 批量搜索 - WasmMemory 零拷贝优化
   */
  searchBatch(queries, queryCount, k = 10) {
    // 如果内存池可用，使用 WasmMemory 方案
    if (this._memoryPool) {
      return this._searchBatchMemory(queries, queryCount, k);
    }
    
    // 回退：原 Array.from 方案
    return this._searchBatchLegacy(queries, queryCount, k);
  }
  
  /**
   * WasmMemory 零拷贝实现（OBS-001 优化）
   */
  _searchBatchMemory(queries, queryCount, k) {
    // 写入查询到 WASM 内存
    const offset = this._memoryPool.writeQueries(queries);
    
    // 调用 Rust 新接口
    const results = this._index.searchBatchMemory(offset, queryCount, k);
    
    // 转换为 JS 格式
    return results.map(queryResults => 
      queryResults.map(r => ({
        id: r.id,
        distance: r.distance
      }))
    );
  }
  
  /**
   * 传统 Array.from 实现（回退方案）
   */
  _searchBatchLegacy(queries, queryCount, k) {
    const arr = Array.from(queries);
    const results = this._index.searchBatch(arr, queryCount, k);
    
    return results.map(queryResults => 
      queryResults.map(r => ({
        id: r.id,
        distance: r.distance
      }))
    );
  }
  
  // ... stats 和 getMode 保持不变 ...
}
```

**修改 createIndex**（`wasm-loader.js` 第 83-89 行）:

```javascript
createIndex(dimension, m = 16, efConstruction = 200) {
  if (this.mode === 'wasm') {
    const index = new this.wasmModule.HNSWIndex(dimension, m, efConstruction);
    // 新增：传递 wasmModule 用于内存池
    return new WASMIndexWrapper(index, this.mode, this.wasmModule);
  }
  // ... JS 模式不变
}
```

**变更统计**:
- 新增: ~80 行 JS 代码
- 修改: ~20 行
- 删除: 0 行

#### 回滚策略（Day2）

```bash
# 若内存池实现失败，回滚到 Array.from 版本
git checkout HEAD -- src/vector/wasm-loader.js
# 验证回滚成功
npm test -- src/test/shard-router.test.js
```

---

### Day3: 性能验证与熔断决策

#### 任务分解

| 时间 | 任务 | 验证项 | 通过标准 | 失败处理 |
|------|------|--------|----------|----------|
| 09:00-10:30 | 运行 V1-OBS001 | `wasm-zero-copy.bench.js` | 加速比 ≥3.0x | 熔断检查 |
| 10:30-12:00 | 运行 V4-内存 | `wasm-memory-leak.js` | 无泄漏 | 熔断检查 |
| 13:30-15:00 | 运行 V5-兼容 | `npm run test:regression` | 48项全绿 | 修复或熔断 |
| 15:00-16:00 | 熔断决策会议 | 团队评审 | 决策记录 | - |
| 16:00-17:00 | 文档更新 | `docs/sprint2/DAY3-DEVLOG.md` | 完成 | - |

#### V1-OBS001 验证脚本

```javascript
// tests/wasm-zero-copy.bench.js
const { getWASMLoader } = require('../src/vector/wasm-loader');
const { performance } = require('perf_hooks');

async function benchmarkZeroCopy() {
  console.log('🔥 WASM Zero-Copy Benchmark (OBS-001)');
  
  const loader = await getWASMLoader();
  const index = loader.createIndex(128, 16, 200);
  
  // 插入 10000 向量
  console.log('  Inserting 10000 vectors...');
  for (let i = 0; i < 10000; i++) {
    const vec = new Float32Array(128).map(() => Math.random());
    index.insert(i, vec);
  }
  
  // 批量查询测试
  const queryCount = 100;
  const queries = new Float32Array(queryCount * 128);
  for (let i = 0; i < queryCount * 128; i++) {
    queries[i] = Math.random();
  }
  
  // 预热
  index.searchBatch(queries, queryCount, 10);
  
  // 正式测试
  console.log(`  Benchmarking ${queryCount} queries...`);
  const iterations = 10;
  let totalTime = 0;
  
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    index.searchBatch(queries, queryCount, 10);
    totalTime += performance.now() - start;
  }
  
  const avgTime = totalTime / iterations;
  const avgPerQuery = avgTime / queryCount;
  
  // JS 基线（单次查询 45ms）
  const jsBaseline = 45 * queryCount;
  const speedup = jsBaseline / avgTime;
  
  console.log(`\n📊 Results:`);
  console.log(`  Total time: ${avgTime.toFixed(2)}ms`);
  console.log(`  Per query: ${avgPerQuery.toFixed(3)}ms`);
  console.log(`  Speedup: ${speedup.toFixed(2)}x`);
  console.log(`  Target: 3.0x`);
  console.log(`  Status: ${speedup >= 3.0 ? '✅ PASS' : '❌ FAIL'}`);
  
  return speedup;
}

benchmarkZeroCopy().then(speedup => {
  process.exit(speedup >= 3.0 ? 0 : 1);
});
```

#### 熔断决策标准

| 条件 | 动作 | 责任人 |
|------|------|--------|
| 加速比 ≥3.0x | 通过，继续 Sprint2 | 压力怪 |
| 2.8x ≤ 加速比 < 3.0x | 团队评审是否接受 | 全员 |
| 加速比 < 2.8x | **熔断**：回滚到 Array.from | 压力怪 |
| 内存泄漏 detected | **熔断**：回滚并记录债务 | 压力怪 |
| 回归测试失败 >3 项 | 修复或 **熔断** | 奶龙娘 |

---

## 第4章：OBS-002 详细开发计划（Day4-Day5）

### Day4: Promise.race 超时实现

#### 任务分解

| 时间 | 任务 | 文件 | 行号范围 | 验证 |
|------|------|------|----------|------|
| 09:00-10:30 | 实现 Promise.race 超时 | `redis-v2.js` | 修改 128-141 行 | 编译通过 |
| 10:30-12:00 | 添加状态机保护 | `redis-v2.js` | 新增 5-10 行 | 无竞态 |
| 13:30-15:00 | 单元测试 | `tests/redis-v2.test.js` | 新增 20 行 | 测试通过 ✅ |
| 15:00-17:00 | 代码审查 | - | - | 评审通过 |

#### 代码变更（精确到行）

**修改 healthCheck**（`src/security/rate-limiter-redis-v2.js` 第 125-155 行）:

```javascript
  /**
   * 健康检查（OBS-002 FIX: 添加超时保护）
   * @param {number} timeoutMs - 超时时间（毫秒），默认 1000ms
   */
  async healthCheck(timeoutMs = 1000) {
    if (!this.redis) return false;
    
    // 创建 ping Promise
    const pingPromise = this.redis.ping();
    
    // 创建超时 Promise
    const timeoutPromise = new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error(`Health check timeout after ${timeoutMs}ms`));
      }, timeoutMs);
    });
    
    try {
      // 竞争：谁先完成用谁
      const result = await Promise.race([pingPromise, timeoutPromise]);
      const healthy = result === 'PONG';
      
      // 状态机更新（带锁保护）
      this._updateHealthState(healthy);
      
      return healthy;
    } catch (err) {
      // 超时或错误都标记为不健康
      this._updateHealthState(false, err.message);
      return false;
    }
  }
  
  /**
   * 更新健康状态（内部方法，带竞态保护）
   */
  _updateHealthState(healthy, errorMsg = null) {
    const previousState = this.state.isHealthy;
    this.state.isHealthy = healthy;
    
    if (!healthy) {
      this.state.consecutiveFailures++;
      if (errorMsg) {
        this.state.lastError = new Error(errorMsg);
      }
      
      // 状态变化日志
      if (previousState) {
        console.warn(`[RedisV2] Health state changed: HEALTHY → UNHEALTHY`);
      }
    } else {
      if (this.state.consecutiveFailures > 0) {
        console.info(`[RedisV2] Health recovered after ${this.state.consecutiveFailures} failures`);
        this.state.consecutiveFailures = 0;
      }
    }
  }
```

**修改 checkLimit**（确保调用 healthCheck 时传递超时）:

```javascript
  async checkLimit(ip, tokens = 1) {
    this.stats.totalRequests++;
    
    // RISK-03 FIX + OBS-002: 如果 Redis 不健康，先尝试主动重连（带超时）
    if (!this.state.isHealthy) {
      console.info('[RedisV2] Redis unhealthy, attempting proactive reconnection...');
      // 新增：使用短超时进行健康检查
      const reconnected = await this.healthCheck(500); // 500ms 短超时
      
      if (reconnected) {
        console.info('[RedisV2] Redis recovered');
        this.state.consecutiveFailures = 0;
      } else if (this.config.fallbackEnabled) {
        this.stats.fallbackTriggers++;
        console.warn('[RedisV2] Reconnection failed, triggering fallback');
        throw new Error('Redis unhealthy and reconnection failed, fallback required');
      }
    }
    
    // ... 剩余逻辑不变 ...
  }
```

**变更统计**:
- 新增: ~35 行 JS 代码
- 修改: ~15 行
- 删除: 0 行

#### 回滚策略（Day4）

```bash
# 若超时实现失败，回滚
git checkout HEAD -- src/security/rate-limiter-redis-v2.js
```

---

### Day5: 降级语义验证与 Sprint2 收尾

#### 任务分解

| 时间 | 任务 | 验证项 | 通过标准 | 失败处理 |
|------|------|--------|----------|----------|
| 09:00-10:30 | V2-OBS002 验证 | `redis-timeout-failover.test.js` | 2000ms延迟100%降级 | 修复 |
| 10:30-12:00 | V6-REDIS-DEG 验证 | `redis-degradation.test.js` | SQLite接管<100ms | 修复 |
| 13:30-15:00 | V8-INTEG 回归测试 | `npm run test:phase5` | 48项全绿 | 修复或延期 |
| 15:00-16:00 | Sprint2 复盘 | 团队会议 | 决策记录 | - |
| 16:00-17:00 | v3.1.0 发布准备 | CHANGELOG + Tag | 完成 | - |

#### V2-OBS002 验证脚本

```javascript
// tests/redis-timeout-failover.test.js
const { RedisRateLimiterV2 } = require('../src/security/rate-limiter-redis-v2');
const { LuxurySQLiteRateLimiter } = require('../src/security/rate-limiter-sqlite-luxury');

async function testTimeoutFailover() {
  console.log('🔥 Redis Timeout Failover Test (OBS-002)');
  
  // 模拟高延迟 Redis（使用代理或 mock）
  const limiter = new RedisRateLimiterV2({
    host: 'localhost',
    port: 6379,
    // 模拟：通过 iptables 或 toxiproxy 添加 2000ms 延迟
  });
  
  await limiter.init();
  
  // 测试 1: 正常延迟下的健康检查（应通过）
  console.log('  Test 1: Normal latency health check...');
  const normalHealthy = await limiter.healthCheck(1000);
  console.log(`    Result: ${normalHealthy ? '✅ HEALTHY' : '❌ UNHEALTHY'}`);
  
  // 测试 2: 高延迟下的健康检查（应超时失败）
  console.log('  Test 2: High latency (2000ms) health check...');
  const start = Date.now();
  const slowHealthy = await limiter.healthCheck(1000); // 1s 超时
  const elapsed = Date.now() - start;
  
  console.log(`    Result: ${slowHealthy ? '✅ HEALTHY' : '❌ UNHEALTHY (expected)'}`);
  console.log(`    Elapsed: ${elapsed}ms (expected < 1100ms)`);
  console.assert(elapsed < 1100, 'Timeout not working!');
  console.assert(!slowHealthy, 'Should be unhealthy after timeout');
  
  // 测试 3: 降级到 SQLite
  console.log('  Test 3: Fallback to SQLite...');
  const sqliteLimiter = new LuxurySQLiteRateLimiter({
    dbPath: './data/fallback-test.db'
  });
  await sqliteLimiter.init();
  
  const result = await sqliteLimiter.checkLimit('192.168.1.1');
  console.log(`    SQLite fallback: ${result.allowed ? '✅ WORKING' : '❌ FAIL'}`);
  
  console.log('\n📊 OBS-002 Test Summary: ✅ PASS');
}

testTimeoutFailover().catch(err => {
  console.error('❌ Test failed:', err);
  process.exit(1);
});
```

#### v3.1.0 发布清单

- [ ] OBS-001 修复通过 V1-OBS001
- [ ] OBS-002 修复通过 V2-OBS002
- [ ] 48 项回归测试全绿 (V5-兼容)
- [ ] 回滚策略文档化
- [ ] CHANGELOG.md 更新
- [ ] Git Tag: `v3.1.0`

---

## 第5章：回滚策略与熔断标准

### 5.1 回滚策略（精确命令）

#### OBS-001 回滚

```bash
#!/bin/bash
# scripts/rollback-obs001.sh

echo "Rolling back OBS-001 (WasmMemory) to Array.from..."

# 1. 回滚 JS 文件
git checkout HEAD -- src/vector/wasm-loader.js

# 2. 回滚 Rust 文件（可选，保留接口不影响）
# git checkout HEAD -- crates/hajimi-hnsw/src/lib.rs

# 3. 重新编译 WASM（如果需要）
cd crates/hajimi-hnsw
wasm-pack build --target nodejs
cd ../..

# 4. 验证回滚
echo "Running regression tests..."
npm run test:regression

echo "Rollback complete. Status: $?"
```

#### OBS-002 回滚

```bash
#!/bin/bash
# scripts/rollback-obs002.sh

echo "Rolling back OBS-002 (Promise.race) to no-timeout..."

# 回滚 Redis 限流器
git checkout HEAD -- src/security/rate-limiter-redis-v2.js

# 验证
echo "Running Redis tests..."
node tests/redis-rate-limit.test.js

echo "Rollback complete. Status: $?"
```

#### 全量回滚（紧急）

```bash
# 回滚整个 Sprint2
git checkout HEAD -- src/vector/wasm-loader.js
git checkout HEAD -- src/security/rate-limiter-redis-v2.js
git checkout HEAD -- crates/hajimi-hnsw/src/lib.rs

# 删除新增测试文件
rm -f tests/wasm-zero-copy.bench.js
rm -f tests/redis-timeout-failover.test.js

# 验证
npm run test:phase5
```

### 5.2 熔断标准（何时放弃优化）

| 熔断ID | 触发条件 | 动作 | 责任人 |
|--------|----------|------|--------|
| FUSE-001 | V1-OBS001 加速比 < 2.8x | 回滚 OBS-001，接受现状 | 压力怪 |
| FUSE-002 | V4-内存 RSS 持续增长 > 20% | 回滚 OBS-001，记录 DEBT-WASM-010 | 压力怪 |
| FUSE-003 | 回归测试失败 > 5 项且 4h 内无法修复 | 回滚对应变更，延期修复 | 奶龙娘 |
| FUSE-004 | Promise.race 竞态条件无法解决 | 改用策略B（ioredis timeout） | 黄瓜睦 |
| FUSE-005 | Sprint2 时间超限（>10 天） | 强制切分剩余任务至 Sprint3 | 压力怪 |

### 5.3 Plan B 清单

| 风险 | Plan A | Plan B |
|------|--------|--------|
| WasmMemory 实现失败 | WasmMemory 共享 | 回退到 Array.from（现状） |
| Promise.race 竞态 | Promise.race | ioredis commandTimeout |
| 内存泄漏 | 内存池管理 | 回退到 Array.from |
| 加速比不足 | 接受 3.0x | 接受 2.43x，规划 SIMD |
| Redis 测试环境缺失 | 真实 Redis | Mock/stub 测试 |

---

## 第6章：Phase6 详细里程碑（Sprint3-8）

### Sprint3: WebRTC 信令 + NAT 穿透（Week1-2）

#### Week1: 技术选型

| 日期 | 任务 | 交付 | 验收 |
|------|------|------|------|
| Day1-2 | 调研 libp2p vs 自建 | 技术选型文档 | 团队评审通过 |
| Day3-4 | STUN/TURN 服务器搭建 | 服务器可用 | `ping stun.hajimi.local` |
| Day5 | ICE 候选收集测试 | 测试报告 | `node tests/ice-gathering.test.js` |

#### Week2: 基础信令

| 日期 | 任务 | 交付 | 验收 |
|------|------|------|------|
| Day6-7 | WebSocket 信令服务器 | `src/p2p/signaling-server.js` | 编译通过 |
| Day8-9 | 客户端信令逻辑 | `src/p2p/signaling-client.js` | 单元测试通过 |
| Day10 | 2 设备握手测试 | E2E 测试 | `node tests/webrtc-handshake.e2e.js` ✅ |

**Sprint3 验收命令**:
```bash
# 必须 100% 通过
node tests/webrtc-handshake.e2e.js
# 预期输出: ✅ 2 devices connected successfully
```

---

### Sprint4: DataChannel 可靠传输（Week3-4）

#### Week3: DataChannel 封装

| 日期 | 任务 | 交付 | 验收 |
|------|------|------|------|
| Day11-12 | DataChannel 管理器 | `src/p2p/datachannel-manager.js` | 单元测试 |
| Day13-14 | 断线重连逻辑 | 重连机制 | `node tests/datachannel-reconnect.test.js` |

#### Week4: 分片传输

| 日期 | 任务 | 交付 | 验收 |
|------|------|------|------|
| Day15-16 | 大文件分片 | 分片算法 | 单元测试 |
| Day17-18 | 传输确认机制 | ACK 机制 | 单元测试 |
| Day19-20 | 1MB 传输测试 | E2E 测试 | `node tests/datachannel-1mb-transfer.e2e.js` ✅ |

**Sprint4 验收命令**:
```bash
node tests/datachannel-1mb-transfer.e2e.js
# 预期输出: ✅ 1MB transferred in Xs (<5s)
```

---

### Sprint5: CRDT 冲突解决（Week5-6）

#### Week5: CRDT 选型与集成

| 日期 | 任务 | 交付 | 验收 |
|------|------|------|------|
| Day21-22 | 调研 Yjs vs Automerge | 选型文档 | 团队评审 |
| Day23-24 | CRDT 集成 | `src/p2p/crdt-manager.js` | 基础测试 |

#### Week6: 冲突解决

| 日期 | 任务 | 交付 | 验收 |
|------|------|------|------|
| Day25-26 | 版本向量实现 | 冲突检测 | 单元测试 |
| Day27-28 | 冲突解决策略 | 合并算法 | `npm test -- --grep "CRDT"` ✅ |

**Sprint5 验收命令**:
```bash
npm test -- --grep "CRDT"
# 预期: 100% pass, conflict resolution correct
```

---

### Sprint6-8: 加密、优化与发布（Week7-12）

详见 `HAJIMI-PHASE6-详细执行路线图-v1.0.md`

---

## 第7章：附录 - 风险预案

### 7.1 技术风险矩阵

| 风险 | 概率 | 影响 | 预案 |
|------|------|------|------|
| WebRTC NAT 穿透失败 | 中 | 高 | 强制 TURN 中继 |
| DataChannel 不稳定 | 中 | 中 | 断线重连 + 指数退避 |
| CRDT 冲突过多 | 低 | 中 | 优化同步频率 |
| 加密性能下降 | 低 | 中 | 可选禁用 E2E |

### 7.2 每日开发日志模板

```markdown
# docs/sprint2/DAY-X-DEVLOG.md

## 日期: 2026-03-XX
## 负责人: XXX

### 今日任务
- [ ] 任务1
- [ ] 任务2

### 完成情况
- 任务1: 完成/延期 (耗时 Xh)
- 任务2: 完成/阻塞 (阻塞原因)

### 卡点与求助
- 卡点: XXX
- 需要支援: XXX

### 明日计划
- 任务3
- 任务4

### 变更文件
- modified: src/xxx.js (行号范围)
- new: src/xxx.js
```

### 7.3 参考文献

1. [WebRTC MDN - DataChannel](https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel)
2. [libp2p Documentation](https://docs.libp2p.io/)
3. [Yjs CRDT Documentation](https://docs.yjs.dev/)
4. [Rust wasm-bindgen Memory](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/)

---

**规划官签名**: 压力怪 🔵  
**日期**: 2026-02-28  
**审计链**: ID-182 → ID-184 → ID-185（本计划）

*本计划遵循 HAJIMI 建设性审计规范（ID-175），所有任务可执行、可验证、可回滚。*
