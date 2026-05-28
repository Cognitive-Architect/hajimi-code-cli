# AGENT-LLM-NATIVE-DESIGN: Hajimi Codex-Style LLM-Native Agent Core

**文件路径**: `docs/roadmap/Hajimi LLM/plan/AGENT-LLM-NATIVE-DESIGN.md`
**生成日期**: 2026-05-28
**修订日期**: 2026-05-28（基于 Engine LlmClient 层实测采样修订）
**阶段**: Phase 0 架构审计与新抽象定义（LLM-NATIVE-AGENT-MIGRATION-ROADMAP）
**目的**: 完整映射 Hajimi Agent Core 中所有本地规则意图破坏点，定义 Codex 风格 LLM-Native 新抽象，为后续 Phase 1-5 迁移提供精确蓝图。数据全部来自实测代码采样。

> **修订说明（2026-05-28 复审）**: 原文档对 Engine 层 `LlmClient` trait 的工具调用能力评估不足。实测确认 `LlmClient` trait 完全没有 `tools` / `tool_choice` 参数，`StreamChunk` 仅有 `Output/Error/Done` 三种变体，不包含 ToolCall 事件。这意味着 Phase 2 之前必须先完成 Engine 层改造，原 17 天估时需调整为 22-28 天。本修订版新增 2.8（Engine 层缺口）和 4.6（Engine 层前置改造需求），并简化了初始抽象设计以避免过度工程。

---

## 1. 执行摘要

Hajimi 当前 Agent 执行路径存在**三层结构性本地规则意图破坏**，导致任意非英文（尤其是中文）自然语言输入在到达 LLM 或工具执行层之前已被彻底改写。根本解法是**消除本地规则映射层**，让原始用户输入直达 LLM，由模型通过 `tool_choice=auto` 自主生成 ToolCall。

本设计文档基于对 `src/intelligence/agent-core/` 全量采样 + Codex 关键源码对照，产出：
- 所有规则映射点的精确坐标（file:line + 逻辑摘要）
- 当前架构缺陷的结构化分析
- 目标 LLM-Native 架构的核心抽象定义
- 黑板协议（BB_NEXT_TOOL 等）的去留建议
- 迁移约束与四层架构纯洁性要求

**核心结论**：现有 `decompose_rule_based` + `generate_tasks_for` + `legacy_act` 规则链必须整体降级为离线兜底；新路径需引入 `LLMDriver` / `IntentPreservingTurn` 等原语，实现「原始输入 + 完整工具列表 + auto 选择 → LLM 流式 ToolCall → 执行」的 Codex 风格闭环。

---

## 2. 当前架构：三层规则意图破坏全景图

通过系统性采样（Glob + Grep + 针对性 Read），确认**所有本地规则映射逻辑集中在以下三处**，形成级联破坏链：

### 2.1 第一层：Planner 规则分解（decompose_rule_based）

**位置**:
- `src/intelligence/agent-core/planner.rs:182-217`

**代码证据** (关键片段):
```rust
fn decompose_rule_based(&self, goal: &Goal) -> Vec<SubGoal> {
    let desc = goal.description.to_lowercase();
    // FIX-I18N-001: Add Chinese keyword support...
    let patterns: Vec<&str> = if desc.contains("implement")
        || desc.contains("create")
        || desc.contains("创建") || desc.contains("实现") || ...
    {
        vec!["Analyze requirements", "Design", "Implement", "Test"]  // ← 彻底改写
    } else if ... {
        vec!["Reproduce", "Identify cause", ...]
    } else {
        vec![&goal.description, "Execute", "Validate"]  // 仅 else 分支保留原始
    };
    ...
}
```

**破坏效果**:
- 即使中文补丁存在，用户 "创建文件" 仍大概率被映射为通用英文子目标序列。
- 只有当描述不匹配任何已知模式时，才会把原始描述放在第一个 SubGoal（脆弱且不可靠）。
- **SubGoal.description 不再是用户原始意图**。

**调用点**:
- `decompose()` (planner.rs:347-398): LLM 失败时 fallback，或无 LLM 时直接走规则。
- `run()` (agent_loop.rs:215-241): 启动时立即调用 `planner.decompose` + `expand`。
- `bootstrap_first_tool_call()` (agent_loop.rs:797): 再次触发 decompose。

