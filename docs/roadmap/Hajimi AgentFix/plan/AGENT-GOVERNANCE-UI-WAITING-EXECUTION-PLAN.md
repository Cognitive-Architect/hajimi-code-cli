# AGENT-GOVERNANCE-UI-WAITING 执行计划 — Day 1~3 每日细化

> **文档版本**: 1.0  
> **所属 Roadmap**: `P0-AGENT-GOVERNANCE-UI-WAITING-REMEDIATION-ROADMAP.md`  
> **关联债务**: `AD-011` / `DEBT-AGENT-GOVERNANCE-UI-WAITING`  
> **优先级**: P0  
> **状态**: PLAN / READY-FOR-IMPLEMENTATION  
> **最后更新**: 2026-06-01  

---

## 已确认的故障基线

真实 Desktop WebView smoke：

```text
/agent 请在当前工作区根目录创建一个名为 agent-smoke-check.txt 的文件，内容写入 Agent smoke OK。
```

失败结果：

```text
ToolCallInitiated: Tool 'write_file' requested. Initiating governance approval gate.
GovernanceWaiting: Tool 'write_file' waiting for Critical approval with risk score 0.95.
Approval timed out while waiting for user response for tool 'write_file' after 30s.
```

UI 现象：

```text
正常聊天可用。
/agent 正常进入 LLM-Native。
Trace 显示 write_file 正在等待 Critical approval。
没有 approval popup / modal / sidebar prompt。
30 秒后任务失败。
agent-smoke-check.txt 没有创建。
```

当前判断：

```text
不是 DeepSeek schema 问题。
不是 LLM-Native budget meltdown。
不是 write_file 风险等级误判。
不是后端 governance 完全失效。
当前 blocker 是 frontend approval_request event bridge 不可达。
```

白话说明：

> 后端已经把“是否允许写文件”的审批单放出来了，也愿意等用户签字。但前端没有收到这张单，所以用户根本没地方点同意或拒绝。

---

## 当前代码风险点

| 风险点 | 文件 | 现象 | 本计划处理方式 |
|:---|:---|:---|:---|
| Frontend event bridge 缺少 listen | `src/interface/web/modules/tauri-bridge.js` | 只暴露 `invoke` / `Channel`，approval event 无统一入口 | Day 1 增加 `HajimiTauri.listen` |
| Approval listener 可能绑定旧全局对象 | `src/interface/web/app.js` | 直接依赖 `window.__TAURI__.event.listen`，packaged app 中不可用 | Day 1/2 改为 `HajimiTauri.listen` |
| `withGlobalTauri=false` | `src/interface/desktop/tauri.conf.json` | `window.__TAURI__` 不可靠 | 不改配置；修桥接层 |
| Missing-listener silent failure | `app.js` | listener 安装失败时用户无感知 | Day 1 加 visible diagnostic |
| approve/reject contract 风险 | `app.js` / `main.rs` | request id 命名或 command 缺失会导致回传失败 | Day 2 做 contract verification |
| 缺少 frontend smoke | `tests/frontend` | 无测试保障 modal / approve / reject | Day 2 新增 smoke |

---

## 总体执行顺序

| Day | 主题 | 目标 | 主要文件 |
|:---:|:---|:---|:---|
| Day 1 | Event Bridge + Listener Wiring | 补 `HajimiTauri.listen`，让 `setupGovernance()` 能订阅 `approval_request` | `tauri-bridge.js`, `app.js` |
| Day 2 | Approval Modal Contract + Frontend Smoke | 确保 modal 出现，approve/reject 调 `resolve_agent_approval`，补 Node smoke | `app.js`, `tests/frontend/agent_governance_approval_smoke.js` |
| Day 3 | Desktop WebView Smoke + Debt Closure | 跑完整自动化 + 真实 approve/reject smoke，更新 debt 状态 | docs, desktop package |

原则：

```text
先修事件桥。
再修 UI 闭环。
最后实机验收。
不削弱安全策略。
不把写工具改成自动通过。
```

---

## Phase 1: Event Bridge Repair（Day 1）

