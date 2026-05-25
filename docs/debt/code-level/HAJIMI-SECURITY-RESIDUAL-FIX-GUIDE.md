# HAJIMI Security Residual Fix Guidance Report

> **报告类型**: 修复指导 / 派单前置设计  
> **目标仓库**: `Cognitive-Architect/hajimi-code-cli`  
> **验证基线**: latest default branch `v3.8.0-batch-1` as inspected on 2026-05-19  
> **适用范围**: 上一轮 Codex Security 验证后仍残留的问题  
> **建议批次名**: `B-18 Security Hardening Closure`  
> **核心结论**: 大部分漏洞已修复；剩余风险集中在 **Tauri global API 暴露**、**确认 token 仍由前端自行领取**、**legacy run_command 未完全退场** 三个点。

---

## 0. 一句话结论

当前安全状态建议标记为：

```text
SECURITY STATUS: MOSTLY REMEDIATED / NOT FULLY CLEARED
BLOCKING RESIDUALS:
1. withGlobalTauri=true 仍然放大 XSS 后果
2. create_tool_confirmation_token 可由前端 JS 直接调用，XSS 模型下不能视为强确认
3. run_command 仍是裸 Tauri command，虽然白名单已收窄，但未并入 ToolPermissions/Governance
```

人话版：  
锁已经换了，门也加固了，但钥匙柜还放客厅。正常人不会乱拿，但如果前端被塞进恶意脚本，它还是能伸手摸钥匙。

---

## 1. 剩余问题总览

| 编号 | 问题 | 当前状态 | 风险等级 | 建议优先级 |
|---|---|---:|---:|---:|
| R-001 | `withGlobalTauri: true` 未关闭 | 未修 | High | P0 |
| R-002 | 工具确认 token 由前端普通 JS 可直接 mint | 部分修 | High | P0 |
| R-003 | legacy `run_command` 未并入 ToolPermissions | 主体缓解 | Medium | P1 |
| R-004 | Security Gate 还需补“反回退”测试 | 部分覆盖 | Medium | P1 |
| R-005 | 没有 CI / receipt 证据闭环 | 未验证 | Medium | P1 |

---

## 2. 修复总原则

### 2.1 不要把 UI `confirm()` 当安全边界

浏览器里的 `confirm()` 只能防普通用户误触，不能防恶意 JS。  
如果前端出现 XSS，恶意脚本可以绕过 UI confirm，直接调用 Tauri command。

**硬原则**:

```text
高风险工具确认必须在后端完成，最好是 Rust/Tauri 原生确认，而不是纯前端 JS confirm。
```

### 2.2 不要一次性大改架构

优先小步修：

```text
Phase 1: 先 block 最高风险路径
Phase 2: 再迁移前端调用方式
Phase 3: 最后关旧入口和补 CI gate
```

### 2.3 每个修复必须有 receipt

每个修复 PR 必须带：

```text
- 代码位置
- 风险前后对比
- 验证命令
- 失败时回滚方式
```

---

# 3. R-001 修复指导：关闭 / 隔离 `withGlobalTauri`

## 3.1 问题背景

当前 `src/interface/desktop/tauri.conf.json` 仍保留：

```json
"withGlobalTauri": true
```

这意味着前端可以通过 `window.__TAURI__` 直接访问 Tauri API。  
正常情况下方便开发；但一旦存在 XSS，攻击脚本也能访问同一套能力。

## 3.2 目标状态

最终目标：

```json
"withGlobalTauri": false
```

同时前端不再全局散落：

```js
window.__TAURI__
tauri.core.invoke
tauri.invoke
```

而是统一通过一个受控桥接层调用：

```js
window.HajimiTauri.invoke(...)
```

## 3.3 推荐落地方案

### Step 1：新增前端 Tauri Adapter

新增文件：

```text
src/interface/web/modules/tauri-bridge.js
```

建议职责：

