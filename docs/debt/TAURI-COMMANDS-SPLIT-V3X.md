# Tauri Commands Split Receipt (V3X-DAY12)

> **工单**: STONE-AUDIT-V3X-DAY12  
> **日期**: 2026-06-16  
> **分支**: `stone-audit-v3x-controlled-demolition`  
> **当前 Commit**: `945b6ec2d33fec7d95b720ae1cd330b171e8b118`

---

## 1. 物理重构概述
本步骤完成了对 `main.rs` 的深度拆分重构，将所有遗留的 Tauri Commands 物理分配至 `src/interface/desktop/src/commands/` 目录下的对应模块。重构过程严格遵循“只移动，不改逻辑，不变语义”的原则，确保所有安全边界与功能不受任何影响。

### 核心变更列表
- **`src/interface/desktop/src/commands/fs.rs`**: 迁移文件/目录系统相关命令，包括 `resolve_workspace_path` (私有 helper)、`create_workspace_dir`、`rename_workspace_path`、`remove_workspace_path` 等。
- **`src/interface/desktop/src/commands/tool.rs`**: 迁移工具执行与编辑预览相关命令，包括 `execute_tool`、`apply_edits`、`preview_edit`、`probe_provider_context_capacity`、`get_probe_result` 等。
- **`src/interface/desktop/src/commands/agent.rs`**: 迁移 Agent 执行与对话流相关命令，包括 `run_agent_task`、`stream_chat`、`write_stream_diagnostic` 等。
- **`src/interface/desktop/src/commands/checkpoint.rs`**: 迁移检查点管理与版本恢复相关命令，包括 `list_checkpoints`、`find_checkpoint_record`、`compare_checkpoint_records`、`export_checkpoints`、`preview_restore_checkpoint`、`apply_restore_plan`、`write_restore_result_to_blackboard` 等。
- **`src/interface/desktop/src/commands/governance.rs`**: 迁移治理审批交互相关命令，包括 `get_governance_approval_level`、`set_governance_approval_level`、`resolve_agent_approval`、`get_pending_approvals` 等。
- **`src/interface/desktop/src/commands/profile.rs`**: 迁移用户 Profile 与配置管理相关命令，包括 `get_active_profile`、`set_active_profile`、`get_session_trace_events` 等.
- **`src/interface/desktop/src/commands/provider.rs`**: 迁移 AI 提供商配置与 Key 交互相关命令，包括 `get_providers`、`get_provider_config`、`set_provider_key`、`delete_provider_key`、`test_provider_key` 等。
- **`src/interface/desktop/src/main.rs`**: 移除上述所有命令的具体函数体与私有局部 helper，将 Tauri 的 `.invoke_handler(tauri::generate_handler![...])` 注册表更新为调用各子模块命令。

---

## 2. 物理行数对比

- **`main.rs` 重构前行数**: `4614`
- **`main.rs` 重构后行数**: `2780`
- **净缩减行数**: `1834` 行 (整体模块化比率显著提升，主入口文件更清晰)

---

## 3. 符号物理移动对照表

