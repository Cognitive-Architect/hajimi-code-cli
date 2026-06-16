# HAJIMI P0 Context & Memory Remediation Roadmap

**文件路径**: `docs/roadmap/Hajimi Context/p0 fix/P0-CONTEXT-REMEDIATION-ROADMAP.md`
**生成日期**: 2026-04-30 (基于03-context-compaction.md + 代码实测分析)
**目标**: 解决 P0 级“单轮对话失忆”核心架构债务。**最小成本、无大重构、严格分层、数据诚实**原则下，逐步升级 LLM 接口、接入现有 codex-twist 内存系统，实现多轮对话 + 上下文压缩能力。必须更新 `src/INDEX.md`、`src/ARCHITECTURE.md`。

---

## 优先级 (P0-Blocker 第一)

1. **P0 多轮对话基础 (Critical)**: `LlmClient::stream_chat(String)` → 支持 `Vec<ChatMessage>`（根因）
2. **P0 codex-twist 集成 (High)**: 当前完全未使用的高质量 MemoryGateway（Focus/Working/Archive 三层 + compact）
3. **P1 Slash Palette & Token UI (Medium)**: 依赖多轮上下文才能真正生效
4. **P2 持久化 & 自动压缩 (Follow-up)**: 会话保存 + 80% 阈值触发

**总原则** (严格遵守 MEMORY.md + CONTRIBUTING.md):
- 每步变更最小化（优先扩展而非重写）
- 下层（Engine/Intelligence）绝不依赖上层（Interface）
- 所有变更必须更新 `src/INDEX.md` 和 `src/ARCHITECTURE.md`
- 实测验证（cargo check + 前端手动测试），禁止虚报行数/TODO数
- 保留现有 `stream_chat(String)` 兼容性（渐进迁移）
- 仅编辑必要文件，绝不引入新 crate 或破坏四层纯洁性
- 每次 commit 带 `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>`

---

## 路线规划图 (Mermaid流程图)

```mermaid
flowchart TD
    A[P0 Debt Discovery<br/>03-context-compaction.md + llm-core/main.rs 实测<br/>codex-twist 完全孤岛] --> B{P0-Blocker?<br/>单轮接口是根因?}
    B -->|Yes| C[Phase 0: 接口升级<br/>- LlmClient trait 扩展为 stream_chat_with_context(Vec<ChatMessage>)\n- 所有 provider 实现同步（保持向后兼容）\n- 更新 mod.rs + anthropic/openai/ollama.rs（最小变更）]
    C --> D[Phase 1: Backend 集成 (main.rs)\n- stream_chat command 接收 messages\n- 注入 MemoryGateway 到 AppState\n- 调用 gateway.optimize() / working.put() 进行上下文管理\n- 保留所有 SAFETY + audit log]
    D --> E[Phase 2: Frontend 状态管理 (app.js)\n- 维护真实 chatMessages 数组（非仅DOM渲染）\n- sendChatMessage() 传递完整历史\n- 实现 /compact 命令（调用后端 gateway.optimize）\n- Token 估算 UI（P2 前置）]
    E --> F[Phase 3: 上下文压缩 & 长期记忆\n- WorkingMemory 滑动窗口\n- ArchiveMemory zstd 压缩\n- 自动触发（>80% budget）+ 手动 /compact\n- 更新 ARCHITECTURE.md 四层记忆流图]
    F --> G[Phase 4: 验证 & 清债 ✅\n- 更新 src/INDEX.md + ARCHITECTURE.md + MEMORY.md\n- 创建 DEBT-P0-REMEDIATION.md\n- 实测多轮对话（连续10轮不失忆）\n- cargo check + 前端 Token/Slash 测试通过]
    G --> H[最终交付 ✅ P0 Cleared<br/>✅ 多轮对话完整支持<br/>✅ codex-twist 正式接入 Intelligence 层<br/>✅ /compact + Token UI 可用<br/>✅ 符合自主Agent长期记忆愿景<br/>✅ 所有文档 honest sync]
    style A fill:#ff4444
    style H fill:#44ff44
```

---

## 详细执行步骤 (最小成本原则)

### Step 0: 准备与文档同步 (30-60 mins, 3文件)
- 编辑 `src/INDEX.md`、`src/ARCHITECTURE.md`、`docs/roadmap/Hajimi Context/03-context-compaction.md`
- 在各文件添加 `<!-- P0-REMEDIATION-2026-04-30: codex-twist integration initiated -->`
- 更新 MEMORY.md 记录 “P0 Context Debt → 进入清偿阶段”
- **验证**: `grep -o "P0-REMEDIATION" src/*.md docs/**/*.md`

