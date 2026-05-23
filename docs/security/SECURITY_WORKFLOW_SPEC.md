# Hajimi Security Workflow Spec

> Status: Day 7 Agent Core DTO skeleton implemented for B-17/07.
> Updated: 2026-05-23.
> Scope: local repository security review workflow only.

## Purpose

Security Workflow turns the existing Hajimi security gate and `security_audit`
tool into a staged, evidence-first workflow. V1 records and strengthens local
gate evidence, V2 will add Agent workflow reporting, and V3 will add dry-run
fix planning plus revalidation receipts.

This is not a complete SAST platform or an automated attack system.

## Current Baseline

| Item | Status | Evidence |
|---|---:|---|
| Execution branch | `codex/security-workflow-day01` | `git switch -c codex/security-workflow-day01` succeeded after initial baseline refresh |
| Baseline HEAD | `b0e2b5bff326b4bd4530f3d176f855efd62fb1e1` | `git rev-parse HEAD` |
| Existing security tool | `EXISTS / STRUCTURED SCHEMA` | `src/engine/tool-system/src/security.rs:11` `SecurityAuditTool`; `FindingStatus`; `Evidence`; `Finding`; `AuditResult`; tool name `security_audit` |
| Existing security gate | `EXISTS` | `tests/security/security_audit_gate.js`; `package.json:12` `test:security-gate` |
| Existing CI hook | `EXISTS` | `.github/workflows/security.yml:25` runs `npm run test:security-gate` |
| B18 security closure | `EXISTS` | `docs/debt/DEBT-B18-SECURITY-HARDENING-CLOSURE.md:9` closes `withGlobalTauri=true`; line 11 closes naked `run_command` |
| Tauri global API | `CLOSED BASELINE` | `src/interface/desktop/tauri.conf.json:13` has `withGlobalTauri=false` |
| Tauri CSP | `CONFIGURED` | `src/interface/desktop/tauri.conf.json:25` has non-null CSP string |
| Legacy DOM HTML API | `WARN BASELINE` | `npm run test:security-gate` reported 108 allowlisted warnings and 0 failures |
| Shell allow-list | `HARDENED / NEEDS V1 RULE IDS` | `src/engine/tool-system/src/shell.rs:21` `ALLOWED_COMMANDS`; tests at lines 332-333 reject `bash` / `sh` as user command |
| Day 5 Rust schema | `IMPLEMENTED / PARTIAL SCAN` | `src/engine/tool-system/src/security.rs` preserves old fields `severity`, `type`, `file`, `line`, `snippet` and adds `rule_id`, `category`, `status`, `evidence`, `recommendation`, `regression_test`, and `confidence` |
| Day 7 Agent Core DTO | `IMPLEMENTED / SKELETON ONLY` | `src/intelligence/agent-core/security_workflow.rs` defines `SecurityWorkflowRequest`, `SecurityWorkflowReport`, `SecurityWorkflowKind`, `SecurityFinding`, `Evidence`, `ValidationReceipt`, and `SecurityWorkflowOrchestrator` |

## V1 Scope

- Keep and document the existing `tests/security/security_audit_gate.js`.
- Keep and document `npm run test:security-gate`.
- Record current B18 anti-regression rules as hard security boundaries.
- Record legacy DOM `innerHTML` usage as allowlisted warning debt, not as cleared risk.
- Define the minimal evidence vocabulary needed by Day 2+ schema work.

## V2 Scope

- Add an Agent Core workflow contract for `security_scan`, `threat_model`,
  `finding_discovery`, `attack_path_analysis`, and `validation`.
- Keep Intelligence layer independent from Interface layer.
- Generate reports with evidence, confidence, status, validation receipts, and
  residual risk.

## V3 Scope

- Generate fix plans in dry-run mode by default.
- Require human review for high and critical findings.
- Require revalidation receipts before any fix can be marked fixed.
- Keep rollback instructions attached to every patch plan.

## Non-Scope

- 不自动执行真实 exploit。
- 不做外部联网扫描。
- 不恢复复杂 shell，也不通过 `bash -c` 绕过安全边界。
- 不把未验证 finding 标记为 `confirmed`。
- 不把未复测 fix 标记为 `fixed`。
- 不把 Security Workflow V1 宣称为完整 SAST。
- 不在 Day 1 修改 `src/` 业务逻辑或新增 gate 规则。

## Finding Status

