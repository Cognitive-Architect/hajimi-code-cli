# Hajimi Security Workflow Spec

> Status: Day 1 baseline skeleton for B-17/01.
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
| Existing security tool | `EXISTS` | `src/engine/tool-system/src/security.rs:11` `SecurityAuditTool`; `security.rs:39` `Finding`; `security.rs:56` `AuditResult`; `security.rs:188` tool name `security_audit` |
| Existing security gate | `EXISTS` | `tests/security/security_audit_gate.js`; `package.json:12` `test:security-gate` |
| Existing CI hook | `EXISTS` | `.github/workflows/security.yml:25` runs `npm run test:security-gate` |
| B18 security closure | `EXISTS` | `docs/debt/DEBT-B18-SECURITY-HARDENING-CLOSURE.md:9` closes `withGlobalTauri=true`; line 11 closes naked `run_command` |
| Tauri global API | `CLOSED BASELINE` | `src/interface/desktop/tauri.conf.json:13` has `withGlobalTauri=false` |
| Tauri CSP | `CONFIGURED` | `src/interface/desktop/tauri.conf.json:25` has non-null CSP string |
| Legacy DOM HTML API | `WARN BASELINE` | `npm run test:security-gate` reported 108 allowlisted warnings and 0 failures |
| Shell allow-list | `HARDENED / NEEDS V1 RULE IDS` | `src/engine/tool-system/src/shell.rs:21` `ALLOWED_COMMANDS`; tests at lines 332-333 reject `bash` / `sh` as user command |

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

| Status | Meaning | Day 1 Rule |
|---|---|---|
| `candidate` | Static signal exists, not yet validated | Allowed only with file/line evidence |
| `unverified` | Evidence exists but no validation receipt | Default for inferred findings |
| `confirmed` | Evidence plus validation receipt confirms issue | Not produced by Day 1 |
| `fixed` | Fix applied and revalidated | Not produced by Day 1 |
| `accepted_risk` | Human accepted residual risk | Requires reason |
| `false_positive` | Human or deterministic rule rejected finding | Requires reason |

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

## Validation Receipt Rules

Validation receipts must record real command execution. A command not run must be
marked `not_run` or `pending`, never `pass`.

```json
{
  "command": "npm run test:security-gate",
  "exit_code": 0,
  "stdout_summary": "Security Audit Gate V1 summary: failures 0, warnings 108, PASS",
  "stderr_summary": "",
  "status": "pass"
}
```

## Day 1 Debt

- Real WebView click validation is `PENDING-WEBVIEW-SMOKE`; no manual click path
  was executed for this baseline.
- `SecurityAuditTool` output is still lightweight and needs Day 5 schema work.
- Legacy DOM warning count is real gate output, but individual warnings remain
  allowlisted debt until safe rendering migration is scheduled.
