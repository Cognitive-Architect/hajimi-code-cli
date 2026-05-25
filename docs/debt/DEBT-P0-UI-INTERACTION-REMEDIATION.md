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

## 9. 前端模块化切片方案 (Day 8 Slice Plan)

为偿还 Interface 层前端单体文件的技术债务，在 Day 8 阶段对前端模块化切片进行了全面扫描与评估，并为 Day 9 提取模块规划了精确的架构蓝图。

### 9.1 前端候选切片对比矩阵 (Candidate Comparison)

| 候选模块 | 功能范围 | 写入/修改操作 | 后端/Tauri 交互 | DOM 边界与 IDs | 模块化风险评估 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Settings Panel** (设置面板) | 通用、模型、MCP、治理、审计五个 Tab 页设置 | 包含 LocalStorage 读写，以及大量**写入**命令（如 `save_provider`, `connect_mcp`, `pause_agent_loop`, `inject_memory`） | 高频，涉及 10+ Tauri 读写命令交互 | `#settingsPanel`, `#settingsTabs`, `#profileSelect`, `#settingTheme` 等 40+ 复杂跨域 DOM 节点 | **极高**。跨越模型、安全、MCP、治理等多业务状态，写操作复杂，极易发生状态不一致或数据丢失。 |
| **Right Inspector** (右侧检查器) | 任务详情、Diff 预览、Agent Trace、Context 小票 | **无写入**（只读渲染 + 只读 receipt refresh） | 仅包含旧 Diff 回退入口与 `get_latest_receipt` 只读小票读取，不触发 write/delete/update | `#rightInspector` 容器内部，涉及 18 个清晰的只读与状态面板元素 | **低**。状态较纯粹，职责聚焦于 RAG 上下文、Trace 与 Context Receipt 只读展现，是最佳架构解耦切片。 |
| **Agent Cards** (智能体卡片) | Chat Feed 中动态插入的文件预览、Diff 预览、步骤与摘要卡片 | **包含写入**（Diff 卡片的 Apply 按钮会触发 `apply_edits` 写入修改） | 中频，包含文件修改写入与 RAG 动作 | 无固定静态 DOM 父节点，动态生成并插入到 `#aiChatMessages` | **高**。事件监听动态绑定，且卡片生命周期受 AI 聊天会话流控管辖，过度解耦易导致流式渲染错乱。 |

### 9.2 目标切片：右侧检查器 (window.HajimiInspector)

经过评估，**Right Inspector** (右侧检查器) 状态最为纯粹、与后端无高危写入交互，被选为 Day 9 模块化提取的低风险切片。Context Receipt 属于 Inspector 内部只读面板，Day 9 不得遗漏其刷新按钮与小票渲染路径。

#### A. 目标命名空间与 API 接口
在 `src/interface/web/modules/inspector.js` 中创建 IIFE 命名空间 `window.HajimiInspector`，提取并治理以下 20 个方法：
1. `init(app)`: 初始化检查器 Tab 点击事件与关闭按钮监听。
2. `showInspectorTab(tabId)`: 切换检查器活动 Tab 页样式与可见性。
3. `withInspectorGuard(label, renderFn)`: 隔离渲染异常的保护装饰器。
4. `safeUpdateTaskDetails(statusText)`: 带保护地更新任务状态。
5. `safeRenderContextFiles()`: 带保护地渲染上下文文件。
6. `safeRenderModelInfo()`: 带保护地渲染活动模型。
7. `safeRenderInspectorDiffPreview()`: 带保护地渲染当前 Diff Hunks 详情。
8. `safeRenderTraceInspector()`: 带保护地渲染最近 15 步的 Agent Trace。
9. `openDiffPreview(file)`: 从编辑器或外部打开 Inspector 并聚焦于 Diff Tab。
10. `updateTaskDetails(statusText)`: 聚合更新任务状态、会话状态与 Token 状态。
11. `renderTaskSteps()`: 渲染会话步骤统计。
12. `renderEditSummary()`: 渲染待处理的修改建议摘要（Hunks 数量）。
13. `renderContextFiles()`: 生成上下文文件列表 HTML 并插入。
14. `renderModelInfo()`: 生成活动提供商/模型详情 HTML 并插入。
15. `renderInspectorDiffPreview()`: 核心 Diff 引擎，对比渲染 Hunks。
16. `renderDiffPreview()`: 回退代理方法。
17. `renderTraceInspector()`: 渲染 Agent Trace 迭代循环详情。
18. `setupReceiptPanel()`: 绑定 `#refreshReceiptBtn` 并触发初次只读小票加载。
19. `loadLatestReceipt()`: 通过 `invokeTauri('get_latest_receipt')` 读取最新 Context Receipt，不执行写入。
20. `renderContextReceiptPanel(receipt)`: 渲染 Context Receipt 估算 token、预算、省略块与免责声明。