> **目标**: 在不改变后端 governance 策略的前提下，让 packaged WebView 前端具备订阅 Tauri event 的统一桥接能力。

---

### Day 1: Event API 采样 + `HajimiTauri.listen` + `setupGovernance` 初步接线

**预计工时**: 4-6 小时  
**风险等级**: 中  
**是否允许改后端**: 原则上不允许；只采样 contract，不修改 governance 策略  

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 重采样当前 Git 坐标 | repo | `git branch --show-current`；`git rev-parse HEAD` |
| 2 | 确认 packaged config | `tauri.conf.json` | `rg -n "withGlobalTauri" src/interface/desktop/tauri.conf.json`，必须保持 `false` |
| 3 | 采样现有 bridge | `tauri-bridge.js` | 确认 `invoke` / `Channel` 存在，`listen` 缺失 |
| 4 | 真实 WebView event API 采样 | packaged app / DevTools | 记录 `window.HajimiTauri`、`window.__TAURI__`、`window.__TAURI__?.event?.listen`、`window.__TAURI_INTERNALS__`、`window.__TAURI_INTERNALS__` keys；不得只凭 API 名字猜 |
| 5 | 新增 event listen wrapper | `tauri-bridge.js` | 仅使用采样证实存在的底层 event listen；`async function listen(eventName, handler)`；导出到 `HajimiTauri` |
| 6 | listener 不可用时报错 | `tauri-bridge.js` | 抛出 `Tauri event listen unavailable`，不静默 |
| 7 | 改 `setupGovernance` listener 来源 | `app.js` | `window.HajimiTauri.listen('approval_request', handler)` |
| 8 | `setupGovernance` 幂等保护 | `app.js` | `this._governanceListenerInstalled` / `this._approvalUnlisten` |
| 9 | missing listener diagnostic | `app.js` | toast + console.warn + 可选 trace diagnostic |
| 10 | 基础 syntax 验证 | web | `node --check src/interface/web/modules/tauri-bridge.js`；`node --check src/interface/web/app.js` |

#### 关键实现约束

```text
不要设置 withGlobalTauri=true。
不要让 setupGovernance 直接依赖 window.__TAURI__。
不要吞掉 listener 安装失败。
不要在 Day 1 改 write_file 风险等级。
不要臆造 `__TAURI_INTERNALS__.listen` 或 `__TAURI_INTERNALS__.event.listen`；只有真实采样看到该 key，才允许使用。
```

#### Day 1 必需采样记录

```text
必须在 Day 1 记录以下内容，哪怕结果是“不可见 / 无法采样”：
- packaged app 路径与 SHA / 修改时间
- withGlobalTauri 当前值
- window.HajimiTauri 是否存在
- window.__TAURI__ 是否存在
- window.__TAURI__?.event?.listen 是否存在
- window.__TAURI_INTERNALS__ 是否存在
- window.__TAURI_INTERNALS__ 的 key 列表里是否有 event/listen 能力
- setupGovernance 是否被执行过

如果无法打开 DevTools 或无法采样 window keys：
- 不能继续写“猜测版 listen wrapper”
- 只能先补 visible diagnostic，随后评估 polling fallback 或 backend channel 方案
```

#### 建议实现片段

`tauri-bridge.js`:

```js
function getEventListen() {
  const tauri = global.__TAURI__;
  if (tauri?.event?.listen) return tauri.event.listen.bind(tauri.event);

  const internals = global.__TAURI_INTERNALS__;
  // Only use internals keys that were confirmed in real WebView sampling.
  if (internals?.event?.listen) return internals.event.listen.bind(internals.event);
  if (typeof internals?.listen === 'function') return internals.listen.bind(internals);

  return null;
}

async function listen(eventName, handler) {
  const rawListen = getEventListen();
  if (!rawListen) {
    throw new Error('Tauri event listen unavailable');
  }
  return rawListen(eventName, handler);
}
```

`app.js`:

```js
async setupGovernance() {
  if (this._governanceListenerInstalled) return;

  const bridge = window.HajimiTauri;
  if (!bridge?.listen) {
    this.reportApprovalUiUnavailable?.('missing HajimiTauri.listen');
    return;
  }

  try {
    this._approvalUnlisten = await bridge.listen('approval_request', (event) => {
      const payload = event?.payload ?? event;
      this.showApprovalModal(payload);
    });
    this._governanceListenerInstalled = true;
  } catch (error) {
    this.reportApprovalUiUnavailable?.(error?.message || String(error));
  }
}
```

#### 验证命令

```bash
git branch --show-current
git rev-parse HEAD
rg -n "withGlobalTauri" src/interface/desktop/tauri.conf.json
rg -n "function listen|listen\\(|HajimiTauri" src/interface/web/modules/tauri-bridge.js
rg -n "setupGovernance|approval_request|HajimiTauri.listen|Approval UI unavailable" src/interface/web/app.js
node --check src/interface/web/modules/tauri-bridge.js
node --check src/interface/web/app.js
```

#### Day 1 验收标准

- [ ] `HajimiTauri.listen` 存在并导出。
- [ ] `HajimiTauri.listen` 使用的底层 event API 有真实 WebView 采样证据，或明确记录 `BRIDGE-BLOCKED` 后停止。
- [ ] `setupGovernance()` 不再直接依赖 `window.__TAURI__.event.listen`。
- [ ] listener 安装失败会产生 visible diagnostic。
- [ ] `withGlobalTauri` 保持 `false`。
- [ ] `node --check` 通过。
- [ ] 没有改后端 governance 策略。

#### Day 1 止损条件

```text
如果当前 Tauri packaged 环境没有任何可用 event listen API，停止。
唯一动作：记录 BRIDGE-BLOCKED 证据，改为 Day 2 评估 polling fallback 或 backend channel 方案。
禁止动作：不要设置 withGlobalTauri=true 来绕过。
禁止动作：不要提交只在 Node stub 中成立、但真实 packaged WebView 无底层 API 证据的 listen wrapper。
```

---

## Phase 2: Approval UX Contract（Day 2）

> **目标**: 让 approval_request 触发可见审批 UI，并让 approve/reject 可靠回传到后端。

---

### Day 2: Modal Contract + Approve/Reject Smoke

**预计工时**: 4-6 小时  
**风险等级**: 中  
**是否允许改后端**: 仅在 `resolve_agent_approval` command 缺失或 payload contract 不匹配时最小修改  

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 确认后端 contract | `main.rs` | `rg -n "approval_request|resolve_agent_approval|pending_approvals"` |
| 2 | payload normalize | `app.js` | `requestId = payload.request_id || payload.requestId` |
| 3 | 审批 modal 渲染 | `app.js` | `.premium-approval-overlay`，显示 tool/action/risk/description |
| 4 | Approve 按钮 | `app.js` | `invokeTauri('resolve_agent_approval', { requestId, approved: true })` |
| 5 | Reject 按钮 | `app.js` | `invokeTauri('resolve_agent_approval', { requestId, approved: false })` |
| 6 | modal lifecycle | `app.js` | resolve 后关闭；invoke 失败时显示错误 |
| 7 | 新增 frontend smoke | `tests/frontend/agent_governance_approval_smoke.js` | stub `HajimiTauri.listen` + fake payload + button click |
| 8 | missing listener smoke | same | 无 `listen` 时断言 diagnostic 出现 |
| 9 | syntax + smoke | web | `node --check` + `node tests/frontend/agent_governance_approval_smoke.js` |

#### 关键测试名称

```text
setup_governance_subscribes_to_approval_request
approval_request_renders_premium_overlay
approval_approve_invokes_resolve_agent_approval_true
approval_reject_invokes_resolve_agent_approval_false
missing_approval_listener_shows_diagnostic
```

#### Node smoke 断言

```text
[PASS] listen called exactly once with "approval_request"
[PASS] fake payload creates .premium-approval-overlay
[PASS] overlay includes write_file / Critical / risk score
[PASS] Approve invokes resolve_agent_approval with approved=true
[PASS] Reject invokes resolve_agent_approval with approved=false
[PASS] missing listener diagnostic is visible
```

#### 验证命令