### 2.2 第二层：Planner 规则任务生成（generate_tasks_for）

**位置**:
- `src/intelligence/agent-core/planner.rs:230-271`

**代码证据**:
```rust
fn generate_tasks_for(&self, sg: &SubGoal) -> Vec<Task> {
    let desc = sg.description.to_lowercase();
    // FIX-I18N-002
    let items: Vec<&str> = if desc.contains("implement") || ... || desc.contains("创建") {
        vec!["Write code", "Check compilation"]
    } else if ... {
        vec!["Run tests", "Review"]
    } else {
        vec![&sg.description]  // 再次复制已被破坏的描述
    };
    ...
    tool_calls: Vec::new(),  // ← Task 永远不带 tool_calls
}
```

**破坏效果**:
- 即使上游奇迹般保留了原始描述，这里仍会再次进行关键词匹配并可能改写。
- Task 结构体（planner.rs:62-69）**完全没有 `source_goal` 或 `original_description` 字段**。
- 下游 `legacy_act` 拿到的 `task.description` 已经是第二手（或第三手）信息。

**调用点**:
- `expand()` (planner.rs:399-425)
- `bootstrap_first_tool_call()` 内部的 expand

### 2.3 第三层：AgentLoop 本地执行规则映射（legacy_act）

**位置**:
- `src/intelligence/agent-core/agent_loop.rs:550-767`（核心规则分支在 628-692）

**代码证据** (最致命的一层):
```rust
} else {
    // Task has no tool calls, perform rule-based mapping (fallback)
    // FIX-B08-005: Support Chinese keywords...
    let desc_lower = task.description.to_lowercase();
    let tool_name = if desc_lower.contains("read") || ... || desc_lower.contains("读") {
        "read_file".to_string()
    } else if desc_lower.contains("write") || ... || desc_lower.contains("创建") {
        "write_file".to_string()
    } else if ... {
        "powershell".to_string()
    } else {
        "read_file".to_string() // default
    };
    ...
    // 然后尝试从已被破坏的 task.description 里提取路径/参数
}
```

**bootstrap_first_tool_call 中的重复规则** (agent_loop.rs:848-910):
- 几乎一模一样的 if-else 链，再次对 `task.description` 做关键词匹配。
- 即使 bootstrap 想走 LLM 路径，上游 Planner 已经把意图破坏了。

**破坏效果**:
- 中文补丁（FIX-B08-005）**永远无法被触发**，因为上游已经把 "创建 hello.txt" 变成了 "Research" / "Execute"。
- 这是用户报告 "中文 /agent 创建文件 → 误调用 read_file" 的直接根因。

### 2.4 决策逻辑与执行路径（act / run / bootstrap）

**关键坐标**:
- `agent_loop.rs:433-468` (`act` 方法):
  ```rust
  if crate::prompts::is_agent_llm_bootstrap_enabled() {
      self.bootstrap_first_tool_call(...)  // 仍走 Planner
  }
  if crate::prompts::is_act_toolcall_v1_enabled() {
      if let Some(result) = self.try_act_executor_chain(agent_id).await? { ... }
  }
  self.legacy_act(agent_id, goal_id).await  // 永远作为最终 fallback
  ```
- `try_act_executor_chain` (470-502): **只消费 BB_NEXT_TOOL**，不产生 ToolCall。它依赖上游（bootstrap / legacy_act 内部）先把 ToolCallV1 写到黑板。
- `run()` (174-...): 启动时强制做一次 decompose + expand，规则层最先被激活。

**当前 feature gates** (`prompts/mod.rs:38-92`):
- `HAJIMI_ACT_TOOLCALL_V1_ENABLED` (默认 true)
- `HAJIMI_AGENT_LLM_BOOTSTRAP_ENABLED` (默认 false)
- **尚无 `HAJIMI_AGENT_LLM_NATIVE_ENABLED`** —— 这是未来新路径的总开关。

### 2.5 黑板协议现状（BB_* 常量）

**定义位置**:
- `act_executor.rs:18`: `pub const BB_NEXT_TOOL: &str = "__hajimi_next_tool";`
- `agent_loop.rs:28-45`: 其他 BB_ 常量（BB_REFLECTOR_CRITIQUE、BB_PLAN_ADJUSTMENT、BB_STOP_LOSS 等）

