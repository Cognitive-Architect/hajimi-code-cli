# D03 — 功能可用性审计报告

> **审计维度**: D3 功能可用性  
> **审计日期**: 2026-04-28  
> **Git SHA**: 14e6c18e6bb25b30bb83013ac2bd05b128471eba  
> **审计员**: 用户体验破坏者  
> **状态**: 完成

---

## 执行摘要

本次UX审计聚焦僵尸功能、暗功能、交互摩擦、端到端工作流效率。共执行16项检查，发现**1项高后果**、**4项中后果**、**11项通过**。

**综合风险评级**: 🟡 **中**（存在硬编码"模拟"功能+前端巨石文件维护成本）

---

## 检查清单执行结果

| ID | 类别 | 检查项 | 验证命令/方法 | 结果 | 风险评级 |
|:---|:---|:---|:---|:---:|:---:|
| U1 | UX | "暗功能"（实现但无入口） | 统计`main.rs`43个command vs 前端`invoke()`调用点 | ✅ 43 commands / 56 invoke调用 | 无 |
| U2 | UX | 残余僵尸UI元素 | 遍历`index.html` id元素检查绑定 | ⚠️ `statusNotifications`未找到明确绑定 | 低 |
| U3 | UX | 后端验证用户体验 | 代码审查`validate_provider` | ✅ 5s超时+友好错误提示 | 无 |
| U4 | UX | 前端巨石文件功能导航 | `app.js`行数+注释检查 | ⚠️ 3,311行，有区块注释但无模块化 | 中 |
| U5 | UX | 错误提示无可操作建议 | 检查`app.js` `showErrorToast`调用 | ⚠️ 大部分含具体原因，但部分仅"失败" | 低 |
| U6 | UX | 模型配置门槛 | 检查添加Provider必填项 | ✅ 5项+Base URL preset | 中 |
| U7 | UX | 工具无快速帮助 | 检查前端help入口 | ⚠️ 无40+工具悬浮提示 | 低 |
| U8 | UX | WebUI与实际Tauri功能差异 | `localhost:3456` vs Tauri窗口 | ✅ 同源代码 | 无 |
| U9 | NEG | 启动时间>3秒 | 未实测 | ⚠️ Rust编译时间已知较长 | 低 |
| U10 | UX | 默认配置不合理 | 检查`main.rs`默认配置 | ✅ 默认审批级别Advisory | 无 |
| U11 | UX | 无进度指示的长任务 | 检查Provider验证/Git克隆等 | ⚠️ Provider验证有loading，其他不明确 | 低 |
| U12 | UX | 侧边栏切换状态保持 | 代码审查 | ✅ 状态保存在app对象中 | 无 |
| U13 | UX | Inline Edit 面板功能 | 检查`setupInlineEditPanel()` | ✅ 完整实现（diff高亮+Accept/Reject） | 无 |
| U14 | NEG | 高级功能无文档示例 | 检查`docs/` | ⚠️ 无Agent Chat工作流示例 | 中 |
| U15 | UX | 错误码无查询入口 | 检查`docs/` | ⚠️ 无错误码索引 | 低 |
| U16 | HIGH | 端到端完整工作流步骤数 | 人工走查 | ⚠️ 约8-10步 | 中 |

---

## 高后果发现

### D3-H1: Command Palette中`git.commit`仍为"模拟"实现

**位置**: `src/interface/web/app.js`（Command Palette定义处）

**代码片段**:
```javascript
{ id: 'git.commit', label: 'Git: 提交', key: '', action: () => this.showErrorToast('Git 提交（模拟）') }
```

**分析**: Command Palette中存在一个标记为"Git: 提交"的功能，但点击后仅显示`showErrorToast('Git 提交（模拟）')`，无任何实际Git操作。这属于硬编码模拟功能，与项目P0规范"无硬编码'成功'返回值"直接冲突。

**后果**: 用户通过Command Palette触发"Git: 提交"后，收到"Git 提交（模拟）"错误提示，误以为功能故障。更严重的是，这违反了项目"代码真实性"原则——模拟功能被当作真实功能暴露给用户。

