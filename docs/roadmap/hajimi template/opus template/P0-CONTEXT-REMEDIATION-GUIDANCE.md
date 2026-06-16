# P0 Context & Memory Remediation Development Guidance

> **文档版本**: 1.0
> **所属 Roadmap**: [P0-CONTEXT-REMEDIATION-ROADMAP.md](./P0-CONTEXT-REMEDIATION-ROADMAP.md) & [P0-CONTEXT-DETAILED-DAILY-PLAN.md](./P0-CONTEXT-DETAILED-DAILY-PLAN.md)
> **目的**: 为 P0 清债工作提供**架构设计思路、决策原则、每日深度指导和技术注意事项**，与 Roadmap 和 Detailed Daily Plan 形成**互补闭环**，确保开发过程严格遵守四层分层纯洁性、最小变更原则和数据诚实性，大幅降低决策成本和返工风险。
> **作者**: Claude Opus 4.6 (基于 03-context-compaction.md + llm-core/main.rs/codex-twist 实测分析生成)
> **最后更新**: 2026-04-30
> **适用对象**: 开发者/维护者执行 P0 Context Remediation 时参考
> **状态**: 与 Roadmap 和 Daily Plan 同步使用

---

## 1. 当前现状 (P0 Debt 发现时)

根据 `src/ARCHITECTURE.md`、`src/INDEX.md`、`src/MEMORY.md` 及 `docs/roadmap/Hajimi Context/03-context-compaction.md`（2026-04-30 更新）：

- **已完成基础**：v3.8.0-batch-1 阶段，Phase 7 Debt Clearance 接近完成，codex-twist 内存系统已完整实现但处于**完全孤岛状态**。
- **核心资产**：
  - `MemoryGateway` + Focus/Working/Archive 三层内存（zstd 压缩、滑动窗口、mmap、TokenBudget）。
  - 7 步 Agent 循环和 Swarm 基础设施已就位。
  - LLM Core 已支持 Anthropic/OpenAI/Ollama 三提供商。
- **主要短板**（与 Kimi Code 对比）：
  - **根本性失忆**：`LlmClient::stream_chat` 只接收 `String` prompt，所有调用均为单轮。
  - **基础设施浪费**：codex-twist 在 `src/intelligence/codex-twist/src/memory/` 完整实现，但 `main.rs` 和前端完全未引用。
  - **状态管理缺失**：前端 `app.js` 仅 DOM 渲染历史，不维护 `chatMessages` 数组。
  - **体验断层**：无 `/compact`、无 Token 预算感知、无多轮上下文压缩。

**当前得分估算**（数据诚实）：核心对话能力 45/100，长期记忆能力 25/100。P0 目标是把对话能力提升至 90+，激活 codex-twist 作为 Intelligence 层核心记忆大脑。

---

## 2. 本阶段目标与成功定义

**最终愿景**：HAJIMI 成为**真正具备连续记忆与上下文压缩能力的自主 AI Agent** —— 用户可以进行多轮深度对话，AI 能记住历史、自动/手动压缩上下文、利用三层内存实现长期记忆，形成与 Kimi Code / Claude Cowork 相当的对话体验，并为 Swarm 和高级 Agent 特性提供坚实基础。

**量化目标**（必须实测）：
- 多轮对话记忆成功率 ≥ 98%（连续 10 轮不失忆，实测验证）
- 上下文压缩后 Token 使用率可控在 60% 以下
- `MemoryGateway` 在实际对话流中被调用 ≥ 5 次/会话
- 新增 ≥ 25 个相关测试/验证用例
- 更新 `src/ARCHITECTURE.md` 记忆流图并在 `INDEX.md` 增加映射

**与 Roadmap/Daily Plan 的配合方式**：Roadmap 告诉你**整体方向和优先级**，Daily Plan 告诉你**具体做什么**，本指南告诉你**为什么这么做 + 如何思考 + 关键决策点 + 避坑指南**。每日开发前应先阅读本指南对应 Day 章节，再执行 Daily Plan 中的具体步骤。

---

## 3. 总体设计思路与原则

