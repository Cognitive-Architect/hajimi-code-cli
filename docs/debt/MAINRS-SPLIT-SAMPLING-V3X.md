# MAINRS-SPLIT-SAMPLING-V3X

## 1. Git Baseline & Worktree State

Date: 2026-06-16
Branch: `stone-audit-v3x-controlled-demolition`
HEAD Hash: `425263ddfec4e324a534deadf92316f99775c5d9`

### Git Status Summary

Command:
```powershell
git status --short
```

Output:
```text
 M docs/debt/DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN.md
 M docs/debt/INDEX.md
 M docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md
 D "docs/roadmap/Hajimi Agent/debt/DEBT-DAY-07-CHECKPOINT-DIFF-UI.md"
 D "docs/roadmap/Hajimi Agent/plan/AGENT-UI-INTEGRATION-SAMPLING-NOTES.md"
 D "docs/roadmap/Hajimi LLM/plan/AGENT-LLM-NATIVE-DESIGN.md"
 D "docs/roadmap/Hajimi LLM/plan/LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-01/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-02/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-03/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-04/snapshot.md"
?? docs/debt/DEBT-AGENT-CHINESE-I18N.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-THINKING-LEAK.md
?? docs/debt/DEBT-AGENT-LOOP-LLM-NO-OP.md
?? docs/debt/DEBT-AGENT-UI-INTEGRATION.md
?? docs/debt/DEBT-TAURI-CHANNEL-ENVELOPE-REGRESSION.md
?? docs/debt/STONE-AUDIT-V1.5-PACKAGE-SMOKE.md
?? "docs/roadmap/Hajimi AgentFix/"
?? "docs/roadmap/Hajimi ToneFix/"
?? "docs/roadmap/hajimi template.7z"
?? "docs/roadmap/hajimi template/"
?? src/interface/desktop/native-smoke.txt
```

> [!NOTE]
> All the above modifications and untracked files are pre-existing dirty worktree files from past steps. No new changes were staged before starting Day 10.

### Staging Verification
Command:
```powershell
git diff --cached --name-only
```
Output: *(empty)*

Conclusion: `old dirty files staged: NO`

### Production Safety Check
Command:
```powershell
git diff --name-only -- src/interface/desktop/src src/interface/desktop/tauri.conf.json src/interface/web src/engine/tool-system
```
Output: *(empty)*

Conclusion: `production changes: NO` (No modifications have been made to production code).

---

## 2. File Line Count & Cargo Baseline

### `main.rs` Line Count
Command:
```powershell
(Get-Content src/interface/desktop/src/main.rs).Count
```
Output: **5079** (Total physical lines: **5094**).

### Cargo Baseline
Command:
```powershell
cargo check -p hajimi-desktop
```
Output:
```text
Finished dev profile [unoptimized + debuginfo] target(s) in 4.65s
```
Status: **PASS**

---

## 3. Managed State & Startup Architecture

### Managed State Inventory (`app.manage`)
Tauri manages exactly two state instances initialized inside `main.rs::main()`:
1. `state: AppState` (line 3663)
   - Mutex-guarded `ToolRegistry` (`registry`)
   - Mutex-guarded active profile configuration (`active_profile`)
   - Mutex-guarded agent-to-provider routing mappings (`agent_providers`)
   - Mutex-guarded TraceEvent broadcast sender (`trace_tx`)
   - Mutex-guarded flow control flags (`paused`)
   - Mutex-guarded security/governance approval level (`approval_level`)
   - Mutex-guarded inline editing history timeline entries (`edit_history`)
   - Shared memory gateways (`memory_gateway`)
   - Shared token metrics tracker (`token_tracker`)
   - Mutex-guarded pending human-in-the-loop approvals (`pending_approvals`)
   - RwLock-guarded runtime active LLM client (`agent_llm_client`)
2. `agent_loop_for_setup: Arc<AgentLoop>` (line 3664)
   - The thread-safe singleton representing the core autonomous loop.
   - Manages task execution lifecycle, state transitions, step planning, reflection, and tracing.

