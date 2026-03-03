# 20-AUDIT-PHASE4-DEBT-REVIEW.md

> **审计编号**: 20
> **审计对象**: HAJIMI-PHASE4 债务复核（Task 17 — 遗留债务落地可行性评估）
> **审计员**: Mike（技术风险顾问）
> **日期**: 2026-02-27
> **审计链**: 09→10→12→13→14→15→16→17→19→**20** ✅
> **前置报告**: 19号（B+/Go，债务诚实申报）

---

## 第一章：总体健康度

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| **债务申报诚实度** | **A** | 四项债务均有量化数据，无掩盖 |
| **代码落地质量** | **A-** | 核心路径完整，存在两处结构性缺陷 |
| **降级链可靠性** | **B+** | Redis→SQLite链路完整，WASM→JS链路有隐患 |
| **测试覆盖真实性** | **B** | 85%通过率，但WASM-Rust 69%含环境因素 |
| **Phase 5 放行条件** | **B+** | 可放行，但两项P1债务须在Phase 5 Sprint 1内处理 |

### 总体评级：**B+/Go（附条件）**

---

## 第二章：风险详情

### RISK-01 ｜ `_prune_connections` 是空函数 ｜ 等级：C

**位置**: `crates/hajimi-hnsw/src/lib.rs:411-420`

```rust
fn _prune_connections(&self, node_id: u32, level: u8) {
    if let Some(node) = self.nodes.get(&node_id) {
        if let Some(connections) = node.connections.get(&level) {
            if connections.len() > self.m * 2 {
                // 需要裁剪 - 由于self.nodes是HashMap，我们需要延迟处理
                // 这里简化处理，实际应该根据距离排序后裁剪
            }
        }
    }
}
```

**不修复会发生什么**：随着节点数增长，每个节点的连接数无上限膨胀。10万节点规模下，单节点连接数可能超过 `2*M=32` 的设计上限，导致：
1. 搜索时遍历邻居数量失控，查询延迟从 O(log N) 退化为 O(N)
2. WASM 内存占用线性增长，触发 400MB 上限（`MemoryManager::max_memory`）
3. 查询加速比从当前 2.43x 进一步下降，DEBT-WASM-002 的 5x 目标更难达到

**落地路径**：
- 短期（Phase 5 Sprint 1）：将 `_prune_connections` 改为 `&mut self`，按距离排序后截断到 `self.m * 2`，约 15 行代码
- 成本：低（单函数修改，不影响接口）
- 收益：高（防止生产环境内存泄漏，同时提升查询加速比）

---

### RISK-02 ｜ `WASMLoader` 单例无并发保护 ｜ 等级：C

**位置**: `src/vector/wasm-loader.js:199-211`

```js
let loaderInstance = null;

async function getWASMLoader() {
  if (!loaderInstance) {
    loaderInstance = new WASMLoader();
    await loaderInstance.init();  // ← 异步间隙
  }
  return loaderInstance;
}
```

**不修复会发生什么**：Node.js 事件循环中，多个并发请求同时调用 `getWASMLoader()` 时，`loaderInstance` 在 `init()` 完成前仍为 `null`，导致多个 `WASMLoader` 实例被创建。每个实例独立持有 WASM 模块引用，造成：
1. 内存重复占用（WASM 模块约 2-5MB，N 个并发 = N 倍占用）
2. 多实例状态不一致（各自维护独立索引，搜索结果不可预期）
3. 服务启动阶段（高并发初始化）最易触发

**落地路径**：
- 短期：用 Promise 缓存替代裸变量，`loaderInstance = WASMLoader().init()` 返回 Promise，后续调用直接 `await` 同一个 Promise
- 成本：极低（5 行改动）
- 收益：中（防止启动阶段内存翻倍，提升稳定性）

---

### RISK-03 ｜ Redis 降级后无法自动恢复 ｜ 等级：B

**位置**: `src/security/rate-limiter-factory.js:111-124`

```js
} catch (err) {
  if (factory.fallbackLimiter && factory.currentMode !== 'sqlite') {
    factory.currentMode = 'sqlite';
    factory.primaryLimiter = factory.fallbackLimiter;
    factory.fallbackLimiter = null;  // ← 降级后 fallback 被清空
    return factory.primaryLimiter.checkLimit(ip, tokens);
  }
  throw err;
}
```

**不修复会发生什么**：Redis 故障恢复后，工厂永久停留在 SQLite 模式，无法自动切回 Redis。对于分布式部署场景（多机共享限流），这意味着：
1. Redis 恢复后各机器独立计数，限流失去分布式语义
2. 需要手动重启服务才能恢复 Redis 模式，运维成本高
3. DEBT-REDIS-002（真实 Redis 验证）通过后，此问题会在生产环境暴露

