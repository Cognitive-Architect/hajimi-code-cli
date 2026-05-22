# Hajimi Agent Skills Golden Tests

> **Directory**: `tests/agent_skills_golden`  
> **Status**: Active / B-07 Planner injection validated

This directory contains deterministic **Golden Fixtures** for the Hajimi IDE Agent Skills Router, Scoring Engine, Planner ContextBlock injection, Reflector skill eval criteria, and constrained runtime tool constraints.

## 🚀 Key Characteristics

1. **100% Local & Rule-Bound**
   - The Skill Router matching, trigger checking, and title/description scoring are executed entirely on the local CPU in `< 1ms`.
   - **No external network access is requested or made.**
   - **No active LLM inference (Claude / OpenAI / Ollama) is called.**

2. **Strict Determinism**
   - All routing decisions and matches are 100% deterministic.
   - Any modifications to the scoring weights (+0.60 trigger, +0.25 name/title, +0.15 description) or the conflict resolution engine will be caught immediately by these golden test cases.
   - Planner injection fixtures assert only assembled context structure and message content; no live LLM provider is contacted.
   - Reflector criteria fixtures assert lightweight `must_include` / `must_not_include` checks and deterministic failure reasons.

3. **Compilation-Linked Fixtures**
   - All golden JSON test cases are loaded into the Rust test suite using `include_str!`.
   - This prevents any silent test skipping due to missing files. If a fixture is renamed or deleted, the project will fail to compile.
   - `injection/` fixtures are compiled by the `planner_skill_injection` bridge tests.
   - `failure/` and `reflector/` fixtures are compiled by the `reflector_skill_eval` and `agent_skills_golden` tests.
   - `runtime/` fixtures are compiled by `agent_skills_golden` and assert tool constraints without executing tools.

## 📁 Directory Structure

- `README.md` — This file.
- `router/` — Golden cases for the Skill Router:
  - `auto_save_trigger.json` — Evaluates a high-scoring input triggering the `auto-save` skill.
  - `no_skill_needed.json` — Evaluates a neutral conversational input where no skills are matched.
  - `below_threshold.json` — Evaluates a weak match (description overlap only) that scores below the 0.55 threshold.
  - `max_active_zero.json` — Evaluates the Top-K limit behavior by setting `max_active_skills` to `0` and asserting the correct rejection reason.
- `injection/` — Golden cases for Planner ContextBlock injection:
  - `planner_auto_save_injected.json` — Asserts active skill instructions are assembled as `active_skill_instructions`, priority `P1`, `truncatable=true`.
  - `planner_no_skill_no_injection.json` — Asserts the no-skill path keeps only the P0 `system_prompt` and `user_prompt` blocks.
- `reflector/` — Golden cases for lightweight Reflector criteria:
  - `skill_eval_criteria.json` — Defines the auto-save `must_include`, `must_not_include`, and optional `expected_structure` checks.
- `failure/` — Negative output cases:
  - `auto_save_missing_block.json` — Proves an output without `=== AUTO SAVE` fails with a readable reason.
- `runtime/` — Golden cases for constrained runtime tool constraints:
  - `write_requires_approval.json` — Proves write tools become `Ask` constraints with Governance approval.
  - `shell_denied_by_default.json` — Proves shell tools are denied when `run_shell=false`.

## Planner Injection Fixture Contract

Each injection fixture contains:

- `skill_instructions`: the Blackboard value to emulate from `BB_SKILL_INSTRUCTIONS`, or `null` for the no-skill path.
- `expected.message_count`: final chat message count after ContextWindow assembly.
- `expected.included_blocks`: expected block names, priorities, and content markers.
- `expected.absent_block_names`: strings that must not appear in the no-skill path.

The V0a Planner injection path is intentionally limited to `HAJIMI_CONTEXT_WINDOW_ENABLED=true`. If the ContextWindowManager gate is disabled, Planner falls back to the legacy path and does not inject `active_skill_instructions`; this is recorded as Day7 debt rather than hidden behavior.

## Reflector Eval Fixture Contract

Skill output fixtures under `evals/output_cases.json` and reflector golden fixtures use:

- `must_include`: markers that must be present in the final output.
- `must_not_include`: forbidden markers such as placeholder text.
- `expected_structure`: optional headings or structural markers expected in the output.
- `failure_reason`: optional human-readable guidance for a missing or forbidden marker.

For `auto-save`, the required markers are `=== AUTO SAVE`, `做了什么`, `当前状态`, `下一步`, and `风险`.

## Runtime Fixture Contract

Runtime fixtures define:

- `available_tools`: the known tool names used to construct `SkillRuntime`.
- `manifest`: a complete Skill manifest with `allowed_tools` and `permissions`.
- `expected_allowed`: constraints that must be allowed after `allowed_tools` and `permissions` are intersected.
- `expected_denied`: constraints that must be denied with a readable reason.

These tests do not execute tools, shell commands, scripts, network requests, or file deletion. They only validate `SkillToolConstraints` reports.