| 符号 (Symbol) | 迁移前位置 | 迁移后文件 | 风险级别与安全说明 |
|---|---|---|---|
| `create_workspace_dir` | `main.rs` | `commands/fs.rs` | **LOW** - 物理移动，语义不变 |
| `rename_workspace_path` | `main.rs` | `commands/fs.rs` | **LOW** - 物理移动，语义不变 |
| `remove_workspace_path` | `main.rs` | `commands/fs.rs` | **LOW** - 物理移动，语义不变 |
| `execute_tool` | `main.rs` | `commands/tool.rs` | **HIGH** - 涉及白名单 Shell/权限检查，完全保留原校验链 |
| `apply_edits` | `main.rs` | `commands/tool.rs` | **HIGH** - 物理移动，语义不变 |
| `preview_edit` | `main.rs` | `commands/tool.rs` | **LOW** - 物理移动，语义不变 |
| `run_agent_task` | `main.rs` | `commands/agent.rs` | **HIGH** - 涉及 Agent 循环拉起，完全继承原有 driver/governance |
| `stream_chat` | `main.rs` | `commands/agent.rs` | **LOW** - 物理移动，语义不变 |
| `list_checkpoints` | `main.rs` | `commands/checkpoint.rs` | **LOW** - 物理移动，语义不变 |
| `find_checkpoint_record` | `main.rs` | `commands/checkpoint.rs` | **LOW** - 物理移动，语义不变 |
| `compare_checkpoint_records` | `main.rs` | `commands/checkpoint.rs` | **LOW** - 物理移动，语义不变 |
| `export_checkpoints` | `main.rs` | `commands/checkpoint.rs` | **LOW** - 物理移动，语义不变 |
| `preview_restore_checkpoint` | `main.rs` | `commands/checkpoint.rs` | **HIGH** - 涉及数据回滚计划，语义不变 |
| `apply_restore_plan` | `main.rs` | `commands/checkpoint.rs` | **HIGH** - 实际写入/备份文件，保留全部安全防护 |
| `get_governance_approval_level` | `main.rs` | `commands/governance.rs` | **LOW** - 物理移动，语义不变 |
| `set_governance_approval_level` | `main.rs` | `commands/governance.rs` | **LOW** - 物理移动，语义不变 |
| `resolve_agent_approval` | `main.rs` | `commands/governance.rs` | **HIGH** - 控制审批流，保留 oneshot channel 机制 |
| `get_providers` | `main.rs` | `commands/provider.rs` | **LOW** - 物理移动，语义不变 |
| `set_provider_key` | `main.rs` | `commands/provider.rs` | **HIGH** - 涉及 OS Keyring 读写，保留原生密钥保护逻辑 |

---

## 4. 自动化质量验证结果

### 4.1 Cargo Test 编译与运行验证
- **hajimi-desktop 单元测试**: `PASS` (所有 63 个 Rust 单元/集成测试全数通过)
  ```bash
  $ cargo test -p hajimi-desktop
  running 63 tests
  test result: ok. 63 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.90s
  ```
- **Workspace 全编译**: `PASS`
  ```bash
  $ cargo check --workspace
  Finished dev profile [unoptimized + debuginfo] target(s) in 8.47s
  ```

### 4.2 Frontend JSDOM Contract Smoke 验证
- **前端 Contract 跑通率**: `100%` (包括 `day26_dom_contract_smoke.js`、`day30_command_palette_delegation_smoke.js` 与 `day36_provider_readonly_dom_smoke.js` 等所有 29 个前端冒烟测试全部通过)
  ```bash
  $ Get-ChildItem -Path tests\frontend\*_smoke.js | ForEach-Object { node $_.FullName }
  ...
  day26 dom contract smoke: PASS
  day30 command palette delegation smoke: PASS
  day36 provider readonly dom smoke: PASS (DOM shell + read-only view/controller/service)
  ```

---

## 5. 安全与语义不变声明
1. **Shell 白名单保持一致**: `src/engine/tool-system/src/shell.rs` 未受任何影响，权限体系原样执行。
2. **Keyring 逻辑零重写**: OS Keyring 与 providers 文件的密钥交互严格保持原样。
3. **前端代码契约零变更**: 物理拆分未引起 Tauri IPC 命令名称变更，对前端完全透明，前端 Contract 测试 100% 保持通过。

---

## 6. 基础设施依赖说明 (Cargo.toml / Cargo.lock)
- **依赖说明**：在本次 Tauri 命令物理拆分中，我们向 `src/interface/desktop/Cargo.toml` 显式添加了对 `futures = { workspace = true }` 的依赖，并更新了 `Cargo.lock`。
- **添加原因**：
  1. 拆分出的命令模块 `src/interface/desktop/src/commands/agent.rs` 包含使用 Stream 处理的 Tauri 流式聊天接口 (`stream_chat`)，其需要使用 `futures::StreamExt`。
  2. 原单体 `main.rs` 位于 desktop 根级，物理拆分到独立模块 `agent.rs` 后，编译系统强制要求独立模块显式导入其用到的 traits。
  3. 为确保子模块能够独立且清洁地通过 `cargo check`，显式引入 workspace 已声明的 `futures` 包。
- **行为改变证明**：仅在包级别声明已被 workspace 精确锁定的通用 Rust 核心库 `futures`，没有引入任何新第三方依赖，不改变任何既有包版本，对应用运行时逻辑零污染、零改变，纯为物理拆分编译过关所需。