### Startup Map (`main.rs` Entry Point)
The startup flow inside `fn main()` follows this sequence:
1. Initialize the system logger.
2. Resolve workspace directories sandbox (`get_workspace_dir`).
3. Construct the global `ToolRegistry` using `build_registry`.
4. Initialize the database connection and construct `MemoryGateway` & `TokenUsageTracker`.
5. Enter the Tauri setup hook (`tauri::Builder::default().setup(|app| { ... })`):
   - Instantiate `AgentLoop` via `AgentLoopBuilder::production_ready("hajimi-desktop")`.
   - Setup trace event broadcast channel and inject the sender to the state (`state.set_trace_tx(...)`).
   - Register `AppState` and `Arc<AgentLoop>` with Tauri using `app.manage()`.
6. Bind the `invoke_handler` register mapping (all 55 commands).
7. Run the Tauri application loop.

---

## 4. Tauri Command Map & Risk Classification

There are exactly **55** registered Tauri commands in `main.rs`. They are categorized below:

### Risk Categories Overview
* **LOW-RISK**: Safe query or telemetry wrapper. Zero side-effects, does not alter files or keyring.
* **HIGH-RISK**: Interacts with local files, executes system commands, manages cryptography/backups, queries external LLMs, or mutates core runtime state.
* **KEEP-FOR-NOW**: Simple command that depends directly on registry initialization or is heavily coupled with state variables defined in `main.rs`.
* **UNKNOWN**: No commands are classified as unknown.

| Line | Command | Category | Purpose / Description | Rationale |
|---|---|---|---|---|
| 273 | `greet` | **LOW-RISK** | Simple string concatenation test | Pure function |
| 278 | `probe_provider_context_capacity` | **HIGH-RISK** | Spawns background context capacity measurement | Spawns LLM API calls, writes results to file |
| 386 | `get_probe_result` | **HIGH-RISK** | Reads capacity measurement cache | File read from `~/.hajimi/` |
| 412 | `get_latest_receipt` | **LOW-RISK** | Gets latest receipt summary | Pure configuration query |
| 914 | `read_file` | **HIGH-RISK** | Reads local workspace file | File read permission check |
| 921 | `write_file` | **HIGH-RISK** | Writes local workspace file | File write permission check |
| 932 | `list_dir` | **HIGH-RISK** | Lists directory contents | Directory listing |
| 966 | `create_dir` | **HIGH-RISK** | Creates directory structure | Workspace write |
| 973 | `rename_path` | **HIGH-RISK** | Renames or moves a file | Workspace write |
| 983 | `delete_path` | **HIGH-RISK** | Removes file or directory | Workspace write |
| 1079 | `list_tools` | **KEEP-FOR-NOW** | Lists registered tools | Coupled with `AppState.registry` |
| 1094 | `execute_tool` | **HIGH-RISK** | Spawns MCP tool execution | Spawns subprocesses, audits execution |
| 1184 | `record_stream_diagnostic` | **LOW-RISK** | Appends diagnostic trace log | Telemetry collection |
| 1190 | `get_stream_diagnostic_info` | **LOW-RISK** | Retrieves stream diagnostic trace logs | Telemetry query |
| 1682 | `get_provider_configs` | **HIGH-RISK** | Retrieves provider JSON profiles | Reads workspace config from disk |
| 1706 | `add_provider_config` | **HIGH-RISK** | Appends a new model provider configuration | Writes to workspace configurations |
| 1746 | `update_provider_config` | **HIGH-RISK** | Modifies existing provider properties | Writes to workspace configurations |
| 1786 | `delete_provider_config` | **HIGH-RISK** | Removes a provider configuration | Writes to workspace configurations |
| 1815 | `get_providers` | **HIGH-RISK** | Retrieves config profile providers | Accesses keyring & filesystem |
| 1869 | `get_current_workspace` | **LOW-RISK** | Gets active workspace path | Query |
| 1877 | `validate_provider` | **HIGH-RISK** | Connects to endpoint to verify credentials | Makes external network calls |
| 2016 | `stream_chat` | **HIGH-RISK** | Starts a chat session stream | LLM network calls, state mutations |
| 2264 | `compact_context` | **HIGH-RISK** | Condenses message history | LLM call & memory compact |
| 2275 | `optimize_context` | **HIGH-RISK** | Optimizes active memory slots | LLM call & memory compact |
| 2292 | `export_provider_backup` | **HIGH-RISK** | Exports encrypted backup of credentials | Cryptography & File write |
| 2334 | `import_provider_backup` | **HIGH-RISK** | Imports credentials from encrypted file | Cryptography & Keyring write |
| 2421 | `list_profiles` | **LOW-RISK** | Lists user configuration profiles | File read (directories list) |
| 2446 | `get_active_profile` | **LOW-RISK** | Reads active profile state | State query |
| 2455 | `set_active_profile` | **KEEP-FOR-NOW** | Changes the active profile | State mutation, coupled |
| 2468 | `create_profile` | **LOW-RISK** | Creates configuration folder structure | File write |
| 2482 | `delete_profile` | **KEEP-FOR-NOW** | Removes profile folder | State mutation & file delete |
| 2515 | `get_agent_providers` | **LOW-RISK** | Queries agent model mapping state | State query |
| 2528 | `set_agent_provider` | **KEEP-FOR-NOW** | Changes agent model provider configuration | State mutation |
| 2633 | `create_agent_with_provider` | **HIGH-RISK** | Spawns agent initialization script | File write |
| 2788 | `run_agent_task` | **HIGH-RISK** | Launches agent loop task execution | Starts background executor & loop thread |
| 2879 | `get_audit_logs` | **LOW-RISK** | Retrieves security audits log | Database read |
| 2887 | `subscribe_agent_trace` | **HIGH-RISK** | Subscribes trace event stream via IPC channel | IPC stream registration |
| 2947 | `pause_loop` | **HIGH-RISK** | Suspends active agent loop execution | Mutates agent loop flow control |
| 2954 | `resume_loop` | **HIGH-RISK** | Resumes agent loop execution | Mutates agent loop flow control |
| 2961 | `set_approval_level` | **HIGH-RISK** | Adjusts human-in-the-loop security barrier | Mutates loop governance rules |
| 2974 | `inject_memory` | **HIGH-RISK** | Injects key/value directly to working memory | Mutates active agent memory state |
| 2979 | `update_plan` | **HIGH-RISK** | Updates planner goals on blackboard | Mutates active planner state |
| 2984 | `list_checkpoints` | **HIGH-RISK** | Lists file system checkpoints | File read (checkpoints folder) |
| 2989 | `get_edit_history` | **LOW-RISK** | Retrieves edits history list | State query |
| 2994 | `restore_checkpoint` | **HIGH-RISK** | Rolls back workspace to previous checkpoint | Full file system overwrite |
| 3028 | `compare_checkpoints` | **HIGH-RISK** | Computes diff between checkpoints | File reads & diff generation |
| 3040 | `export_checkpoint` | **HIGH-RISK** | Generates export bundle of a checkpoint | File read & bundle write |
| 3060 | `get_resource_metrics` | **LOW-RISK** | Retrieves system resource metrics | Telemetry query |
| 3082 | `run_agent_command` | **HIGH-RISK** | Executes slash commands | Parsers & spawns agent execution |
| 3122 | `subscribe_resource_alerts` | **HIGH-RISK** | Hooks alerts listener | IPC stream registration |
| 3152 | `apply_edits` | **HIGH-RISK** | Writes diff chunks | File writes, Git status checks |
| 3188 | `preview_edit` | **HIGH-RISK** | Generates temporary editing previews | File reads |
| 3262 | `get_ast_context` | **LOW-RISK** | Resolves code AST references | Read query |
| 3280 | `get_cumulative_stats` | **LOW-RISK** | Computes token accumulation data | Database queries |
| 3563 | `resolve_agent_approval` | **HIGH-RISK** | Delivers approval decision payload | Governance channel write |

