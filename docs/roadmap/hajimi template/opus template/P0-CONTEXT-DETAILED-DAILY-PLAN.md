# P0 Context & Memory Remediation Detailed Daily Development Plan

> **文档版本**: 1.0
> **所属 Roadmap**: [P0-CONTEXT-REMEDIATION-ROADMAP.md](./P0-CONTEXT-REMEDIATION-ROADMAP.md)
> **目标**: 彻底清偿 P0 级“单轮对话失忆”架构债务。**最小成本、无大重构、严格四层分层、数据诚实**原则下，升级 LLM 接口、激活 codex-twist MemoryGateway，实现多轮对话、上下文压缩与长期记忆能力。为后续 Token 统计、Slash Palette 和 Agent Swarm 奠定基础。
> **预计工期**: 9 个工作日（2026-05-01 至 2026-05-13）
> **作者**: Claude Opus 4.6 (基于 03-context-compaction.md + llm-core/main.rs/codex-twist 实测分析生成)
> **最后更新**: 2026-04-30
> **优先级**: ★★★★★（最高，P0-Blocker，直接影响自主 Agent 定位与 MEMORY.md 长期记忆愿景）

---

## P0 Context Remediation 重点优化方向

本阶段核心目标是**以最小变更修复单轮对话根因**，确保：

- `LlmClient::stream_chat(String)` 升级为支持 `Vec<ChatMessage>`（保留兼容）。
- codex-twist（当前完全孤岛的三层内存系统：Focus/Working/Archive + MemoryGateway）正式接入 Intelligence 层。
- 前端从“仅 DOM 渲染”升级为真实 `chatMessages` 状态管理。
- `/compact` 命令可用 + 基础上下文预算感知。
- 严格遵守四层架构（Engine 不依赖 Interface，Intelligence 提供记忆服务）。
- 所有变更更新 `src/INDEX.md`、`src/ARCHITECTURE.md`、`MEMORY.md`，metric 必须 100% 实测。

### 核心修复点 (来自 03-context-compaction.md + 代码实测)
1. **P0 接口根因**: `src/engine/llm-core/src/mod.rs:133` 及三个 provider 实现均为单 `prompt: String`。
2. **P0 基础设施浪费**: `src/intelligence/codex-twist/src/memory/memory_gateway.rs` 及相关文件完全未被 `main.rs` 或前端调用。
3. **P0 前端状态缺失**: `src/interface/web/app.js` 中 `aiChatMessages` 仅用于渲染，不传递历史。
4. **联动债务**: Token 统计（P2）、Slash Palette（P1）、会话持久化均依赖此基础。
5. **架构合规**: 必须保持下层纯洁性，变更规模最小化。

**预期效果**：AI 具备连续记忆能力（连续 10+ 轮对话可引用历史），codex-twist 从“漂亮孤岛”变为 Intelligence 层核心，符合 HAJIMI V3 “自主AI Agent系统”定位与债务清偿文化。

---

## 详细 9 天开发计划

### **Day 1: 准备、Baseline 测量 & Step 0 文档同步（2026-05-01）**
- 运行基准命令：`cargo check --workspace`、`grep -r "stream_chat|MemoryGateway|LlmClient" src/`、`wc -l src/engine/llm-core/src/mod.rs src/interface/desktop/src/main.rs src/interface/web/app.js`。
- 重新阅读 `src/INDEX.md`、`src/ARCHITECTURE.md`、`docs/roadmap/Hajimi Context/03-context-compaction.md` 及 `MEMORY.md` 中所有相关声明。
- 执行 Roadmap Step 0：编辑 4 个 MD 文件，同步 P0 状态为“remediation initiated”，添加 `<!-- P0-CONTEXT-REMEDIATION-2026-04-30: based on real code audit -->`。
- 更新本 daily plan 和 `P0-CONTEXT-REMEDIATION-ROADMAP.md` 相互引用。
- **质量保证**：
  - 运行 `grep "P0-REMEDIATION|P0-CONTEXT" -r docs/roadmap/Hajimi Context/` 验证一致性。
  - `cargo check --workspace` 必须 0 errors。
