# DEBT-THINKING-UI-BASELINE — Thinking UI 方案C 基线债务记录

> **文档版本**: 1.0
> **生成日期**: 2026-05-08
> **Git 基线**: `acec06c`
> **状态**: 🔄 Day 1/12 基线测量完成，方案C 实施中
> **关联**: `docs/roadmap/Hajimi Thinking UI/investigation-report.md`

---

## 1. Baseline measured data

所有数据来自 2026-05-08 实测命令。

### 1.1 代码文件规模

| 文件 | 实测命令 | 行数 |
|:---|:---|:---:|
| `src/interface/desktop/src/main.rs` | `wc -l` | 1588 |
| `src/intelligence/agent-core/agent_loop.rs` | `wc -l` | 327 |
| `src/interface/web/app.js` | `wc -l` | 4111 |
| `src/interface/web/style.css` | `wc -l` | ~2500 |
| Rust 源文件总数 | `find src -name "*.rs" \| wc -l` | 249 |

### 1.2 关键断点实测证据

| 断点 | 位置 | 实测证据 |
|:---|:---|:---|
| trace_tx 初始化为 None | `main.rs:1521` | `trace_tx: std::sync::Mutex::new(None)` |
| subscribe_agent_trace 存在 | `main.rs:1242` | `async fn subscribe_agent_trace(...)` 骨架完整 |
| AgentLoop trace_tx 孤立 | `agent_loop.rs:85` | `trace_tx: Some(tokio::sync::broadcast::channel(64).0)`，无暴露接口 |
| MCP 模拟数据 | `trace_handler.ts:11` | `DEBT-W2-TRACE-DATA-001: Data source is a LoopState sequence generator` |
| addThinking 纯动画 | `app.js:2570` | 三跳动点，无推理文本传入 |
| renderTraceCards 浅渲染 | `app.js:1560` | 仅渲染 `step`+`details`，忽略 reflection_key_points 等富字段 |

### 1.3 编译与测试基线

| 验证项 | 命令 | 结果 |
|:---|:---|:---:|
| workspace 编译 | `cargo check --workspace` | 0 errors |
| agent-core lib 测试 | `cargo test -p intelligence-agent-core --lib` | 103 passed |
| memory lib 测试 | `cargo test -p memory --lib` | 150 passed |
| 四层纯洁性 | `grep -r "use.*interface" src/intelligence/memory/src/` | 0 |

---

## 2. 债务清单

### 2.1 P0 债务（阻断性）

| 债务ID | 描述 | 位置 | 计划清偿 |
|:---|:---|:---|:---|
| DEBT-TAURI-BRIDGE-001 | `trace_tx` 初始化为 `None`，`subscribe_agent_trace` 返回错误 | `main.rs:1521` | Day 2-3: Step 1 |
| DEBT-TAURI-BRIDGE-002 | `AgentLoop.trace_tx` 与 `AppState.trace_tx` 完全隔离 | `agent_loop.rs:85` | Day 2-3: Step 1 |
| DEBT-MCP-TRACE-001 | MCP Trace Handler 使用模拟数据 | `trace_handler.ts:11` | Day 4: Step 2 |

### 2.2 P1 债务（高优先级）

| 债务ID | 描述 | 位置 | 计划清偿 |
|:---|:---|:---|:---|
| DEBT-THINKING-RENDER-001 | Chat Thinking 仅显示动画，无推理文本 | `app.js:2570` | Day 7-9: Step 4 |
| DEBT-TRACE-RICH-001 | `renderTraceCards()` 忽略 plan_summary/reflection_key_points 等富字段 | `app.js:1560` | Day 4-5: Step 3 |
| DEBT-OPERATION-VIZ-001 | 无操作摘要条、无 diff 预览、无实时进度 | — | Day 10-11: Step 5 |

### 2.3 已知限制（非债务）

| 限制ID | 描述 | 状态 |
|:---|:---|:---:|
| LIMIT-STYLE-001 | `style.css` ~2500 行，新增 Thinking 样式需谨慎控制范围 | ⚪ 已知 |
| LIMIT-APPJS-001 | `app.js` ~4111 行，新增组件需最小化侵入 | ⚪ 已知 |

---

## 3. 验证命令清单

```bash
# 确认 trace_tx 为 None
grep -n "trace_tx" src/interface/desktop/src/main.rs

# 确认 AgentLoop trace_tx
grep -n "trace_tx" src/intelligence/agent-core/agent_loop.rs

# 确认 MCP 模拟数据
grep -n "TRACE_STEPS\|DEBT-W2" src/interface/mcp-server/handlers/trace_handler.ts

# 确认 Chat 纯动画
grep -n "addThinking\|removeThinking" src/interface/web/app.js

# 确认四层纯洁性
grep -r "use.*interface" src/intelligence/memory/src/; echo $?

# 编译检查
cargo check --workspace
```

---

*本债务文件与代码同步维护，metric 必须实测，禁止估算。*
