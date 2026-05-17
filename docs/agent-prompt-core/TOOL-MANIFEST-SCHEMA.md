# Tool Manifest Schema

> Scope: Contract for Tool Manifest V1 schema, generation inputs, risk fields, feature-gate relationships, and current limitations. This document reflects current code facts and does not modify tool runtime behavior.

## Source Evidence

| Item | Current source |
|---|---|
| Tool Manifest types | `src/intelligence/agent-core/tool_manifest.rs` |
| Planner V1 bridge use | `src/intelligence/agent-core/llm/bridge.rs` |
| Planner DTO suggested tools | `src/intelligence/agent-core/planner_dto.rs` |
| Executor DTO risk taxonomy | `src/intelligence/agent-core/act_dto.rs` |
| Tool runtime trait and permissions | `src/engine/tool-system/src/mod.rs` |

## Status

Tool Manifest V1 types exist and are used by Planner V1 prompt construction. Current generation is a minimal StepType-to-ToolCategory mapping that produces synthetic entries such as `tool-0-Search`; it is not yet a fully wired reflection of the live `ToolRegistry`. The document therefore treats registry-backed scoring, manual catalog loading, and omitted-tool trace metadata as future/debt.

## Feature-Gate

Tool Manifest does not have a standalone environment gate in current code. It is active through Planner V1 usage.

| Gate | Default | Disable value | Current behavior | Rollback |
|---|---:|---|---|---|
| `HAJIMI_PLANNER_V1_ENABLED` | enabled | `false` or `0` | Planner V1 builds a `ToolManifestRequest`, generates a Tool Manifest, injects it into the prompt, and filters suggested tools against generated names. | Disable Planner V1 to use legacy planner prompt without Tool Manifest injection. |

Related gates:

| Gate | Relationship |
|---|---|
| `HAJIMI_ACT_TOOLCALL_V1_ENABLED` | Executor consumes tool names and risk levels from `ToolCallV1`; manifest quality affects safe tool selection. |
| `HAJIMI_REFLECTOR_V1_ENABLED` | Reflector can recommend `UseAlternativeTool`, which should be grounded in manifest or blackboard suggested tools. |

## Input Schema

`ToolManifestRequest`:

| Field | Type | Required | Meaning |
|---|---|---:|---|
| `goal_description` | string | yes | Current goal used for intent classification. |
| `step_type` | enum | yes | One of `Observe`, `Retrieve`, `Plan`, `Act`, `Reflect`, `Store`, `Decide`. |
| `current_task` | string or null | yes | Active task description when available. |
| `recently_failed_tools` | array of string | yes | Tools to deprioritize to avoid retry loops. |
| `available_budget_tokens` | number | yes | Token budget for tool descriptions. |
| `max_tools` | number | yes | Maximum manifest entries; default intent is 15. |

## Output Schema

`ToolManifestEntryV1`:

| Field | Type | Required | Current code expectation |
|---|---|---:|---|
| `name` | string | yes | Tool name visible to Planner and Act. Current generator emits synthetic names. |
| `description` | string | yes | Compact tool description. |
| `category` | enum | yes | Functional category. |
| `available` | boolean | yes | Whether the tool is enabled. Current generator emits `true`. |
| `risk_level` | enum | yes | `Low`, `Medium`, `High`, or `Critical`. Current generator emits `Low`. |
| `requires_confirmation` | boolean | yes | Whether governance/user confirmation is required. Current generator emits `false`. |
| `parameters_schema` | JSON value | yes | JSON Schema for tool parameters. Current generator emits `{}`. |
| `when_to_use` | array of string | yes | Positive usage guidance. |
| `do_not_use_when` | array of string | yes | Negative usage guidance. |
| `recovery_hints` | array of string | yes | Known recovery steps. |
| `evidence_expected` | array of string | yes | Expected artifacts after success. |
| `known_failure_kinds` | array of string | yes | Known failure modes. |

## Enums

`ToolCategory` values:

`FileRead`, `FileWrite`, `Search`, `Git`, `Build`, `Test`, `Lsp`, `Mcp`, `Shell`, `Analysis`, `Docs`, `Network`, `Other`.

`RiskLevel` values:

`Low`, `Medium`, `High`, `Critical`.

`StepType` values:

`Observe`, `Retrieve`, `Plan`, `Act`, `Reflect`, `Store`, `Decide`.

## Current Generation Mapping

The current `generate()` implementation maps each step to three categories:

| StepType | Current categories |
|---|---|
| `Plan` | `Search`, `Analysis`, `FileRead` |
| `Act` | `FileWrite`, `Shell`, `Git` |
| `Observe` | `FileRead`, `Search`, `Lsp` |
| `Retrieve` | `Search`, `Lsp`, `Mcp` |
| `Reflect` | `Test`, `Analysis`, `FileRead` |
| `Store` | `Git`, `Docs`, `Network` |
| `Decide` | `FileRead`, `Analysis`, `Test` |

## Unknown Tool Filtering

Planner V1 bridge filters `suggested_tools` by the names present in the generated Tool Manifest. The contract is:

| Situation | Required behavior |
|---|---|
| Suggested tool exists in manifest | Preserve it in subgoal metadata. |
| Suggested tool is absent | Remove it before runtime metadata is stored. |
| A required capability is absent | Planner must use `stop_conditions`, ask-user routing, or a lower-risk available alternative. |

## Failure And Fallback

| Failure | Current fallback |
|---|---|
| Planner V1 disabled | No Tool Manifest injection; legacy planner path. |
| Live registry data unavailable | Current generator still emits StepType category entries. This is current behavior, not proof of live tool availability. |
| Unknown suggested tool | Filter out unknown tool. |
| Token budget too small | Future/debt: compaction policy is described in code comments but not fully implemented. |
| Tool recently failed | Future/debt: request field exists; current scoring does not yet use live failure scoring. |

## Evidence Fields

Tool Manifest entries must expose evidence expectations rather than just tool names.

| Evidence field | Meaning |
|---|---|
| `parameters_schema` | Validates whether a ToolCall can be formed safely. |
| `risk_level` | Informs governance and approval routing. |
| `requires_confirmation` | Signals human or governance confirmation. |
| `evidence_expected` | Defines what successful execution should prove. |
| `known_failure_kinds` | Helps Reflector and Executor choose fallback or stop-loss. |

## Security Boundary

The Tool Manifest is advisory prompt input. Runtime enforcement remains in `ToolRegistry`, individual tool implementations, governance, shell allow-listing, and workspace path validation.

The manifest must not:

| Prohibited claim | Reason |
|---|---|
| Mark a tool available unless runtime evidence supports it | Current generator entries are minimal prompt hints, not live registry proof. |
| Lower `risk_level` to avoid governance | Runtime risk and governance still apply. |
| Invent parameter schemas for unimplemented tools | Would create unsafe or misleading ToolCall output. |
| Treat network or shell tools as low-risk without evidence | Security-sensitive tools need runtime permission checks. |

## Day 12 Golden Regression Mapping

Day 12 should create Tool Manifest golden cases for:

| Case | Expected assertion |
|---|---|
| Plan step manifest | Contains `Search`, `Analysis`, `FileRead` categories. |
| Act step manifest | Contains `FileWrite`, `Shell`, `Git` categories. |
| Unknown suggested tool | Planner metadata excludes the unknown name. |
| Risk propagation | Planner and Executor preserve `RiskLevel` values. |
| Current limitation | Tests distinguish synthetic category entries from live registry-backed availability. |

## Future / Debt

| Debt | Statement |
|---|---|
| `DEBT-DOC-B11-001` | Tool Manifest V1 schema exists, but live `ToolRegistry` scoring, manual catalog loading, token-budget compaction, and omitted-tool trace metadata remain future/debt. This document must not be read as proof that those pieces are already productized. |