#### B. 治理 DOM IDs 与 Class 边界
`HajimiInspector` 严格管理且仅限管辖以下 DOM 边界：
- 容器 DOM: `#rightInspector`, `#inspectorTabs`, `#inspectorContent`
- Tab 点击节点: `.inspector-tab` (包含 `data-inspector-tab` 属性)
- 面板节点: `.inspector-panel` (包含 `data-inspector-panel` 属性)
- 动作按钮: `#inspectorCloseBtn`, `#refreshReceiptBtn`
- 渲染挂载点:
  - `#inspectorTaskDetail` (任务详情面板)
  - `#inspectorTaskStatus` (任务详情状态)
  - `#inspectorContextFiles` (上下文列表)
  - `#inspectorModelInfo` (模型状态)
  - `#inspectorTaskSteps` (会话统计)
  - `#inspectorEditSummary` (修改建议摘要)
  - `#inspectorDiffContent` (Diff 预览内容容器)
  - `#inspectorTraceContent` (Trace 步骤内容容器)
  - `#contextReceiptTab` (Context 小票 Tab)
  - `#contextReceiptPanel` (Context 小票面板)
  - `#contextReceiptBody` (Context 小票容器)

#### C. 后端交互边界
- 允许：`loadLatestReceipt()` 调用 `get_latest_receipt`，该路径只读取最近 Context Receipt。
- 允许：旧 Diff 回退按钮沿用现有只读/预览入口。
- 禁止：Day 9 在 `HajimiInspector` 中新增 `write_file`、`apply_edits`、`delete_*`、`update_*`、`save_*` 等写入命令。
- 禁止：把 Settings、Provider 配置、Agent Cards 或 Chat stream 生命周期代码并入 `inspector.js`。

#### D. 加载顺序 (Loading Sequence in index.html)
为了保证依赖正确，`inspector.js` 应在 Tauri 适配层加载完毕后、主入口加载前加载：
```html
  <script defer src="modules/security-dom.js"></script>
  <script defer src="modules/workspace.js"></script>
  <script defer src="modules/sessions.js"></script>
  <script defer src="modules/thinking-ui.js"></script>
  <script defer src="modules/slash-palette.js"></script>
  <script defer src="modules/tauri-bridge.js"></script>
  <script defer src="modules/inspector.js"></script> <!-- 新增 -->
  <script defer src="app.js"></script>
```

#### E. 后端兼容性与平滑代理层
在 `app.js` 中保留同名方法代理，将 `window.app.setupInspector` 等调用无缝重定向到 `HajimiInspector` 上，以确保主干逻辑零修改：
```javascript
  setupInspector() {
    window.HajimiInspector.init(this);
  },
  showInspectorTab(tabId) {
    window.HajimiInspector.showInspectorTab(tabId);
  }
  // ... 其他方法同理重定向
```

Day 9 wrapper 必须至少覆盖：
- `setupInspector`
- `showInspectorTab`
- `safeUpdateTaskDetails`
- `safeRenderContextFiles`
- `safeRenderModelInfo`
- `safeRenderInspectorDiffPreview`
- `safeRenderTraceInspector`
- `openDiffPreview`
- `updateTaskDetails`
- `renderTaskSteps`
- `renderEditSummary`
- `renderContextFiles`
- `renderModelInfo`
- `renderInspectorDiffPreview`
- `renderDiffPreview`
- `renderTraceInspector`
- `setupReceiptPanel`
- `loadLatestReceipt`
- `renderContextReceiptPanel`

#### F. 验证与回滚边界 (Verification & Rollback)
- **回滚文件列表**: Day 9 若发生任何回归故障，应无条件一键回滚以下文件：
  - `src/interface/web/app.js`
  - `src/interface/web/index.html`
  - `src/interface/web/modules/inspector.js`
  - `tests/frontend/day18_inspector_smoke.js`
- **基线验证命令**:
  - `node --check src/interface/web/app.js`
  - `node --check src/interface/web/modules/inspector.js` (Day 9 新增后执行)
  - `node tests/frontend/day13_workspace_modules_smoke.js`
  - `node tests/frontend/day14_sessions_thinking_modules_smoke.js`
  - `node tests/frontend/day17_thinking_ui_v2_security_smoke.js`
  - `node tests/frontend/day18_inspector_smoke.js` (Day 9 新增后执行，覆盖 tab 切换、关闭按钮、Diff fallback、Context Receipt refresh)
  - `npm run test:security-gate`
  - `git diff --check`

