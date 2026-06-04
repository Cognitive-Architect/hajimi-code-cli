# Hajimi V1.5 — Provider Config 只读分片设计方案

> **项目**: Hajimi IDE v1  
> **文档版本**: v1.5.0-F  
> **目标**: 采样评估 `provider-config` 相关前端逻辑，制定安全抽离计划。  
> **注意**: 本轮不修改任何生产代码，仅输出调研方案与决策结论。  

---

## 1. 结论与决策

### A. 是否建议进入 Provider Config 只读展示抽离？
**结论：不建议在当前阶段进入实施。**  
通过对 `src/interface/web/app.js` 中 Provider 相关代码的详细采样发现，只读展示逻辑与高风险操作（API Key 交互、文件写入保存、删除确认、物理容量探测）在事件绑定和渲染层存在**强耦合（Strong Coupling）**。强行拆分极易破坏现有的 OS Keyring 密钥清理机制及探测取消状态机，风险较高。

### B. 如果建议，第一刀具体抽哪几个函数？
*若后续必须进行抽离，建议仅抽取与表单/Modal无关联的“顶部栏/侧边栏纯展示组件”*：
- `renderModelButton()` (更新顶部栏当前激活模型按钮文本)
- `renderSidebarModelSummary()` (更新侧栏模型与状态小绿点)
- `selectProvider(id)` (切换 activeProviderId 状态并触发局部渲染)

### C. 如果不建议，推荐转向哪个更安全的模块？
建议转向 **`command-palette`** (命令面板) 或 **`audit-log`** (审计日志) 模块。  
- **理由**：这两个模块的功能为 100% 只读或调用独立的无敏感状态 Tauri command，与密钥（API Key）管理、文件写入、容量探测完全解耦，拆分风险极低。

### D. 明确哪些函数禁止在下一刀移动？
为了保障密钥与网络安全，以下涉及写入、删除、密码管理、物理探针的核心函数**严禁移动**或剥离出主控 `app.js`：
- `saveProviderConfig()` (敏感表单读取与 Tauri 保存交互)
- `deleteProviderConfig(id)` (带 confirm 机制的删除命令)
- `exportProviderBackup(password)` / `importProviderBackup(password, filePath)` / `confirmBackup()` (备份导入导出及密码解密)
- `setupProviderSettings()` 内的 **Capacity Probe** 核心逻辑 (涉及异步定时器、取消信号 `activeContextProbeCancelled`、费用警告 Confirm 弹窗及 Tauri 探针调用)

---

## 2. 采样数据分析

### 2.1 相关函数列表及分类

在 `app.js` 中，共有 19 个与 Provider Config 直接相关的函数，分类如下：

| 函数名 | 职责类型 | 风险等级 | 说明 |
|:---|:---:|:---:|:---|
| `loadProviders()` | 只读数据加载 | Low | 获取 `providerConfigs` 并触发渲染 |
| `renderModelButton()` | 只读界面渲染 | Low | 渲染顶部栏激活模型名称 |
| `setupModelPicker()` | 交互事件绑定 | Low | 绑定模型选择器弹窗开关 |
| `openModelPicker()` | 只读界面渲染 | Low | 触发模型选择器渲染并显示 |
| `closeModelPicker()` | 界面状态变更 | Low | 隐藏模型选择器 |
| `renderModelPicker()` | 混合界面渲染 | Medium | 渲染选择列表，绑定 Use (只读)/Edit (只读)/Delete (高危) 按钮 |
| `selectProvider(id)` | 状态只读切换 | Low | 切换当前模型 ID，触发各区域文本刷新 |
| `renderProviderList()` | 混合界面渲染 | Medium | 渲染设置页中的 Provider 列表，包含数据源标签 |
| `setupProviderSettings()` | 强耦合绑定区 | **High** | **痛点所在**。同时绑定了 Preset 切换、Key 显示隐藏、备份 Modal 以及 **Context Capacity Probe**（容量探测）事件 |
| `openProviderModal(cfg)` | 只读/状态初始化 | Low | 填充表单数据，加载 Probe 结果 |
| `updateCapabilityStatusDisplay()` | 只读渲染 | Low | 显示 Verified/Stale/Fallback/Declared 状态与颜色 |
| `closeProviderModal()` | 状态重置 | Low | 重置表单，清空 `dataset.hasSavedKey` |
| `openBackupModal(mode)` | 界面切换 | Low | 弹出导入/导出密码框 |
| `closeBackupModal()` | 界面切换 | Low | 关闭导入/导出密码框 |
| `confirmBackup()` | 写入/网络 | **High** | 校验备份参数，分发导入导出 |
| `exportProviderBackup(pwd)` | 写入/密钥 | **High** | 调用 Tauri 导出加密备份文件 |
| `importProviderBackup(pwd, path)` | 写入/密钥 | **High** | 调用 Tauri 导入加密备份文件 |
| `saveProviderConfig()` | 写入/密钥 | **High** | 验证表单，写入 OS Keyring，**清空内存 API Key 敏感字段** |
| `editProviderConfig(id)` | 状态切换 | Low | 调起 Modal 编辑已有配置 |
| `deleteProviderConfig(id)` | 删除/高危 | **High** | 确认弹窗并调用 Tauri 删除配置及相关密钥 |

