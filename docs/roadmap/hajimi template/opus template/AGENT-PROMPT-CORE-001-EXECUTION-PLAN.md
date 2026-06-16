# AGENT-PROMPT-CORE-001 执行计划 — Day 4~14 每日细化

> **文档版本**: 1.0  
> **所属 Roadmap**: [INTEGRATION_ROADMAP.md](../INTEGRATION_ROADMAP.md)  
> **前置条件**: Phase 0（文档落地）和 Phase 1（System Prompt 注入）已完成  
> **最后更新**: 2026-04-30  

---

## 已完成的基线

| Phase | 天数 | 状态 | 关键 Commit |
|:---|:---|:---:|:---|
| Phase 0 | Day 1 | ✅ 完成 | `85bd53c` — 8 份核心文档落地 |
| Phase 1 | Day 2-3 | ✅ 完成 | `c32284d` + `305293f` — Persona 注入 + feature-gate |

**当前代码基线**:
- `cargo check --workspace`: 0 errors
- `cargo test -p intelligence-agent-core --lib`: 105 passed
- `PlannerLlmBridge` 和 `ReflectorLlmBridge` 已改用 `stream_chat_with_context`
- Feature-gate: `HAJIMI_PROMPT_PERSONA_ENABLED` 环境变量控制

---

## Phase 2: Planner + ToolManifest（Day 4-6）

> **目标**: 让 Planner 生成工具感知的子目标，使用 `PlannerSubgoalPlanV1` Schema 和筛选后的 `ToolManifest`

---

### Day 4: ToolManifestGenerator 骨架 + Planner DTO 定义

**预计工时**: 4-6 小时  
**风险等级**: 🟡 中（新增模块，但无现有代码侵入）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 定义 `ToolManifestEntryV1` | 新建 `tool_manifest.rs` | `struct ToolManifestEntryV1 { name: String, description: String, category: ToolCategory, available: bool, risk_level: RiskLevel, requires_confirmation: bool, parameters_schema: serde_json::Value, when_to_use: Vec<String>, do_not_use_when: Vec<String>, recovery_hints: Vec<String>, evidence_expected: Vec<String>, known_failure_kinds: Vec<String> }` |
| 2 | 定义 `ToolCategory` + `RiskLevel` | `tool_manifest.rs` | `enum ToolCategory { FileRead, FileWrite, Search, Git, Build, Test, Lsp, Mcp, Shell, Analysis, Docs, Network, Other }`；`enum RiskLevel { Low, Medium, High, Critical }` |
| 3 | 定义 `ToolManifestRequest` | `tool_manifest.rs` | `struct ToolManifestRequest { goal_description: String, step_type: StepType, current_task: Option<String>, recently_failed_tools: Vec<String>, available_budget_tokens: usize, max_tools: usize }` |
| 4 | 实现 `ToolManifestGenerator` 骨架 | `tool_manifest.rs` | `pub struct ToolManifestGenerator { registry: Arc<ToolRegistry>, catalog: ToolCatalog }`；`impl ToolManifestGenerator { pub fn generate(&self, request: ToolManifestRequest) -> Vec<ToolManifestEntryV1> { ... } }` |
| 5 | 实现评分算法骨架 | `tool_manifest.rs` | `+5` category 匹配 intent；`+3` keyword 匹配；`+2` step type 常用；`-10` recently_failed；`-99` unavailable；排序取 top `max_tools`（默认 15） |
| 6 | 定义 `PlannerSubgoalPlanV1Dto` | 新建 `planner_dto.rs` | `struct PlannerSubgoalPlanV1Dto { schema_version: String, goal_id: String, summary: String, subgoals: Vec<PlannerSubgoalDto>, global_risks: Vec<String>, notes: Vec<String> }` |
| 7 | 定义 `PlannerSubgoalDto` | `planner_dto.rs` | `struct PlannerSubgoalDto { id_hint: String, description: String, priority: Priority, depends_on: Vec<String>, suggested_tools: Vec<String>, expected_evidence: Vec<String>, validation_intent: String, risk_level: String, requires_user_approval: bool, stop_conditions: Vec<String> }` |
| 8 | 添加 `serde::Deserialize` derive | `tool_manifest.rs` + `planner_dto.rs` | 所有 DTO 结构体添加 `#[derive(Debug, Clone, Deserialize)]` |
| 9 | 注册新模块到 `lib.rs` | `lib.rs` | 在 `pub mod prompts;` 下方添加 `pub mod tool_manifest;` 和 `pub mod planner_dto;` |

#### 验证命令

```bash
cargo check -p intelligence-agent-core
grep "ToolManifestGenerator" src/intelligence/agent-core/tool_manifest.rs
grep "PlannerSubgoalPlanV1Dto" src/intelligence/agent-core/planner_dto.rs
grep "pub mod tool_manifest" src/intelligence/agent-core/lib.rs
```

