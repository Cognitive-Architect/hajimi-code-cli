# DEBT-P0-REMEDIATION — P0 上下文债务清偿记录

> **基线 Git SHA**: `848a9b032109bf77b794725b9c18aeff715a3cb2`  
> **清偿完成日期**: 2026-04-30  
> **关联工单**: B-01/09 ~ B-09/09（9 工单闭环）  
> **验证状态**: `cargo check --workspace` 0 errors，`node --check app.js` 通过  

---

## 债务清单总览

| ID | 债务项 | 严重程度 | 修复工单 | 状态 |
|:---|:---|:---:|:---|:---:|
| DEBT-001 | 单轮 LLM 接口 — `stream_chat(String)` 不支持消息数组 | 🔴 P0 | B-02/09 ~ B-03/09 | ✅ 已清偿 |
| DEBT-002 | 后端无 messages — `stream_chat` command 仅接收 `prompt: String` | 🔴 P0 | B-04/09 ~ B-05/09 | ✅ 已清偿 |
| DEBT-003 | MemoryGateway 孤岛 — `codex-twist` 完整实现但完全未被引用 | 🔴 P0 | B-05/09 ~ B-08/09 | ✅ 已清偿 |
| DEBT-004 | 前端无对话状态 — 聊天历史仅 DOM 渲染，无状态数组 | 🔴 P0 | B-06/09 ~ B-08/09 | ✅ 已清偿 |

---

## 逐项清偿记录

### DEBT-001: 单轮 LLM 接口

**问题描述**  
`LlmClient` trait 仅暴露 `stream_chat(&self, prompt: String)`，所有 Provider（Anthropic/OpenAI/Ollama）请求体仅含单条 `user` 消息。AI 每次回复都是"失忆"状态。

**修复前代码位置**
```rust
// src/engine/llm-core/src/mod.rs:L133（基线 848a9b0）
async fn stream_chat(&self, prompt: String) -> Result<ChannelStream, EngineError>;
```

**修复后代码位置**
```rust
// src/engine/llm-core/src/mod.rs:L149（当前）
async fn stream_chat_with_context(
    &self,
    messages: Vec<ChatMessage>,
    system_prompt: Option<String>,
) -> Result<ChannelStream, EngineError>;
```

**Provider 实现验证**
- `src/engine/llm-core/src/openai.rs` L31-80: `stream_chat_with_context` 完整实现，请求体 `messages` 字段传递完整消息数组
- `src/engine/llm-core/src/anthropic.rs` L39-80: `stream_chat_with_context` 完整实现，支持 `system` 顶级参数
- `src/engine/llm-core/src/ollama.rs`: `stream_chat_with_context` 完整实现

**实测验证命令**
```bash
cargo check --package engine-llm-core  # 0 errors
grep -n "stream_chat_with_context" src/engine/llm-core/src/mod.rs  # L149
grep -n "stream_chat_with_context" src/engine/llm-core/src/openai.rs  # L31
grep -n "stream_chat_with_context" src/engine/llm-core/src/anthropic.rs  # L39
```

**前后对比**
| 指标 | 修复前 | 修复后 |
|:---|:---|:---|
| LLM 接口 | `stream_chat(String)` 单轮 | `stream_chat_with_context(Vec<ChatMessage>, Option<String>)` 多轮 |
| Provider 覆盖 | 0/3 支持多轮 | 3/3 支持多轮 |
| 向后兼容 | — | `stream_chat` 保留，内部调用 `stream_chat_with_context` |

---

### DEBT-002: 后端无 messages

**问题描述**  
`interface/desktop/src/main.rs` 的 `stream_chat` Tauri command 仅接收 `prompt: String`，不接收消息历史。后端无法将多轮上下文传递给 LLM。

**修复前代码位置**
```rust
// src/interface/desktop/src/main.rs:L824（基线 848a9b0）
async fn stream_chat(provider: String, prompt: String, config: Option<ProviderConfig>, ...) { ... }
```

