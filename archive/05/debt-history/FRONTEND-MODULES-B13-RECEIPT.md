# Frontend Modules B13 Receipt

日期: 2026-05-17
工单: B-13/15 Day-13-Frontend-SecurityDom-Workspace-Modules

## 目标

渐进拆分前端安全 DOM helper 与 workspace/file tree/file ops 高频逻辑，不引入 bundler，不删除 `window.app` 兼容入口。

## 交付文件

- `src/interface/web/modules/security-dom.js`
- `src/interface/web/modules/workspace.js`
- `src/interface/web/app.js`
- `src/interface/web/index.html`
- `src/ARCHITECTURE.md`
- `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`
- `docs/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`
- `tests/frontend/day13_workspace_modules_smoke.js`

## 模块边界

- `window.HajimiSecurityDom`: `safeText`, `escapeHtml`, `escapeAttr`, `setSafeHtml`
- `window.HajimiWorkspace`: `initWorkspace`, `loadFileTree`, `buildTreeFromEntries`, `renderFileTree`, `renderTreeNode`, `createNewFolder`, `renameFile`, `deleteFile`, `collapseAllFolders`
- `app.js`: 保留旧同名方法作为 wrapper，旧事件绑定继续调用 `this.loadFileTree()` / `this.createNewFolder()` / `this.renameFile()` / `this.deleteFile()`。

## 验证结果

- `node --check src/interface/web/app.js`: PASS
- `node --check src/interface/web/modules/security-dom.js`: PASS
- `node --check src/interface/web/modules/workspace.js`: PASS
- `cargo check -p hajimi-desktop`: PASS；普通沙箱首次因 Windows `target/debug` ACL 拒绝访问失败，提升权限重跑通过
- framework/bundler 扫描: `rg -n "React|Vue|Vite|webpack" src/interface/web package.json` 无命中
- ESM 扫描: `rg -n "import .* from|type=\"module\"" src/interface/web package.json` 无命中
- file ops 专用命令扫描: `workspace.js` 调用 `create_dir` / `rename_path` / `delete_path`；`cmd: 'mkdir'|'mv'|'rm'` 无命中
- workspace 模块级 smoke: PASS，覆盖 `get_current_workspace`, `list_dir`, `create_dir`, `rename_path`, `delete_path`
- security-dom 模块级 smoke: PASS，覆盖 null text、HTML escape、attribute escape
- `node tests/frontend/day13_workspace_modules_smoke.js`: PASS，固化 workspace/security 模块级 smoke，覆盖模块挂载、文件树构建、文件树渲染入口、文件操作专用 invoke、reload 请求、已打开 tab 关闭路径
- 根债务总表同步: `docs/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 已与 roadmap 债务总表同步为 `PARTIAL`
- `git diff --check`: PASS，仅有既有 CRLF warning

## 未完成 / 边界

- 未拆 sessions、thinking-ui、command-palette、slash-palette、`style.css`；属于 Day 14 或后续。
- 未完成真实 Tauri 窗口点击 smoke；本轮以 `cargo check`、脚本加载顺序、可复现模块级 smoke 和静态命令扫描作为证据。真实窗口点击验收仍归入后续 WebView smoke，不在本轮伪装关闭。阻塞详情见 `docs/debt/DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md`。
- `DEBT-COMPLEXITY-B13-001`: `app.js` 仍保留大量状态与 UI 方法，Day 13 仅完成 security-dom + workspace 的最小可拆边界。
