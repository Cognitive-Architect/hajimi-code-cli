# 02 - Slash Command 输入提示面板缺失

> 状态：已识别，待排期  
> 优先级：P1（高体验影响，中等实现成本）  
> 涉及范围：前端 UI  
> 关联：Issue #01（Token 统计）

---

## 问题描述

用户在聊天输入框中输入 `/` 后，**没有任何命令提示面板弹出**。Slash Command 功能只能通过"盲打 + Enter 发送"的方式使用，用户完全不知道有哪些命令可用。

### 现象对比

| 产品 | 输入 `/` 后的体验 |
|---|---|
| **Kimi Code** | 弹出浮动面板，显示命令列表 + 描述，支持 ↑↓ 选择、Tab 补全 |
| **Hajimi 当前** | 无任何反应，必须完整输入命令并发送后才能知道是否有效 |

---

## 根本原因

`setupChat()` 中 `chatInput` 仅绑定两个事件：

```javascript
// src/interface/web/app.js:L1885
chatInput.addEventListener('input', () => { /* 仅调整高度 */ });
chatInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) { /* 发送消息 */ }
});
```

**缺少：**
- `/` 键的检测逻辑
- 命令提示面板的渲染与定位
- 键盘导航（↑↓ / Tab / Esc）

Slash Command 的解析逻辑存在于 `sendChatMessage()` → `handleChatCommand()` 中，但仅在**消息发送后**执行，属于"事后判断"而非"输入时提示"。

---

## 当前实际可用的命令（盲打可用）

| 命令 | 功能 | 参数 |
|---|---|---|
| `/tools` | 列出 38 个可用工具 | 无 |
| `/providers` | 列出已配置的模型提供商 | 无 |
| `/tool <name> <args>` | 直接执行指定工具 | `name`: 工具名, `args`: JSON 参数 |
| `/chat <provider> <prompt>` | 切换提供商发送消息 | `provider`: 提供商 ID, `prompt`: 提示词 |
| `/mcp list` | 列出已连接的 MCP 服务器 | 无 |
| `/mcp init <url>` | 初始化 MCP 连接 | `url`: 服务器地址 |
| `/mcp invoke <url> <tool> [args]` | 调用 MCP 工具 | `url`, `tool`, `args` |
| `/search <pattern>` | 代码搜索（grep） | `pattern`: 搜索模式 |
| `/git status` | 查看 Git 状态 | 无 |
| `/git diff [file]` | 查看 Git 差异 | `file`: 可选文件路径 |
| `/git commit <message>` | 提交代码 | `message`: 提交信息 |
| `/extensions` | 列出已安装扩展 | 无 |

> 注：输入无效命令时，会返回可用命令列表（L2248），但这是发送后的反馈，非输入时提示。

---

## 与 Kimi Code 的差距

| 能力 | Kimi Code | Hajimi 当前 |
|---|---|---|
| `/` 触发面板 | ✅ | ❌ |
| 命令列表可视化 | ✅ | ❌ |
| 命令描述说明 | ✅ | ❌ |
| 参数提示 | ✅ | ❌ |
| ↑↓ 选择 | ✅ | ❌ |
| Tab / Enter 补全 | ✅ | ❌ |
| 命令分类 | ✅（Skill / Action） | ❌ |
| 子命令提示 | ✅（如 `/git` 下提示 status/diff/commit） | ❌ |

---

## 实现方案

### Phase 1：基础 Slash Command Palette（MVP）

1. **监听 `input` 事件**：检测输入框内容是否以 `/` 开头且光标前无其他字符
2. **新增 `SlashCommandPalette` 组件**：
   - 定位：输入框上方浮动面板
   - 样式：参考模型选择器 Modal 或 Kimi Code 的命令列表
   - 内容：命令名 + 描述 + 参数模板
3. **键盘导航**：
   - `↑` / `↓`：切换选中项
   - `Enter` / `Tab`：补全命令到输入框
   - `Esc`：关闭面板
4. **命令元数据结构**：
   ```javascript
   const slashCommands = [
     { name: 'tools', desc: '列出可用工具', params: [] },
     { name: 'tool', desc: '执行工具', params: ['name', 'args'] },
     { name: 'git', desc: 'Git 操作', subcommands: ['status', 'diff', 'commit'] },
     // ...
   ];
   ```

### Phase 2：增强体验

- **子命令提示**：输入 `/git ` 后提示 `status` / `diff` / `commit`
- **参数补全**：输入 `/tool ` 后提示可用工具名列表
- **命令分类**：区分"工具类"、"Git 类"、"MCP 类"、"系统类"
- **最近使用**：置顶最近使用过的命令

---

## 相关代码位置

```
前端：src/interface/web/app.js
  - setupChat()               [L1885]  聊天输入框事件绑定（缺失 / 键监听）
  - sendChatMessage()         [L1927]  发送消息（包含 slash 命令判断）
  - handleChatCommand()       [L2014]  Slash 命令执行逻辑
  - showCommandPalette()      [L3407]  全局命令面板（Ctrl+Shift+P，非输入框）
```

---

## 结论

- **当前状态**：Slash 命令功能存在但完全不可发现，只能盲打
- **阻塞性**：否，盲打可用
- **体验影响**：高，用户不知道工具集的存在
- **实现成本**：中（纯前端，约 200-400 行 JS + CSS）
- **建议排期**：在"文件引用"功能之后、"Token 统计"之前实施
- **首选方案**：Phase 1（基础面板）快速补齐，后续迭代至 Phase 2（子命令 + 参数补全）

---

## 关联问题

- **#01 Token 统计**：Slash 命令面板未来可扩展 `/usage` 命令来快速查看 Token 消耗
- **#03（待讨论）@ 文件引用**：`@` 和 `/` 的输入提示面板可复用同一套 UI 组件