- **当天验收**：所有文档更新完成，grep 确认 P0 remediation 标记存在，baseline 数据记录完毕。

### **Day 2-3: Phase 0 - LLM Core 接口升级（2026-05-02 ~ 05-03）**
- 重点阅读 `src/engine/llm-core/src/mod.rs:130-155`（LlmClient trait）、`anthropic.rs:30-50`、`openai.rs:30-50`、`ollama.rs:30-50`。
- 扩展 `LlmClient` trait，新增 `stream_chat_with_context(messages: Vec<ChatMessage>, system_prompt: Option<String>)`（保留原有方法做兼容）。
- 在三个 provider 中实现新方法（最小复用现有请求构建逻辑，仅把 Vec 转为 API messages 数组）。
- 更新 `mod.rs` re-exports 和错误处理。
- **质量保证**：
  - 每天 `cargo check --package engine-llm-core`。
  - 编写最小单元测试验证新接口（非破坏性）。
  - 确保 Engine 层不引入任何 Interface 或 codex-twist 依赖（严格分层）。
- **当天验收**（Day 3 结束）：新 trait 方法可用，所有 provider 实现完成，变更总行数控制在最小必要范围，cargo check 通过。

### **Day 4-5: Phase 1 - Backend 集成 & MemoryGateway 激活（2026-05-04 ~ 05-05）**
- 重点阅读 `src/interface/desktop/src/main.rs:780-880`（`create_llm_client` + `stream_chat` command）、`AppState` 定义及 `codex-twist/src/memory/memory_gateway.rs:20-65`。
- 修改 `stream_chat` command 接收 `messages` 参数，注入 `MemoryGateway` 到 `AppState`。
- 在 `main()` 中初始化 `MemoryGateway::with_budget(...)`，在对话流程中调用 `gateway.optimize()` / `working.put()` 处理上下文。
- 更新 audit log、SAFETY 注释和所有相关 Tauri handler。
- **质量保证**：
  - `cargo check --package interface-desktop` + `grep "MemoryGateway|stream_chat_with_context" src/interface/desktop/src/main.rs`。
  - 验证 Intelligence 层（codex-twist）仅被 Interface 消费，下层保持纯洁。
  - 所有内存操作添加 `// SAFETY: MemoryGateway is thread-safe via Arc<RwLock>` 注释。
- **当天验收**（Day 5 结束）：后端可接收并使用多轮消息历史，MemoryGateway 被正式实例化并调用，变更最小化。

### **Day 6-7: Phase 2 - Frontend 状态管理 & Command 支持（2026-05-06 ~ 05-07）**
- 重点阅读 `src/interface/web/app.js:1885+`（setupChat）、`1927+`（sendChatMessage）、`2014+`（handleChatCommand）、`2252+`（streamChat）。
- 引入真实 `chatMessages: []` 状态数组（取代仅 DOM 渲染）。
- 修改发送逻辑传递完整历史给后端，实现 `/compact` 命令调用 `MemoryGateway.optimize()`。
- 增加基础 Token 使用率估算显示（为 P2 Token 统计铺路），并初步支持 Slash 输入提示。
- **质量保证**：
  - 浏览器 devtools 验证连续对话能正确引用历史。
  - `grep "chatMessages|compact|streamChat" src/interface/web/app.js`。
  - 确保前端不直接依赖 Rust 内存细节（通过 Tauri command 抽象）。
- **当天验收**（Day 7 结束）：前端维持真实对话状态，`/compact` 命令可用，多轮对话在 UI 中连贯，变更控制在最小范围。

### **Day 8: Phase 3 - 上下文压缩逻辑 & 集成测试（2026-05-08）**
- 完善 WorkingMemory/ArchiveMemory 的 compact 流程（复用 codex-twist 已有 `optimize()` 和压缩逻辑）。
- 实现自动压缩触发（预算 >80% 时）。
- 跨层端到端测试：连续 10 轮对话 + /compact + Token 估算。
- 更新 `src/ARCHITECTURE.md` 中的记忆流图和模块说明。
- **质量保证**：
  - 完整手动测试 + `cargo check --workspace`。
  - 验证压缩后上下文能被后续对话正确使用。
