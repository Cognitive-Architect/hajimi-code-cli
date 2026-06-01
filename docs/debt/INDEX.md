# Hajimi Debt Documentation Index

> Updated: 2026-06-01
> Current source of truth: `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md`

## Active Summary

Use this file first:

- `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md`

## Active Debt Documents

| Document | Current status |
|---|---|
| `DEBT-AGENT-SKILLS-V0.md` | Agent Skills V0 is partially integrated: V0a cleared (manifest, registry, router, planner/reflector injection, output evaluation), V0b integrated (constrained runtime permissions, ActExecutor filtering), V0c partial (blackboard execution receipts, templates). Graph memory, cloud sync, and list-validate are deferred. |
| `DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md` | Manual Tauri/WebView UI smoke remains blocked. |
| `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | Frontend modularization and global Tauri API migration remain partial. |
| `DEBT-THINKING-UI.md` | Thinking UI and checkpoint V1 exist; richer diff, transaction restore, malformed stream tests, provider-token integration, and WebView smoke remain. |
| `DEBT-UX-AGENT-001.md` | Startup/filetree/session fixes are code-level complete but need real GUI verification. |
| `DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md` | Tauri dev smoke blocker remains active. |
| `DEBT-AGENT-GOVERNANCE-UI-WAITING.md` | Agent Governance UI approval bridge requires user intervention (Required/Critical levels); blocks tokio thread via oneshot channels; UI modal is dynamic glassmorphism; pending physical WebView smoke verification. |
| `DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG.md` | **P0** — Code-level fixed; pending rebuilt real WebView smoke. `/agent` no longer has an unbounded approval wait in code, read-only tools are not promoted to Critical, and native tool/governance trace events are forwarded. |
| `DEBT-AGENT-LLM-NATIVE-SUCCESS-RESULT-DROPPED.md` | **P1** — [CODE-LEVEL FIXED / PENDING-REAL-WEBVIEW-SMOKE] Real desktop `/agent` smoke proved DeepSeek and `list_directory` complete while the chat card only displayed `Success`; V1 now preserves `TurnOutcome.final_message` via `LoopOutcome::SuccessWithMessage`, desktop result output, and frontend rendering. |
| `DEBT-AGENT-LLM-NATIVE-THINKING-LEAK.md` | **P1** — [OPEN / DIAGNOSED] Real desktop `/agent` smoke now shows final content, but raw `<thinking>...</thinking>` tags are rendered inline in the answer body because the `/agent` result branch skips the existing Thinking UI parser. |
| `DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN.md` | **P1** — [FIXED-CANDIDATE / NEEDS-RECHECK] `/agent` reaches model/tool execution but repeats `list_directory`, crosses the 8192 token meltdown threshold (`8611` tokens observed), and returns `ActFailed` before producing the simple directory-list answer. |
| `DEBT-AGENT-CHECKPOINT-DIFF-UI.md` | Agent execution trace events do not contain granular raw file diff details or physical content snapshots, resulting in empty restore/compare states for trace-driven checkpoints; mitigated via premium checkpoint ID badge linking and honest user warnings. |
| `SHELL-FEATURE-DEBT-002.md` | Complex shell features remain intentionally downgraded by design. |
| `DEBT-COMPLEXITY-DAY05-001.md` | `bootstrap_first_tool_call` method exceeds 80 lines due to async locks, rule-based safe fallback tool mapping, registry verification, and UX trace emission. |

## Code-Level Guides & Closure Receipts

- `docs/debt/code-level/PRIORITY-GUIDE.md`: Actions and prioritization guide for P0/P1/P1-C security and functional debt.
- `docs/debt/code-level/DEBTFIX-V5-CLOSURE-RECEIPT.md`: Closure receipt for the DebtFix V5 code-level automation pass, with residual real-provider and WebView manual debts retained.
- `docs/debt/DEBT-AGENT-UI-REMEDIATION.md`: Closure receipt for the Agent UI Integration (Day 1 - Day 8) code-level E2E pass, documenting triggers, state trace, approval bridge, and residual manual smoke blockers.
- `docs/debt/DEBT-AGENT-LOOP-LLM-NO-OP.md`: **P0** — [RESOLVED IN DAY 4/5] Fully resolved via Arc-Mutex ToolRegistry desktop main injection (Day 4) and LLM Bootstrap mechanism (Day 5).
- `docs/debt/DEBT-AGENT-CHINESE-I18N.md`: **P1** — [RESOLVED IN DAY 22] Completely resolved by removing brittle rule-based intent tamperings and allowing Raw User Intent direct-to-LLM streaming.
- `docs/debt/DEBT-AGENT-LLM-NATIVE-MIGRATION.md`: **P0/P1** — [RESOLVED IN DAY 24] Grand technical debt closure report, archiving both loop no-op and i18n intent tampering issues and detailing residual micro-debts.
- `docs/debt/DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG.md`: **P0** — [CODE-LEVEL FIXED + RELEASE PACKAGED / PENDING-REAL-WEBVIEW-SMOKE] Real-machine `/agent` hang diagnosis addressed in code by read-only tool approval classification, approval timeout cleanup, single approval point, native trace forwarding, and a rebuilt Tauri release package.


## Archived In Latest Pass

Moved to `archive/05/debt-history`:

- `02-slash-command-palette.md`
- `DEBT-B16-SLASH-SAFETY-REMEDIATION.md`
- `ACTIVE-DEBT-STATUS-2026-05-17-B16-D06-SUGGESTED.md`
- `DEBT-AGENT-PROMPT-001.md`
- `DEBT-B18-SECURITY-HARDENING-CLOSURE.md`

## Archive

Cleared, inactive, or superseded debt documents are stored in:

- `archive/05/debt-history`

Notable archived groups:

- Token/context tracking history.
- Agent Core historical debt declarations.
- Signaling PSK inactive debt record.
- Slash Palette V1 and B16 Safety Gate receipts.
- Agent Prompt baseline debt superseded by persona/tool-manifest/golden work.
- Day 1-15 debt-remediation receipts.
- DOM/CSP/file-ops/checkpoint/frontend-module verification receipts.
- Superseded 2026-05-15 current-status snapshot.

## Maintenance Rule

When a debt changes status, update `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md` first. If the debt is truly cleared or superseded, move the old debt or receipt document to `archive/05/debt-history` and record the archive reason in the active summary.
