# ACT_PROMPT_SPEC.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 gap-fill / docs-only deliverable  
> Scope: Defines the Act-step prompt contract, tool call schema, chained execution protocol, retry rules, and governance handoff points.  
> Non-scope: Does not implement `ActExecutor`, provider-native tool calling, or tool runtime changes.

## 1. Purpose

The Act step is where the Agent turns an approved task into exactly one concrete tool call. Without an Act prompt contract, the Agent can plan intelligently but still execute blindly.

人话版：Planner 是派工单，Act 是手。脑子会想但手不会拿工具，最后还是干不成活。

## 2. Design Rule: One Next Tool Call

Act MUST return exactly one next tool call per LLM decision.

Reasons:

1. Tool results are uncertain and may change the next step.
2. Governance may block high-risk actions.
3. Validation and reflection need clear evidence after each call.
4. Chained calls should be explicit, not hidden inside one giant plan.

人话版：不要一口气下十个命令。先拿锅，再开火，再下菜；每一步看一眼，别把厨房炸了才复盘。

## 3. Runtime Inputs

```json
{
  "agent_persona_ref": "AGENT-PERSONA.md",
  "current_goal": {
    "id": "string",
    "description": "string",
    "priority": "Critical|High|Medium|Low"
  },
  "approved_task": {
    "id": "string",
    "description": "string",
    "suggested_tools": ["string"],
    "expected_evidence": ["string"],
    "risk_level": "Low|Medium|High|Critical"
  },
  "current_step": {
    "type": "Act",
    "attempt": 1,
    "previous_tool_call": null,
    "previous_error": null
  },
  "tool_manifest": [
    {
      "schema_version": "ToolManifestEntryV1",
      "name": "string",
      "description": "string",
      "parameters_schema": {},
      "risk_level": "Low|Medium|High|Critical",
      "requires_confirmation": false,
      "when_to_use": ["string"],
      "do_not_use_when": ["string"],
      "recovery_hints": ["string"],
      "evidence_expected": ["string"]
    }
  ],
  "focus_memory": ["string"],
  "blackboard_state": {
    "__hajimi_next_tool": "string|null",
    "__hajimi_last_error": "string|null",
    "__hajimi_failed_tool_fingerprint": "string|null"
  },
  "output_contract": "ToolCallV1"
}
```

人话版：Act 前要给它：当前任务、可用工具、上次有没有翻车、黑板上有没有下一步提示。否则它会像蒙眼找锅铲。

## 4. Act Prompt Template

```text
You are executing the Act step of Hajimi Agent Core.

Use the stable Agent Persona rules already provided.

Current approved goal:
{current_goal_json}

Current approved task:
{approved_task_json}

Available tools for this Act step:
{tool_manifest_json}

Relevant focus memory:
{focus_memory_json}

Blackboard state:
{blackboard_state_json}

Instructions:
1. Select exactly one tool to call for the current task.
2. Use only a tool listed in tool_manifest.
3. Provide all required parameters with correct JSON types.
4. If the task requires multiple tools, return only the NEXT tool call.
5. Do not retry the same failed tool with identical parameters.
6. If no safe tool can be selected, return action_type = "CannotAct" with a reason.
7. For High/Critical risk calls, mark governance_required = true.
8. Return ONLY valid JSON matching ToolCallV1.
```

人话版：这段是执行前提示：只能选一把工具，参数要填对，翻车不能原样重试，高危操作要举手。

## 5. ToolCallV1 Schema

```json
{
  "schema_version": "ToolCallV1",
  "action_type": "CallTool|CannotAct|AskUser|StopAndHandoff",
  "tool_name": "string|null",
  "parameters": {},
  "reason": "string",
  "expected_output": "string",
  "expected_evidence": ["string"],
  "fallback_tool": "string|null",
  "governance_required": false,
  "risk_level": "Low|Medium|High|Critical",
  "idempotency_key": "string",
  "next_step_hint": "string|null"
}
```

Field rules:

- `schema_version` MUST equal `ToolCallV1`.
- `action_type` MUST be one of the enum values.
- If `action_type == CallTool`, `tool_name` MUST be non-null and exist in `tool_manifest`.
- `parameters` MUST satisfy the selected tool's `parameters_schema` when available.
- `idempotency_key` SHOULD be a stable hash of task id + tool name + parameters.
- `next_step_hint` MAY describe the likely next tool, but MUST NOT execute it.

人话版：这是工具调用订单。工具名、参数、为什么用、预期产出、高不高危，都要写清楚。

## 6. CannotAct / AskUser / StopAndHandoff

Use non-tool actions when safer than guessing:

