# Reflector Contract

> Scope: Contract for Reflector V1 prompt behavior and `ReflectorCritiqueV1Dto`. This document records current behavior and expected evidence fields for Day 12 regression.

## Source Evidence

| Item | Current source |
|---|---|
| Reflector V1 gate | `src/intelligence/agent-core/prompts/mod.rs` |
| Reflector DTO schema | `src/intelligence/agent-core/reflector_dto.rs` |
| Reflector LLM bridge prompt and fallback | `src/intelligence/agent-core/llm/bridge.rs` |
| Runtime critique type | `src/intelligence/agent-core/reflector.rs` |
| Agent loop plan-adjustment routing | `src/intelligence/agent-core/agent_loop.rs` |

## Status

Reflector V1 is feature-gated and defaults to enabled. It expects `ReflectorCritiqueV1` JSON, maps it to the legacy runtime `Critique`, and preserves fallback to legacy `Critique` JSON parsing when V1 parsing fails.

## Feature-Gate

| Gate | Default | Disable value | Current behavior | Rollback |
|---|---:|---|---|---|
| `HAJIMI_REFLECTOR_V1_ENABLED` | enabled | `false` or `0` | Parses `ReflectorCritiqueV1Dto` and maps structured root cause, plan adjustment, risks, and stop-loss into runtime critique fields. | Disable the gate to use legacy `Critique` JSON parsing. |

## Input Contract

Reflector input is the execution result plus current goal or subgoal context.

| Input | Required | Evidence expectation |
|---|---:|---|
| Execution result | yes | Tool output, error, task result, or validation summary |
| Expected evidence | when available | Planner `expected_evidence` or task metadata |
| Prior failures | when available | Blackboard errors, retry count, failed tool fingerprint |
| Risk context | when available | Planner risk level, governance result, tool risk |
| Plan context | when available | Current subgoal and dependency status |

## Output Schema

The LLM must return only valid JSON matching `ReflectorCritiqueV1Dto` when V1 is enabled.

| Field | Type | Required | Current code expectation |
|---|---|---:|---|
| `schema_version` | string | yes | Must be `ReflectorCritiqueV1`. |
| `success` | boolean | yes | Overall success judgment. |
| `severity` | enum | yes | Runtime `CritiqueSeverity`. |
| `confidence` | number | yes | 0.0 to 1.0 confidence score. |
| `evidence` | array of string | yes | Observable proof used for judgment. |
| `root_cause` | object | yes | `RootCauseDto`. |
| `issues` | array of string | yes | Concrete issues found. |
| `new_risks` | array of string | yes | Added into runtime issues as risk lines. |
| `suggestions` | array of string | yes | Actionable next steps. |
| `plan_adjustment` | object or null | yes | Optional `PlanAdjustmentDto`. |
| `stop_loss` | object or null | yes | Optional `StopLossDto`. |

`RootCauseDto`:

| Field | Type | Allowed values |
|---|---|---|
| `category` | enum | `None`, `ToolFailure`, `BadPlan`, `MissingContext`, `Permission`, `ValidationFailure`, `ParseFailure`, `UserInputNeeded`, `Unknown` |
| `description` | string | Human-readable root cause |
| `confidence` | number | 0.0 to 1.0 |

`RecommendedAction` values:

`Continue`, `RetryWithNewArgs`, `UseAlternativeTool`, `RevisePlan`, `AskUser`, `StopAndHandoff`.

## Failure And Fallback

| Failure | Current fallback |
|---|---|
| `HAJIMI_REFLECTOR_V1_ENABLED=false` | Legacy `Critique` JSON parsing path is used. |
| V1 JSON parse failure | Bridge logs a V1 parse warning and falls back to legacy `Critique` JSON parsing. |
| Missing evidence | Reflector must report low confidence or `MissingContext`; it must not fabricate evidence. |
| Repeated identical failure | Reflector should recommend `StopAndHandoff` or changed arguments; ActExecutor also has retry guards. |
| Unknown root cause | Use `RootCauseCategory::Unknown` with a concrete description. |

## Stop-Loss Contract

Stop-loss is a safety and quality boundary, not a styling field.

| Trigger | Required output |
|---|---|
| Same tool failure repeats with no new evidence | `stop_loss.triggered=true`, action `StopAndHandoff` or `RetryWithNewArgs` only if parameters change |
| Permission or governance rejection | root cause `Permission`, action `AskUser` or `StopAndHandoff` |
| Validation proves the plan is wrong | root cause `BadPlan` or `ValidationFailure`, action `RevisePlan` |
| User input is required | root cause `UserInputNeeded`, action `AskUser` |

Current DTO mapping appends `STOP-LOSS: <reason>` into runtime `Critique.issues` when triggered.

## Evidence Fields

Reflector evidence must be observable:

| Evidence type | Examples |
|---|---|
| Tool output | stdout, stderr, exit code summary |
| Test or build result | command, pass/fail, error line |
| File evidence | changed path, diff summary, hash, checkpoint ID |
| Governance evidence | approved, rejected, escalated, timeout |
| Missing evidence | unavailable command, environment block, parse error |

## Day 12 Golden Regression Mapping

Day 12 should create Reflector golden cases for:

| Case | Expected assertion |
|---|---|
| Successful validation | `success=true`, root cause `None`, action `Continue` |
| Tool failure | root cause `ToolFailure`, evidence includes error |
| Bad plan | action `RevisePlan`, plan adjustment reason present |
| Missing context | action `AskUser` or `StopAndHandoff`, no fake evidence |
| Repeated failure | `stop_loss.triggered=true` and stop reason present |

## Future / Debt

| Debt | Statement |
|---|---|
| `DEBT-DOC-B11-001` | Reflector V1 DTO and fallback exist, but Day 12 still needs golden regression coverage for parse failure, stop-loss, and plan-adjustment routing. |
