# INTEGRATION_ROADMAP.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 gap-fill / docs-only deliverable  
> Scope: Defines a phased path from prompt specifications to runtime integration in `hajimi-code-cli`.  
> Non-scope: Does not implement Rust changes, open a PR, or claim runtime integration is complete.

## 1. Purpose

This roadmap converts the docs package into an implementation sequence. Each phase has a narrow goal, target files, expected deliverables, validation commands, receipts, and rollback notes.

人话版：v0.1 是菜谱，v0.2 这份路线图是施工排期：先改哪、后改哪、每一步怎么验、翻车怎么退。

## 2. Baseline Assumptions

Observed repository baseline:

- Repository: `https://github.com/Cognitive-Architect/hajimi-code-cli`
- Observed ref: `v3.8.0-batch-1`
- Architecture: local-first Hajimi IDE with Rust backend and Intelligence / Engine layering.
- Existing Agent Core loop: Observe → Retrieve → Plan → Act → Reflect → Store → Decide.
- Existing LLM client trait already provides both `stream_chat` and `stream_chat_with_context`.
- Existing Planner / Reflector bridge still uses short prompt strings and backward-compatible `stream_chat(prompt)`.
- Existing tool system provides `Tool`, `ToolRegistry`, permission levels, 40+ tools, and tests.

人话版：仓库里已经有路、有车、有工具库；路线图不是从零造车，而是把驾驶规范接到方向盘上。

## 3. Release Strategy

Use five incremental phases. Each phase must be mergeable and testable independently.

| Phase | Name | Goal | Risk |
|---:|---|---|---|
| 0 | Docs landing | Copy v0.2 docs into repository | Low |
| 1 | System prompt injection | Use Agent Persona as runtime system prompt | Medium |
| 2 | Planner + ToolManifest | Make planning tool-aware | High |
| 3 | Reflector + stop-loss | Make reflection actionable and routable | High |
| 4 | ContextWindowManager | Add token-aware context assembly | High |
| 5 | ActExecutor | Add tool-call decision contract and chain protocol | Critical |

人话版：不要一口气改全车。先把说明书放进去，再接中控，再接方向盘，最后再接机械臂。

## 4. Phase 0 — Docs Landing

### Goal

Land prompt docs in the repository without runtime behavior changes.

### Target paths

```text
docs/agent-prompt-core/
├── AGENT-PERSONA.md
├── PLANNER_PROMPT_SPEC.md
├── REFLECTOR_PROMPT_SPEC.md
├── CONTEXT_WINDOW_POLICY.md
├── TOOL_MANIFEST_SPEC.md
├── ACT_PROMPT_SPEC.md
├── INTEGRATION_ROADMAP.md
└── PROMPT_VALIDATION_CHECKLIST.md
```

### Validation

```bash
test -f docs/agent-prompt-core/AGENT-PERSONA.md
test -f docs/agent-prompt-core/TOOL_MANIFEST_SPEC.md
test -f docs/agent-prompt-core/ACT_PROMPT_SPEC.md
test -f docs/agent-prompt-core/INTEGRATION_ROADMAP.md
```

### Receipts

- Commit containing docs only.
- `git diff --stat` showing only docs paths.

人话版：第一步只把施工图放进项目文件夹，不动机器。这个 PR 最安全。

## 5. Phase 1 — System Prompt Injection

### Goal

Inject the stable Agent Persona as a system prompt into Planner and Reflector LLM calls while keeping existing output structs unchanged.

### Target files

```text
src/engine/llm-core/src/mod.rs                 # Confirm `stream_chat_with_context` contract
src/intelligence/agent-core/llm/bridge.rs      # Use context-aware chat path
src/intelligence/codex-twist/src/thread.rs     # Replace generic default system prompt or route to persona
src/intelligence/agent-core/prompts/mod.rs     # Optional new PromptProvider
src/intelligence/agent-core/prompts/agent_persona.md
```

### Minimal implementation

1. Add `agent_persona.md` under agent-core prompt resources.
2. Add a `PromptProvider` that returns the persona string.
3. Change `PlannerLlmBridge::chat_and_collect` to call `stream_chat_with_context(messages, Some(system_prompt))`.
4. Change `ReflectorLlmBridge::chat_and_collect` the same way.
5. Keep existing JSON formats unchanged for this phase.