#### Day 4 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] `tool_manifest.rs` 编译通过，包含 `ToolManifestGenerator` 和 `ToolManifestEntryV1`
- [ ] `planner_dto.rs` 编译通过，包含 `PlannerSubgoalPlanV1Dto` 和 `PlannerSubgoalDto`
- [ ] `lib.rs` 注册了两个新模块
- [ ] 评分算法骨架有明确注释，未实现的部分标记 `TODO: Phase 2 Day 5`

---

### Day 5: PlannerLlmBridge 改造 + 工具清单注入

**预计工时**: 5-7 小时  
**风险等级**: 🟡 中（修改现有 bridge 和 planner，需保持 backward compatible）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 修改 `PlannerLlmBridge` 结构 | `llm/bridge.rs` | 添加 `tool_registry: Option<Arc<dyn engine_tool_system::ToolRegistry>>` 字段（注：需确认 `ToolRegistry` trait 是否存在于 engine-tool-system 公共接口） |
| 2 | 实现 `with_tool_registry` builder | `llm/bridge.rs` | `pub fn with_tool_registry(mut self, registry: Arc<dyn engine_tool_system::ToolRegistry>) -> Self { self.tool_registry = Some(registry); self }` |
| 3 | 实例化 `ToolManifestGenerator` | `llm/bridge.rs` | 在 `decompose_goal` 中，若 `tool_registry` 存在则生成 tool manifest |
| 4 | 修改 `decompose_goal` prompt | `llm/bridge.rs` | 在原有 prompt 前注入 `"Given the following available tools:\n{tool_manifest_json}\n\n"` |
| 5 | 解析 `PlannerSubgoalPlanV1Dto` | `llm/bridge.rs` | 替换原有的 `Vec<SubGoalDto>` 解析为 `PlannerSubgoalPlanV1Dto` 解析 |
| 6 | 验证 `suggested_tools` | `llm/bridge.rs` | 遍历每个 `subgoal.suggested_tools`，用 `tool_registry.get(name)` 验证存在性；不存在时从 suggested_tools 移除并记录警告 |
| 7 | 适配映射 `id_hint → SubGoalId` | `planner.rs` | 在 `HierarchicalPlanner::decompose` 中，将 `id_hint` 映射为 `format!("{}-{}", goal_id, id_hint)` |
| 8 | 新字段存入 `SubGoal.metadata` | `planner.rs` | 由于 `SubGoal` 结构体变更风险，将 `suggested_tools`, `expected_evidence`, `validation_intent`, `risk_level`, `stop_conditions` 存入 `metadata: HashMap<String, String>` |
| 9 | 实现 `depends_on` 解析 | `planner.rs` | 将 `depends_on` 中的 `id_hint` 引用映射为运行时 `SubGoalId` |

#### 关键代码片段

**PlannerLlmBridge prompt 修改**:
```rust
async fn decompose_goal(&self, goal: &Goal) -> ReplResult<Vec<SubGoal>> {
    let tool_manifest = self.tool_registry.as_ref().map(|reg| {
        let generator = ToolManifestGenerator::new(reg.clone());
        generator.generate(ToolManifestRequest {
            goal_description: goal.description.clone(),
            step_type: StepType::Plan,
            current_task: None,
            recently_failed_tools: vec![],
            available_budget_tokens: 1600,
            max_tools: 15,
        })
    });
    
    let prompt = if let Some(ref manifest) = tool_manifest {
        format!(
            "Given the following available tools:\n{}\n\n\
             Decompose the goal into sub-goals that can be executed using these tools. \
             Return ONLY valid JSON matching PlannerSubgoalPlanV1.\nGoal: {}\nPriority: {:?}",
            serde_json::to_string(manifest).unwrap_or_default(),
            goal.description, goal.priority
        )
    } else {
        // fallback to old prompt
        format!(
            "Decompose the goal into sub-goals. Return ONLY JSON array.\nGoal: {}\nPriority: {:?}",
            goal.description, goal.priority
        )
    };
    // ... rest
}
```

