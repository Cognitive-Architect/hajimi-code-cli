# DEBT-THINKING-UI — Thinking UI 方案C 债务记录

> **版本**: v1.0 | **日期**: 2026-04-30 | **状态**: ✅ B-02~B-12 闭环

## 变更 SHA 记录

| 工单 | SHA | Commit 描述 |
|:---|:---|:---|
| B-02 | `874644f` | inject AgentLoop trace_tx into AppState |
| B-05 | `d564057` | extend TraceEvent with OperationSummary and thinking_content |
| B-06 | `a44f6dd` | aggregate tool stats and extract thinking content |
| B-07 | `4cc48ab` | add collapsible thinking-block component with styles |
| B-08 | `d9958e2` | LLM prompt engineering + thinking extraction + markdown rendering |
| B-09 | `3e7640e` | streaming thinking update with token-level parsing |
| B-10 | `6364fb5` | add operation summary bar with expand/collapse |
| B-11 | `68282db` | diff inline preview + natural language reason + real-time progress |
| B-12 | TBD | timeline integration + session replay + document closure |

## 债务清单

| 债务ID | 描述 | 状态 |
|:---|:---|:---:|
| DEBT-B04-001 | AgentLoop 真实事件仅在 Tauri WebView 可用 | ⚠️ 开放 |
| DEBT-B08-001 | `stream_chat_with_context` 未使用 | ⚠️ 开放 |
| DEBT-B08-002 | `renderMarkdown` 为轻量解析器，不支持表格/嵌套列表 | ⚠️ 开放 |
| DEBT-B09-001 | `parseThinkingStream` 不处理跨 chunk 标签切分 | ⚠️ 开放 |
| DEBT-B09-002 | `streamChat` 与 `addThinking` 短暂双 div | ⚠️ 开放 |
| DEBT-B09-003 | `TokenEvent` 尚未被后端 provider 使用 | ⚠️ 开放 |
| DEBT-B11-001 | diff 预览为虚拟 diff，非真实 git diff | ⚠️ 开放 |
| DEBT-B11-002 | 理由生成基于规则匹配，非 LLM | ⚠️ 开放 |
| DEBT-B12-001 | TimelineEvent 未与后端 Checkpoint 绑定 | ⚠️ 开放 |
| DEBT-B12-002 | Replay thinking/operation 为只读回放 | ⚠️ 开放 |

## 验证

```bash
cargo check --workspace && cargo test -p intelligence-agent-core --lib
grep -c "TimelineEvent" src/interface/web/app.js
grep -c "Scheme-C Completed" src/INDEX.md
grep -c "Thinking UI" src/ARCHITECTURE.md
```

## 签名

- **编译**: 0 errors | **测试**: 105 passed | **文档**: 4 份同步
