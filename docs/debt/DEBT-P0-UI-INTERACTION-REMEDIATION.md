# 技术债务声明：UI-INTERACTION-REMEDIATION (Phase 5)

## 1. 债务标识
- **ID**: DEBT-P0-UI-INTERACTION-REMEDIATION
- **级别**: P0 (架构级硬约束)
- **模块**: Interface Layer (`src/interface/web/`)
- **状态**: Active
- **记录日期**: 2026-05-15
- **关联阶段**: HAJIMI-UI-INTERACTION-CORE Day 1-10

## 2. 债务背景
在 Hajimi IDE v1 的界面改造过程中（Day 1-10），我们实行了“无框架引入”的极端策略（Protected DOM Contract）。所有 UI 交互、事件绑定、面板渲染均通过原生 vanilla HTML/CSS/JS 实现。

该策略保证了 Phase 5 不改变构建链、不引入 React/Vue/Vite/Webpack、不扰动 Tauri/Rust 后端边界；代价是 Interface 层前端文件继续集中增长。

## 3. 产生代价与风险
- **DOM 操作冗长**: `app.js` 行数逼近 5000 行，事件委托和 DOM 查找非常脆弱。
- **状态管理困难**: 依赖全局 `window.app` 单例，极易发生状态与 DOM 不同步。
- **样式冲突**: `style.css` 已超过 3200 行，层叠覆盖较难维护。
- **交互回归风险**: Settings / Inspector / Command Palette 之间共享 DOM ID 与事件绑定，局部移动节点时容易漏改 JS 选择器。
- **证据分散风险**: Day 1-10 的 receipts 是验证证据，不应作为唯一开发入口；正式入口已经同步到 `src/INDEX.md` 和 `src/ARCHITECTURE.md`。

## 4. 当前可接受边界
- Phase 5 只收口 UI interaction core，不在本阶段重构前端工程化。
- `index.html`、`app.js`、`style.css` 仍是单体前端核心文件。
- `Task Steps` 与 `Edit Summary` 只完成前端结构化入口；真实后端流式数据接入属于后续工作。
- Day 10 未新增 bitmap screenshot，使用明确占位记录；视觉参考沿用 Day 8/Day 9 receipts。

## 5. 触发升级条件
- `app.js` 因新增功能再增长超过 500 行且没有拆分计划。
- 新增 UI 节点删除或更名 DOM ID，但未同步事件绑定和 reference map。
- Settings / Inspector / Command Palette 任一关键入口出现不可达功能。
- 新增前端交互使用静态假数据、硬编码成功态或未声明 mock/simulation。
- `node --check src/interface/web/app.js` 或 `git diff --check` 失败仍进入 handoff。

## 6. 偿还策略 / 解决前提
- 短期：继续维护 Protected DOM Contract，每次 UI 移动必须同步 `index.html`、`app.js`、`style.css` 与 `src/INDEX.md`。
- 中期：按功能拆分 `app.js`，优先拆出 `settings.js`、`inspector.js`、`agent-cards.js`、`command-palette.js`。
- 长期：在 v2 架构中评估轻量声明式前端方案；若引入 Svelte 等框架，必须先出 ADR 并证明不会破坏 Tauri v2 本地优先边界。
- 在偿还前，禁止在 `app.js` 中继续堆砌超过 500 行的新功能逻辑。

## 7. 验证命令
```powershell
node --check src\interface\web\app.js
cargo check --workspace
git diff --check
```

## 8. 正式入口
- 架构说明：`src/ARCHITECTURE.md`
- 源码索引：`src/INDEX.md`
- 验证证据：`docs/receipts/ui-interaction/`
