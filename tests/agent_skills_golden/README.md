# Hajimi Agent Skills Golden Tests

> **Directory**: `tests/agent_skills_golden`  
> **Status**: Active / B-07 Planner injection validated

This directory contains deterministic **Golden Fixtures** for the Hajimi IDE Agent Skills Router, Scoring Engine, and Planner ContextBlock injection.

## 🚀 Key Characteristics

1. **100% Local & Rule-Bound**
   - The Skill Router matching, trigger checking, and title/description scoring are executed entirely on the local CPU in `< 1ms`.
   - **No external network access is requested or made.**
   - **No active LLM inference (Claude / OpenAI / Ollama) is called.**

2. **Strict Determinism**
   - All routing decisions and matches are 100% deterministic.
   - Any modifications to the scoring weights (+0.60 trigger, +0.25 name/title, +0.15 description) or the conflict resolution engine will be caught immediately by these golden test cases.
   - Planner injection fixtures assert only assembled context structure and message content; no live LLM provider is contacted.

3. **Compilation-Linked Fixtures**
   - All golden JSON test cases are loaded into the Rust test suite using `include_str!`.
   - This prevents any silent test skipping due to missing files. If a fixture is renamed or deleted, the project will fail to compile.
   - `injection/` fixtures are compiled by the `planner_skill_injection` bridge tests.

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

## Planner Injection Fixture Contract

Each injection fixture contains:

- `skill_instructions`: the Blackboard value to emulate from `BB_SKILL_INSTRUCTIONS`, or `null` for the no-skill path.
- `expected.message_count`: final chat message count after ContextWindow assembly.
- `expected.included_blocks`: expected block names, priorities, and content markers.
- `expected.absent_block_names`: strings that must not appear in the no-skill path.

The V0a Planner injection path is intentionally limited to `HAJIMI_CONTEXT_WINDOW_ENABLED=true`. If the ContextWindowManager gate is disabled, Planner falls back to the legacy path and does not inject `active_skill_instructions`; this is recorded as Day7 debt rather than hidden behavior.
