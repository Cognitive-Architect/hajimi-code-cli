# DEBT-SECURITY-WORKFLOW-V1

> Status: INITIATED / CONTRACT DEFINED / SECURITY GATE RULES PARTIAL/GATED.
> Updated: 2026-05-23.
> Work items: B-17/01 Security Workflow Baseline Audit + Docs Skeleton; B-17/02 Security Workflow Contract + Finding Schema; B-17/03 Security Audit Gate Allowlist + JSON Summary; B-17/04 Gate Rules CSP/DOM/Shell/File Ops.

## Baseline

| Item | Value |
|---|---|
| Initial branch before work branch | `codex/agent-skills-v0b` |
| Work branch | `codex/security-workflow-day01` |
| HEAD | `b0e2b5bff326b4bd4530f3d176f855efd62fb1e1` |
| Pre-existing dirty state | `.gitignore` modified; `docs/roadmap/hajimi build/Hajimi Skills/audit report/B-01-AUDIT-REPORT.md` deleted; `.codex/`, `docs/roadmap/Hajimi security workflow/`, and `target-ui-refresh/` untracked |
| Day 1 source task | `docs/roadmap/Hajimi security workflow/task/B-17-15-HAJIMI-SECURITY-WORKFLOW-Day01-Baseline-Audit-Docs.md` |
| Day 2 source task | `docs/roadmap/Hajimi security workflow/task/B-17-15-HAJIMI-SECURITY-WORKFLOW-Day02-Contract-Finding-Schema.md` |
| Day 3 source task | `docs/roadmap/Hajimi security workflow/task/B-17-15-HAJIMI-SECURITY-WORKFLOW-Day03-Gate-Allowlist-Enhancement.md` |
| Day 4 source task | `docs/roadmap/Hajimi security workflow/task/B-17-15-HAJIMI-SECURITY-WORKFLOW-Day04-Gate-Rules-CSP-DOM-Shell-FileOps.md` |
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
| Day 3 gate syntax | `node --check tests/security/security_audit_gate.js` | exit 0 |
| Day 3 gate run | `node tests/security/security_audit_gate.js` | exit 0; `findings: 108`; `failures: 0`; `warnings: 108`; `allowlisted: 108`; JSON summary emitted |
| Day 3 npm script | `npm run test:security-gate` | exit 0; existing package script preserved |
| Day 3 allowlist evidence | `rg -n "allowlist|reason|ALLOWLIST-001|findings|summary" tests/security/security_audit_gate.js tests/security/security_audit_allowlist.json` | Structured finding collection, reason validation, and allowlist entries found |
| Day 3 status correction | Review note after `2a56b7c1` | Static gate failures default to `unverified`; deterministic `ALLOWLIST-001` policy violations are explicitly `confirmed`; allowlist string checks trim whitespace |
| Day 4 gate syntax | `node --check tests/security/security_audit_gate.js` | exit 0 |
| Day 4 gate run | `node tests/security/security_audit_gate.js` | exit 0; `findings: 108`; `failures: 0`; `warnings: 108`; `allowlisted: 108`; JSON summary emitted |
| Day 4 npm script | `npm run test:security-gate` | exit 0; existing package script preserved |
| Day 4 rule IDs | `rg -n "TAURI-CSP-001|TAURI-GLOBAL-001|DOM-INLINE-001|DOM-HTML-001|SHELL-ALLOW-001|FILE-OPS-001|ALLOWLIST-001" tests/security/security_audit_gate.js` | All seven B-17/04 rule IDs found |
| Day 4 scope check | `git diff --name-only -- src/interface/desktop/tauri.conf.json src/interface/web src/engine/tool-system/src` | empty; no business config or source code modified |

## Current Security Capability

| Capability | Status | Notes |
|---|---:|---|
| SecurityAuditTool | `EXISTS / LIGHT SCHEMA` | Current Rust output has `Finding` and `AuditResult`, but Day 5 still needs `rule_id`, `status`, `evidence`, `recommendation`, `regression_test`, and `confidence`. |
| Security Gate V1 | `EXISTS / RULES PARTIAL/GATED` | Gate is runnable, can fail on hard regressions, validates allowlist reasons, emits structured findings, prints JSON summary, and has explicit CSP/DOM/Shell/File Ops rule IDs. Coverage is still not a complete security audit. |
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
- Missing allowlist `path` or `pattern` must fail.

## Residual Risk