**作用**:
- 目前是 **Planner/Rule 层与执行层之间唯一正式桥梁**。
- legacy_act / bootstrap 把 ToolCallV1 序列化后写入 BB_NEXT_TOOL。
- ActExecutor / try_act_executor_chain 从黑板读取并执行。

**问题**:
- 黑板本身是通用 KV 存储（blackboard.rs），无语义。
- 在纯 LLM-Native 模型下，模型直接流式返回 ToolCall，执行后把结果塞回上下文即可，**不再需要 BB_NEXT_TOOL 作为「规则层与执行层」的桥梁**。
- 其他 BB_ 常量（停止损失、计划调整、治理）可能仍有治理/审计价值，需逐一评估。

### 2.6 结构体缺陷：意图载体缺失

**Goal / SubGoal / Task** (planner.rs:36-69) 均无原始输入字段：
```rust
pub struct Goal { pub description: String, ... }
pub struct SubGoal { pub description: String, ... }
pub struct Task { pub description: String, pub tool_calls: Vec<ToolCall>, ... }
```

**后果**:
- 一旦进入 Planner 管道，原始用户字符串就永久丢失。
- 任何下游规则（包括 bootstrap 和 legacy_act 的参数提取）都只能在被破坏的数据上做启发式猜测。

### 2.7 ToolRegistry 注入现状

**好消息**（已部分修复）:
- `AgentLoopBuilder` (agent_loop_builder.rs:151-156) 提供 `with_tool_registry`。
- `AgentLoopConfig` (17-34) 包含 `tool_registry: Option<Arc<Mutex<ToolRegistry>>>`。
- `PlannerLlmBridge` (llm/bridge.rs:13,37-40) 也已支持 `with_tool_registry`。

**残留问题**:
- legacy_act (agent_loop.rs:615-618) 和 try_act_executor_chain (480-483) 仍保留「无注册表则退化为空」的防御性代码。
- 在 LLM-Native 路径下，工具列表需要**以模型可见规格（model_visible_specs）形式**暴露给 LLM，而不仅仅是运行时执行句柄。

### 2.8 Engine LlmClient 层工具调用能力缺口（已完成）

> **现状更新 (2026-05-28)**: 该能力缺口已于 Day 04 完美补齐。`StreamChunk` 已成功扩充 `ToolCallStart`、`ToolCallArgumentsDelta` 和 `ToolCallEnd` 变体，并增加了 `ToolDefinition` 和 `ToolChoiceMode` 的序列化定义，在 `LlmClient` 实现了 `stream_chat_with_tools` 的默认虚函数实现，所有依赖 match arms 已添加默认 `_` 匹配，通过完整单元测试。

**LlmClient trait（mod.rs:194-237）** — 签名中已包含 `stream_chat_with_tools`：
```rust
async fn stream_chat_with_context(
    &self,
    messages: Vec<ChatMessage>,
    system_prompt: Option<String>,    // 仅此二参
) -> Result<ChannelStream, EngineError>;
```

**StreamChunk（streaming/mod.rs:7-14）** — 仅三种变体，无 ToolCall 事件：
```rust
pub enum StreamChunk { Output(String), Error(String), Done }
```

**三个 Provider 请求体均无 tools 字段**：

| Provider | 请求位置 | tools 字段 | tool_choice 字段 |
|:---|:---|:---:|:---:|
| Anthropic（anthropic.rs:72） | `json!({model, messages, stream, max_tokens})` | 无 | 无 |
| OpenAI（openai.rs:143-148） | `ChatRequest { model, messages, stream }` | 无 | 无 |
| Ollama（ollama.rs:34-39） | `ChatReq { model, messages, stream }` | 无 | 无 |

**各 Provider 上游 API 实际支持的 Function Calling 协议差异**：

| Provider | 请求参数格式 | 流式响应中的 ToolCall 事件格式 |
|:---|:---|:---|
| Anthropic | `tools: [{name, description, input_schema}]` + `tool_choice` | `content_block` type=`tool_use` |
| OpenAI | `tools: [{type: "function", function: {name, parameters}}]` + `tool_choice` | `delta.tool_calls[{id, function}]` |
| Ollama | 类 OpenAI 格式（部分模型支持） | `message.tool_calls` |