* **Total Commands**: 55
* **LOW-RISK Count**: 14
* **HIGH-RISK Count**: 37
* **KEEP-FOR-NOW Count**: 4
* **UNKNOWN Count**: 0

---

## 5. Day 11 Architecture Plan: Modular Skeleton

Day 11 will establish module boundaries without refactoring complex code, keeping all implementations compile-ready and modularized.

### Target Module Structure
```
src/interface/desktop/src/
├── main.rs
├── audit.rs
├── state.rs          # Managed AppState and associated structures
└── commands/         # Tauri commands sub-modules
    ├── mod.rs        # Main command registrar
    ├── fs.rs         # Filesystem commands
    ├── tool.rs       # Tooling and list/exec commands
    ├── provider.rs   # Provider keyring and credential backup/import
    ├── agent.rs      # Run agent tasks, chat streaming, slash commands
    ├── profile.rs    # User configuration profiles
    ├── checkpoint.rs # Rollback and comparison checkpoints
    ├── info.rs       # Metadata queries, logs, metrics, history, greet
    └── governance.rs # Pause/resume loop, approval level gates
```

### Day 11 allowed files
The only workspace files that may be created or changed during Day 11:
- `src/interface/desktop/src/state.rs` (Created)
- `src/interface/desktop/src/commands/mod.rs` (Created)
- `src/interface/desktop/src/commands/info.rs` (Created)
- `src/interface/desktop/src/main.rs` (Modified to import `mod state;` and `mod commands;`)