### Validation

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
```

Runtime audit:

- Log or trace confirms `system_prompt` is non-empty.
- Planner still parses the existing short JSON array.
- Reflector still parses existing `Critique`.

### Rollback

Feature-gate Persona injection behind config:

```text
agent_core.prompt_persona_enabled = true|false
```

人话版：先只给员工发手册，不改派工单格式。这样风险最低，出事也能一键关。

## 6. Phase 2 — Planner + ToolManifest

### Goal

Make Planner produce tool-aware subgoals using `PlannerSubgoalPlanV1` and a filtered `ToolManifest`.

### Target files

```text
src/intelligence/agent-core/planner.rs
src/intelligence/agent-core/llm/bridge.rs
src/intelligence/agent-core/prompts/planner_prompt.md
src/intelligence/agent-core/tool_manifest.rs        # New or under engine/tool-system if preferred
src/engine/tool-system/src/registry.rs              # Add describe/list metadata helper if needed
src/engine/tool-system/src/mod.rs                   # Expose types only if needed
```

### Minimal implementation

1. Implement `ToolManifestGenerator` read-only path.
2. Generate <= 15 relevant tools for Plan step.
3. Add DTOs for `PlannerSubgoalPlanV1` without immediately changing persistent Plan shape.
4. Map DTO → existing `SubGoal` with optional metadata fields if available.
5. Validate `suggested_tools` against `ToolRegistry`.
6. Keep rule-based fallback.

### Suggested compatibility path

If changing `SubGoal` is risky, store new fields in `Goal.metadata` / task metadata first, then promote to struct fields in a later PR.

### Validation

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
cargo test -p engine-tool-system -- test_registry_40_tools
```

Behavior checks:

- Planner output includes `suggested_tools`.
- Suggested tools exist in registry.
- No more than 15 tools injected.
- Planner fallback still works when LLM returns invalid JSON.

### Rollback

- Disable `planner_v1_schema_enabled`.
- Fall back to current `description + priority` DTO.

人话版：第二步让派工单知道工具箱，但别把数据库结构一把梭炸掉。先做兼容映射。

## 7. Phase 3 — Reflector + Stop-Loss Routing

### Goal

Make reflection include root cause, plan adjustment, risk, and stop-loss routing.

### Target files

```text
src/intelligence/agent-core/reflector.rs
src/intelligence/agent-core/llm/bridge.rs
src/intelligence/agent-core/plan_optimizer.rs
src/intelligence/agent-core/agent_loop.rs
src/intelligence/agent-core/prompts/reflector_prompt.md
```

### Minimal implementation

1. Add `ReflectorCritiqueV1Dto` for LLM parsing.
2. Map DTO into existing `Critique` for backward compatibility.
3. Add optional `root_cause`, `plan_adjustment`, and `stop_loss` fields if struct migration is approved.
4. Route `plan_adjustment.recommended_action`:
   - Continue → Decide
   - RetryWithNewArgs → Act
   - UseAlternativeTool → Act
   - RevisePlan → PlanOptimizer
   - AskUser → UI pause
   - StopAndHandoff → loop termination + handoff summary
5. Add repeated-failure counters in `AgentLoop` or blackboard.

### Validation

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
```

Behavior checks:

- Failed task yields `root_cause.category`.
- Same class failure twice triggers stop-loss.
- `RevisePlan` calls or schedules `PlanOptimizer`.
- Missing validation is reported as unknown, not hidden.

### Rollback

- Keep original `Critique` parser as fallback.
- Disable routing behind config:

```text
agent_core.reflector_v1_routing_enabled = false
```

人话版：第三步让复盘官不只说“失败”，还要说“哪里坏、下一步去哪、要不要停火”。

## 8. Phase 4 — ContextWindowManager

### Goal

Assemble LLM context with deterministic priority and token accounting.

### Target files

```text
src/intelligence/agent-core/context_window_manager.rs
src/intelligence/agent-core/memory_retriever.rs
src/intelligence/agent-core/agent_loop.rs
src/intelligence/agent-core/llm/bridge.rs
src/engine/llm-core/src/mod.rs
```

### Minimal implementation

1. Define `ContextBlock`, `ContextPriority`, and `TokenAccount`.
2. Use `LlmClient::count_tokens` where available.
3. Fallback to heuristic token count.
4. Inject P0 blocks first, P1 next, then P2/P3 by budget.
5. Record omitted blocks and reasons.
6. Reserve 10% response budget.

### Validation

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
```