---

### 2.2 相关 DOM ID 与作用范围

| DOM ID | 所在区域 | 作用说明 | 只读/写/高危 |
|:---|:---|:---|:---:|
| `modelSelectBtn` | Window Top Bar | 顶部激活模型按钮，点击打开 picker | 只读 |
| `modelPickerModal` | Modal Overlay | 模型选择器模态背景 | 只读 |
| `modelPickerBody` | Modal Content | 渲染模型选择列表的容器 | 只读 |
| `modelPickerClose` | Modal Header | 关闭模型选择器按钮 | 只读 |
| `modelPickerAddBtn` | Modal Footer | "+ 添加" 按钮，点击关闭 picker 并打开 provider modal | 只读 |
| `providerListTab` | Settings Panel | 设置面板内 Provider 列表容器 | 只读 |
| `addProviderBtnTab` | Settings Panel | 设置面板内 "+ 添加" 按钮 | 只读 |
| `exportProviderBtnTab` | Settings Panel | 设置面板内 "导出" 备份按钮 | 写入 |
| `importProviderBtnTab` | Settings Panel | 设置面板内 "导入" 备份按钮 | 写入 |
| `providerModal` | Modal Overlay | 模型编辑/添加表单 Overlay | 只读 |
| `providerModalTitle` | Modal Header | 表单标题 (添加模型 / 编辑模型) | 只读 |
| `providerForm` | Modal Content | 模型配置表单 DOM 节点 | 只读 |
| `providerId` | Form Input | Provider 唯一标识符，编辑时 disabled | 只读 |
| `providerName` | Form Input | Provider 友好名称 | 只读 |
| `providerModalType` | Form Select | 提供商类型 (openai-compatible, ollama 等) | 只读 |
| `providerBaseUrl` | Form Input | API 基础路径 | 只读 |
| `providerModel` | Form Input | 模型名称 | 只读 |
| `providerApiKey` | Form Input | API 密钥输入框，密码类型 | **高危 (敏感密钥)** |
| `providerModalToggleKey` | Form Input | 密钥可见性切换眼睛图标 | 只读 |
| `providerModalBaseUrlPreset` | Form Select | Base URL 常用预设下拉框 | 只读 |
| `providerLongContextPreset` | Form Select | 长上下文规格预设下拉框 (128K, 200K, 1M) | 只读 |
| `providerMaxContext` | Form Input | 最大上下文 Token 限制 | 只读 |
| `providerMaxOutput` | Form Input | 最大输出 Token 限制 | 只读 |
| `providerReserveOutput` | Form Input | 保留输出 Token 大小 | 只读 |
| `providerSafetyMargin` | Form Input | 安全裕度 Token 大小 | 只读 |
| `providerRetrievalBudget` | Form Input | 检索预算 Token 大小 | 只读 |
| `providerLongContextMode` | Form Checkbox| 开启长上下文模式复选框 | 只读 |
| `providerCapabilityStatus` | Form Section | 显示该模型的容量探针状态 (Verified等) | 只读 |
| `testContextCapacityBtn` | Form Section | 启动容量探测按钮 | **高危 (产生高额计费)** |
| `cancelContextCapacityBtn` | Form Section | 取消容量探测按钮 | 界面交互 |
| `providerProbeLevelSelect` | Form Section | 选择探针档位 (128K, 256K, 512K, 900K) | 界面交互 |
| `providerProbeStatusDetails` | Form Section | 探针进度详情面板 | 只读 |
| `probeDetailStatus` | Form Section | 探针详细状态显示文本 | 只读 |
| `probeDetailTested` | Form Section | 已测试 Token 数展示 | 只读 |
| `probeDetailDeclared` | Form Section | 声明上限展示 | 只读 |
| `probeDetailLatency` | Form Section | 探针延时展示 | 只读 |
| `probeDetailError` | Form Section | 探针错误提示 | 只读 |
| `backupModal` | Modal Overlay | 备份输入密码 Modal Overlay | 只读 |
| `backupPassword` | Modal Input | 备份加密/解密密码框 | **高危 (敏感密钥)** |
| `backupFilePath` | Modal Input | 导入备份时的物理文件路径 | 写入 |
| `profileSelect` | Settings Panel | 切换配置 Profile 选项下拉框 | 写入 |
| `createProfileBtn` | Settings Panel | 创建 Profile 按钮 | 写入 |
| `deleteProfileBtn` | Settings Panel | 删除 Profile 按钮 (带 confirm) | **高危 (级联删除)** |
| `agentBindProviderTab` | Settings Panel | Agent 与模型绑定下拉框 | 只读 |
| `agentProviderListTab` | Settings Panel | Agent 模型绑定列表容器 | 只读 |
| `sidebarModelShortcut` | Sidebar Panel | 侧栏快捷修改模型按钮 | 只读 |
| `sidebarModelName` | Sidebar Panel | 侧栏当前模型名称 | 只读 |
| `sidebarProviderMeta` | Sidebar Panel | 侧栏模型类型与在线小绿点 | 只读 |
| `statusModel` | Status Bar | 底部状态栏当前模型名称 | 只读 |

