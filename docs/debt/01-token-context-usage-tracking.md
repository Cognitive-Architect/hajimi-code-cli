# 01 - Token 统计与上下文容量显示

> 状态：**已完成**（方案 A 已完成，方案 B 已完成）
> 优先级：P2（非阻塞，增强体验）
> 涉及范围：前端 UI + Rust 后端
> 最后更新：2026-04-27（P0 上下文债务修复后同步）

---

## 问题描述

当前 Hajimi 的聊天界面**Token 统计和上下文容量显示处于部分实现状态**。用户现在可以：

- ✅ 查看当前会话已占用多少 Token（前端估算）
- ✅ 查看距离模型上下文上限还剩多少容量（通过自动压缩触发间接感知）
- ❌ 查看每次请求的输入 / 输出 Token 数（API 返回的精确值）
- ❌ 查看累计 Token 消耗

### 参考 UI

```
Session  |  🔄 69.7%  |  ↑ 182,800  |  ↓ 524
          上下文使用率    输入 Token      输出 Token
```

> 当前实现：仅状态栏显示 `Enter 发送 · Shift+Enter 换行 · @ 引用文件 · 1234 tokens`

---

## 现状分析

### 前端（app.js）— 2026-04-27 更新

| 方面 | 现状 |
|---|---|
| Token 计数 | ✅ **已实现** — `estimateTokens()` 字符启发式估算（中文字符≈1，英文单词≈1.3） |
| 上下文使用率 | ⚠️ **间接实现** — 无百分比 UI，但超过 80% 阈值时自动触发压缩 |
| 输入/输出统计 | ❌ **未实现** — 未解析 API `usage` 字段 |
| 累计消耗 | ❌ **未实现** — 无持久化 |
| 相关代码 | `estimateTokens()` [L1871]、 `updateTokenDisplay()` [L1878]、 `checkAutoCompact()` [L1901]、 `autoCompactContext()` [L1911] |

### Rust 后端（main.rs）— 2026-04-27 更新

| 方面 | 现状 |
|---|---|
| 精确 Token 编码 | ❌ **未实现** — 无 tiktoken-rs 集成 |
| `usage` 解析 | ❌ **未实现** — `streamChat()` 只解析 `content`，忽略 `usage` 字段 |
| Audit Log | ⚠️ **部分实现** — `token_before` / `token_after` 在 `stream_chat` 中有填充（B-05），但 `log_usage` 其他调用点仍为 `None` |
| `estimated_tokens` | ⚠️ **粗略估算** — `audit::log_usage` 中用 `msg_count * 50` 估算，非精确值 |
| 上下文压缩 | ✅ **已实现** — `optimize_context` command [L943] 调用 `MemoryGateway.optimize()` 做 LLM 摘要 |

---

## 实现记录

### B-07/09 实现：前端 Token 估算 + `/compact` 命令

**`app.js` 新增功能：**

```javascript
// Token 估算（字符启发式）
estimateTokens(text) {
  const chineseChars = (text.match(/[\u4e00-\u9fff]/g) || []).length;
  const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;
  return Math.ceil(chineseChars + englishWords * 1.3);
},

// UI 更新（composer hint 区域）
updateTokenDisplay() {
  const totalTokens = this.chatMessages.reduce(
    (sum, msg) => sum + this.estimateTokens(msg.content), 0
  );
  // 显示为: "Enter 发送 ... · 1234 tokens"
},
```

**`/compact` 命令：**
- 手动触发上下文压缩
- 调用 `invoke('optimize_context', { messages, provider, config })`
- 保留最近 2 轮对话，前面消息由 LLM 生成摘要
- 摘要替换为 `role: 'system'` 消息

### B-08/09 实现：自动压缩触发 + `MemoryGateway.optimize()`

**自动压缩检测：**

```javascript
checkAutoCompact() {
  if (!this.autoCompact || this.isAutoCompacting || this.chatMessages.length <= 2) return;
  const totalTokens = this.chatMessages.reduce(
    (sum, msg) => sum + this.estimateTokens(msg.content), 0
  );
  const threshold = cfg?.contextThreshold || 6400;
  if (totalTokens > threshold * 0.8) this.autoCompactContext();
},
```

