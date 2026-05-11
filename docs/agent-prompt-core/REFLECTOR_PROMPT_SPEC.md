# REFLECTOR_PROMPT_SPEC.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 revised / docs-only deliverable  
> Scope: Defines Reflector prompt inputs, output schema, feedback rules, and stop-loss behavior.  
> Non-scope: Does not modify `reflector.rs`, `bridge.rs`, `PlanOptimizer`, or persistence code.

## 1. Purpose

This spec defines how the Agent Reflector should evaluate execution results, identify root causes, assess risk, and recommend plan adjustments. Reflection must be actionable, not a shallow success/failure label.

人话版：Reflector 是复盘官。它不能只说“过了/没过”，必须说证据是什么、问题在哪里、下一步怎么改。

## 2. Existing Gap Addressed

Current debt symptoms this spec addresses:

- LLM critique prompt is too short.
- Rule-based fallback only branches on `success` and returns generic suggestions.
- Reflection does not explicitly model root cause, retry strategy, new risk, or stop-loss.
- Optimized-plan suggestions are plain text and may be hard for Planner / Decide to consume.

人话版：以前像“这锅糊了，建议重做”。现在要说“火太大、锅太薄、下次中火、如果再糊就停单”。

## 3. Reflector Runtime Inputs

```json
{
  "agent_persona_ref": "AGENT-PERSONA.md",
  "goal": {
    "id": "string",
    "description": "string",
    "priority": "Critical|High|Medium|Low"
  },
  "task": {
    "id": "string|null",
    "description": "string|null",
    "suggested_tools": ["string"],
    "expected_evidence": ["string"]
  },
  "execution_result": {
    "success": false,
    "output": "string",
    "timestamp": "RFC3339 string|null",
    "tool_name": "string|null",
    "tool_args_summary": "string|null",
    "exit_code": "integer|null"
  },
  "validation_result": {
    "ran": false,
    "command": "string|null",
    "passed": "boolean|null",
    "output_summary": "string|null"
  },
  "recent_history": [
    {
      "step": "Observe|Retrieve|Plan|Act|Reflect|Store|Decide",
      "summary": "string",
      "result": "string"
    }
  ],
  "output_contract": "ReflectorCritiqueV1"
}
```

人话版：复盘前要拿到订单、这一步要做啥、实际输出、有没有测试、前面翻没翻过车。

## 4. Reflector Prompt Template

```text
You are executing the Reflect step of Hajimi Agent Core.

Use the stable Agent Persona rules already provided.

Goal:
{goal_json}

Task:
{task_json}

Execution result:
{execution_result_json}

Validation result:
{validation_result_json}

Recent relevant history:
{recent_history_json}

Instructions:
1. Determine whether the task succeeded based on evidence, not optimism.
2. If failed, identify root cause category and evidence.
3. Identify whether the original plan, selected tool, arguments, context, permission, or validation caused the issue.
4. Recommend exactly one next action category.
5. Trigger stop-loss if the same failure pattern repeated or no new evidence was produced.
6. Return only valid JSON matching ReflectorCritiqueV1.
```

人话版：复盘官要按证据说话，别“感觉还行”。下一步也只能给一个主动作，不要菜单开满天飞。

## 5. Output Schema: ReflectorCritiqueV1

```json
{
  "schema_version": "ReflectorCritiqueV1",
  "success": false,
  "severity": "Low|Medium|High|Critical",
  "confidence": 0.0,
  "evidence": ["string"],
  "root_cause": {
    "category": "None|ToolFailure|BadPlan|MissingContext|Permission|ValidationFailure|ParseFailure|UserInputNeeded|Unknown",
    "summary": "string",
    "details": ["string"]
  },
  "issues": ["string"],
  "new_risks": ["string"],
  "suggestions": ["string"],
  "plan_adjustment": {
    "needed": true,
    "recommended_action": "Continue|RetryWithNewArgs|UseAlternativeTool|RevisePlan|AskUser|StopAndHandoff",
    "target_step": "Observe|Retrieve|Plan|Act|Reflect|Store|Decide",
    "rationale_summary": "string"
  },
  "stop_loss": {
    "triggered": false,
    "reason": "string|null",
    "handoff_required": false
  }
}
```