**落地路径**：
- 中期（Phase 5 Sprint 2）：增加后台健康检查定时器，Redis 恢复后重新初始化并切回主模式
- 短期兜底：至少保留 `fallbackLimiter` 引用，不要置 `null`，防止二次故障时无降级可用
- 成本：中（需要状态机设计，约 50 行）
- 收益：高（分布式场景的核心可靠性保障）

---

### RISK-04 ｜ `JSIndexWrapper.stats()` 维度字段语义错误 ｜ 等级：B

**位置**: `src/vector/wasm-loader.js:183-195`

```js
stats() {
  const s = this._index.getStats();
  return {
    nodeCount: s.elementCount,
    maxLevel: s.maxLevel,
    dimension: s.config?.maxElements, // 近似  ← 语义错误
    m: s.config?.M,
    mode: this._mode
  };
}
```

**不修复会发生什么**：`dimension` 字段返回的是 `maxElements`（最大容量），而非向量维度。调用方若依赖 `stats().dimension` 做维度校验（如动态路由、监控告警），会得到错误数据。当前 `WASMIndexWrapper.stats()` 返回正确的 `dimension`，两个包装器行为不一致，降级切换时会产生静默数据错误。

**落地路径**：
- 短期：在 `JSIndexWrapper` 构造时记录 `dimension` 参数，`stats()` 直接返回
- 成本：极低（构造函数加一个字段）
- 收益：中（防止监控/路由层静默错误）

---

## 第三章：可维护性评估

| 模块 | 可维护性 | 说明 |
|:---|:---:|:---|
| `lib.rs` HNSW 核心 | **B+** | 结构清晰，但 `_prune_connections` 是定时炸弹 |
| `wasm-loader.js` | **B** | 单例并发问题 + stats 语义错误，两处隐患并存 |
| `rate-limiter-redis.js` | **A-** | Lua 脚本原子性设计正确，接口规范 |
| `rate-limiter-factory.js` | **B** | 降级逻辑正确，但无恢复机制，长期运维成本高 |

**整体可维护性：B+**

代码结构和接口设计质量高，主要风险集中在"边界状态处理"层面（并发初始化、故障恢复、数据语义一致性），这类问题在单元测试中不易暴露，但在生产环境高并发或故障场景下必然触发。

---

## 第四章：落地建议

### 短期（Phase 5 Sprint 1，必须）

| 优先级 | 任务 | 文件 | 工时估算 |
|:---:|:---|:---|:---:|
| P0 | 实现 `_prune_connections` 裁剪逻辑 | `lib.rs:411` | 0.5h |
| P0 | 修复 `getWASMLoader` 并发初始化 | `wasm-loader.js:201` | 0.5h |
| P1 | 修复 `JSIndexWrapper.stats().dimension` | `wasm-loader.js:187` | 0.25h |

### 中期（Phase 5 Sprint 2，建议）

| 优先级 | 任务 | 文件 | 工时估算 |
|:---:|:---|:---|:---:|
| P1 | Redis 故障恢复自动切回机制 | `rate-limiter-factory.js` | 2h |
| P1 | DEBT-REDIS-002：真实 Redis 环境验证 | `tests/redis-rate-limit.test.js` | 1h |

### 长期（Phase 5 Sprint 3，可选）

| 优先级 | 任务 | 说明 |
|:---:|:---|:---|
| P2 | DEBT-WASM-002：SharedArrayBuffer 查询加速 | 达成 5x 目标的关键路径 |
| P2 | DEBT-WASM-003：wasm-opt 启用 | 网络环境解决后执行 |

---

## 第五章：结论

### 放行判定：**B+/Go（附条件）**

**可以放行 Phase 5 的理由**：
- 核心功能链路完整（WASM 加载、降级、Redis 限流、工厂模式）
- 债务申报诚实，无掩盖，19 号报告数据可信
- 两项 P0 修复工时合计 ≤ 1h，不阻塞 Phase 5 启动

**附加条件**：
1. **RISK-01 和 RISK-02 须在 Phase 5 Sprint 1 第一个 PR 中修复**，不得推迟
2. RISK-03 降级恢复机制须在 Sprint 2 内完成，否则分布式部署不可用
3. DEBT-REDIS-002 须在有真实 Redis 环境时立即执行，不得以"环境限制"为由无限期推迟

**审计链完整性**：09→10→12→13→14→15→16→17→19→20 ✅ 连续无断号

---

*审计结论：Phase 4 代码质量达标，债务申报诚实，两处 P0 缺陷须在 Phase 5 Sprint 1 内修复，整体可继续推进。*