| Status | Meaning | Required evidence |
|---|---|---|
| `candidate` | Static signal exists, not yet validated | At least one `Evidence` item with `file` and `line` when code-backed |
| `unverified` | Evidence exists but no validation receipt confirms exploitability or policy breach | Default for inferred findings |
| `confirmed` | Evidence plus validation receipt confirms issue | Requires at least one validation receipt with `status=pass` for the check that proves the finding |
| `fixed` | Fix applied and revalidated | Requires a revalidation receipt; 没有 revalidation / 未复测不得标记为 `fixed` |
| `accepted_risk` | Human accepted residual risk | Requires `accepted_by`, `reason`, and review date in report metadata |
| `false_positive` | Human or deterministic rule rejected finding | Requires `reason` and evidence that explains rejection |

无 evidence / 没有 evidence 的 finding 不得标记为 `confirmed`。No evidence also caps `confidence` at `0.4`.

## Severity

| Severity | Meaning | Automation boundary |
|---|---|---|
| `low` | Localized issue with limited impact | May receive dry-run fix plan |
| `medium` | Meaningful security weakness or policy drift | Dry-run fix plan; human review recommended |
| `high` | Possible credential, execution, trust-boundary, or data-loss issue | `human_review_required=true`; no automatic apply |
| `critical` | Clear secret exposure, arbitrary code execution, or destructive path | `human_review_required=true`; no automatic apply |

## Category

Initial categories use snake_case:

- `secret`
- `panic_safety`
- `tauri_security`
- `shell_safety`
- `frontend_xss`
- `workspace_fs`
- `supply_chain`
- `governance`
- `unknown`

## Confidence

`confidence` is a number from `0.0-1.0`.

| Range | Meaning |
|---|---|
| `0.0-0.4` | Weak signal, missing direct evidence, or broad heuristic |
| `0.41-0.7` | Code evidence exists, but validation receipt is missing |
| `0.71-0.9` | Evidence plus local validation strongly support the finding |
| `0.91-1.0` | Deterministic rule with precise evidence and reproducible validation |

Rules:

- No evidence means `confidence <= 0.4`.
- A finding without validation should normally remain `candidate` or `unverified`.
- `confirmed` should normally require `confidence >= 0.71`.
- `accepted_risk` and `false_positive` still keep their original confidence for audit history.

## Finding Schema

All JSON-facing fields use `snake_case`. Day 5 Rust implementation should keep
the old `security_audit` fields for compatibility while adding these fields.

```json
{
  "finding_id": "FINDING-001",
  "rule_id": "DOM-HTML-001",
  "title": "Unsafe HTML rendering candidate",
  "severity": "medium",
  "category": "frontend_xss",
  "status": "unverified",
  "confidence": 0.55,
  "type": "UnsafeHtmlApi",
  "file": "src/interface/web/app.js",
  "line": 265,
  "snippet": "statusEl.innerHTML = ...",
  "evidence": [],
  "attack_path": "Human-readable narrative only; 不输出可执行 exploit.",
  "recommendation": "Use textContent or the safe DOM helper for untrusted text.",
  "regression_test": "npm run test:security-gate",
  "human_review_required": false,
  "validation_receipts": [],
  "residual_risk": []
}
```

Required fields:

| Field | Type | Required | Notes |
|---|---|---:|---|
| `finding_id` | string | yes | Stable report-local ID |
| `rule_id` | string | yes | Rule or detector ID |
| `title` | string | yes | Human-readable short title |
| `severity` | enum | yes | `low`, `medium`, `high`, `critical` |
| `category` | enum/string | yes | Use known category when possible |
| `status` | enum | yes | Finding status state machine |
| `confidence` | number | yes | `0.0-1.0` |
| `type` | string | yes | 保留旧字段兼容; Rust may keep `type_` with serde rename |
| `file` | string | yes for code findings | 保留旧字段兼容 |
| `line` | number | yes for code findings | 保留旧字段兼容 |
| `snippet` | string | yes for code findings | 保留旧字段兼容; redact secrets |
| `evidence` | Evidence[] | yes | Can be empty only for `candidate`; caps confidence |
| `attack_path` | string | no | Human-readable narrative only |
| `recommendation` | string | yes | Concrete mitigation |
| `regression_test` | string/null | no | Local command, no external scan |
| `human_review_required` | boolean | yes | Required for high/critical and dangerous fix |
| `validation_receipts` | ValidationReceipt[] | yes | Empty until validation runs |
| `residual_risk` | string[] | no | Known gaps |

Compatibility rule: Day 5 Rust work must preserve old fields `severity`, `type`,
`file`, `line`, and `snippet` in `security_audit` output. New fields should be
additive; no new crate is required by this contract.

## SecurityAuditTool Rust Output

B-17/05 implements the additive Rust `security_audit` finding schema in
`src/engine/tool-system/src/security.rs` while keeping the tool name and old
JSON fields stable.

Implemented DTOs:

- `FindingStatus`
- `Evidence`
- `ValidationReceipt`
- `Finding`
- `AuditResult`