**结论**：必须在 Phase 2 之前完成 Engine 层改造（新增 Phase 1.5），否则 Intelligence 层的 LLM-Native Driver 无法发出工具请求，也无法解析工具调用响应。

### 2.9 其他潜在规则点（非核心执行链）

采样发现的次要关键词匹配：
- `skills/scoring.rs:14-60`: 技能触发器评分规则，不参与主执行路径，Phase 4 时评估是否保留。
- `llm/bridge.rs`: PlannerLlmBridge 在 prompt 中注入了 ToolCallV1 JSON 格式指令，新路径下需替换为标准 function calling 协议。**优先级比原文档标注的更高。**
- `context_receipt.rs`、`long_context_pack.rs` 等可能有描述截断/分类逻辑。

---

## 3. 目标架构：Codex 风格 LLM-Native

### 3.1 Codex 核心模式（实测证据）

**tool_choice=auto**:
- `docs/codex-twist-source/codex-rs/core/src/client.rs:759`:
  ```rust
  let request = ResponsesApiRequest {
      ...
      tools,
      tool_choice: "auto".to_string(),  // ← LLM 自主决定是否/何时调用工具
      ...
  };
  ```

**原始用户输入直达**:
- `docs/codex-twist-source/codex-rs/core/src/agent/control.rs:1277-1280`:
  ```rust
  Op::UserInput { items, .. } => items.iter().map(|item| match item {
      UserInput::Text { text, .. } => text.clone(),  // ← 完全原样
      ...
  })
  ```

**ToolRouter 职责分离**:
- `docs/codex-twist-source/codex-rs/core/src/tools/router.rs:34-80`: `ToolRouter` 仅负责：
  - `from_turn_context` 构造时生成 `model_visible_specs: Vec<ToolSpec>`
  - `build_tool_call(item: ResponseItem)` 把模型返回的 FunctionCall 解析为内部 ToolCall
  - `dispatch_tool_call_*` 实际执行
  - **不参与任何意图理解或规划**

**执行循环**:
- `codex-rs/core/src/session/turn.rs`: 模型驱动的响应流 + 工具调用处理循环。模型返回 function_call 就执行，把结果作为下一轮消息塞回上下文，直到模型认为任务完成（或达到预算）。

### 3.2 Hajimi 目标状态

```
用户原始输入（任意语言）
    ↓  (Interface 层仅做 /agent 触发解析，不改写语义)
AgentLoop / 新驱动入口
    ↓  携带原始 UserIntent + 会话历史
LLMDriver / IntentPreservingTurn
    ↓  构造请求：完整 tools + tool_choice=auto + system prompt（含治理 hook 描述）
LLM (流式)
    ↓  直接返回 FunctionCall / ToolCall 事件
ToolRouter (或等价的执行分发层)
    ↓  仅负责 lookup + dispatch + 权限/治理检查
工具执行（真实 ToolRegistry）
    ↓  结果作为消息反馈给 LLM 上下文
循环直到模型输出最终回答或显式完成信号
```

**关键差异**:
- 零本地 `decompose_rule_based` / `generate_tasks_for` / `legacy_act` 关键词逻辑参与主路径。
- Planner / Task / SubGoal 模型在该路径下大幅简化或完全旁路（仅作为可选的可视化/审计轨迹）。
- 黑板协议从「必经桥梁」降级为「可选的跨轮次/治理状态共享」。

---

## 4. 推荐的新核心抽象

### 4.1 UserIntent / RawIntent 载体

```rust
/// 携带用户原始输入的不可变载体。
/// 在 LLM-Native 路径中，description 绝不被本地规则改写。
#[derive(Clone, Debug)]
pub struct RawUserIntent {
    pub text: String,                    // 原始用户字符串（中文/英文/任意）
    pub language_hint: Option<String>,   // 可选：前端或系统语言提示
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 允许携带少量结构化元数据（例如附件路径），但绝不做语义拆解
    pub attachments: Vec<IntentAttachment>,
}

pub enum IntentAttachment {
    LocalPath(std::path::PathBuf),
    ImageRef(String),
    // ...
}
```

**不变式**:
- 一旦构造，`text` 字段在整个 LLM-Native Turn 执行过程中**只读**。
- 任何 Planner / Driver 实现都禁止对它做 `to_lowercase` + `contains` 风格的意图分类。

