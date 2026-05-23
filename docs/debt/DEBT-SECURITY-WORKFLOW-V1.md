# DEBT-SECURITY-WORKFLOW-V1

> Status: INITIATED / CONTRACT DEFINED / IMPLEMENTATION PENDING.
> Updated: 2026-05-23.
> Work items: B-17/01 Security Workflow Baseline Audit + Docs Skeleton; B-17/02 Security Workflow Contract + Finding Schema.

## Baseline

| Item | Value |
|---|---|
| Initial branch before work branch | `codex/agent-skills-v0b` |
| Work branch | `codex/security-workflow-day01` |
| HEAD | `b0e2b5bff326b4bd4530f3d176f855efd62fb1e1` |
| Pre-existing dirty state | `.gitignore` modified; `docs/roadmap/hajimi build/Hajimi Skills/audit report/B-01-AUDIT-REPORT.md` deleted; `.codex/`, `docs/roadmap/Hajimi security workflow/`, and `target-ui-refresh/` untracked |
| Day 1 source task | `docs/roadmap/Hajimi security workflow/task/B-17-15-HAJIMI-SECURITY-WORKFLOW-Day01-Baseline-Audit-Docs.md` |
| Day 2 source task | `docs/roadmap/Hajimi security workflow/task/B-17-15-HAJIMI-SECURITY-WORKFLOW-Day02-Contract-Finding-Schema.md` |
| Plan docs read | `SECURITY-WORKFLOW-V1-V3-DAILY-PLAN.md`; `SECURITY-WORKFLOW-V1-V3-ROADMAP.md` |

## Command Evidence

| Check | Command | Result |
|---|---|---|
| Git branch | `git branch --show-current` | `codex/agent-skills-v0b` before branch creation; `codex/security-workflow-day01` after branch creation |
| Git HEAD | `git rev-parse HEAD` | `b0e2b5bff326b4bd4530f3d176f855efd62fb1e1` |
| Git status | `git status --short` | Pre-existing dirty state listed above; no `src/` file was modified for Day 1 |
| SecurityAuditTool | `rg -n "SecurityAuditTool|security_audit|Finding|AuditResult" src/engine/tool-system/src/security.rs src/engine/tool-system/src/mod.rs` | Found `SecurityAuditTool`, `Finding`, `AuditResult`, and tool name `security_audit` |
| Gate script | `rg -n "security_audit_gate|test:security-gate" tests/security package.json` | `package.json:12` points to `node tests/security/security_audit_gate.js` |
| CI | `rg -n "security\\.yml|test:security-gate" .github/workflows/security.yml package.json` | `.github/workflows/security.yml:25` runs `npm run test:security-gate` |
| B18 closure | `rg -n "withGlobalTauri=false|withGlobalTauri|run_command|Security Audit Gate V1|B-18|B18" docs/debt/DEBT-B18-SECURITY-HARDENING-CLOSURE.md tests/security/security_audit_gate.js` | B18 closure and gate fail checks found |
| Tauri config | `rg -n '"csp"|withGlobalTauri' src/interface/desktop/tauri.conf.json` | `withGlobalTauri=false`; CSP is non-null |
| DOM surface | `rg -n "innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=|onload=" src/interface/web` | 108 matches, matching current gate warning count |
| Direct Tauri global usage | `rg -n "window\\.__TAURI__|tauri\\.core|tauri\\.invoke" src/interface/web` | 0 matches |
| Shell allow-list | `rg -n "ALLOWED_COMMANDS|bash|sh|pwsh|powershell" src/engine/tool-system/src` | `shell.rs` contains allow-list and tests rejecting shell interpreters as user command payloads |
| Node syntax | `node --check tests/security/security_audit_gate.js` | exit 0 |
| Gate smoke | `npm run test:security-gate` | exit 0; `failures: 0`; `warnings: 108`; `Security Audit Gate V1: PASS` |
| Day 2 source schema baseline | `rg -n "struct Finding|severity|snippet|SecurityAuditTool" src/engine/tool-system/src/security.rs` | Current `security.rs` still has lightweight fields: `severity`, `type_`, `file`, `line`, `snippet` |

## Current Security Capability

