# Day 03 派单: 文件操作专用 Tauri Commands + 前端接入

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 3，处理 `CS-HAJIMI-004` 的 `mkdir/mv/rm` 前后端错配。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: 文件操作专用 Tauri Commands + 前端接入
- **轰炸目标**: 新增 `create_dir`、`rename_path`、`delete_path` 三个专用 Tauri commands，并把 `createNewFolder()`、`renameFile()`、`deleteFile()` 从 `run_command('mkdir/mv/rm')` 切换到专用 command
- **任务性质**: Bug 修复 + 安全约束下的功能恢复
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 前后端文件操作可用 + 不扩大通用命令白名单 + JS/Rust 基础验证通过
- **通用铁律**:
  1. 严禁把 `mkdir`、`mv`、`rm` 加入通用 `run_command` 白名单
  2. 三个 command 必须复用 Day 2 安全 resolver
  3. 删除必须由前端明确确认后才调用后端
  4. rename 的源和目标都必须做 workspace containment 校验
  5. 成功后刷新文件树，失败时给用户可读错误

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 记录当前分支和 HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `CS-HAJIMI-004` 当前 `OPEN / P1` | 债务总表第 5.1 节 | 必须 |
| 后端入口 | `main.rs` command 和 handler 区域 | `src/interface/desktop/src/main.rs:166-229`, `:1864-1908` | 必须 |
| 前端入口 | 三个 UI 操作函数 | `src/interface/web/app.js:786`, `:1018`, `:1038` | 必须 |
| 当前错配 | 前端调用 `run_command('mkdir/mv/rm')`，后端白名单不允许 | `rg -n "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js` | 必须 |
| 安全依赖 | Day 2 `resolve_workspace_path` 已存在 | `rg -n "resolve_workspace_path|PathIntent" src/interface/desktop/src/main.rs` | 必须 |
| 禁止方案 | 不允许将 `mkdir/mv/rm` 加入白名单 | `rg -n "mkdir|mv|rm" src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs` | 必须 |
| 文档同步 | 状态最多 `OPEN -> VERIFY` | 当前债务总表或 Day receipt | 按需 |

### 探索补充栏

本任务不是探索任务。唯一可选设计点是 `delete_path` 的 `recursive` 参数命名与行为：推荐 `recursive: bool`，目录删除必须由前端确认后传 `true`，后端仍需做安全 resolver。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-03/15
- **角色**: Engineer
- **目标**: 恢复新建文件夹、重命名、删除三类文件操作，同时保持命令白名单收紧
- **输入**: Day 2 resolver、`main.rs` command handler、`app.js` 三个函数
- **依赖关系**: 依赖 Day 2 resolver；不得抢修 Day 4 Shell 白名单

### 2）输出交付物

- **变更文件**:
  - `src/interface/desktop/src/main.rs`
  - `src/interface/web/app.js`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`，如状态同步需要
- **核心修改点**:
  - 新增 `#[tauri::command] fn create_dir(path: &str, app_handle: tauri::AppHandle) -> Result<(), String>`
  - 新增 `rename_path(old_path, new_path, app_handle) -> Result<(), String>`
  - 新增 `delete_path(path, recursive, app_handle) -> Result<(), String>`
  - `generate_handler![]` 注册三项
  - 前端三处从 `run_command` 切到 `tauri.core.invoke('create_dir'|'rename_path'|'delete_path')`
- **必须包含**:
  - `create_dir` 使用 `PathIntent::NewDir`
  - `rename_path` 源路径使用 existing intent，目标路径使用 new path parent 校验
  - `delete_path` 使用 existing intent，目录递归删除必须尊重 `recursive`
  - 成功后调用 `loadFileTree()` 或等价刷新
- **禁止包含**:
  - 后端 `run_command` 新增 `mkdir/mv/rm`
  - 前端保留 `cmd: 'mkdir'`, `cmd: 'mv'`, `cmd: 'rm'`
  - 后端跳过 resolver 直接 `std::fs::remove_dir_all(path)`
  - 删除无确认
- **交付证明**:
  - `cargo check -p hajimi-desktop`
  - `node --check src/interface/web/app.js`
  - `rg` 确认 command 注册和旧调用消失
  - 手动或自动文件操作 receipt

### 3）规模与复杂度观察