| Risk | Status | Next Step |
|---|---:|---|
| SecurityAuditTool schema is lightweight | `PENDING` | Day 5: implement structured finding schema while preserving old `severity/type/file/line/snippet` fields |
| Security gate coverage is still limited | `PARTIAL/GATED` | Day 4: add CSP/DOM/Shell/File Ops rule expansion without claiming full security audit coverage |
| Legacy DOM `innerHTML` debt remains | `PENDING` | Later UI safe-render migration; do not claim cleared |
| Real WebView/manual click validation absent | `PENDING-WEBVIEW-SMOKE` | Record as debt until a human or browser session validates UI paths |
| Security Workflow Agent/UI/Fix modules absent | `PLANNED` | Day 7+ per roadmap |
| DEBT-SCHEMA-B17-02 | `CONTRACT ONLY` | Contract is documented; Rust/JS implementation remains pending |
| DEBT-SECURITY-GATE-B17-03 | `PASS WITH NOTES / PARTIAL/GATED` | Gate now has allowlist reason validation, trimmed string checks, allowlisted count, and JSON summary, but still covers only the existing V1 rule set plus legacy DOM warning inventory |
| DEBT-SECURITY-GATE-B17-04 | `PARTIAL/GATED` | Explicit CSP/DOM/Shell/File Ops rule IDs now exist; this is a focused regression gate, not a full SAST engine |

## Security Gate V1 Rule Coverage

| Rule ID | Block Level | Coverage Scope | False Positive / allowlist Boundary |
|---|---:|---|---|
| `TAURI-CSP-001` | `fail` | `src/interface/desktop/tauri.conf.json` must not set CSP to `null`. | No allowlist; `csp: null` is a B18-class regression. |
| `TAURI-GLOBAL-001` | `fail` | `withGlobalTauri=true` and direct frontend global Tauri access outside the bridge. | Adapter implementation file is exempt; other hits require code change, not allowlist. |
| `DOM-INLINE-001` | `fail` | Inline HTML event handlers such as `onclick`, `onerror`, `onload`, `onmouseover`. | No historical allowlist yet; any hit should be reviewed as DOM XSS risk. |
| `DOM-HTML-001` | `warn` when allowlisted, `fail` when new/unallowlisted | `innerHTML`, `outerHTML`, and `insertAdjacentHTML` in frontend HTML/JS/CSS scan set. | Historical DOM debt is allowed only with `rule_id/path/pattern/reason`; new hits fail until rewritten or explicitly risk-accepted. |
| `SHELL-ALLOW-001` | `fail` | Complex shell interpreters, high-capability legacy desktop commands, and naked `run_command` exposure. | No allowlist; B18 cleared items must remain strong blockers. |
| `FILE-OPS-001` | `fail` | Workspace file tools must stay path-bound; inline edits must use workspace resolver; frontend file ops must not bypass dedicated commands through shell. | Regex may be conservative around `run_command`; false positives should document reason before any allowlist path is considered. |
| `ALLOWLIST-001` | `fail` | Every allowlist entry must carry non-empty trimmed `path`, `pattern`, and `reason`. | Deterministic policy violation; marked `confirmed`. |

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

## Day 3 Closure

Day 3 enhanced the existing Security Audit Gate in place. The gate now:

- collects structured findings with `rule_id`, `severity`, `status`, `file`,
  `line`, `evidence`, and `reason`;
- validates allowlist entries with `ALLOWLIST-001` so every exception must carry
  non-empty trimmed `path`, `pattern`, and `reason`;
- supports allowlist entries shaped as `rule_id`, `path`, `pattern`, `reason`,
  and optional `expires_at`;
- defaults static gate failure findings to `unverified` unless a deterministic
  policy violation explicitly sets `confirmed`;
- skips build/dependency directories including `.git`, `target`,
  `node_modules`, `dist`, and `target-ui-refresh`;
- preserves the human-readable summary and appends a JSON summary report with
  `allowlisted` count;
- preserves `npm run test:security-gate` without adding dependencies.

Real WebView/manual click validation remains `PENDING-WEBVIEW-SMOKE` per user
instruction and is not claimed as completed.

## Day 4 Closure

Day 4 upgraded the Day 3 gate from descriptive legacy rule names to explicit
B-17/04 Security Gate V1 rule IDs:

- `TAURI-CSP-001` fails `csp: null`;
- `TAURI-GLOBAL-001` fails `withGlobalTauri=true` and direct global Tauri API
  use outside the bridge;
- `DOM-INLINE-001` fails inline handler attributes;
- `DOM-HTML-001` warns only for reasoned allowlist matches and fails new
  dangerous HTML API usage;
- `SHELL-ALLOW-001` fails complex shell/high-capability command regressions and
  naked `run_command`;
- `FILE-OPS-001` fails workspace file-op bypass and missing path-bound file
  tool safeguards;
- `ALLOWLIST-001` fails malformed allowlist entries.

No Tauri config, frontend business code, engine source code, package script, or
CI workflow was changed for Day 4. Real WebView/manual click validation remains
`PENDING-WEBVIEW-SMOKE` per user instruction.