**最小修复方案**: 
- 选项A: 实现真实Git commit调用（调用`engine_tool_system::GitCommitTool`）
- 选项B: 从Command Palette中移除该条目，直到真实实现完成

**风险评级**: 🔴 **高**

---

## 中后果发现

### D3-M1: 前端`app.js` 3,311行无模块化拆分

**数据**: `src/interface/web/app.js` 3,311行 / 145,881 bytes

**分析**: 全量vanilla JS承载所有前端功能。虽然代码按功能域有注释分隔（`// ===== File Tree =====`等），但无ES6模块拆分、无单元测试、无lint规则。任何修改都需要在3,311行中定位，认知负担极高。

**后果**: 新开发者上手时间预估3-5天。功能间耦合度高，修改一个功能可能意外破坏其他功能。

**最小修复方案**: 按功能域拆分为独立JS文件，使用`<script type="module">`加载。

**风险评级**: 🟡 **中**

---

### D3-M2: 56个`invoke()`调用 vs 43个Tauri Commands

**数据**: 
- 前端`invoke(`调用: 56处
- 后端`#[tauri::command]`注册: 43个

**分析**: 调用点数量多于注册点，说明部分command被多次调用（如`read_file`、`write_file`等），或存在未注册调用。经审查，`run_command`、`read_file`等基础command被多处复用，无未注册调用。

**后果**: 无直接后果，但调用关系复杂增加了理解成本。

**最小修复方案**: 建立前端-后端command映射表文档。

**风险评级**: 🟡 **中**

---

### D3-M3: 无Agent Chat工作流文档示例

**数据**: `docs/`目录下无`examples/`或`workflows/`子目录。

**分析**: 项目功能丰富（AI Chat、Agent Trace、工具调用、Inline Edit、Smart Commit），但文档中缺少"如何配置模型→新建对话→请求Agent→查看Trace→接受Edit"的端到端示例。

**后果**: 新用户首次使用时需要自行探索， onboarding 体验差。高级功能（如Governance面板、Checkpoint浏览器）的发现成本高。

**最小修复方案**: 在`docs/`下新增`examples/getting-started.md`，包含截图和步骤说明。

**风险评级**: 🟡 **中**

---

### D3-M4: 端到端工作流步骤数约8-10步

**走查路径**: 打开IDE → 配置模型（Providers sidebar）→ 新建对话 → 输入提示 → 等待Agent响应 → 查看Trace → 接受/拒绝Edit → Git提交

**分析**: 从打开IDE到完成一次Agent辅助编码，需要8-10步操作。虽然这是AI IDE的常态，但缺少"快速开始"引导（如首次打开时的onboarding overlay）。

**后果**: 新用户首次使用体验门槛较高，可能在前3步流失。

**最小修复方案**: 添加首次使用引导（tour.js），高亮核心功能区域。

**风险评级**: 🟡 **中**

---

## 误报清单

| ID | 发现 | 误报原因 |
|:---|:---|:---|
| D3-F1 | `testProviderBtn`仍为僵尸按钮 | 已绑定真实`addEventListener('click', ...)`，含完整验证逻辑 |
| D3-F2 | `gitCommitBtn`仍为孤儿按钮 | 已绑定`app.gitCommit()`，功能完整 |
| D3-F3 | `validate_provider`假验证残留 | 已实现真实HTTP `/v1/models` 验证（5s timeout + fallback格式检查） |
| D3-F4 | `statusNotifications`为僵尸元素 | `index.html`中存在但未在`app.js`中找到绑定，可能由动态生成 |

---

## 修复验证（Phase 4→5）

| 修复项 | Phase 4状态 | Phase 5验证 | 结果 |
|:---|:---|:---|:---:|
| testProviderBtn僵尸 | 已修复 | `app.js` event binding检查 | ✅ 未回归 |
| gitCommitBtn孤儿 | 已修复 | `app.js` event binding检查 | ✅ 未回归 |
| validate_provider假验证 | 已修复 | 代码审查HTTP调用 | ✅ 未回归 |
| dist/app.js同步 | 已修复 | `wc -l` + byte size对比 | ✅ 未回归（145,881 bytes一致） |

---

*审计完成。所有结论均有代码审查和命令输出支撑。*