#### 验证命令

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
grep "suggested_tools" src/intelligence/agent-core/llm/bridge.rs
grep "ToolManifestGenerator" src/intelligence/agent-core/llm/bridge.rs
```

#### Day 5 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] `cargo test -p intelligence-agent-core --lib` 105 passed（不破坏现有测试）
- [ ] Planner prompt 包含 tool manifest 注入
- [ ] `suggested_tools` 被验证存在于 ToolRegistry
- [ ] `id_hint` 正确映射为运行时 `SubGoalId`
- [ ] 新字段通过 `metadata` HashMap 存储，不破坏 `SubGoal` 结构体

---

### Day 6: 规则降级路径 + feature-gate + 集成测试

**预计工时**: 4-5 小时  
**风险等级**: 🟢 低（主要是测试和 fallback 逻辑）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 完善 `decompose` fallback | `planner.rs` | LLM 返回无效 JSON 时，fallback 到 `decompose_rule_based`，并在 `Goal.metadata` 中标记 `plan_quality: "DEGRADED"` |
| 2 | 添加 `planner_v1_schema_enabled` feature-gate | `prompts/mod.rs` 或环境变量 | `pub fn is_planner_v1_enabled() -> bool { std::env::var("HAJIMI_PLANNER_V1_ENABLED").map(|v| v != "false").unwrap_or(true) }` |
| 3 | 在 `PlannerLlmBridge` 中检查 feature-gate | `llm/bridge.rs` | `if !crate::prompts::is_planner_v1_enabled() { return self.decompose_goal_legacy(goal).await; }` |
| 4 | 保留 `decompose_goal_legacy` 方法 | `llm/bridge.rs` | 将旧逻辑提取为私有方法，作为降级路径 |
| 5 | 添加 Planner DTO 单元测试 | `planner_dto.rs` | 测试 `PlannerSubgoalPlanV1Dto` 的序列化/反序列化，验证字段完整性 |
| 6 | 添加 ToolManifest 单元测试 | `tool_manifest.rs` | 测试评分算法（mock ToolRegistry，验证排序和截断） |
| 7 | 运行完整测试 | 全 workspace | `cargo test -p intelligence-agent-core --lib` + `cargo test -p engine-tool-system` |

#### 验证命令

```bash
cargo test -p intelligence-agent-core --lib
cargo test -p engine-tool-system -- test_registry
# feature-gate 关闭测试
$env:HAJIMI_PLANNER_V1_ENABLED="false"
cargo test -p intelligence-agent-core --lib
```

#### Day 6 验收标准

- [ ] `cargo test -p intelligence-agent-core --lib` 全部通过
- [ ] feature-gate 关闭时回到旧 `decompose_goal` 行为
- [ ] 无效 JSON 时 fallback 到 rule-based，标记 DEGRADED
- [ ] `PlannerSubgoalPlanV1Dto` 单元测试覆盖正常/边界情况
- [ ] ToolManifest 评分算法单元测试通过

---

## Phase 3: Reflector + Stop-Loss（Day 7-9）

> **目标**: 让反思包含根因分析、计划调整路由和止损机制

---

### Day 7: ReflectorCritiqueV1 DTO + LlmBridge 改造

**预计工时**: 5-6 小时  
**风险等级**: 🟡 中（新增 DTO，修改现有 Reflector 流程）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 定义 `RootCauseCategory` | 新建 `reflector_dto.rs` | `enum RootCauseCategory { None, ToolFailure, BadPlan, MissingContext, Permission, ValidationFailure, ParseFailure, UserInputNeeded, Unknown }` |
| 2 | 定义 `RecommendedAction` | `reflector_dto.rs` | `enum RecommendedAction { Continue, RetryWithNewArgs, UseAlternativeTool, RevisePlan, AskUser, StopAndHandoff }` |
| 3 | 定义 `RootCauseDto` | `reflector_dto.rs` | `struct RootCauseDto { category: RootCauseCategory, summary: String, details: Vec<String> }` |
| 4 | 定义 `PlanAdjustmentDto` | `reflector_dto.rs` | `struct PlanAdjustmentDto { needed: bool, recommended_action: RecommendedAction, target_step: String, rationale_summary: String }` |
| 5 | 定义 `StopLossDto` | `reflector_dto.rs` | `struct StopLossDto { triggered: bool, reason: Option<String>, handoff_required: bool }` |
| 6 | 定义 `ReflectorCritiqueV1Dto` | `reflector_dto.rs` | `struct ReflectorCritiqueV1Dto { schema_version: String, success: bool, severity: String, confidence: f32, evidence: Vec<String>, root_cause: RootCauseDto, issues: Vec<String>, new_risks: Vec<String>, suggestions: Vec<String>, plan_adjustment: PlanAdjustmentDto, stop_loss: StopLossDto }` |
| 7 | 修改 `ReflectorLlmBridge::llm_critique` prompt | `llm/bridge.rs` | 构建更丰富的 prompt，注入 `execution_result`, `validation_result`, `recent_history`，要求返回 `ReflectorCritiqueV1` JSON |
| 8 | 解析 `ReflectorCritiqueV1Dto` | `llm/bridge.rs` | 替换原有的 `Critique` 解析为 `ReflectorCritiqueV1Dto` 解析，然后映射到现有 `Critique` |
| 9 | 注册 `reflector_dto` 模块 | `lib.rs` | 添加 `pub mod reflector_dto;` |

#### 关键代码片段

**Reflector prompt 模板**:
```rust
async fn llm_critique(&self, goal: &Goal, result: &TaskResult) -> ReplResult<Critique> {
    let prompt = format!(
        "You are executing the Reflect step of Hajimi Agent Core.\n\n\
         Goal: {}\n\
         Execution result: success={}, output={}\n\
         Instructions:\n\
         1. Determine success based on evidence, not optimism.\n\
         2. If failed, identify root cause category and evidence.\n\
         3. Recommend exactly one next action.\n\
         4. Return ONLY valid JSON matching ReflectorCritiqueV1.\n\n\
         Format: {{\"schema_version\":\"ReflectorCritiqueV1\",\"success\":true,\"severity\":\"Low\",\"confidence\":0.9,\"evidence\":[],\"root_cause\":{{\"category\":\"None\",\"summary\":\"...\",\"details\":[]}},\"issues\":[],\"new_risks\":[],\"suggestions\":[],\"plan_adjustment\":{{\"needed\":false,\"recommended_action\":\"Continue\",\"target_step\":\"Decide\",\"rationale_summary\":\"...\"}},\"stop_loss\":{{\"triggered\":false,\"reason\":null,\"handoff_required\":false}}}}",
        goal.description, result.success, result.output
    );
    // ... parse ReflectorCritiqueV1Dto, then map to Critique
}
```

#### 验证命令

```bash
cargo check -p intelligence-agent-core
grep "ReflectorCritiqueV1Dto" src/intelligence/agent-core/reflector_dto.rs
```

#### Day 7 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] `reflector_dto.rs` 包含完整的 `ReflectorCritiqueV1Dto` 及所有子类型
- [ ] `llm_critique` prompt 要求返回 `ReflectorCritiqueV1` JSON
- [ ] DTO 能正确反序列化（至少一个单元测试）

---

### Day 8: 计划调整路由 + Stop-Loss 触发逻辑

**预计工时**: 5-7 小时  
**风险等级**: 🟡 中（修改 AgentLoop 控制流）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 在 `AgentLoop::run()` 中读取 `plan_adjustment` | `agent_loop.rs` | `reflect()` 返回后，从 blackboard 读取 `__hajimi_plan_adjustment_action` |
| 2 | 实现 `Continue → Decide` 路由 | `agent_loop.rs` | 无变化，直接进入 Decide 步骤 |
| 3 | 实现 `RetryWithNewArgs → Act` 路由 | `agent_loop.rs` | 修改当前 task 的 metadata，设置 `retry_with_new_args: true`，重新进入 Act 步骤 |
| 4 | 实现 `UseAlternativeTool → Act` 路由 | `agent_loop.rs` | 从 `SubGoal.metadata["suggested_tools"]` 中选择备选工具，更新当前 task |
| 5 | 实现 `RevisePlan → PlanOptimizer` 路由 | `agent_loop.rs` | 调用 `self.plan_optimizer.optimize(goal, &critique).await` |
| 6 | 实现 `AskUser → UI pause` 路由 | `agent_loop.rs` | 设置 `self.paused = true`，写入 blackboard `__hajimi_ask_user_reason` |
| 7 | 实现 `StopAndHandoff → 终止` 路由 | `agent_loop.rs` | 生成 handoff summary，设置 `outcome = LoopOutcome::Aborted`，break loop |
| 8 | 实现失败计数器 | `agent_loop.rs` | 在 blackboard 中维护 `__hajimi_failure_count`（HashMap<category, count>） |
| 9 | 实现 Stop-Loss 触发检查 | `agent_loop.rs` | 在 `reflect()` 后检查：同类失败 ≥2 次且无新证据 → 强制设置 `recommended_action = StopAndHandoff` |

#### 路由逻辑伪代码

```rust
// In AgentLoop::run(), after reflect()
let adjustment = self.blackboard.read("__hajimi_plan_adjustment").await;
if let Some(adj) = adjustment {
    match adj.value.as_str() {
        "Continue" => { /* proceed to Decide */ }
        "RetryWithNewArgs" => { /* mark task for retry, continue loop */ }
        "UseAlternativeTool" => { /* select fallback tool, continue loop */ }
        "RevisePlan" => { self.plan_optimizer.optimize(&goal, &critique).await?; }
        "AskUser" => { self.pause(); self.blackboard.write("__hajimi_ask_user_reason", &reason, "agent_loop").await; }
        "StopAndHandoff" => { 
            let summary = self.generate_handoff_summary(&goal_id).await?;
            self.blackboard.write("__hajimi_handoff_summary", &summary, "agent_loop").await;
            outcome = LoopOutcome::Aborted;
            break;
        }
        _ => { /* unknown action, log warning and continue */ }
    }
}
```

#### 验证命令

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
grep "plan_adjustment\|StopAndHandoff\|AskUser" src/intelligence/agent-core/agent_loop.rs
```