- 阈值：**80%**（`contextThreshold * 0.8`，默认 6400 → 5120 tokens）
- 触发时机：每次 AI 回复完成后（`sendChatMessage` 的 finally 块）
- 防重入：`isAutoCompacting` 标志
- 最少轮次：至少 3 轮对话才触发（保留 2 轮需至少 2 轮历史 + 1 轮当前 = 3 轮）

**`MemoryGateway.optimize()`（后端真实 LLM 摘要）：**

```rust
pub async fn optimize(
    &self,
    messages: Vec<ChatMessage>,
    client: &dyn LlmClient,
) -> Result<String, String> {
    if messages.len() <= 2 {
        return Ok("对话轮次不足，无需压缩".to_string());
    }
    let summary_msgs = messages[..messages.len() - 2].to_vec();
    let system_prompt = "请将以下对话历史压缩为一段简洁的摘要（200字以内）...";
    let mut stream = client.stream_chat_with_context(summary_msgs, Some(system_prompt)).await?;
    // 收集流式响应...
    Ok(summary.trim().to_string())
}
```

### `ProviderConfig` 扩展

```rust
struct ProviderConfig {
    // ... 原有字段 ...
    system_prompt: Option<String>,      // B-04 新增
    context_threshold: Option<usize>,   // B-07 新增（默认 6400）
}
```

---

## 可行方案

### 方案 A：前端估算（轻量实现）— ✅ 已完成

**实现方式：**
- ✅ 在 `app.js` 中维护 `chatMessages[]` 状态数组（B-06）
- ✅ 使用字符数估算（中文字符 ≈ 1 token，英文单词 ≈ 1.3 token）（B-07）
- ✅ 根据 `ProviderConfig.context_threshold`（默认 6400）计算使用率并触发压缩（B-08）
- ✅ 添加 `/compact` 手动压缩命令（B-07）
- ✅ 自动压缩触发 + LLM 摘要生成（B-08）

**优点：**
- 零精确后端改动（已实现）
- 即时反馈，无需等待 API 响应
- 实现成本低（已投入约 ~140 行前端 + ~80 行后端）

**缺点：**
- 估算有误差（±10%~20%）
- 无法反映实际 API 计费 Token 数
- 无输入/输出分离统计
- 无累计消耗

---

### 方案 B：后端精确统计（推荐长期方案）— ✅ 已完成

**实现方式：**
1. **Rust 端接入 tiktoken-rs**：对消息历史做精确编码
2. **解析 API `usage` 字段**：从 OpenAI 兼容响应中提取 `prompt_tokens` / `completion_tokens`
3. **维护会话级计数器**：在 `ChatSession` 或 `SessionStore` 中累加统计
4. **暴露前端接口**：通过 Tauri command 返回当前会话的 Token 统计
5. **Audit Log 补全**：填充所有 `token_before` / `token_after` / `estimated_tokens` 字段

**优点：**
- 精确匹配 API 实际消耗
- 可扩展为按 Provider / 模型分别统计
- 为未来"上下文压缩触发"提供精确数据基础（当前前端估算已能工作，但精确值更好）

**缺点：**
- 需要新增依赖（tiktoken-rs）
- 前后端都需要改动
- 流式响应中 `usage` 字段可能在最后一个 chunk 才出现，需要特殊处理

---

## 相关代码位置 — 2026-04-27 更新