### 核心设计哲学
- **最小侵入 + 分层纯洁**：优先通过扩展现有 `LlmClient` trait（新增方法而非修改签名）实现兼容；`codex-twist` 只在 Intelligence 层提供服务，Interface 层通过 Tauri command 消费，Engine 层绝不直接依赖 MemoryGateway。
- **复用现有资产**：充分利用 codex-twist 已有的 `optimize()`、`compact()`、`TokenBudget` 和三层内存实现，绝不重复造轮子。
- **数据诚实性**：所有成功率、Token 数、压缩效果必须通过真实对话测试 + `cargo check` 实测记录，写入自测报告，严禁估算。
- **渐进式增强**：先打通多轮消息传递（Phase 0-1），再实现状态管理（Phase 2），最后激活压缩与自动优化（Phase 3）。
- **P0 安全**：所有内存持久化、LLM 调用必须添加 SAFETY 注释，严格使用现有白名单机制，绝不引入新 unsafe 路径。

### 关键抽象与决策
- **`ChatMessage` 结构**：在 `engine/llm-core` 中定义统一消息类型，支持 role/content/timestamp。
- **`MemoryGateway` 接入点**：在 `AppState` 中持有实例，在 `stream_chat` command 中调用 `optimize(target)` 获取压缩上下文。
- **前端状态管理**：使用 `chatMessages` 数组作为单一事实来源，同时驱动 UI 渲染和后端请求。
- **压缩策略**：WorkingMemory 负责最近对话，ArchiveMemory 负责摘要，FocusMemory 负责当前高优先级内容。

**架构决策（将写入 ARCHITECTURE.md 新 ADR）**：
- ADR-P0-01: Multi-turn Context Pipeline（以 `stream_chat_with_context` 作为唯一入口）
- ADR-P0-02: Codex-Twist Activation（MemoryGateway 作为 Intelligence 层长期记忆核心）

---

## 4. 每日开发指导（精确到每天的设计思路与注意事项）

### **Day 1: 准备、Baseline & 文档同步**
- **设计思路**：先建立真实 baseline，确保所有后续变更基于诚实数据。强调“文档先行”，让 P0 债务在所有核心 MD 中可见。
- **关键决策**：所有 metric 必须来自当天 `grep`/`cargo check` 命令输出。
- **注意事项**：
  - 严格遵守 MEMORY.md “数据诚实性”要求，绝不虚报。
  - 更新 INDEX.md 时同时增加 P0 remediation 条目映射。
- **与计划配合**：完成 Daily Plan 中文档编辑后，立即验证 grep 一致性。

### **Day 2-3: Phase 0 - LLM Core 接口升级**
- **设计思路**：这是整个 P0 的根因突破口。采用“扩展而非修改”策略，新增 `stream_chat_with_context` 方法，保持原有单轮接口兼容，降低迁移风险。
- **关键决策**：`ChatMessage` 定义放在 `llm-core` 中；provider 实现优先复用现有请求构建逻辑，只转换 messages 数组。
- **注意事项**：
  - 严格分层：Engine 层只定义接口，不引入 codex-twist。
  - 每个 provider 变更后立即运行 `cargo check --package engine-llm-core`。
  - 为新方法添加详细 Rust doc 注释说明多轮使用方式。

### **Day 4-5: Phase 1 - Backend 集成 & MemoryGateway 激活**
- **设计思路**：让 codex-twist 从“漂亮的未使用基础设施”变为实际参与对话生命周期的核心。`MemoryGateway` 应在每轮对话前后被调用，实现“Retrieve → Optimize → Store”闭环。
- **关键决策**：在 `AppState` 中以 `Arc<MemoryGateway>` 持有；在 `stream_chat` 中先通过 gateway 获取压缩上下文，再调用 LLM。
- **注意事项**：
  - 所有内存操作必须添加 `// SAFETY: ...` 注释。
  - 保留完整 audit log 路径，记录 token_before/after。
  - 验证下层 Intelligence 不被 Interface 反向依赖。