#### Day 8 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] 6 种 `RecommendedAction` 均有路由处理分支
- [ ] `RevisePlan` 调用 `PlanOptimizer::optimize()`
- [ ] `StopAndHandoff` 生成 handoff summary 并终止循环
- [ ] 失败计数器在 blackboard 中正确维护

---

### Day 9: feature-gate + 集成测试 + blackboard 协议

**预计工时**: 4-5 小时  
**风险等级**: 🟢 低

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 添加 `reflector_v1_routing_enabled` feature-gate | `prompts/mod.rs` | `pub fn is_reflector_v1_enabled() -> bool { std::env::var("HAJIMI_REFLECTOR_V1_ENABLED").map(|v| v != "false").unwrap_or(true) }` |
| 2 | 在 `AgentLoop` 中检查 feature-gate | `agent_loop.rs` | `if !crate::prompts::is_reflector_v1_enabled() { /* skip routing, use legacy Decide logic */ }` |
| 3 | 定义 blackboard 标准 keys | `agent_loop.rs` | `const BB_REFLECTOR_CRITIQUE: &str = "__hajimi_reflector_critique_v1"; const BB_PLAN_ADJUSTMENT: &str = "__hajimi_plan_adjustment_action"; const BB_STOP_LOSS: &str = "__hajimi_stop_loss_state";` |
| 4 | 保留旧 `Critique` 解析作为 fallback | `reflector.rs` | `ReflectorLlmBridge` 尝试解析 `ReflectorCritiqueV1Dto`，失败时 fallback 到旧的 `Critique` JSON 解析 |
| 5 | 添加 Reflector DTO 单元测试 | `reflector_dto.rs` | 测试成功/失败场景的序列化/反序列化 |
| 6 | 添加 Stop-Loss 单元测试 | `agent_loop.rs` 或测试文件 | 模拟连续两次同类失败，验证触发 handoff |
| 7 | 运行完整测试 | 全 workspace | `cargo test -p intelligence-agent-core --lib` |