---

### 2.3 涉及的 Tauri Command 接口

| 命令名 | 风险评级 | 涉及逻辑范围 |
|:---|:---:|:---|
| `get_provider_configs` | Low | 读取当前工作空间合并后的模型配置列表 |
| `get_probe_result` | Low | 读取指定模型历史保存的探针测试小票 |
| `list_profiles` | Low | 获取所有的 Profile 文件夹列表 |
| `get_active_profile` | Low | 读取当前激活的 Profile 名称 |
| `get_agent_providers` | Low | 读取 Agent 到模型的映射表 |
| `probe_provider_context_capacity` | **High** | 发起真实或模拟容量探针网络请求 |
| `add_provider_config` | **High** | 新增模型并将 API Key 写入 OS 钥匙圈 |
| `update_provider_config` | **High** | 更新模型并更新相关密钥 |
| `delete_provider_config` | **High** | 彻底删除模型配置及对应的密钥记录 |
| `export_provider_backup` | **High** | 用给定密码加密导出所有配置和密钥 |
| `import_provider_backup` | **High** | 用密码解密并导入配置与密钥 |
| `set_active_profile` | Medium | 切换当前激活配置 Profile |
| `create_profile` | Medium | 创建新 Profile 隔离目录 |
| `delete_profile` | **High** | 删除 Profile 并清理旗下所有模型的钥匙圈密钥 |
| `set_agent_provider` | Medium | 绑定或解绑指定 Agent 与模型配置 |

---

## 3. 强耦合与止损分析

### 3.1 `setupProviderSettings()` 中的超高耦合度
在 `app.js` 的 `setupProviderSettings`（第 3759-4028 行）中，不仅绑定了只读预设值（`providerLongContextPreset`），还通过嵌套函数直接处理了以下交互：
1.  **Capacity Probe (容量探测)**：
    *   直接绑定了 `#testContextCapacityBtn` 的点击事件。
    *   在事件内直接获取 `#providerId`、`#providerModel` 等表单值。
    *   包含 256K / 512K / 900K 挡位的 `confirm(...)` 费用警示逻辑。
    *   内嵌 `setInterval` 动画，直接依赖并修改 `this.activeContextProbeCancelled` 只读状态以支持中途取消。
    *   测试完成后直接通过 `this.invokeTauri('probe_provider_context_capacity')` 与后端网络通信，并再次调用 `this.loadProviders()` 刷新。