```bash
node --check src/interface/web/modules/tauri-bridge.js
node --check src/interface/web/app.js
node tests/frontend/agent_governance_approval_smoke.js
cargo check -p hajimi-desktop
rg -n "resolve_agent_approval|approval_request|pending_approvals" src/interface/desktop/src/main.rs
```

#### Day 2 验收标准

- [ ] `approval_request` fake event 能渲染 modal。
- [ ] Approve 调用 `resolve_agent_approval` 且 `approved=true`。
- [ ] Reject 调用 `resolve_agent_approval` 且 `approved=false`。
- [ ] request id 兼容 `request_id` / `requestId`。
- [ ] listener 缺失时不静默。
- [ ] `cargo check -p hajimi-desktop` 通过。
- [ ] 不修改高风险工具策略。

#### Day 2 止损条件

```text
如果 event 能到前端但 resolve_agent_approval command 不存在，允许最小补后端 command。
如果 command 存在但 payload 命名不匹配，优先在前端 normalize。
如果 approve 后 write_file 自动执行但未经过 resolve_agent_approval，立即停止并回滚。
```

---

## Phase 3: Real WebView Validation & Closure（Day 3）

> **目标**: 用真实 packaged WebView 验证 approve/reject 全链路，并更新债务状态。

---

### Day 3: 自动化回归 + 实机 smoke + 清债记录

**预计工时**: 3-5 小时  
**风险等级**: 中  
**是否允许改生产逻辑**: 原则上不允许，只做回归修补  

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 跑 frontend syntax | web | `node --check src/interface/web/modules/tauri-bridge.js`；`node --check src/interface/web/app.js` |
| 2 | 跑 frontend smoke | tests | `node tests/frontend/agent_governance_approval_smoke.js` |
| 3 | 跑 desktop compile | desktop | `cargo check -p hajimi-desktop` |
| 4 | 跑历史 frontend smoke | tests | `agent_result_rendering_smoke.js`、`agent_thinking_leak_smoke.js` |
| 5 | 可选打包 | desktop | `cargo tauri build` |
| 6 | Reject 实机 smoke | packaged app | prompt 写文件 -> 点 Reject -> 文件不创建 |
| 7 | Approve 实机 smoke | packaged app | prompt 写文件 -> 点 Approve -> 文件创建 |
| 8 | 文件内容验证 | workspace | `agent-smoke-check.txt` 内容必须等于 `Agent smoke OK` |
| 9 | 清理 smoke 文件 | workspace | 删除 `agent-smoke-check.txt` |
| 10 | 更新 debt | docs | `DEBT-AGENT-GOVERNANCE-UI-WAITING.md` 标记 `FIXED-CANDIDATE / NEEDS-RECHECK` |
| 11 | 提交收卷 | git | 只 stage 相关 web/test/docs 文件 |

#### 真实 WebView Smoke Prompt

```text
/agent 请在当前工作区根目录创建一个名为 agent-smoke-check.txt 的文件，内容写入 Agent smoke OK。
```

#### Reject 路径通过标准

```text
[PASS] Approval UI appears before 30s timeout.
[PASS] UI shows write_file / Critical / risk score.
[PASS] User clicks Reject.
[PASS] Agent fails safely with user rejection / permission denied.
[PASS] agent-smoke-check.txt does not exist.
[PASS] Trace shows GovernanceWaiting -> GovernanceRejected.
```

#### Approve 路径通过标准

```text
[PASS] Approval UI appears before 30s timeout.
[PASS] User clicks Approve.
[PASS] Agent continues after approval.
[PASS] agent-smoke-check.txt exists.
[PASS] File content exactly equals: Agent smoke OK
[PASS] Trace shows GovernanceWaiting -> GovernanceApproved -> ToolExecutionSuccess.
[PASS] Smoke file is removed after validation.
```

#### 验证命令

```bash
node --check src/interface/web/modules/tauri-bridge.js
node --check src/interface/web/app.js
node tests/frontend/agent_governance_approval_smoke.js
node tests/frontend/agent_result_rendering_smoke.js
node tests/frontend/agent_thinking_leak_smoke.js
cargo check -p hajimi-desktop
```

可选打包：

```bash
cd src/interface/desktop
cargo tauri build
```

