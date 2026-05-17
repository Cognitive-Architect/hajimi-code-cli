# Executor Contract

> Scope: Contract for Act/Executor prompt output, `ToolCallV1`, governance routing, blackboard protocol, and fallback behavior. This document records current code facts only.

## Source Evidence

| Item | Current source |
|---|---|
| Act ToolCall gate | `src/intelligence/agent-core/prompts/mod.rs` |
| ToolCall DTO | `src/intelligence/agent-core/act_dto.rs` |
| ActExecutor | `src/intelligence/agent-core/act_executor.rs` |
| Agent loop Act routing | `src/intelligence/agent-core/agent_loop.rs` |
| Governance API | `src/intelligence/agent-core/governance.rs` |
| Tool registry execution | `src/engine/tool-system/src/mod.rs` |

## Status

`ActExecutor` can execute `ToolCallV1` through the runtime `ToolRegistry`, route critical calls through governance, write blackboard state, and stop repeated identical failures. The `ActLlmBridge::llm_decide()` function is not fully implemented in current code; this document therefore treats LLM-driven Act decision generation as future/debt, while the executor chain protocol itself is documented as current.

## Feature-Gate

| Gate | Default | Disable value | Current behavior | Rollback |
|---|---:|---|---|---|
| `HAJIMI_ACT_TOOLCALL_V1_ENABLED` | enabled | `false` or `0` | Agent loop attempts ActExecutor chain execution when a serialized `ToolCallV1` is available on the blackboard. | Disable the gate to use the legacy swarm/local act path. |

Related gates:

| Gate | Relationship |
|---|---|
| `HAJIMI_PLANNER_V1_ENABLED` | Planner metadata can produce expected evidence, stop conditions, and suggested tools that inform Act. |
| `HAJIMI_REFLECTOR_V1_ENABLED` | Reflector V1 can recommend retry, alternative tools, plan revision, ask-user, or stop-loss after Act results. |

## Input Contract

Executor input is a `ToolCallV1` plus runtime context.

| Input | Required | Source |
|---|---:|---|
| `ToolCallV1` JSON | yes | Blackboard key `__hajimi_act_next_tool` or direct executor caller |
| Agent context | yes | `AgentContext` |
| Agent ID | yes | Blackboard writer identity |
| Tool registry | yes | `ToolRegistry` |
| Governance implementation | yes | `AgentGovernance` |

## ToolCallV1 Schema

| Field | Type | Required | Current code expectation |
|---|---|---:|---|
| `schema_version` | string | yes | Version string; current executor does not enforce a specific value. |
| `action_type` | enum | yes | Usually `CallTool`; enum also supports `CannotAct`, `AskUser`, `StopAndHandoff`. |
| `tool_name` | string | yes | Must exist in `ToolRegistry`. Unknown tool returns `Tool not found`. |
| `parameters` | JSON value | yes | Must be a JSON object or executor returns an error. |
| `reason` | string | yes | Human-readable rationale. |
| `expected_output` | string | yes | Expected tool output. |
| `expected_evidence` | string | yes | Evidence to inspect after execution. |
| `fallback_tool` | string or null | yes | Optional fallback tool written to blackboard after failure. |
| `governance_required` | boolean | yes | Routes through governance when true. |
| `risk_level` | enum | yes | `Critical` also routes through governance. |
| `idempotency_key` | string | yes | Used in fallback mutation; current duplicate guard fingerprints tool name and parameters. |
| `next_step_hint` | string or null | yes | Written to `__hajimi_act_next_tool` after success when present. |

## Blackboard Protocol

| Key | Meaning | Write behavior |
|---|---|---|
| `__hajimi_act_next_tool` | Serialized next `ToolCallV1` or next-step hint | Read before chain execution; written after success if `next_step_hint` exists; written after failure if `fallback_tool` exists. |
| `__hajimi_act_last_tool` | Last attempted tool name | Written before tool execution. |
| `__hajimi_act_last_tool_result` | Summary of successful tool output | Written on success. |
| `__hajimi_act_last_error` | Latest tool error and micro-reflect summary | Written on failure or stop guard. |
| `__hajimi_act_failed_tool_fingerprint` | Latest failed tool and parameter fingerprint | Written on failure and cleared on success. |
| `__hajimi_act_attempt_count` | Consecutive corrected-argument failure count | Incremented on failure and reset on success. |

## Governance Contract

Executor must ask governance before executing when:

| Condition | Runtime behavior |
|---|---|
| `governance_required=true` | Sends `GovernanceRequest` with action `invoke_tool:<tool_name>` and `ApprovalLevel::Critical`. |
| `risk_level=Critical` | Same as above even if `governance_required=false`. |

Governance results:

| Decision | Executor output |
|---|---|
| Approved | Execute tool. |
| Rejected | Return tool error with rejection reason. |
| Escalated | Return tool error with escalation level. |
| Timeout | Return governance timeout error. |
| Governance internal error | Return governance error. |

## Failure And Fallback

| Failure | Current fallback |
|---|---|
| Parameters are not a JSON object | Return `Tool parameters must be a valid JSON object`. |
| Tool missing from registry | Return `Tool not found: <tool_name>`. |
| Governance blocks execution | Return a tool error; do not execute. |
| Tool execution fails | Write micro-reflect error, failed fingerprint, and attempt count. |
| Same tool and parameters repeat | Stop and hand off before re-executing. |
| Corrected arguments fail twice | Return `StopAndHandoff`. |
| `fallback_tool` is present | Write fallback `ToolCallV1` to `__hajimi_act_next_tool` with modified idempotency key. |

## Output Contract

`execute_chain()` returns `ActChainResult`:

| Field | Type | Meaning |
|---|---|---|
| `success` | boolean | Whether the tool execution succeeded. |
| `output` | string | Summary of stdout, stderr, exit code, or failure reflection. |
| `decision` | `ActDecision` | `ToolCall`, `CannotAct`, `AskUser`, or `StopAndHandoff`. |

## Evidence Fields

Executor evidence should include:

| Evidence | Source |
|---|---|
| Tool name and parameters fingerprint | Executor fingerprint guard |
| stdout/stderr/exit code | `ToolOutput` summary |
| Governance decision | Governance result branch |
| Retry count | `__hajimi_act_attempt_count` |
| Last error | `__hajimi_act_last_error` |
| Fallback or next-step hint | `__hajimi_act_next_tool` |

## Day 12 Golden Regression Mapping

Day 12 should create Executor golden cases for:

| Case | Expected assertion |
|---|---|
| Safe read tool call | JSON parameters object accepted; success writes last tool/result. |
| Non-object parameters | rejected before tool execution. |
| Unknown tool | returns `Tool not found` and records failure. |
| Critical risk | governance path invoked before tool execution. |
| Repeated identical failure | second identical call stops and does not execute the tool again. |
| Fallback tool | fallback `ToolCallV1` written to blackboard. |

## Future / Debt

| Debt | Statement |
|---|---|
| `DEBT-DOC-B11-001` | `ActExecutor` chain exists, but `ActLlmBridge::llm_decide()` currently returns an explicit not-fully-implemented error. LLM generation of `ToolCallV1` must remain future/debt until implemented and tested. |