- **推荐目标**: 三个 command 单职责，错误处理共用小 helper
- **复杂度说明**: 删除目录和文件的分支可留在 `delete_path` 内；若引入复杂回收站/备份机制，需声明超出本日范围
- **禁止行为**: 通过放开通用 shell 命令来“快速修好按钮”

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | Rust 格式通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增明显 warning | `cargo clippy -p hajimi-desktop -- -D warnings` 或 `N/A + 原因` | 返工或声明债务 |
| TEST | JS 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| ARCH | 不扩大 Shell 权限 | `rg -n "mkdir|mv|rm" src/engine/tool-system/src/shell.rs` 并人工确认未新增白名单 | 返工 |
| REAL | 前端旧调用消失 | `rg -n "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js` 应无命中 | 返工 |
| DOC | 状态更新有 receipt | 债务总表 diff 或 Day receipt | 返工或声明无文档改动 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `create_dir` command 存在 | `rg -n "fn create_dir|create_dir" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-002 | `rename_path` command 存在 | `rg -n "fn rename_path|rename_path" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-003 | `delete_path` command 存在 | `rg -n "fn delete_path|delete_path" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-004 | 三个 command 已注册 | `rg -n "generate_handler|create_dir|rename_path|delete_path" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-001 | 三个 command 复用 resolver | `rg -n "resolve_workspace_path|PathIntent::NewDir|PathIntent::AnyExisting" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-002 | 前端新建调用专用 command | `rg -n "createNewFolder|create_dir" src/interface/web/app.js` | [ ] |
| CONST | CONST-003 | 前端重命名调用专用 command | `rg -n "renameFile|rename_path" src/interface/web/app.js` | [ ] |
| CONST | CONST-004 | 前端删除调用专用 command | `rg -n "deleteFile|delete_path" src/interface/web/app.js` | [ ] |
| NEG | NEG-001 | 旧 `mkdir` 调用消失 | `rg -n "cmd: 'mkdir'" src/interface/web/app.js` 无命中 | [ ] |
| NEG | NEG-002 | 旧 `mv` 调用消失 | `rg -n "cmd: 'mv'" src/interface/web/app.js` 无命中 | [ ] |
| NEG | NEG-003 | 旧 `rm` 调用消失 | `rg -n "cmd: 'rm'" src/interface/web/app.js` 无命中 | [ ] |
| NEG | NEG-004 | 白名单未加入 `mkdir/mv/rm` | `rg -n "\"mkdir\"|\"mv\"|\"rm\"" src/engine/tool-system/src/shell.rs` 应无新增允许项 | [ ] |
| UX | UX-001 | 删除前有明确确认 | `rg -n "confirm\\(|deleteFile" src/interface/web/app.js` | [ ] |
| UX | UX-002 | 成功后刷新文件树 | `rg -n "loadFileTree\\(" src/interface/web/app.js` | [ ] |
| E2E | E2E-001 | Rust 与 JS 基础验证通过 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | [ ] |
| High | HIGH-001 | symlink 越界路径被拒绝 | 复用 Day 2 测试或手动 receipt: `link -> outside` 后 create/rename/delete 均拒绝 | [ ] |

---

## 【模块3-B】地狱红线

1. 加白名单 `mkdir/mv/rm`，返工
2. 只修新建文件夹，漏掉重命名和删除，返工
3. command 未注册导致前端无法 invoke，返工
4. 后端 command 不走安全 resolver，返工
5. 删除没有确认，返工
6. 成功后不刷新文件树且未说明，返工
7. `node --check` 失败仍收卷，返工
8. `cargo check -p hajimi-desktop` 失败仍收卷，返工
9. 越界路径没有验证，返工
10. 把状态直接标成 `CLEARED` 但无实机 receipt，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 新建/重命名/删除三条主路径是否可用 | [ ] | FUNC-001~004 | |
| RG | 旧 `run_command` 错配是否移除 | [ ] | NEG-001~003 | |
| NG | 越界和 symlink 是否拒绝 | [ ] | HIGH-001 | |
| UX | 删除确认和刷新是否存在 | [ ] | UX-001~002 | |
| E2E | Rust/JS 是否都通过 | [ ] | E2E-001 | |
| High | 是否未放大 Shell 权限 | [ ] | NEG-004 | |
| 字段完整性 | receipt 是否含命令输出 | [ ] | 收卷报告 | |
| 需求映射 | 是否映射 `CS-HAJIMI-004` | [ ] | 债务总表 5.1 | |
| 自测执行 | 是否至少手动验证一次 UI | [ ] | 手动 receipt | |
| 范围边界与债务 | 是否未触碰 Day 4 Shell 修复 | [ ] | git diff | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-03/15 完成并提交

### 提交信息
- Commit: `fix(interface/web): route file operations through dedicated tauri commands`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/desktop/src/main.rs`
  - `src/interface/web/app.js`

### 本轮目标与实际结果
- 目标: 修复 `mkdir/mv/rm` 文件操作错配
- 实际完成: `<列出三个 command 和三处前端替换>`
- 未完成/不在范围: ShellTool 白名单收紧属 Day 4

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- `rg -n "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" ...`: `<摘要>`

### 债务声明
- `DEBT-TEST-B03-001`: `<若未完成实机 Tauri dev 验证，写清原因与补验日期>`

### 风险与回滚点
- 主要风险: 文件删除误操作
- 回滚方式: `git restore src/interface/desktop/src/main.rs src/interface/web/app.js`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | Day 2 resolver 不存在或不可靠 | 暂停文件操作 command，先回补 Day 2 | 不交付半安全 command |
| QUALITY-001 | 前端 invoke 参数与后端签名不一致 | 停止扩展，修签名和注册 | 返工 |
| TEST-001 | 无法启动 Tauri dev | 保留自动化验证，并写手动待验 debt | 有条件交付 |
| SAFETY-001 | 删除逻辑可能越界或无确认 | 禁用 delete UI 或回退删除功能 | 不带风险上线 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 03 文件操作专用 Tauri Commands + 前端接入**。

### 关键约束
- 禁止把 `mkdir/mv/rm` 加入通用命令白名单
- 三个 command 必须走 Day 2 resolver
- 删除必须有确认
- 前端旧 `run_command('mkdir/mv/rm')` 必须消失

### 验收铁律
- `cargo check -p hajimi-desktop` 通过
- `node --check src/interface/web/app.js` 通过
- `create_dir/rename_path/delete_path` 已注册
- 文件操作错配状态最多推进到 `VERIFY`

闭环启动，Day 03，执行。