#### 验证命令

```bash
cargo test -p intelligence-agent-core --lib
$env:HAJIMI_REFLECTOR_V1_ENABLED="false"
cargo test -p intelligence-agent-core --lib
```

#### Day 9 验收标准

- [ ] `cargo test -p intelligence-agent-core --lib` 全部通过
- [ ] feature-gate 关闭时回到旧 `Critique` 解析和 Decide 逻辑
- [ ] Stop-Loss 触发时正确生成 handoff summary
- [ ] 缺失验证被报告为 UNKNOWN 而非隐藏
- [ ] blackboard 标准 keys 有文档注释

---

## Phase 4: ContextWindowManager（Day 10-12）

> **目标**: 实现 Token 预算管理和上下文优先级组装

---

### Day 10: ContextWindowManager 核心类型 + assemble 逻辑

**预计工时**: 5-6 小时  
**风险等级**: 🟡 中（新增核心模块）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 定义 `ContextBlock` | 新建 `context_window_manager.rs` | `struct ContextBlock { name: String, priority: ContextPriority, content_type: ContentType, content: String, token_estimate: usize, truncatable: bool }` |
| 2 | 定义 `ContextPriority` | `context_window_manager.rs` | `enum ContextPriority { P0, P1, P2, P3, P4 }` |
| 3 | 定义 `ContentType` | `context_window_manager.rs` | `enum ContentType { SystemPrompt, Json, Text, Markdown }` |
| 4 | 定义 `TokenAccount` | `context_window_manager.rs` | `struct TokenAccount { max_tokens: usize, reserved_response: usize, used: usize, remaining: usize }` |
| 5 | 定义 `OmittedBlock` | `context_window_manager.rs` | `struct OmittedBlock { name: String, priority: ContextPriority, reason: String }` |
| 6 | 实现 `ContextWindowManager` | `context_window_manager.rs` | `pub struct ContextWindowManager { max_tokens: usize, reserved_response_ratio: f32 }` |
| 7 | 实现 `assemble` 方法 | `context_window_manager.rs` | `pub fn assemble(&self, blocks: Vec<ContextBlock>) -> Result<AssembledContext, ContextError>` |
| 8 | P0 快速失败逻辑 | `context_window_manager.rs` | 若 P0 block 无法容纳，返回 `Err(ContextError::Overflow(ContextPriority::P0))` |
| 9 | P1-P4 截断逻辑 | `context_window_manager.rs` | P1 尝试 compact，P2-P4 按优先级逐个省略 |
| 10 | 注册新模块 | `lib.rs` | 添加 `pub mod context_window_manager;` |

#### `assemble` 方法伪代码

```rust
pub fn assemble(&self, blocks: Vec<ContextBlock>) -> Result<AssembledContext, ContextError> {
    let mut account = TokenAccount {
        max_tokens: self.max_tokens,
        reserved_response: (self.max_tokens as f32 * self.reserved_response_ratio) as usize,
        used: 0,
        remaining: self.max_tokens,
    };
    let mut included = Vec::new();
    let mut omitted = Vec::new();
    
    for block in blocks {
        let estimate = block.token_estimate;
        if account.can_fit(estimate) {
            account.used += estimate;
            account.remaining -= estimate;
            included.push(block);
        } else {
            match block.priority {
                ContextPriority::P0 => return Err(ContextError::Overflow(ContextPriority::P0)),
                ContextPriority::P1 => {
                    // Try compact
                    if let Some(compact) = self.compact_block(&block) {
                        if account.can_fit(compact.token_estimate) {
                            account.used += compact.token_estimate;
                            account.remaining -= compact.token_estimate;
                            included.push(compact);
                            continue;
                        }
                    }
                    omitted.push(OmittedBlock { name: block.name, priority: block.priority, reason: "budget".into() });
                }
                _ => {
                    omitted.push(OmittedBlock { name: block.name, priority: block.priority, reason: "budget".into() });
                }
            }
        }
    }
    
    Ok(AssembledContext { blocks: included, omitted, total_tokens: account.used })
}
```

