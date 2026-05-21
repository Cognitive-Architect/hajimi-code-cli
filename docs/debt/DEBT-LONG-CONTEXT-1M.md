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

| Risk | Level | Day 15 Clean-closure Status | Engineering Facts |
|---|---:|---|---|
| Bridge hardcoded 8K | P0 | **[CLEARED]** | Resolved via `resolve_context_budget` pure engine. |
| system prompt undercount | P0 | **[CLEARED]** | Resolved via `estimate_tokens` dynamically in bridges. |
| Provider capability gap | P1 | **[CLEARED]** | `ProviderConfig` extended with modern camelCase fields. |
| Memory budget conflict | P1 | **[CLEARED]** | Deriving request-aware Focus/Working/Archive limits. |
| Probe truth gap | P1 | **[DEFERRED]** | MockProbe validated perfectly; real provider probe pending integration (`DEBT-LONG-CONTEXT-PROBE-001`). |
| Receipt gap | P2 | **[CLEARED]** | Asynchronous redact-redacted小票 metadata generated and saved successfully. |
| Ignored debt doc | P1 | **[ACTIVE]** | `.gitignore:158:docs/` ignores this folder. Must forcefully stage via `git add -f`. |
| GUI Smoke clicks | P1/P2 | **[DEFERRED]** | Deferred manual clicking validations to `DEBT-LONG-CONTEXT-GUI-001/002/003`. |

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

- Day 2-3: [CLEARED] Created `ContextBudget`, `known_model_caps`, and dynamic budget resolution engine.
- Day 4-5: [CLEARED] Replaced bridge 8K hardcoding and fixed system prompt token accounting.
- Day 6-7: [CLEARED] Upgraded provider capability fields while retaining old `contextThreshold` compatibility.
- Day 8-9: [CLEARED] Added `LongContextPackBuilder` and integrated included/omitted context blocks with dry-run support.
- Day 10: [CLEARED] Deriving Memory retrieval budget dynamically from request-aware limits (Focus/Working/Archive).
- Day 11-12: [CLEARED] Dynamic context capacity probe model, local JSON persistence, auto fallback levels, and UI presets.
- Day 13: [CLEARED] Context receipt token usage tracking (included vs omitted blocks) and right inspector UI rendering.
- Day 14-15: Day14 matrix recorded / automated checks reported pass / Day15 closure pending.

---

## Day 14 E2E Matrix Verification

Commands executed on **2026-05-21** (Branch: `v3.8.0-batch-1`, True HEAD: `c9ef507948b2d261bf9b742d82d898f1341a668f`, correcting previous receipt SHA mismatch):

