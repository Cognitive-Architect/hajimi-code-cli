# DEBT-SECURITY-WORKFLOW-V2

> Status: DESKTOP SLASH CONTRACT / REPORT-ONLY / PARTIAL.
> Updated: 2026-05-23.
> Work items: B-17/07 Agent Core Security Workflow DTO + Orchestrator Skeleton; B-17/08 Security Workflow V2 Core Implementation; B-17/09 Desktop Command + Slash Contract.

## Scope

V2 starts the Intelligence-layer security workflow contract. Day 7 added DTOs
and a minimal report assembly orchestrator in Agent Core only. Day 8 implements
the core report-only workflow branches without connecting ToolRegistry execution.
Day 9 exposes that report-only contract through a dedicated desktop command and
vanilla `/security` slash commands. It does not add a full UI panel.

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
| Desktop command | `DONE / REPORT-ONLY` | `run_security_workflow(request)` is registered in `src/interface/desktop/src/main.rs` |
| Slash contract | `DONE / CONTRACT ONLY` | `/security scan`, `/security threat-model`, `/security validate`, `/security fix <finding-id>` are parsed by `src/interface/web/modules/security-workflow.js` |
| UI feature gate | `DONE / DEFAULT OFF` | `HAJIMI_SECURITY_UI_ENABLED` / `__HAJIMI_SECURITY_UI_ENABLED__` defaults disabled; no panel is shown by default |
| Agent workflow execution | `PARTIAL` | ToolRegistry integration is not implemented in Day 8 |
| UI integration | `PARTIAL / CONTRACT ONLY` | Slash entry exists; visual panel waits for Day 10 |
| Fix application | `PENDING / DRY-RUN ONLY` | `/security fix <finding-id>` creates a report-only request and does not apply changes |

## Command Evidence

| Check | Command | Result |
|---|---|---|
| Agent Core check | `cargo check -p intelligence-agent-core` | exit 0 |
| Agent Core tests | `cargo test -p intelligence-agent-core security_workflow` | exit 0; 8 `security_workflow` tests passed |
| Feature-gate off tests | `$env:HAJIMI_SECURITY_WORKFLOW_ENABLED="false"; cargo test -p intelligence-agent-core security_workflow` | exit 0; 8 `security_workflow` tests passed |
| Desktop check | `cargo check -p hajimi-desktop` | exit 0; existing deprecated `context_threshold` warnings only |
| Web module syntax | `node --check src/interface/web/modules/security-workflow.js` | exit 0 |
| App syntax | `node --check src/interface/web/app.js` | exit 0 |
| Slash smoke | `node tests/frontend/day17_security_workflow_smoke.js` | exit 0 |
| DTO/workflow search | `rg -n "security_scan|threat_model|finding_discovery|attack_path_analysis|validation|HAJIMI_SECURITY_WORKFLOW_ENABLED" src/intelligence/agent-core docs/debt` | Workflow branches and feature gate found |
| Layer boundary | `rg -n "interface|desktop|web" src/intelligence/agent-core/security_workflow.rs` | exit 1 expected; no upper-layer dependency names matched |
| Diff check | `git diff --check -- src/intelligence/agent-core docs/security docs/debt` | exit 0 |

## Safety Boundary

- Tauri command 使用 dedicated `run_security_workflow`，不通过通用命令工具触发。
- UI 只接 slash contract，不默认展示 panel。
- 不执行真实 exploit。
- attack path 只输出人类可读叙述。
- 不做外部联网扫描。
- 不自动应用 fix。
- `/security fix <finding-id>` 只进入 dry-run/report-only 契约。
- 无 evidence 的 finding 不得保持 `confirmed`。
- 未复测的 finding 不得标记 `fixed`。

## Residual Risk

| Debt ID | Status | Notes |
|---|---:|---|
| DEBT-WORKFLOW-SKELETON-B17-07 | `PARTIAL` | DTO and report assembly skeleton exist; real workflow execution waits for later V2 tasks. |
| DEBT-INTEGRATION-B17-08 | `PARTIAL` | ToolRegistry is not connected; V2 currently accepts supplied gate/report findings and emits report-only receipts. |
| DEBT-BRIDGE-B17-09 | `PARTIAL` | Interface now calls Agent Core DTOs directly through `run_security_workflow`, but no full UI panel or ToolRegistry execution is connected. |
| DEBT-WEBVIEW-SMOKE-B17-09 | `PENDING` | Real WebView click validation for `/security` is deferred per task instruction. |
| PENDING-WORKFLOW-EXECUTION | `PENDING` | `SecurityWorkflowOrchestrator::run` does not call tools or record executed validation commands. |
| PENDING-WEBVIEW-SMOKE | `PENDING` | Real WebView/manual click validation remains unexecuted per task instruction. |

## Rollback

Unregister `run_security_workflow` from `src/interface/desktop/src/main.rs`,
remove `src/interface/web/modules/security-workflow.js` and its script tag,
remove the `/security` branch from `src/interface/web/app.js`, and keep the
existing V1 gate/report artifacts unchanged.
