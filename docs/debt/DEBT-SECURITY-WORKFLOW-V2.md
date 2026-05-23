# DEBT-SECURITY-WORKFLOW-V2

> Status: AGENT CORE DTO SKELETON / PARTIAL.
> Updated: 2026-05-23.
> Work item: B-17/07 Agent Core Security Workflow DTO + Orchestrator Skeleton.

## Scope

V2 starts the Intelligence-layer security workflow contract. Day 7 adds DTOs
and a minimal report assembly orchestrator in Agent Core only.

| Item | Status | Evidence |
|---|---:|---|
| Module path | `DONE` | `src/intelligence/agent-core/security_workflow.rs` |
| Module registration | `DONE` | `src/intelligence/agent-core/lib.rs` exports `security_workflow` |
| Request DTO | `DONE` | `SecurityWorkflowRequest` includes `kind`, `scope`, `dry_run`, `max_findings`, and `findings` |
| Report DTO | `DONE` | `SecurityWorkflowReport` includes summary, findings, validation receipts, and residual risk |
| Orchestrator skeleton | `DONE / SKELETON` | `SecurityWorkflowOrchestrator::run` assembles report data from in-memory findings |
| Agent workflow execution | `PENDING` | No tool invocation or workflow state machine is implemented in Day 7 |
| UI integration | `PENDING` | No interface layer files are changed |
| Fix application | `PENDING / DRY-RUN ONLY` | `dry_run` is part of the contract; Day 7 does not apply fixes |

## Command Evidence

| Check | Command | Result |
|---|---|---|
| Agent Core check | `cargo check -p intelligence-agent-core` | exit 0 |
| Agent Core tests | `cargo test -p intelligence-agent-core security_workflow` | exit 0; 4 `security_workflow` tests passed |
| DTO search | `rg -n "SecurityWorkflowKind|SecurityWorkflowRequest|SecurityWorkflowReport|SecurityWorkflowOrchestrator" src/intelligence/agent-core` | DTOs and orchestrator found in `security_workflow.rs` |
| Layer boundary | `rg -n "interface|desktop|web" src/intelligence/agent-core/security_workflow.rs` | exit 1 expected; no upper-layer dependency names matched |

## Safety Boundary

- 不接 Tauri command。
- 不接 UI。
- 不执行真实攻击路径。
- 不做外部联网扫描。
- 不自动应用 fix。
- 无 evidence 的 finding 不得保持 `confirmed`。
- 未复测的 finding 不得标记 `fixed`。

## Residual Risk

| Debt ID | Status | Notes |
|---|---:|---|
| DEBT-WORKFLOW-SKELETON-B17-07 | `PARTIAL` | DTO and report assembly skeleton exist; real workflow execution waits for later V2 tasks. |
| PENDING-WORKFLOW-EXECUTION | `PENDING` | `SecurityWorkflowOrchestrator::run` does not call tools or record new validation commands. |
| PENDING-WEBVIEW-SMOKE | `PENDING` | Real WebView/manual click validation remains unexecuted per task instruction. |

## Rollback

Remove `src/intelligence/agent-core/security_workflow.rs`, remove the
`security_workflow` module registration from `src/intelligence/agent-core/lib.rs`,
and keep the existing V1 gate/report artifacts unchanged.
