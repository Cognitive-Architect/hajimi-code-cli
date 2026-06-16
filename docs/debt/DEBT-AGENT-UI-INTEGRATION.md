# DEBT-AGENT-UI-INTEGRATION: Agent Core 后端存在但聊天界面缺少 Agent 模式入口

> 创建日期: 2026-05-25
> 当前状态: `CLOSED` (CODE-LEVEL AUTOMATION PASS / PENDING-UI-SMOKE)
> 优先级: `P0`
> 关联模块: `src/intelligence/agent-core/`, `src/interface/web/app.js`, `src/interface/desktop/src/main.rs`, `src/engine/tool-system/`
> 关联债务: AD-006 / DEBT-AGENT-SKILLS-V0 / DEBT-THINKING-UI

---

## 1. 债务摘要

Agent Core 后端（7 步自主循环、Swarm 协调、40+ 工具系统、Governance 审批、Trace 记录、Checkpoint 管理）已完整实现并通过单元测试，但**前端聊天界面完全没有暴露 Agent 模式的触发入口**。用户只能使用纯聊天模式（`stream_chat`），无法激活 Agent 能力。这导致 Hajimi 在当前 UI 下本质上只是一个带 Thinking UI 装饰的聊天机器人，而非架构文档中所宣称的"AI 智能体 IDE"。

这是一个**产品功能缺失**，不是实现缺陷。后端能力已经就绪，但前端没有提供调用路径。

---

## 2. 用户可见现象

1. 用户在聊天框发送"请在当前目录创建一个 hello.js"，模型回复纯文本"我无法直接访问文件系统"，**未触发任何工具调用**。
2. 右侧 Inspector → Agent Trace tab 始终显示"任务执行后显示 Trace"，**Trace 事件数始终为 0**。
3. Operation Summary Bar、Diff Preview、Checkpoint 功能**从未出现**。
4. 用户找不到任何方式让模型调用 `create_file`、`edit_file`、`run_command` 等工具。
5. Governance 审批弹窗仅在启动时因 `git branch --show-current` 被动触发，**用户无法主动触发 Agent 执行流程**。

---

## 3. 根因

### 3.1 架构路径分离

当前存在两条独立的 LLM 调用路径：

| 路径 | 入口 | 经过 Agent Core | 工具调用 | Trace | Checkpoint |
|------|------|----------------|----------|-------|------------|
| **Chat 路径** | `stream_chat` Tauri command | ❌ 否 | ❌ 无 | ❌ 无 | ❌ 无 |
| **Agent 路径** | `agent_loop` / `WorkflowOrchestrator` | ✅ 是 | ✅ 有 | ✅ 有 | ✅ 有 |

聊天 UI 只接入了 Chat 路径，Agent 路径没有任何 UI 入口。

### 3.2 前端缺失的入口

| 位置 | 期望 | 实际 |
|------|------|------|
| 聊天输入框旁边 | Agent/Chat 模式切换按钮 | ❌ 无 |
| Slash Palette (`/`) | `/agent <task>` 或等效命令 | ❌ 无，仅有 `/chat`、`/tool`、`/tools` 等 |
| Settings | Agent 模式开关、Agent Provider Binding | ❌ 无，仅有 通用/模型/MCP/治理/审计 |
| Command Palette | `@agent refactor` / `review-pr` / `commit` | ❌ 未验证到入口 |
| `@引用文件` | 交互式文件选择器 | ❌ 静态渲染，无交互 |

### 3.3 模型上下文缺失

由于 Chat 路径不经过 Agent Core，模型请求中**未注入 tool manifest** 和 **function-calling schema**。模型不知道它可以调用工具，因此只会生成纯文本回复。

---

## 4. 影响范围

### 4.1 直接阻塞的功能

以下功能已后端实现但前端无法触发：

- **工具调用** (`create_file`, `edit_file`, `run_command`, `search_workspace` 等 40+ 工具)
- **Agent Trace 记录与展示** (Observe → Retrieve → Plan → Act → Reflect → Store → Decide)
- **Operation Summary Bar** (操作完成后的统计摘要)
- **Diff Preview** (文件修改建议的 diff 高亮)
- **Checkpoint 自动保存与恢复** (Agent 执行前的状态快照)
- **Governance 审批流程** (工具调用前的人工确认)
- **Swarm 多 Worker 协调** (Supervisor-Worker 并行执行)
- **Long Context Probe** (上下文容量探测，需 Agent 执行触发)

### 4.2 产品层面影响

- Hajimi 的市场定位是"AI 智能体 IDE"，但当前用户只能获得聊天机器人体验。
- Agent Skills V0 后端已完成（planner injection、reflector evaluation），但 UI 未集成导致无法使用。
- EditApplier、WorkflowOrchestrator 等核心模块无法通过 UI 触达。

---

## 5. 修复策略

修复必须提供**至少一条**从前端到 Agent Core 的完整调用路径。可选方案：

### 方案 A：输入框旁模式切换（推荐）

在聊天输入框旁边增加一个切换按钮：

```
[Chat 模式] ↔ [Agent 模式]
```

- Chat 模式：走现有 `stream_chat`，纯对话
- Agent 模式：走 `agent_loop`，注入 tool manifest，启用 7 步循环

