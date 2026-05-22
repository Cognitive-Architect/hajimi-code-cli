# Hajimi Agent Skills V0 Skill Pack Specification

<!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

> Status: V0a partial / V0b constrained runtime integrated
> Debt record: `docs/debt/DEBT-AGENT-SKILLS-V0.md`
> Feature gate: `HAJIMI_AGENT_SKILLS_V0=true`
> Runtime gate: `HAJIMI_AGENT_SKILL_RUNTIME=true`

## Purpose

Agent Skills V0 defines a local, reusable workflow layer for Hajimi Agent Core. A Skill describes when a reusable workflow should apply, what instructions should be loaded, and how the output can be checked. Skills do not replace Tools, MCP, Memory, or Governance.

V0 is split into three stages:

| Stage | Status | Scope |
|---|---|---|
| V0a | Initiated / planned | Local Skill Pack schema, Registry, Loader, Router, Planner injection, and `auto-save` output evaluation. |
| V0b | Integrated | Constrained runtime tool constraints, Blackboard handoff, and ActExecutor/Governance filtering. Runtime is not a script executor. |
| V0c | Planned | Skill execution receipts, Memory handoff, and read-only Interface management. |

## Directory Model

Runtime skills live in the workspace and are scanned only when the V0 gate is enabled:

```text
.hajimi/
└── skills/
    └── auto-save/
        ├── skill.json
        ├── SKILL.md
        ├── references/
        ├── scripts/
        └── evals/
```

Deterministic tests must not write to runtime `.hajimi/skills`. Test fixtures live under `tests/fixtures/skills/`:

```text
tests/
└── fixtures/
    └── skills/
        └── auto-save/
            ├── skill.json
            ├── SKILL.md
            └── evals/
```

Optional built-in examples may live under `templates/skills/`, but they are templates only. They are copied or activated only by an explicit future action and are not scanned as runtime state in V0a.

```text
templates/
└── skills/
    └── auto-save/
        ├── skill.json
        ├── SKILL.md
        └── evals/
```

## Skill Pack Files

| File or directory | V0 requirement |
|---|---|
| `skill.json` | Required manifest. Registry loads this metadata first. |
| `SKILL.md` | Required instruction entry loaded only after Router selects the Skill. |
| `references/` | Optional supporting material. V0a treats it as reference-only and loads it only when a later task explicitly implements selective reference loading. |
| `evals/` | Optional route or output evaluation cases. V0a uses this for deterministic checks. |
| `scripts/` | Reference-only in V0a. Scripts are never executed directly by the Skill layer. |

## Manifest V0 Shape

```json
{
  "schema_version": "hajimi.skill.v0",
  "version": "0.1.0",
  "name": "auto-save",
  "title": "自动存档",
  "description": "当任务推进、状态变化、出现风险或即将停止时，生成标准 AUTO SAVE 存档块。",
  "enabled": true,
  "category": "handoff",
  "exclusive_group": "handoff-output",
  "triggers": ["自动存档", "存档", "项目状态", "下一步", "handoff", "风险"],
  "risk_level": "low",
  "entry": "SKILL.md",
  "eval_entry": "evals/output_cases.json",
  "context_budget_tokens": 1200,
  "allowed_tools": [],
  "permissions": {
    "read_workspace": true,
    "write_workspace": false,
    "run_shell": false,
    "network": false,
    "delete": false
  }
}
```

## Loading Rules

1. `HAJIMI_AGENT_SKILLS_V0` must be exactly `true` before any V0a routing or injection path can run.
2. Registry keeps only `skill.json` metadata resident.
3. Loader reads `SKILL.md` only after Router selects a Skill.
4. `entry` and `eval_entry` must be relative paths inside the Skill directory.
5. `..` path segments and absolute paths are invalid.
6. One user turn may activate at most three Skills.
7. V0a must not scan `templates/skills/` as runtime state.

## Constrained Runtime V0b

