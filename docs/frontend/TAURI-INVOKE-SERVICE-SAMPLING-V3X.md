# STONE-AUDIT-V3X-DAY06 Tauri Invoke Service Sampling

日期：2026-06-16

分支：`stone-audit-v3x-controlled-demolition`

HEAD before：`d1c0869053c2da7b75a611dff19ced462d95f73b`

任务性质：docs-only sampling。本文只记录 `invokeTauri` / `getTauriInvoke` / `window.__TAURI__` / `HajimiTauri.invoke` 调用事实，不移动 invoke，不修改 bridge，不执行高风险后端命令。

人话版：Tauri invoke 就像前台递给后厨的小票。今天只把小票分类：哪些只是查菜单，哪些会改订单，哪些会让后厨开火，哪些要先别碰。

## Scope

只读范围：

- `src/interface/web/app.js`
- `src/interface/web/modules/*.js`
- `src/interface/web/controllers/*.js`
- `src/interface/web/views/*.js`
- `src/interface/web/services/tauri-service.js`
- `tests/frontend/*.js`
- `docs/frontend/*.md`

禁止范围：

- 不修改 `src/interface/web/app.js`
- 不修改 `src/interface/web/services/tauri-service.js`
- 不修改 `src/interface/web/modules/tauri-bridge.js`
- 不修改 `src/interface/web/index.html`
- 不修改 `src/interface/desktop/**`
- 不修改 `src/engine/tool-system/**`
- 不改 CSP / `withGlobalTauri`
- 不执行 Shell / Checkpoint / Provider Keyring / Agent streaming 实机操作

WebView：NOT RUN。本文不证明真实 WebView 点击行为。

## Baseline Commands

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
rg -n "invokeTauri|getTauriInvoke|window\.__TAURI__|__TAURI__" src/interface/web tests/frontend docs/frontend
rg -n "getTauriBridge|isTauriAvailable|getTauriInvoke|invokeTauri" src/interface/web/app.js
Get-Content src/interface/web/services/tauri-service.js
rg -n "invokeTauri|getTauriInvoke|__TAURI__" src/interface/web/modules
rg -n "invokeTauri|getTauriInvoke|__TAURI__" tests/frontend
npm run test:security-gate
git diff --name-only -- src/interface/web src/interface/desktop src/engine/tool-system
git diff --cached --check
```

## Skeleton Status

`src/interface/web/services/tauri-service.js` is still a skeleton:

```js
'use strict';