**修复后代码位置**
```rust
// src/interface/desktop/src/main.rs:L830（当前）
async fn stream_chat(
    provider: String,
    prompt: String,
    messages: Option<Vec<ChatMessage>>,
    config: Option<ProviderConfig>,
    ...
) { ... }
```

**实测验证命令**
```bash
grep -n "messages: Option<Vec<ChatMessage>>" src/interface/desktop/src/main.rs  # L833
grep -n "memory_gateway: Arc<MemoryGateway>" src/interface/desktop/src/main.rs  # L70
```

**前后对比**
| 指标 | 修复前 | 修复后 |
|:---|:---|:---|
| stream_chat 参数 | `provider, prompt, config` | `provider, prompt, messages, config` |
| 消息历史传递 | ❌ 不传递 | ✅ `messages` Option 传递完整数组 |
| MemoryGateway 注入 | ❌ 未引用 | ✅ `AppState` 含 `memory_gateway: Arc<MemoryGateway>` |
| 上下文持久化 | ❌ 无 | ✅ `gateway.working().put(session_key, ctx_json)` |

---

### DEBT-003: MemoryGateway 孤岛

**问题描述**  
`intelligence/codex-twist/src/memory/memory_gateway.rs` 94 行完整实现（Focus/Working/Archive 三层 + `optimize()`），但 `main.rs` 完全未引用 `codex_twist`，`grep` 返回 0 匹配。`optimize()` 为占位实现，返回格式化字符串。

**修复前代码位置**
```rust
// src/intelligence/codex-twist/src/memory/memory_gateway.rs:L61（基线 848a9b0）
pub async fn optimize(&self, target: &str) -> String {
    format!("Optimized for {}", target)
}
```

**修复后代码位置**
```rust
// src/intelligence/codex-twist/src/memory/memory_gateway.rs:L62-97（当前）
pub async fn optimize(
    &self,
    messages: Vec<ChatMessage>,
    client: &dyn LlmClient,
) -> Result<String, String> {
    if messages.len() <= 2 { return Ok("对话轮次不足，无需压缩".to_string()); }
    let summary_msgs = messages[..messages.len() - 2].iter().map(...).collect();
    let system_prompt = "请将以下对话历史压缩为一段简洁的摘要...".to_string();
    let mut stream = client.stream_chat_with_context(summary_msgs, Some(system_prompt)).await?;
    let mut summary = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk { StreamChunk::Output(t) => summary.push_str(&t), ... }
    }
    Ok(summary.trim().to_string())
}
```

**实测验证命令**
```bash
grep -n "codex_twist\|MemoryGateway" src/interface/desktop/src/main.rs  # L6, L70, L855, L903, L960
grep -n "optimize" src/intelligence/codex-twist/src/memory/memory_gateway.rs  # L62
grep -n "stream_chat_with_context\|LlmClient" src/intelligence/codex-twist/src/memory/memory_gateway.rs  # L5, L83
```

**前后对比**
| 指标 | 修复前 | 修复后 |
|:---|:---|:---|
| main.rs 引用 codex_twist | 0 处 | 5 处（import + AppState + 3 调用点） |
| optimize() 实现 | 占位，返回格式化字符串 | 真实 LLM 驱动摘要，保留最近 2 轮 |
| 分层依赖 | — | codex-twist 新增 `engine-llm-core` 依赖，Intelligence→Engine 合规 |

---

### DEBT-004: 前端无对话状态

**问题描述**  
`interface/web/app.js` 聊天历史仅渲染在 DOM（`aiChatMessages`），无 `chatMessages` 状态数组。无 `/compact` 命令，无 Token 估算，无自动压缩。

**修复前代码位置**
```javascript
// src/interface/web/app.js（基线 848a9b0）
// 无 chatMessages 状态
// handleChatCommand 中无 /compact 分支
// streamChat(provider, prompt, config) 仅传递 prompt
```

