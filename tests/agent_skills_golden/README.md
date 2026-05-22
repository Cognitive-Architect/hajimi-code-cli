# Hajimi Agent Skills Golden Tests

> **Directory**: `tests/agent_skills_golden`  
> **Status**: Active / B-05 Validated  

This directory contains the deterministic, rule-based **Golden Fixtures** for the Hajimi IDE Agent Skills Router and Scoring Engine.

## 🚀 Key Characteristics

1. **100% Local & Rule-Bound**
   - The Skill Router matching, trigger checking, and title/description scoring are executed entirely on the local CPU in `< 1ms`.
   - **No external network access is requested or made.**
   - **No active LLM inference (Claude / OpenAI / Ollama) is called.**

2. **Strict Determinism**
   - All routing decisions and matches are 100% deterministic.
   - Any modifications to the scoring weights (+0.60 trigger, +0.25 name/title, +0.15 description) or the conflict resolution engine will be caught immediately by these golden test cases.

3. **Compilation-Linked Fixtures**
   - All golden JSON test cases are loaded into the Rust test suite using `include_str!`.
   - This prevents any silent test skipping due to missing files. If a fixture is renamed or deleted, the project will fail to compile.

## 📁 Directory Structure

- `README.md` — This file.
- `router/` — Golden cases for the Skill Router:
  - `auto_save_trigger.json` — Evaluates a high-scoring input triggering the `auto-save` skill.
  - `no_skill_needed.json` — Evaluates a neutral conversational input where no skills are matched.
  - `below_threshold.json` — Evaluates a weak match (description overlap only) that scores below the 0.55 threshold.
  - `max_active_zero.json` — Evaluates the Top-K limit behavior by setting `max_active_skills` to `0` and asserting the correct rejection reason.
