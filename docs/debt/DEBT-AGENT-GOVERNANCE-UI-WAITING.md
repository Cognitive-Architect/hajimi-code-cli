# Technical Debt: Agent Governance UI Waiting & E2E Verification
ID: `AD-011` / `DEBT-AGENT-GOVERNANCE-UI-WAITING`
Priority: **P0**
Date: 2026-05-26
Status: **BRIDGE-BLOCKED / COMPLETED-BRIDGE-DEFENSIVE-ADAPTER-AND-DIAGNOSTICS**
Last updated: 2026-06-02

---

## 1. Context & Architectural Challenge

Under the **Hajimi IDE**'s strict four-layer architecture (`Interface -> Intelligence -> Engine -> Foundation`), the **Agent Core** operates asynchronously within tokio background tasks, while the **Tauri event loop** executes in the main OS thread. 

When high-risk tasks are executed by the Agent, the `AgentGovernance` system is invoked to enforce approval rules (Required / Critical levels). To maintain architectural purity, **lower layers (Intelligence) cannot directly call upper layers (Interface/Tauri)** to display popups or await UI responses.

To bridge this gap while preserving zero dependencies between layers:
1. We introduced a custom `UiBridgeGovernance` implementation at the `Interface` layer (`desktop/src/main.rs`).
2. We leveraged `tokio::sync::oneshot` channels to safely suspend the Agent execution thread without locking the tokio executor or blocking other workspace services.
3. We mapped request IDs (`request_id`) inside the Tauri `AppState` to active oneshot senders, allowing safe cross-thread E2E synchronization when a `resolve_agent_approval` command is invoked from the UI.
4. We sanitized all action descriptions and inputs using regex helpers before emitting `approval_request` events to the WebView.

Plain-language read: the backend can put a "please approve this write" ticket on the counter and wait for a yes/no answer. The problem now is that the user does not see the counter at all, so no answer can be returned.

---

## 2. Technical Debt & Safety Concerns

While the asynchronous E2E blocking channel works flawlessly, the following technical debts are recognized:

### A. Lifecycle Cleanup & Potential Leakage
If the frontend window crashes, reloads, or ignores the `approval_request` event, the oneshot receiver will remain suspended until the Agent execution times out or is aborted. 
- *Remediation*: Senders are stored in a mutex-locked `HashMap` inside the global `AppState`. Senders are cleaned up instantly upon resolution. However, a garbage collection sweep or timeout mechanism is still desired to handle orphaned pending channels.

### B. WebView GUI Smoke Test Blocker (AD-003 Integration)
This is no longer merely pending. Real-machine `/agent` write-file smoke on 2026-06-01 confirmed that no approval popup was visible to the user. The backend waited for a user response, timed out after 30 seconds, and rejected the write operation safely.

### C. Temporary UI Design
The current implementation utilizes a custom glassmorphism overlay dynamically injected into the DOM within `app.js`. While styled to look extremely premium (using blur backgrounds, custom gradient buttons, and risk score badges), it is still a temporary modal. It is not integrated into a dedicated security/governance panel sidebar or authorization log dashboard.

### D. Frontend Event Bridge Mismatch

The approval modal listener in `src/interface/web/app.js` currently binds directly to:

```js
const tauri = window.__TAURI__;
if (tauri && tauri.event && typeof tauri.event.listen === 'function') {
  tauri.event.listen('approval_request', ...)
}
```

But `src/interface/desktop/tauri.conf.json` has:

```json
"withGlobalTauri": false
```

Most frontend backend calls now go through `window.HajimiTauri`, implemented by `src/interface/web/modules/tauri-bridge.js`, which supports `invoke` and `Channel` but does not expose an event-listen helper. This means the approval listener is likely not installed in the packaged app, so emitted `approval_request` events are dropped from the user's perspective.

Plain-language read: most of the app now uses a new front-desk phone line, but the approval popup is still waiting beside the old phone line. If the old phone is unplugged, the backend can keep calling forever and the user never hears it.

---

## 3. Real WebView Failure Evidence - 2026-06-01

### 3.1 User prompt

```text
/agent 请在当前工作区根目录创建一个名为 agent-smoke-check.txt 的文件，内容写入 Agent smoke OK。
```

### 3.2 Observed UI behavior

- Normal chat works.
- `/agent` correctly starts the LLM-Native path.
- DeepSeek chooses the `write_file` tool.
- The right Agent Trace shows the tool is waiting for Critical approval.
- No approval popup, modal, sidebar prompt, or other visible approve/reject UI appears.
- After 30 seconds, the task fails visibly.

### 3.3 Agent Trace evidence

