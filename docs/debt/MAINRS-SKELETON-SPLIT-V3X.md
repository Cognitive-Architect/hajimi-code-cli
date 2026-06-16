# main.rs Skeleton Split Receipt (V3X-DAY11)

> **工单**: STONE-AUDIT-V3X-DAY11  
> **日期**: 2026-06-16  
> **分支**: `stone-audit-v3x-controlled-demolition`  
> **前置 Commit**: `cfc2443dfb2b00ad5dbd666a7fcaee6d4bfd370d`

---

## 1. 物理重构概述
本步骤为非破坏性的 Rust 后端骨架提取工程。在保持所有命令物理实现、安全策略和原有行为完全一致的前提下，将 `main.rs` 的低风险状态、工具注册和启动组件物理拆分至独立的子模块。

### 核心变更列表
- **`src/interface/desktop/src/state.rs`** (创建): 存放 `AppState`、`EditHistoryEntry` 以及各类 Checkpoint/Restore 相关的数据结构定义与 UI 审批通道接收器。
- **`src/interface/desktop/src/registry.rs`** (创建): 存放 40+ 核心工具的注册器初始化函数 `build_registry`。
- **`src/interface/desktop/src/startup.rs`** (创建): 存放 LLM-Native Agent 启动驱动 `DesktopAgentTurnDriver` 与 UI-Bridge 治理器 `UiBridgeGovernance` 的实现。
- **`src/interface/desktop/src/error.rs`** (创建): 存放桌面的通用 Error 骨架枚举。
- **`src/interface/desktop/src/commands/mod.rs`** (创建): 声明各命令子模块。
- **`src/interface/desktop/src/commands/info.rs`** (创建): 迁移 `greet` 和 `get_latest_receipt` 两个低风险非核心命令作为编译切片。
- **`src/interface/desktop/src/main.rs`** (修改): 移除已移动的结构、注册器构建逻辑、启动逻辑以及被迁移的两个命令，并将 tauri 注册指向新的 info 命令模块。

---

## 2. 物理行数对比

- **`main.rs` 重构前行数**: `5079` (`Get-Content` 计数) / `5094` (物理文件行数)
- **`main.rs` 重构后行数**: `4614` (`Get-Content` 计数)
- **净缩减行数**: `465` 行 (全部以模块化导入 and 子文件形式物理分流)

---

## 3. 符号物理移动对照表

| 符号 (Symbol) | 迁移前位置 | 迁移后文件 | 风险级别与说明 |
|---|---|---|---|
| `EditHistoryEntry` | `main.rs` (App State 节) | `state.rs` | **LOW** - UI 状态结构体 |
| `CheckpointFileRef` | `main.rs` (App State 节) | `state.rs` | **LOW** - 检查点文件引用 |
| `CheckpointDiffSummary` | `main.rs` (App State 节) | `state.rs` | **LOW** - 检查点 Diff 摘要 |
| `CheckpointMetadata` | `main.rs` (App State 节) | `state.rs` | **LOW** - 检查点元数据 |
| `CheckpointRecord` | `main.rs` (App State 节) | `state.rs` | **LOW** - 本地检查点记录 |
| `CheckpointExportBundle` | `main.rs` (App State 节) | `state.rs` | **LOW** - 导出包结构 |
| `CheckpointFileChange` | `main.rs` (App State 节) | `state.rs` | **LOW** - 文件变更结构 |
| `CheckpointCompareResult` | `main.rs` (App State 节) | `state.rs` | **LOW** - 比较结果结构 |
| `RestoreFilePlan` | `main.rs` (App State 节) | `state.rs` | **LOW** - 恢复计划结构 |
| `RestoreResult` | `main.rs` (App State 节) | `state.rs` | **LOW** - 恢复结果结构 |
| `AppState` | `main.rs` (App State 节) | `state.rs` | **LOW** - 全局托管状态结构 |
| `await_ui_approval_response` | `main.rs` (App State 节) | `state.rs` | **LOW** - 异步审批接收辅助器 |
| `build_registry` | `main.rs` | `registry.rs` | **LOW** - 工具容器注册辅助函数 |
| `DesktopAgentTurnDriver` | `main.rs` | `startup.rs` | **LOW** - Agent 执行期 LLM 驱动 |
| `UiBridgeGovernance` | `main.rs` | `startup.rs` | **LOW** - UI 与后台异步决策审批桥接器 |
| `sanitize_description` | `main.rs` | `startup.rs` | **LOW** - 敏感 Key 脱敏正则工具 |
| `greet` (command) | `main.rs` (Legacy commands 节) | `commands/info.rs` | **LOW** - 验证用测试命令 |
| `get_latest_receipt` (command) | `main.rs` (Legacy commands 节) | `commands/info.rs` | **LOW** - 验证用只读上下文收据获取命令 |

---

## 4. 自动化质量验证结果

### 4.1 Cargo Check 编译验证
- **hajimi-desktop 包编译**: `PASS`
  ```bash
  $ cargo check -p hajimi-desktop
  Finished dev profile [unoptimized + debuginfo] target(s) in 5.51s
  ```
- **Workspace 全编译**: `PASS`
  ```bash
  $ cargo check --workspace
  Finished dev profile [unoptimized + debuginfo] target(s) in 8.47s
  ```

### 4.2 格式化规范检查
- **Rust fmt**: `PASS`
  ```bash
  $ cargo fmt -- --check
  (无输出，代表格式完全合规)
  ```

---

## 5. 安全与语义不变声明
1. **安全策略无变更**: `src/engine/tool-system/src/shell.rs` 未做任何修改，未放宽或变更任何命令白名单；
2. **高风险命令语义未做任何修改**: Keyring、Checkpoint、Provider 以及 Shell 命令函数体全部保存在 `main.rs` 原位置，未做任何代码变动；
3. **前端代码零变更**: 未对 `src/interface/web` 进行任何修改；
4. **编译依赖无变更**: `tauri.conf.json` 及其他配置文件完整保持原样。

---

## 6. 回滚与重构基线指示
若后续步骤发现异常，可通过以下 Git 回滚锚点恢复：
- **回滚命令**: `git reset --hard cfc2443dfb2b00ad5dbd666a7fcaee6d4bfd370d`