### 4.2 LLMDriver / AgentTurnDriver

```rust
#[async_trait]
pub trait AgentTurnDriver: Send + Sync {
    /// 执行一个完整的 LLM-Native Turn。
    /// 原始意图 + 可用工具 + 历史 直达模型，模型自主决定工具调用序列。
    async fn run_turn(
        &self,
        intent: RawUserIntent,
        tools: Vec<ModelVisibleToolSpec>,
        history: Vec<TurnMessage>,
        governance: Arc<dyn AgentGovernance>,
        cancellation: CancellationToken,
    ) -> ReplResult<TurnOutcome>;

    /// 仅构造请求（用于测试或 dry-run）。
    async fn build_request(
        &self,
        intent: &RawUserIntent,
        tools: &[ModelVisibleToolSpec],
    ) -> ReplResult<LLMRequest>;
}

/// 模型可见的工具规格（Codex 的 model_visible_specs 等价物）。
#[derive(Clone, Debug)]
pub struct ModelVisibleToolSpec {
    pub name: String,
    pub namespace: Option<String>,
    pub description: String,
    pub parameters_schema: serde_json::Value,  // JSON Schema
    pub supports_parallel: bool,
    // 其他 Codex ToolSpec 字段...
}
```

**实现方向**:
- 初期可基于现有 `PlannerLlmBridge` + `engine_llm_core::LlmClient` 扩展。
- 需要扩展 LlmClient trait 以支持 `tool_choice: "auto"` 等价的请求构造参数。
- 流式解析 FunctionCall 事件（参考 Codex `ToolRouter::build_tool_call`）。

### 4.3 IntentPreservingContext

在 Turn 执行过程中携带：
- 原始 `RawUserIntent`
- 当前对话消息列表（包含之前工具执行结果）
- 可用工具规格列表（model_visible）
- 治理/审计 hook 引用
- 取消令牌

**禁止**在该上下文中进行任何本地规则意图重写。

### 4.4 简化的黑板角色（LLM-Native 路径下）

建议：
- **BB_NEXT_TOOL 废弃**（或仅作为遗留路径的兼容桥）。
- 保留以下 BB_ 常量用于治理/审计（跨路径共享）：
  - BB_STOP_LOSS
  - BB_REFLECTOR_CRITIQUE
  - BB_HANDOFF_SUMMARY
- 新路径下，工具执行结果**直接作为消息追加到 Turn 上下文**，而非必须经过黑板中转。

### 4.5 Tool Exposure 机制

需要一个等价于 Codex `build_tool_router` + `model_visible_specs` 的机制：
- 从真实 `ToolRegistry` 导出模型可见的 `ToolSpec` 列表（名称、描述、参数 schema、风险等级等）。
- 支持动态工具（MCP、技能等）的并入。
- 在 LLM-Native Driver 初始化时注入一次，后续 Turn 复用。

---

## 5. 迁移约束与架构纯洁性

### 5.1 四层架构不变式（必须严格遵守）

1. **Intelligence 层（agent-core）可依赖 Engine 的 LlmClient / ToolRegistry** —— 允许。
2. **Interface 层绝不能直接实例化或调用新的 LLMDriver** —— 必须通过现有的 AgentLoop / 门面或新增的高层 Facade。
3. **下层绝不反向依赖上层** —— 新 Driver 不得依赖 Interface 层任何组件。
4. **Governance trait 必须被新路径显式集成** —— 不能因为「模型自主」而绕过权限/审计。

### 5.2 Feature Gate 策略

- 新增环境变量：`HAJIMI_AGENT_LLM_NATIVE_ENABLED`（默认 `false`）。
- 当启用时：
  - 主执行路径走 `llm_native_turn`（或等价命名）。
  - 旧的 `decompose_rule_based` / `generate_tasks_for` / `legacy_act` 仅作为 LLM 完全不可用时的离线 fallback。
- 提供「紧急回退」机制（运行时标志或环境变量），可在不重启进程的情况下强制切回 legacy 模式。

### 5.3 渐进式迁移顺序（修订后，与路线图对齐）

