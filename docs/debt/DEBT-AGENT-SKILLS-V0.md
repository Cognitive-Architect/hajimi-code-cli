# DEBT-AGENT-SKILLS-V0

<!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

> Status: V0a partial / V0b not started / V0c not started
> Created: 2026-05-22
> Scope: Agent Skills V0 local Skill Pack integration
> Spec: `docs/agent-skills/SKILL-PACK-SPEC.md`
> Gate: `HAJIMI_AGENT_SKILLS_V0=true`

## Summary

Agent Skills V0 is initiated as a staged Intelligence-layer capability. Day 1 establishes only the documentation baseline, debt record, and default-off feature gate. No Registry, Router, Runtime, AgentLoop integration, `.hajimi/skills` scan, or Interface management is implemented in this step.

## Stage Status

| Stage | Current status | Planned closure evidence |
|---|---|---|
| V0a | Initiated / in progress | Skill Pack schema, Registry, Loader, Router, Planner injection, `auto-save` output evaluation, and default-off rollback evidence. |
| V0b | Planned / not started | Skill Runtime maps allowed tools and permissions into stricter Tool System and Governance constraints. |
| V0c | Planned / not started | Skill execution receipt enters Blackboard and Memory; Interface provides read-only list/validate views only. |

## Day 1 Boundary

Implemented in Day 1:

- `docs/agent-skills/SKILL-PACK-SPEC.md`
- this debt record
- documentation index references
- `is_agent_skills_v0_enabled()` in `src/intelligence/agent-core/prompts/mod.rs`

Not implemented in Day 1:

- `src/intelligence/agent-core/skills/`
- `SkillManifest`
- `SkillRegistry`
- `SkillLoader`
- `SkillRouter`
- `SkillRuntime`
- Blackboard keys
- AgentLoop routing or Planner injection
- `.hajimi/skills` runtime scanning
- test fixtures under `.hajimi/skills`
- Skill Store, URL install, auto-update, cloud sync, or marketplace features

## Feature Gate Contract

`HAJIMI_AGENT_SKILLS_V0` is default-off. The V0 path may run only when:

```text
HAJIMI_AGENT_SKILLS_V0=true
```

Unset, empty, `false`, `0`, `TRUE`, or any other value must preserve the old path and must not write Skill Blackboard keys.

## Runtime and Fixture Boundary

Runtime root:

```text
.hajimi/skills/<name>/
```

Test fixture root:

```text
tests/fixtures/skills/<name>/
```

Template root:

```text
templates/skills/<name>/
```

Tests must use `tests/fixtures/skills/`. Day 1 does not create runtime fixture data under `.hajimi/skills`.

## Open Debt

| ID | Status | Description |
|---|---|---|
| AGENT-SKILLS-V0A-001 | partial | Skill Pack spec, manifest types, validation logic, errors, auto-save fixture and template are completed. Registry, Loader, Router, Planner injection, and output evaluation are planned follow-up work. |
| AGENT-SKILLS-V0B-001 | not started | Runtime permissions and tool constraints are not wired. |
| AGENT-SKILLS-V0C-001 | not started | Skill execution receipts and Memory handoff are not wired. |
| AGENT-SKILLS-STORE-001 | out of scope | Store, marketplace, URL install, automatic update, and cloud sync are intentionally excluded from V0. |

## DAILY RECEIPT Template

```text
=== DAILY RECEIPT ===
Day:
Commit / SHA:
做了什么:
验证命令:
验证结果:
未完成 / 风险:
下一步:
=====================
```

## Day 1 Receipt

```text
=== DAILY RECEIPT ===
Day: B-01/13
Commit / SHA: base e8226da202dff8284af115e9b5d786a825b24053; local changes not committed in this task
做了什么: Added Agent Skills V0a documentation baseline, debt declaration, index references, and default-off feature gate.
验证命令: cargo fmt -- --check; cargo check -p intelligence-agent-core; node --check src/interface/web/app.js; cargo test -p intelligence-agent-core --lib; HAJIMI_AGENT_SKILLS_V0=false cargo test -p intelligence-agent-core --lib; cargo clippy -p intelligence-agent-core -- -D warnings; rg markers and boundary checks
验证结果: fmt/check/node/clippy passed; lib tests passed 222/222 in default env and 222/222 with HAJIMI_AGENT_SKILLS_V0=false; direct gate test covers unset, false, 0, TRUE, True, yes, and true.
未完成 / 风险: Registry, Router, Runtime, AgentLoop integration, Memory receipt, and Interface management remain planned follow-up work. Day 1 docs are explicitly unignored in .gitignore so they appear as normal untracked files before staging.
下一步: B-02/13 should add SkillManifest types and deterministic tests/fixtures/skills/auto-save fixture without touching .hajimi/skills.
=====================
```

## Day 2 Receipt

```text
=== DAILY RECEIPT ===
Day: B-02/13
Commit / SHA: feat(intelligence/agent-core): refactor skill manifest validations and add real fixture tests (SHA: 72a55e6)
做了什么: Defined SkillManifest, SkillPermissions, SkillRiskLevel, LoadedSkill and SkillError with full JSON deserialization. Created first test fixture auto-save in tests/fixtures/skills/auto-save and templates in templates/skills/auto-save. Refactored path and kebab-case validation helpers, added test_real_fixture_deserialization, test_invalid_exclusive_group, and test_path_with_drive_letter_denied unit tests.
验证命令: cargo fmt -- --check; cargo clippy -p intelligence-agent-core -- -D warnings; cargo test -p intelligence-agent-core --lib skills::types::tests; cargo check --workspace
验证结果: Formatting, clippy, and all 9 unit tests passed perfectly. Workspace compiles cleanly.
未完成 / 风险: Day 3 tasks (Registry, Loader scanning of workspace skills) are pending. Registry, Loader, and Router are still out of scope for Day 2.
下一步: Implement SkillRegistry and SkillLoader to scan and load local skill packs in the workspace.
=====================
```

## Closure Rules

The total V0 debt can close only after V0a, V0b, and V0c evidence is present. Intermediate updates must use segmented status such as `V0a partial`, `V0b not started`, and `V0c not started`.
