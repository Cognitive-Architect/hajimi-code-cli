# DEBT-P1-TOKEN-TRACKER-INTEGRATION — P1 Token Tracker 集成清偿记录

> **工单批次**: P1-01/05 ~ P1-05/05
> **清偿日期**: 2026-04-30
> **最终 HEAD**: `d2f3de5`
> **来源债务**: `docs/debt/DEBT-SCHEME-B.md` 诚实声明 3 项已知限制

---

## 清偿摘要

| 检查项 | 实测命令 | 结果 |
|:---|:---|:---:|
| E2E 测试 | `cargo test -p codex-twist --test token_tracking_e2e` | 12 passed |
| 编译状态 | `cargo check --workspace` | 0 errors |
| 前端语法 | `node --check src/interface/web/app.js` | 通过 |
| Engine↔Intelligence 合规 | `grep codex_twist src/engine/` | 0 匹配 |
| Intelligence↔Interface 合规 | `grep "use.*interface" src/intelligence/` | 0 匹配 |
| token_tracker.rs 变更 | `git diff src/intelligence/codex-twist/src/memory/token_tracker.rs` | 零变更 |

---

## 清债记录

### DEBT-1: `TokenUsageTracker` 未集成到 `main.rs`

| 字段 | 内容 |
|:---|:---|
| **原状态** | 功能完整但 `interface/desktop/src/main.rs` 未实例化或使用 |
| **清偿工单** | P1-02/05 Backend 集成 |
| **修复位置** | `src/interface/desktop/src/main.rs` |
| **精确变更** | `AppState` 新增 `token_tracker: Arc<TokenUsageTracker>` 字段；`main()` 实例化 `Arc::new(TokenUsageTracker::new())`；`stream_chat` 获取 usage 后调用 `record_usage()` |
| **验证** | `grep "token_tracker" src/interface/desktop/src/main.rs` 返回多处匹配；`cargo check --workspace` 0 errors |
| **状态** | ✅ 已清偿 |

### DEBT-2: 前端 `cumulativeStats` 为内存内存储

| 字段 | 内容 |
|:---|:---|
| **原状态** | `cumulativeStats` 对象在页面刷新后丢失 |
| **清偿工单** | P1-04/05 Frontend 持久化 |
| **修复位置** | `src/interface/web/app.js` |
| **精确变更** | 新增 `loadCumulativeFromBackend()`（invoke `get_cumulative_stats`）+ `loadCumulativeFromLocalStorage()` + `saveCumulativeToLocalStorage()`；`init()` 调用恢复；`sendChatMessage()` finally 中保存 |
| **验证** | `grep "loadCumulativeFromBackend\|saveCumulativeToLocalStorage" src/interface/web/app.js` 返回多处匹配；`node --check src/interface/web/app.js` 通过 |
| **状态** | ✅ 已清偿 |

### DEBT-3: `exact-tokens` feature 默认关闭

| 字段 | 内容 |
|:---|:---|
| **原状态** | `tiktoken-rs` 为 optional 依赖，需显式启用 `exact-tokens` |
| **处理策略** | 维持 feature flag 设计决策，非债务项 |
| **说明** | 此约束为 intentional 设计：默认编译使用启发式估算，需精确计数时显式启用 `--features exact-tokens`。 Scheme B 已在 `B-02/06` 完成 `tiktoken-rs` 集成，`count_tokens()` 在 feature 启用时提供精确计数。 |
| **状态** | ⚪ 设计决策（非债务，保留 feature flag 灵活性）|

---

## 分层合规声明

- **Engine 层**: 零依赖 Intelligence（`codex_twist`），符合分层规则 ✅
- **Intelligence 层**: `TokenUsageTracker` 功能完整，由 Interface 层消费，无反向依赖 ✅
- **Interface 层**: `desktop`/`web` 已接入 Tracker，不影响下层纯洁性 ✅

---

## 关联文档

- `docs/debt/DEBT-SCHEME-B.md` — Scheme B 精确 Token 统计全批次债务汇总
- `src/ARCHITECTURE.md` — P1 Token Tracker Integration 架构设计
- `src/INDEX.md` — P1 工单映射与状态
- `src/MEMORY.md` — 数据诚实性与上下文债务基线

---

*本文件与代码同步维护，所有 metric 来自当天实测命令输出。*
