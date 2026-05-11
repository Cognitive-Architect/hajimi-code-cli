# PROMPT_VALIDATION_CHECKLIST.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 revised / docs-only deliverable  
> Scope: Provides acceptance checks for the prompt framework and later runtime integration.  
> Non-scope: Does not execute tests by itself and does not claim runtime integration is complete.

## 1. Purpose

This checklist turns prompt quality into verifiable acceptance criteria. It prevents the team from accepting vague claims such as “the Agent feels smarter” without artifacts, schema checks, behavior tests, and receipts.

人话版：这是验菜表。不能只说“闻起来不错”，要看菜端没端上来、熟没熟、小票有没有。

## 2. Document Existence Checks

Run from the repository root after copying the docs package into the repo:

```bash
test -f docs/agent-prompt-core/AGENT-PERSONA.md
test -f docs/agent-prompt-core/PLANNER_PROMPT_SPEC.md
test -f docs/agent-prompt-core/REFLECTOR_PROMPT_SPEC.md
test -f docs/agent-prompt-core/CONTEXT_WINDOW_POLICY.md
test -f docs/agent-prompt-core/PROMPT_VALIDATION_CHECKLIST.md
```

PASS if all commands exit 0.

人话版：先确认五份文件真的在，不要还没上菜就开始点评口味。

## 3. Prompt Audit Checklist

| Check | PASS condition | Result |
|---|---|---|
| Persona defines Agent role | Names Hajimi Agent Core as local-first IDE software agent | UNKNOWN |
| Persona defines safety rules | Mentions workspace, shell allow-list, destructive action safeguards | UNKNOWN |
| Persona defines evidence-first completion | Completion requires artifacts / validation / receipts | UNKNOWN |
| Planner has exact JSON schema | `PlannerSubgoalPlanV1` is present and parseable as intended | UNKNOWN |
| Planner includes tool manifest rules | Suggested tools must exist in manifest | UNKNOWN |
| Planner includes dependency rules | `depends_on` references `id_hint` values | UNKNOWN |
| Reflector has exact JSON schema | `ReflectorCritiqueV1` is present and parseable as intended | UNKNOWN |
| Reflector includes root cause | `root_cause.category`, `summary`, `details` are required | UNKNOWN |
| Reflector includes stop-loss | `stop_loss` object is required | UNKNOWN |
| Context policy has priority rules | P0/P1/P2/P3/P4 blocks or equivalent are defined | UNKNOWN |
| Context policy avoids all-tool dumps | Tool selection strategy is explicit | UNKNOWN |

人话版：这张表是文档质检，不看情绪价值，看格子有没有填齐。

## 4. Schema Validation Checklist

For Planner examples:

- [ ] JSON is valid.
- [ ] `schema_version == "PlannerSubgoalPlanV1"`.
- [ ] `goal_id` exists.
- [ ] `subgoals` is non-empty for executable goals.
- [ ] Every `depends_on` points to a known `id_hint`.
- [ ] Every `suggested_tools` item exists in the provided manifest.
- [ ] Every subgoal has expected evidence.
- [ ] No subgoal expands scope.

For Reflector examples:

- [ ] JSON is valid.
- [ ] `schema_version == "ReflectorCritiqueV1"`.
- [ ] `success` is backed by evidence.
- [ ] Failure has root cause category.
- [ ] `recommended_action` is exactly one primary action.
- [ ] Stop-loss state is explicit.
- [ ] Missing validation is reported, not hidden.

人话版：JSON 就像订单二维码，扫不出来就不能进后厨。

## 5. Behavior Test Scenarios

### Scenario A: Bug Fix Planning

Input goal:

```text
Fix a failing unit test in intelligence-agent-core and verify the fix.
```

Expected Planner behavior:

- Plan starts with inspect/retrieve steps.
- Suggested tools include search/read/test categories if available.
- Edit step depends on diagnosis step.
- Validation step expects focused test output.
- High-risk/destructive Git actions are not suggested.

PASS if the plan contains an inspect → diagnose → edit → validate chain with evidence per step.

人话版：修 bug 不能上来就改，得先复现、再定位、再修、再测。

### Scenario B: Refactor Planning

Input goal:

```text
Refactor a utility module while preserving public behavior.
```

Expected Planner behavior:

- Plan includes dependency analysis.
- Plan includes tests before/after or baseline validation.
- Plan marks broader refactor risk higher than documentation edits.
- Plan does not expand into unrelated modules.

PASS if refactor scope is controlled and validation is explicit.

人话版：重构像翻新厨房，不是把整栋楼拆了重新盖。

### Scenario C: Tool Failure Reflection

Input result:

```json
{
  "success": false,
  "output": "command not allowed: rm",
  "tool_name": "shell"
}
```

Expected Reflector behavior:

- `success` is false.
- Root cause category is `Permission` or `ToolFailure`.
- Recommended action is not repeating the same command.
- Suggests safe alternative or asks user if blocked.
- Stop-loss triggers if repeated.

PASS if the Reflector avoids retrying the same unsafe action.

人话版：系统说这把刀不能用，就别再拿同一把刀往案板上砍。