// V3X Day 3-A skeleton only. No production wiring yet.
const tauriServiceSkeleton = Object.freeze({ phase: 'v3x-day3a', domain: 'tauri-service' });
```

Sampling result：`tauri-service.js` is not implemented and not wired as an invoke service.

## Bridge Inventory

| Helper | Evidence | Current behavior | Sampling status |
|---|---|---|---|
| `getTauriBridge()` | `src/interface/web/app.js:144` | Reads `window.HajimiTauri`; throws if unavailable. | KEEP-FOR-NOW |
| `isTauriAvailable()` | `src/interface/web/app.js:150` | Boolean guard for `window.HajimiTauri?.isAvailable?.()`. | KEEP-FOR-NOW |
| `getTauriInvoke()` | `src/interface/web/app.js:154` | Returns `this.getTauriBridge().invoke`. | KEEP-FOR-NOW |
| `invokeTauri(command,args)` | `src/interface/web/app.js:158` | App-level wrapper around `getTauriInvoke()`. | KEEP-FOR-NOW |
| `HajimiTauri.invoke` | `src/interface/web/modules/tauri-bridge.js:20` | Central browser adapter over `__TAURI__.core.invoke` / internals. | DO-NOT-MOVE-YET |
| `HajimiTauri.Channel` | `src/interface/web/modules/tauri-bridge.js:30` | Tauri callback channel adapter. | STREAMING-RISK |
| `HajimiTauri.listen` | `src/interface/web/modules/tauri-bridge.js:55` | Tauri event listener adapter. | STREAMING-RISK |

## Invoke Callsites

| Area | Path / line | Function | Command / tool | Classification | Evidence / note | Next action |
|---|---:|---|---|---|---|---|
| Tool bridge | `src/interface/web/app.js:172` | `executeTool` | `execute_tool` | EXECUTION-RISK | Generic tool runner. Can route to grep/git/lsp/shell/mcp. | Keep out of first service migration. |
| Shell helper | `src/interface/web/app.js:194` | `runShellCommand` | `execute_tool` via shell tool | EXECUTION-RISK | Builds shell line and calls `executeTool`. | Do not migrate in Day07 read-only slice. |
| Workspace file create | `src/interface/web/app.js:844` | `createNewFile` | `write_file` | WRITE-RISK | Creates file. | Keep for later write-gated service. |
| Editor open | `src/interface/web/app.js:1428` | `openFile` | `read_file` | READ-ONLY | Reads file content. | Candidate only after bridge helper contract is locked. |
| Inline diff apply | `src/interface/web/app.js:1507` | `addDiffMessageCard` | `apply_edits` | WRITE-RISK | Applies edits. | Forbidden for read-only service slice. |
| Cumulative stats | `src/interface/web/app.js:2127` | `loadCumulativeFromBackend` | `get_cumulative_stats` | READ-ONLY | Reads stats. | Candidate after dashboard/audit read-only helpers. |
| Context compaction | `src/interface/web/app.js:2231` | `autoCompactContext` | `optimize_context` | UNKNOWN | LLM/provider path; changes chat state after summary. | Keep for later Chat/Provider review. |
| Context prompt | `src/interface/web/app.js:2255` | `buildContextPrompt` | `read_file` | READ-ONLY | Reads context files. | Candidate only after file-read boundary smoke. |
| Slash tools | `src/interface/web/app.js:2551` | `handleChatCommand` | `list_tools` | READ-ONLY | Lists tools for `/tools`. | Candidate after slash-command smoke. |
| Slash compact | `src/interface/web/app.js:2788` | `handleChatCommand` | `optimize_context` | UNKNOWN | Provider/LLM context compaction path. | Keep for later Chat/Provider review. |
| Stream diagnostics | `src/interface/web/app.js:2829` | `recordStreamDiagnostic` | `record_stream_diagnostic` | STREAMING-RISK | Best-effort diagnostic write/log around streaming. | Keep with Agent streaming. |
| Agent task | `src/interface/web/app.js:3104` | `invokeAgentTask` | `run_agent_task` | STREAMING-RISK | Agent task execution path. | Do not migrate in read-only service. |
| Chat stream | `src/interface/web/app.js:3367` | `streamChat` | `stream_chat` | STREAMING-RISK | Uses `Channel`; core model streaming path. | Keep with Agent streaming. |
| Provider list | `src/interface/web/app.js:3572` | `loadProviders` | `get_provider_configs` | READ-ONLY / PROVIDER-BOUNDARY | Reads provider metadata; adjacent to keyring. | Candidate only as provider read-only slice with explicit smoke. |
| Provider probe cancel | `src/interface/web/app.js:3905` | `setupProviderSettings` | `probe_provider_context_capacity` | WRITE-RISK / PROVIDER-BOUNDARY | Probe can consume API/network budget. | Do not migrate first. |
| Provider probe run | `src/interface/web/app.js:3939` | `setupProviderSettings` | `probe_provider_context_capacity` | WRITE-RISK / PROVIDER-BOUNDARY | High-cost provider probing. | Keep for dedicated provider probe task. |
| Provider probe result | `src/interface/web/app.js:4040` | `openProviderModal` | `get_probe_result` | READ-ONLY / PROVIDER-BOUNDARY | Reads probe status. | Candidate after provider read-only UI is wired. |
| Provider backup export | `src/interface/web/app.js:4175` | `exportProviderBackup` | `export_provider_backup` | WRITE-RISK / PROVIDER-BOUNDARY | Exports backup with password. | Keep out. |
| Provider backup import | `src/interface/web/app.js:4185` | `importProviderBackup` | `import_provider_backup` | WRITE-RISK / PROVIDER-BOUNDARY | Imports provider configs. | Keep out. |
| Provider save | `src/interface/web/app.js:4240` | `saveProviderConfig` | dynamic `add_provider_config` / `update_provider_config` | WRITE-RISK / PROVIDER-KEYRING | Saves config and API key. | Forbidden for read-only migration. |
| Provider delete | `src/interface/web/app.js:4261` | `deleteProviderConfig` | `delete_provider_config` | WRITE-RISK / PROVIDER-KEYRING | Deletes provider config. | Forbidden for read-only migration. |
| Profile list | `src/interface/web/app.js:4274` | `loadProfiles` | `list_profiles` | READ-ONLY / PROVIDER-BOUNDARY | Reads profiles. | Candidate only after provider settings boundary. |
| Active profile | `src/interface/web/app.js:4275` | `loadProfiles` | `get_active_profile` | READ-ONLY / PROVIDER-BOUNDARY | Reads active profile. | Candidate only after provider settings boundary. |
| Profile switch | `src/interface/web/app.js:4297` | `setupProfileSettings` | `set_active_profile` | WRITE-RISK / PROVIDER-BOUNDARY | Changes active profile. | Keep out. |
| Profile create | `src/interface/web/app.js:4311` | `setupProfileSettings` | `create_profile` | WRITE-RISK / PROVIDER-BOUNDARY | Creates profile. | Keep out. |
| Profile delete | `src/interface/web/app.js:4327` | `setupProfileSettings` | `delete_profile` | WRITE-RISK / PROVIDER-KEYRING | Deletes profile and related keys. | Keep out. |
| Agent provider map | `src/interface/web/app.js:4344` | `loadAgentProviders` | `get_agent_providers` | READ-ONLY / PROVIDER-BOUNDARY | Reads bindings. | Candidate after provider read-only smoke. |
| Agent bind | `src/interface/web/app.js:4382` | `setupAgentProvider` | `set_agent_provider` | WRITE-RISK / PROVIDER-BOUNDARY | Changes agent/provider binding. | Keep out. |
| Agent unbind | `src/interface/web/app.js:4398` | `unbindAgentProvider` | `set_agent_provider` | WRITE-RISK / PROVIDER-BOUNDARY | Changes agent/provider binding. | Keep out. |
| Agent approval reject | `src/interface/web/app.js:4820` | `showApprovalModal` | `resolve_agent_approval` | EXECUTION-RISK | Controls agent approval outcome. | Keep with governance. |
| Agent approval approve | `src/interface/web/app.js:4831` | `showApprovalModal` | `resolve_agent_approval` | EXECUTION-RISK | Controls agent approval outcome. | Keep with governance. |
| Governance generic | `src/interface/web/app.js:4842` | `invokeGovernance` | dynamic `cmd` | EXECUTION-RISK / UNKNOWN | Dynamic command: pause/resume/approval/memory/plan. | Keep until each command is enumerated. |
| Checkpoint list | `src/interface/web/app.js:4860` | `loadCheckpoints` | `list_checkpoints` | READ-ONLY / CHECKPOINT-BOUNDARY | Reads checkpoint list. | Candidate only in checkpoint read-only task. |
| Checkpoint dry run | `src/interface/web/app.js:4909` | `restoreCheckpoint` | `restore_checkpoint` dryRun | UNKNOWN / CHECKPOINT-BOUNDARY | Dry run call precedes write restore. | Keep with restore flow. |
| Checkpoint restore | `src/interface/web/app.js:4919` | `restoreCheckpoint` | `restore_checkpoint` confirm | WRITE-RISK / CHECKPOINT | Restores files. | Forbidden for read-only service. |
| Checkpoint export | `src/interface/web/app.js:4929` | `exportCheckpoint` | `export_checkpoint` | READ-ONLY / CHECKPOINT-BOUNDARY | Exports JSON blob. | Candidate only after checkpoint export review. |
| Checkpoint compare | `src/interface/web/app.js:4940` | `compareCheckpoints` | `compare_checkpoints` | READ-ONLY / CHECKPOINT-BOUNDARY | Compares checkpoint metadata. | Candidate only in checkpoint read-only task. |
| All checkpoint export | `src/interface/web/app.js:4991` | `exportAllCheckpoints` | `export_checkpoint` id `all` | READ-ONLY / CHECKPOINT-BOUNDARY | Exports all checkpoint data. | Keep until privacy/export risk reviewed. |
| Accept edits | `src/interface/web/app.js:5294` | `acceptAllEdits` | `apply_edits` | WRITE-RISK | Applies edits to files. | Forbidden for read-only service. |
| Agent command | `src/interface/web/app.js:5337` | `runAgentCommand` | `run_agent_command` | EXECUTION-RISK / STREAMING-RISK | Runs agent command from palette/trace. | Keep with Agent. |
| Edit history | `src/interface/web/app.js:5358` | `loadEditHistory` | `get_edit_history` | READ-ONLY | Reads edit history. | Candidate after trace/read-only smoke. |
| Provider validate | `src/interface/web/app.js:5557` | post-init provider test bind | `validate_provider` | PROVIDER-BOUNDARY / UNKNOWN | Network/provider validation. | Keep for provider-specific task. |
| Audit log | `src/interface/web/modules/audit-log.js:40` | `loadAuditLogs` | `get_audit_logs` | READ-ONLY | Existing readonly module. | Strong candidate for future tauri read-only wrapper. |
| Inspector receipt | `src/interface/web/modules/inspector.js:444` | `loadLatestReceipt` | `get_latest_receipt` | READ-ONLY | Existing inspector readonly path. | Candidate after inspector wiring debt clears. |
| Dashboard metrics | `src/interface/web/controllers/dashboard-controller.js:25` | `updateMetrics` | `get_resource_metrics` | READ-ONLY | Existing dashboard readonly controller. | Strong candidate. |
| Workspace init | `src/interface/web/modules/workspace.js:13` | `initWorkspace` | `get_current_workspace` | READ-ONLY | Reads current workspace. | Candidate after workspace boundary review. |
| Workspace tree | `src/interface/web/modules/workspace.js:28,53,64` | `loadFileTree` / `buildTreeFromEntries` | `list_dir` | READ-ONLY | Reads directories recursively. | Candidate after path boundary smoke. |
| Workspace folder create | `src/interface/web/modules/workspace.js:90` | `createNewFolder` | `create_dir` | WRITE-RISK | Creates directory. | Keep out. |
| Workspace rename | `src/interface/web/modules/workspace.js:192` | `renameFile` | `rename_path` | WRITE-RISK | Renames path. | Keep out. |
| Workspace delete | `src/interface/web/modules/workspace.js:210` | `deleteFile` | `delete_path` | WRITE-RISK | Deletes path recursively. | Forbidden for read-only service. |
| Agent trace subscription | `src/interface/web/modules/thinking-ui.js:287` | `startTraceSubscription` | `subscribe_agent_trace` | STREAMING-RISK | Uses Tauri `Channel`; Agent trace stream. | Keep with Agent streaming. |

## Classification Counts

These counts are based on classification tags from the sampled callsite rows above. Some rows intentionally carry more than one risk tag, for example `EXECUTION-RISK / UNKNOWN`.

| Classification | Count | Notes |
|---|---:|---|
| READ-ONLY | 19 | Includes audit logs, dashboard metrics, inspector receipt, directory/file reads, checkpoint list/compare/export candidates. |
| WRITE-RISK | 18 | Includes file write/edit, workspace create/rename/delete, provider save/delete/import/export/probe, profile changes, checkpoint restore. |
| EXECUTION-RISK | 6 | Includes generic `execute_tool`, shell helper, governance dynamic calls, agent approvals, agent commands. |
| STREAMING-RISK | 5 | Includes `stream_chat`, trace subscription, run agent task, stream diagnostic, agent command overlap. |
| UNKNOWN | 5 | Includes optimize context, governance dynamic command, checkpoint dry run in restore flow, provider validation, provider-adjacent reads requiring boundary review. |
| KEEP-FOR-NOW | 7 | Bridge helpers and Provider/Checkpoint/Agent-boundary paths should not move in first service cut. |

## Existing Smoke Evidence

| Area | Evidence pointer | Result / meaning |
|---|---|---|
| Audit log readonly | `tests/frontend/day23_audit_log_smoke.js` | Covers `get_audit_logs` mock, empty/multiple rows, status classes, XSS escaping, missing DOM. |
| Resource dashboard readonly | `tests/frontend/day24_resource_dashboard_smoke.js` | Covers `get_resource_metrics`, no Tauri path, missing DOM, no provider/checkpoint/agent/shell touch. |
| Inspector safety | `tests/frontend/day25_inspector_safety_smoke.js` and `tests/frontend/day35_inspector_rendering_safe_dom_smoke.js` | Covers inspector render safety; does not prove WebView wiring in this doc. |
| Provider readonly helper | `tests/frontend/day36_provider_readonly_dom_smoke.js` | Covers read-only provider helper not calling `invokeTauri`; not a provider keyring/write smoke. |
| Handle chat command invoke | `tests/frontend/day27_handle_chat_command_invoke_smoke.js` | Covers local invoke source for `/chat`, `/search`, `/compact`; does not prove WebView behavior. |
| Tauri bridge mocks | `tests/frontend/day13_workspace_modules_smoke.js`, `tests/frontend/day14_sessions_thinking_modules_smoke.js`, `tests/frontend/day21_event_bridge.test.js` | Provide mock evidence for `__TAURI__`/module behavior, not a service migration proof. |

## Security Gate Result

Command:

```powershell
npm run test:security-gate
```

Observed result:

```text
Security Audit Gate V1 summary
findings: 97
failures: 3
warnings: 94
allowlisted: 94

