# Hajimi 当前技术债务总表（Web 只读 + 本地复核版）

> 生成时间：2026-05-15  
> 适用仓库：`Cognitive-Architect/hajimi-code-cli`  
> Web 端核验分支：`v3.8.0-batch-1`  
> 输入来源：`DEBT ALL IN ONE.zip` 内 14 份 Markdown 技术债文档 + GitHub 当前分支只读代码核验  
> 本地复核：2026-05-16，分支 `v3.8.0-batch-1`；Day 2 workspace resolver 修复与验证见 4.2 节，Day 3 文件操作专用命令修复与验证见 5.1 节，Day 4 Shell allow-list 收紧见 4.1 节，Day 6 CSP baseline 与 Global Tauri API 迁移计划见 4.3 节  
> 重要说明：本文档最初是“网页端看到的债务真相快照”；2026-05-16 已追加本地源码与聚焦测试复核。本地如果有后续未提交改动，以最新本地构建 / 测试结果为最终准绳。

---

## 0. 一句话结论

当前债务不是从零开始修。已有不少债务被推进或清偿，但仍然存在几类需要优先对齐的债务：

1. **P0 安全边界债**：Shell 用户白名单已在 Day 4 移除 shell 解释器并统一 metachar 拦截，状态进入 `VERIFY`；Tauri CSP 已在 Day 6 从 `null` 推进到 baseline，状态进入 `PARTIAL/VERIFY`；Global Tauri API 仍保持开启并等待迁移；workspace symlink 已在 Day 2 进入 `VERIFY`。
2. **P1 功能错配债**：前端“新建文件夹 / 重命名 / 删除”已在 Day 3 改为专用 Tauri commands，状态进入 `VERIFY`；仍建议补一次真实 Tauri UI smoke 后关闭。
3. **P1/P2 半成品体验债**：Thinking UI、Checkpoint/Replay、Command Palette、Slash Command Palette。
4. **P2 架构质量债**：前端 `app.js/style.css` 单体过大，Agent Prompt 已增强但还未形成稳定产品级策略。

人话版：这不是“满地都是 bug 没法救”，更像厨房已经装了不少新设备，但煤气阀、后门、菜刀架还要先上锁。不然界面再好看，迟早有人把锅端飞。

---

## 1. 本次核验范围

### 1.1 上传债务文档清单