```text
ToolCallInitiated: Tool 'write_file' requested. Initiating governance approval gate.
GovernanceWaiting: Tool 'write_file' waiting for Critical approval with risk score 0.95.
LLM-Native driver completed with failure: Security Gate Alert: Permission Denied!
Action [execute_tool: write_file] was rejected by the Governance policy.
Reason: Approval timed out while waiting for user response for tool 'write_file' after 30s.
Risk score evaluated: 0.95
```

### 3.4 Local filesystem evidence

The requested file did not appear in the workspace:

```text
F:\hajimi-code-cli\agent-smoke-check.txt
```

That part is correct safety behavior. A high-risk write must not execute without approval.

### 3.5 Current interpretation

The backend governance logic is alive and safer than before: it blocks `write_file`, emits trace, waits, times out, and rejects. The missing piece is the actual user-facing approval UI.

This is not the previous DeepSeek schema issue and not the previous budget meltdown. It is a frontend/bridge product gap in the Required/Critical approval flow.

---

## 4. Likely Root Cause

High confidence:

1. Backend emits `approval_request` from `UiBridgeGovernance::approve`.
2. Backend stores `request_id` in `AppState.pending_approvals`.
3. Backend waits with a 30 second timeout.
4. Frontend `app.js` only listens through `window.__TAURI__.event.listen`.
5. Packaged desktop config disables the global Tauri object: `withGlobalTauri: false`.
6. The project already introduced `window.HajimiTauri` as the central frontend bridge, but that wrapper does not support events.

Therefore, the likely missing seam is:

```text
approval_request event emitted by backend
-> no active frontend listener in packaged WebView
-> no modal rendered
-> no resolve_agent_approval call
-> timeout rejection after 30 seconds
```

Plain-language read: the backend sent a permission slip, but the classroom mailbox it used is not the mailbox the frontend actually checks anymore.

---

## 5. Recommended Fix Direction

### Option A - Extend `HajimiTauri` with event listening

Add a safe event wrapper to `src/interface/web/modules/tauri-bridge.js`:

```text
HajimiTauri.listen(eventName, handler)
```

It should use whatever Tauri v2 event API is available in this app environment and return an unlisten function when possible.

Then change `setupGovernance()` in `src/interface/web/app.js` to use `window.HajimiTauri.listen('approval_request', ...)` instead of directly touching `window.__TAURI__`.

### Option B - Make missing approval listener visible

If the event listener cannot be installed, surface a visible toast or trace entry:

```text
Approval UI unavailable: cannot subscribe to approval_request events.
```

This prevents another silent product failure.

### Option C - Add frontend smoke coverage

Add a Node frontend smoke that stubs the event bridge and asserts:

1. `setupGovernance()` subscribes to `approval_request`.
2. A fake `approval_request` payload creates `.premium-approval-overlay`.
3. Clicking approve invokes `resolve_agent_approval` with `{ requestId, approved: true }`.
4. Clicking reject invokes `resolve_agent_approval` with `{ requestId, approved: false }`.
5. Missing event bridge creates an explicit diagnostic instead of silent no-op.

### Option D - Productize approvals beyond a transient modal

Longer term, add a persistent "Pending Approvals" section in the right inspector or governance settings panel. A modal is easy to miss; a visible queue is harder to lose.

Plain-language read: don't rely only on a pop-up receipt. Also put the waiting order on the counter, so even if the receipt printer fails the user can still approve or reject.

---

## 6. Proposed Test Cases

### 6.1 Frontend event bridge contract

```text
Given window.HajimiTauri.listen is available
When app.setupGovernance() runs
Then it subscribes to approval_request exactly once
```

### 6.2 Approval modal render

```text
Given an approval_request payload for write_file
When the event callback fires
Then a visible .premium-approval-overlay appears in document.body
And it shows action_type, description, risk score, approve, and reject controls
```

### 6.3 Approve callback

```text
Given the approval modal is visible
When the user clicks approve
Then frontend invokes resolve_agent_approval with approved=true and the same request_id
```

### 6.4 Reject callback

```text
Given the approval modal is visible
When the user clicks reject
Then frontend invokes resolve_agent_approval with approved=false and the same request_id
```

### 6.5 Real WebView smoke

```text
Prompt:
/agent 请在当前工作区根目录创建一个名为 agent-smoke-check.txt 的文件，内容写入 Agent smoke OK。

Expected:
1. Approval UI appears before timeout.
2. Reject path leaves no file and reports user rejection.
3. Approve path creates agent-smoke-check.txt with exact content.
4. Agent Trace shows GovernanceWaiting -> GovernanceApproved -> ToolExecutionSuccess.
5. The file is removed after the smoke test.
```

---

## 7. Stop Conditions

Stop and record a separate debt instead of weakening security if any of these happen:

1. The proposed fix requires setting `withGlobalTauri: true` globally.
2. The proposed fix makes `write_file` Auto-approved by default.
3. The approval event reaches frontend but `resolve_agent_approval` fails.
4. The approval modal works only in dev mode but not in packaged release.
5. A write proceeds without an explicit user approval.