`SkillRuntime` is a constraint builder, not a tool runner or script executor. It reads a selected Skill manifest and produces a deterministic `SkillToolConstraints` report. When both `HAJIMI_AGENT_SKILLS_V0=true` and `HAJIMI_AGENT_SKILL_RUNTIME=true`, AgentLoop writes those reports to Blackboard as `__hajimi_skill_tool_constraints`, and ActExecutor uses them to filter candidate ToolCallV1 actions before dispatch.

The Day9 runtime gate is default-off:

```text
HAJIMI_AGENT_SKILL_RUNTIME=true
```

Unset, empty, `false`, `TRUE`, or any other value keeps the constrained runtime path disabled. This is separate from the V0 routing gate so Skill instructions can remain active while tool constraints are rolled back.

### Permission Intersections

`allowed_tools` and `permissions` are intersected. A tool must be both listed in `allowed_tools` and permitted by the matching permission field before it can appear in the allowed constraint set. Unknown tools are filtered out and recorded as warnings in the report.

| Tool family | Required permission | Runtime decision |
|---|---|---|
| Read workspace tools | `read_workspace=true` | `Allow` with `ApprovalLevel::Auto` |
| Write workspace tools | `write_workspace=true` | `Ask` with `ApprovalLevel::Required` |
| Shell tools | `run_shell=true` | `Ask` with `ApprovalLevel::Required`; `run_shell=false` denies shell tools |
| Network tools | `network=true` | `Ask` with `ApprovalLevel::Required`; `network=false` denies network tools |
| Delete tools | `delete=true` | Still `Deny` by default in V0b with `ApprovalLevel::Critical` |

The constrained runtime must never:

- execute commands
- execute files from `scripts/`
- perform network requests
- delete files directly
- bypass ToolRegistry, ToolPermissions, or Governance
- create Interface commands

## Eval Fixture V0 Shape

When `skill.json` declares `eval_entry`, it points to a deterministic output evaluation fixture such as `evals/output_cases.json`. The fixture is read only after the Skill is selected, and only lightweight criteria are written to Blackboard as `__hajimi_skill_eval_criteria`.

```json
{
  "schema_version": "hajimi.skill.eval.v0",
  "skill_name": "auto-save",
  "criteria": {
    "skill_name": "auto-save",
    "must_include": ["=== AUTO SAVE", "做了什么", "当前状态", "下一步", "风险"],
    "must_not_include": ["TODO", "simulation", "mock"],
    "expected_structure": ["做了什么", "当前状态", "下一步", "风险"],
    "failure_reason": "missing required auto-save archive block"
  },
  "cases": [
    {
      "name": "missing_auto_save_block",
      "output": "已完成本轮修改，但没有附加存档块。",
      "expected_pass": false,
      "expected_failure_reason": "missing required marker: === AUTO SAVE"
    }
  ]
}
```

`must_include` and `must_not_include` are required arrays. `expected_structure` and `failure_reason` are optional. V0a criteria must not embed the full `SKILL.md`; Reflector receives only the concise acceptance markers.

## Safety Boundary

Agent Skills are workflow instructions, not executable capabilities. V0 explicitly does not provide:

- Skill Store or marketplace
- URL installation
- automatic update
- cloud sync
- Runtime direct shell execution
- Runtime direct network access
- direct file deletion
- direct bypass of ToolRegistry, ToolPermissions, or Governance
- full-context injection of every installed Skill

V0b translates Skill permissions into stricter constrained runtime reports, but actual tool execution must still go through the existing Tool System and Governance approval path.

## Initial Built-In Skill Target

`auto-save` is the first V0a validation Skill. Its expected behavior is to require a final archive block containing:

- `=== AUTO SAVE`
- `做了什么`
- `当前状态`
- `下一步`
- `风险`

`plain-language` and `project-handoff` are planned for later skeleton/template work and must not become default active runtime Skills in Day 1.

## Documentation Markers

All V0 documents and indexes use this marker while the work is active:

```text
AGENT-SKILLS-V0-2026-05-19
DEBT-AGENT-SKILLS-V0
SKILL-PACK-SPEC
```