### **Day 6-7: Phase 2 - Frontend 状态管理 & Command 支持**
- **设计思路**：前端从“被动渲染”升级为“主动状态管理”。`chatMessages` 数组作为单一事实来源，同时驱动 UI 和后端请求，实现真正闭环。
- **关键决策**：`/compact` 命令应调用后端 gateway.optimize() 并更新本地状态；Slash Palette 可复用现有全局面板逻辑。
- **注意事项**：
  - 避免在 app.js 中引入复杂状态管理库，保持纯 vanilla JS。
  - 每处状态更新后立即测试连续对话记忆效果。
  - 为 Token 估算添加简单字符→token 启发式（作为 P2 前置）。

### **Day 8: Phase 3 - 上下文压缩逻辑 & 集成测试**
- **设计思路**：真正激活 codex-twist 的压缩能力。WorkingMemory 处理短期滑动窗口，ArchiveMemory 处理长期摘要，实现自动 + 手动双模式。
- **关键决策**：压缩阈值默认 80%，可通过 config 调整；压缩后保留最近 2 轮完整对话 + 摘要。
- **注意事项**：
  - 重点测试长上下文场景（>50 轮对话）。
  - 确保压缩结果能被后续 LLM 调用正确理解。

### **Day 9: 清债验证、文档闭环 & 最终交付**
- **设计思路**：以实测数据驱动收尾，确保所有文档 honest。DEBT 文件必须包含精确变更位置和验证证据。
- **关键决策**：在 ARCHITECTURE.md 中新增记忆流图，明确 codex-twist 在四层架构中的位置。
- **注意事项**：
  - 最终 commit 必须符合 CONTRIBUTING.md（最小变更 + Co-Authored-By）。
  - 自测报告需包含真实多轮对话截图和 Token 压缩前后数据。

---

## 5. 最佳实践与风险防控

- **测试策略**：每日必须进行端到端多轮对话测试（推荐至少 8 轮）；重点覆盖压缩后记忆保留、边界 Token 超限、会话切换场景。
- **文档同步**：每天结束前更新对应 Roadmap/Daily Plan 中的完成标记，并在 Day 9 集中更新 INDEX/ARCHITECTURE/MEMORY.md。
- **常见陷阱**：
  - 不要让 Engine 层直接依赖 codex-twist（违反分层）。
  - 禁止大重构 app.js 或引入新状态管理抽象。
  - 所有性能/成功率数字必须实测，绝不估算。
  - 避免一次性修改过多文件（遵守最小变更原则）。
- **效率提升技巧**：每日开始前并排打开本指南 + Daily Plan + 对应源代码文件（尤其是 `mod.rs:130`、`main.rs:824`、`app.js:1927`、`memory_gateway.rs`）。使用 Task 工具跟踪每日 Step 进度。

---

## 6. 最终验证清单（与 Daily Plan 最终回归验收联动）

- 所有每日验收项均实测通过
- `MemoryGateway` 在实际对话中被有效使用，三层内存均参与
- 多轮对话记忆与压缩功能达到量化目标
- `src/INDEX.md`、`src/ARCHITECTURE.md`、`MEMORY.md` 及 DEBT 文件全部更新
- 自测报告包含真实数据、截图和前后对比
- 主 Roadmap 文件已标记 P0 完成
- 符合 HAJIMI 四层架构纯洁性和债务清偿文化

---

**本指南与 Roadmap/Daily Plan 互为表里**：Roadmap 提供方向，Daily Plan 提供清单，本指南提供**设计思想、决策 rationale 和避坑指南**。三者结合可确保 P0 清债工作高质量、低风险、高一致性完成。

**文档路径**：`docs/roadmap/Hajimi Context/p0 fix/P0-CONTEXT-REMEDIATION-GUIDANCE.md`
**使用建议**：开发过程中将此文件与 Roadmap 和 Daily Plan 并排打开，作为每日决策参考和 checklist 使用。

*严格遵循 `src/CONTRIBUTING.md`、`src/ARCHITECTURE.md`、`src/MEMORY.md` 中的四层分层规则、数据诚实性规范、P0 安全要求和债务清偿文化。所有决策必须可追溯、可测试、可文档化。*