#### G. Day 8 范围收卷说明
- Day 8 的正式交付物是本节切片方案；不以减少 `app.js` 行数作为成功标准。
- 本分支同时带有前序 baseline 稳定化修正：thinking parser 生命周期、Day13/Day14 smoke 的 Tauri bridge stub、Day4/Day7 debt 文档同步。上述修正用于保证 Day 8 baseline smoke 可复现，不属于 `HajimiInspector` 提取范围，Day 9 不应继续扩大到 thinking stream 或 provider/long-context 文档。
- WebView manual smoke 仍为开放债务：Node smoke 不能替代真实 Tauri 窗口点击。Day 9 提取完成后必须在真实窗口手动确认 Inspector Tab、关闭按钮、Context Receipt 刷新、旧 Diff fallback 均可用。
- `DEBT-SCOPE-DFV5-DAY08`: 已完成候选扫描、切片选择、wrapper/DOM/load-order/rollback/verification 边界；未完成真实模块提取，Day 9 才允许新增 `modules/inspector.js`。

#### H. Day 9 模块化第一阶段提取完成结果
- **实际完成状态**: 已成功创建 `src/interface/web/modules/inspector.js` 并将 20 个 Right Inspector / Context Receipt 只读渲染、切换、刷新方法从 `app.js` 物理抽取。
- **兼容性保障**: `app.js` 保留全部 20 个同名 forwarding wrappers，保证既有调用路径向下兼容。
- **安全检查门锁**: `tests/security/security_audit_allowlist.json` 已追加 `inspector.js` 的 allowlist entry；`npm run test:security-gate` 安全门锁已通过。
- **自动化测试**:
  - `node --check src/interface/web/modules/inspector.js` -> 🟢 PASS
  - `node --check src/interface/web/app.js` -> 🟢 PASS
  - `node tests/frontend/day13_workspace_modules_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day14_sessions_thinking_modules_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day16_slash_palette_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day17_thinking_ui_v2_security_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day18_inspector_smoke.js` -> 🟢 PASS
  - `cargo check -p hajimi-desktop` -> 🟢 PASS

#### I. Day 10 前端第二切片 Settings Panel 提取完成结果
- **路径选择**: `SECOND-SLICE`。Day 9 `inspector.js` 已通过 Day18 smoke / security gate / desktop check，因此 Day 10 继续提取第二切片，而非走 `REGRESSION-BUFFER`。
- **实际完成状态**: 🟢 已成功创建 `src/interface/web/modules/settings-panel.js` 并将 sidebar 切换与 settings tab 导航方法 (`showSidebar`, `setupSettingsTabs`, `switchSettingsTab`) 从 `app.js` 物理抽取。
- **低风险与兼容性保障**: `app.js` 保留了全部 3 个同名 forwarding wrappers，保证既有调用路径向下兼容。设置模块仅处理页面 DOM 类 toggle 与 sidebar 显示状态，所有的 Provider 添加、更新、删除等高内聚写逻辑完全保留在 `app.js` 原有位置，绝不越界。
- **安全免审合规**: 新增模块不使用任何 raw `innerHTML` 或 `insertAdjacentHTML`，完全满足 P0 级安全沙箱零新增安全隐患要求，无需修改 allowlist 文件。
- **自动化测试**:
  - `node --check src/interface/web/modules/settings-panel.js` -> 🟢 PASS
  - `node --check src/interface/web/app.js` -> 🟢 PASS
  - `node tests/frontend/day13_workspace_modules_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day14_sessions_thinking_modules_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day16_slash_palette_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day17_thinking_ui_v2_security_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day18_inspector_smoke.js` -> 🟢 PASS
  - `node tests/frontend/day19_settings_smoke.js` -> 🟢 PASS
  - `npm run test:security-gate` -> 🟢 PASS
  - `cargo check -p hajimi-desktop` -> 🟢 PASS
- **Day 11 closure input**:
  - 模块化状态：`inspector.js` 与 `settings-panel.js` 均已抽取，`app.js` 保留兼容 wrappers。
  - 自动化状态：前端语法、Day13/14/16/17/18/19 smoke、security gate、desktop check、`git diff --check` 均为绿灯。
  - 回滚点：`src/interface/web/app.js`、`src/interface/web/index.html`、`src/interface/web/modules/inspector.js`、`src/interface/web/modules/settings-panel.js`、`tests/frontend/day18_inspector_smoke.js`、`tests/frontend/day19_settings_smoke.js`。
  - Day 11 closure blocker：无自动化 blocker。
  - 残余风险：真实 Tauri WebView manual smoke 待补，需人工确认 Inspector Tab、Context Receipt refresh、Settings tab/sidebar navigation 在真实窗口内可点击。