| action_type | Use when | Required fields |
|---|---|---|
| `CannotAct` | Required tool is unavailable, params are impossible, or manifest is empty | `reason`, `expected_evidence=[]` |
| `AskUser` | Missing user decision blocks safe execution | `reason`, `next_step_hint` |
| `StopAndHandoff` | Stop-loss triggered or unsafe repeated failure | `reason`, `expected_evidence`, `next_step_hint` |

人话版：不会就说不会，该问老板就问老板，锅糊两次就停火交接。别硬装厨神。

## 7. Multi-Step Tool Chains

For multi-tool workflows, Act uses a chain protocol:

1. Return exactly one `ToolCallV1`.
2. Runtime executes the tool.
3. Runtime writes result summary to blackboard:
   - `__hajimi_last_tool`
   - `__hajimi_last_tool_result`
   - `__hajimi_last_error` when failed
4. If the next tool is obvious, Act may set `next_step_hint`.
5. Runtime may copy `next_step_hint` to `__hajimi_next_tool`.
6. Next Act call receives updated blackboard and chooses again.

Recommended blackboard keys:

```text
__hajimi_next_tool
__hajimi_last_tool
__hajimi_last_tool_result
__hajimi_last_error
__hajimi_failed_tool_fingerprint
__hajimi_attempt_count
__hajimi_governance_decision
```

人话版：多步任务像接力赛。每跑完一棒，把棒交清楚，下一棒再决定怎么跑。

## 8. Immediate Retry and Micro-Reflect

When a tool call fails:

1. Runtime records failure kind, error summary, and args fingerprint.
2. If same tool + same args already failed, do not retry unchanged.
3. Trigger micro-reflect with only current tool error context.
4. Micro-reflect returns one of:
   - fix parameters and retry;
   - use fallback tool;
   - ask user;
   - stop and handoff.
5. Same tool with corrected parameters may be retried once.
6. Same failure category twice should trigger `StopAndHandoff` or `AskUser`.

人话版：螺丝拧不进去，先看是不是型号错了；别拿同一把螺丝刀用同一个角度怼五遍。

## 9. Governance Integration

Before executing a `ToolCallV1`:

- If `governance_required == true`, route to governance before execution.
- If tool manifest marks `requires_confirmation == true`, route to governance even if the model forgot.
- If risk is `Critical`, default action is block unless pre-approved.
- If governance rejects, write decision to blackboard and route to Reflect.

人话版：高危工具要先问。模型说“我觉得能删”不算数，老板或治理层说了才算数。

## 10. Provider Compatibility

### 10.1 JSON-Only Fallback

All providers must support plain JSON output. This is the minimum compatible mode for local models and generic streaming clients.

### 10.2 OpenAI / Anthropic Tool Use

When provider-native function calling is available, runtime may convert `ToolManifestEntryV1.parameters_schema` into provider-specific tool definitions. The prompt contract still remains `ToolCallV1` for traceability.

### 10.3 Local Model Simplification

For weaker local models:

- reduce manifest to <= 8 tools;
- use smaller parameter schemas;
- include one valid example;
- enforce strict JSON repair and fallback.

人话版：高级模型可以直接用函数调用；小模型就少给点菜单，别把它喂撑。

## 11. Example: Bug Fix Chain

Goal: fix a failing unit test in `intelligence-agent-core`.

Act call sequence SHOULD look like:

1. `grep` / `find` to locate failing test or symbol.
2. `read_file` to inspect relevant file.
3. `lsp_definition` or `grep` for dependencies if needed.
4. `edit_file` or `apply_patch` for minimal change.
5. `cargo_test` / `cargo_build` for focused validation.
6. `git_diff` for evidence summary.

Each Act decision returns only one tool call, then reflects on the result.

人话版：修测试不是上来就改代码。先找病灶，再看片子，再下刀，最后复查。

## 12. Validation Checklist

- [ ] Act output is valid JSON.
- [ ] `schema_version == "ToolCallV1"`.
- [ ] Exactly one action is selected.
- [ ] `tool_name` exists in the manifest when action is `CallTool`.
- [ ] Parameters match tool schema or missing schema is marked `UNKNOWN`.
- [ ] Same failed tool+params are not repeated unchanged.
- [ ] High/Critical calls route to governance.
- [ ] Tool output evidence is captured after execution.
- [ ] Multi-tool task uses blackboard chain protocol.
- [ ] Stop-loss fires after repeated same-class failures.

人话版：验收看十件事：JSON 对、只选一个工具、参数对、危险操作审批、翻车不复读、证据有记录。

## 13. Out of Scope for This Spec

- Implementing runtime execution.
- Implementing provider-native function calls.
- Changing the existing `ToolCall` struct.
- Guaranteeing all tools have complete schemas in v0.2.

人话版：这份文件管“手该怎么下单”，不直接改机械臂。