### Scenario D: Invalid JSON Reflection

Input result:

```text
Planner returned prose before JSON and serde_json parsing failed.
```

Expected Reflector behavior:

- Root cause category is `ParseFailure`.
- Suggests one strict retry with schema reminder.
- If repeated, suggests fallback and degraded-quality marking.

PASS if format failure leads to structured recovery, not blind repetition.

人话版：点餐码扫不出来，先让顾客重新下单一次；还不行就手写单并贴异常标签。

## 6. Runtime Integration Checks for Later Alex Phase

Only run these after source code changes are explicitly approved:

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
cargo test -p engine-tool-system -- test_allow_list
```

Expected results:

- `cargo check -p intelligence-agent-core` passes.
- Agent Core unit tests pass.
- Shell allow-list tests pass if shell/tool policy code is touched.

人话版：文档阶段不用硬跑发动机测试。真改发动机时，再跑编译和单测。

## 7. Receipts Checklist

Every delivery must record:

- [ ] Artifact paths.
- [ ] SHA256 or equivalent file fingerprints.
- [ ] Source repository URL and observed branch/tag.
- [ ] Owner decision log.
- [ ] Known non-scope items.
- [ ] Validation commands run or explicitly not run.
- [ ] PASS / FAIL / UNKNOWN status.

人话版：外卖袋里要有菜、小票、订单号、骑手记录。没有小票别说送达。

## 8. Final Acceptance Rubric

| Area | PASS | FAIL | UNKNOWN |
|---|---|---|---|
| Artifacts | All 5 docs exist | Missing required doc | Cannot access files |
| Persona | Role, safety, workflow, completion covered | Generic chatbot prompt only | Ambiguous role or boundaries |
| Planner | Schema and validation clear | No tool/dependency/evidence model | Schema cannot be assessed |
| Reflector | Root cause and plan adjustment clear | Only success/failure | Evidence missing |
| Context | Budget, memory tiers, truncation clear | Full dump / no policy | Token model unclear |
| Receipts | Paths and fingerprints recorded | No evidence | Partial evidence only |

人话版：最终判卷不是“看着挺努力”，而是每一栏能不能打 PASS。

## 9. Current v0.1 Expected Status

For this docs-only delivery, expected status is:

```text
Artifacts: PASS if all five Markdown files exist.
Runtime integration: UNKNOWN because code changes are explicitly out of scope.
Compilation/tests: NOT RUN unless Owner separately approves source-code implementation.
Overall docs package: PASS if receipts are complete.
```

人话版：这轮交付的是菜谱包，不是已经把餐厅系统接上线。别拿上线标准误杀文档阶段，也别把文档阶段吹成上线完成。

---

## v0.2 Addendum: Expanded Document Checks

Run from the repository root after copying the v0.2 docs package:

```bash
test -f docs/agent-prompt-core/AGENT-PERSONA.md
test -f docs/agent-prompt-core/PLANNER_PROMPT_SPEC.md
test -f docs/agent-prompt-core/REFLECTOR_PROMPT_SPEC.md
test -f docs/agent-prompt-core/CONTEXT_WINDOW_POLICY.md
test -f docs/agent-prompt-core/TOOL_MANIFEST_SPEC.md
test -f docs/agent-prompt-core/ACT_PROMPT_SPEC.md
test -f docs/agent-prompt-core/INTEGRATION_ROADMAP.md
test -f docs/agent-prompt-core/PROMPT_VALIDATION_CHECKLIST.md
```

PASS if all commands exit 0.

人话版：v0.2 不再是五件套，而是八件套。少一个都别说“最终交付”。

## v0.2 Addendum: ToolManifest Checks

- [ ] `ToolManifestEntryV1` schema is present.
- [ ] Runtime-to-manifest mapping is defined for `name`, `description`, `permissions`, and `is_enabled`.
- [ ] Manual catalog fields are identified as required enrichment, not hallucinated runtime facts.
- [ ] Filtering algorithm caps injected tools to <= 15 by default.
- [ ] Failure-aware filtering prevents same tool + same args retry loops.
- [ ] Risk and governance rules are explicit.

人话版：工具清单要能从真实仓库来，不能靠模型脑补一把“万能扳手”。

## v0.2 Addendum: Act Prompt Checks

- [ ] `ToolCallV1` schema is present.
- [ ] Act prompt selects exactly one next tool call.
- [ ] `CannotAct`, `AskUser`, and `StopAndHandoff` are defined.
- [ ] Multi-step chain protocol uses blackboard keys.
- [ ] Immediate retry rules prevent unchanged repeated failures.
- [ ] Governance routing is required for High/Critical actions.

人话版：Act 验收就看手会不会拿正确工具、一次拿一把、危险先问、摔了不复读。

## v0.2 Addendum: Roadmap Checks

- [ ] Integration phases are ordered and independently testable.
- [ ] Each phase lists target files.
- [ ] Each phase lists validation commands.
- [ ] Each phase has rollback or feature-gate guidance.
- [ ] Roadmap does not claim runtime integration has already been completed.

人话版：路线图要能给工程队开工，不是“后续优化”四个大字糊墙。
