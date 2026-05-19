# Hajimi Active Debt Status - 2026-05-17

> Branch: `v3.8.0-batch-1`
> HEAD: `add4d46`
> Scope: `docs/debt` debt triage after B16 Slash Palette + Safety Gate.
> Archive target: `archive/05/debt-history`

## 1. Conclusion

This document is the current source of truth for `docs/debt`.

After rechecking the debt documents against the current source, the old slash palette debt and the old Agent Prompt P0 baseline are no longer accurate active debt declarations. They have been archived with the B16 receipt. The remaining debt falls into two groups:

1. **Manual Tauri/WebView verification debt**: implementation appears present, but a real desktop window run is still required before closure.
2. **Non-manual active debt**: design or implementation work remains beyond real-machine verification.

Excluding manual real-machine verification debt, the still-unhandled active debt is:

- `AD-001`: complex shell feature restoration remains intentionally deferred.
- `AD-002`: `withGlobalTauri: true` and broad `window.__TAURI__` usage remain.
- `AD-004`: frontend modularization is improved but still partial.
- `AD-005`: Thinking UI/checkpoint depth still has non-WebView gaps.
- `AD-006`: Agent Prompt productization is improved but not fully productized.
- `AD-008`: Security Audit Gate V1 exists, but it is still lightweight fixed-pattern coverage.

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
| AD-002 Tauri global API migration | `PARTIAL/VERIFY` | P1 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | CSP is no longer `null`, but `withGlobalTauri: true` remains and the frontend still contains many direct `window.__TAURI__` call sites. Closing this requires a wrapper migration plus WebView smoke. |
| AD-003 Tauri GUI/WebView smoke blocker | `ACTIVE BLOCKED / MANUAL` | P1 | `docs/debt/DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md`; `docs/debt/DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md`; `docs/debt/DEBT-UX-AGENT-001.md` | Startup, file tree, sessions, malicious DOM samples, file-operation clicks, and slash palette interaction need real Tauri window evidence. This group is excluded from the "non-manual unhandled" count. |
| AD-004 Frontend modularization | `PARTIAL/IMPROVED` | P2 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | `security-dom`, `workspace`, `sessions`, `thinking-ui`, and `slash-palette` modules exist. Broader `app.js`, provider/settings, command palette, and `style.css` decomposition remain active work. |
| AD-005 Thinking UI and checkpoint depth | `PARTIAL/VERIFY` | P1/P2 | `docs/debt/DEBT-THINKING-UI.md` | V1 trace/checkpoint work exists. Still active beyond WebView smoke: richer file snapshots, full git/file diff evidence, transaction-log restore, malformed thinking stream tests, and provider-token integration. |
| AD-006 Agent Prompt productization | `PARTIAL/IMPROVED` | P2 | active summary; archived `archive/05/debt-history/DEBT-AGENT-PROMPT-001.md` | Agent Persona, context window manager, tool manifest, DTO/contracts, and prompt golden tests exist. Remaining work is productization: live runtime consistency, broader policy integration, and product scoring beyond deterministic golden cases. |
| AD-007 Slash command suggestion panel | `IMPLEMENTED/PENDING-UI-SMOKE` | P1/P2 | archived `archive/05/debt-history/02-slash-command-palette.md`; archived B16 receipt | Slash Palette V1 is implemented and Node-smoked. The old "panel missing" debt is archived. Only real Tauri/WebView interaction evidence remains before final UI closure. |
| AD-008 SecurityAuditTool quality | `PARTIAL/GATED` | P2 | archived B16 receipt; `tests/security/security_audit_gate.js` | Security Audit Gate V1 exists and passes. Remaining work is stronger audit quality: narrower allowlist precision, broader sink coverage, and eventual full security-audit policy beyond fixed-pattern checks. |

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

Previously archived cleared, inactive, or superseded documents remain in `archive/05/debt-history`.

## 6. Remaining Top-Level Debt Documents

The root `docs/debt` directory intentionally keeps only active debt declarations plus this index layer:

```text
DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md
DEBT-P0-UI-INTERACTION-REMEDIATION.md
DEBT-THINKING-UI.md
DEBT-UX-AGENT-001.md
DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md
SHELL-FEATURE-DEBT-002.md
```

## 7. Closure Rule

Do not close AD-002, AD-003, AD-005, or AD-007 based on static checks or Node smoke alone. Anything that claims WebView behavior must include a real Tauri/WebView run with logs, screenshots, or equivalent durable evidence.