#### 验证命令

```bash
cargo check -p intelligence-agent-core
grep "ContextWindowManager\|assemble" src/intelligence/agent-core/context_window_manager.rs
```

#### Day 10 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] `ContextWindowManager` 包含 `assemble` 方法
- [ ] P0 overflow 返回错误而非静默省略
- [ ] `AssembledContext` 包含 `included` 和 `omitted` blocks
- [ ] 至少一个单元测试验证 assemble 逻辑

---

### Day 11: Block 压缩规则 + MemoryRetriever 集成

**预计工时**: 4-6 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 实现 `compact_block` | `context_window_manager.rs` | 按 ContentType 压缩：SystemPrompt → 精简版；ToolManifest → 截断描述；WorkingMemory → 摘要；CodeContext → 保留 symbol；Logs → 保留 error lines |
| 2 | 实现 Token 估算 | `context_window_manager.rs` | `fn estimate_tokens(content: &str) -> usize` — 优先调用 `LlmClient::count_tokens`，fallback 到启发式（中文 0.9/char，英文 1.3/word） |
| 3 | 修改 `MemoryRetriever` | `memory_retriever.rs` | 添加 `retrieve_for_context(&self, goal: &str, budget: usize) -> Vec<ContextBlock>` 方法 |
| 4 | Focus Memory 注入 | `memory_retriever.rs` | Focus Memory 始终作为 P1 block 返回 |
| 5 | Working Memory 摘要 | `memory_retriever.rs` | Working Memory 压缩为摘要，作为 P2 block 返回 |
| 6 | Archive Memory 筛选 | `memory_retriever.rs` | Archive Memory 仅返回 top-ranked 片段，作为 P3 block 返回 |
| 7 | 连接 `ContextWindowManager` 和 `MemoryRetriever` | `context_window_manager.rs` | `assemble` 支持从 `MemoryRetriever` 获取的记忆 blocks |