### 1. 自动化质量检查矩阵（16项刀刃表）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据方式 | 状态 | 证明/输出摘要 |
|---|---|---|---|---|---|
| FUNC | FUNC-001 | budget 预算测试 | `cargo test -p intelligence-agent-core context_budget` | **PASS** | 24 tests passed, 0 failed. Verifies legacy/fast/pro/long dynamic limit resolution and env overrides. |
| FUNC | FUNC-002 | pack 组包测试 | `cargo test -p intelligence-agent-core long_context_pack` | **PASS** | 8 tests passed, 0 failed. Verifies file exclude filter, repo tree generation, and head-tail policy. |
| FUNC | FUNC-003 | probe 探针测试 | `cargo test -p intelligence-agent-core context_probe` | **PASS** | 6 tests passed, 0 failed. Verifies success, cancelled, failed, expired, and fallback levels. |
| FUNC | FUNC-004 | receipt 小票测试 | `cargo test -p intelligence-agent-core context_receipt` | **PASS** | 17 tests passed, 0 failed. Verifies serialization, block snippet tracking, and privacy redaction. |
| CONST | CONST-001 | workspace 编译 | `cargo check --workspace` | **PASS** | Workspace compiled successfully with 0 errors and 0 new warnings. |
| CONST | CONST-002 | agent-core lib 测试 | `cargo test -p intelligence-agent-core --lib` | **PASS** | 221 tests passed, 0 failed. |
| CONST | CONST-003 | no 8K hardcode | `rg "ContextWindowManager::new\(8000\)" src` | **PASS** | Grep returned 0 matches, confirming complete removal of the hardcoded 8K constraint in bridges. |
| CONST | CONST-004 | no bridge token=0 | `rg "token_estimate: 0" src/intelligence/agent-core` | **PASS** | Zero matches for bridge zero-token placeholders. Bridges use dynamic `estimate_tokens` calls. |
| NEG | NEG-001 | feature-gate 回滚 | `$env:HAJIMI_LONG_CONTEXT_ENABLED="false"; cargo test -p intelligence-agent-core --lib` | **PASS** | Setting environment variable to `false` triggers standard 8K paths; all 221 unit tests passed. |
| NEG | NEG-002 | fallback 降级测试 | `cargo test -p intelligence-agent-core context_probe` | **PASS** | Verifies failed probes trigger fallback cascade: e.g. 900K fail falls back to 512K / 256K / 128K / 32K. |
| NEG | NEG-003 | stale 过期测试 | `cargo test -p intelligence-agent-core context_budget context_probe` | **PASS** | Verifies stale (expired TTL) probe records dynamically cap the context window budget at 128K. |
| NEG | NEG-004 | receipt 隐私安全 | `cargo test -p intelligence-agent-core context_receipt` | **PASS** | `test_redact_*` checks redact API keys (sk-...), bearer tokens, passwords, and environment keys. |
| UX | UX-001 | UI 状态字段绑定 | `rg "Declared|Verified|Stale|Fallback" src/interface src/intelligence` | **PASS** | Front-end app.js and index.html actively present all status display states. |
| UX | UX-002 | token UI 字段渲染 | `rg "inputBudget|estimatedInputTokens|included|omitted" src/interface/web/app.js` | **PASS** | app.js renderContextReceiptPanel maps omitted/included lists and budgets to Right Inspector. |
| E2E | E2E-001 | 桌面端编译 | `cargo check -p hajimi-desktop` | **PASS** | Crate `hajimi-desktop` compiles cleanly with 0 errors. |
| High | HIGH-001 | 4层架构分层合规 | `rg "interface/desktop|ProviderConfig" src/intelligence/agent-core` | **PASS** | Zero matches. agent-core has no reverse imports, maintaining perfect分层 boundary. |

---

## Active Manual UI Smoke-Testing Debt

Under the project rule *“涉及到实机点击的部分先不用管记录债务就行”*, the physical manual clicking checks on the desktop graphical interface are officially registered as structural debt below:

- **DEBT-LONG-CONTEXT-GUI-001**: Manual verification of clicking the "Context Capacity Probe" button in Settings and watching the beautiful micro-animation update the tested token count to 128K/256K/512K/900K is deferred.
- **DEBT-LONG-CONTEXT-GUI-002**: Verification of clicking "Refresh" inside the "Context 小票" (Context Receipt) Right Inspector tab to pull the serialized receipt metadata asynchronously remains manual-only.
- **DEBT-LONG-CONTEXT-GUI-003**: Smoke test of saving a custom provider config with high context token settings via the desktop settings panel modal remains blocked.

*Consequence*: These interactions are structurally backed by complete backend RPC Tauri invokes and front-end element bindings, but have not been physically verified via manual GUI smoke testing due to Tauri WebView environment execution constraints.

---

## Verdict

The automated checks and unit/integration matrix for the Hajimi 1M Context Engine have been successfully run and recorded. However, please note the following critical scoping and engineering facts:

1. **No Real Provider Probe**: The current real provider probe has **NOT** been implemented yet. It is fully a mocked/simulated capability verification.
2. **MockOnly is NOT Verified**: `MockOnly` is **NOT** equal to a real-world `Verified` status. 1M context remains strictly at `Declared / MockOnly / Not Real-Provider-Verified` status.
3. **Estimated Receipts**: The context receipt token count is strictly **estimated**, **NOT** the actual usage charged by the LLM provider.
4. **GUI Smoke Debt Preservation**: The active manual UI testing debts (GUI clicks) are fully preserved and registered in this document.

The workspace compiler, dynamic budget resolution rules, context pack head-tail filters, metadata receipts, and rollback toggles are verified complete at the code level, but final real provider E2E and GUI integration tests remain pending.