Implemented finding fields:

| Field | Status | Notes |
|---|---:|---|
| `severity` | `preserved` | Existing lower-case severity output remains. |
| `type` | `preserved` | Rust keeps `type_` with `#[serde(rename = "type")]`. |
| `file` | `preserved` | Existing file path output remains. |
| `line` | `preserved` | Existing 1-based line output remains. |
| `snippet` | `preserved` | Secret-like snippets are redacted/truncated. |
| `finding_id` | `added` | Report-local ID derived from rule/file/line. |
| `rule_id` | `added` | Examples: `SECRET-AWS-001`, `PANIC-UNWRAP-001`, `PANIC-TODO-001`. |
| `title` | `added` | Short human-readable finding title. |
| `category` | `added` | Current categories: `secret`, `panic_safety`; `unknown` reserved. |
| `status` | `added` | Static findings default to `unverified`, not `confirmed`. |
| `confidence` | `added` | Constructed through a `0.0-1.0` clamp. |
| `evidence` | `added` | Code evidence includes file, line, snippet, and note. |
| `recommendation` | `added` | Mitigation guidance per detector family. |
| `regression_test` | `added` | Currently points to `cargo test -p engine-tool-system security`. |
| `human_review_required` | `added` | True for high/critical findings. |
| `validation_receipts` | `added` | Empty until a validation workflow runs. |
| `residual_risk` | `added` | Empty until report-level risk review runs. |

Current detector families covered by B-17/05:

- `secret`: AWS key, GitHub token, Stripe live key, private key patterns.
- `panic_safety`: `todo!`, `.unwrap()`, and `panic!`.

This remains a lightweight local scanner, not a complete SAST engine. Additional
categories from the Day 2 contract remain reserved for later workflow phases.

## Evidence Schema

`Evidence` is intentionally small enough for Rust, JS gate, Agent Core, and UI
to share.

```json
{
  "kind": "code",
  "file": "src/interface/web/app.js",
  "line": 265,
  "snippet": "statusEl.innerHTML = ...",
  "command": "rg -n \"innerHTML\" src/interface/web/app.js",
  "output_hash": "sha256:<optional>",
  "note": "Legacy allowlisted DOM rendering warning from current gate baseline"
}
```

Required minimum support:

| Field | Type | Required | Notes |
|---|---|---:|---|
| `kind` | string | yes | `code`, `command`, `config`, `receipt`, `human` |
| `file` | string/null | no | Repository-relative path |
| `line` | number/null | no | 1-based line |
| `snippet` | string/null | no | Redacted when needed |
| `command` | string/null | no | Real local command if command-backed |
| `output_hash` | string/null | no | Stable hash for long output |
| `note` | string/null | no | Short explanation |

## Validation Receipt Schema

`ValidationReceipt` records local validation. A command not run must be marked
`not_run` or `pending`; never write fake `pass`.

```json
{
  "command": "npm run test:security-gate",
  "exit_code": 0,
  "stdout_summary": "Security Audit Gate V1 summary: failures 0, warnings 108, PASS",
  "stderr_summary": "",
  "status": "pass"
}
```

Required fields:

| Field | Type | Required | Notes |
|---|---|---:|---|
| `command` | string | yes | Local validation command only |
| `exit_code` | number/null | yes | `null` when `not_run` or `pending` |
| `stdout_summary` | string | yes | Short summary, not full logs |
| `stderr_summary` | string | yes | Short summary |
| `status` | enum | yes | `pass`, `fail`, `not_run`, `pending` |

## Workflow Kind

`SecurityWorkflowKind` values are lower snake_case across Rust DTOs, JS payloads,
and reports:

| Kind | Output | Forbidden behavior |
|---|---|---|
| `security_scan` | Findings and raw scan receipts | No external scan |
| `threat_model` | Assets, entry points, trust boundaries | Do not invent missing entry points |
| `finding_discovery` | Candidate or unverified findings | No evidence means no confirmed status |
| `attack_path_analysis` | Human-readable attack path narrative | 不输出可执行 exploit |
| `validation` | ValidationReceipt list | Do not run real exploit |
| `fix_finding` | Dry-run patch plan and revalidation plan | High/Critical require human review |

Day 7 DTO names should align with this contract:

- `SecurityWorkflowRequest`
- `SecurityWorkflowReport`
- `SecurityWorkflowKind`
- `SecurityFinding`
- `FindingStatus`
- `Evidence`
- `ValidationReceipt`

## Agent Core Security Workflow DTO

B-17/07 implements the Intelligence-layer DTO skeleton in
`src/intelligence/agent-core/security_workflow.rs`. The module is registered
from `src/intelligence/agent-core/lib.rs` and does not connect to the UI.

