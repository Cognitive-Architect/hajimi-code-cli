# Planner Prompt Contract

> Scope: Contract for Planner V1 prompt behavior and `PlannerSubgoalPlanV1Dto`. This document follows current code facts and does not modify Agent Core runtime logic.

## Source Evidence

| Item | Current source |
|---|---|
| Planner V1 gate | `src/intelligence/agent-core/prompts/mod.rs` |
| Planner DTO schema | `src/intelligence/agent-core/planner_dto.rs` |
| Planner LLM bridge | `src/intelligence/agent-core/llm/bridge.rs` |
| Runtime planner mapping | `src/intelligence/agent-core/planner.rs` and bridge adapter code |
| Tool Manifest request | `src/intelligence/agent-core/tool_manifest.rs` |

## Status

Planner V1 is currently feature-gated and defaults to enabled. It parses `PlannerSubgoalPlanV1Dto`, filters suggested tools against the generated manifest, and stores selected metadata on runtime subgoals. The current implementation still keeps a legacy planner fallback path.

## Feature-Gate

| Gate | Default | Disable value | Current behavior | Rollback |
|---|---:|---|---|---|
| `HAJIMI_PLANNER_V1_ENABLED` | enabled | `false` or `0` | Uses Tool Manifest injection and expects `PlannerSubgoalPlanV1` JSON from the LLM. | Disable the gate to use the legacy planner path. |

Related gates:

| Gate | Relationship |
|---|---|
| `HAJIMI_PROMPT_PERSONA_ENABLED` | Controls whether the Persona system prompt is included around LLM calls. |
| `HAJIMI_CONTEXT_WINDOW_ENABLED` | Controls whether context-window integration is used when building LLM context. |

## Input Contract

Planner V1 prompt input is built from the current goal, priority, and generated Tool Manifest.

| Input | Required | Source |
|---|---:|---|
| `goal_id` | yes | Runtime goal identifier |
| `goal_description` | yes | User or agent goal |
| `priority` | yes | Runtime planner priority |
| `Tool Manifest` | yes for V1 prompt | `ToolManifestGenerator::generate()` |
| Current context | when available | LLM bridge and context-window path |

The Planner must treat the manifest as authoritative. It may suggest only tool names present in the manifest. Unknown tools are filtered by the bridge and should be treated as unavailable by later stages.

## Output Schema

The LLM must return only valid JSON matching `PlannerSubgoalPlanV1Dto`.

| Field | Type | Required | Current code expectation |
|---|---|---:|---|
| `schema_version` | string | yes | Must be `PlannerSubgoalPlanV1`. |
| `goal_id` | string | yes | Must match the input goal. |
| `summary` | string | yes | One-sentence plan summary. |
| `subgoals` | array of `PlannerSubgoalDto` | yes | Non-empty unless the goal is blocked. |
| `global_risks` | array of string | yes | Risks applying to the whole plan. |
| `notes` | array of string | yes | Assumptions and scope notes. |

`PlannerSubgoalDto` fields:

| Field | Type | Required | Evidence or runtime mapping |
|---|---|---:|---|
| `id_hint` | string | yes | Local dependency key; mapped into runtime subgoal ID. |
| `description` | string | yes | Runtime task description. |
| `priority` | enum | yes | Uses runtime `Priority`. |
| `depends_on` | array of string | yes | Must reference existing `id_hint` values. |
| `suggested_tools` | array of string | yes | Filtered against Tool Manifest; unknown tools removed with warning behavior in bridge. |
| `expected_evidence` | array of string | yes | Stored in subgoal metadata by bridge. |
| `validation_intent` | enum | yes | `None`, `StaticCheck`, `UnitTest`, `IntegrationTest`, `ManualReview`, or `CommandOutput`. |
| `risk_level` | enum | yes | Uses `RiskLevel` from Tool Manifest taxonomy. |
| `requires_user_approval` | boolean | yes | Must be true for high-risk or scope-changing work. |
| `stop_conditions` | array of string | yes | Stored in subgoal metadata by bridge. |

## Failure And Fallback

| Failure | Current fallback |
|---|---|
| `HAJIMI_PLANNER_V1_ENABLED=false` | Legacy planner prompt path is used. |
| Invalid JSON or schema parse failure | Planner bridge returns an error or falls back according to existing bridge behavior; Day 11 does not change recovery logic. |
| Bad dependency reference | `validate_dependencies()` reports `unknown dependency`. |
| Unknown suggested tool | Bridge filters the unknown tool out of `suggested_tools`; the plan must not depend on it as evidence. |
| Insufficient context | Planner should express assumptions in `notes` and use `stop_conditions` or ask-user routing rather than inventing facts. |

## Stop Conditions

Planner V1 must include concrete `stop_conditions` when any of the following are true:

| Condition | Required planner output |
|---|---|
| Required file, branch, credential, or environment is missing | Add a stop condition and validation intent. |
| A write or destructive operation is likely | Mark risk and `requires_user_approval`. |
| Tool Manifest does not contain a necessary tool | Do not invent the tool; add a stop or ask-user condition. |
| Evidence cannot be collected automatically | Use `ManualReview` and expected evidence text. |

## Evidence Fields

Every subgoal should name observable evidence, not only a desired result.

Examples:

| Task type | Expected evidence |
|---|---|
| Code edit | Changed file path and diff summary |
| Test | Test command and pass/fail output |
| Search | Search query and matching paths |
| Git operation | `git status`, diff, commit SHA, or branch name |
| Blocked task | Missing input and user question |

## Day 12 Golden Regression Mapping

Day 12 should create Planner golden cases for:

| Case | Expected assertion |
|---|---|
| Simple read-only search | `validation_intent=None` or `CommandOutput`, low risk, search/read suggested tools only |
| Workspace edit | expected evidence includes changed path and validation command |
| Missing context | stop condition or ask-user plan, no fabricated path |
| Unknown tool in model output | unknown tool is filtered and not treated as available |
| Dependency graph | invalid `depends_on` is rejected by validation |

## Future / Debt

| Debt | Statement |
|---|---|
| `DEBT-DOC-B11-001` | Planner V1 schema exists, but complete runtime validation of `schema_version` and every suggested tool is not fully centralized in the DTO validator. Day 12+ can add golden tests and stricter validation without claiming this document changed runtime behavior. |