| 文件 | 当前归类 | 本次处理方式 |
|---|---|---|
| `INDEX.md` | 总索引 | 作为旧状态基准 |
| `hajimi_codex_security_workflow.md` | 安全审查工作流 / 发现清单 | 逐条和当前代码对照 |
| `DEBT-UX-AGENT-001.md` | 启动 / 文件树 / 历史会话 | 和当前前后端代码对照 |
| `DEBT-THINKING-UI.md` | Thinking UI 方案 C 后续债 | 和当前 trace/checkpoint 代码对照 |
| `DEBT-THINKING-UI-BASELINE.md` | Thinking UI 基线 | 作为历史基线参考 |
| `DEBT-AGENT-PROMPT-001.md` | Agent Core 提示词债 | 和当前 planner/bridge 代码对照 |
| `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | UI 交互债 / 前端复杂度 | 作为架构债保留 |
| `02-slash-command-palette.md` | Slash Command 提示面板缺失 | 保留为 P1 UX 债 |
| `01-token-context-usage-tracking.md` | Token / Context 统计 | 多数已清偿，转入观察 |
| `DEBT-SCHEME-B.md` | Scheme B 精确 Token 汇总 | 多数已清偿，转入观察 |
| `SHELL-FEATURE-DEBT-002.md` | Shell 功能降级债 | 保留，等待安全方案后恢复 |
| `DEBT-P0-001.md` | WebRTC Signaling PSK 长期管理 | 需要确认模块仍是否 active |
| `DEBT-ACTIVE-DECLARATION.md` | Agent Core 清债声明 | 多数清偿，作为历史证据 |
| `DEBT-REWORK-001-声明.md` | 返工诚实声明 | 作为历史背景，不列入第一修复批 |

### 1.2 当前代码核验重点文件

| 文件 | 核验目的 |
|---|---|
| `src/engine/tool-system/src/shell.rs` | ShellTool 白名单 / shell 解释器 / 执行方式 |
| `src/interface/desktop/src/main.rs` | workspace 路径校验、run_command、checkpoint、AgentLoop trace 注入 |
| `src/interface/desktop/tauri.conf.json` | Tauri CSP / global API 暴露面 |
| `src/interface/web/app.js` | 新建文件夹、会话持久化、Thinking UI、DOM 渲染风险 |
| `src/intelligence/agent-core/llm/bridge.rs` | Agent Prompt、Tool Manifest、Context Window、Thinking 解析 |

### 1.3 本地复核补记（2026-05-16）

本地复核没有推翻原报告主体结论，但修正了三处粒度：

1. `CS-HAJIMI-004` 的原始问题不只是 `createNewFolder()` 调 `mkdir`，还包括 `renameFile()` 调 `mv`、`deleteFile()` 调 `rm -rf`；三者都不在桌面端 `run_command` 白名单内。Day 3 已改为专用 Tauri commands，详见 5.1。
2. `DEBT-P0-001` 已执行本地关键词搜索：当前 `src` 下没有发现 active signaling server / PSK runtime，只有 `src/intelligence/memory/src/cloud.rs` 的 WebRTC DataChannel 同步准备、测试和文档引用。因此状态从 `UNKNOWN` 收窄为 `ARCHIVE CANDIDATE`，正式归档前仍建议由模块 owner 确认产品路径。
3. `DEBT-AGENT-PROMPT-001` 的 feature gate 默认状态已本地确认：Persona、Planner V1、Context Window 默认开启；该债保持 `PARTIAL/P2`，不进入第一修复批。

本地验证命令：

```text
node --check src\interface\web\app.js
cargo test -p engine-tool-system -- test_allow_list
cargo test -p intelligence-agent-core test_decompose_goal_v1_valid_json
git diff --check
```

结果：全部通过。两个 `cargo test` 首次在沙箱写 `target` 时遇到 Windows `拒绝访问`，提升权限重跑后通过。

---

## 2. 状态标记规则

| 状态 | 含义 |
|---|---|
| `OPEN` | 当前代码仍能看到问题或缺口，应保留债务 |
| `PARTIAL` | 已经做了一部分，但还有关键验收项未闭环 |
| `VERIFY` | 代码看起来已改，但还需要本地构建 / 实机操作验证 |
| `CLEARED` | 从文档和当前代码看，已经基本闭环；只保留历史记录 |
| `ARCHIVE` | 历史背景，不进入当前修复队列 |
| `UNKNOWN` | 文档提到，但当前代码未找到足够证据；需要本地确认 |

---

## 3. 当前总览矩阵

| ID / 文档 | 类别 | 优先级 | 当前状态 | 本次判断 |
|---|---|---:|---|---|
| `CS-HAJIMI-001` / Shell 白名单绕过 | Security | P0 | `VERIFY` | Day 4 已从用户 allow-list 移除 `bash/sh/pwsh/powershell`，Bash/PowerShell 共用 metachar 拦截，测试与 workspace check 通过；等待实机 shell 工具 smoke 后关闭 |
| `CS-HAJIMI-002` / workspace symlink 逃逸 | Security | P0 | `VERIFY` | Day 2 已实现 `resolve_workspace_path` 并补 resolver 测试；等待 Day 3 文件操作复用后再关闭 |
| `CS-HAJIMI-003` / Tauri CSP/global API | Security | P0/P1 | `PARTIAL/VERIFY` | Day 6 已开启 CSP baseline 并修复静态 inline `onclick` blocker；`withGlobalTauri: true` 仍保留，等待模块化 Tauri API 迁移与实机 WebView smoke |
| `CS-HAJIMI-004` / mkdir/mv/rm 功能错配 | UX/Security | P1 | `VERIFY` | Day 3 已新增 `create_dir` / `rename_path` / `delete_path` 并替换前端入口；静态检查和桌面端测试通过，等待 Tauri UI smoke 后关闭 |
| `CS-HAJIMI-005` / SecurityAuditTool 偏轻量 | Security Quality | P2 | `OPEN` | 作为安全门禁增强项保留，不作为第一刀 |
| `DEBT-UX-AGENT-001` / 启动、文件树、历史会话 | UX | P1 | `VERIFY` | 代码层已出现 `get_current_workspace`、`loadChatSessions`、`saveChatSessions` 等修复痕迹，但需要本地运行确认 |
| `DEBT-THINKING-UI` | UX/Agent | P1/P2 | `PARTIAL/VERIFY` | trace 注入已做；Day 8-10 已完成 checkpoint model/export/compare/restore/replay V1，WebView smoke、richer file diff、事务级 restore 仍开放 |
| `DEBT-AGENT-PROMPT-001` | Agent Quality | P2 | `PARTIAL` | 已有 Tool Manifest、Planner V1、Context Window、Persona gate；可降级为质量增强债 |
| `02-slash-command-palette` | UX | P1 | `OPEN` | 当前已有命令列表和 slash 命令处理，但缺少输入提示面板验收证据 |
| `01-token-context-usage-tracking` | Token UX | P2 | `VERIFY/CLEARED` | 从代码可见 token stats / context optimize 已接入，转本地验收 |
| `DEBT-SCHEME-B` | Token Core | P2 | `VERIFY/CLEARED` | 文档自称已完成；当前主流程有累计统计接口，非当前第一修复批 |
| `SHELL-FEATURE-DEBT-002` | Security Downgrade | P2 | `OPEN BY DESIGN` | Shell 功能被安全降级是合理状态，恢复前必须先修 P0 Shell 边界 |
| `DEBT-P0-001` / Signaling PSK | Security | P0 if active | `ARCHIVE CANDIDATE` | 本地搜索未发现 active signaling server / PSK runtime；只发现 WebRTC DataChannel 同步准备、测试和文档引用 |
| `DEBT-P0-UI-INTERACTION-REMEDIATION` | Frontend Arch | P2 | `PARTIAL` | Day 13 已拆出 `security-dom.js` / `workspace.js` 并保留 `app.js` wrapper；sessions/thinking/style 仍未拆 |

---

## 4. P0 / 高优先级债务详情

### 4.1 `CS-HAJIMI-001` Shell 白名单可被 shell 解释器绕过

**当前状态：`VERIFY`**  
**建议优先级：P0**  
**影响区域：`src/engine/tool-system/src/shell.rs`**

#### 原始代码观察

原始 `ALLOWED_COMMANDS` 中包含：

```text
bash
sh
pwsh
powershell
```

同时执行流程是：

```text
check_allow_list(command)
-> 选择 bash/powershell 作为外层 shell
-> bash -c <用户命令> 或 powershell -Command <用户命令>
```

这意味着即便拦截了 `; | & $` 等元字符，允许 shell 解释器本身仍然会扩大攻击面。

#### 当前风险

- `bash script.sh` 这种形式可以把真正的危险动作藏进脚本文件。
- Windows 下 `powershell/pwsh` 被允许，测试里还明确断言允许 `powershell -Command ...` 和 `pwsh -Command ...`。
- 当前实现更像“第一层白名单”，不是完整命令沙箱。

#### 修复方向

短期：

1. 从用户可传入命令白名单中移除 `bash/sh/pwsh/powershell`。
2. 补回归测试，确保这些命令被拒绝。
3. 保留 `git/cargo/npm/node/python3/ls/cat/echo/pwd/which` 等低风险命令，但仍要限制参数。

中期：

1. 将命令从字符串改为结构化参数：`program + args[]`。
2. 按命令设计子命令级规则，例如 `git status` 可允许，`git push --force` 需要确认。
3. 高风险命令走额外确认 / sandbox。

#### 回归测试建议

```text
bash script.sh                 => reject
sh script.sh                   => reject
pwsh -Command Get-Process      => reject
powershell -Command Get-Date   => reject
git status                     => allow
cargo check                    => allow
echo hello                     => allow
echo ; rm -rf /                => reject
```

人话版：不要让用户直接把”万能钥匙师傅”请进屋。要修水管，就只给扳手；别把整套开锁工具都递过去。

#### Day 04 修复与验证

**修改日期**: 2026-05-16  
**修改人**: Engineer (Day 04 工单 B-04/15) + 收尾复审

**核心变更**:
- 从 `ALLOWED_COMMANDS` 移除 `bash`, `sh`, `pwsh`, `powershell`（shell.rs:20-42）
- 更新 `test_allow_list`：`powershell -Command` 和 `pwsh -Command` 改为 `is_err()`
- 新增 `bash script.sh` 和 `sh script.sh` 拒绝测试断言
- 保留内部 `ShellExecutor::shell_cmd()` 外层选择逻辑（不影响用户白名单）
- 将 Bash / PowerShell 的 forbidden metachar 收敛到共享常量，覆盖分号、管道、重定向、变量替换、命令替换等 shell 元字符
- 新增 PowerShell 管道、重定向、命令替换拒绝测试：`git status | ...`、`git status > ...`、`echo $(...)`

**验证命令**:
```text
cargo check -p engine-tool-system
Finished `dev` profile ... 0 errors

