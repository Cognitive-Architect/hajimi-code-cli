# DEBT-SCHEME-B — Scheme B 精确 Token 统计全批次债务汇总

> **批次**: B-01/06 ~ B-06/06
> **状态**: 已完成
> **最终 commit**: `B-06/06`
> **最后更新**: 2026-04-30

---

## 批次交付清单

| 工单 | Phase | 交付物 | 行数 | 熔断状态 |
|:---|:---|:---|:---:|:---:|
| B-01/06 | Day 1 Baseline | INDEX/ARCHITECTURE/MEMORY/01.md 同步 | ~180 | 未触发 |
| B-02/06 | Day 2 Engine 精确计数 | `tiktoken-rs` + `count_tokens()` + 7 测试 | ~231 | 未触发 |
| B-03/06 | Day 3 Backend usage 解析 | `Usage` struct + `last_usage()` + Audit precise | ~227 | 未触发 |
| B-04/06 | Day 4 Intelligence 统计聚合 | `TokenUsageTracker` + 6 测试 | ~137 | 未触发 |
| B-05/06 | Day 5 Frontend 精确 UI | `StreamEvent` usage + `app.js` UI 升级 | ~146 | 未触发 |
| B-06/06 | Day 6 Integration Closeout | E2E 测试 12 + 文档闭环 + 清债 | ~120 | 未触发 |

---

## DEBT-LINES 汇总

| 批次 | DEBT-ID | 差异 | 原因 | 状态 |
|:---|:---|:---:|:---|:---:|
| B-01/06 ~ B-06/06 | 无 | — | 所有批次均未触发弹性行数熔断 | ✅ 已清偿 |

---

## 已知限制（诚实声明）

1. **`TokenUsageTracker` 未集成到 `main.rs`**
   - 位置: `src/intelligence/codex-twist/src/memory/token_tracker.rs`
   - 状态: ✅ **已清偿**（P1-02/05, HEAD `d2f3de5`）
   - 修复: `AppState` 新增 `token_tracker: Arc<TokenUsageTracker>`，`stream_chat` 调用 `record_usage()`
   - 验证: `cargo check --workspace` 0 errors；`grep token_tracker src/interface/desktop/src/main.rs` 多处匹配

2. **前端累计为内存内存储**
   - 位置: `src/interface/web/app.js`
   - 状态: ✅ **已清偿**（P1-04/05, HEAD `d2f3de5`）
   - 修复: 混合持久化 — `loadCumulativeFromBackend()`（Tauri Command 主路径）+ `localStorage` 兜底
   - 验证: `node --check src/interface/web/app.js` 通过

3. **`exact-tokens` feature 默认关闭**
   - 位置: `src/engine/llm-core/Cargo.toml`
   - 状态: ⚪ **设计决策保留**（非债务）
   - 说明: `tiktoken-rs` 为 optional 依赖，需显式启用 `exact-tokens`。默认编译使用启发式估算，精确计数时启用 `--features exact-tokens`。已在 B-02/06 完成集成。
   - 验证: `cargo test -p codex-twist --test token_tracking_e2e` 12 passed

---

## 验证证据

```powershell
# E2E 测试
cargo test -p codex-twist --test token_tracking_e2e   # 12 passed

# 全量 token/usage 测试计数
cargo test 2>&1 | grep -c "test.*token\|test.*usage"   # ≥ 12

# 编译检查
cargo check --workspace                                 # 0 errors

# 前端检查
node --check src/interface/web/app.js                   # 通过

# 分层合规
grep -r "use.*codex_twist" src/engine/                  # 0 匹配
grep -r "use.*interface" src/intelligence/              # 0 匹配
```

---

*本文件与代码同步维护。Scheme B 全批次已完成，无未清偿 DEBT-LINES。*