- **当天验收**：上下文压缩功能可用，codex-twist 三层内存被有效利用。

### **Day 9: 清债验证、文档闭环 & Closure（2026-05-09）**
- 创建 `docs/debt/DEBT-P0-REMEDIATION.md`，记录所有 P0 finding、精确代码位置、最小 fix、实测前后对比。
- 更新 `src/INDEX.md`、`src/ARCHITECTURE.md`、`MEMORY.md` 标记 “P0 Context Debt Cleared”。
- 运行完整验证命令集并记录。
- 最终 commit（使用 HEREDOC + Co-Authored-By）。
- **质量保证**：
  - `cargo check --workspace` + 浏览器多轮对话测试。
  - `grep "P0-CONTEXT-REMEDIATION" -r docs/` 确认所有 patch 存在。
  - 所有量化描述必须来自当天实测。
- **当天验收**：DEBT 文件完整，文档 100% honest，P0 正式标记为 Cleared，git 干净。

---

## 整体质量保证策略

- **测试要求**：每天结束前必须 `cargo check --workspace` 0 errors，所有新增逻辑有对应手动/浏览器验证。
- **分层合规**：每天使用 Grep 验证下层绝不依赖上层（Engine → Intelligence → Interface 严格单向）。
- **数据诚实性**：所有行数、文件引用、测试结果必须来自当天实测命令，禁止估算。变更后立即 re-grep 验证。
- **工具使用**：优先使用 Read/Grep/Glob/Edit/Task（跟踪每日 Step），仅在必要时使用 Bash（只读验证命令）。Shell 必须走白名单。
- **最小变更文化**：严格遵守 anti-over-engineering — 仅编辑现有文件，无新抽象、无 app.js 大重构、无不必要 helper。
- **P0 安全**：所有内存和 LLM 调用必须保留 SAFETY 注释，绝不引入 unsafe 路径。
- **每日 Review**：每天结束前更新本计划文件，记录实际完成情况与实测输出。

---

## 最终回归验收清单（P0 完成后执行）

- [ ] `cargo check --workspace` **0 errors**
- [ ] 浏览器验证：连续 10 轮对话 AI 可引用历史，`/compact` 命令有效，Token 估算显示正常
- [ ] `MemoryGateway` 被 `main.rs` 实例化并在对话流中使用
- [ ] `LlmClient` 新接口在所有 provider 中实现且向后兼容
- [ ] `src/INDEX.md`、`src/ARCHITECTURE.md`、`MEMORY.md` 全部更新，P0 状态为 “Cleared with minimal interface extension”
- [ ] `docs/debt/DEBT-P0-REMEDIATION.md` 记录所有 finding + fix SHA + 实测证据
- [ ] grep 确认所有 `P0-CONTEXT-REMEDIATION` 和 `stream_chat_with_context` patch 存在
- [ ] 四层架构纯洁性验证通过（无反向依赖）
- [ ] git status 干净，最终 commit 符合 CONTRIBUTING.md

---

**P0 完成标准**：单轮对话失忆问题彻底解决，codex-twist 内存系统正式激活为 Intelligence 层核心，多轮对话、上下文压缩能力可用，所有文档与实测完全一致，债务清偿文化得到体现，为 HAJIMI V3 后续 Agent 特性奠定坚实诚信基础。

---

*本计划严格遵循 `src/CONTRIBUTING.md`、`src/ARCHITECTURE.md`、`src/MEMORY.md` 中的四层分层规则、数据诚实性规范、P0 安全要求和债务清偿文化。所有量化指标将在实际执行中每日实测更新。禁止任何超过最小必要范围的变更。*

**文档路径**：`docs/roadmap/Hajimi Context/p0 fix/P0-CONTEXT-DETAILED-DAILY-PLAN.md`
**最后更新**：2026-04-30
