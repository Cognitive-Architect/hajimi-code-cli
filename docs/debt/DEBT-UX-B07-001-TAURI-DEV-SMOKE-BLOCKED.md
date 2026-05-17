# DEBT-UX-B07-001: Day 07 Tauri UI Smoke 被本地 dev 启动链路阻塞

> 创建日期: 2026-05-16  
> 关联工单: `docs/roadmap/hajimi debtFix/task/Day-07-UX-Startup-Filetree-Session-Verify.md`  
> 关联债务: `DEBT-UX-AGENT-001` / 启动、文件树、历史会话  
> 当前状态: `OPEN`  
> 优先级: P1  

---

## 1. 结论

Day 07 目标是实机验证启动、文件树、会话持久化、关闭重开和 Day 3 文件操作。当前无法 A 级收尾，因为 Tauri UI smoke 没有拿到真实窗口证据。

本次收尾复核确认:

1. `npm run dev` 不能作为稳定前端启动方式：`npx serve` 因离线缓存缺失失败。
2. 使用 Node 标准库临时静态 server 后，`http://127.0.0.1:3456` 可以返回 `200`。
3. 在 3456 可访问后执行 `cargo tauri dev`，编译 `hajimi-desktop` 时失败，错误为 Windows `target/debug/incremental` 路径 `拒绝访问 (os error 5)`。
4. 因未打开可操作 Tauri 窗口，无法证明:
   - 启动无“加载文件树失败”toast。
   - 文件树真实渲染 workspace。
   - 新建文件夹 / 重命名 / 删除在 UI 中可用。
   - 会话 A/B 切换与关闭重开后恢复可用。

因此 `DEBT-UX-AGENT-001` 必须保持 `VERIFY`，不得迁移到 `CLEARED`。

---

## 2. 复现记录

### 2.1 原始前端 dev server 失败

```text
cd src/interface/web
npm run dev
```

结果摘要:

```text
npx serve . -p 3456 -s
npm error code ENOTCACHED
npm error request to https://registry.npmjs.org/serve failed:
cache mode is 'only-if-cached' but no cached response is available.
```

说明: 当前环境网络受限，且 `serve` 不在 npm cache 中。

### 2.2 临时静态 server 可用

使用 Node 标准库临时 server 托管 `src/interface/web` 后:

```text
Invoke-WebRequest http://127.0.0.1:3456
```

结果:

```text
StatusCode: 200
```

日志:

```text
docs/day07-runtime/static-server.out.log
day07 static server listening on http://127.0.0.1:3456
```

### 2.3 Tauri dev 编译失败

```text
cd src/interface/desktop
cargo tauri dev
```

结果摘要:

```text
Running DevCommand (`cargo run --no-default-features --color always --`)
Compiling hajimi-desktop v0.1.0
error: failed to move dependency graph from ... dep-graph.part.bin to ... dep-graph.bin: 拒绝访问。 (os error 5)
error: unable to delete old query cache at ... query-cache.bin: 拒绝访问。 (os error 5)
error: could not compile `hajimi-desktop` (bin "hajimi-desktop") due to 2 previous errors
```

日志:

```text
docs/day07-runtime/tauri-dev.err.log
```

---

## 3. 已完成的非 GUI 验证

```text
node --check src/interface/web/app.js
cargo check -p hajimi-desktop
rg -n "invoke\('create_dir'|invoke\('rename_path'|invoke\('delete_path'" src/interface/web/app.js
rg -n "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js
git diff --check
```

结果:

- JS 语法通过。
- `cargo check -p hajimi-desktop` 通过。
- 前端已调用 Day 3 专用 commands。
- 未发现旧 `mkdir` / `mv` / `rm` 前端路径。
- `git diff --check` 无 whitespace error，仅既有 CRLF warning。

这些只能证明静态和编译层面健康，不能替代 UX 实机验收。

---

## 4. 未完成验收项

| 项目 | 当前状态 | 解除条件 |
|---|---|---|
| 启动无异常 toast | 未实测 | Tauri 窗口启动后截图或日志证明无“加载文件树失败”toast |
| 文件树真实加载 | 未实测 | 截图显示 workspace 文件树 |
| 新建文件夹 | 未实测 | UI 操作创建目录并刷新可见 |
| 重命名 | 未实测 | UI 操作重命名并刷新可见 |
| 删除确认与删除 | 未实测 | 截图或日志证明删除前确认，删除后刷新 |
| 会话 A/B 切换 | 未实测 | 截图或 localStorage 输出证明 A/B 消息可切换 |
| 关闭重开恢复 | 未实测 | 重启后 `hajimi_chat_sessions` 和 session list 仍保留 |

---

## 5. 修复建议

1. 固化前端 dev server 启动方式:
   - 选项 A: 将 `serve` 加入可离线安装的依赖并锁定。
   - 选项 B: 在 `tauri.conf.json` 配置稳定 `beforeDevCommand`。
   - 选项 C: 增加项目内无依赖静态 server 脚本，作为 Tauri dev smoke 专用入口。
2. 处理 Windows `target/debug/incremental` 权限问题:
   - 关闭残留 `cargo` / `hajimi-desktop` 进程后重试。
   - 必要时以提升权限重跑 `cargo tauri dev`。
   - 如果仍失败，清理该 crate 的 incremental 目录后重试，但清理前必须确认目标路径在 repo `target` 内。
3. 在可用窗口中补全 Day 07 UX receipt:
   - 截图目录建议: `docs/screenshots/day07-ux-verification/`
   - receipt 建议更新: `docs/roadmap/hajimi debtFix/debt/UX-FILETREE-SESSION-VERIFY.md`

---

## 6. 关闭标准

本债务只有在以下条件全部满足时才能关闭:

1. `http://127.0.0.1:3456` 或配置的 `devUrl` 可稳定访问。
2. `cargo tauri dev` 成功打开窗口。
3. 完成启动、文件树、Day 3 文件操作、会话 A/B、关闭重开全链路。
4. 证据写入 receipt，包含截图、日志或 localStorage 输出。
5. `DEBT-UX-AGENT-001` 的状态迁移有证据支撑。