```js
(function (global) {
  'use strict';

  function getTauriGlobal() {
    return global.__TAURI__ || null;
  }

  function getInvoke() {
    const tauri = getTauriGlobal();
    return tauri ? (tauri.core?.invoke || tauri.invoke || null) : null;
  }

  async function invoke(command, args = {}) {
    const rawInvoke = getInvoke();
    if (!rawInvoke) {
      throw new Error('Tauri invoke unavailable');
    }
    return rawInvoke(command, args);
  }

  function isAvailable() {
    return Boolean(getInvoke());
  }

  global.HajimiTauri = {
    invoke,
    isAvailable,
  };
})(window);
```

⚠️ 这是过渡层。它短期仍可用 `window.__TAURI__`，但可以先把 app.js 里的直接调用集中到一个位置。  
等后续引入正式 Tauri JS API / bundler / preload 方案时，只改这个 adapter。

### Step 2：替换前端散落调用

把：

```js
const tauri = window.__TAURI__;
const invoke = tauri.core?.invoke || tauri.invoke;
await invoke('xxx', args);
```

逐步替换为：

```js
await window.HajimiTauri.invoke('xxx', args);
```

优先替换高风险路径：

```text
/tool
run_command
provider config
checkpoint restore
apply edits
```

### Step 3：release gate 先加 fail，不急着立即全量关闭

在 `tests/security/security_audit_gate.js` 加：

```js
function scanTauriGlobalReleaseMode() {
  const raw = readText(tauriConfigPath);
  const config = JSON.parse(raw);
  if (config.app?.withGlobalTauri === true) {
    addFailure(
      'tauri-global-api-release-blocker',
      tauriConfigPath,
      findLine(raw, 'withGlobalTauri'),
      'withGlobalTauri=true is not allowed for security closure; migrate to HajimiTauri adapter'
    );
  }
}
```

如果你担心一下关掉会炸 UI，可以先做两段式：

```text
Day 1: Gate warning -> fail only in release profile
Day 2: app.js 全部走 adapter
Day 3: withGlobalTauri=false
```

## 3.4 验收标准

```bash
rg -n "window\.__TAURI__|tauri\.core|tauri\.invoke" src/interface/web
node tests/security/security_audit_gate.js
cargo check --workspace
```

验收要求：

```text
[ ] app.js 不再直接出现 window.__TAURI__
[ ] tauri.conf.json withGlobalTauri=false
[ ] Security Gate 对 withGlobalTauri=true 会 fail
[ ] WebView smoke 通过：聊天、provider、文件树、checkpoint 基本入口可用
```

## 3.5 回滚策略

如果关闭 `withGlobalTauri` 后 UI 大面积不可用：

```text
1. 不回滚安全目标
2. 暂时恢复 withGlobalTauri=true
3. 保留 tauri-bridge.js 和所有调用集中化改造
4. 将 Gate 状态标记为 BLOCKED-BY-TAURI-API-MIGRATION
```

---

# 4. R-002 修复指导：确认 token 改成后端强确认

## 4.1 问题背景

最新代码已经有 `enforce_tool_permissions`，这是好事。  
但 `create_tool_confirmation_token` 也是 Tauri command，并且前端可以直接调用它。

当前模型大概是：

```text
前端 confirm → create_tool_confirmation_token → execute_tool(token)
```

这能防误触，但不能防 XSS，因为恶意 JS 可以：

```text
create_tool_confirmation_token(...)
execute_tool(..., token)
```

## 4.2 目标状态

确认流程必须变成：

```text
execute_tool 请求高危工具
↓
后端发现需要确认
↓
后端创建 challenge
↓
后端触发原生确认 / 强确认
↓
用户同意
↓
后端执行工具
```

重点：**token 不应该作为普通前端可自由领取的钥匙。**

## 4.3 推荐设计：Challenge + Native Confirm + Execute

### 新增结构

建议在 `main.rs` 中新增：

```rust
#[derive(Clone)]
struct PendingToolConfirmation {
    tool_name: String,
    args_hash: String,
    created_at_ms: i64,
    expires_at_ms: i64,
    risk_level: String,
    summary: String,
}
```

AppState 改为：

```rust
tool_confirmations: std::sync::Mutex<HashMap<String, PendingToolConfirmation>>,
```

### API 改造

替代现在的 `create_tool_confirmation_token`：