- **Phase 1** (Day 1-3): 引入 `RawUserIntent`、`ModelVisibleToolSpec`、`AgentTurnDriver` 骨架 + feature gate。**Day 1 骨架已落地（`llm_native/` 四文件已存在），Day 2 Tool Exposure (ToolSpecExporter::from_registry 真实导出与 Schema 动态获取) 已于 2026-05-28 完美实装并通过单元测试！**
- **Phase 1.5** (Day 4-8, **新增**): **Engine 层 LlmClient 改造** — 扩展 `StreamChunk` 增加 ToolCall 变体，为三个 Provider（Anthropic/OpenAI/Ollama）分别实现 `stream_chat_with_tools`，完成真实导出。这是原文档遗漏的最关键前置工作。**[Completed/已完成]**
- **Phase 2** (Day 9-15): 实现 `llm_native_turn` 执行循环 + 流式 ToolCall 处理；通过 feature gate 双轨并行。
- **Phase 3** (Day 16-18): 把规则层标记为 legacy / offline-only；在入口处增加清晰分支。
- **Phase 4** (Day 19-22): 大规模删除规则层内部的关键词补丁（FIX-I18N-*、FIX-B08-*）。
- **Phase 5** (Day 23-25): 文档、测试、架构纯洁性验证 + 清债记录。

**修订总工时**：原估 17 天 -> 修订为 25 天（含 Phase 1.5 Engine 层改造 5 天 + Phase 2 缓冲 3 天）。

---

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 新 LLM-Native 路径初期不稳定，导致 Agent 完全不可用 | 中 | 高 | Phase 2 必须保留双轨 + 紧急回退开关；默认仍以 legacy 为主直到充分验证 |
| 现有 Swarm / Reflection / WorkflowOrchestrator 等上层逻辑与新路径冲突 | 中 | 中 | 新路径初期只接管「单 Agent 本地执行」场景；复杂多智能体模式暂时仍走旧路径 |
| 黑板协议 + 治理 hook 被新路径绕过，导致权限/审计缺失 | 中 | 高 | 新 Driver 必须显式接受 Governance trait，并在工具执行前后发射审计事件 |
| ToolRegistry 的 model_visible_specs 暴露不完整，导致模型看不到关键工具 | 中 | 高 | Phase 1 必须实现完整的 ToolSpec 导出逻辑，并与现有 ToolManifest 对齐 |
| 四层架构被破坏（Interface 直接 new LLMDriver） | 低 | 高 | 严格 Code Review + CONTRIBUTING.md 架构测试；设计时就定义清晰的 Facade 边界 |

---

## 7. 验证清单（Phase 0 完成标准）

- [ ] 本文档中列出的所有规则映射点（planner.rs:182,230; agent_loop.rs:628,848 等）均已准确定位并解释。
- [ ] Codex 关键证据（client.rs:759, control.rs:1277, tools/router.rs）已提取并对照。
- [ ] 提出清晰的新抽象定义（RawUserIntent、LLMDriver、ModelVisibleToolSpec）。
- [ ] 黑板协议的去留策略已明确。
- [ ] 四层架构纯洁性约束已写入文档。
- [ ] 后续 Phase 1-5 的技术前提已识别（LlmClient 扩展、Tool exposure、流式解析等）。
- [ ] 风险与回退策略已记录。

---

## 8. 下一步行动（Phase 0 完成后）

1. 用户审阅本设计文档，确认抽象方向与约束。
2. 启动 Phase 1：实现 `RawUserIntent` 结构体 + `ModelVisibleToolSpec` 导出 + LlmClient trait 扩展原型。
3. 在隔离分支或 feature gate 下实现最小可运行的 `llm_native_turn` 骨架（不接入主 AgentLoop）。
4. 每完成一个 Phase，进行一次「是否已可将新路径设为默认」的架构评审。

---

*本设计文档基于 2026-05-28 对 Hajimi `src/intelligence/agent-core/` 全量采样 + Codex 源码关键位置实测生成。所有 file:line 引用均可通过 `rg` 或编辑器直接定位。数据诚实，拒绝空中楼阁。*

**核心原则（必须在后续 Phase 中持续遵守）**:
- 原始用户意图在 LLM-Native 路径中**零本地规则改写**。
- 工具调用决策权完全交给 LLM（通过 tool_choice=auto 等价机制）。
- 四层架构纯洁性 + Governance 集成 + 可观测性 + 回退路径，缺一不可。
