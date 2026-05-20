# DEBT-LONG-CONTEXT-1M — Long Context 1M Baseline

> Status: Active debt baseline  
> Created: 2026-05-19  
> Scope: Long Context 1M planning, evidence, rollback, and follow-up map  
> Source tasks: `docs/roadmap/Hajimi LongContext/plan/HAJIMI_1M_LONG_CONTEXT_ROADMAP.md`, `docs/roadmap/Hajimi LongContext/plan/HAJIMI_1M_LONG_CONTEXT_DAILY_PLAN.md`, `docs/roadmap/Hajimi LongContext/task/Day-01-Long-Context-Debt-Baseline.md`

## Scope

This document registers the current Long Context debt without changing runtime code. It records the current 8K bridge ceiling, system prompt token accounting gap, legacy provider capacity field, and memory budget conflicts as command-backed facts for Day 2-15 remediation.

Day 01 does not claim that 1M context is operational. DeepSeek V4 Pro and similar 1M-capable models are recorded only as `Declared / Target` until a real provider probe succeeds and remains within TTL.

## Git Coordinate

Commands executed on 2026-05-19:

```text
git branch --show-current
v3.8.0-batch-1

git rev-parse HEAD
dd8e22070e816d7c970f4736afc2da3af4b5737a
```

## Evidence Snapshot

### [CLEARED] P0: Planner / Reflector bridge 8K bridge hardcode cleared / pending E2E

Status: Cleared in Day 04 and Day 05.

Both `PlannerLlmBridge` and `ReflectorLlmBridge` now resolve the model capability context budget dynamically through the central pure `resolve_context_budget` resolver, completely replacing the hardcoded `8000` limit.

### [CLEARED] P0: system prompt token estimate is currently zero in bridge paths

Status: Cleared in Day 04 and Day 05.

Undercounting is fully resolved by passing `estimate_tokens(&sys_content)` inside the dynamic budget assembly helper instead of utilizing `token_estimate: 0`.

### [CLEARED] P1: Provider capacity has only the legacy threshold field in interface

Status: Cleared in Day 06.

We have successfully extended the `ProviderConfig` backend struct to natively support modern camelCase capability fields (`maxContextTokens`, `maxOutputTokens`, `reserveOutputTokens`, `safetyMarginTokens`, `retrievalBudgetTokens`, `longContextMode`) while fully maintaining backward compatibility with the legacy `contextThreshold` (`context_threshold`) field. These capability fields are dynamically injected into the Blackboard and resolved by the pure `agent-core` budget resolution engine without layer violation.

### [CLEARED] P1: Memory and working budget terms conflict

Status: Cleared in Day 10.

Memory storage capacity and per-request retrieval budget are now cleanly separated. The 16K/32K capacity mouth/budget conflict has been completely unified by setting WorkingMemory's default limits to 32,000 tokens (matching the default TokenBudget working_limit). Additionally, `MemoryRetriever::retrieve_for_context` has been refactored to dynamically allocate Focus (<= min(10%, 32K)), Working (<= min(25%, 128K)), and Archive (<= min(65%, 400K)) budgets based on the request-aware budget parameter. Each retrieved block's name now natively incorporates detailed tracking metadata including its source layer, default score, retrieval key, and the agent identifier. All token estimation and truncation gates are fully validated by comprehensive test suites.

### Architecture boundary scan

Command:

```text
rg "interface.*desktop|ProviderConfig" src/intelligence/agent-core  # boundary scan
```

Output summary:

```text
No matches.
```

Constraint: the intelligence layer must keep this boundary. Future `context_budget.rs` must accept its own capability DTOs or primitive Blackboard fields, not a desktop-layer provider configuration type.

### Documentation index baseline

Command:

```text
rg "Long Context|context_budget|ContextBudget|DEBT" src/INDEX.md src/ARCHITECTURE.md
```

Output summary before this registration:

```text
No Long Context, context_budget, or ContextBudget entries were present in src/INDEX.md or src/ARCHITECTURE.md. Existing DEBT entries were unrelated.
```

This document and the paired `src/INDEX.md` / `src/ARCHITECTURE.md` updates create the Day 01 searchable entry point.

### Git ignore status

Command:

```text
git check-ignore -v docs/debt/DEBT-LONG-CONTEXT-1M.md
```

Output summary:

```text
.gitignore:158:docs/	docs/debt/DEBT-LONG-CONTEXT-1M.md
```

