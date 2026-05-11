# Source References — AGENT-PROMPT-CORE-001 v0.2

Generated: 2026-05-10

## Owner / Project Inputs

- `DEBT-AGENT-PROMPT-001.md` — original P0 debt source.
- `AGENT-PROMPT-CORE-001-GAP-ANALYSIS.md` — v0.2 gap analysis and improvement request.
- `02_PLAYBOOK.md` — project process: scope control, evidence-first, stop-loss.
- `03_ROLE_CARDS.md` — Mike / Emma / Bob / Alex role boundaries.
- `04_OWNER_TASK_TEMPLATE.md` — Owner decision and change rules.

## Public Source Repository

- Repository: `https://github.com/Cognitive-Architect/hajimi-code-cli`
- Observed ref: `v3.8.0-batch-1`

## Source Areas Consulted

- README / root repository page: architecture, feature list, tests, tool count.
- `AGENTS.md`: architecture and test guidance.
- `src/engine/llm-core/src/mod.rs`: `LlmClient`, `ChatMessage`, `stream_chat_with_context`, `count_tokens`, `heuristic_token_count`.
- `src/engine/tool-system/src/mod.rs`: `Tool`, permissions, errors, exports.
- `src/engine/tool-system/src/registry.rs`: `ToolRegistry`, list/get/register, 38+ registry test.
- `src/intelligence/agent-core/llm/bridge.rs`: existing short Planner and Reflector prompts using `stream_chat(prompt)`.
- `src/intelligence/agent-core/planner.rs`: current Goal/SubGoal/Task/ToolCall/TaskResult concepts.
- `src/intelligence/agent-core/reflector.rs`: current Critique and PlanOptimizer relationship.
- `src/intelligence/agent-core/agent_loop.rs`: 7-step loop and iteration/failure constants.

## Notes

This package references observed source shape for documentation design only. It does not claim that any runtime integration has been implemented.