cargo fmt -- --check
(no output, formatting OK)

cargo test -p engine-tool-system -- test_allow_list
test shell::tests::test_allow_list ... ok

cargo test -p engine-tool-system
73 passed; 0 failed

cargo clippy -p engine-tool-system -- -D warnings
Finished `dev` profile ... 0 errors

cargo check --workspace
Finished `dev` profile ... 0 errors
```

**验收证据**:
- `ALLOWED_COMMANDS` 已不含 `bash/sh/pwsh/powershell`
- `bash script.sh` / `sh script.sh` / `powershell -Command` / `pwsh -Command` 全部被 `check_allow_list` 拒绝
- `git status`, `cargo check`, `ls -la` 等低风险命令仍通过
- `echo ; rm -rf /`、`git status | Get-Process`、`git status > out.txt`、`echo $(Get-Process)` 等 metachar 组合仍被拒绝
- 内部执行器（`ShellExecutor::shell_cmd()`）保留，仅用于平台 wrapper

**仍需后续验证**:
- 实机 Tauri dev 手动测试：shell 工具调用 `bash` 应返回 “not in strict allow-list”

人话版：Day 04 把 shell 解释器从”用户万能钥匙”降级为”内部执行器专属”，同时把 PowerShell 的管道口也堵上了；低风险命令仍可用，安全边界收紧。

---

### 4.2 `CS-HAJIMI-002` workspace symlink 新文件写入逃逸

**当前状态：`VERIFY`**  
**建议优先级：P0**  
**影响区域：`src/interface/desktop/src/main.rs`**

#### 原始缺陷

旧实现对不存在目标使用 `canonicalize().unwrap_or(resolved)`。当 workspace 内存在指向外部目录的 symlink / junction 时，新文件写入可能绕过真实路径校验。

典型风险：

```text
workspace/
  link -> /tmp/outside/

write_file("link/new.txt", "payload")
```

旧校验看到的是 `workspace/link/new.txt`，实际写入可能落到 `/tmp/outside/new.txt`。

#### Day 2 修复（2026-05-16）

- 新增 `PathIntent` enum：`ExistingFile`、`ExistingDir`、`NewFile`、`NewDir`、`AnyExisting`。
- 实现 `resolve_workspace_path(input, base_dir, intent)` 统一解析函数。
- 替换 `read_file` / `write_file` / `list_dir` 的旧 `validate_path_within_workspace` 调用。
- existing 路径 canonicalize 目标本身，并按 intent 校验文件 / 目录类型。
- new 路径 canonicalize 父目录，再拼接 leaf name。
- 移除 `canonicalize().unwrap_or(resolved)` 旧 fallback。
- 修复 resolver 测试临时目录并行竞态。
- 新增 Windows junction / symlink 父目录外跳拒绝测试。

#### 验收证据

```text
cargo check -p hajimi-desktop
Finished `dev` profile ... 0 errors

cargo test -p hajimi-desktop
8 passed; 0 failed

cargo clippy -p hajimi-desktop -- -D warnings
Finished `dev` profile ... 0 errors