Plain-language read: do not solve a broken doorbell by removing the front door. Writes should still need permission; we just need the permission button to actually show up.

---

## 8. Verification Plan

1. **Rust Compilation**: `cargo check -p hajimi-desktop` must compile cleanly with 0 warnings.
2. **Rust Tests**: `cargo test -p intelligence-agent-core governance` must pass 100% green.
3. **Frontend Check**: `node --check src/interface/web/app.js` must yield 0 syntax errors.
4. **Interactive Validation**:
   - Trigger a high-risk tool call (e.g., executing a command or writing a file in a restricted directory).
   - Verify the glassmorphism approval modal pops up with correct risk-score classification, action name, and redacted/sanitized text.
   - Click "Reject" -> Verify tool execution is aborted and the agent transitions to failed/aborted status safely.
   - Click "Approve" -> Verify tool execution proceeds and the agent continues its loop successfully.

---

## 9. Current Conclusion

`write_file` is correctly classified as high-risk and should continue requiring explicit user approval. The current failure is that the approval UI is not reachable in the packaged desktop WebView.

The next fix should focus on the frontend event bridge and visible approval UX, not on DeepSeek provider logic, not on budget thresholds, and not on downgrading file-write security.

---

## 10. WebView Event API 采样与 BRIDGE-BLOCKED 判定 (2026-06-02)

在自主智能体 (Autonomous Agent) 运行的 CI/CD 或 headless 环境中，由于缺乏真实的物理 GUI 窗口且无法直接使用浏览器 DevTools 检查打包后的 WebView `window` 属性，我们根据工单规则判定为 **BRIDGE-BLOCKED** 阶段，并降级为“补齐可见诊断信息 + 评估轮询降级机制”。

### 10.1 window 属性可达性评估与采样分析
- `withGlobalTauri`: 保持为 `false` (未发生改变)。
- `window.HajimiTauri`: 已创建并由 `tauri-bridge.js` 导出，提供统一 IPC 桥接能力。
- `window.__TAURI__`: 打包后的 WebView 中为 `undefined`，导致传统的 `window.__TAURI__.event.listen` 路径彻底失效。
- `window.__TAURI_INTERNALS__`: 存在并用于内部 Channel 转换。但由于缺乏 packaged WebView 下的真实事件通道采样数据，为了防止打破 Tauri Capabilities 安全屏障并遵守安全红线，**我们完全移除了任何未经采样的猜测性 internals event/listen 封装代码**。事件监听通道目前处于不可达（Blocked）状态。

### 10.2 桥接层防御性适配与可见诊断 (Defensive Adaptation & Diagnostics)
我们在事件桥接层和前端订阅逻辑中实现了安全的防御性适配与显式诊断：
1. **统一事件监听桥接接口**：在 `src/interface/web/modules/tauri-bridge.js` 中新增了 `HajimiTauri.listen(eventName, handler)` 方法。该方法仅在 `window.__TAURI__?.event?.listen` 合法存在时调用，否则直接抛出异常 `new Error('Tauri event listen unavailable')`。
2. **幂等性与安装状态保护**：在 `src/interface/web/app.js` 的 `setupGovernance()` 中引入了 `this._governanceListenerInstalled` 与 `this._governanceListenerInstalling` 双重状态锁，强力防止异步多并发调用时的重复注册。
3. **显式诊断提示**：当 `HajimiTauri.listen` 由于 API 不可达触发失败时，前端将捕获异常，并使用 `this.showErrorToast` 和 `console.warn` 直接呈递可视化的错误警告：`"Approval UI unavailable: cannot subscribe to approval_request events."`。这使得在打包好的桌面应用程序中，由于 Event Bridge 阻断导致的审批窗口无法渲染的故障不再是静默失败，避免了用户陷入 30 秒的无感等待。

目前，高危操作审批事件桥接通道仍待未来在真实环境中采样事件接口，或者采用 Polling Fallback（状态轮询）进行彻底解决。

### 10.3 轮询降级机制 (Polling Fallback) 方案设计
当 Event Bridge 被阻断时，为确保高风险写操作仍能正常被审批通过，可考虑如下 Polling Fallback 替代方案：
1. **接口支持**：后端维持一个活跃的审批请求队列 `pending_approvals`。我们可以新增一个轻量级的 `tauri::command` 如 `get_pending_approvals()` 返回所有当前等待中的 `request_id` 及其描述。
2. **前端轮询**：若 `HajimiTauri.listen` 失败并触发诊断，前端可以开启一个低频周期定时器 (每 2.5 秒) 调用 `invokeTauri('get_pending_approvals')`。
3. **安全与资源考量**：在 Agent 进入 `GovernanceWaiting` 状态时开启轮询，在状态移除或超时后关闭，确保极低的资源消耗与完美的安全边界合规。

