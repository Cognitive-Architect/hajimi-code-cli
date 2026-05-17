# DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED: Day 13 WebView / 文件树点击验收阻塞

> 创建日期: 2026-05-17  
> 关联工单: `docs/roadmap/hajimi debtFix/task/Day-13-Frontend-SecurityDom-Workspace-Modules.md`  
> 关联审计: `audit report/HAJIMI-DEBTFIX-DAY13-AUDIT-REPORT.md`  
> 当前状态: `OPEN / BLOCKED`  
> 优先级: `P1`  

---

## 1. 债务摘要

Day 13 已完成前端 `security-dom.js` 与 `workspace.js` 的渐进拆分，并通过静态检查、desktop 编译与模块级 smoke；但未能完成真实 Tauri WebView 中的文件树点击验收，因此不能把 Day 13 UI 交互链路标记为完全 A 级闭环。

阻塞项集中在真实窗口 smoke：

1. Tauri dev 启动需要 `http://localhost:3456/` 前端服务。
2. 当前环境中直接运行 `cargo tauri dev` 后，Tauri 持续等待该 dev server，180 秒后失败。
3. 尝试启动临时本地静态 server 后，`Invoke-WebRequest http://127.0.0.1:3456/` 返回“目标计算机积极拒绝，无法连接”，未拿到可供 Tauri WebView 加载的前端入口。
4. 因此没有形成真实窗口级证据来证明右键菜单、新建文件夹、重命名、删除、刷新文件树在 WebView 内完整可点通。

---

## 2. 已完成并可复现的证据

### 2.1 代码拆分与接入

- `src/interface/web/modules/security-dom.js`
  - 挂载 `window.HajimiSecurityDom`
  - 暴露 `safeText`、`escapeHtml`、`escapeAttr`、`setSafeHtml`
- `src/interface/web/modules/workspace.js`
  - 挂载 `window.HajimiWorkspace`
  - 承接 `initWorkspace`、`loadFileTree`、`buildTreeFromEntries`、`renderFileTree`、`renderTreeNode`、`createNewFolder`、`renameFile`、`deleteFile`、`collapseAllFolders`
- `src/interface/web/app.js`
  - 保留旧方法名 wrapper
  - 旧事件绑定仍可调用 `this.loadFileTree()` / `this.createNewFolder()` / `this.renameFile()` / `this.deleteFile()`
- `src/interface/web/index.html`
  - 使用普通 `defer` script 顺序加载 `security-dom.js`、`workspace.js`、`app.js`

### 2.2 已通过命令

```powershell
node --check src/interface/web/app.js
node --check src/interface/web/modules/security-dom.js
node --check src/interface/web/modules/workspace.js
node --check tests/frontend/day13_workspace_modules_smoke.js
node tests/frontend/day13_workspace_modules_smoke.js
cargo check -p hajimi-desktop
git diff --check -- src/interface/web/app.js src/interface/web/index.html src/interface/web/modules/security-dom.js src/interface/web/modules/workspace.js tests/frontend/day13_workspace_modules_smoke.js src/ARCHITECTURE.md docs/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md docs/debt/FRONTEND-MODULES-B13-RECEIPT.md "audit report/HAJIMI-DEBTFIX-DAY13-AUDIT-REPORT.md"
```

已观测结果：

- `node --check` 全部退出码 0。
- `node tests/frontend/day13_workspace_modules_smoke.js` 输出 `day13 workspace/security modules smoke: PASS`。
- `cargo check -p hajimi-desktop` 输出 `Finished dev profile`。
- `git diff --check` 退出码 0，仅有 CRLF warning。
- `rg` 扫描未发现 `cmd: 'mkdir'` / `cmd: 'mv'` / `cmd: 'rm'` 文件操作回归。
- `rg` 扫描未发现新增 React / Vue / Vite / webpack / ESM import 改造。

---

## 3. 阻塞复现记录

### 3.1 直接运行 Tauri dev

命令：

```powershell
cargo tauri dev
```

工作目录：

```text
F:\hajimi-code-cli\src\interface\desktop
```

结果摘要：

```text
Warn Waiting for your frontend dev server to start on http://localhost:3456/...
Error Could not connect to `http://localhost:3456/` after 180s.
```

判断：

- 不是 `hajimi-desktop` 编译失败。
- 阻塞在 Tauri devUrl 对本地前端服务的等待。

### 3.2 尝试临时静态 server

尝试用 Node 启动本地静态 server 后，再访问：

```powershell
Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3456/
```

结果摘要：

```text
由于目标计算机积极拒绝，无法连接。
```

判断：

- 当前没有拿到可靠的本地前端服务监听。
- 未继续伪造 Tauri 窗口 smoke 结果。

---

## 4. 影响范围

### 已经可信的范围

- 模块文件存在并可被浏览器普通 script 方式加载。
- security helper 的文本与属性 escape 行为已有模块级验证。
- workspace 模块会调用 `get_current_workspace`、`list_dir`、`create_dir`、`rename_path`、`delete_path`。
- 文件操作没有回退到 `run_command` 的 `mkdir` / `mv` / `rm`。
- desktop crate 编译通过。

### 尚未可信的范围

- 真实 Tauri WebView 内加载 `index.html` 后是否有控制台错误。
- 文件树真实 DOM 展示效果。
- 右键菜单在 WebView 内是否正确弹出。
- 新建文件夹、重命名、删除在真实窗口点击后是否刷新文件树。
- prompt / confirm 在 Tauri WebView 中的交互行为。

---

## 5. 关闭条件

要关闭此债务，至少需要补一份 WebView smoke receipt，包含：

1. 成功启动 `cargo tauri dev`，或等价的 Tauri WebView 加载路径。
2. 前端入口 `/`、`/modules/security-dom.js`、`/modules/workspace.js`、`/app.js` 被成功请求。
3. WebView 控制台无 `HajimiSecurityDom is undefined` / `HajimiWorkspace is undefined` / script loading error。
4. 文件树能显示当前 workspace。
5. 在真实窗口中执行：
   - 新建文件夹
   - 重命名
   - 删除
   - 刷新文件树
6. 操作后确认后端仍调用专用 commands：
   - `create_dir`
   - `rename_path`
   - `delete_path`
7. 截图或日志归档到 `docs/receipts/` 或 `docs/debt/`，并在审计报告中引用。

---

## 6. 当前建议

Day 13 的代码交付可以继续保留，评级建议维持“代码 A / 真实窗口 smoke 未闭环”。后续若要正式达到全链路 A 级，应先解决 `localhost:3456` 前端服务可达性，再补真实 Tauri WebView 点击验收。
