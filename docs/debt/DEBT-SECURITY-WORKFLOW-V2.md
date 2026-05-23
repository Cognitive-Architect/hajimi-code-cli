# DEBT-SECURITY-WORKFLOW-V2

> Status: AGENT CORE WORKFLOW CORE / REPORT-ONLY / PARTIAL.
> Updated: 2026-05-23.
> Work items: B-17/07 Agent Core Security Workflow DTO + Orchestrator Skeleton; B-17/08 Security Workflow V2 Core Implementation.

## Scope

V2 starts the Intelligence-layer security workflow contract. Day 7 added DTOs
and a minimal report assembly orchestrator in Agent Core only. Day 8 implements
the core report-only workflow branches without connecting ToolRegistry execution.

| Item | Status | Evidence |
|---|---:|---|
| Module path | `DONE` | `src/intelligence/agent-core/security_workflow.rs` |
| Module registration | `DONE` | `src/intelligence/agent-core/lib.rs` exports `security_workflow` |
| Request DTO | `DONE` | `SecurityWorkflowRequest` includes `kind`, `scope`, `dry_run`, `max_findings`, and `findings` |
| Report DTO | `DONE` | `SecurityWorkflowReport` includes summary, findings, validation receipts, and residual risk |
| Orchestrator skeleton | `DONE / SKELETON` | `SecurityWorkflowOrchestrator::run` assembles report data from in-memory findings |
| Security scan branch | `DONE / REPORT-ONLY` | `security_scan` consumes supplied local findings |
| Threat model branch | `DONE / REPORT-ONLY` | `threat_model` derives assets, entry points, trust boundaries, and assumptions from supplied scope/evidence |
| Finding discovery branch | `DONE / REPORT-ONLY` | `finding_discovery` reruns status classification |
| Attack path branch | `DONE / REPORT-ONLY` | `attack_path_analysis` writes human-readable narrative only |
| Validation branch | `DONE / REPORT-ONLY` | `validation` creates `pending` or `not_run` receipts from a local allowlist and does not execute commands |
| Feature gate | `DONE / DEFAULT OFF` | `HAJIMI_SECURITY_WORKFLOW_ENABLED` defaults disabled via `prompts::is_security_workflow_enabled()` |
| Agent workflow execution | `PARTIAL` | ToolRegistry integration is not implemented in Day 8 |
| UI integration | `PENDING` | No interface layer files are changed |
| Fix application | `PENDING / DRY-RUN ONLY` | `dry_run` is part of the contract; Day 7 does not apply fixes |

## Command Evidence

| Check | Command | Result |
|---|---|---|
| Agent Core check | `cargo check -p intelligence-agent-core` | exit 0 |
| Agent Core tests | `cargo test -p intelligence-agent-core security_workflow` | exit 0; 8 `security_workflow` tests passed |
| Feature-gate off tests | `$env:HAJIMI_SECURITY_WORKFLOW_ENABLED="false"; cargo test -p intelligence-agent-core security_workflow` | exit 0; 8 `security_workflow` tests passed |
| DTO/workflow search | `rg -n "security_scan|threat_model|finding_discovery|attack_path_analysis|validation|HAJIMI_SECURITY_WORKFLOW_ENABLED" src/intelligence/agent-core docs/debt` | Workflow branches and feature gate found |
| Layer boundary | `rg -n "interface|desktop|web" src/intelligence/agent-core/security_workflow.rs` | exit 1 expected; no upper-layer dependency names matched |
| Diff check | `git diff --check -- src/intelligence/agent-core docs/security docs/debt` | exit 0 |

## Safety Boundary

- 不接 Tauri command。
- 不接 UI。
- 不执行真实 exploit。
- attack path 只输出人类可读叙述。
- 不做外部联网扫描。
- 不自动应用 fix。
- 无 evidence 的 finding 不得保持 `confirmed`。
- 未复测的 finding 不得标记 `fixed`。

## Residual Risk

| Debt ID | Status | Notes |
|---|---:|---|
| DEBT-WORKFLOW-SKELETON-B17-07 | `PARTIAL` | DTO and report assembly skeleton exist; real workflow execution waits for later V2 tasks. |
| DEBT-INTEGRATION-B17-08 | `PARTIAL` | ToolRegistry is not connected; V2 currently accepts supplied gate/report findings and emits report-only receipts. |
| PENDING-WORKFLOW-EXECUTION | `PENDING` | `SecurityWorkflowOrchestrator::run` does not call tools or record executed validation commands. |
| PENDING-WEBVIEW-SMOKE | `PENDING` | Real WebView/manual click validation remains unexecuted per task instruction. |

## Rollback

Remove `src/intelligence/agent-core/security_workflow.rs`, remove the
`security_workflow` module registration from `src/intelligence/agent-core/lib.rs`,
and keep the existing V1 gate/report artifacts unchanged.