```rust
#[tauri::command]
async fn request_tool_execution(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    name: String,
    args: Value,
) -> Result<ToolResult, String> {
    let tool = state.registry.get(&name)
        .ok_or_else(|| format!("tool '{}' not found", name))?;

    let permissions = tool.permissions();

    match permissions.default_level {
        PermissionLevel::Deny => return Err(format!("tool '{}' is denied by policy", name)),
        PermissionLevel::Allow if !permissions.requires_confirmation => {
            let output = tool.execute(args).await.map_err(|e| e.message)?;
            return Ok(output.into());
        }
        _ => {
            // 后端触发确认；不要把 token 直接交给任意 JS
            let approved = confirm_tool_native(&app_handle, &name, &args).await?;
            if !approved {
                return Err(format!("tool '{}' denied by user", name));
            }
            let output = tool.execute(args).await.map_err(|e| e.message)?;
            Ok(output.into())
        }
    }
}
```

### Native Confirm 方向

可选方案：

```text
方案 A：Tauri 原生 dialog plugin
方案 B：后端生成 challenge，前端只能显示；最终确认必须调用带窗口焦点/nonce 的 confirm_and_execute
方案 C：高危工具只允许通过 Agent governance 审批流，不允许 /tool 手工直调
```

最稳的是 A。  
人话：别让网页自己盖章，要让系统弹窗盖章。

## 4.4 如果短期不能接 native dialog

先做次优防护：

```text
- token TTL <= 30s
- token single-use
- token 绑定 tool name + args hash + active window label
- token 只能由 request_tool_confirmation 生成 challenge 后确认
- create_tool_confirmation_token 不再公开给普通 /tool 流
```

并在报告中诚实标记：

```text
STATUS: PARTIAL / UI-confirmation only, not XSS-hard
```

## 4.5 验收标准

```bash
cargo test -p interface-desktop -- execute_tool_requires_confirmation
cargo test -p interface-desktop -- confirmation_token_expires
cargo test -p interface-desktop -- confirmation_token_single_use
cargo test -p interface-desktop -- confirmation_token_mismatched_args_rejected
cargo test -p interface-desktop -- xss_like_create_token_then_execute_not_supported
```

必须覆盖：

```text
[ ] 高危工具无确认无法执行
[ ] 旧 token 不能复用
[ ] token 过期不能用
[ ] token 和 args 不匹配不能用
[ ] 前端不能单独 mint 可执行 token
```

---

# 5. R-003 修复指导：legacy `run_command` 退场或接入权限系统

## 5.1 当前状态

当前 allow-list 已收窄，移除了：

```text
npx / pnpm / pip / pip3 / code / cursor
```

这是明显进步。  
但 `run_command` 仍然是 Tauri command，未并入 `ToolPermissions` / confirmation / governance。

## 5.2 目标状态

最终目标：

```text
run_command 不再作为裸 Tauri command 暴露
```

推荐改为：

```text
/tool bash or /tool powershell
↓
ToolRegistry
↓
ToolPermissions
↓
confirmation/governance
↓
ShellTool allow-list
```

## 5.3 最小修复方案

### 方案 A：直接从 invoke_handler 移除

在 `tauri::generate_handler!` 中删除：

```rust
run_command,
```

如果前端没有强依赖，这是最干净的。

### 方案 B：保留但强制转发 ToolRegistry

```rust
#[tauri::command]
async fn run_command(
    cmd: &str,
    args: Vec<String>,
    state: tauri::State<'_, AppState>,
    confirmation_token: Option<String>,
) -> Result<String, String> {
    let shell_command = std::iter::once(cmd.to_string())
        .chain(args.into_iter())
        .collect::<Vec<_>>()
        .join(" ");

    let tool_name = if cfg!(target_os = "windows") { "powershell" } else { "bash" };
    let tool_args = serde_json::json!({ "command": shell_command });

    let tool = state.registry.get(tool_name)
        .ok_or_else(|| format!("tool '{}' not found", tool_name))?;

    enforce_tool_permissions(
        &tool.permissions(),
        tool_name,
        &tool_args,
        confirmation_token.as_deref(),
        &state.tool_confirmation_tokens,
    )?;

    let output = tool.execute(tool_args).await.map_err(|e| e.message)?;
    if output.exit_code.unwrap_or(1) != 0 {
        return Err(output.stderr);
    }
    Ok(output.stdout)
}
```