Behavior checks:

- P0 blocks are never omitted silently.
- Single LLM call stays under configured token budget.
- Focus memory is included when available.
- Archive memory is included only when relevant and budget allows.

### Rollback

- Set `context_window_manager_enabled = false`.
- Revert to simple prompt assembly.

人话版：第四步是给模型打包背包。证件和任务单必须带，旧聊天和长日志按空间挑。

## 9. Phase 5 — ActExecutor and ToolCallV1

### Goal

Make Act choose one concrete tool call using `ToolCallV1`, execute it through existing tool runtime, and chain multi-step workflows through blackboard state.

### Target files

```text
src/intelligence/agent-core/act_executor.rs          # New
src/intelligence/agent-core/agent_loop.rs
src/intelligence/agent-core/llm/bridge.rs
src/intelligence/agent-core/prompts/act_prompt.md
src/intelligence/agent-core/tool_manifest.rs
src/engine/tool-system/src/registry.rs
```

### Minimal implementation

1. Add `ActLlmBridge` or extend bridge with Act-specific method.
2. Parse `ToolCallV1`.
3. Validate tool exists and params match schema when available.
4. Route High/Critical actions to governance.
5. Execute through `ToolRegistry.get(tool_name).execute(params)`.
6. Write result to blackboard keys.
7. Trigger micro-reflect on failure.
8. Block repeated identical failed calls.

### Validation

```bash
cargo check -p intelligence-agent-core
cargo test -p intelligence-agent-core --lib
cargo test -p engine-tool-system
```

Behavior checks:

- Agent picks one next tool.
- Tool params are valid JSON.
- Chained tool flow uses blackboard state.
- Same failed params are not repeated.
- High-risk call waits for governance approval.

### Rollback

- Disable `act_toolcall_v1_enabled`.
- Fall back to existing hard-coded tool/task execution path.

人话版：第五步才是接机械臂。每次只让它拿一把工具，失败就记录，危险就审批。

## 10. Phase Gates

Do not start a later phase until the prior phase passes:

| Gate | Required evidence |
|---|---|
| Phase 0 → 1 | Docs exist and diff is docs-only. |
| Phase 1 → 2 | Persona injection does not break existing Planner/Reflector tests. |
| Phase 2 → 3 | Planner can emit or map `suggested_tools`; invalid JSON fallback works. |
| Phase 3 → 4 | Reflector can route at least Continue / Retry / StopAndHandoff. |
| Phase 4 → 5 | Token accounting keeps prompt under configured budget. |
| Phase 5 → final | ToolCallV1 executes at least one safe read-only tool and one validation tool. |

人话版：每扇门过了再进下一间。别厨房水电都没验，就开始装米其林摆盘灯。

## 11. Receipts Required Per PR

Each implementation PR should include:

```text
receipts/AGENT-PROMPT-CORE-001/
├── phase-N-summary.md
├── changed-files.txt
├── commands-run.txt
├── test-output-summary.txt
├── prompt-audit-sample-redacted.md
└── known-risks.md
```

Rules:

- Do not include API keys or full private prompts containing sensitive workspace data.
- Redact secrets and local paths if needed.
- Include exact command names and PASS / FAIL / UNKNOWN.

人话版：每次改动都要留施工照片、验收单、失败记录。不是“我本地好了”。

## 12. Suggested Implementation Order by Owner Value

1. Phase 0 + 1: immediately improves Agent identity with low risk.
2. Phase 2: makes planning visibly smarter.
3. Phase 3: prevents dumb repeated failures.
4. Phase 4: controls cost and context quality.
5. Phase 5: unlocks tool-native autonomous execution, highest value and highest risk.

人话版：先让员工知道自己是谁，再让他知道工具箱，再教他复盘，最后才放他真上手干活。

## 13. Out of Scope

- Shipping a full runtime implementation in this docs package.
- Changing public user-facing product behavior before Owner approval.
- Removing existing rule-based fallbacks.
- Breaking current 249-test baseline.

人话版：路线图不是施工完成证明。它只是告诉工程队怎么一步一步安全落地。
