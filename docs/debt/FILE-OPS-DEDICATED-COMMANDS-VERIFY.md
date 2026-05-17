# Hajimi File Ops Dedicated Commands Verify (B-03/15)

> 日期: 2026-05-16  
> 分支: `v3.8.0-batch-1`  
> HEAD: `d697414f42584a0d0c9c85346a6a692e691c4dad`  
> 对应债务: `CS-HAJIMI-004` / `mkdir/mv/rm` 前后端错配  
> 范围: `src/interface/desktop/src/main.rs`, `src/interface/web/app.js`

---

## 1. 结论

Day 03 文件操作专用命令链路已验证通过。当前实现已具备:

- `create_dir(path)` Tauri command，使用 `PathIntent::NewDir`。
- `rename_path(old_path, new_path)` Tauri command，源路径使用 `PathIntent::AnyExisting`，目标路径使用 new-path parent 校验。
- `delete_path(path, recursive)` Tauri command，使用 `PathIntent::AnyExisting`，文件与目录按类型分流。
- 三个 command 已注册到 `tauri::generate_handler![]`。
- 前端 `createNewFolder()` / `renameFile()` / `deleteFile()` 已改用专用 command，不再走 `run_command('mkdir/mv/rm')`。
- 删除前仍有 `confirm()`，成功后调用 `loadFileTree()` 刷新。
- 通用 Shell 白名单没有加入 `mkdir/mv/rm`。

状态建议: `CS-HAJIMI-004` 保持 `VERIFY`，等待真实 Tauri UI smoke 后再进入 `CLEARED`。

---

## 2. 扫描 Receipt

### 后端 command 与注册

```text
rg -n "fn create_dir|fn rename_path|fn delete_path|create_dir|rename_path|delete_path|generate_handler|resolve_workspace_path|PathIntent::NewDir|PathIntent::AnyExisting" src/interface/desktop/src/main.rs

命中摘要:
resolve_workspace_path: 存在
create_dir: 存在并注册
rename_path: 存在并注册
delete_path: 存在并注册
PathIntent::NewDir / PathIntent::AnyExisting: 已用于文件操作链路
```

### 前端替换

```text
rg -n "createNewFolder|renameFile|deleteFile|create_dir|rename_path|delete_path|cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'|confirm\(|loadFileTree\(" src/interface/web/app.js

命中摘要:
createNewFolder -> invoke('create_dir', { path })
renameFile -> invoke('rename_path', { oldPath, newPath })
deleteFile -> confirm(...) -> invoke('delete_path', { path, recursive: true })
成功路径调用 loadFileTree()
旧 cmd: 'mkdir' / 'mv' / 'rm' 无命中
```

### Shell 白名单

```text
rg -n '"mkdir"|"mv"|"rm"' src/engine/tool-system/src/shell.rs src/interface/desktop/src/main.rs
结果: 无命中
```

---

## 3. 自动化验证

```text
node --check src/interface/web/app.js
结果: 通过

cargo fmt -- --check
结果: 通过

cargo check -p hajimi-desktop
结果: 通过；沙箱首次写 target 遇到 Windows 拒绝访问，提升权限后通过

cargo clippy -p hajimi-desktop -- -D warnings
结果: 通过；沙箱首次写 target 遇到 Windows 拒绝访问，提升权限后通过

cargo test -p hajimi-desktop
结果: 13 passed; 0 failed
```

`cargo test -p hajimi-desktop` 覆盖:

- existing file / dir resolver
- new file / dir resolver
- traversal / absolute outside reject
- missing parent reject
- parent symlink / junction escape reject
- create workspace dir
- rename workspace path
- remove file
- remove directory recursive
- reject non-empty directory without recursive

---

## 4. 手动待验

本轮未启动完整 Tauri UI smoke。需在 Day 07 或后续实机验收中补:

```text
1. 点击“新建文件夹” => 成功创建 workspace 内目录
2. 重命名 workspace 内文件/目录 => 成功
3. 删除 workspace 内文件/目录 => 弹出确认后成功
4. create_dir("../outside") => reject
5. workspace/link -> outside 后 create/rename/delete => reject
```

### `DEBT-TEST-B03-001`

原因: 当前轮次以自动化和静态验证为主，未运行 Tauri dev UI。  
状态: 自动化通过，UI smoke 待补。  
建议补验日期: Day 07 UX Startup/Filetree/Session Verify。

---

## 5. 风险与回滚

主要风险: 文件删除误操作。当前缓解:

- 前端删除前 `confirm()`
- 后端路径走 `resolve_workspace_path`
- 删除逻辑按文件 / 目录分支执行

回滚方式:

```text
git restore src/interface/desktop/src/main.rs src/interface/web/app.js docs/debt/FILE-OPS-DEDICATED-COMMANDS-VERIFY.md
```