不过这个方案仍有 shell 拼接问题，需要小心元字符。更建议方案 A。

## 5.4 验收标准

```bash
rg -n "run_command" src/interface/desktop/src/main.rs src/interface/web tests
npm run test:security-gate
cargo test -p interface-desktop -- run_command_removed_or_requires_confirmation
```

要求：

```text
[ ] run_command 不再裸暴露，或必须经过 confirmation
[ ] Security Gate 检测 run_command 是否仍在 invoke_handler
[ ] npm/node/python 这类命令不能无确认执行
```

---

# 6. R-004 修复指导：Security Gate 补反回退规则

## 6.1 当前 Gate 应该覆盖的新增规则

在 `tests/security/security_audit_gate.js` 增加以下规则：

| 规则 | 目的 |
|---|---|
| `tauri-global-api-fail` | `withGlobalTauri=true` fail |
| `execute-tool-permission-gate` | `execute_tool` 必须调用 `enforce_tool_permissions` |
| `confirmation-token-not-public-mint` | 禁止公开裸 `create_tool_confirmation_token` |
| `desktop-file-tools-workspace-bound` | desktop registry 必须使用 `with_allowed_paths` |
| `inline-edit-resolver` | `apply_edits` / `preview_edit` 必须走 `resolve_workspace_path` |
| `run-command-not-naked` | `run_command` 不得裸出现在 invoke_handler，或必须有权限门禁 |

## 6.2 建议脚本检查点

```js
function scanExecuteToolPermissionGate() {
  const raw = readText('src/interface/desktop/src/main.rs');
  const executeToolBlock = extractFunction(raw, 'async fn execute_tool');
  if (!executeToolBlock.includes('enforce_tool_permissions')) {
    addFailure('execute-tool-permission-gate', 'src/interface/desktop/src/main.rs', 1,
      'execute_tool must call enforce_tool_permissions before tool.execute');
  }
}
```

```js
function scanRunCommandExposure() {
  const raw = readText('src/interface/desktop/src/main.rs');
  if (raw.includes('run_command,') && !raw.includes('run_command_removed_or_requires_confirmation')) {
    addFailure('run-command-naked-tauri', 'src/interface/desktop/src/main.rs', findLine(raw, 'run_command,'),
      'run_command must be removed from invoke_handler or routed through permission confirmation');
  }
}
```

```js
function scanTauriGlobalApi() {
  const raw = readText('src/interface/desktop/tauri.conf.json');
  const config = JSON.parse(raw);
  if (config.app?.withGlobalTauri === true) {
    addFailure('tauri-global-api', 'src/interface/desktop/tauri.conf.json', findLine(raw, 'withGlobalTauri'),
      'withGlobalTauri=true is forbidden after security closure');
  }
}
```

## 6.3 验收标准

```bash
node --check tests/security/security_audit_gate.js
npm run test:security-gate
```

要求：

```text
[ ] 故意改回 withGlobalTauri=true 时 gate fail
[ ] 删除 enforce_tool_permissions 时 gate fail
[ ] 文件工具改回 new() 时 gate fail
[ ] run_command 裸暴露时 gate fail
```

---

# 7. R-005 修复指导：CI / Receipt 闭环

## 7.1 新增修复 receipt

建议新增：

```text
docs/debt/DEBT-B18-SECURITY-HARDENING-CLOSURE.md
```

内容必须包含：

```markdown
# DEBT-B18 Security Hardening Closure

## Fixed Findings
- R-001 ...
- R-002 ...
- R-003 ...

## Verification Commands
```bash
npm run test:security-gate
cargo check --workspace
cargo test -p engine-tool-system
cargo test -p interface-desktop
```

## Residual Risk
...

## Rollback
...
```

## 7.2 CI 建议

如果项目有 GitHub Actions，新增或扩展：

```yaml
name: security-gate

on:
  pull_request:
  push:

jobs:
  security-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm ci || npm install
      - run: npm run test:security-gate
```

Rust CI：

```yaml
  rust-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --workspace
      - run: cargo test -p engine-tool-system
```