| Capability | Status | Notes |
|---|---:|---|
| SecurityAuditTool | `EXISTS / LIGHT SCHEMA` | Current Rust output has `Finding` and `AuditResult`, but Day 5 still needs `rule_id`, `status`, `evidence`, `recommendation`, `regression_test`, and `confidence`. |
| Security Gate V1 | `EXISTS / PARTIAL` | Gate is runnable and fails on hard regressions, but Day 3-4 still need rule IDs and structured summary work. |
| Security Workflow contract | `DEFINED / NOT IMPLEMENTED` | Day 2 defines `FindingStatus`, `Evidence`, `ValidationReceipt`, workflow kinds, confidence, and feature-gate rules in docs only. |
| B18 anti-regression | `CLEARED / GATED` | `withGlobalTauri=true` and naked `run_command` are fail-level regressions. |
| Security CI | `EXISTS` | Existing workflow runs `npm run test:security-gate`. |
| DOM rendering debt | `WARN / ALLOWLISTED` | 108 current warnings are tracked as legacy dangerous HTML API usage. |
| Real WebView click validation | `PENDING-WEBVIEW-SMOKE` | User directed that real machine click portions should be recorded as debt for now. |

## Fail-Level Regressions

- `withGlobalTauri=true` must fail.
- `csp: null` must fail.
- Naked `run_command` must fail.
- Frontend file operations through shell `run_command` must fail.
- Restoring complex shell interpreters to user command allow-lists must fail.
- Allowlist entries without `reason` must fail.

## Residual Risk

| Risk | Status | Next Step |
|---|---:|---|
| SecurityAuditTool schema is lightweight | `PENDING` | Day 5: implement structured finding schema while preserving old `severity/type/file/line/snippet` fields |
| Gate warnings are not machine-report JSON yet | `PENDING` | Day 3-4: rule IDs, allowlist metadata, structured summary |
| Legacy DOM `innerHTML` debt remains | `PENDING` | Later UI safe-render migration; do not claim cleared |
| Real WebView/manual click validation absent | `PENDING-WEBVIEW-SMOKE` | Record as debt until a human or browser session validates UI paths |
| Security Workflow Agent/UI/Fix modules absent | `PLANNED` | Day 7+ per roadmap |
| DEBT-SCHEMA-B17-02 | `CONTRACT ONLY` | Contract is documented; Rust/JS implementation remains pending |

## Safety Boundary

- 不自动执行真实 exploit。
- 不做外部联网扫描。
- 不恢复复杂 shell。
- 不修改 `src/` 业务逻辑。
- 不把未验证 finding 标记为 `confirmed`。
- 不把未复测 fix 标记为 `fixed`。

## Contract Rules Added In B-17/02

- Finding status state machine: `candidate`, `unverified`, `confirmed`, `fixed`,
  `accepted_risk`, `false_positive`.
- Severity: `low`, `medium`, `high`, `critical`.
- Confidence: `0.0-1.0`; no evidence caps confidence at `0.4`.
- Evidence minimum fields: `kind`, `file`, `line`, `snippet`, `command`,
  `output_hash`, `note`.
- ValidationReceipt minimum fields: `command`, `exit_code`, `stdout_summary`,
  `stderr_summary`, `status`.
- Workflow kinds: `security_scan`, `threat_model`, `finding_discovery`,
  `attack_path_analysis`, `validation`, `fix_finding`.
- Feature gates: `HAJIMI_SECURITY_WORKFLOW_ENABLED`,
  `HAJIMI_SECURITY_UI_ENABLED`, `HAJIMI_SECURITY_FIX_ENABLED`,
  `HAJIMI_SECURITY_FIX_DRY_RUN`.
- `High` / `Critical` and dangerous fix paths require
  `human_review_required=true`.
- Attack paths remain human-readable only; no executable exploit output.

## Day 1 Closure

Day 1 initialized the security workflow spec, report template, and V1 debt
receipt using real command output. No source business logic or gate behavior was
changed.

## Day 2 Closure

Day 2 defined the shared Security Workflow contract for later V1-V3
implementation. No Rust, JavaScript, gate rule, package script, or CI behavior
was changed.
