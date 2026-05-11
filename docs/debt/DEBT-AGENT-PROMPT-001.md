# DEBT-AGENT-PROMPT-001: Agent Core 缺乏核心驱动提示词

> **类型**: 架构债务 / 智能层缺陷  
> **优先级**: P0（阻塞 Agent 从"聊天机器人"升级为"自主 Agent"）  
> **创建日期**: 2026-04-30  
> **关联模块**: `intelligence/agent-core`, `engine/llm-core`, `intelligence/codex-twist`  
> **影响范围**: Agent 自主决策能力、工具调用质量、用户体验  

---

## 1. 问题概述

Hajimi Agent Core 的 7 步循环（Observe→Retrieve→Plan→Act→Reflect→Store→Decide）在**代码编排层面**是完整的，但在 **LLM 提示词工程层面**严重缺失。当前 LLM 调用是零散的 `format!` 字符串拼接，没有统一的系统提示词（System Prompt）来定义 Agent 的角色、工作流、工具使用策略和反思深度。这导致 Agent 本质上是一个**"命令执行器"**（给 LLM 发简短指令 → 期望 JSON 返回），而非**"自主 Agent"**（有角色认知、情境理解、策略规划）。

**不修复会导致**：用户感知 Hajimi 的 AI 能力与普通 ChatGPT/Claude 网页版无差异，无法体现本地 IDE Agent 的自主编辑、代码理解、渐进式修改等核心卖点。

---

## 2. 根因分析

### 2.1 默认 System Prompt 过于通用

```rust
// src/intelligence/codex-twist/src/thread.rs:157-159
pub fn system_message(&self) -> String {
    self.config.system_prompt.clone()
        .unwrap_or_else(|| "You are a helpful AI assistant.".to_string())
}
```

**问题**:  `"You are a helpful AI assistant."` 是通用聊天机器人的默认 prompt，完全没有定义：
- Agent 在 IDE 中的角色定位
- 可用的 40+ 工具及其调用规范
- 代码编辑的安全约束（Shell 白名单、路径沙箱）
- 渐进式修改原则（先读后改、验证后提交）
- 反思深度要求（不仅判断 success/failure，还要分析根因）

### 2.2 Agent Core LLM 桥接层零 System Prompt

```rust
// src/intelligence/agent-core/llm/bridge.rs (PlannerLlmBridge)
async fn chat_and_collect(&self, prompt: String) -> ReplResult<String> {
    // ...
    let prompt = format!("{}\n\n{}", crate::planner::THINKING_FORMAT_INSTRUCTION, prompt);
    let mut stream = client.stream_chat(prompt).await?;  // ← 调用的是 stream_chat，不是 stream_chat_with_context
    // ...
}
```

**问题**: 
- `PlannerLlmBridge` 和 `ReflectorLlmBridge` 都调用 `stream_chat(prompt)`（单字符串接口），而非 `stream_chat_with_context(messages, system_prompt)`（支持 system prompt 的接口）
- 虽然 `engine-llm-core` 的 `LlmClient` trait 已经定义了 `stream_chat_with_context(messages, system_prompt)`，但 Agent Core 完全未利用
- 每次 LLM 调用都是**无状态**的，LLM 不知道自己是 Agent 的一部分

### 2.3 提示词碎片化 — 每个模块各自为政

| 模块 | Prompt 示例 | 问题 |
|:---|:---|:---|
| Planner | `"Decompose the goal into sub-goals. Return ONLY JSON array."` | 无角色、无上下文、无工具感知 |
| Reflector | `"Critique execution result. Return ONLY JSON."` | 无反思深度要求、无优化策略 |
| Memory Gateway | `"请将以下对话历史压缩为一段简洁的摘要..."` | 唯一质量尚可的 prompt，但与 Agent Core 无关 |

**问题**: 各模块的 prompt 是独立的 `format!` 字符串，没有统一的"Agent 人格"来贯穿整个 7 步循环。LLM 在 Plan 步骤不知道 Reflect 步骤会做什么，在 Act 步骤不知道工具的限制。

### 2.4 无工具感知提示词 — 40+ 工具对 LLM 不可见

```rust
// src/intelligence/agent-core/planner.rs
async fn decompose(&mut self, goal_id: &GoalId) -> ReplResult<Vec<SubGoalId>> {
    let sgs = if let Some(ref llm) = self.llm { 
        llm.decompose_goal(&goal).await?  // ← prompt 中完全没有提及可用工具
    } else { 
        self.decompose_rule_based(&goal)  // ← 降级为规则匹配
    };
    // ...
}
```

**问题**: 
- `engine-tool-system` 注册了 40+ 工具（文件操作、Git、搜索、构建、LSP、MCP）
- 但 Planner 分解目标时，prompt 中**完全没有**告诉 LLM 有哪些工具可用、每个工具什么时候该用
- 结果是：LLM 只能做纯文本推理，无法做出"为了实现 X，我应该先搜索文件 Y，然后读取内容，再调用 LSP 获取定义"这类工具链决策
- 工具调用退化为**硬编码规则**（`desc.contains("implement")` → 固定子目标序列）