Implemented DTOs:

- `SecurityWorkflowKind`
- `SecurityScope`
- `SecurityWorkflowRequest`
- `SecurityFinding`
- `Evidence`
- `ValidationReceipt`
- `SecurityWorkflowSummary`
- `SecurityWorkflowReport`
- `SecurityWorkflowOrchestrator`

Implemented request/report fields:

| Field | Status | Notes |
|---|---:|---|
| `kind` | `added` | Uses lower snake_case workflow kinds. |
| `scope` | `added` | Repository, branch, commit, paths, and out-of-scope notes. |
| `dry_run` | `added` | Required on request and report; defaults are left to callers. |
| `max_findings` | `added` | Orchestrator caps report assembly to this limit. |
| `findings` | `added` | Uses the shared finding contract shape. |
| `confidence` | `added` | Clamped to `0.0-1.0`; no evidence caps at `0.4`. |
| `validation_receipts` | `added` | Aggregated from findings only; no command is executed here. |
| `residual_risk` | `added` | Assembled from dry-run status, missing evidence, missing receipts, and finding risks. |

The Day 7 orchestrator is a report assembly skeleton only. It does not apply
fixes, does not connect to Tauri commands, and does not run validation commands.
If a finding has no evidence but arrives as `confirmed` or `fixed`, report
assembly downgrades it to `unverified`.

## Feature Gates

| Gate | Default | Scope | Rollback |
|---|---|---|---|
| `HAJIMI_SECURITY_WORKFLOW_ENABLED` | `false` | Agent Core workflow orchestration | Fall back to existing gate/tool report |
| `HAJIMI_SECURITY_UI_ENABLED` | `false` | `/security` slash entry and Security panel | Hide UI entry |
| `HAJIMI_SECURITY_FIX_ENABLED` | `false` | Fix workflow | Report-only |
| `HAJIMI_SECURITY_FIX_DRY_RUN` | `true` | Patch planning | No apply |

## Human Review Policy

- `high` and `critical` findings set `human_review_required=true`.
- Any `fix_finding` request touching credentials, shell execution, workspace file
  deletion, trust boundaries, or network config sets `human_review_required=true`.
- High/Critical fix 默认人工确认 and must remain dry-run until explicit approval.
- `fixed` requires revalidation; 未复测 / 没有 revalidation receipt 不得标记为 `fixed`.

## Hard Fail Regressions

The following regressions must be treated as fail-level security issues, not warnings:

- `withGlobalTauri=true`
- `csp: null`
- naked Tauri `run_command` command exposure
- frontend shell bypass for file create/delete/rename/write operations
- restoring complex shell interpreters to user command allow-lists
- allowlist entries without a reason

## Evidence Rules

Every finding must carry at least one evidence item:

```json
{
  "kind": "code",
  "file": "src/interface/web/app.js",
  "line": 265,
  "snippet": "statusEl.innerHTML = ...",
  "command": "rg -n \"innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=|onload=\" src/interface/web",
  "note": "Legacy allowlisted DOM rendering warning from current gate baseline"
}
```

## Day 1 Debt

- Real WebView click validation is `PENDING-WEBVIEW-SMOKE`; no manual click path
  was executed for this baseline.
- `SecurityAuditTool` output was lightweight at Day 1; Day 5 enhanced the Rust
  schema while leaving scan coverage partial.
- Legacy DOM warning count is real gate output, but individual warnings remain
  allowlisted debt until safe rendering migration is scheduled.

## Day 2 Debt

- `DEBT-SCHEMA-B17-02`: this file defines the contract only. Rust/JS code is not
  implemented in Day 2 and must not be described as implemented.
- Real WebView click validation remains `PENDING-WEBVIEW-SMOKE`.
- Day 5 may choose serde defaults / rename for compatibility if adding fields to
  `security.rs` exposes legacy caller constraints.

## Day 5 Debt

- `DEBT-SECURITY-SCHEMA-B17-05`: Rust `SecurityAuditTool` schema is enhanced,
  but scanning remains partial and limited to secrets plus panic-safety patterns.
- `validation_receipts` are present in the DTO but remain empty until a later
  validation workflow records real command receipts.
- Real WebView click validation remains `PENDING-WEBVIEW-SMOKE`.

## Day 7 Debt

- `DEBT-WORKFLOW-SKELETON-B17-07`: Agent Core DTO and report assembly skeleton
  are implemented, but workflow execution logic is pending.
- The orchestrator only assembles local in-memory DTOs; it does not invoke
  tools, does not apply fixes, and does not produce UI state.
- Real WebView click validation remains `PENDING-WEBVIEW-SMOKE`.