failures:
- [DOM-HTML-001] src/interface/web/views/command-palette-view.js:36 dangerous HTML API requires allowlist reason or safe DOM rewrite
- [DOM-HTML-001] src/interface/web/views/session-list-view.js:22 dangerous HTML API requires allowlist reason or safe DOM rewrite
- [DOM-HTML-001] src/interface/web/views/session-list-view.js:26 dangerous HTML API requires allowlist reason or safe DOM rewrite

Security Audit Gate V1: FAIL
```

Sampling interpretation：known baseline FAIL from Command Palette / Session List View safe-DOM debt. Day06 did not modify those files and does not mark security-gate as PASS.

## Migration Order

Recommended order for later work:

1. Build `services/tauri-service.js` contract around availability/invoke lookup only, without moving app.js callsites.
2. Add Node smoke for service behavior: unavailable, mock invoke success, mock invoke failure, no direct `__TAURI__` mutation.
3. Migrate strongest READ-ONLY candidates first:
   - `get_resource_metrics`
   - `get_audit_logs`
   - `get_latest_receipt`
4. Migrate low-risk file/directory reads only after workspace path-boundary smoke:
   - `get_current_workspace`
   - `list_dir`
   - `read_file`
5. Keep Provider / Keyring / Checkpoint / Shell / Agent streaming out until separate dedicated tasks.
6. Do not migrate dynamic commands (`command`, `cmd`, `execute_tool`) until each command is enumerated and smoke-covered.

## Forbidden / Keep For Now

| Area | Commands | Reason |
|---|---|---|
| Shell / generic tool execution | `execute_tool`, shell-backed `runShellCommand` | Can execute commands; not read-only. |
| Provider write/keyring | `add_provider_config`, `update_provider_config`, `delete_provider_config`, `import_provider_backup`, `export_provider_backup`, `delete_profile` | Adjacent to API keys and backup secrets. |
| Provider probe/validate | `probe_provider_context_capacity`, `validate_provider` | Can trigger network/API spend and provider-specific behavior. |
| Checkpoint restore/edit | `restore_checkpoint`, `apply_edits` | Can write files. |
| Agent streaming | `stream_chat`, `subscribe_agent_trace`, `run_agent_task`, `run_agent_command` | Runtime stream path; needs WebView/Channel proof. |
| Governance dynamic | `invokeGovernance(cmd,args)` | Dynamic command. Must enumerate before migration. |
| Workspace writes | `write_file`, `create_dir`, `rename_path`, `delete_path` | File-system mutation. |

## Risks / Unknowns

- `optimize_context` is model/provider-adjacent and changes chat state after summary; static scan cannot prove it is safe to move as read-only.
- `restore_checkpoint` dry-run and confirm-run live in the same function; dry-run alone should not be split without dedicated checkpoint smoke.
- `get_provider_configs`, `get_agent_providers`, and profile reads are read-like but provider/keyring-adjacent; mark them as Provider-boundary, not globally safe.
- `execute_tool` is the largest risk surface because callers include grep/git/LSP/MCP/shell flows.
- This report does not prove WebView behavior, does not run Tauri commands, and does not fix security-gate failures.

## Validation Receipt

| Check | Result |
|---|---|
| Branch | `stone-audit-v3x-controlled-demolition` |
| HEAD before | `d1c0869053c2da7b75a611dff19ced462d95f73b` |
| Production code changed | NO |
| `app.js` changed | NO |
| `tauri-service.js` changed | NO |
| `modules/tauri-bridge.js` changed | NO |
| Shell / Provider Keyring / Checkpoint / Agent streaming executed | NO |
| WebView smoke | NOT RUN |
| old dirty files staged | NO at sampling time |
| `git diff --cached --check` before doc stage | PASS |

## Next

Proceed to a separate Day07 implementation task only if it starts with a service contract smoke and limits migration to readonly wrappers. The first practical implementation target should be `services/tauri-service.js` availability/invoke wrapper plus tests; do not move Provider / Checkpoint / Shell / Agent streaming in that same slice.

## 工单 V3X-DAY06 完成

- Commit: `docs(frontend): sample tauri invoke service migration`
- 分支: `stone-audit-v3x-controlled-demolition`
- HEAD before / after: before `d1c0869053c2da7b75a611dff19ced462d95f73b`; after filled by commit receipt
- 变更文件: `docs/frontend/TAURI-INVOKE-SERVICE-SAMPLING-V3X.md`
- invoke callsites: 51 sampled rows
- READ-ONLY count: 19
- WRITE-RISK count: 18
- EXECUTION-RISK count: 6
- STREAMING-RISK count: 5
- UNKNOWN count: 5
- security-gate: FAIL, known baseline 3 failures listed above
- production changes: NO
- old dirty files staged: NO
- next: Day07 service contract smoke before any migration