### Step 1: LLM Core 接口升级 (核心瓶颈, 2-3 hours, 4文件)
- `src/engine/llm-core/src/mod.rs:133`: 扩展 `LlmClient` trait，新增:
  ```rust
  async fn stream_chat_with_context(&self, messages: Vec<ChatMessage>, system_prompt: Option<String>) -> Result<ChannelStream, EngineError>;
  ```
- 更新 `anthropic.rs`、`openai.rs`、`ollama.rs` 实现（复用现有逻辑，仅把 Vec 转为 API 格式）
- 保留原有 `stream_chat(String)` 方法做平滑过渡
- **绝不**: 大改请求结构体或引入新依赖
- **验证**: `cargo check --package engine-llm-core`

### Step 2: Backend 接入 codex-twist (main.rs, 3-4 hours)
- `src/interface/desktop/src/main.rs:824`: 修改 `stream_chat` command 签名，增加 `messages` 参数
- 在 `AppState` 中添加 `memory_gateway: Arc<MemoryGateway>`
- 在 `create_llm_client` 后调用 `state.memory_gateway.optimize(...)` 获取压缩上下文
- 将 MemoryGateway 实例化加入 `main()`（复用现有 with_budget）
- 更新所有相关 audit log + SAFETY 注释
- **最小变更**: 优先扩展现有函数而非重构
- **验证**: `cargo check --package interface-desktop`

### Step 3: Frontend 状态化 + Command 支持 (app.js, 2 hours)
- `src/interface/web/app.js`:
  - 维护 `chatMessages: []` 真实状态（当前仅 DOM）
  - 修改 `streamChat()` / `sendChatMessage()` 传递完整历史
  - 实现 `handleChatCommand('/compact')` → 调用后端 optimize
  - 增加基础 Token 使用率估算显示（为 P2 铺路）
- 复用现有 `showCommandPalette()` 逻辑扩展 Slash 输入提示（P1 联动）
- **验证**: 浏览器测试连续对话是否能引用上一轮内容

### Step 4: 清债验证 & 文档闭环 (1 hour)
- 创建 `docs/debt/DEBT-P0-REMEDIATION.md` 记录所有变更 SHA + 实测结果
- 更新 `src/ARCHITECTURE.md` 四层图，增加 “Intelligence: MemoryGateway 正式激活”
- 运行验证命令：
  ```bash
  cargo check --workspace
  grep -E "P0|MemoryGateway|stream_chat_with_context" -r src/
  ```
- 最终 commit 消息: "fix(p0): remediate context memory debt - enable multi-turn + codex-twist integration (minimal interface extension)"

---

## 预期成果 (必须实测验证)

- **P0 Cleared**: AI 具备连续对话记忆能力（连续 8+ 轮不失忆）
- **codex-twist 激活**: MemoryGateway 从孤岛变为 Intelligence 层核心
- **文档一致性**: INDEX.md、ARCHITECTURE.md、MEMORY.md 全部更新，metric honest
- **体验提升**: `/compact` 命令可用，基础 Token 显示上线，Slash Palette 可后续快速跟进
- **架构遵守**: 零破坏四层分层规则，Engine 只暴露接口，Intelligence 提供记忆服务
- **变更规模**: 预计总变更 < 350 行（分散多文件），全可逆

---

## 风险 & 回滚

- **风险**: 接口变更可能影响现有 MCP/Agent 调用 → 保留旧方法 + 逐步迁移
- **回滚**: `git revert` 任意 commit 均安全（所有变更均为扩展）
- **P0 安全**: 所有内存路径必须加 SAFETY 注释，绝不使用 bash -c，严格白名单
- 若 MemoryGateway 初始化失败，fallback 到当前单轮模式

**此 roadmap 本身即 P0 清债文化产物** — 基于 03.md + llm-core/main.rs/codex-twist 真实代码分析生成。所有步骤均可使用 Task 工具跟踪。

**下一步**: 用户批准后立即执行 Step 0-4，使用 Edit 工具进行精准最小化变更。

---
*Generated per user request on 2026-04-30. All analysis based on real codebase inspection. Aligns with HAJIMI V3 debt clearance culture and four-layer architecture purity.*