### 2.5 反思能力浅层 — 只有二元判断

```rust
// src/intelligence/agent-core/reflector.rs (Rule-based fallback)
fn critique_rule_based(&self, _goal: &Goal, result: &TaskResult) -> Critique {
    if result.success {
        Critique { success: true, issues: vec![], suggestions: vec!["Continue with next task".to_string()], severity: CritiqueSeverity::Low }
    } else {
        Critique { success: false, issues: vec![result.output.clone()], suggestions: vec!["Retry with modified parameters".to_string()], severity: CritiqueSeverity::High }
    }
}
```

**问题**:
- 规则降级只有 success/failure 二元分支
- LLM 驱动的 critique prompt 也极其简单： `"Critique execution result. Return ONLY JSON."`
- 没有要求 LLM 分析：为什么失败？根因是什么？是否有替代方案？是否需要调整整体策略？
- 反思结果没有**反馈到 Planner** 来调整后续计划

### 2.6 上下文管理缺失 — Token 预算与记忆注入无策略

```rust
// src/intelligence/agent-core/agent_loop.rs
const MAX_ITERATIONS: usize = 100;
const ITERATION_BUDGET: usize = 50;
```

**问题**:
- 有迭代次数限制（100 次）和预算限制（50 次），但**没有 Token 预算管理**
- `MemoryRetriever` 检索记忆后，没有策略决定哪些记忆注入 prompt、优先级如何、Token 超限如何截断
- 没有区分：Hot 记忆（Focus，必须注入）vs Warm 记忆（Working，选择性注入）vs Cold 记忆（Archive，仅 RAG 检索）
- 结果是：要么注入过多记忆导致 prompt 爆炸，要么注入过少导致 LLM 缺乏上下文

---

## 3. 影响评估

### 3.1 用户感知层面

| 场景 | 当前体验 | 预期体验 |
|:---|:---|:---|
| 用户说"帮我修复这个 bug" | Agent 生成固定子目标序列（Reproduce→Identify→Fix→Verify），机械执行 | Agent 先搜索相关文件 → 读取代码 → 分析根因 → 提出修复方案 → 询问用户确认 → 执行修改 → 运行测试验证 |
| 用户说"重构这个模块" | Agent 不知道从何下手，可能直接报错 | Agent 先分析模块依赖 → 识别重构范围 → 制定渐进式计划 → 每步修改后编译验证 |
| 工具调用失败 | Agent 标记 failure，可能重试同样参数 | Agent 分析错误信息 → 调整参数 → 或选择替代工具 → 或请求用户指导 |

### 3.2 技术层面

- **Plan 质量低**: LLM 无法利用工具信息，只能做纯文本分解，子目标与实际工具能力不匹配
- **Act 盲目性**: Task 执行时不知道有哪些工具可用，工具调用靠硬编码
- **Reflect 无效**: 反思结果没有反馈回路，同样的错误会重复发生
- **Context 浪费**: 缺乏 Token 管理策略，LLM 调用成本高、响应慢

---

## 4. 修复路径

### 4.1 推荐方案：构建 AGENT-PERSONA 提示词框架

**实施成本**: 2-3 天 / 5-8 文件  
**预期收益**: Agent 从"命令执行器"升级为"自主 Agent"，工具调用质量提升 3-5x

#### Step 1: 设计核心系统提示词 (`AGENT-PERSONA.md`)

创建一个完整的 Agent 人格定义文档（约 2000-3000 tokens），包含：

```markdown
# Hajimi Agent Core System Prompt

## 角色定义
你是 Hajimi IDE 的自主 Agent，一个本地优先的 AI 软件开发助手。
你运行在用户本地，拥有对本地文件系统的受控访问权限。
你的目标是通过自主规划、工具调用和代码修改，帮助用户完成软件开发任务。

## 工作原则
1. 最小侵入：只修改必要的代码，保持现有代码风格
2. 验证优先：每次修改后运行测试或编译验证
3. 渐进式：大问题分解为小步骤，每步验证后再继续
4. 安全第一：严格遵守 Shell 白名单（38 个命令）和路径沙箱
5. 透明性：向用户解释你的决策过程，重要操作请求确认

## 可用工具（动态注入）
你将收到当前可用的工具列表，每个工具包含：
- name: 工具名称
- description: 工具功能描述
- when_to_use: 适用场景
- parameters: 参数规范（类型、必填、示例）
- error_handling: 常见错误及恢复策略

## 7步循环规范
[详细描述每个步骤的目标、输入、输出、决策逻辑]

## 反思深度要求
不要只判断 success/failure，必须分析：
- 为什么成功/失败？根因是什么？
- 是否有更好的方法或替代工具？
- 是否引入了新的风险（如破坏现有功能）？
- 是否需要调整后续计划？

## 错误恢复策略
- 工具调用失败 → 分析错误信息 → 调整参数重试（最多3次）
- 编译错误 → 读取错误信息 → 定位问题文件 → 修复
- 测试失败 → 分析失败原因 → 修复代码或调整测试
- 连续失败3次 → 暂停并请求用户指导

## 输出格式
所有响应必须遵循：
<thinking>...</thinking>  ← 你的推理过程
<response>...</response>  ← 最终输出（JSON 或纯文本）
```