2.  **API Key 安全切换**：
    *   绑定眼睛图标点击事件，直接切换 `#providerApiKey` 的密码明文类型。
3.  **Backup 加密 Modal 交互**：
    *   绑定密码显示开关与备份导入导出确认。

### 3.2 渲染与高危指令的直接混杂
*   `renderModelPicker()` (模型选择器列表渲染) 直接内嵌了 `data-edit` 和 `data-delete` 按钮。
    *   点击 `delete` 按钮直接在事件监听器内调用 `confirm(...)` 阻断，并执行 `this.deleteProviderConfig(id)`。
*   `renderProviderList()` (设置面板列表渲染) 同理，直接通过 `querySelectorAll('[data-provider-delete]')` 绑定了物理删除函数。
*   如果将这两个渲染函数移至新模块，新模块将不得不依赖并调用主控的 `deleteProviderConfig` 和 `editProviderConfig` 写入函数，破坏了“只读模块”的纯洁度。

### 3.3 结论判定
基于止损条件：
> *如果采样发现只读展示与 API key / save / delete / probe 强耦合，立即停止，不进入实施。*

本方案建议**立即触发止损**，本轮不进入实施，未来亦应将 Provider 模块作为低优先重构目标。

---

## 4. 替代重构推荐

为了继续前端的去巨石化治理，推荐在下一轮优先对以下**低风险、低耦合的 100% 只读展示模块**进行抽离：

### 推荐候选 A: `command-palette` & `slash-palette`
*   **文件位置**：`src/interface/web/app.js` 中关于命令面板的绑定与检索过滤。
*   **状态**：100% 只读。其核心数据源均来自于 `command-palette-catalog.js` 和 `slash-command-catalog.js`，与密钥及写文件无关。
*   **优势**：极易隔离，无后向耦合风险。

### 推荐候选 B: `audit-log` (审计日志面板)
*   **文件位置**：`src/interface/web/modules/audit-log.js` (已部分拆分，但仍可进一步完善只读隔离)。
*   **状态**：100% 只读，仅渲染从 Tauri 拉取的日志数据。

---

## 5. 变更回滚与 Smoke 验证策略

### 5.1 回滚文件列表
如果任何实验性改动破坏了 Provider 的正常运作，必须一键回滚以下 Interface 层文件：
1.  `src/interface/web/app.js`
2.  `src/interface/web/index.html`
3.  `src/interface/web/style.css`

### 5.2 验证与 Smoke 检查清单
在未来实施任何 Provider 相关的提取动作后，必须手动验证以下功能正常：
- [ ] **顶部模型选择**：点击顶部栏 `#modelSelectBtn`，能正常拉起模型选择弹窗，点击 "使用" 能无缝切换激活模型，且底部状态栏及侧边栏能同步更新。
- [ ] **侧栏 summary 渲染**：切换 Profile 或模型后，侧边栏 `#sidebarModelName` 能显示正确的模型名，`#sidebarProviderMeta` 能正确展示 "类型 · 模型" 且在线状态小绿点保持正常。
- [ ] **Modal 初始化状态**：点击配置列表中任一模型的 "编辑" 按钮，弹出的 `#providerModal` 中参数及 API Key 占位符 (`API Key 已安全保存，留空则保持不变`) 应回填正常。
- [ ] **密码可见性切换**：在 Modal 内输入 API Key，点击眼睛图标，能在 `text` 与 `password` 状态间流畅切换。
- [ ] **探针安全验证**：
    *   在 Modal 中点击 "Test context capacity"，在 128K 档位下不弹出 confirm 直接开始动画。
    *   切换至 256K / 512K / 900K 档位点击，必须拦截并正确弹出费用警告 confirm。
    *   测试过程中点击 "取消"，能立即中断动画并使状态显示为 "已取消"。
