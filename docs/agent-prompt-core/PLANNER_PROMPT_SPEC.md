# PLANNER_PROMPT_SPEC.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 revised / docs-only deliverable  
> Scope: Defines Planner prompt inputs, output schema, validation rules, and mapping guidance.  
> Non-scope: Does not modify `planner.rs`, `bridge.rs`, or runtime structs.

## 1. Purpose

This spec defines how the Agent Planner should turn an approved `Goal` into executable `SubGoal` and `Task` plans using the Agent Persona, relevant context, and a task-scoped tool manifest.

人话版：Planner 就是派工单的人。这份文件规定它怎么把“大活”拆成“小活”，并写清楚先后顺序、工具和验收证据。

## 2. Existing Gap Addressed

Current debt symptoms this spec addresses:

- Planner prompts are short and generic.
- Planner output has too little structure to express dependencies, suggested tools, risks, or validation.
- Planner does not receive a relevant tool manifest.
- Rule-based fallback creates fixed sequences such as Analyze → Design → Implement → Test or Reproduce → Identify → Apply fix → Verify.

人话版：以前像“去修 bug”，最多拆成“复现、定位、修复、验证”。这能跑，但不够聪明，也不知道该拿什么工具。

## 3. Planner Runtime Inputs

Planner prompts should be assembled from the following blocks:

```json
{
  "agent_persona_ref": "AGENT-PERSONA.md",
  "goal": {
    "id": "string",
    "description": "string",
    "priority": "Critical|High|Medium|Low",
    "approved": true
  },
  "workspace_context": {
    "root": "string",
    "git_state_summary": "string|null",
    "relevant_files": ["string"],
    "constraints": ["string"]
  },
  "memory_context": {
    "focus": ["string"],
    "working_summary": "string|null",
    "archive_hits": ["string"]
  },
  "tool_manifest": [
    {
      "name": "string",
      "description": "string",
      "when_to_use": ["string"],
      "parameters_schema": {},
      "risk_level": "Low|Medium|High|Critical",
      "recovery_hints": ["string"],
      "evidence_expected": ["string"]
    }
  ],
  "output_contract": "PlannerSubgoalPlanV1"
}
```

人话版：派工前要知道订单、现场、旧记录、工具箱和交付格式。缺一个就容易瞎派活。

## 4. Planner Prompt Template

```text
You are executing the Plan step of Hajimi Agent Core.

Use the stable Agent Persona rules already provided.

Goal:
{goal_json}

Relevant workspace context:
{workspace_context_json}

Relevant memory context:
{memory_context_json}

Available tools for this task:
{tool_manifest_json}

Instructions:
1. Decompose the goal into small, ordered subgoals.
2. Each subgoal must be executable using the available tools or explicitly marked as requiring user input.
3. Prefer read/inspect steps before edit steps.
4. Add dependencies where a later subgoal relies on an earlier result.
5. Include expected evidence for each subgoal.
6. Include risk level and validation intent.
7. Do not invent tools not present in the tool manifest.
8. Return only valid JSON matching PlannerSubgoalPlanV1.
```

人话版：这就是给 Planner 的派工话术：别乱编工具，先看再改，每一步都得说怎么证明完成。

## 5. Output Schema: PlannerSubgoalPlanV1

```json
{
  "schema_version": "PlannerSubgoalPlanV1",
  "goal_id": "string",
  "summary": "string",
  "subgoals": [
    {
      "id_hint": "string",
      "description": "string",
      "priority": "Critical|High|Medium|Low",
      "depends_on": ["string"],
      "suggested_tools": ["string"],
      "expected_evidence": ["string"],
      "validation_intent": "None|StaticCheck|UnitTest|IntegrationTest|ManualReview|CommandOutput",
      "risk_level": "Low|Medium|High|Critical",
      "requires_user_approval": false,
      "stop_conditions": ["string"]
    }
  ],
  "global_risks": ["string"],
  "notes": ["string"]
}
```

人话版：每个小活都要有任务名、优先级、依赖、工具、验收小票、风险和停手条件。

## 6. Field Rules