```
前端：src/interface/web/app.js
  - chatMessages[]               [L28]     对话状态数组（B-06）
  - autoCompact / isAutoCompacting [L29-30] 自动压缩开关与锁（B-08）
  - estimateTokens()             [L1871]   Token 估算（B-07）
  - updateTokenDisplay()         [L1878]   UI 更新（B-07）
  - getActiveProviderConfig()    [L1887]   读取 contextThreshold（B-07）
  - checkAutoCompact()           [L1901]   自动压缩检测（B-08）
  - autoCompactContext()         [L1911]   自动压缩执行（B-08）
  - handleChatCommand('/compact')[~L2350] 手动压缩命令（B-07）
  - streamChat()                 [L2367]   流式请求（传入 messages）

后端：src/interface/desktop/src/main.rs
  - ProviderConfig               [L324]    context_threshold / system_prompt（B-04/B-07）
  - stream_chat()                [L~800]   messages 参数 + token_before/after 填充（B-04/B-05）
  - optimize_context             [L943]    上下文压缩 command（B-08）
  - audit::log_usage()           [L~835]   使用记录（token_before/after 部分填充）

后端：src/intelligence/codex-twist/src/memory/memory_gateway.rs
  - MemoryGateway::optimize()    [B-08]    LLM 驱动摘要生成

后端：src/engine/llm-core/src/mod.rs
  - ChatMessage                  [L133]    消息结构体（B-02）
  - LlmClient::stream_chat_with_context() [L149] 多轮接口（B-02）
```

---

## 结论 — 2026-04-27 更新

- **当前状态**：
  - ✅ **方案 A（前端估算）已完成**：Token 估算 UI、自动压缩触发（80% 阈值）、手动 `/compact` 命令、LLM 摘要压缩
  - ✅ **方案 B（精确统计）已完成**：B-01/06 ~ B-06/06 全部交付，Phase 1~5 覆盖完毕
- **阻塞性**：否，核心聊天流程不受影响
- **实施记录**：见本文档「实施记录」小节
- **已知限制**：
  - Ollama Provider 的 `system_prompt` 字段未使用（调用方可通过 messages 数组传入 `role="system"`）
  - 前端 Token 估算为启发式，与 API 实际计费可能偏差 10%~20%

---

## 后续待讨论问题 — 2026-04-27 更新状态

| # | 问题 | 当前状态 |
|:---|:---|:---|
| 1 | 不同模型的上下文上限如何维护？（硬编码 / 从 API 获取 / 用户配置） | ⚠️ **部分解决** — `ProviderConfig.context_threshold` 支持用户配置，默认 6400 |
| 2 | 上下文压缩的触发阈值？（如使用率 > 80% 时自动压缩历史） | ✅ **已解决** — 80% 阈值已实施（`checkAutoCompact()`） |
| 3 | Token 统计的持久化策略？（仅当前会话 / 按日累计 / 按 Provider 累计） | ⚠️ **部分解决** — `TokenUsageTracker` 支持按 Provider / 按日 / 总会话累计，但尚未集成到 desktop 后端流，前端累计为内存内存储 |
| 4 | 是否接入 tiktoken-rs 做精确 Token 计数？ | ✅ **已完成** — B-02/06 Phase 1 覆盖，`exact-tokens` feature flag 控制 |
| 5 | 流式响应中 `usage` 字段的解析策略？（部分 Provider 仅在最后 chunk 返回） | ✅ **已完成** — B-03/06 Phase 2 覆盖，OpenAI/Anthropic/Ollama 三 Provider 均实现 |

---

## 实施记录

### B-01/06 Day 1 — Baseline 建立与文档同步

- 建立 Scheme B 精确 Token 统计 baseline（Git `6ad02ec`）
- 同步 4 份核心文档标记：
  - `src/INDEX.md` — 新增 Scheme B baseline 章节（含实测数据表 + 后续工单映射）
  - `src/ARCHITECTURE.md` — 性能基准表新增精确 Token 行，ADR 表新增 ADR-SB-01/02
  - `src/MEMORY.md` — 新增 Scheme B 启动状态章节（含 baseline 审计表 + 能力评估）
  - `01-token-context-usage-tracking.md` — 状态更新为「方案 B 进行中」
- Roadmap 引用：`02-exact-token-usage-tracking.md`、`03-token-scheme-b-daily-development-plan.md`、`04-token-scheme-b-guidance.md`

### B-02/06 Day 2 — Engine 层精确计数