### Day 11 Implementation Steps (Skeleton Mode)
1. **Extract `state.rs`**: Move the definition of `AppState`, `EditHistoryEntry`, `CheckpointFileRef`, `CheckpointDiffSummary`, `CheckpointMetadata`, `CheckpointRecord`, `CheckpointExportBundle`, `CheckpointFileChange`, `CheckpointCompareResult`, `RestoreFilePlan`, and `RestoreResult` to `state.rs`. Update all field visibilities to `pub`.
2. **Setup `commands/mod.rs`**: Establish the sub-module imports (`pub mod fs;`, etc.).
3. **Move `greet` & `get_latest_receipt` to `commands/info.rs`**: As a verification slice, move these two simple commands to `info.rs` and verify compile.
4. **Compile-check**: Verify using `cargo check -p hajimi-desktop`.

---

## 6. Day 12 Architecture Plan: Split Execution Map

Day 12 will execute the physical migration of command functions from `main.rs` into the command sub-modules defined in Day 11.

### Stage 1: Info & Telemetry Commands -> `commands/info.rs`
- Move: `record_stream_diagnostic`, `get_stream_diagnostic_info`, `get_current_workspace`, `get_edit_history`, `get_resource_metrics`, `get_cumulative_stats`.
- Risk: LOW

### Stage 2: Profile Settings Commands -> `commands/profile.rs`
- Move: `list_profiles`, `get_active_profile`, `set_active_profile`, `create_profile`, `delete_profile`.
- Risk: LOW-MEDIUM

### Stage 3: Workspace Filesystem Commands -> `commands/fs.rs`
- Move: `read_file`, `write_file`, `list_dir`, `create_dir`, `rename_path`, `delete_path`.
- Risk: MEDIUM (Verify path validation functions like `resolve_workspace_path` are imported properly).

### Stage 4: Checkpoints Commands -> `commands/checkpoint.rs`
- Move: `list_checkpoints`, `restore_checkpoint`, `compare_checkpoints`, `export_checkpoint`.
- Risk: HIGH (Interacts with workspace backups and zip generation).

### Stage 5: Provider Keyring & Backups Commands -> `commands/provider.rs`
- Move: `get_providers`, `get_provider_configs`, `add_provider_config`, `update_provider_config`, `delete_provider_config`, `validate_provider`, `export_provider_backup`, `import_provider_backup`.
- Risk: HIGH (Keyring library, cryptography, raw passwords).

### Stage 6: Governance Loop Commands -> `commands/governance.rs`
- Move: `pause_loop`, `resume_loop`, `set_approval_level`, `inject_memory`, `update_plan`, `resolve_agent_approval`.
- Risk: HIGH (Flow control, locking oneshot channels).

### Stage 7: Agent Runtime & Chat Commands -> `commands/agent.rs`
- Move: `stream_chat`, `run_agent_task`, `run_agent_command`, `subscribe_agent_trace`, `subscribe_resource_alerts`.
- Risk: HIGH (Thread synchronization, async streams).

### Stage 8: Tool & AST Queries -> `commands/tool.rs`
- Move: `list_tools`, `execute_tool`, `apply_edits`, `preview_edit`, `get_ast_context`, `probe_provider_context_capacity`, `get_probe_result`, `compact_context`, `optimize_context`.
- Risk: HIGH (AST contexts, child processes, LLM queries).

---

## 7. Rollback & Safety Gates

### Rollback Point
If compilation fails or if test suites break during modularization:
- **Discard all uncommitted changes** using:
  ```powershell
  git checkout -- src/interface/desktop/src/main.rs
  git clean -fd src/interface/desktop/src/
  ```
- Re-check compilation: `cargo check -p hajimi-desktop`

### Quality Gates
- **Gate 1: Compilation Integrity**: Every single stage of migration must compile successfully. Do not move to the next stage if `cargo check` fails.
- **Gate 2: Zero Behavior Change**: Command names, argument structures, and output schemas must remain identical. Do not modify signatures.
- **Gate 3: Security Sandboxing**: Filesystem resolution paths must continue utilizing `resolve_workspace_path` logic without modifications.
