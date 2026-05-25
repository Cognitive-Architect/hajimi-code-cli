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
| **Right Inspector** (右侧检查器) | 任务详情、Diff 预览、Agent Trace、Context 小票 | **无**（完全只读渲染） | 仅在旧 Diff 回退按钮处触发只读 RAG 命令 | `#rightInspector` 容器内部，涉及 15 个清晰的只读与状态面板元素 | **极低**。状态单一纯粹，职责完全聚焦于 RAG 上下文与 Trace 数据只读展现，是最佳架构解耦切片。 |
| **Agent Cards** (智能体卡片) | Chat Feed 中动态插入的文件预览、Diff 预览、步骤与摘要卡片 | **包含写入**（Diff 卡片的 Apply 按钮会触发 `apply_edits` 写入修改） | 中频，包含文件修改写入与 RAG 动作 | 无固定静态 DOM 父节点，动态生成并插入到 `#aiChatMessages` | **高**。事件监听动态绑定，且卡片生命周期受 AI 聊天会话流控管辖，过度解耦易导致流式渲染错乱。 |

### 9.2 目标切片：右侧检查器 (window.HajimiInspector)

经过评估，**Right Inspector** (右侧检查器) 状态最为纯粹、与后端无高危交互，被选为 Day 9 模块化提取的低风险切片。

#### A. 目标命名空间与 API 接口
在 `src/interface/web/modules/inspector.js` 中创建 IIFE 命名空间 `window.HajimiInspector`，提取并治理以下 17 个方法：
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

#### B. 治理 DOM IDs 与 Class 边界
`HajimiInspector` 严格管理且仅限管辖以下 DOM 边界：
- 容器 DOM: `#rightInspector`, `#inspectorTabs`, `#inspectorContent`
- Tab 点击节点: `.inspector-tab` (包含 `data-inspector-tab` 属性)
- 面板节点: `.inspector-panel` (包含 `data-inspector-panel` 属性)
- 动作按钮: `#inspectorCloseBtn`, `#refreshReceiptBtn`
- 渲染挂载点:
  - `#inspectorTaskStatus` (任务详情状态)
  - `#inspectorContextFiles` (上下文列表)
  - `#inspectorModelInfo` (模型状态)
  - `#inspectorTaskSteps` (会话统计)
  - `#inspectorEditSummary` (修改建议摘要)
  - `#inspectorDiffContent` (Diff 预览内容容器)
  - `#inspectorTraceContent` (Trace 步骤内容容器)
  - `#contextReceiptBody` (Context 小票容器)

#### C. 加载顺序 (Loading Sequence in index.html)
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

#### D. 后端兼容性与平滑代理层
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

#### E. 验证与回滚边界 (Verification & Rollback)
- **回滚文件列表**: Day 9 若发生任何回归故障，应无条件一键回滚以下文件：
  - `src/interface/web/app.js`
  - `src/interface/web/index.html`
- **基线验证命令**:
  - `node tests/frontend/day14_sessions_thinking_modules_smoke.js` (确保历史模块冒烟测试 100% 通过)
  - Day 9 新增 `node tests/frontend/day18_inspector_smoke.js` 测试，对 `HajimiInspector` 进行完全隔离测试。

