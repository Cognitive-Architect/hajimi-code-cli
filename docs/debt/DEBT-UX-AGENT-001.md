# DEBT-UX-AGENT-001: 启动闪屏 / 历史对话缺失 / 文件树加载错误

> **创建日期**: 2026-05-10  
> **状态**: 修复中（Rust 后端 + JS 前端已修改，待构建验证）  
> **严重级别**: P1（影响日常可用性）  
> **关联提交**: `main.rs` 修改（`get_current_workspace` 签名变更）、`app.js` 修改（会话持久化）  

---

## 问题概述

用户在运行 `target/release/hajimi-desktop.exe` 时反馈三个明显的可用性问题：

1. **启动闪屏** — 打开应用时突然弹出一个提示窗口，4 秒后自动消失
2. **历史对话缺失** — 点击"新会话"后，之前的对话完全丢失，无法找回
3. **文件树加载错误** — 启动即提示"加载文件树失败"

这三个问题并非独立，它们共享同一个根因：**工作区路径与沙箱安全策略不匹配**。

---

## 根因分析

### 问题 3：文件树加载错误（根本原因）

**文件**: `src/interface/desktop/src/main.rs`

```rust
#[tauri::command]
fn get_current_workspace() -> Option<String> {
    std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string())
}
```

当用户双击 `target/release/hajimi-desktop.exe` 运行时，`std::env::current_dir()` 返回的是 `.exe` 所在目录：

```
F:\hajimi-code-cli\target\release\
```

而 `list_dir` 命令内部调用了 `validate_path_within_workspace`，该函数强制要求路径必须在沙箱根目录内：

```rust
fn get_workspace_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app_handle.path().document_dir()?;
    let workspace = base.join("hajimi-workspace");
    // ...
}
```

沙箱路径通常是：

```
C:\Users\<用户名>\Documents\hajimi-workspace\
```

因此 `list_dir("F:\hajimi-code-cli\target\release\")` 触发沙箱越界检查，返回错误：

```
路径越界: F:\hajimi-code-cli\target\release 不在工作目录 C:\Users\...\Documents\hajimi-workspace 内
```

### 问题 1：启动闪屏（问题 3 的衍生症状）

**文件**: `src/interface/web/app.js`

```js
async loadFileTree(path) {
    // ...
    try {
      const entries = await invoke('list_dir', { path: rootPath });
      // ...
    } catch (e) {
      console.error('loadFileTree error:', e);
      this.showErrorToast('加载文件树失败: ' + (e.message || e));  // ← 闪屏来源
    }
}
```

`showErrorToast` 的实现：

```js
showErrorToast(message) {
    // 创建/显示 toast DOM
    toast.classList.add('active');
    setTimeout(() => { toast.classList.remove('active'); }, 4000);  // ← 4 秒后消失
}
```

用户看到的"突然弹出一个窗口又不见"，就是启动时 `loadFileTree` 失败后弹出的错误提示 toast，4000ms 后自动隐藏。

### 问题 2：历史对话缺失（独立功能缺失）

**文件**: `src/interface/web/app.js`

```js
newChatSession() {
    this.chatMessages = [];           // ← 直接清空，无备份
    document.getElementById('aiChatMessages').innerHTML = '';
    this.addChatMessage('ai', '新会话已开始。有什么可以帮您的？');
    // 没有任何持久化逻辑
},
```

`sessionList` 面板中只有一个硬编码的"当前会话"项，没有：
- 会话列表数据结构
- `localStorage` 持久化
- 会话切换逻辑
- 会话标题/预览生成

---

## 影响范围

| 影响项 | 严重程度 | 说明 |
|:---|:---:|:---|
| 首次启动体验 | 🔴 高 | 每次打开都看到错误提示，给用户"软件坏了"的印象 |
| 文件树功能 | 🔴 高 | 完全不可用，无法浏览/操作文件 |
| 对话连续性 | 🟡 中 | 无法保留历史上下文，多轮对话体验断裂 |
| 用户信任度 | 🟡 中 | 频繁报错降低对产品质量的信心 |

---

## 修复方案

### 修复 1：工作区路径对齐（解决问题 3 和问题 1）

**文件**: `src/interface/desktop/src/main.rs`

将 `get_current_workspace` 改为返回沙箱目录路径，而非 `current_dir`：

```rust
#[tauri::command]
fn get_current_workspace(app_handle: tauri::AppHandle) -> Option<String> {
    get_workspace_dir(&app_handle).ok().map(|p| p.to_string_lossy().to_string())
}
```

**影响**: 修复后 `list_dir` 接收的路径在沙箱内，校验通过，文件树正常加载，错误 toast 不再触发，闪屏消失。

**状态**: ✅ 已修改

### 修复 2：会话持久化（解决问题 2）

**文件**: `src/interface/web/app.js`

新增以下功能和数据结构：

```js
// 新增状态字段
chatSessions: [],      // 会话列表
activeSessionId: null, // 当前活动会话 ID

// 核心方法
newChatSession()       // 保存当前会话 → 创建新会话 → 更新列表
saveChatSessions()     // 序列化到 localStorage
loadChatSessions()     // 从 localStorage 恢复
switchSession(id)      // 切换会话
renderSessionList()    // 渲染会话列表面板
renderChatMessages()   // 渲染消息历史
```

持久化键名：`hajimi_chat_sessions`

**状态**: ✅ 已修改

---

## 验证清单

- [ ] 启动应用，无错误 toast 弹出
- [ ] 文件树面板正常显示 `hajimi-workspace` 目录内容
- [ ] 发送消息后可点击"新会话"创建新对话
- [ ] 左侧会话列表显示多个历史会话
- [ ] 点击历史会话可切换回之前的对话内容
- [ ] 关闭应用重新打开，历史会话仍然保留

---

## 后续建议

1. **沙箱初始化提示**：首次启动时如果 `hajimi-workspace` 为空，可以显示一个友好的欢迎/引导界面，而非空文件树。
2. **会话搜索**：当历史会话增多时，添加搜索/过滤功能。
3. **会话导出**：允许用户将会话历史导出为 `.md` 或 `.json` 文件。
4. **清理策略**：`localStorage` 容量有限（通常 5-10MB），建议添加会话数量上限（如保留最近 50 个）或按时间清理旧会话。

---

*文档与代码同步维护。如修复状态变更，请更新本文件。*