人话版：这张复盘表会告诉你：成没成、凭什么、哪里坏、危险多大、下一步该干嘛、要不要关火交接。

## 6. Field Rules

| Field | Required | Rule |
|---|---:|---|
| `schema_version` | Yes | Must equal `ReflectorCritiqueV1`. |
| `success` | Yes | Based on execution and validation evidence. |
| `severity` | Yes | Highest current risk if continuing. |
| `confidence` | Yes | Number from 0.0 to 1.0. Low confidence must produce cautious action. |
| `evidence` | Yes | Must include concrete output, file path, test result, trace ID, or reason evidence is missing. |
| `root_cause.category` | Yes | Must be `None` only when success is true and no issue remains. |
| `issues` | Yes | Empty only for clean success. |
| `new_risks` | Yes | Empty allowed. |
| `suggestions` | Yes | Must be actionable. |
| `plan_adjustment.recommended_action` | Yes | Exactly one primary action. |
| `stop_loss` | Yes | Must reflect repeat failures and evidence gaps. |

人话版：复盘表不是朋友圈小作文。每格都有用途，尤其是证据和下一步。

## 7. Example Failure Output

```json
{
  "schema_version": "ReflectorCritiqueV1",
  "success": false,
  "severity": "High",
  "confidence": 0.82,
  "evidence": [
    "Planner output failed JSON parsing",
    "Returned prose before JSON despite JSON-only instruction"
  ],
  "root_cause": {
    "category": "ParseFailure",
    "summary": "The LLM did not follow the required machine-readable output contract.",
    "details": [
      "The output included non-JSON text",
      "The bridge parser expects serde_json::from_str-compatible JSON"
    ]
  },
  "issues": [
    "Planner result cannot be converted into subgoals",
    "Autonomous loop cannot continue safely with malformed plan"
  ],
  "new_risks": [
    "A naive retry may produce another invalid response if schema is not restated"
  ],
  "suggestions": [
    "Retry once with compact schema reminder and no extra prose",
    "If retry fails, fall back to rule-based planning and mark plan quality DEGRADED"
  ],
  "plan_adjustment": {
    "needed": true,
    "recommended_action": "RetryWithNewArgs",
    "target_step": "Plan",
    "rationale_summary": "The failure is likely format-related and may be recoverable with stricter schema reminder."
  },
  "stop_loss": {
    "triggered": false,
    "reason": null,
    "handoff_required": false
  }
}
```

人话版：这不是简单说“JSON 错了”。它指出 parser 吃不进去、为什么、只允许再试一次，失败就降级。

## 8. Example Success Output

```json
{
  "schema_version": "ReflectorCritiqueV1",
  "success": true,
  "severity": "Low",
  "confidence": 0.9,
  "evidence": [
    "Expected document exists at docs/agent-prompt-core/PLANNER_PROMPT_SPEC.md",
    "Schema section includes required fields and validation rules"
  ],
  "root_cause": {
    "category": "None",
    "summary": "The task met its expected evidence criteria.",
    "details": []
  },
  "issues": [],
  "new_risks": [],
  "suggestions": ["Continue with the next approved subgoal"],
  "plan_adjustment": {
    "needed": false,
    "recommended_action": "Continue",
    "target_step": "Decide",
    "rationale_summary": "Evidence is sufficient to proceed."
  },
  "stop_loss": {
    "triggered": false,
    "reason": null,
    "handoff_required": false
  }
}
```

人话版：成功也要带证据，不是“我觉得挺好”。

## 9. Mapping to Current Reflector Concepts

| ReflectorCritiqueV1 | Existing concept | Note |
|---|---|---|
| `success` | `Critique.success` | Direct match. |
| `severity` | `Critique.severity` | Direct match. |
| `issues` | `Critique.issues` | Direct match. |
| `suggestions` | `Critique.suggestions` | Direct match. |
| `root_cause` | New / sidecar | Requires struct extension or metadata sidecar. |
| `plan_adjustment` | `optimized_plan` / Decide input | Should become machine-readable. |
| `stop_loss` | Governance / Decide | Should trigger handoff path. |
| `evidence` | Receipts / trace | Should be persisted outside current Critique if struct is unchanged. |

