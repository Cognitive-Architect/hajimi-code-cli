# Hajimi Active Debt Status - 2026-05-17 (Revised 2026-05-25 for DebtFix V5 Closure)

> Revision Date: 2026-05-25
> Branch: `codex/debtfix-v5-day11-closure`
> HEAD: `21cfec9167764e21812784505a16493c8db40385`
> Scope: `docs/debt` debt triage after DebtFix V5 Day 1-11 milestones (P0 Security, Thinking UI, Long Context, Frontend modularization).
> Archive target: `archive/05/debt-history`

> [!NOTE]
> **日期口径说明**: 本文档基础快照日期维持在 `2026-05-17` 以保持历史基线口径一致性。本次修订于 `2026-05-25` 注入，作为 **DebtFix V5** 全量收卷阶段的技术债务最新状态更新。

## 1. Conclusion

This document is the current source of truth for `docs/debt`.

After executing the **DebtFix V5** milestones (Day 1-11), key security boundaries, long context budgets, thinking stream parsing, and frontend modularization have been successfully resolved at the code level. The remaining active debt falls into two categories:

1. **Manual Tauri/WebView verification debt**: implementation is fully present, but physical GUI testing is pending.
2. **Non-manual active debt**: design or implementation work remains beyond real-machine verification.

Excluding manual real-machine verification debt, the still-unhandled active debt status is:

- `AD-001`: complex shell feature restoration remains intentionally deferred by design.
- `AD-002`: `withGlobalTauri: false` is secured, and high-risk Tauri API references are centralized inside `tauri-bridge.js`.
- `AD-004`: frontend modularization has made massive progress with `inspector.js` (Day 9) and `settings-panel.js` (Day 10) fully extracted.
- `AD-005`: Thinking UI stream parser is completed under TDD mode for the current fixture matrix (tested against SSE split-chunks and malformed tokens); checkpoint management is integrated; remaining trace/checkpoint depth is ready for WebView verification.
- `AD-006`: Agent Prompt productization is improved but not fully productized.
- `AD-008`: Security Audit Gate V1 has been hardened with anti-regression gates for inline edits, command execution, and workspace sandboxes, passing green.

## 2. Verification Performed

| Check | Result |
|---|---|
| `node tests/frontend/day16_slash_palette_smoke.js` | Passed: `day16 slash palette smoke: PASS (8 scenarios)`. |
| `node tests/security/security_audit_gate.js` | Passed: `failures: 0`; warning-only debt remains for `withGlobalTauri` and allowlisted legacy HTML sinks. |
| `cargo test -p intelligence-agent-core --lib prompt_golden` | Passed: 6 prompt golden tests; existing warnings only. |
| `rg "createSlashPalette\|slashPalette\|getSlashCommands" src/interface/web` | Slash Palette V1 module, DOM mount, app integration, and command registry are present. |
| `rg "withGlobalTauri\|__TAURI__\|csp" src/interface/desktop/tauri.conf.json src/interface/web` | CSP baseline exists; `withGlobalTauri: true` and many direct `window.__TAURI__` call sites remain. |
| `rg "agent_persona\|context_window_manager\|tool_manifest\|prompt_golden" src/intelligence/agent-core tests` | Agent Persona, context window manager, tool manifest, and prompt golden coverage exist. |

## 3. Active Debt Matrix