---

# 8. 推荐执行顺序

## Day 1：Security Gate 先加硬

目标：防止回退。

```bash
node --check tests/security/security_audit_gate.js
npm run test:security-gate
```

交付：

```text
tests/security/security_audit_gate.js
docs/debt/DEBT-B18-SECURITY-HARDENING-CLOSURE.md
```

## Day 2：Tauri Global API 迁移准备

目标：前端调用集中到 adapter。

交付：

```text
src/interface/web/modules/tauri-bridge.js
src/interface/web/index.html
src/interface/web/app.js
```

验收：

```bash
rg -n "window\.__TAURI__|tauri\.core|tauri\.invoke" src/interface/web
```

## Day 3：关闭 withGlobalTauri

目标：`withGlobalTauri=false`。

交付：

```text
src/interface/desktop/tauri.conf.json
tests/security/security_audit_gate.js
```

验收：

```bash
npm run test:security-gate
cargo tauri dev
```

## Day 4：确认 token 强化

目标：不再暴露“前端自由 mint token”。

交付：

```text
src/interface/desktop/src/main.rs
src/interface/web/app.js
```

验收：

```bash
cargo test -p interface-desktop -- confirmation
```

## Day 5：run_command 退场

目标：移除或强确认。

交付：

```text
src/interface/desktop/src/main.rs
tests/security/security_audit_gate.js
```

验收：

```bash
rg -n "run_command" src/interface/desktop/src/main.rs src/interface/web tests
```

---

# 9. 最终验收矩阵

| Gate | 命令 | 目标 |
|---|---|---|
| JS Gate | `npm run test:security-gate` | 安全回退 fail |
| Rust Build | `cargo check --workspace` | 编译通过 |
| Tool Tests | `cargo test -p engine-tool-system` | 文件工具/shell 工具通过 |
| Desktop Tests | `cargo test -p interface-desktop` | Tauri command 安全测试通过 |
| WebView Smoke | `cargo tauri dev` | 真实界面可用 |
| Search Gate | `rg -n "window\.__TAURI__" src/interface/web` | 不再散落 global Tauri |
| Config Gate | `rg -n "withGlobalTauri.*true" src/interface/desktop/tauri.conf.json` | release 不允许 true |

---

# 10. 风险与止损

## 10.1 最大风险：关闭 `withGlobalTauri` 导致前端调用全断

止损方式：

```text
只回滚 tauri.conf.json，不回滚 tauri-bridge.js adapter 改造。
```

## 10.2 最大风险：后端强确认影响开发速度

止损方式：

```text
本地 dev profile 可允许 advisory mode；
release profile 必须 strict mode。
```

## 10.3 最大风险：run_command 退场影响已有快捷入口

止损方式：

```text
保留 wrapper，但 wrapper 必须走 ToolRegistry + ToolPermissions，不再裸 Command::new。
```

---

# 11. 派单口令

```text
启动 B-18 Security Hardening Closure。

目标：
关闭 Tauri global API 高风险暴露；
把工具确认从前端自律确认升级为后端强确认；
移除或封装 legacy run_command；
补齐 Security Gate 反回退规则；
用真实命令输出完成 receipt。

红线：
不能把 UI confirm 当安全边界；
不能让前端自由 mint 高危工具 token；
不能让 run_command 裸暴露；
不能虚报 cargo/npm 验证结果；
不能关闭 withGlobalTauri 后不做 WebView smoke。

收卷：
提交 DEBT-B18-SECURITY-HARDENING-CLOSURE.md；
附 npm/cargo/tauri 验证摘要；
标记 Security Status 从 MOSTLY REMEDIATED → CLEARED 或 BLOCKED-BY-TAURI-API-MIGRATION。
```

---

## 12. 最终建议

先不要继续堆 Security Workflow V2/V3。  
现在最值钱的一刀是：

```text
withGlobalTauri=false + backend-native tool confirmation + run_command deprecation
```

这三个做完，上一轮 Codex Security 报告里的核心攻击链基本就断了。

人话版：  
现在不是给保安配对讲机的时候，是先把钥匙柜从客厅挪到保险箱里。