rg -n "unwrap_or\(resolved\)|validate_path_within_workspace\(" src/interface/desktop/src/main.rs
无命中
```

Windows 本地测试已创建 junction：

```text
test_resolve_new_file_rejects_parent_symlink_escape ... ok
```

#### 仍需后续验证

- Day 3 `create_dir` / `rename_path` / `delete_path` 已复用本 resolver。
- 已补删除、重命名、新建目录的自动化回归；正式关闭前仍建议补真实 Tauri UI smoke 和 symlink 逃逸操作验收。

人话版：地址边界和文件操作现在已经接到同一套门禁上，下一步只差在真实桌面 UI 里再走一遍操作流。

---

### 4.3 `CS-HAJIMI-003` Tauri CSP 关闭 + Global Tauri API 开启

**当前状态：`PARTIAL/VERIFY`**  
**建议优先级：P0/P1**  
**影响区域：`src/interface/desktop/tauri.conf.json` + 前端 DOM 渲染**

#### 原始问题

原始 Tauri 配置同时存在 `withGlobalTauri: true` 与 `csp: null`，未来一旦文件名、Git 输出、聊天内容或错误信息形成 XSS，后果会被放大。

#### Day 6 本地修复观察

当前 Tauri 配置已从 `csp: null` 推进到基础 baseline：

```json
{
  "app": {
    "withGlobalTauri": true,
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: data:; connect-src 'self' http://127.0.0.1:* http://localhost:*"
    }
  }
}
```

Day 5 已完成 DOM render audit 与 escape 基线；Day 6 收尾阶段额外修复欢迎页 inline `onclick`，改为 `data-welcome-action` + `addEventListener`，避免 `script-src 'self'` 下点击入口被 CSP 拦截。

`withGlobalTauri` 仍保持 `true`。本地扫描 `src/interface/web/app.js` 仍有 52 个 `window.__TAURI__` 命中，覆盖 Workspace/FileOps、Git/Search、Chat/LLM、Provider/Profile、MCP、Trace/Checkpoint 等核心路径，直接关闭会导致主流程中断。迁移计划见 `docs/debt/SECURITY-CSP-VERIFY.md`。

#### 风险说明

CSP baseline 已降低脚本注入后的执行面，但这不等于 `CS-HAJIMI-003` 已关闭。只要 `withGlobalTauri` 仍为 `true`，前端 XSS 后果仍可能触达 Tauri 命令面，因此当前只能标为 `PARTIAL/VERIFY`，不能标 `CLEARED`。

#### 修复方向

1. 保持 CSP baseline，不回退到 `csp: null`。
2. 逐步关闭 `withGlobalTauri`，前端改为模块化 Tauri API 调用。
3. 全面审查 `innerHTML`。原则：
   - 用户输入、文件名、Git 输出、模型输出必须 escape。
   - 能用 `textContent` / DOM API，就别拼 HTML。

#### Day 6 验证记录

```text
node --check src/interface/web/app.js
cargo check -p hajimi-desktop
cargo clippy -p hajimi-desktop -- -D warnings
rg -n "\"csp\"\s*:\s*null" src/interface/desktop/tauri.conf.json
rg -n "onclick=" src/interface/web src/interface/desktop/tauri.conf.json
rg -n "default-src \*|script.*https://|cdn|<script[^>]*https" src/interface/web src/interface/desktop/tauri.conf.json
```

结果：语法、编译与 clippy 通过；`csp:null`、inline `onclick`、远程脚本 / CDN / `default-src *` 均无命中。

#### 验收建议

```text
恶意文件名：<img src=x onerror=alert(1)>
恶意聊天内容：<script>alert(1)</script>
恶意 Git 输出：包含 HTML 标签
预期：只显示文本，不执行脚本
```

---

## 5. P1 功能错配债务

### 5.1 `CS-HAJIMI-004` 文件操作前后端不对齐

**当前状态：`VERIFY`**  
**建议优先级：P1**  
**影响区域：`src/interface/web/app.js` + `src/interface/desktop/src/main.rs`**

#### 原始问题

原始债务不是“Shell 白名单少了三个命令”这么简单，而是 UI 文件操作把高风险系统命令暴露成通用执行路径：

```text
createNewFolder() -> run_command(cmd: "mkdir")
renameFile()      -> run_command(cmd: "mv")
deleteFile()      -> run_command(cmd: "rm", args: ["-rf", path])
```

后端 `run_command` 白名单不允许 `mkdir/mv/rm`，所以功能不可用；如果直接把 `rm` 等命令加入通用白名单，又会扩大安全面。

#### Day 3 修复状态

2026-05-16 本地收尾已完成以下修复：

1. `src/interface/desktop/src/main.rs` 新增并注册专用 Tauri commands：

```text
create_dir(path)
rename_path(old_path, new_path)
delete_path(path, recursive)
```

2. 三个命令统一复用 Day 2 的 `resolve_workspace_path()`，不把 `mkdir/mv/rm` 加回通用 Shell 白名单。
3. `delete_path()` 已按目标类型分流：文件始终使用 `remove_file`，目录按 `recursive` 选择 `remove_dir_all` / `remove_dir`，避免 `recursive=true` 删除普通文件失败。
4. `src/interface/web/app.js` 的新建文件夹、重命名、删除入口已切换到 `create_dir` / `rename_path` / `delete_path`。
5. 新增 create / rename / delete helper 级回归测试，覆盖文件删除、递归目录删除和非递归删除非空目录失败。

#### 本地验证证据

```text
node --check src/interface/web/app.js
cargo fmt -- --check
cargo check -p hajimi-desktop
cargo clippy -p hajimi-desktop -- -D warnings
cargo test -p hajimi-desktop
rg "cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js
rg '"mkdir"|"mv"|"rm"' src/engine/tool-system/src/shell.rs src/interface/desktop/src/main.rs
git diff --check
```

结果：全部通过；`rg` 检查未发现前端继续调用 `mkdir/mv/rm`，也未发现 Shell 白名单新增这些命令。

#### 剩余关闭条件

状态保持 `VERIFY`，不直接标记 `CLEARED`。建议在下一次可运行桌面端时补一轮 Tauri UI smoke：

```text
点击“新建文件夹” => 成功创建 workspace 内文件夹
重命名 workspace 内文件/文件夹 => 成功
删除 workspace 内文件/文件夹 => 明确确认后成功
越界路径 / symlink 逃逸路径 => reject
```

---

## 6. UX 债务状态

### 6.1 `DEBT-UX-AGENT-001` 启动闪屏 / 文件树 / 历史会话

**当前状态：`VERIFY`**  
**建议优先级：P1**

#### 已看到的修复痕迹

当前前端初始化流程已经调用：

```text
initWorkspace() -> loadFileTree()
loadChatSessions()
```

当前 `initWorkspace()` 会从后端拿 `get_current_workspace`，不再直接依赖浏览器当前路径。

当前会话持久化已经存在：

```text
loadChatSessions()
saveChatSessions()
switchSession(id)
renderSessionList()
localStorage key: hajimi_chat_sessions
```

#### 仍需本地验收

```text
1. 双击 target/release/hajimi-desktop.exe 启动
2. 不出现“加载文件树失败”toast
3. 文件树显示 hajimi-workspace
4. 新建会话后旧会话仍在左侧列表
5. 关闭应用再打开，会话仍存在
```

#### 当前判断

代码层面：已修或部分已修。  
产品层面：必须本地跑一次确认。因为这类问题和 Windows 路径、Tauri 打包路径、localStorage 状态强相关。

#### Day 7 收尾复核补记（2026-05-16）

Day 7 尝试按 A 级标准补 Tauri UI smoke，但未能形成窗口级验收证据：

```text
npm run dev
=> npx serve 因 ENOTCACHED 失败，serve 不在离线 cache 中

临时 Node 静态 server
=> http://127.0.0.1:3456 返回 200