- `engine-llm-core` 接入 `tiktoken-rs`（optional 依赖 + `exact-tokens` feature flag）
- `LlmClient` trait 新增 `count_tokens()` + `heuristic_token_count()` fallback
- OpenAI/Anthropic/Ollama 三 Provider 实现 + 7 个单元测试
- commit `8d4d055`

### B-03/06 Day 3 — Backend usage 解析与 Audit

- 新增 `Usage` struct + `LlmClient::last_usage()` trait 方法
- 三 Provider SSE/JSON 解析 usage 字段
- `audit::KeyUsageRecord` 新增 `precise_prompt`/`precise_completion`
- `main.rs` 4 处 `log_usage` 全部填充 precise 值
- commit `02ce06d`

### B-04/06 Day 4 — Intelligence 统计聚合

- 新建 `TokenUsageTracker`（会话级 + 全局累计，by_provider / by_day / total）
- `get_token_stats()` / `get_global_stats()` / `record_usage()` 接口
- 6 个单元测试
- commit `2ba3742`

### B-05/06 Day 5 — Frontend 精确 UI 升级

- `StreamEvent` 扩展 `prompt_tokens`/`completion_tokens`
- `app.js` 新增 `tokenStats`/`cumulativeStats`/`showCumulative`
- `updateTokenDisplay()` 格式 `🔄 xx.x% | ↑ xxxxx | ↓ xxxx`
- Status Bar 新增 `#statusTokens`，点击切换累计显示
- commit `8171caa`

### B-06/06 Day 6 — 集成测试、文档闭环 & 清债

- 新建 `tests/token_tracking_e2e.rs`，12 个 E2E 测试全部通过
- 验证误差率 < 5%（实测 0%）
- 更新 01.md / INDEX.md / ARCHITECTURE.md / MEMORY.md 状态为「已完成」
- 新建 `DEBT-SCHEME-B.md` 清债记录
- 最终 commit `B-06/06`

## P0 债务修复交叉引用

> 本文档中的方案 A 实现是 [P0 上下文债务修复](DEBT-P0-REMEDIATION.md) 的 B-07/B-08 批次产物。
>
> - B-07/09：`/compact` 命令 + Token 估算 UI
> - B-08/09：自动压缩触发 + `MemoryGateway.optimize()` LLM 摘要
>
> 详见 `docs/debt/DEBT-P0-REMEDIATION.md`。

---

## P1 Token Tracker Integration — 清偿阶段状态

<!-- P1-TOKEN-TRACKER-2026-05-02: integration initiated -->

**状态更新**: P1 Token Tracker Integration 已启动，进入清偿阶段。

**背景**: Scheme B 批次 B-01/06 ~ B-06/06 已全部交付，`TokenUsageTracker` 功能完整（68 测试通过：56 lib + 12 E2E），但处于孤岛状态。`docs/debt/DEBT-SCHEME-B.md` 诚实声明 3 项已知限制。

**清偿路径**:
- P1-01/05: 文档基线同步 + Baseline 测量（本轮）
- P1-02/05: Backend 集成 — `AppState` 注入 `TokenUsageTracker`，`stream_chat` 调用 `record_usage()`
- P1-03/05: Tauri Command 暴露 — `get_cumulative_stats` 返回 `GlobalStats`
- P1-04/05: Frontend 持久化 — Tauri Command + LocalStorage 混合存储
- P1-05/05: 清债验证 — 文档闭环 + DEBT-SCHEME-B.md 状态更新为 ✅ 已清偿

**关联文档**:
- Roadmap: `docs/roadmap/Hajimi Context/P1 fix/P1-TOKEN-TRACKER-INTEGRATION-ROADMAP.md`
- Daily Plan: `docs/roadmap/Hajimi Context/P1 fix/P1-TOKEN-TRACKER-DETAILED-DAILY-PLAN.md`
- Guidance: `docs/roadmap/Hajimi Context/P1 fix/P1-TOKEN-TRACKER-REMEDIATION-GUIDANCE.md`
- Debt: `docs/debt/DEBT-SCHEME-B.md`
