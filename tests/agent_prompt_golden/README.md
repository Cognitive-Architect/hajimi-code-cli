# Agent Prompt Golden Regression

This directory contains Day 12 golden cases for Agent Prompt V2 contracts. The cases are deterministic JSON fixtures and do not call a real LLM, external API, or network.

## Contract Mapping

| Area | Contract document | Runtime schema |
|---|---|---|
| Persona behavior boundaries | `docs/agent-prompt-core/AGENT-PERSONA.md` | Evidence-first and stop-loss assertions in case metadata |
| Planner | `docs/agent-prompt-core/PLANNER-PROMPT-CONTRACT.md` | `PlannerSubgoalPlanV1Dto` |
| Reflector | `docs/agent-prompt-core/REFLECTOR-CONTRACT.md` | `ReflectorCritiqueV1Dto` |
| Executor / ToolCall | `docs/agent-prompt-core/EXECUTOR-CONTRACT.md` | `ToolCallV1` and `ActDecision` |
| Tool Manifest | `docs/agent-prompt-core/TOOL-MANIFEST-SCHEMA.md` | `RiskLevel`, `suggested_tools`, and tool filtering expectations |

## Case Format

Every JSON case has:

| Field | Meaning |
|---|---|
| `id` | Stable case identifier used by test failures |
| `title` | Human-readable scenario name |
| `contract_mapping` | Day 11 contract paths and sections covered |
| `input` | Prompt-side input context, never a real LLM request |
| `expected` / `expected_tool_call` / `expected_decision` | DTO payload that must deserialize with current Rust schema |
| `expected_failure_reason` | Why the case would fail if the contract is violated |

## Cases

Planner:

- `planner/bug_fix_static_check.json`
- `planner/search_codebase.json`
- `planner/read_file_evidence.json`
- `planner/write_file_requires_approval.json`
- `planner/ask_user_missing_context.json`

Reflector:

- `reflector/success_continue.json`
- `reflector/failure_tool_retry.json`
- `reflector/unknown_missing_context.json`
- `reflector/retry_with_new_args.json`
- `reflector/stop_loss_repeated_failure.json`

ToolCall:

- `toolcall/safe_read.json`
- `toolcall/risky_write.json`
- `toolcall/cannot_act_missing_tool.json`

## Run / 验证

From the repository root:

```powershell
cargo test -p intelligence-agent-core --lib prompt_golden
cargo test -p intelligence-agent-core --lib
Get-ChildItem -Recurse tests/agent_prompt_golden
rg -n "contract|expected|schema_version|stop-loss|unknown|ask_user|safe_read|risky_write" tests/agent_prompt_golden
```

The Rust harness is in `src/intelligence/agent-core/prompt_golden_tests.rs`. It uses `include_str!` to read these fixtures at compile time, then checks schema deserialization and minimum contract fields. It intentionally does not weaken runtime DTOs and does not make network calls.

## Debt Boundary

`DEBT-TEST-B12-001` remains future work for a shared, data-driven harness that discovers all fixture files dynamically. The current harness is explicit on purpose: each fixture is named in Rust so a missing or renamed golden case fails compilation or test review instead of being silently skipped.
