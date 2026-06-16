# STONE-AUDIT-V3X-DAY09 app.js High-risk Remainder Decision

## Scope

- Task: `STONE-AUDIT-V3X-DAY09`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `22efbaf99b4acb79264f297711540ff71c1ba255`
- Mode: docs-only decision
- Production code modified: `NO`
- WebView: `NOT RUN`
- Decision: `RETARGET`

Human version: the low-risk old wires are already cut. The remaining `app.js`
mass is closer to gas, power, and water lines: chat streaming, Provider/Keyring,
Checkpoint, Shell/tool execution, and Agent streaming. Do not keep swinging at
the `app.js <=1200` target as a single demolition goal without separate high-risk
work orders.

## Git Baseline

| Check | Result |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `22efbaf99b4acb79264f297711540ff71c1ba255` |
| `git pull --ff-only origin stone-audit-v3x-controlled-demolition` | Already up to date |
| Historical dirty files | Present, not staged |
| Day08 receipt | `docs/frontend/APPJS-FALLBACK-REMOVAL-BATCHES-V3X.md` |

## Target Status

| Target | Current | Status | Evidence |
|---|---:|---|---|
| `app.js <=1200` | `5568` lines | `NOT MET` | `(Get-Content src/interface/web/app.js).Count` |
| docs-only Day09 | no production diff | `MET` | forbidden diff command had no output |
| decision output | `RETARGET` | `MET` | this document |

## Day08 Input Summary

Day08 completed as `BLOCKED / NO-OP`. It did not remove production code because
Day07 had already completed every `SAFE-TO-REMOVE` fallback candidate:

1. Command Palette six pure fallback bodies.
2. Resource Dashboard compatibility fallback.
3. Session List View compatibility fallback.

Day08 explicitly says there is no new `app.js` fallback block with enough Node
plus WebView evidence to delete safely.

## High-risk Function Inventory

| Area | app.js evidence | Risk class | Reason |
|---|---|---|---|
| Chat send path | `sendChatMessage()` at line `2376`; calls `streamChat()` at line `2455` | HIGH-RISK | Main chat path, active provider state, message history, command parsing, streaming boundary |
| Slash command chat branch | `/chat` branch around lines `2598-2622` | HIGH-RISK | Direct provider selection and stream call; user-visible model behavior |
| Agent invoke path | `invokeAgentTask()` at line `3055`; `handleAgentEvent()` at line `3126` | HIGH-RISK | Tauri channel event stream, trace rendering, Agent lifecycle |
| Agent streaming / chat stream | `streamChat(provider, prompt, config, messages)` at line `3229`; invokes `stream_chat` around line `3367` | HIGH-RISK | Streaming channel, cancellation/finalization, markdown rendering, provider config |
| Provider settings | `setupProviderSettings()` at line `3733`; `saveProviderConfig()` at line `4193`; `deleteProviderConfig()` at line `4257` | HIGH-RISK | Provider CRUD, API key handling, keyring-adjacent save/delete behavior |
| Provider backup/import | `exportProviderBackup()` at line `4172`; `importProviderBackup()` at line `4182` | HIGH-RISK | Encrypted backup/import and file path handling |
| Provider probe/test | probe setup around lines `3837-3990`; post-init validate provider around lines `5527-5562` | HIGH-RISK | Provider capability probe and validation invoke paths |
| Agent provider binding | `loadAgentProviders()` at line `4341`; `setupAgentProvider()` at line `4373`; `unbindAgentProvider()` at line `4395` | HIGH-RISK | Agent-provider mapping changes model routing |
| Checkpoint list/restore/export/compare/replay | checkpoint area around lines `4856-4994` | HIGH-RISK | Restore/export/compare/replay can affect workspace state or user recovery paths |
| Shell/tool execution adjacency | shell language helpers around lines `185`, `1394`, `1419`, tool text around `3422-3442` | EXECUTION-RISK | Shell/tool execution is policy-sensitive even when sampled from frontend text paths |

## UNKNOWN / BLOCKED Table

| Area | Status | What is unknown or blocked |
|---|---|---|
| Chat streaming extraction | `BLOCKED` | Needs dedicated Node smoke plus real WebView stream receipt before any fallback/wrapper removal |
| Provider/Keyring extraction | `BLOCKED` | Must not modify save/delete/keyring/probe/backup semantics without a separate guarded task |
| Checkpoint extraction | `BLOCKED` | Restore/export/compare/replay needs separate dry-run and WebView receipt |
| Shell/tool execution | `BLOCKED` | Shell policy and tool execution must remain isolated; do not touch in frontend demolition task |
| Agent streaming/governance | `BLOCKED` | Channel event handling and Agent lifecycle require separate high-risk migration plan |
| `app.js <=1200` as a hard near-term target | `UNKNOWN / NOT MET` | Current remaining mass is high-risk dominated; no evidence that a safe single-pass reduction can reach 1200 |

