# Hajimi IDE — Agent UI 代码采样与适配点确认报告

> **项目**: Hajimi IDE v1 — 本地优先 AI 智能体 IDE  
> **任务编号**: AGENT-UI-DAY-01/08  
> **最后更新**: 2026-05-26  
> **执行分支**: `v3.8.0-batch-1`  
> **HEAD SHA**: `858c962ff689a50eee521e90e2ef4f8e082a50ce`  

---

## 1. 现状基线与代码证据

### 1.1 `run_agent_task` 缺失证据 (FUNC-001)
经全局搜索确认，当前后端（`main.rs`）及前端（`app.js`）不存在任何 `run_agent_task` 相关符号。
* **验证命令**: `git grep -n "run_agent_task"`
* **执行结果**: `exit code: 1 (未找到匹配行)`
* **结论**: 适配入口完全空白，Day 2 拥有纯净的实现空间。

### 1.2 后端 AgentLoop 核心接口与执行入口 (FUNC-002)
经代码采样，已确认 `AgentLoop` 拥有高可用的自主 goal 运行接口 `execute_goal`，定义在 [src/intelligence/agent-core/agent_loop.rs:L325-L336](file:///F:/hajimi-code-cli/src/intelligence/agent-core/agent_loop.rs#L325-L336)：

```rust
pub async fn execute_goal(
    &self,
    agent_id: AgentId,
    description: &str,
) -> ReplResult<LoopOutcome> {
    let (parsed_goal, priority) = Self::from_natural_language(description);
    info!(
        "Executing goal '{}' with priority {:?}",
        parsed_goal, priority
    );
    self.run(agent_id, &parsed_goal).await
}
```

* **参数解析**:
  - `agent_id`: `String` 类型，用于隔离不同智能体实例的数据和 Blackboard 上下文。
  - `description`: `&str` 类型，接收用户从前端输入的自然语言任务指令。
* **返回类型 `LoopOutcome`**:
  定义在 [src/intelligence/agent-core/agent_loop.rs:L1251-L1257](file:///F:/hajimi-code-cli/src/intelligence/agent-core/agent_loop.rs#L1251-L1257)，共有 `InProgress`, `Success`, `Aborted`, `BudgetExceeded`, `ActFailed(String)` 等 5 种状态，能够完美承载复杂的执行反馈。

### 1.3 Trace 订阅入口与广播机制 (FUNC-003)
经代码采样，已确认 `AgentLoop` 通过 `tokio::sync::broadcast` 广播实时推理步骤（`TraceEvent`），前端可通过该机制订阅高频思考流。
* **AgentLoop 广播定义** [src/intelligence/agent-core/agent_loop.rs:L840-L848](file:///F:/hajimi-code-cli/src/intelligence/agent-core/agent_loop.rs#L840-L848)：
  ```rust
  pub fn subscribe_trace(&self) -> Option<tokio::sync::broadcast::Receiver<TraceEvent>> {
      self.trace_tx.as_ref().map(|tx| tx.subscribe())
  }
  pub fn trace_tx(&self) -> Option<tokio::sync::broadcast::Sender<TraceEvent>> {
      self.trace_tx.clone()
  }
  ```
* **桌面层 (Interface) 订阅指令** [src/interface/desktop/src/main.rs:L2731-L2789](file:///F:/hajimi-code-cli/src/interface/desktop/src/main.rs#L2731-L2789)：
  后端通过 `subscribe_agent_trace` 指令包装并桥接了该广播。
  ```rust
  #[tauri::command]
  async fn subscribe_agent_trace(
      on_event: Channel<TraceEvent>,
      state: tauri::State<'_, AppState>,
      app: tauri::AppHandle,
  ) -> Result<(), String>
  ```
  该命令将通过 `Channel<TraceEvent>` 将步骤事件直接推送给前端，同时使用 `app_clone.emit("agent:trace", &event)` 进行全局事件发射。该机制非常健全且已生产化，**完全可直接复用**。

---

## 2. 前端接入适配点设计

### 2.1 Slash Command 入口最佳插入点 (FUNC-004)
经前端模块化结构采样，确认 `/agent` 命令的最佳插入位置如下：

1. **命令注册注册点** [src/interface/web/app.js:L2351-L2363](file:///F:/hajimi-code-cli/src/interface/web/app.js#L2351-L2363) 的 `getSlashCommands()` 返回数组中加入 `/agent` 项：
   ```javascript
   {
     id: 'agent',
     trigger: '/agent',
     title: '运行 Agent',
     description: '启动 Hajimi Proactive 智能体循环执行任务',
     category: 'agent',
     riskLevel: 'high',
     enabled: true,
     executeMode: 'fill',
     insertText: '/agent '
   }
   ```
2. **命令调度分发点** [src/interface/web/app.js:L2494](file:///F:/hajimi-code-cli/src/interface/web/app.js#L2494) 的 `handleChatCommand(text)` 方法中新增调度：
   ```javascript
   if (text.startsWith('/agent ')) {
     const goal = text.slice(7).trim();
     if (!goal) {
       this.addChatMessage('ai', '用法: `/agent <任务目标>`');
       return;
     }
     await this.runAgentTask(goal);
     return;
   }
   ```

### 2.2 Tauri Channel Envelope 解包复用评估 (CONST-003)
在 [src/interface/web/modules/tauri-bridge.js:L33-L62](file:///F:/hajimi-code-cli/src/interface/web/modules/tauri-bridge.js#L33-L62) 中，`Channel` 类通过以下代码对接收到的事件消息进行了安全解包：
```javascript
this.id = internals.transformCallback((message) => {
  const payload = message
    && typeof message === 'object'
    && Object.prototype.hasOwnProperty.call(message, 'message')
    ? message.message
    : message;
  this._handler(payload);
});
```
* **解包能力评测**:
  - 该逻辑自动提取 `message.message` 属性（如果存在），否则原样返回 `message` 本身。
  - 由于 Rust 端 `Channel<TraceEvent>` 的事件在序列化后可能被 Tauri 封装在 `{ message: TraceEvent }` 中，也可能直接作为 `TraceEvent` 输出，此解包机制具有极佳的**向下兼容性与鲁棒性**。
  - **结论**: **可 100% 直接复用**，前端开发时直接订阅 `channel.onmessage = (event) => { ... }` 即可获取未经信封包装的纯净 `TraceEvent`。

---

## 3. 分层依赖合规与暂不触碰区域 (CONST-001 / CONST-002)

为绝对遵守 Hajimi 的 **四层分层架构**（下层零依赖上层）和 **零回归红线**，已确立以下工程边界：

```
+-------------------------------------------------------------+
| 界面层 (Interface) : src/interface/web/ & src/interface/desktop/ |
+-------------------------------------------------------------+
                              | (仅单向调用，依赖下层)
                              v
+-------------------------------------------------------------+
| 智能层 (Intelligence): src/intelligence/agent-core/ (AgentLoop) |
+-------------------------------------------------------------+
```

### 3.1 核心资产清单与变动规约
* **暂不触碰区域 (零改动红线)**:
  - `src/intelligence/agent-core/agent_loop.rs` 的核心算法逻辑、`observe`、`plan`、`act` 等阶段核心循环实现。
  - `src/interface/desktop/src/main.rs` 的普通 `stream_chat` 路径，保证非 Agent 普通 AI 对话体验 0 回归。
* **可直接复用函数/结构**:
  - `AgentLoop::execute_goal`
  - `AgentLoop::subscribe_trace` 和 `AppState::trace_tx`
  - `HajimiTauri::Channel` 和 `subscribe_agent_trace`
* **需要新增的符号**:
  - `src/interface/desktop/src/main.rs`: 新增 `#[tauri::command] async fn run_agent_task(...)`

---

## 4. Day 2 后端签名与接口草案 (CONST-004)

为实现高内聚、易维护 of Agent 执行通路，已为 Day 2 设计如下接口定义草案：

### 4.1 Tauri Command 接口草案 (`run_agent_task`)
```rust
/// 启动并运行 Proactive Agent 核心任务循环
#[tauri::command]
async fn run_agent_task(
    agent_id: String,
    goal: String,
    provider_id: Option<String>,
    state: tauri::State<'_, AppState>,
    agent_loop: tauri::State<'_, std::sync::Arc<agent_core::agent_loop::AgentLoop>>,
) -> Result<String, String> {
    // 1. 获取当前激活的 Profile 配置
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());
    
    // 2. 加载对应的 Provider 规格信息
    let config = if provider == "ollama" || provider == "anthropic" || provider == "openai" {
        None
    } else {
        let configs = read_merged_configs(None, profile.as_deref());
        configs.into_iter().find(|c| c.id == provider)
    };

    // 3. 将 Context Budget 规格信息同步写入 Blackboard
    write_provider_caps_to_blackboard(
        agent_loop.blackboard(),
        &agent_id,
        &provider,
        config.as_ref(),
    )
    .await;

    // 4. 触发真实的 AgentLoop 执行通路
    match agent_loop.execute_goal(agent_id.clone(), &goal).await {
        Ok(outcome) => {
            Ok(format!("Agent {} 任务执行完毕。最终结果: {:?}", agent_id, outcome))
        }
        Err(e) => {
            Err(format!("Agent 执行失败: {}", e))
        }
    }
}
```

* **设计亮点与自解耦建议**:
  - 该指令与 `create_agent_with_provider` 结构高度对齐，复用了 `write_provider_caps_to_blackboard` 工具函数，最大化减少冗余。
  - 由于 `execute_goal` 是长耗时异步操作，后续开发中建议在 Tauri 中采用 `tokio::spawn` 托管运行以防前端连接阻塞，并在 UI 端通过 trace 订阅实时响应。

---

## 5. 风险防范与止损方案 (HIGH-001 / NEG-001)

### 5.1 最大风险评估与技术熔断
* **最大风险**: 
  - `AgentLoop::execute_goal` 属于独占性、长时间运行的任务，如果在 Tauri 线程池中同步等待 (`.await`)，极易导致前端请求假死（UI 卡顿、其他 Tauri Command 积压）。
* **止损与设计熔断设计 (ARCH-001)**:
  - 若在 Day 2 联调中出现死锁或 UI 挂起，后端应立即重构为**“触发即忘 (Fire-and-Forget)”** 模式：
    `run_agent_task` 仅负责在 `tokio::spawn` 中后台拉起 `agent_loop.execute_goal`，并立即向前端返回 `TaskStarted(agent_id)` 确认包；前端通过 `subscribe_agent_trace` 独立追踪执行进度与终态。
  - 这种前后端解耦的设计，将保证 Hajimi 主界面的完全平滑与高性能。

---

## 6. CLI 验证证据与可复现指令 (REAL)

为确保数据的真实性与结论 of 诚实性，以下是验证本采样报告的所有原始指令：

### 6.1 获取当前开发坐标环境
```bash
# 确认当前工作分支
git branch --show-current
# 输出: v3.8.0-batch-1

# 确认当前 HEAD 节点
git rev-parse HEAD
# 输出: 858c962ff689a50eee521e90e2ef4f8e082a50ce
```

### 6.2 确认后端 Agent 核心执行点
```bash
git grep -n "pub async fn execute_goal"
# 输出: src/intelligence/agent-core/agent_loop.rs:325:    pub async fn execute_goal(
```

### 6.3 确认 Trace 广播源
```bash
git grep -n "pub fn subscribe_trace"
# 输出: src/intelligence/agent-core/agent_loop.rs:840:    pub fn subscribe_trace(&self) -> Option<tokio::sync::broadcast::Receiver<TraceEvent>> {
```