#### Day 3 验收标准

- [ ] 所有自动化 frontend smoke 通过。
- [ ] `cargo check -p hajimi-desktop` 通过。
- [ ] Reject 实机 smoke 通过。
- [ ] Approve 实机 smoke 通过。
- [ ] debt 文档记录真实截图/日志/命令输出。
- [ ] 未设置 `withGlobalTauri=true`。
- [ ] 未降低 write_file 安全等级。
- [ ] 未遗留 smoke 文件。

#### Day 3 止损条件

```text
如果 Node smoke 通过但 packaged WebView 仍无 modal，停止并采样真实 window keys / bundle sha。
唯一动作：确认 dist 同步、Tauri event API 可达性、setupGovernance 是否执行。
禁止动作：不要改成 write_file 自动通过。
```

---

## Feature / Rollback Notes

本次修复不新增环境变量 feature-gate。回滚保持简单：

```text
如果 listen wrapper 导致前端初始化失败：
  回滚 tauri-bridge.js + app.js。

如果 modal 渲染成功但 approve/reject invoke 失败：
  不回滚全部，先修 payload contract。

如果 packaged app 与 dev 行为不一致：
  不设置 withGlobalTauri=true；先验证 dist 是否同步、bundle 是否加载最新 app.js。

如果 write_file 在未批准时执行：
  立即回滚并标记 SECURITY-P0。
```

---

## 文件修改总表

| 文件 | Day | 修改类型 | 说明 |
|:---|:---:|:---|:---|
| `src/interface/web/modules/tauri-bridge.js` | Day 1 | 生产逻辑 | 新增 `HajimiTauri.listen` |
| `src/interface/web/app.js` | Day 1-2 | 生产逻辑 | `setupGovernance` listener 改桥接层；modal approve/reject 闭环 |
| `tests/frontend/agent_governance_approval_smoke.js` | Day 2 | 测试 | 覆盖 listener、modal、approve、reject、diagnostic |
| `src/interface/desktop/src/main.rs` | Day 2 | 条件修改 | 仅在 `resolve_agent_approval` 缺失时最小补齐 |
| `docs/debt/DEBT-AGENT-GOVERNANCE-UI-WAITING.md` | Day 3 | 文档 | 记录修复证据与实机 smoke 结果 |

---

## 最终回归验收清单

- [ ] `node --check src/interface/web/modules/tauri-bridge.js` 通过。
- [ ] `node --check src/interface/web/app.js` 通过。
- [ ] `node tests/frontend/agent_governance_approval_smoke.js` 通过。
- [ ] `node tests/frontend/agent_result_rendering_smoke.js` 通过。
- [ ] `node tests/frontend/agent_thinking_leak_smoke.js` 通过。
- [ ] `cargo check -p hajimi-desktop` 通过。
- [ ] packaged WebView Reject 路径通过。
- [ ] packaged WebView Approve 路径通过。
- [ ] `agent-smoke-check.txt` approve 后内容正确，测试后已删除。
- [ ] `withGlobalTauri` 仍为 `false`。
- [ ] write_file 仍要求 explicit approval。
- [ ] debt 文档记录修复证据。

---

## Commit 建议

分两次提交更稳：

```text
fix(desktop): route governance approval events through HajimiTauri bridge
test(frontend): cover agent governance approval modal flow
```

如果 Day 3 文档一起提交：

```text
docs(agent): record governance approval ui smoke results
```

---

## 最终交付物路径

```text
F:\hajimi-code-cli\docs\roadmap\Hajimi AgentFix\plan\P0-AGENT-GOVERNANCE-UI-WAITING-REMEDIATION-ROADMAP.md
F:\hajimi-code-cli\docs\roadmap\Hajimi AgentFix\plan\AGENT-GOVERNANCE-UI-WAITING-EXECUTION-PLAN.md
```

---

*本执行计划与 `P0-AGENT-GOVERNANCE-UI-WAITING-REMEDIATION-ROADMAP.md` 同步维护。每完成一天，请在对应 Day 的验收清单中打勾，并把真实命令输出或 WebView smoke 结果补进 debt 文档。*