#### 验证命令

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
grep "compact_block\|estimate_tokens" src/intelligence/agent-core/context_window_manager.rs
```

#### Day 11 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] 每种 ContentType 的 compact 规则有独立单元测试
- [ ] Focus Memory 始终被包含（即使预算紧张）
- [ ] Archive Memory 仅在预算允许时包含
- [ ] Token 估算方法有 fallback 策略

---

### Day 12: LLM Bridge 集成 + feature-gate

**预计工时**: 4-5 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 修改 `PlannerLlmBridge::chat_and_collect` | `llm/bridge.rs` | 在调用 LLM 前，用 `ContextWindowManager::assemble` 组装 system prompt + user prompt 为 `Vec<ChatMessage>` |
| 2 | 修改 `ReflectorLlmBridge::chat_and_collect` | `llm/bridge.rs` | 同上 |
| 3 | 添加 `context_window_manager_enabled` feature-gate | `prompts/mod.rs` | `pub fn is_context_window_enabled() -> bool { std::env::var("HAJIMI_CONTEXT_WINDOW_ENABLED").map(|v| v != "false").unwrap_or(true) }` |
| 4 | 在 bridge 中检查 feature-gate | `llm/bridge.rs` | `if crate::prompts::is_context_window_enabled() { /* use assemble */ } else { /* use simple 2-message path */ }` |
| 5 | 运行完整测试 | 全 workspace | `cargo test -p intelligence-agent-core --lib` |
| 6 | 验证 Token 预算 | 手动 | 确保单次 LLM 调用 token 数在配置预算内 |

#### 验证命令

```bash
cargo test -p intelligence-agent-core --lib
$env:HAJIMI_CONTEXT_WINDOW_ENABLED="false"
cargo test -p intelligence-agent-core --lib
```

#### Day 12 验收标准

- [ ] `cargo test -p intelligence-agent-core --lib` 全部通过
- [ ] P0 blocks 从不被静默省略
- [ ] 单次 LLM 调用控制在 8K tokens 以内
- [ ] feature-gate 关闭时回到简单 2-message 路径

---

## Phase 5: ActExecutor + ToolCallV1（Day 13-14）

> **目标**: 实现工具自主选择和执行，支持多步链式调用

---

### Day 13: Act DTO + ActExecutor 骨架 + 工具调用执行

**预计工时**: 5-7 小时  
**风险等级**: 🔴 高（直接操作工具，错误代价最大）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 定义 `ActionType` | 新建 `act_dto.rs` | `enum ActionType { CallTool, CannotAct, AskUser, StopAndHandoff }` |
| 2 | 定义 `ToolCallV1` | `act_dto.rs` | `struct ToolCallV1 { schema_version: String, action_type: ActionType, tool_name: Option<String>, parameters: serde_json::Value, reason: String, expected_output: String, expected_evidence: Vec<String>, fallback_tool: Option<String>, governance_required: bool, risk_level: RiskLevel, idempotency_key: String, next_step_hint: Option<String> }` |
| 3 | 定义 `ActDecision` | `act_dto.rs` | `enum ActDecision { ToolCall(ToolCallV1), CannotAct { reason: String }, AskUser { reason: String }, StopAndHandoff { reason: String } }` |
| 4 | 实现 `ActExecutor` 结构 | 新建 `act_executor.rs` | `pub struct ActExecutor { tool_registry: Arc<dyn engine_tool_system::ToolRegistry>, governance: Arc<dyn AgentGovernance> }` |
| 5 | 实现 `ActLlmBridge` | `act_executor.rs` | `pub struct ActLlmBridge { inner: Arc<dyn engine_llm_core::LlmClient> }`；构建 Act prompt，注入 task + tool_manifest + blackboard state |
| 6 | 实现工具调用执行 | `act_executor.rs` | `pub async fn execute_tool_call(&self, call: &ToolCallV1) -> Result<ToolOutput, ToolError>` |
| 7 | 验证工具存在 | `act_executor.rs` | `self.tool_registry.get(&call.tool_name).ok_or(ToolError::new("Tool not found"))` |
| 8 | 验证参数 | `act_executor.rs` | 检查 parameters 是否为有效 JSON，若 schema 可用则验证 |
| 9 | governance 路由 | `act_executor.rs` | `if call.governance_required || call.risk_level == RiskLevel::Critical { /* route to governance */ }` |
| 10 | 执行工具 | `act_executor.rs` | `tool.execute(ToolArgs::from_json(&call.parameters)).await` |
| 11 | 注册新模块 | `lib.rs` | 添加 `pub mod act_dto;` 和 `pub mod act_executor;` |

#### Act prompt 模板

```rust
let prompt = format!(
    "You are executing the Act step of Hajimi Agent Core.\n\n\
     Current approved task:\n{}\n\n\
     Available tools:\n{}\n\n\
     Instructions:\n\
     1. Select exactly one tool.\n\
     2. Provide all required parameters.\n\
     3. Return ONLY valid JSON matching ToolCallV1.\n\n\
     Format: {{\"schema_version\":\"ToolCallV1\",\"action_type\":\"CallTool\",\"tool_name\":\"...\",\"parameters\":{{}},\"reason\":\"...\",\"expected_output\":\"...\"}}",
    task_json, tool_manifest_json
);
```

#### 验证命令

```bash
cargo check -p intelligence-agent-core
grep "ActExecutor\|ToolCallV1" src/intelligence/agent-core/act_executor.rs
```

#### Day 13 验收标准

- [ ] `cargo check -p intelligence-agent-core` 0 errors
- [ ] `ActExecutor` 包含 `execute_tool_call` 方法
- [ ] `ToolCallV1` DTO 能正确序列化/反序列化
- [ ] governance 路由对 Critical 工具生效
- [ ] 工具不存在时返回错误（不 panic）

---

### Day 14: 多步链式协议 + 重试规则 + feature-gate

**预计工时**: 5-6 小时  
**风险等级**: 🔴 高

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 定义 blackboard 链式 keys | `act_executor.rs` | `const BB_NEXT_TOOL: &str = "__hajimi_next_tool"; const BB_LAST_TOOL: &str = "__hajimi_last_tool"; const BB_LAST_TOOL_RESULT: &str = "__hajimi_last_tool_result"; const BB_LAST_ERROR: &str = "__hajimi_last_error"; const BB_FAILED_TOOL_FINGERPRINT: &str = "__hajimi_failed_tool_fingerprint"; const BB_ATTEMPT_COUNT: &str = "__hajimi_attempt_count";` |
| 2 | 实现链式协议 | `act_executor.rs` | Act 完成后将结果写入 blackboard；下次 Act 读取 blackboard 决定下一步 |
| 3 | 实现 micro-reflect | `act_executor.rs` | 工具失败时触发简短反思（仅分析当前工具错误，不进入完整 Reflect 循环） |
| 4 | 实现重试规则 | `act_executor.rs` | 相同 tool + 相同参数 fingerprint → 不重复；修正参数 → 可重试一次；两次同类失败 → 强制 `StopAndHandoff` |
| 5 | 修改 `AgentLoop::act()` | `agent_loop.rs` | 调用 `ActExecutor` 替代原有的硬编码执行逻辑 |
| 6 | 添加 `act_toolcall_v1_enabled` feature-gate | `prompts/mod.rs` | `pub fn is_act_toolcall_v1_enabled() -> bool { std::env::var("HAJIMI_ACT_TOOLCALL_V1_ENABLED").map(|v| v != "false").unwrap_or(true) }` |
| 7 | 在 `AgentLoop` 中检查 feature-gate | `agent_loop.rs` | `if crate::prompts::is_act_toolcall_v1_enabled() { /* use ActExecutor */ } else { /* use legacy act path */ }` |
| 8 | 运行完整测试 | 全 workspace | `cargo test -p intelligence-agent-core --lib` + `cargo test -p engine-tool-system` |
| 9 | 行为验证 | 手动 | 验证 Agent 能执行至少一个安全只读工具和一个验证工具 |

#### 验证命令

```bash
cargo test -p intelligence-agent-core --lib
cargo test -p engine-tool-system
$env:HAJIMI_ACT_TOOLCALL_V1_ENABLED="false"
cargo test -p intelligence-agent-core --lib
```

#### Day 14 验收标准

- [ ] `cargo test -p intelligence-agent-core --lib` 全部通过
- [ ] `cargo test -p engine-tool-system` 全部通过
- [ ] Agent 每次只选一个工具
- [ ] 工具参数为有效 JSON
- [ ] 链式工具流使用 blackboard 状态传递
- [ ] 相同失败参数不被重复调用
- [ ] High-risk 调用等待 governance 审批
- [ ] feature-gate 关闭时回到旧执行路径

---

## Day 15: 清债验证、文档闭环 & Closure

**预计工时**: 3-4 小时  
**风险等级**: 🟢 低（纯文档和验证工作）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 创建 DEBT 清偿文档 | 新建 `docs/debt/DEBT-AGENT-PROMPT-001-REMEDIATION.md` | 记录所有 finding、精确代码位置、fix SHA、实测前后对比 |
| 2 | 更新 `src/INDEX.md` | `INDEX.md` | 标记 AGENT-PROMPT-CORE-001 状态为 "Cleared" |
| 3 | 更新 `src/ARCHITECTURE.md` | `ARCHITECTURE.md` | 更新 Agent Core 提示词架构描述 |
| 4 | 运行完整验证命令集 | 全 workspace | `cargo check --workspace` + `cargo test -p intelligence-agent-core --lib` + `cargo test -p engine-tool-system` |
| 5 | 验证 5 个 Phase 的 feature-gate | 手动 | 逐个禁用每个 feature-gate，验证回滚正常 |
| 6 | 验证四层架构纯洁性 | `grep` | `grep -r "use interface\|use intelligence.*interface" src/engine/` 等，确保无反向依赖 |
| 7 | 最终 commit | git | 符合 CONTRIBUTING.md 规范的 commit message |

#### 最终回归验收清单

- [ ] `cargo check --workspace` **0 errors**
- [ ] `cargo test -p intelligence-agent-core --lib` **全部通过**
- [ ] `cargo test -p engine-tool-system` **全部通过**
- [ ] Phase 0: 8 个核心文档 + receipts 存在于 `docs/agent-prompt-core/`
- [ ] Phase 1: Persona 注入生效，`HAJIMI_PROMPT_PERSONA_ENABLED` feature-gate 可用
- [ ] Phase 2: Planner 输出包含 `suggested_tools`，`HAJIMI_PLANNER_V1_ENABLED` feature-gate 可用
- [ ] Phase 3: Reflector 输出包含 `root_cause.category` 和路由动作，`HAJIMI_REFLECTOR_V1_ENABLED` feature-gate 可用
- [ ] Phase 4: ContextWindowManager 控制 Token 预算，`HAJIMI_CONTEXT_WINDOW_ENABLED` feature-gate 可用
- [ ] Phase 5: ActExecutor 执行 ToolCallV1，`HAJIMI_ACT_TOOLCALL_V1_ENABLED` feature-gate 可用
- [ ] `src/INDEX.md`、`src/ARCHITECTURE.md` 已更新
- [ ] `docs/debt/DEBT-AGENT-PROMPT-001-REMEDIATION.md` 记录完整
- [ ] 四层架构纯洁性验证通过
- [ ] git status 干净，最终 commit 符合规范

---

## Feature-Gate 汇总

| 环境变量 | 控制 Phase | 默认值 | 回滚行为 |
|:---|:---|:---:|:---|
| `HAJIMI_PROMPT_PERSONA_ENABLED` | Phase 1 | `true` | 回到 `stream_chat(prompt)` 无 system prompt |
| `HAJIMI_PLANNER_V1_ENABLED` | Phase 2 | `true` | 回到旧 `decompose_goal` 无 tool manifest |
| `HAJIMI_REFLECTOR_V1_ENABLED` | Phase 3 | `true` | 回到旧 `Critique` 无路由 |
| `HAJIMI_CONTEXT_WINDOW_ENABLED` | Phase 4 | `true` | 回到简单 2-message 组装 |
| `HAJIMI_ACT_TOOLCALL_V1_ENABLED` | Phase 5 | `true` | 回到旧硬编码执行路径 |

---

## 工作量统计

| Phase | 天数 | 新建文件 | 修改文件 | 预计总工时 |
|:---|:---:|:---:|:---:|:---:|
| Phase 2 | Day 4-6 | 3 | 3 | 14-18h |
| Phase 3 | Day 7-9 | 1 | 3 | 14-18h |
| Phase 4 | Day 10-12 | 1 | 3 | 13-17h |
| Phase 5 | Day 13-14 | 2 | 2 | 10-13h |
| Closure | Day 15 | 1 | 2 | 3-4h |
| **总计** | **11 天** | **8** | **13** | **54-70h** |

---

*本执行计划与 AGENT-PROMPT-CORE-001 文档包同步维护。每完成一天，请在对应天数的验收清单中打勾并记录实测输出。*
