# HAJIMI-DEBTFIX Day 03 建设性审计报告

> 审计对象：`docs/roadmap/hajimi debtFix/task/Day-03-File-Ops-Dedicated-Commands.md`
> 审计官：压力怪
> 审计日期：2026-05-16
> 关联阶段：HAJIMI-DEBTFIX Phase Day 03
> 当前状态：A 级 / Go（2026-05-16 收尾复审）

---

## 审计背景

### 项目阶段

HAJIMI-DEBTFIX Day 03：文件操作专用 Tauri Commands + 前端接入。目标是修复 `CS-HAJIMI-004` 中前端 `mkdir/mv/rm` 与后端通用命令白名单不一致的问题，同时保持 Shell 白名单不扩大。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `main.rs` | `src/interface/desktop/src/main.rs` | 新增 `create_dir` / `rename_path` / `delete_path` 三个 Tauri commands，并注册到 `generate_handler!` | Engineer | 自动闸门通过 |
| 2 | `app.js` | `src/interface/web/app.js` | 将 `createNewFolder` / `renameFile` / `deleteFile` 改为调用专用 commands | Engineer | JS 语法通过 |
| 3 | `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 5.1 详情标为 `VERIFY`，但总览仍保留旧 `OPEN` 描述 | Engineer | 部分同步 |

### 关键代码片段

```rust
// 来自 src/interface/desktop/src/main.rs:285-311
#[tauri::command]
fn create_dir(path: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::NewDir)?;
    std::fs::create_dir_all(&safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_path(old_path: &str, new_path: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_old = resolve_workspace_path(old_path, &base_dir, PathIntent::AnyExisting)?;
    let safe_new = resolve_workspace_path(new_path, &base_dir, PathIntent::NewFile)?;
    std::fs::rename(&safe_old, &safe_new).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_path(path: &str, recursive: bool, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::AnyExisting)?;
    if recursive {
        std::fs::remove_dir_all(&safe_path).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(&safe_path).map_err(|e| e.to_string())
    }
}
```

```js
// 来自 src/interface/web/app.js:1040-1046
if (!confirm(`确定要删除 "${name}" 吗？`)) return;
...
await invoke('delete_path', { path, recursive: true });
this.loadFileTree();
```

### 已知限制/环境问题

- Cargo 命令在默认沙箱偶发 Windows `target` 写入权限问题；本审计按规则提权复跑关键 Rust 命令。
- 当前工作区仍包含既有 `src/MEMORY.md` 修改，不属于 Day 03 本次审计范围。
- `audit report/` 与 `docs/roadmap/` 被 `.gitignore` 忽略，提交报告和债务文档时需要 `git add -f`。

---

## 质量门禁

- 已读取 Day 03 工单、建设性审计模板、B-09 审计报告示例。
- 已抽查 `src/interface/desktop/src/main.rs` 中三个 command 的实现和 handler 注册。
- 已抽查 `src/interface/web/app.js` 中三个前端入口替换。
- 已执行 `node --check src/interface/web/app.js`、`cargo fmt -- --check`、`cargo check -p hajimi-desktop`、`cargo clippy -p hajimi-desktop -- -D warnings`、`cargo test -p hajimi-desktop`。
- 已验证前端旧 `cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'` 调用消失。
- 已验证 ShellTool 白名单没有新增 `"mkdir"`、`"mv"`、`"rm"`。

质量门禁满足出报告条件，但不满足 A 级放行条件。

---

## 审计目标

1. 三个专用 Tauri commands 是否存在、注册，并复用 Day 2 resolver？
2. 前端三处文件操作是否从 `run_command('mkdir/mv/rm')` 切到专用 command？
3. 删除确认、成功刷新、错误提示是否存在？
4. 新建、重命名、删除三条主路径是否都能成立，且债务文档是否和实际实现一致？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 后端 command 覆盖 | B | 三个 command 已存在并注册，均调用 resolver；但 `delete_path` 的文件/目录分支和前端参数不匹配。 |
| 前端替换完整性 | A | `createNewFolder` / `renameFile` / `deleteFile` 已改用 `create_dir` / `rename_path` / `delete_path`，旧 `mkdir/mv/rm` 调用无命中。 |
| Shell 权限边界 | A | `src/engine/tool-system/src/shell.rs` 未新增 `"mkdir"`、`"mv"`、`"rm"`。 |
| 自动化闸门 | A | `node --check`、`cargo fmt -- --check`、`cargo check`、`cargo clippy`、`cargo test` 均通过。 |
| 删除主路径正确性 | C | 前端始终传 `recursive: true`，后端对应无条件 `remove_dir_all`；删除普通文件路径会走目录删除 API。 |
| 越界/文件操作 receipt | C | 复用了 Day 2 resolver 测试，但未发现 create/rename/delete 三个 command 自身的手动或自动文件操作 receipt。 |
| 文档同步 | C | 债务总表总览仍写 `CS-HAJIMI-004` 为 `OPEN` 且旧 `mkdir/mv/rm` 描述，5.1 详情却写 `VERIFY`，前后冲突。 |

整体健康度评级：C 级。结构性迁移完成，但“删除文件”这个主用例存在实现缺口，且文档证据链不一致。

---

## 关键疑问回答（Q1-Q3）

**Q1：是否通过放开 `mkdir/mv/rm` 白名单来修功能？**

否。`rg -n '"mkdir"|"mv"|"rm"' src/engine/tool-system/src/shell.rs src/interface/desktop/src/main.rs` 无命中，旧前端 `cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'` 也无命中。这个方向是正确的。

**Q2：三条文件操作是否都能按工单主路径成立？**

不完全成立。新建文件夹和重命名的静态实现基本符合工单；删除路径有明显问题：前端 `deleteFile()` 对所有节点都传 `recursive: true`，后端 `delete_path()` 在 `recursive` 为 true 时只调用 `std::fs::remove_dir_all`。这会把普通文件删除也送进目录删除 API，和工单“新建/重命名/删除三条主路径可用”的目标不一致。

**Q3：债务状态是否可以推进到 `VERIFY`？**

暂不建议。5.1 详情已经写成 `VERIFY`，但总览仍写 `OPEN` 且保留旧观察；更重要的是删除普通文件主路径没有验收。应先修 `delete_path` 文件/目录分支并补 receipt，再把 `CS-HAJIMI-004` 统一推进到 `VERIFY`。

---

## 验证结果（V1-V14）

| 验证ID | 命令 | 结果 | 证据 |
|:---|:---|:---:|:---|
| V1 | `git branch --show-current` | PASS | `v3.8.0-batch-1` |
| V2 | `git rev-parse HEAD` | PASS | `d697414f42584a0d0c9c85346a6a692e691c4dad` |
| V3 | `rg -n "fn create_dir|fn rename_path|fn delete_path" src/interface/desktop/src/main.rs` | PASS | 三个 command 位于 `main.rs:285-303` |
| V4 | `rg -n "create_dir,|rename_path,|delete_path," src/interface/desktop/src/main.rs` | PASS | 三项已注册于 handler |
| V5 | `rg -n "resolve_workspace_path|PathIntent::NewDir|PathIntent::AnyExisting" src/interface/desktop/src/main.rs` | PASS | 三个 command 复用 resolver |
| V6 | `rg -n "create_dir|rename_path|delete_path" src/interface/web/app.js` | PASS | 前端三处已调用专用 command |
| V7 | `rg -n "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js` | PASS | 无命中 |
| V8 | `rg -n '"mkdir"|"mv"|"rm"' src/engine/tool-system/src/shell.rs src/interface/desktop/src/main.rs` | PASS | 无命中，未扩大白名单 |
| V9 | `rg -n "confirm\\(|deleteFile" src/interface/web/app.js` | PASS | `deleteFile()` 有确认弹窗 |
| V10 | `node --check src/interface/web/app.js` | PASS | 退出码 0 |
| V11 | `cargo fmt -- --check` | PASS | 退出码 0 |
| V12 | `cargo check -p hajimi-desktop` | PASS | 退出码 0 |
| V13 | `cargo clippy -p hajimi-desktop -- -D warnings` | PASS | 退出码 0 |
| V14 | `cargo test -p hajimi-desktop` | PASS | 8 passed，包含 Day 2 junction 外跳测试 |

---

## 问题与建议

### 必须返工

1. 修复 `delete_path`：后端应先判断 `safe_path.is_dir()` / `safe_path.is_file()`，目录按 `recursive` 决定 `remove_dir` 或 `remove_dir_all`，文件走 `remove_file`。不要让文件删除依赖 `recursive: false`，因为前端没有文件类型信息。
2. 补充 delete file / delete dir 的测试或手动 receipt。最小可接受证据：workspace 内创建 `file.txt` 和 `dir/`，分别调用后端逻辑后确认消失。
3. 补充 create/rename/delete 三个 command 的越界或 symlink 外跳 receipt，不能只引用 Day 2 resolver 测试就声明 Day 3 文件操作全部安全闭环。
4. 修正债务总表：总览矩阵与 5.1 详情必须一致；旧 `run_command('mkdir/mv/rm')` 观察应改成“历史缺陷 / Day 3 修复情况”，不要同时写 `OPEN` 和 `VERIFY`。

### 建议补强

- `rename_path` 目标目前统一使用 `PathIntent::NewFile`，目录重命名可以工作，但语义不够直观。可以保留作为“new path parent 校验”，但建议注释说明它同时用于目录 rename 的目标路径。
- 前端 `deleteFile(path)` 没有节点类型参数。Day 3 若不打算扩展 file tree node 类型，后端就必须承担 file/dir 自动分派。

---

## 评级结论

- 评级：C 级
- 状态：返工
- 与自测报告一致性：部分一致
- 地狱红线触发：是，删除主路径不完整，且缺少 Day 3 文件操作 receipt
- 是否需要返工：需要

---

## 压力怪评语

“这次不是方向错，是最后一米没走完。把 `mkdir/mv/rm` 从 shell 里拿出来是对的，resolver 也接上了；但删除文件会被送去 `remove_dir_all`，这就不是可用的删除功能。修掉这个，再补一张真实文件操作 receipt，Day 3 就有机会冲 A。”

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-DEBTFIX-DAY03-AUDIT-REPORT.md`
- 关联状态：HAJIMI-DEBTFIX Day 03
- 下一步建议：原始 C 级审计要求先返工 Day 03；2026-05-16 收尾复审已完成返工，当前可进入 Day 04。

---

## 收尾复审结论（2026-05-16）

### 复审范围

本次收尾针对原 C 级审计中的三项阻塞问题进行复核：

1. `delete_path(recursive=true)` 删除普通文件失败。
2. 缺少 create / rename / delete 文件操作级回归证据。
3. 债务总表中 `CS-HAJIMI-004` 总览与 5.1 详情状态冲突。

### 修复结果

| 原问题 | 收尾处理 | 结果 |
|:---|:---|:---:|
| 删除文件被错误送入 `remove_dir_all` | `delete_path()` 改为按 `safe_path.is_dir()` / `safe_path.is_file()` 分流；文件始终走 `remove_file` | PASS |
| 文件操作缺少 Day 3 自身测试 | 新增 5 个桌面端回归测试：创建目录、重命名文件、`recursive=true` 删除文件、递归删除目录、非递归删除非空目录失败 | PASS |
| 债务文档状态冲突 | `CS-HAJIMI-004` 总览改为 `VERIFY`；5.1 改写为历史缺陷、Day 3 修复、验证证据、剩余 UI smoke 关闭条件 | PASS |

### 复审验证

| 验证项 | 结果 |
|:---|:---:|
| `node --check src/interface/web/app.js` | PASS |
| `cargo fmt -- --check` | PASS |
| `cargo check -p hajimi-desktop` | PASS |
| `cargo clippy -p hajimi-desktop -- -D warnings` | PASS |
| `cargo test -p hajimi-desktop` | PASS，13 passed |
| `rg "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js` | PASS，无命中 |
| `rg '"mkdir"|"mv"|"rm"' src/engine/tool-system/src/shell.rs src/interface/desktop/src/main.rs` | PASS，无命中 |
| `git diff --check` | PASS，仅出现既有 CRLF 提示，无 whitespace error |

说明：Cargo 命令在默认沙箱内仍会遇到 Windows `target/debug` 写入 `拒绝访问`，已按流程提权复跑并通过。

### 最终评级

- 评级：A 级
- 状态：Go
- 债务状态建议：`CS-HAJIMI-004` 保持 `VERIFY`
- 剩余关闭条件：补一轮真实 Tauri UI smoke 后可考虑从 `VERIFY` 推进到 `CLEARED`

Day 3 当前已满足进入下一日工单的 A 级基线。