| Field | Required | Rule |
|---|---:|---|
| `schema_version` | Yes | Must equal `PlannerSubgoalPlanV1`. |
| `goal_id` | Yes | Must match the input goal ID. |
| `summary` | Yes | One-sentence plan summary. |
| `subgoals` | Yes | Non-empty array unless goal is blocked. |
| `id_hint` | Yes | Stable local identifier, e.g. `observe-runtime-prompt-gap`. |
| `depends_on` | Yes | Use `id_hint` values; empty array allowed. |
| `suggested_tools` | Yes | Names must exist in the input `tool_manifest`; empty only for user/manual steps. |
| `expected_evidence` | Yes | Must identify observable proof such as file path, command output, trace ID, or diff. |
| `validation_intent` | Yes | Must match the kind of proof expected after execution. |
| `risk_level` | Yes | Higher risk when edits, shell, Git, dependencies, or destructive operations are involved. |
| `requires_user_approval` | Yes | True for high-risk or scope-changing operations. |
| `stop_conditions` | Yes | Must include concrete conditions that trigger handoff or user ask. |

人话版：字段不是装饰品，是订单格子。格子填不对，后厨就不知道该怎么做。

## 7. Example Output

```json
{
  "schema_version": "PlannerSubgoalPlanV1",
  "goal_id": "goal-123",
  "summary": "Inspect current LLM bridge prompts, draft runtime prompt contract, and validate schemas before implementation.",
  "subgoals": [
    {
      "id_hint": "inspect-current-bridge",
      "description": "Inspect Planner and Reflector LLM bridge prompt construction and current output parsing constraints.",
      "priority": "High",
      "depends_on": [],
      "suggested_tools": ["file_read", "code_search"],
      "expected_evidence": ["source file references", "current prompt/output schema notes"],
      "validation_intent": "ManualReview",
      "risk_level": "Low",
      "requires_user_approval": false,
      "stop_conditions": ["Required source files are unavailable"]
    },
    {
      "id_hint": "draft-planner-schema",
      "description": "Draft PlannerSubgoalPlanV1 schema with dependencies, suggested tools, validation intent, and risk labels.",
      "priority": "High",
      "depends_on": ["inspect-current-bridge"],
      "suggested_tools": ["file_write"],
      "expected_evidence": ["docs/agent-prompt-core/PLANNER_PROMPT_SPEC.md"],
      "validation_intent": "ManualReview",
      "risk_level": "Low",
      "requires_user_approval": false,
      "stop_conditions": ["Schema cannot map to existing SubGoal or Task concepts"]
    }
  ],
  "global_risks": ["Prompt schema may need adapter code before runtime use"],
  "notes": ["Docs-only stage; no source code changes expected"]
}
```

人话版：这个例子展示的是“先查现场，再写规格，再验文档”。不是一上来就改桥接层。

## 8. Mapping to Current Planner Concepts

Recommended future adapter mapping:

| PlannerSubgoalPlanV1 | Existing concept | Note |
|---|---|---|
| `goal_id` | `Goal.id` | Direct match. |
| `subgoals[].description` | `SubGoal.description` | Direct match. |
| `subgoals[].priority` | `SubGoal.priority` | Direct match. |
| `subgoals[].depends_on` | `SubGoal.dependencies` | Requires id mapping from `id_hint` to generated `SubGoalId`. |
| `subgoals[].suggested_tools` | `Task.tool_calls` or metadata | Current implementation may need adapter work. |
| `expected_evidence` | Task/result metadata or receipts | Current struct may need extension or sidecar receipts. |
| `validation_intent` | Task description/result metadata | Can be encoded in task description initially. |
| `risk_level` | Governance request risk | Can help set approval level later. |

人话版：新订单格式要能塞进旧收银系统。塞不进去的字段，后续要么扩表，要么先放备注。

## 9. Validation Rules

A Planner response is PASS only if:

1. JSON parses successfully.
2. `schema_version` matches.
3. All required fields exist.
4. Suggested tools are present in the manifest.
5. Dependencies reference existing `id_hint` values.
6. No subgoal silently expands scope beyond the approved goal.
7. At least one evidence item exists per subgoal.
8. Risk and stop conditions are explicit.

人话版：派工单能不能过，不看文采，看能不能被系统读懂、被工人执行、被老板验收。

## 10. Fallback Policy

If LLM planning fails or returns invalid JSON:

1. Retry once with a compact schema reminder and the original goal.
2. If it fails again, use rule-based fallback.
3. Mark plan quality as `DEGRADED`.
4. Record the invalid output and parse error in receipts.
5. Do not claim autonomous planning quality for the degraded plan.

人话版：点餐系统崩了可以手写单，但要贴个“手写单”标签，别冒充自动化高科技。

---

## v0.2 Addendum: Adapter Implementation Notes

This section describes how `PlannerSubgoalPlanV1` can map to the current `planner.rs` concepts without forcing a high-risk rewrite in the first implementation PR.

### Existing Runtime Shape

Current planner concepts include `Goal`, `SubGoal`, `Task`, `ToolCall`, `TaskResult`, and `Plan`. Existing `SubGoal` already has `dependencies: Vec<SubGoalId>`, and `Task` already has `tool_calls: Vec<ToolCall>`.

人话版：旧订单系统不是空的，已经有目标、小目标、任务和工具调用格子；v0.2 是加字段和映射，不是推倒重做。

### Recommended DTO First

Add a DTO for LLM output before changing persisted structs:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlannerSubgoalPlanV1Dto {
    pub schema_version: String,
    pub goal_id: String,
    pub subgoals: Vec<PlannerSubgoalDto>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlannerSubgoalDto {
    pub id_hint: String,
    pub description: String,
    pub priority: Priority,
    pub depends_on: Vec<String>,
    pub suggested_tools: Vec<String>,
    pub expected_evidence: Vec<String>,
    pub validation_intent: ValidationIntent,
    pub risk_level: RiskLevel,
    pub stop_conditions: Vec<String>,
}
```

人话版：先让模型按新表格填单，再把新表格翻译成旧系统能吃的格式。别一开始就改数据库大动脉。

### Mapping `id_hint` to Runtime IDs

`id_hint` is a stable local identifier such as `inspect-current-bridge`. Runtime IDs should remain deterministic within a goal:

```rust
fn runtime_subgoal_id(goal_id: &str, id_hint: &str) -> SubGoalId {
    format!("{}-{}", goal_id, id_hint)
}
```

Rules:

- `id_hint` MUST be unique within `subgoals[]`.
- `depends_on[]` references `id_hint`, not runtime IDs.
- Adapter converts all dependency hints to runtime IDs after validating uniqueness.

人话版：模型给的是“厨房备菜”这种短名，系统落地时要换成“订单123-厨房备菜”这种不会撞车的编号。

### Backward-Compatible Struct Extension

When Owner approves struct changes, extend `SubGoal` with optional or defaultable fields:

```rust
pub struct SubGoal {
    pub id: SubGoalId,
    pub parent_goal: GoalId,
    pub description: String,
    pub priority: Priority,
    pub status: PlanStatus,
    pub tasks: Vec<TaskId>,
    pub dependencies: Vec<SubGoalId>,

    // v0.2 optional extensions
    pub suggested_tools: Vec<String>,
    pub expected_evidence: Vec<String>,
    pub validation_intent: ValidationIntent,
    pub risk_level: RiskLevel,
    pub stop_conditions: Vec<String>,
}
```

If this breaks too many tests, store these fields in metadata first:

```rust
subgoal_metadata.insert(subgoal.id.clone(), serde_json::to_value(extension)?);
```

人话版：能加栏位就加栏位；暂时加不了，就先贴便签，别为了多写几个字段把收银系统砸了。

### Suggested Tools Validation

Before accepting a plan:

```rust
for tool in &subgoal.suggested_tools {
    if registry.get(tool).is_none() {
        return Err(PlanValidationError::UnknownTool(tool.clone()));
    }
}
```

If registry access is not available in the first phase, mark validation as `UNKNOWN_NOT_WIRED` in receipts.

人话版：派工单上写“用激光锅铲”，仓库里没有，那就是无效派工。别让模型发明工具。

### Fallback Strategy

If `PlannerSubgoalPlanV1` parsing fails:

1. Try JSON repair once.
2. If still invalid, fall back to current DTO `[ { description, priority } ]`.
3. If that also fails, use rule-based fallback.
4. Write degraded status to trace:

```text
__hajimi_planner_mode = "DEGRADED_RULE_BASED"
```

人话版：新收银机扫码失败，先修二维码；还不行就用旧收银机；再不行手写单，但要贴“降级处理”。