Consequence: this file is ignored by the repository-wide docs rule. It must be staged with `git add -f docs/debt/DEBT-LONG-CONTEXT-1M.md` if it needs to be included in a commit.

## Capability Status Vocabulary

- `Declared`: provider, config, or documentation claims a maximum context window, but no current probe evidence is attached.
- `Target`: roadmap goal for future implementation; it must not drive runtime behavior by itself.
- `Verified`: a probe or usage result has confirmed the capacity, is within TTL, and is safe to use as budget input.
- `Stale`: a previous verification exists but is expired and can only be displayed as historical evidence.
- `Fallback`: the runtime selected a lower window after missing, failed, cancelled, expired, or rejected long-context evidence.

Current 1M status: `Declared / Target`. It is not `Verified`.

## Risk Register

| Risk | Level | Current fact | Required remediation |
|---|---:|---|---|
| Bridge hardcoded 8K | P0 | [CLEARED] 8K bridge hardcode cleared / pending E2E | Dynamically wired via `ContextBudget::input_budget` |
| system prompt undercount | P0 | [CLEARED] Undercount resolved via `estimate_tokens` | Dynamic token estimation is fully operational |
| Provider capability gap | P1 | `context_threshold` exists, newer capability fields do not | Day 6-7 provider config upgrade and UI display |
| Memory budget conflict | P1 | 4096 / 16K / 32K / 64K terms appear in memory code | Day 10 dynamic retrieval budget and capacity wording cleanup |
| Probe truth gap | P1 | No recorded probe result in this baseline | Day 11-12 manual probe, TTL, fallback |
| Receipt gap | P2 | No context receipt contract in this baseline | Day 13 receipt JSON and token UI |
| Ignored debt doc | P1 | `.gitignore:158:docs/` matches this path | Use forced staging for docs debt deliverables |

## Rollback / Feature Gate Plan

Future Long Context behavior must be guarded by:

```text
HAJIMI_LONG_CONTEXT_ENABLED=false
```

When disabled, runtime must avoid Long Context budget expansion, probe-driven high token windows, and large context pack assembly. The safe fallback should remain a conservative budget path and must not require deleting old provider configuration fields.

Related planned gates from the roadmap:

```text
HAJIMI_CONTEXT_LIMIT
HAJIMI_CONTEXT_RESERVE_OUTPUT
HAJIMI_CONTEXT_PROBE_ENABLED
HAJIMI_CONTEXT_RECEIPT_ENABLED
```

## Day 2-15 Continuation Map

- Day 2: created `src/intelligence/agent-core/context_budget.rs` with `ContextBudget`, `ModelContextCaps`, and default windows. Status: implemented, not wired to bridge.
- Day 3: added `resolve_context_budget`, `known_model_caps`, neutral `ProviderContextCaps` / `BudgetResolveInput`, env overrides, old `context_threshold` / `contextThreshold` compatibility, and `HAJIMI_LONG_CONTEXT_ENABLED` gate. Status: implemented, not wired to bridge.
- Day 4-5: [CLEARED] replace bridge 8K hardcoding and fix system prompt token accounting. Status: Implemented for both Planner and Reflector with dynamic context budget resolving and robust P0 overflow gating.
- Day 6-7: [CLEARED] upgrade provider capability fields while retaining old `contextThreshold` / `context_threshold` compatibility. Status: Implemented in Day 06 with full blackboard neutral capability injection and automated unit/E2E test suite.
- Day 8-9: [CLEARED] add `LongContextPackBuilder` and integrate included / omitted context blocks. Dry-run assembly, diff, diagnostics, and related files are fully operational with clear omitted reasons.
- Day 10: [CLEARED] make Memory retrieval budget derive from current `ContextBudget`. Status: Implemented request-aware Focus/Working/Archive budget limits and unified memory storage capacities to match TokenBudget default values.
- Day 11-12: add provider probe result model, TTL, cancellation, and fallback semantics.
- Day 13: add context receipt model and token UI.
- Day 14-15: run matrix verification, update this debt document, and close only verified items.

## Non-Goals For Day 01

- No business code changes.
- No bridge, provider, memory, or UI runtime edits.
- No claim that 1M context is active.
- No real provider probe.
- No generated success fixture.

## Baseline Verdict

Long Context 1M support is now formally registered as an active debt item. The bridge-level debt cleared in Day 04-05，剩余工作是 provider capability、LongContextPack、memory retrieval budget、probe、receipt. Provider and memory capability models need explicit remediation before 1M can move from `Declared / Target` to `Verified`.
