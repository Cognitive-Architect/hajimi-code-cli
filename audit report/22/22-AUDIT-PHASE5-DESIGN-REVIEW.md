# 审计报告 #22 — Phase 5 设计评审
**任务来源**: task-audit/19.md
**审计日期**: 2026-02-27
**审计员**: Mike (审计汪)
**评级**: B+ / Go（附条件放行）

---

## 总体健康度

| 维度 | 评级 | 说明 |
|------|------|------|
| 架构设计 | A | SAB零拷贝思路正确，分层清晰 |
| 代码实现 | B | 核心逻辑完整，存在2处设计缺陷 |
| 可维护性 | B+ | 模块边界清晰，注释充分 |
| 落地可行性 | B | SAB在Worker环境有前提条件限制 |

**综合评级：B+**，Phase 5 可启动，但 RISK-01 须在 Sprint 1 内修复，否则 SAB 优化在生产环境中将静默失效。

---

## 风险详情

### RISK-01（C级）— SAB 在 Worker 环境缺少 COOP/COEP 前提检查

**位置**: `src/vector/hnsw-index-wasm-v3.js:27`

```js
this.buffer = new SharedArrayBuffer(this.config.initialSize);
```

**问题**：`SharedArrayBuffer` 自 Chrome 92 起要求页面设置 `Cross-Origin-Opener-Policy: same-origin` + `Cross-Origin-Embedder-Policy: require-corp`。在 Node.js Worker Threads 环境中，SAB 可用，但代码在 `init()` 中创建 SAB 时没有任何环境检测，若运行在不支持 SAB 的上下文（如旧版 Electron、某些 CI 环境），会直接抛出 `TypeError`，且错误信息不友好。

**不修复会导致**：SAB 初始化失败时，`catch` 块（`hnsw-index-wasm-v3.js:159`）会静默降级，但 `_sabPool` 为 null，后续所有 `insert()` 调用都走普通路径，**5x 加速目标完全失效**，且无任何告警日志说明原因，排查成本极高。

**修复路径**：在 `SABMemoryPool` 构造函数中加入环境检测：
```js
if (typeof SharedArrayBuffer === 'undefined') {
  throw new Error('SharedArrayBuffer not available in this environment');
}
```
并在降级时输出明确的 WARN 日志，说明 SAB 不可用原因。
**工时估算**：0.5h

---

### RISK-02（B级）— `searchBatch` 未利用 SAB，批量搜索优化名不副实

**位置**: `src/vector/hnsw-index-wasm-v3.js:225-243`

```js
searchBatch(queries, k = 10) {
  for (const query of queries) {
    results.push(this._index.search(query, k));  // 逐条调用
  }
}
```

**问题**：`searchBatch` 的设计意图是"减少 JS↔WASM 边界跨越开销"，但实现上仍是逐条调用 `this._index.search()`，每次调用都有完整的 WASM 边界跨越开销。SAB 内存池（`_sabPool`）在搜索路径中完全未被使用——向量数据已存入 SAB，但搜索时仍从 JS 侧传入 `query`，没有利用 SAB 的零拷贝优势。

**不修复会导致**：Phase 5 的核心性能指标（查询加速比 ≥5x）无法达成。白皮书中承诺的 SAB 零拷贝优化实际上只在 `insert` 路径有效（且仅是存储，不影响 WASM 内部计算），搜索路径完全没有优化。基准测试结果会与预期严重偏差，Phase 5 验收存在风险。

**修复路径**：需要在 Rust 侧暴露 `search_from_sab(offset, k)` 接口，接受 SAB 内的 offset 而非 JS 传入的向量数组。这是较大改动，建议列为 Phase 5 Sprint 2 的核心任务，Sprint 1 先完成基础功能验证。
**工时估算**：4-8h（含 Rust 侧接口修改 + 重新编译 WASM）

---

### RISK-03（B级）— Redis V2 健康恢复无主动重连机制

**位置**: `src/security/rate-limiter-redis-v2.js:146-157`

```js
_startHealthCheck() {
  this.healthCheckTimer = setInterval(async () => {
    const healthy = await this.healthCheck();
    if (!healthy && this.state.consecutiveFailures > 3) {
      console.warn('[RedisV2] Health check failed, marking unhealthy');
    }
  }, this.config.healthCheckInterval);
}
```

**问题**：健康检查只做检测，不做恢复。当 Redis 从故障中恢复后，`isHealthy` 标志位会在下次 `healthCheck()` 成功时被置为 `true`（`rate-limiter-redis-v2.js:134`），这部分逻辑是正确的。但 `checkLimit()` 中的降级判断（`line:166`）在 `isHealthy=false` 时直接抛出异常，不会等待健康检查周期（默认 30s）自动恢复。

**不修复会导致**：Redis 短暂抖动（如重启）后，即使 Redis 已恢复，系统仍会在最长 30s 内持续触发降级，所有分布式限流请求都走 fallback，限流语义失效，高流量场景下存在被绕过的风险。

**修复路径**：在 `checkLimit` 检测到 `!isHealthy` 时，先尝试一次 `healthCheck()`，若成功则继续正常流程。或将健康检查间隔缩短为 5s。
**工时估算**：1h

---

### RISK-04（A级）— `_prune_connections` 两阶段借用策略已验证，无新风险

**位置**: `crates/hajimi-hnsw/src/lib.rs:412-480`

代码抽查确认：Task 18 修复的两阶段借用策略（先不可变借用检查 → clone 数据 → 可变借用修改）实现正确，逻辑完整，无回归。

---

## 可维护性评估

- `HNSWIndexWASMV3` 与 `SABMemoryPool` 职责分离清晰，可独立测试 ✅
- `RedisRateLimiterV2` 状态机（`isConnected` / `isHealthy` / `consecutiveFailures`）语义清晰，但三个字段存在冗余，`isConnected=true` 时 `isHealthy` 可能为 `false`，调用方需理解两者区别，文档缺失
- `lib.rs` 注释充分，算法步骤有中文说明，可维护性良好

---

## 落地建议

**Sprint 1（阻塞项，须完成后才能进行性能基准测试）**：
1. 修复 RISK-01：SAB 环境检测 + 降级日志（0.5h）
2. 修复 RISK-03：Redis 健康恢复主动重连（1h）

**Sprint 2（性能目标达成的前提）**：
3. 修复 RISK-02：Rust 侧暴露 `search_from_sab` 接口，实现真正的 SAB 零拷贝搜索（4-8h）

**Sprint 3（技术债清理）**：
4. 补充 `RedisRateLimiterV2` 状态字段文档，明确 `isConnected` vs `isHealthy` 的语义差异

---

## 结论

Phase 5 设计方向正确，SAB 零拷贝 + Redis V2 加固的组合具备落地可行性。当前主要问题是**实现与设计意图存在落差**：SAB 优化在搜索路径上尚未生效，Redis 恢复机制不完整。

**放行条件**：Sprint 1 的 RISK-01 + RISK-03 修复完成后，可进行 Phase 5 功能验收。RISK-02 须在性能基准测试前完成，否则 5x 加速目标无法验证。

**下一份审计建议**：Sprint 2 完成后，对 `search_from_sab` 接口实现 + 基准测试结果进行专项审计。
