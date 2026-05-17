# Hajimi Active Debt Status - 2026-05-17

> Branch: `v3.8.0-batch-1`
> HEAD: `ece6cd9`
> Scope: `docs/debt` debt triage, cleared-debt archive, active-debt summary.
> Archive target: `archive/05/debt-history`

## 1. Conclusion

This document supersedes the previous top-level debt snapshot in `docs/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`.

Confirmed cleared, inactive, or superseded debt documents have been moved to `archive/05/debt-history`. Current active debt is now limited to the items below. The key rule used here: if a debt still lacks GUI/WebView evidence, keeps a deliberate product limitation, or has only a partial implementation, it remains active even when the implementation work has useful receipts.

## 2. Verification Performed

| Check | Result |
|---|---|
| `rg "ALLOWED_COMMANDS\|bash\|pwsh\|powershell" src/engine/tool-system/src/shell.rs` | User allow-list no longer contains `bash`, `sh`, `pwsh`, or `powershell`; executor wrappers still use platform shells internally. |
| `cargo test -p engine-tool-system -- test_allow_list` | Passed: `1 passed`; existing unused-import warning remains. |
| `rg "resolve_workspace_path\|create_dir\|rename_path\|delete_path" src/interface/desktop/src/main.rs` | Secure resolver and dedicated file commands exist and are registered. |
| `rg "withGlobalTauri\|csp" src/interface/desktop/tauri.conf.json` | CSP baseline exists; `withGlobalTauri` is still `true`. |
| `rg "WebRTC\|signaling\|PSK\|KMS\|Vault" src Cargo.toml package.json` | No active signaling server or PSK runtime found in current product path. |
| `node --check src/interface/web/app.js` | Passed. |
| `node tests/frontend/day13_workspace_modules_smoke.js` | Passed: `day13 workspace/security modules smoke: PASS`. |
| `node tests/frontend/day14_sessions_thinking_modules_smoke.js` | Passed: `day14 sessions/thinking modules smoke: PASS`. |

## 3. Active Debt Matrix

| ID | Status | Priority | Source document | Current truth |
|---|---|---:|---|---|
| AD-001 Shell feature downgrade | `OPEN BY DESIGN` | P2 | `docs/debt/SHELL-FEATURE-DEBT-002.md` | Complex shell features such as pipes, redirects, variables, and subshells remain disabled. This is intentional until sandbox, cwd, env, network, audit, timeout, and approval controls are designed. |
| AD-002 Tauri global API migration | `PARTIAL/VERIFY` | P1 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | CSP is no longer `null`, but `withGlobalTauri: true` remains because the frontend still uses `window.__TAURI__` heavily. Closing this requires a `tauri-api` wrapper migration and WebView smoke. |
| AD-003 Tauri GUI/WebView smoke blocker | `ACTIVE BLOCKED` | P1 | `docs/debt/DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md`; `docs/debt/DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md`; `docs/debt/DEBT-UX-AGENT-001.md` | Startup, file tree, sessions, malicious DOM samples, and file-operation clicks still need real Tauri window verification. Node smoke passed, but it does not replace WebView evidence. |
| AD-004 Frontend modularization | `PARTIAL/P2` | P2 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | `security-dom`, `workspace`, `sessions`, and `thinking-ui` modules exist. Command/slash palette, provider/settings, broader `app.js`, and `style.css` remain monolithic. |
| AD-005 Thinking UI and checkpoint depth | `PARTIAL/VERIFY` | P1/P2 | `docs/debt/DEBT-THINKING-UI.md` | Trace and checkpoint export/compare/restore/replay V1 exist. Still active: WebView smoke, richer file snapshots, full git/file diff evidence, transaction-log restore, and stronger malformed-thinking-stream tests. |
| AD-006 Agent Prompt productization | `PARTIAL/P2` | P2 | `docs/debt/DEBT-AGENT-PROMPT-001.md` | Persona/contracts/golden regression exist, but live ToolRegistry-backed manifest scoring, full runtime prompt consistency, and broader product-grade prompt policy remain future work. |
| AD-007 Slash command suggestion panel | `OPEN` | P1/P2 | `docs/debt/02-slash-command-palette.md` | Slash commands are parsed, but the `/` input hint panel itself is still missing. |
| AD-008 SecurityAuditTool quality | `OPEN` | P2 | Superseded security workflow archived to `archive/05/debt-history/hajimi_codex_security_workflow.md` | The original `CS-HAJIMI-005` remains a product-quality debt: security audit coverage is still lightweight and should become a stronger automated gate. |

## 4. Archived Documents

Moved to `archive/05/debt-history`:

| Document | Archive reason |
|---|---|
| `01-token-context-usage-tracking.md` | Token/context usage tracking is implemented and only needs ordinary regression coverage. |
| `DEBT-ACTIVE-DECLARATION.md` | Historical Agent Core debt declaration; no longer the current source of truth. |
| `DEBT-P0-001.md` | Signaling PSK debt is inactive in current source: no active signaling server or PSK runtime found. |
| `DEBT-REMEDIATION-CLOSURE-2026-05-17.md` | Closure receipt superseded by this active status summary. |
| `DEBT-REWORK-001-声明.md` | Historical rework declaration. |
| `DEBT-SCHEME-B.md` | Scheme B token tracking record is historical/cleared. |
| `DEBT-THINKING-UI-BASELINE.md` | Baseline record superseded by the active Thinking UI debt. |
| `FILE-OPS-DEDICATED-COMMANDS-VERIFY.md` | Dedicated file-op implementation receipt archived; GUI smoke remains active under AD-003. |
| `FRONTEND-MODULES-B13-RECEIPT.md` | Day 13 module receipt archived; residual modularization remains active under AD-004. |
| `FRONTEND-MODULES-B14-RECEIPT.md` | Day 14 module receipt archived; residual modularization remains active under AD-004. |
| `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | Superseded status snapshot. |
| `SECURITY-CSP-VERIFY.md` | CSP baseline receipt archived; global API migration remains active under AD-002. |
| `SECURITY-DOM-AUDIT.md` | DOM escape audit receipt archived; real WebView malicious-sample smoke remains active under AD-003. |
| `THINKING-CHECKPOINT-PLAN.md` | Checkpoint model plan archived as V1 implementation history. |
| `THINKING-CHECKPOINT-VERIFY.md` | Export/compare V1 receipt archived. |
| `THINKING-RESTORE-REPLAY-VERIFY.md` | Restore/replay V1 receipt archived. |
| `hajimi_codex_security_workflow.md` | Superseded security review workflow; remaining `CS-HAJIMI-005` is represented by AD-008. |

## 5. Remaining Top-Level Debt Documents

The root `docs/debt` directory intentionally keeps only active debt declarations plus this index layer:

```text
02-slash-command-palette.md
DEBT-AGENT-PROMPT-001.md
DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md
DEBT-P0-UI-INTERACTION-REMEDIATION.md
DEBT-THINKING-UI.md
DEBT-UX-AGENT-001.md
DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md
SHELL-FEATURE-DEBT-002.md
```

## 6. Closure Rule

Do not close AD-002, AD-003, or AD-005 based on static or Node smoke alone. They need at least one successful real Tauri/WebView verification run with logs or screenshots recorded in a debt receipt.