人话版：旧表只有四个格子，新表格子更多。后续实现可以先把新字段放 sidecar，再慢慢改结构。

## 10. Stop-Loss Policy

Set `stop_loss.triggered = true` and recommend `StopAndHandoff` if any condition is met:

1. Same failure category appears twice with no new evidence.
2. Two consecutive cycles produce no new verifiable artifact or validation result.
3. The next action requires credentials, private access, or external approval not available to the Agent.
4. Continuing risks destructive changes outside approved scope.
5. Parser or runtime contract failures persist after one strict retry.

人话版：锅糊两次就别继续装大厨。关火、写清现场、让能接手的人接。

## 11. Validation Rules

A Reflector response is PASS only if:

1. JSON parses successfully.
2. `schema_version` matches.
3. Success/failure is tied to evidence.
4. Failure includes root cause category and details.
5. `recommended_action` is exactly one primary action.
6. Stop-loss state is explicit.
7. The response does not expand scope or claim validation that did not run.

人话版：复盘能不能过，看它是否能指导下一步，而不是字数多不多。

---

## v0.2 Addendum: Plan Adjustment Routing

`plan_adjustment.recommended_action` must be consumed by `AgentLoop`, `PlanOptimizer`, or UI. If it is only logged, the reflection upgrade does not change behavior.

人话版：复盘官说“下次中火”，后厨没人听，那复盘就是念经。

### Routing Table

| Recommended action | Route target | Runtime side effect |
|---|---|---|
| `Continue` | Decide | Mark task complete or move to next pending task. |
| `RetryWithNewArgs` | Act | Modify tool parameters and retry once. |
| `UseAlternativeTool` | Act | Select fallback tool from manifest or Planner suggestions. |
| `RevisePlan` | Plan / PlanOptimizer | Generate a revised plan fragment and merge. |
| `AskUser` | UI / Governance | Pause loop and request one blocking decision. |
| `StopAndHandoff` | AgentLoop termination | Produce handoff summary and stop execution. |

人话版：不同复盘结论走不同窗口：继续、重试、换工具、重排计划、问老板、停火交接。

### RevisePlan Flow

1. Reflector returns:

```json
{
  "plan_adjustment": {
    "needed": true,
    "recommended_action": "RevisePlan",
    "target_step": "Plan",
    "rationale": "The current plan assumes the failing test is in module A, but evidence points to module B."
  }
}
```

2. `AgentLoop` sends current `Goal`, current `Plan`, and `ReflectorCritiqueV1` to `PlanOptimizer`.
3. `PlanOptimizer` returns a plan patch, not a full new universe.
4. `HierarchicalPlanner` validates the patch:
   - no scope expansion;
   - dependencies still valid;
   - suggested tools exist;
   - expected evidence remains explicit.
5. Increment `Plan.version`.
6. Continue at the revised pending task.

人话版：不是一复盘就推倒重来，而是像改菜单备注：这道菜别用牛肉，改用鸡肉，其他桌别乱动。

### StopAndHandoff Minimum Payload

When routing to `StopAndHandoff`, produce:

```json
{
  "handoff_summary": {
    "goal_id": "string",
    "current_task_id": "string|null",
    "failure_pattern": "string",
    "attempts": 2,
    "last_error_summary": "string",
    "artifacts_created": ["string"],
    "commands_or_tools_run": ["string"],
    "recommended_next_action": "string"
  }
}
```

人话版：停火不是跑路，要把锅在哪、火多大、糊成什么样、下一位先干嘛写清楚。

### Compatibility Note

If existing `Critique` cannot yet hold `plan_adjustment`, store the full `ReflectorCritiqueV1` JSON in trace or blackboard:

```text
__hajimi_reflector_critique_v1
__hajimi_plan_adjustment_action
__hajimi_stop_loss_state
```

人话版：旧表格放不下新复盘，就先把完整复盘贴黑板上，别丢信息。
