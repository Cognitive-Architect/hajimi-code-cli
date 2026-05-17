# Agent Persona Contract

> Scope: Day 11 Agent Prompt V2 contract documentation. This document describes the current Persona prompt contract for `agent-core`; it does not change runtime behavior.

## Source Evidence

| Item | Current source |
|---|---|
| Embedded persona text | `src/intelligence/agent-core/prompts/agent_persona.md` |
| Persona loader | `src/intelligence/agent-core/prompts/mod.rs` |
| Chat and stream injection | `src/intelligence/agent-core/llm/bridge.rs` |
| Debt mapping | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` section 8.1 |

## Status

The Persona is implemented as an embedded static system prompt. It is part of the current Agent Prompt baseline, but Prompt V2 is not declared fully productized by this document. `DEBT-AGENT-PROMPT-001` remains `PARTIAL/P2`.

## Feature-Gate

| Gate | Default | Disable value | Current behavior | Rollback |
|---|---:|---|---|---|
| `HAJIMI_PROMPT_PERSONA_ENABLED` | enabled | `false` or `0` | Includes `agent_persona.md` as system prompt when `llm/bridge.rs` builds chat or stream requests. | Set the environment variable to `false` or `0`; runtime falls back to the non-persona system prompt path. |

## Input Contract

The Persona receives no user data directly in `load_agent_persona()`. It is loaded from a static Markdown resource and then combined by the LLM bridge with runtime conversation context.

Required runtime context around the Persona:

| Field | Producer | Evidence expectation |
|---|---|---|
| User task or message | LLM bridge caller | User-visible goal or chat message |
| IDE state and history | Agent Core / bridge path | Current context, prior messages, retrieved context if available |
| Tool availability | Planner or Act prompt paths | Tool Manifest or explicit runtime tool context |
| Governance rules | Agent Core and tool execution | Approval level, risk boundary, sandbox or allow-list result |

## Output Contract

The Persona does not define a standalone JSON schema. It constrains all later Planner, Reflector, Executor, and chat outputs.

Required output qualities:

| Requirement | Meaning |
|---|---|
| Evidence-first | Completion claims must cite file paths, command output, trace records, test results, or explicit missing evidence. |
| Minimal change | The agent must preserve current architecture and avoid unrelated refactors. |
| Tool-aware | The agent must not claim or call unavailable tools. Unknown tools must be treated as unavailable. |
| Safe execution | Workspace boundaries, shell allow-list, governance, and destructive-action safeguards must be respected. |
| Stop-loss | Repeating the same failed pattern twice requires handoff, changed parameters, or a different plan. |
| Uncertainty marking | Unknown facts must be labeled as `UNKNOWN`, not guessed. |

## Failure And Fallback

| Failure | Required fallback |
|---|---|
| Persona gate disabled | Use the legacy non-persona system prompt path. Do not claim Persona policy is active. |
| Missing runtime evidence | Mark the claim as unverified and record the missing evidence. |
| Tool unavailable or absent from manifest | Do not invent the tool. Planner/Executor must choose a listed alternative or ask/stop. |
| Repeated execution failure | Apply stop-loss: revise plan, ask user, or hand off with evidence. |
| Safety conflict | Safety and governance win over task completion. |

## Evidence Fields

Persona-constrained responses should preserve these evidence fields when they are available:

| Evidence field | Examples |
|---|---|
| Source paths | `src/intelligence/agent-core/prompts/agent_persona.md`, changed files |
| Validation output | `cargo test`, `cargo check`, `node --check`, `rg` summaries |
| Trace or blackboard data | Agent trace IDs, checkpoint IDs, blackboard keys |
| Risk and approval state | Risk level, governance decision, confirmation state |
| Missing evidence | Environment failure, skipped WebView smoke, unavailable test runner |

## Security Boundary

The Persona is advisory text for model behavior, not a security control. Runtime security must still be enforced by code paths such as shell allow-listing, workspace path resolution, governance approval, and Tauri command boundaries.

Persona output must not:

| Prohibited claim or behavior | Reason |
|---|---|
| Fabricate command output or test results | Violates evidence-first contract |
| Treat prompt text as permission to bypass governance | Runtime approvals are authoritative |
| Claim hidden chain-of-thought as evidence | Only user-safe summaries and artifacts count |
| Declare future Prompt V2 work complete | Day 11 is documentation only |

## Day 12 Golden Regression Mapping

Day 12 golden cases should validate that persona-driven responses:

| Golden case | Expected check |
|---|---|
| Missing file or unknown repo fact | Response marks `UNKNOWN` or inspects context before acting |
| Tool absent from manifest | Planner/Executor refuses or selects a listed alternative |
| Repeated failing command | Stop-loss or changed parameters after repeated failure |
| Completed edit claim | Response includes file and validation evidence |
| Safety conflict | Response prioritizes governance and workspace safety |

## Future / Debt

| Debt | Statement |
|---|---|
| `DEBT-DOC-B11-001` | Persona has a stable text resource and feature-gate, but it is not yet backed by a complete golden regression suite. Day 12 should add golden cases without changing this Day 11 contract into a productization claim. |