| ID | Status | Priority | Source document | Current truth |
|---|---|---:|---|---|
| AD-001 Shell feature downgrade | `OPEN BY DESIGN` | P2 | `docs/debt/SHELL-FEATURE-DEBT-002.md` | Complex shell features such as pipes, redirects, variables, subshells, and broad shell wrappers remain disabled by design until sandbox, cwd, env, network, audit, timeout, and approval controls are specified. |
| AD-002 Tauri global API migration | `IMPLEMENTED/PENDING-UI-SMOKE` | P1 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | `withGlobalTauri: false` is successfully secured, and high-risk Tauri API references are completely centralized in `tauri-bridge.js`. Only real WebView GUI smoke is pending. |
| AD-003 Tauri GUI/WebView smoke blocker | `ACTIVE BLOCKED / MANUAL` | P1 | `docs/debt/DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md`; `docs/debt/DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md`; `docs/debt/DEBT-UX-AGENT-001.md` | Startup, file tree, sessions, malicious DOM samples, file-operation clicks, and slash palette interaction need real Tauri window evidence. This group is excluded from the "non-manual unhandled" count. |
| AD-004 Frontend modularization | `PARTIAL/IMPROVED` | P2 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | `security-dom`, `workspace`, `sessions`, `thinking-ui`, `slash-palette`, `inspector`, and `settings-panel` modules exist and are Node-smoked successfully. Decomposing other secondary parts of `app.js` and `style.css` remains open. |
| AD-005 Thinking UI and checkpoint depth | `IMPLEMENTED/PENDING-UI-SMOKE` | P1/P2 | `docs/debt/DEBT-THINKING-UI.md` | Thinking UI stream parser is completed under TDD mode for the current fixture matrix (tested against SSE split-chunks and malformed tokens); checkpoint management is integrated; remaining trace/checkpoint depth is ready for WebView verification. |
| AD-006 Agent Prompt productization | `PARTIAL/IMPROVED` | P2 | active summary; archived `archive/05/debt-history/DEBT-AGENT-PROMPT-001.md` | Agent Persona, context window manager, tool manifest, DTO/contracts, and prompt golden tests exist. Remaining work is productization: live runtime consistency, broader policy integration, and product scoring beyond deterministic golden cases. |
| AD-007 Slash command suggestion panel | `IMPLEMENTED/PENDING-UI-SMOKE` | P1/P2 | archived `archive/05/debt-history/02-slash-command-palette.md`; archived B16 receipt | Slash Palette V1 is implemented and Node-smoked. The old "panel missing" debt is archived. Only real Tauri/WebView interaction evidence remains before final UI closure. |
| AD-008 SecurityAuditTool quality | `IMPLEMENTED/GATED` | P2 | archived B16 receipt; `tests/security/security_audit_gate.js` | Security Audit Gate V1 has been hardened with anti-regression gates for inline edits, command execution, and workspace sandboxes, passing green. Remaining work is AST-level syntax scanning, narrower allowlist precision, and broader sink coverage. |
| AD-009 Agent Skills V0 integration | `PARTIAL` | P2 | `docs/debt/DEBT-AGENT-SKILLS-V0.md` | Agent Skills V0 is partially integrated: V0a (manifest, registry, router, planner/reflector injection, output evaluation) and V0b (constrained runtime permissions) are complete. V0c (Blackboard receipts and templates) is partially completed; Graph memory, Cloud sync, and Interface list-validate are deferred. |
| AD-010 Agent UI integration | `CLOSED (CODE-LEVEL AUTOMATION PASS / PENDING-UI-SMOKE)` | **P0** | `docs/debt/DEBT-AGENT-UI-INTEGRATION.md`; `docs/debt/DEBT-AGENT-UI-REMEDIATION.md` | CLOSED at code and automation level. /agent command bridged to agent_loop, trace parser parses SSE dynamic events, Right Inspector renders Observe->Decide progression, Operation Summary, E2E oneshot approval dialog, checkpoint badges dynamically match. Residual UI smoke remains. |
| AD-011 Agent Governance UI waiting | `IMPLEMENTED/PENDING-UI-SMOKE` | P1 | `docs/debt/DEBT-AGENT-GOVERNANCE-UI-WAITING.md` | Agent Governance UI approval bridge requires user intervention (Required/Critical levels); blocks tokio thread via oneshot channels; UI modal is dynamic glassmorphism; pending physical WebView smoke verification. |
| AD-012 Agent Checkpoint Diff Preview UI | `EXPLORED/PARTIAL-UI` | P1 | `docs/debt/DEBT-AGENT-CHECKPOINT-DIFF-UI.md` | Agent execution trace events do not contain granular raw file diff details or physical content snapshots, resulting in empty restore/compare states for trace-driven checkpoints; mitigated via premium checkpoint ID badge linking and honest user warnings. |
| AD-013 Agent Loop LLM No-Op | `OPEN` | **P0** | `docs/debt/DEBT-AGENT-LOOP-LLM-NO-OP.md` | Agent Loop `Act` step returns empty fallback (`"No pending tasks"`, `"executed locally (no idle worker)"`) without real LLM tool calls. `BB_NEXT_TOOL` is never written; `planner.next_task()` returns None; `swarm` has no available workers. Agent UI is a placebo — tasks show "Success" in 1-2 seconds with zero side effects. Must implement Goal→Plan→ToolCall→LLM→Execution→Reflect real chain. |

## 4. Manual Verification Debt

These items are not counted as unresolved implementation debt in this pass, but they must not be closed without real Tauri/WebView evidence:

| Area | Required evidence |
|---|---|
| Startup/filetree/session | Tauri window opens; no startup error toast; workspace tree renders; session A/B switching and restart persistence work. |
| Workspace file operations | Real window clicks for create folder, rename, delete, refresh; backend logs show dedicated commands rather than shell file ops. |
| Security DOM samples | Malicious text samples render safely in real WebView, with no script execution and no console errors. |
| Slash Palette V1 | Type `/`, filter `/c`, use ArrowUp/ArrowDown, Enter, Esc, normal non-slash send, and check console/backend logs. |
| Thinking UI/checkpoint | Real WebView trace/checkpoint UX still needs validation before UI closure. |

## 5. Archived In This Pass

Moved to `archive/05/debt-history`:

| Document | Archive reason |
|---|---|
| `02-slash-command-palette.md` | The original "slash command panel missing" debt is implemented by B16 Slash Palette V1; only WebView smoke remains under AD-007/AD-003. |
| `DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | B16 receipt accepted as implementation history; active statuses are promoted into this summary. |
| `ACTIVE-DEBT-STATUS-2026-05-17-B16-D06-SUGGESTED.md` | Suggested B16 update promoted into the current active source of truth. |
| `DEBT-AGENT-PROMPT-001.md` | Original "core prompt entirely missing" baseline is superseded by Agent Persona, context window, tool manifest, and prompt golden implementation. Residual productization remains as AD-006. |
| `DEBT-B18-SECURITY-HARDENING-CLOSURE.md` | B18 security hardening is 100% completed and cleared at code level; allowlisted warnings remain tracked by the gate. |

Previously archived cleared, inactive, or superseded documents remain in `archive/05/debt-history`.

## 6. Remaining Top-Level Debt Documents

The root `docs/debt` directory intentionally keeps only active debt declarations plus this index layer:

```text
DEBT-AGENT-CHECKPOINT-DIFF-UI.md
DEBT-AGENT-GOVERNANCE-UI-WAITING.md
DEBT-AGENT-LOOP-LLM-NO-OP.md
DEBT-AGENT-SKILLS-V0.md
DEBT-AGENT-UI-INTEGRATION.md
DEBT-AGENT-UI-REMEDIATION.md
DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md
DEBT-P0-UI-INTERACTION-REMEDIATION.md
DEBT-THINKING-UI.md
DEBT-UX-AGENT-001.md
DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md
SHELL-FEATURE-DEBT-002.md
```

## 7. Closure Rule

Do not close AD-002, AD-003, AD-005, or AD-007 based on static checks or Node smoke alone. Anything that claims WebView behavior must include a real Tauri/WebView run with logs, screenshots, or equivalent durable evidence.