#### Step 2: 修改 LLM 桥接层注入 System Prompt

```rust
// src/intelligence/agent-core/llm/bridge.rs
impl PlannerLlmBridge {
    async fn chat_and_collect(&self, prompt: String) -> ReplResult<String> {
        let client = /* select client */;
        
        // 构建完整的消息上下文
        let system_prompt = load_agent_persona().await?;  // 加载 AGENT-PERSONA.md
        let messages = vec![
            ChatMessage { role: "system".into(), content: system_prompt, timestamp: None },
            ChatMessage { role: "user".into(), content: prompt, timestamp: None },
        ];
        
        // 使用支持 system_prompt 的接口
        let mut stream = client.stream_chat_with_context(messages, None).await?;
        // ...
    }
}
```

#### Step 3: 工具描述动态注入

修改 `PlannerLlmBridge::decompose_goal`，将 `ToolRegistry` 中可用工具的描述注入 prompt：

```rust
async fn decompose_goal(&self, goal: &Goal) -> ReplResult<Vec<SubGoal>> {
    let tools_description = self.tool_registry.describe_available_tools().await?;
    let prompt = format!(
        "Given the following available tools:\n{}\n\n\
         Decompose the goal into sub-goals that can be executed using these tools. \
         Return ONLY JSON array.\nGoal: {}\nPriority: {:?}\n\n\
         Format: [{\"description\":\"...\",\"priority\":\"High\",\"suggested_tools\":[\"...\"]}]",
        tools_description, goal.description, goal.priority
    );
    // ...
}
```

#### Step 4: 上下文管理策略

实现 `ContextWindowManager`：
- Token 预算分配：System Prompt (30%) + 记忆注入 (40%) + 当前任务 (30%)
- 记忆优先级：Focus (必注入) → Working (摘要注入) → Archive (RAG 检索)
- 超限截断策略：从最低优先级开始截断，保留关键决策和代码上下文

### 4.2 替代方案：渐进式增强（成本更低）

**实施成本**: 4-6 小时 / 2-3 文件  
**预期收益**: 基础 Agent 能力提升，为完整框架铺路

1. **先改默认 system prompt**: 将 `"You are a helpful AI assistant."` 替换为包含基本角色和约束的 200-token prompt
2. **Planner prompt 增强**: 在分解目标时注入当前 workspace 的文件树摘要
3. **Reflect prompt 增强**: 要求 LLM 输出 `issues`/`suggestions`/`severity` 时附带根因分析

---

## 5. 验证方式

修复完成后，通过以下方式验证：

1. **Prompt 审计**: 检查每次 LLM 调用是否包含完整的 system prompt + 工具描述 + 上下文
2. **工具调用测试**: 给 Agent 一个需要多工具协作的复杂任务（如"重构 utils.rs 中的函数并确保测试通过"），验证 Agent 是否能自主规划工具链
3. **反思深度测试**: 故意让某个工具调用失败，验证 Agent 是否能分析根因并调整策略
4. **Token 效率测试**: 验证上下文管理策略是否将 Token 使用量控制在合理范围（单次调用 < 8K tokens）

---

## 6. 关联债务

| ID | 描述 | 关系 |
|:---|:---|:---|
| DEBT-AGENT-USABILITY-001 | 3 个运行时可用性 bug | 独立问题，但修复后可提升用户对 Agent 能力的感知 |
| DEBT-PHASE1-001 | WebRTC 传输层 | 无关 |
| DEBT-WASM-001 | SAB Node.js 兼容性 | 无关 |

---

## 7. 结论

当前 Hajimi Agent Core 在**架构层面**（7 步循环、Swarm、Governance、Memory）是完整且先进的，但在**提示词工程层面**（LLM 交互质量）严重落后。这就像一个拥有顶级赛车底盘但没有调校好发动机的车队 — 基础设施到位，但核心驱动力不足。

**基线判定**: 【必须修复】（P0）  
**理由**: 没有核心驱动提示词，Agent Core 的所有架构优势都无法转化为用户可感知的智能体验。用户不会关心底层有多少个循环步骤，只会关心 Agent 是否能"理解"任务、"自主"完成、"聪明"地处理异常。

---

*本债务文档与代码同步维护。如有架构变更或提示词策略调整，请务必同步更新。*