**修复后代码位置**
```javascript
// src/interface/web/app.js:L28（当前）
chatMessages: [],
autoCompact: true,
isAutoCompacting: false,

// L1876-1883: updateTokenDisplay() — Token 估算显示
// L1885-1904: checkAutoCompact() — 80% 阈值自动触发
// L1906-1927: autoCompactContext() — 自动压缩逻辑
// L2306-2329: /compact slash 命令 — 手动触发真实 LLM 摘要
// L1958: sendChatMessage 中 push user 消息
// L2018: streamChat 完成后 push assistant 消息
```

**实测验证命令**
```bash
grep -n "chatMessages" src/interface/web/app.js | wc -l  # 20+ 处
grep -n "/compact" src/interface/web/app.js  # L2306
grep -n "autoCompact\|threshold.*0.8" src/interface/web/app.js  # L30, L1891
grep -n "estimateTokens\|updateTokenDisplay" src/interface/web/app.js  # L1869, L1876
```

**前后对比**
| 指标 | 修复前 | 修复后 |
|:---|:---|:---|
| 消息状态 | ❌ 无 | ✅ `chatMessages: []` 数组，含 role/content/timestamp |
| `/compact` 命令 | ❌ 无 | ✅ 手动触发，调用 `optimize_context` 获取真实摘要 |
| 自动压缩 | ❌ 无 | ✅ Token > threshold*0.8 时自动触发，有开关 |
| Token 估算 | ❌ 无 | ✅ 字符启发式（中文≈1，英文≈1.3），显示在 composer-hint |
| 压缩策略 | — | 保留最近 2 轮 + system 摘要消息 |

---

## 分层合规声明

- **Engine 层** (`engine/llm-core`): `stream_chat_with_context` 已添加，三 Provider 全部实现。不依赖上层，分层合规 ✅
- **Intelligence 层** (`codex-twist`): `MemoryGateway` 新增 `engine-llm-core` 依赖，`optimize()` 通过 Engine 层 LLM 客户端完成，不直接 HTTP。Intelligence→Engine 合规 ✅
- **Interface 层** (`desktop`/`web`): `main.rs` 注入 `MemoryGateway`，`app.js` 维护 `chatMessages`。不违反下层纯洁性 ✅

---

## 验证汇总

| 验证项 | 命令 | 结果 |
|:---|:---|:---:|
| 全 workspace 编译 | `cargo check --workspace` | ✅ 0 errors |
| 前端语法检查 | `node --check src/interface/web/app.js` | ✅ 通过 |
| LLM trait 多轮接口 | `grep "stream_chat_with_context" src/engine/llm-core/src/mod.rs` | ✅ L149 |
| 后端 messages 参数 | `grep "messages: Option" src/interface/desktop/src/main.rs` | ✅ L833 |
| MemoryGateway 注入 | `grep "memory_gateway" src/interface/desktop/src/main.rs` | ✅ 5 处 |
| optimize 真实实现 | `grep "stream_chat_with_context" src/intelligence/codex-twist/src/memory/memory_gateway.rs` | ✅ L83 |
| 前端消息状态 | `grep "chatMessages" src/interface/web/app.js` | ✅ 20+ 处 |
| 自动压缩阈值 | `grep "threshold.*0.8" src/interface/web/app.js` | ✅ L1891 |
| /compact 命令 | `grep "/compact" src/interface/web/app.js` | ✅ L2306 |

---

## 关联文档交叉引用

| 文档 | 路径 | 内容 |
|:---|:---|:---|
| 源代码索引 | `src/INDEX.md` | P0-CONTEXT-REMEDIATION 章节，含债务清单与清偿状态 |
| 架构文档 | `src/ARCHITECTURE.md` | codex-twist 状态更新为"已激活"，债务注释已清偿 |
| 数据诚实性基线 | `src/MEMORY.md` | P0 清偿记录与验证汇总 |
| 根因分析 | `docs/roadmap/Hajimi Context/03-context-compaction.md` | 状态更新为"已修复"，含修复后代码位置 |

---

*本文档与代码同步维护。所有数据基于 `cargo check`、`grep`、`node --check` 等命令的真实输出，禁止估算。*  
*P0 Context Debt Cleared ✅*