## Decision

Decision: `RETARGET`.

Do not continue a broad `app.js <=1200` demolition target as the next direct
implementation goal. Retarget V3X from one hard line-count goal to a staged risk
goal:

1. Keep the completed low-risk extractions as done.
2. Treat remaining high-risk areas as separate work orders with their own smoke,
   WebView, rollback, and debt criteria.
3. Use line count as a secondary metric after each high-risk domain passes
   dedicated evidence gates.

This is not a `CONTINUE` decision because continuing broad deletion would violate
Day08's evidence boundary. This is not a pure `DEFER` decision because the repo
can still move forward, but only by changing the target shape from "make app.js
<=1200 now" to "one high-risk domain at a time, with proof."

## Required Future Tasks

| Future task ID | Purpose | Allowed direction | Required evidence before production removal |
|---|---|---|---|
| `V3X-DAY09-F1-CHAT-STREAMING-SAMPLING` | Sample `sendChatMessage`, `/chat`, `streamChat`, message rendering, and channel events | docs-only first | node smoke map + WebView chat receipt |
| `V3X-DAY09-F2-PROVIDER-KEYRING-GUARD` | Sample Provider CRUD, keyring-adjacent fields, backup/import, probe/test | docs-only first | no keyring write during smoke unless explicitly approved |
| `V3X-DAY09-F3-CHECKPOINT-READONLY-GUARD` | Sample checkpoint list/export/compare/replay/restore boundaries | docs-only first | restore stays dry-run unless separate approval |
| `V3X-DAY09-F4-SHELL-TOOL-EXECUTION-GUARD` | Map frontend shell/tool command paths and security assumptions | docs-only first | no shell execution during sampling |
| `V3X-DAY09-F5-AGENT-STREAMING-GUARD` | Sample Agent task/channel/trace/governance event handling | docs-only first | channel event smoke + WebView Agent receipt |
| `V3X-DAY09-F6-SECURITY-DOM-VIEW-DEBT` | Address current security-gate high findings in view files | focused fix task | `npm run test:security-gate` improvement without allowlist laundering |

## Validation Results

| Command | Result |
|---|---|
| `git branch --show-current` | `stone-audit-v3x-controlled-demolition` |
| `git rev-parse HEAD` | `22efbaf99b4acb79264f297711540ff71c1ba255` |
| `git status --short` | Historical dirty files present |
| `(Get-Content src/interface/web/app.js).Count` | `5568` |
| `rg -n "sendChatMessage|streamChat|handleAgentEvent|invokeAgentTask|Provider|provider|keyring|probe|backup|checkpoint|shell|Agent" src/interface/web/app.js` | Located high-risk paths |
| `node --check src/interface/web/app.js` | PASS |
| `npm run test:security-gate` | FAIL: 97 findings, 3 failures, 94 warnings, 94 allowlisted |
| `git diff --name-only -- src/interface/web src/interface/desktop src/engine/tool-system` | No output |
| `git diff --cached --check` | PASS before staging |

## Security Gate Status

The security gate is not cleared.

Known current failures:

1. `src/interface/web/views/command-palette-view.js:36`
2. `src/interface/web/views/session-list-view.js:22`
3. `src/interface/web/views/session-list-view.js:26`

These failures are outside the docs-only Day09 change. Day09 does not mark them
as fixed.

## WebView Status

WebView: `NOT RUN`.

Reason: Day09 is a docs-only decision task. It does not modify a UI path and
does not claim WebView PASS.

## Production Diff Receipt

Production changes: `NO`.

Checked forbidden paths:

- `src/interface/web`
- `src/interface/desktop`
- `src/engine/tool-system`

The forbidden diff command produced no output during Day09 sampling.

## Rollback / Debt

- Rollback point: `22efbaf99b4acb79264f297711540ff71c1ba255`
- Since Day09 is documentation-only, rollback is limited to this document if the
  decision text needs correction.
- Residual debt: `app.js <=1200` remains `NOT MET`; high-risk frontend domains
  require separate future work orders.

## 工单 V3X-DAY09 完成

- Commit: `docs(debt): record appjs high risk remainder`
- 分支: `stone-audit-v3x-controlled-demolition`
- HEAD before / after: `22efbaf99b4acb79264f297711540ff71c1ba255` / pending commit
- app.js lines: `5568`
- app.js <=1200: `NOT MET`
- decision: `RETARGET`
- high-risk functions: chat send/stream, Agent task/event, Provider/Keyring, Provider backup/probe, Checkpoint, Shell/tool execution
- node --check app.js: PASS
- security-gate: FAIL, known baseline, not cleared
- production changes: NO
- old dirty files staged: NO
- next: execute Day10 `main.rs` docs-only split sampling, or open one of the Day09 future high-risk frontend work orders