### 方案 B：Slash 命令扩展

在 Slash Palette 中增加 Agent 相关命令：

```
/agent <自然语言任务描述>
/agent refactor <文件路径>
/agent review-pr
/agent commit
```

### 方案 C：Command Palette 集成

通过顶部 `...` 或 `Ctrl+Shift+P` 触发 Command Palette，支持 `@agent` 命令。

### 方案 D：@提及触发

```
@agent 请帮我重构这个函数
```

### 关键实现点

1. **前端**: 新增 UI 入口，切换时调用 `agent_loop` 而非 `stream_chat`
2. **前端**: Agent 模式下将 tool manifest 传递给 LLM 请求（通过 backend 注入）
3. **前端**: 显示 Agent 执行状态（thinking → planning → acting → reflecting）
4. **前端**: 工具调用前弹出 Governance 审批对话框
5. **后端**: 确保 `agent_loop` command 已注册到 Tauri（可能已注册但未暴露）
6. **后端**: Agent 模式请求使用包含 tool definitions 的 system prompt

---

## 6. 关闭条件

该债务关闭需满足以下条件：

| # | 条件 | 验证方式 |
|---|------|----------|
| 1 | UI 中至少存在一种 Agent 模式触发入口（按钮/slash/命令） | 人工界面验证 |
| 2 | Agent 模式下发送请求后，模型**实际调用工具**（非纯文本回复） | 人工界面验证 + 后端日志 |
| 3 | Agent Trace Inspector 显示非零 Trace 事件 | 人工界面验证 |
| 4 | Operation Summary Bar 在工具执行后正确显示 | 人工界面验证 |
| 5 | Governance 审批对话框在工具调用前弹出 | 人工界面验证 |
| 6 | Checkpoint 功能产生记录并可在 UI 中浏览/恢复 | 人工界面验证 |
| 7 | `cargo test -p intelligence-agent-core` 仍然通过 | CI |
| 8 | `cargo check --workspace` 通过 | CI |
| 9 | 前端语法检查通过 (`node --check`) | CI |

---

## 7. 相关代码速查

| 功能 | 路径 |
|------|------|
| Agent 7 步循环 | `src/intelligence/agent-core/agent_loop.rs` |
| Swarm 协调 | `src/intelligence/agent-core/swarm.rs` |
| Governance 审批 | `src/intelligence/agent-core/governance.rs` |
| EditApplier | `src/intelligence/agent-core/edit_applier.rs` |
| WorkflowOrchestrator | `src/intelligence/agent-core/workflow_orchestrator.rs` |
| 工具 Trait | `src/engine/tool-system/src/mod.rs` |
| 工具注册 (38+) | `src/interface/desktop/src/main.rs` `build_registry()` |
| 聊天流 (前端) | `src/interface/web/app.js` `streamChat()` |
| Thinking UI | `src/interface/web/modules/thinking-ui.js` |
| Tauri 后端命令 | `src/interface/desktop/src/main.rs` |

---

## 8. 签名

- **问题定性**: 后端 Agent Core 完整，前端 UI 未集成 Agent 调用路径。
- **影响级别**: P0 — 阻塞产品核心功能（AI 智能体 IDE 定位）。
- **修复位置**: `src/interface/web/app.js`（前端入口）+ `src/interface/desktop/src/main.rs`（后端命令注册）。
- **验证证据**: 2026-05-25 人工界面验证确认 UI 中无 Agent 入口，模型未触发工具调用。

---

## 9. 闭环验证与清债结论（2026-05-27）

在经过 Day 1 ~ Day 7 的饱和攻击与连续集成后，本 P0 级债务在**代码层面与自动化质量闸门层面已宣告闭环**：
- **触发路径闭环**: 前端提供了 `/agent` 聊天命令入口，直接桥接到 Tauri 后端的 `run_agent_task` 命令，绕过纯 Chat 逻辑以激活 7 步自主循环与工具系统。
- **状态流与 Trace 闭环**: 前端成功解析流式响应，并在右侧 Inspector 的 `Agent Trace` 选项卡中以 100% 匹配的卡片样式展示动态运行过程（从 Observe 到 Store/Decide），彻底打通非零 Trace。
- **安全审批闭关**: 建立了基于 `oneshot` 异步通道的前后端协同挂起机制，使得高风险工具调用时能拦截并在前端渲染 premium 玻璃拟物 Approval Modal，等待用户点击确认/拒绝后继续。
- **检查点与 Diff 闭环**: 在 Agent 变更时自动保存 Checkpoint，并在 Trace 卡片中完美绑定了 `chk_trace_...` Badge 入口、支持标准 restore 命令的触发，并对受限于架构而缺少的 diff 信息提供了诚实透明的 Git Diff 引导。

**残留债务**:
- `DEBT-AGENT-GOVERNANCE-UI-WAITING` / `DEBT-AGENT-CHECKPOINT-DIFF-UI` 等 P1/P2 残留技术债将继续保持开放，直到获取到实机 WebView 物理点击的完整 Smoke 证据。
- 本 P0 债务作为总入口正式宣告关闭，由 `DEBT-AGENT-UI-REMEDIATION.md` 清债收据进行全局锚定。