cargo tauri dev
=> 编译 hajimi-desktop 时 target/debug/incremental 路径拒绝访问 (os error 5)
```

已登记专门债务：

```text
docs/debt/DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md
```

因此 `DEBT-UX-AGENT-001` 继续保持 `VERIFY`，不得迁移到 `CLEARED`。关闭条件是成功打开 Tauri 窗口，并补齐启动、文件树、Day 3 文件操作、会话 A/B 与关闭重开证据。

---

## 7. Thinking UI / Agent Trace 债务状态

### 7.1 `DEBT-THINKING-UI` 当前状态

**当前状态：`PARTIAL`**  
**建议优先级：P1/P2**

#### 已推进部分

当前 `AgentLoop` 创建后会取 `trace_tx` 并注入 `AppState`：

```text
if let Some(tx) = agent_loop.trace_tx() {
    state.set_trace_tx(tx);
}
```

`subscribe_agent_trace` 会订阅 trace channel，把事件发给前端，并同步 emit `agent:trace`。

这说明原 baseline 里的 “trace_tx 没接上 / 前端全靠模拟” 已经至少部分改善。

#### 仍然开放的债务

Day 08 已移除 checkpoint 相关命令的假成功返回；Day 09 已完成 export / compare V1；Day 10 已完成 safe restore / replay V1：

```text
restore_checkpoint(id, confirmRestore, dryRun) -> Ok(RestoreResult) / safe Err
compare_checkpoints(id_a, id_b) -> Ok(CheckpointCompareResult { files_added, files_modified, files_removed, summary, data_source, ... })
export_checkpoint(id) -> Ok(pretty_json_checkpoint_or_bundle)
```

当前 checkpoint DTO、workspace-local 存储和 trace 事件落盘入口已存在；Store checkpoint 捕获已改为大小写无关匹配，覆盖 `Storing checkpoint for iteration ...`。`export_checkpoint(id)` 会读取 `<workspace>/.hajimi/checkpoints/*.json`，支持单个 checkpoint 和 `id == "all"` bundle。`compare_checkpoints(id_a, id_b)` 会按 `CheckpointRecord.files` 分类 added / modified / removed；当 `files` 为空时明确降级到 `checkpoint.diff_summary+metadata`，不生成假文件级 diff。`restore_checkpoint(id, confirmRestore, dryRun)` 会真实读取 checkpoint，支持 dry-run plan，非 dry-run 必须 `confirmRestore == true`，写入前创建 backup，路径走 `resolve_workspace_path` 派生的 restore resolver，写入失败会 best-effort rollback。Replay 已可从 `list_checkpoints` 和 `get_edit_history` 生成只读回放事件。

仍未完成的是 WebView 实机 trace/export/compare/restore/replay smoke、事务日志式跨文件原子 restore，以及从 edit payload / git diff / file hash 填实 `CheckpointRecord.files.content` / `after_content` 的 richer diff。Operation Summary 现在会明确显示 `TraceEvent.operation_summary` 来源和“无文件级 diff 数据”降级说明，但它仍不是完整 git/file diff evidence。

#### 推荐后续拆分

| 子债 | 优先级 | 说明 |
|---|---:|---|
| Trace 事件真实链路验收 | P1 | 确认前端是否实时收到 AgentLoop 事件 |
| Checkpoint restore/replay safe V1 | P1 | Day 10 已完成；事务级原子提交仍为后续债务 |
| Checkpoint content snapshots / richer diff 填实 | P1 | export/compare/restore V1 已完成，但真实写入 restore 依赖 `content` / `after_content` 后续填充 |
| WebView trace/export/compare/restore/replay 实机验收 | P1 | 当前仍缺真实桌面会话点验 |
| Operation Summary 接真实文件级 diff | P2 | 已停止伪造文件名，但仍不是完整 git/file diff evidence |
| Thinking 标签跨 chunk 解析压力测试 | P2 | 当前已有 parser，但需测畸形流 / 半截标签 |

---

## 8. Agent Prompt 债务状态

### 8.1 `DEBT-AGENT-PROMPT-001`

**当前状态：`PARTIAL`，建议降级为 P2 质量债**

#### 已看到的推进

当前 `PlannerLlmBridge` 已经有：

1. Legacy planner prompt。
2. Planner V1 DTO 解析。
3. Tool Manifest 动态注入。
4. suggested tools 过滤未知工具。
5. expected evidence / stop conditions 写入 metadata。
6. ContextWindowManager feature gate。
7. Persona feature gate。
8. Thinking 标签提取并写入 blackboard。

这说明文档里“完全没有工具感知 / 零 system prompt / 无上下文策略”的状态已经不是当前完整现实。

#### 仍然存在的质量问题

1. 旧路径仍然存在：legacy prompt 仍比较简单。
2. feature gate 默认开关状态未知，需要本地确认。
3. Planner / Reflector / Executor 的提示词策略仍可能不一致。
4. 没看到完整“产品级 Agent Persona 规范”与验收集。

#### 推荐后续做法

不要第一刀修这里。等安全边界和 UX 错配修完后，再做 Agent Prompt V2：

```text
AGENT-PERSONA.md
Tool Manifest Schema
Planner Prompt Contract
Reflector Critique Contract
Context Budget Policy
Failure Recovery Policy
Golden Task Regression Set
```

#### Day 11 契约文档化补记（2026-05-17）

Day 11 已在 `docs/agent-prompt-core/` 下补齐 Prompt V2 契约文档：

```text
AGENT-PERSONA.md
PLANNER-PROMPT-CONTRACT.md
REFLECTOR-CONTRACT.md
EXECUTOR-CONTRACT.md
TOOL-MANIFEST-SCHEMA.md
```

这些文档对齐当前 `agent-core` 代码事实，覆盖 Persona、Planner V1、Reflector V1、ActExecutor/ToolCall、Tool Manifest 的输入、输出、fallback、feature-gate、证据字段和 Day 12 golden regression 映射。该补记不代表 Prompt V2 已全量产品化；`DEBT-AGENT-PROMPT-001` 继续保持 `PARTIAL/P2`，等待 Day 12 golden cases 和后续运行时增强。

#### Day 12 Golden Regression 补记（2026-05-17）

Day 12 已新增不依赖真实 LLM / 网络的 golden regression：

```text
tests/agent_prompt_golden/README.md
tests/agent_prompt_golden/planner/*.json      # 5 cases
tests/agent_prompt_golden/reflector/*.json    # 5 cases
tests/agent_prompt_golden/toolcall/*.json     # 3 cases
src/intelligence/agent-core/prompt_golden_tests.rs
```

当前 harness 通过 `include_str!` 显式加载 fixtures，校验 `PlannerSubgoalPlanV1Dto`、`ReflectorCritiqueV1Dto`、`ToolCallV1` / `ActDecision` 的反序列化和关键契约字段，覆盖 success / failure / unknown / retry / stop-loss / safe_read / risky_write / cannot_act。该补记仍不代表 Prompt V2 全量产品化；`DEBT-AGENT-PROMPT-001` 保持 `PARTIAL/P2`，后续仍需 live ToolRegistry manifest、Act LLM decision、动态 fixture discovery 等增强。

---

## 9. Token / Context 债务状态

### 9.1 `01-token-context-usage-tracking.md`

**当前状态：`VERIFY/CLEARED`**

文档显示前端估算、后端精确统计、上下文压缩、Token Tracker Integration 多数已完成。当前前端也能看到：

```text
loadCumulativeFromBackend()
updateTokenDisplay()
/compact
optimize_context
promptTokens / completionTokens
```

建议不进入第一批修复，只做回归验收。

### 9.2 `DEBT-SCHEME-B.md`

**当前状态：`VERIFY/CLEARED`**

文档自身是批次汇总和清偿记录。当前代码已有累计统计接口 `get_cumulative_stats` 注册。保留为历史记录，不作为当前主动修复项。

---

## 10. Shell 功能降级债务

### 10.1 `SHELL-FEATURE-DEBT-002`

**当前状态：`OPEN BY DESIGN`**  
**建议优先级：P2，仍需安全 sandbox 方案后再恢复复杂功能**

当前 shell 工具严格限制了管道、重定向、子 shell、变量替换等复杂功能。这是安全换体验，短期合理。

Day 4 已清理用户 allow-list 中的 `bash/sh/pwsh/powershell`，并统一 Bash / PowerShell metachar 拦截。复杂 shell 能力仍不应在当前阶段恢复。

恢复复杂 shell 功能前，必须有：

```text
1. sandbox / firejail / nsjail / Windows Job Object 之类隔离策略
2. cwd 限制
3. env 限制
4. 网络限制策略
5. 审计日志
6. 用户确认等级
7. 超时 / 输出限制
```

---

## 11. 前端架构债务

### 11.1 `DEBT-P0-UI-INTERACTION-REMEDIATION`

**当前状态：`PARTIAL`**  
**建议优先级：P2，暂不大重构**

文档提到 `app.js` / `style.css` 已经很大，并依赖 `window.app` 单例、DOM ID 和直接事件绑定。当前从代码片段看，这个判断仍成立。

#### 当前风险

- 小改容易误伤旧功能。
- 测试粒度不够，UI 回归靠手感。
- 安全修复时，`innerHTML` 和模板拼接很难一次性审完。

#### 建议策略

不要现在大重构。先建立“边修边切”的拆分策略：

```text
src/interface/web/modules/
  workspace.js
  sessions.js
  command-palette.js
  slash-palette.js
  thinking-ui.js
  security-dom.js
```

第一阶段只拆和本次修复强相关的模块，比如 workspace path / create folder / session。

#### Day 13 security-dom + workspace 渐进拆分补记（2026-05-17）

Day 13 已新增无 bundler 前端模块：

```text
src/interface/web/modules/security-dom.js
src/interface/web/modules/workspace.js
```

`security-dom.js` 通过 `window.HajimiSecurityDom` 暴露 `safeText`、`escapeHtml`、`escapeAttr`、`setSafeHtml`；`workspace.js` 通过 `window.HajimiWorkspace` 承接 `initWorkspace`、`loadFileTree`、`buildTreeFromEntries`、`renderFileTree`、`renderTreeNode`、`createNewFolder`、`renameFile`、`deleteFile` 等 workspace/file tree/file ops 高频逻辑。`app.js` 保留同名 wrapper，`window.app` 兼容入口未删除；`index.html` 改为普通 defer script 顺序加载模块和 `app.js`。

该补记只代表前端架构债从单体化 `OPEN` 推进到 `PARTIAL`：sessions、thinking-ui、command/slash palette 与 `style.css` 拆分仍未处理，不能标记为 `CLEARED`。

#### Day 14 sessions + thinking-ui 渐进拆分补记（2026-05-17）

Day 14 在 Day 13 模块模式上继续新增：

```text
src/interface/web/modules/sessions.js
src/interface/web/modules/thinking-ui.js
```

`sessions.js` 通过 `window.HajimiSessions` 承接 `hajimi_chat_sessions` 兼容存储、A/B 会话切换、会话列表渲染与 reload 恢复；`thinking-ui.js` 通过 `window.HajimiThinkingUI` 承接 Thinking tag 解析、trace card 渲染、真实 `subscribe_agent_trace` 链路处理、Operation Summary、Timeline/Replay helper。`app.js` 继续保留旧方法 wrapper，checkpoint `export_checkpoint` / `compare_checkpoints` / `restore_checkpoint` 仍保持真实 Tauri invoke 路径。

该补记仍只代表前端架构债保持 `PARTIAL/P2`：command/slash palette、provider/settings、`style.css` 等大块仍未拆，真实 Tauri 窗口手动 smoke 仍需后续补证，不能标记为 `CLEARED`。

---

## 12. Slash Command Palette 债务

### 12.1 `02-slash-command-palette.md`

**当前状态：`OPEN`**  
**建议优先级：P1/P2**

当前代码已经有 slash 命令处理：

```text
/tools
/providers
/tool
/chat
/mcp
/search
/git
/extensions
/compact
```

但文档重点是“输入 `/` 后应该出现提示面板”。当前只读核验没有看到完整 Slash Command Palette 的交互验收证据。

建议状态保留为 `OPEN`，但不进入第一修复批。因为它是体验增强，不是基础安全或功能错配。

---

## 13. Signaling PSK 债务

### 13.1 `DEBT-P0-001`

**当前状态：`ARCHIVE CANDIDATE`**  
**建议优先级：P0 if active / ARCHIVE if inactive**

文档记录 WebRTC Signaling Server PSK 长期管理问题，包括 KMS/Vault/Rotation。2026-05-16 本地复核已执行关键词搜索，当前 `src` 下没有发现 active signaling server / PSK runtime；可见的 WebRTC 命中主要来自 `src/intelligence/memory/src/cloud.rs` 的 DataChannel 同步准备、测试样例和文档引用。

#### 已执行确认命令

在本地仓库运行：

```bash
grep -R "WebRTC\|signaling\|psk\|pre-shared\|KMS\|Vault" -n src crates docs Cargo.toml package.json 2>/dev/null | head -n 80
```

#### 当前判断

该债不建议进入 Batch 1。若模块 owner 确认当前产品路径没有 signaling server，则可正式降级为 `ARCHIVE`；若后续恢复 signaling server，则必须重新打开为 P0，并补 PSK 生命周期、轮换和常量时间校验策略。

---

## 14. 建议的新债务目录结构

建议在仓库中新增一个“当前真相快照”，不要覆盖旧债务文档：

```text
docs/debt/
  DEBT-CURRENT-STATUS-2026-05-15.md      # 本文档
  SECURITY-P0-BATCH-1.md                 # 下一批安全修复计划
  UX-FILETREE-SESSION-VERIFY.md          # 启动/文件树/会话验收记录
  archive/
    old-index-2026-05-15.md
```

这样能避免“旧债务文档说没修，当前代码其实已修一半”的信息错配。

---

## 15. 推荐修复批次

### Batch 0：对账，不改代码

目标：让网页端 / 本地 workspace / 债务文档三方一致。

```text
1. 把本文档放入 docs/debt/DEBT-CURRENT-STATUS-2026-05-15.md
2. 本地运行只读 grep/check 命令
3. 给每个债务打状态：OPEN / PARTIAL / VERIFY / CLEARED / ARCHIVE / UNKNOWN
4. 不做功能代码修改
```

验收：本文档进入仓库，团队知道下一刀从哪里下。

---

### Batch 1：安全 + 文件夹功能最小修复

目标：修掉本机能力边界和明显功能错配。

```text
1. 新增安全路径解析函数
2. 新增 create_dir Tauri command
3. 前端 createNewFolder 改用 create_dir
4. ShellTool 移除 bash/sh/pwsh/powershell 用户白名单
5. 补最小回归测试
```

验收：

```bash
cargo test -p engine-tool-system -- test_allow_list
cargo check --workspace
node --check src/interface/web/app.js
```

人工验收：

```text
新建文件夹成功
重命名成功
删除经确认后成功
shell 拒绝 bash/sh/pwsh/powershell
symlink 指向 workspace 外时写入被拒绝
```

---

### Batch 2：Tauri 安全面收敛

目标：降低未来 XSS 的后果。

```text
1. 全量审查 innerHTML
2. 统一 escape 工具
3. 开启 CSP
4. 关闭 withGlobalTauri 或按 `docs/debt/SECURITY-CSP-VERIFY.md` 执行迁移计划
5. 加恶意文件名 / Git 输出 / chat 内容测试
```

---

### Batch 3：UX 验收与小修

目标：把“已修改待验证”的 UX 债务真正关单。

```text
1. 启动无 toast
2. 文件树稳定加载
3. 会话持久化稳定
4. 新会话 / 切换会话 / 重启恢复可用
5. 本地验收截图或日志入档
```

---

### Batch 4：Thinking UI & Checkpoint 真闭环

目标：把演示 UI 变成可信操作记录。

```text
1. restore_checkpoint safe V1（Day 10 已完成；事务级原子提交仍待后续）
2. compare_checkpoints 真 diff V1（Day 09 已完成；richer file diff 仍待 CheckpointRecord.files 填实）
3. export_checkpoint 导出真实 JSON/Markdown（Day 09 已完成）
4. Operation Summary 接真实 diff 来源（Day 09 已停止伪造文件名；完整 git/file diff 仍待后续）
5. Replay 和后端 checkpoint 绑定（Day 10 已完成只读回放 V1）
```

---

### Batch 5：Agent Prompt V2

目标：质量提升，不抢 P0。

```text
1. AGENT-PERSONA.md
2. Planner/Reflector/Executor 统一提示词契约
3. Tool Manifest schema 固化
4. Context Budget Policy
5. Golden Task Regression Set
```

---

## 16. 本地对账命令清单

### 16.1 一键生成债务对账信息

Windows PowerShell：

```powershell
# 在仓库根目录运行
$Out = "docs/debt/local-debt-audit-$(Get-Date -Format yyyyMMdd-HHmmss).txt"
New-Item -ItemType Directory -Force docs/debt | Out-Null
"# Local Debt Audit" | Out-File $Out -Encoding utf8
"## Git" | Out-File $Out -Append -Encoding utf8
git branch --show-current | Out-File $Out -Append -Encoding utf8
git rev-parse HEAD | Out-File $Out -Append -Encoding utf8
git status --short | Out-File $Out -Append -Encoding utf8
"## Shell allow-list" | Out-File $Out -Append -Encoding utf8
Select-String -Path src/engine/tool-system/src/shell.rs -Pattern 'bash|sh|pwsh|powershell|ALLOWED_COMMANDS' | Out-File $Out -Append -Encoding utf8
"## Workspace validation" | Out-File $Out -Append -Encoding utf8
Select-String -Path src/interface/desktop/src/main.rs -Pattern 'validate_path_within_workspace|canonicalize|create_dir|run_command|get_current_workspace' | Out-File $Out -Append -Encoding utf8
"## Tauri security" | Out-File $Out -Append -Encoding utf8
Select-String -Path src/interface/desktop/tauri.conf.json -Pattern 'withGlobalTauri|csp' | Out-File $Out -Append -Encoding utf8
"## Frontend folder/session" | Out-File $Out -Append -Encoding utf8
Select-String -Path src/interface/web/app.js -Pattern 'createNewFolder|mkdir|loadChatSessions|saveChatSessions|hajimi_chat_sessions' | Out-File $Out -Append -Encoding utf8
Write-Host "Wrote $Out"
```

macOS / Linux / Git Bash：

```bash
mkdir -p docs/debt
OUT="docs/debt/local-debt-audit-$(date +%Y%m%d-%H%M%S).txt"
{
  echo '# Local Debt Audit'
  echo '## Git'
  git branch --show-current
  git rev-parse HEAD
  git status --short
  echo '## Shell allow-list'
  grep -nE 'bash|sh|pwsh|powershell|ALLOWED_COMMANDS' src/engine/tool-system/src/shell.rs || true
  echo '## Workspace validation'
  grep -nE 'validate_path_within_workspace|canonicalize|create_dir|run_command|get_current_workspace' src/interface/desktop/src/main.rs || true
  echo '## Tauri security'
  grep -nE 'withGlobalTauri|csp' src/interface/desktop/tauri.conf.json || true
  echo '## Frontend folder/session'
  grep -nE 'createNewFolder|mkdir|loadChatSessions|saveChatSessions|hajimi_chat_sessions' src/interface/web/app.js || true
} > "$OUT"
echo "Wrote $OUT"
```

---

## 17. 风险提醒

1. **不要先大重构前端。** 当前更需要先修安全和明确功能错配。
2. **不要为修新建文件夹直接放开 `mkdir/rm/mv`。** 应做专用 Tauri command。
3. **不要把文档里的“已修”当成最终事实。** 必须本地跑验收。
4. **不要把网页端结论当成本地未提交代码的结论。** 如果本地 workspace 有改动，优先以本地 audit 输出为准。
5. **不要忽视 Windows 路径行为。** symlink/junction、盘符、反斜杠路径都要测。

---

## 18. Day 15 清债收口验证补记（2026-05-17）

Day 15 已完成最终收口验证与 closure 文档:

```text
docs/debt/DEBT-REMEDIATION-CLOSURE-2026-05-17.md
```

### 18.1 最终验证命令

```text
git branch --show-current
git rev-parse HEAD
cargo check --workspace
cargo fmt -- --check
cargo test -p engine-tool-system
cargo test -p intelligence-agent-core --lib
node --check src/interface/web/app.js
Get-ChildItem -LiteralPath src/interface/web/modules -Filter *.js | ForEach-Object { node --check $_.FullName }
rg -n "use interface|interface::" src/engine src/intelligence
rg -n "workspace resolver|CSP|checkpoint|frontend modules|modules|security-dom|prompt golden" src/ARCHITECTURE.md src/INDEX.md
git diff --check
```

结果摘要:

- `cargo check --workspace`: 首次普通沙箱因 Windows `target/debug/incremental` ACL 拒绝访问失败；提升权限复跑通过。
- `cargo fmt -- --check`: 通过。
- `cargo test -p engine-tool-system`: 73 passed；0 failed。
- `cargo test -p intelligence-agent-core --lib`: 161 passed；0 failed；包含 Day 12 `prompt_golden_tests::*`。
- `node --check src/interface/web/app.js` 与 `modules/*.js`: 通过。
- 分层扫描 `use interface|interface::`: 无命中。
- `git diff --check`: 通过；仅 CRLF warning，无 whitespace error。

### 18.2 Day 15 状态结论

| ID / 文档 | Day 15 状态 | 收口说明 |
|---|---|---|
| `CS-HAJIMI-001` | `VERIFY` | 自动化验证通过，shell runtime smoke 后可关闭。 |
| `CS-HAJIMI-002` | `VERIFY` | workspace resolver 与路径测试已覆盖，仍建议 GUI/symlink 实机 smoke。 |
| `CS-HAJIMI-003` | `PARTIAL/VERIFY` | CSP baseline 已启用；`withGlobalTauri: true` 仍保留，不能关闭。 |
| `CS-HAJIMI-004` | `VERIFY` | 专用 Tauri commands 已接入；缺 Tauri UI smoke。 |
| `CS-HAJIMI-005` | `OPEN` | 本批次未处理。 |
| `DEBT-UX-AGENT-001` | `VERIFY` | 代码路径和构建通过；GUI smoke 被 `DEBT-UX-B07-001` 阻塞。 |
| `DEBT-THINKING-UI` | `PARTIAL/VERIFY` | Checkpoint export/compare/restore/replay V1 已完成；WebView smoke、richer diff、事务级 restore 仍保留。 |
| `DEBT-AGENT-PROMPT-001` | `PARTIAL/P2` | Contracts + golden regression 已落地；live runtime prompt 质量增强仍待后续。 |
| `02-slash-command-palette` | `OPEN` | 本批次未实现 slash suggestion panel。 |
| `SHELL-FEATURE-DEBT-002` | `OPEN BY DESIGN` | 复杂 shell 继续安全降级。 |
| `DEBT-P0-001` | `ARCHIVE CANDIDATE` | 当前未发现 active signaling server；仍待 owner 确认归档。 |
| `DEBT-P0-UI-INTERACTION-REMEDIATION` | `PARTIAL/P2` | security-dom/workspace/sessions/thinking-ui 已拆；command/slash/provider/style 仍未拆。 |

### 18.3 提交提醒

`docs/` 当前被 ignore。若提交本批次产物，必须显式处理:

```text
git add -f docs/debt docs/agent-prompt-core "docs/roadmap/hajimi debtFix"
```

---

## 19. 当前推荐的一句话状态

```text
当前项目已完成多轮债务推进，但仍存在 P0/P1 安全边界债务（Global Tauri API 迁移）和若干 P1/P2 体验债。Shell 用户白名单已在 Day 4 进入 `VERIFY`；Tauri CSP 已在 Day 6 进入 baseline `PARTIAL/VERIFY`；workspace symlink 逃逸已在 Day 2 进入 `VERIFY`；文件操作错配已在 Day 3 改为 `create_dir` / `rename_path` / `delete_path` 专用命令并进入 `VERIFY`，等待真实 Tauri UI smoke 后再关闭。
```

---

## 20. 附录：旧文档状态建议

| 旧文档 | 建议状态 | 处理建议 |
|---|---|---|
| `INDEX.md` | 需要更新 | 把本文档加入 Active/Current Snapshot |
| `hajimi_codex_security_workflow.md` | 保留 | 安全审查流程有效，但发现状态需按本文档更新 |
| `DEBT-UX-AGENT-001.md` | 改为 `VERIFY` | 代码层已改，等待实机验收 |
| `DEBT-THINKING-UI.md` | 改为 `PARTIAL/VERIFY` | Day 8-10 已推进 checkpoint/replay V1；WebView smoke、richer diff、事务级 restore 仍开放 |
| `DEBT-THINKING-UI-BASELINE.md` | 归档 | 作为 Day 1 baseline，不再代表当前状态 |
| `DEBT-AGENT-PROMPT-001.md` | 改为 `PARTIAL/P2` | 已有 Tool Manifest / Context Window 等增强 |
| `02-slash-command-palette.md` | 保持 `OPEN` | 等 UX 批次处理 |
| `01-token-context-usage-tracking.md` | 改为 `VERIFY/CLEARED` | 本地验收通过后归档 |
| `DEBT-SCHEME-B.md` | 改为 `VERIFY/CLEARED` | 本地验收通过后归档 |
| `SHELL-FEATURE-DEBT-002.md` | 保持 `OPEN BY DESIGN` | P0 Shell 修复后再谈恢复复杂功能 |
| `DEBT-P0-001.md` | 改为 `ARCHIVE CANDIDATE` | 本地未发现 active signaling server；待模块 owner 确认可归档 |
| `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | 改为 `PARTIAL/P2` | Day 13-14 已完成 security-dom / workspace / sessions / thinking-ui 渐进拆分，后续继续拆 command/slash/provider/style |

---

## 21. 维护规则

每次修复一个债务，必须给本文档或后续状态文档补三样东西：

```text
1. 修改了什么文件
2. 跑了什么验证命令
3. 验证输出 / 截图 / 日志在哪里
```

没有验证 receipt，就不要把状态从 `OPEN` 改成 `CLEARED`。这条很重要，防止技术债文档再次变成玄学许愿池。
