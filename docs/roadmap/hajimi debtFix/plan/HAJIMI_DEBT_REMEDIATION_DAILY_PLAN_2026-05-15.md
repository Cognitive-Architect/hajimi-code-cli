# HAJIMI Debt Remediation Daily Plan — Day 0~15 每日细化

> **文档版本**: 1.0  
> **所属 Roadmap**: `DEBT-REMEDIATION-ROADMAP-2026-05-15.md`  
> **输入基线**: `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`  
> **适用仓库**: `Cognitive-Architect/hajimi-code-cli`  
> **Web 端核验分支**: `v3.8.0-batch-1`  
> **最后更新**: 2026-05-16  
> **计划目标**: 用 16 个工作日以内完成对账、P0 安全修复、P1 功能/UX 验收、Thinking UI 真闭环、Agent Prompt V2 质量增强与最终清债闭环。  
> **工作方式**: 每天结束必须留下 receipt：命令输出、截图、日志、修改文件列表、债务状态变更。

---

## 已确认的当前基线

| 区域 | 当前状态 | 处理策略 |
|---|---|---|
| Shell 安全 | `OPEN / P0` | 第一批修 |
| workspace symlink | `OPEN / P0` | 第一批修 |
| Tauri CSP/global API | `OPEN / P0-P1` | 第二批修 |
| 文件操作错配 | `OPEN / P1` | 第一批顺手修：`create_dir` / `rename_path` / `delete_path` |
| 启动/文件树/历史会话 | `VERIFY / P1` | 第三批本地验收 |
| Thinking UI / Checkpoint | `PARTIAL / P1-P2` | 第四批补真实现 |
| Agent Prompt | `PARTIAL / P2` | 第五批质量增强 |
| Token / Context | `VERIFY/CLEARED / P2` | 回归验收，不抢优先级 |
| 前端架构 | `OPEN / P2` | 第六批渐进拆 |
| Signaling PSK | `ARCHIVE CANDIDATE` | 本地未发现 active signaling server；Day 1 只做复核确认 |

---

## Phase 0: 对账与文档落地（Day 0-1）

> **目标**: 不急着修代码，先让网页端、本地 workspace、债务文档一致。

---

### Day 0: 文档入仓 + 本地 audit

**预计工时**: 1-2 小时  
**风险等级**: 🟢 低  
**代码改动**: 无，文档为主。

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 / 说明 |
|---:|---|---|---|
| 1 | 创建目录 | `docs/roadmap/hajimi debtFix/debt/` + `docs/roadmap/hajimi debtFix/plan/` | 若目录不存在则创建；如需正式入仓注意 `docs/` ignore |
| 2 | 放入当前债务总表 | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 当前真相快照 |
| 3 | 放入 Roadmap | `docs/roadmap/hajimi debtFix/plan/HAJIMI_DEBT_REMEDIATION_ROADMAP_2026-05-15.md` | 批次级路线 |
| 4 | 放入 Daily Plan | `docs/roadmap/hajimi debtFix/plan/HAJIMI_DEBT_REMEDIATION_DAILY_PLAN_2026-05-15.md` | 每日计划 |
| 5 | 运行 local audit | `docs/debt/local-debt-audit-*.txt` | 记录本地真实状态 |
| 6 | 检查 git 状态 | repo root | 判断本地是否有未提交变更 |

#### 一键命令：Windows PowerShell

```powershell
New-Item -ItemType Directory -Force "docs/roadmap/hajimi debtFix/debt" | Out-Null
New-Item -ItemType Directory -Force "docs/roadmap/hajimi debtFix/plan" | Out-Null

$Out = "docs/roadmap/hajimi debtFix/debt/local-debt-audit-$(Get-Date -Format yyyyMMdd-HHmmss).txt"
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
Select-String -Path src/interface/web/app.js -Pattern "createNewFolder|renameFile|deleteFile|mkdir|mv|rm|loadChatSessions|saveChatSessions|hajimi_chat_sessions" | Out-File $Out -Append -Encoding utf8

Write-Host "Wrote $Out"
```

#### 验证命令

```bash
git status --short
ls "docs/roadmap/hajimi debtFix/debt"
ls "docs/roadmap/hajimi debtFix/plan"
```

#### Day 0 验收标准

- [ ] 3 份新文档已放入工作区；若要提交，已处理 `.gitignore` 中 `docs/` ignore（例如 `git add -f`）
- [ ] local audit 文件已生成
- [ ] 知道当前本地分支和 HEAD
- [ ] 知道是否存在未提交变更
- [ ] 未修改功能代码

---

### Day 1: 状态复核 + Signaling PSK 归档候选确认

**预计工时**: 2-3 小时  
**风险等级**: 🟢 低  
**代码改动**: 无，最多改文档状态。

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 / 说明 |
|---:|---|---|---|
| 1 | 复核 Shell 债 | `shell.rs` | 确认 shell 解释器是否仍在白名单 |
| 2 | 复核 workspace 债 | `main.rs` | 确认 `canonicalize().unwrap_or(resolved)` 是否仍在 |
| 3 | 复核 Tauri 债 | `tauri.conf.json` | 确认 CSP/global API |
| 4 | 复核文件操作错配 | `app.js` + `main.rs` | 确认 `mkdir/mv/rm` 错配 |
| 5 | 复核 signaling | repo | 当前为 `ARCHIVE CANDIDATE`；只确认是否有遗漏 active runtime |
| 6 | 更新状态表 | 当前债务总表 | 只更新已复核状态，不乱清债 |

#### 验证命令

```bash
rg -n "WebRTC|signaling|psk|pre-shared|KMS|Vault" src Cargo.toml package.json
rg -n "bash|sh|pwsh|powershell|ALLOWED_COMMANDS" src/engine/tool-system/src/shell.rs
rg -n "validate_path_within_workspace|canonicalize|run_command" src/interface/desktop/src/main.rs
rg -n "withGlobalTauri|csp" src/interface/desktop/tauri.conf.json
rg -n "createNewFolder|renameFile|deleteFile|cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js
```

#### Day 1 验收标准

- [ ] `DEBT-P0-001` 保持 `ARCHIVE CANDIDATE` 或被明确重新打开为 `OPEN`
- [ ] P0/P1 当前状态与本地 audit 一致
- [ ] 不因为“文档说已修”而清债
- [ ] 形成 Day 2 修复前确认清单，文件操作错配覆盖 `mkdir/mv/rm`

---

## Phase 1: P0 安全边界 + 文件操作错配（Day 2-4）

> **目标**: 先修本机能力边界，不搞花活。

---

### Day 2: 安全路径解析函数

**预计工时**: 4-6 小时  
**风险等级**: 🔴 高  
**主要目标**: 修 workspace symlink / nonexistent path 逃逸。

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 设计 `PathIntent` | `src/interface/desktop/src/main.rs` 或新模块 | `ExistingFile`, `ExistingDir`, `NewFile`, `NewDir`, `AnyExisting` |
| 2 | 实现 `resolve_workspace_path` | 同上 | 对相对/绝对路径统一解析 |
| 3 | 对 base_dir canonicalize | 同上 | base 必须可解析 |
| 4 | 对 existing path canonicalize | 同上 | existing path 必须真实在 base 内 |
| 5 | 对 new path canonicalize parent | 同上 | 父目录必须存在并在 base 内 |
| 6 | 拒绝 path traversal | 同上 | `..` 和 Windows 变体都要考虑 |
| 7 | 保留用户友好错误 | 同上 | 错误信息说明路径越界，不泄露过多细节 |
| 8 | 替换 `read_file/write_file/list_dir` | 同上 | 先替换核心三个命令 |
| 9 | 添加基础测试 | 同上 | 至少覆盖 normal / traversal / missing parent |

#### 推荐接口草案

```rust
enum PathIntent {
    ExistingFile,
    ExistingDir,
    NewFile,
    NewDir,
    AnyExisting,
}

fn resolve_workspace_path(
    input: &str,
    base_dir: &Path,
    intent: PathIntent,
) -> Result<PathBuf, String> {
    // 1. reject traversal
    // 2. join base if relative
    // 3. canonicalize base
    // 4. for existing: canonicalize target
    // 5. for new: canonicalize parent, then join filename
    // 6. starts_with canonical_base
}
```

#### 验证命令

```bash
cargo check -p hajimi-desktop
rg -n "resolve_workspace_path|PathIntent" src/interface/desktop/src/main.rs
```

#### Day 2 验收标准

- [ ] `cargo check -p hajimi-desktop` 0 errors
- [ ] `read_file/write_file/list_dir` 已走新 resolver
- [ ] 目标不存在时不再直接 fallback 到 unresolved path
- [ ] traversal 被拒绝
- [ ] 父目录不存在时错误明确

---

### Day 3: 文件操作专用命令 + 前端接入

**预计工时**: 3-5 小时  
**风险等级**: 🟡 中  
**主要目标**: 修新建文件夹 / 重命名 / 删除功能错配，不扩大命令白名单。

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 新增 `create_dir` command | `src/interface/desktop/src/main.rs` | `fn create_dir(path: &str, app_handle: tauri::AppHandle) -> Result<(), String>` |
| 2 | 新增 `rename_path` command | `main.rs` | old/new path 都走 resolver；目标使用 new intent |
| 3 | 新增 `delete_path` command | `main.rs` | 删除走 existing intent；递归删除必须由前端确认后传入 |
| 4 | 走安全 resolver | `main.rs` | `create_dir` 用 `NewDir`，`rename_path` 用 existing + new parent，`delete_path` 用 existing |
| 5 | 注册 commands | `generate_handler![]` | 加入 `create_dir` / `rename_path` / `delete_path` |
| 6 | 修改前端文件操作 | `src/interface/web/app.js` | `createNewFolder()` / `renameFile()` / `deleteFile()` 改用专用 command |
| 7 | 不改 `run_command` 白名单 | `main.rs` | 不加入 `mkdir/mv/rm` |
| 8 | 手动文件操作测试 | Tauri dev | 新建、重命名、删除成功后文件树刷新 |
| 9 | 写验收记录 | `docs/debt/UX-FILETREE-SESSION-VERIFY.md` 或当天 receipt | 记录截图/日志 |

#### 验证命令

```bash
cargo check -p hajimi-desktop
node --check src/interface/web/app.js
rg -n "create_dir|rename_path|delete_path" src/interface/desktop/src/main.rs
rg -n "createNewFolder|renameFile|deleteFile|create_dir|rename_path|delete_path|cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js
```

#### Day 3 验收标准

- [ ] `create_dir` / `rename_path` / `delete_path` 已注册为 Tauri commands
- [ ] `createNewFolder()` 不再调用 `run_command('mkdir')`
- [ ] `renameFile()` 不再调用 `run_command('mv')`
- [ ] `deleteFile()` 不再调用 `run_command('rm')`
- [ ] 后端通用命令白名单仍不包含 `mkdir/mv/rm`
- [ ] 新建文件夹本地手动成功
- [ ] 重命名、删除本地手动成功且有明确确认
- [ ] 越界路径被拒绝

---

### Day 4: ShellTool 白名单收紧

**预计工时**: 4-6 小时  
**风险等级**: 🔴 高  
**主要目标**: 禁止用户通过 ShellTool 显式调用 shell 解释器。

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 移除用户白名单项 | `src/engine/tool-system/src/shell.rs` | 删除 `bash`, `sh`, `pwsh`, `powershell` |
| 2 | 修正测试 | `shell.rs` | 将 `powershell/pwsh is_ok()` 改为 `is_err()` |
| 3 | 新增 bash/sh 拒绝测试 | `shell.rs` | `bash script.sh`, `sh script.sh` |
| 4 | 保留外层执行器 | `shell.rs` | 实现仍可用 bash/powershell wrapper，但用户命令不能是 shell |
| 5 | 检查 metachar 拦截仍有效 | `shell.rs` | 保留 `; & | $` 等测试 |
| 6 | 更新 Shell 债状态 | 当前债务总表 | `OPEN -> VERIFY`，等待完整验收 |

#### 验证命令

```bash
cargo test -p engine-tool-system -- test_allow_list
cargo test -p engine-tool-system
cargo check --workspace
rg -n '"bash"|"sh"|"pwsh"|"powershell"' src/engine/tool-system/src/shell.rs
```

#### Day 4 验收标准

- [ ] `bash/sh/pwsh/powershell` 不再作为用户命令通过 allow-list
- [ ] `git status` / `cargo check` / `ls -la` 仍通过
- [ ] metachar 测试仍通过
- [ ] `cargo test -p engine-tool-system` 通过
- [ ] 若出现大量非本次相关 workspace 错误，输出清单，不扩修

---

## Phase 2: Tauri 安全面（Day 5-6）

> **目标**: 降低 XSS 后果，先做安全渲染和 CSP，再考虑关闭 global API。

---

### Day 5: DOM 渲染审计 + 高风险点修复

**预计工时**: 5-7 小时  
**风险等级**: 🔴 高

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 扫描 `innerHTML` | `src/interface/web/app.js` | 列出所有位置 |
| 2 | 创建 DOM audit 文档 | `docs/debt/SECURITY-DOM-AUDIT.md` | 标记 source: user/file/git/model/static |
| 3 | 统一 escape helper | `app.js` 或 `modules/security-dom.js` | 先最小引入 |
| 4 | 修文件树文件名渲染 | `app.js` | 用户/文件系统输入必须 escape |
| 5 | 修聊天内容渲染策略 | `app.js` | 明确 markdown 渲染边界 |
| 6 | 修 Git/工具输出渲染 | `app.js` | 输出默认 code block escaped |
| 7 | 恶意样例本地测试 | 手动 | 文件名/聊天/Git 输出 |

#### 验证命令

```bash
rg -n "innerHTML" src/interface/web/app.js > "docs/roadmap/hajimi debtFix/debt/innerhtml-audit.txt"
node --check src/interface/web/app.js
```

#### Day 5 验收标准

- [ ] `SECURITY-DOM-AUDIT.md` 存在
- [ ] 高风险 `innerHTML` 已修或已标注 TODO
- [ ] 恶意文件名不执行 JS
- [ ] 恶意聊天内容不执行 JS
- [ ] `node --check` 通过

---

### Day 6: CSP baseline + global Tauri API 迁移计划

**预计工时**: 4-6 小时  
**风险等级**: 🔴 高

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 配置基础 CSP | `src/interface/desktop/tauri.conf.json` | 从 `csp: null` 改为基础策略 |
| 2 | Tauri dev 启动 | 本地 | 看是否有 CSP 报错 |
| 3 | 收集 CSP 报错 | `docs/debt/SECURITY-CSP-VERIFY.md` | 记录 console / terminal |
| 4 | 判断 `withGlobalTauri` | `tauri.conf.json` + `app.js` | 如无法立即关闭，写迁移计划 |
| 5 | 建立迁移计划 | `SECURITY-CSP-VERIFY.md` | 从 `window.__TAURI__` 到模块 API |
| 6 | 更新债务表 | 当前债务总表 | Tauri CSP `OPEN -> VERIFY/PARTIAL` |

#### 验证命令

```bash
cargo check -p hajimi-desktop
node --check src/interface/web/app.js
rg -n "withGlobalTauri|csp" src/interface/desktop/tauri.conf.json
```

#### Day 6 验收标准

- [ ] CSP 不再是 `null`，或有明确阻塞原因与迁移计划
- [ ] 应用能启动
- [ ] 核心功能不被 CSP 破坏
- [ ] `withGlobalTauri` 若未关闭，必须有具体后续任务
- [ ] 记录 receipt

---

## Phase 3: UX 实机验收（Day 7）

> **目标**: 把 `VERIFY` 类 UX 债务变成有证据的状态。

---

### Day 7: 启动 / 文件树 / 会话持久化验收

**预计工时**: 3-5 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 说明 |
|---:|---|---|---|
| 1 | Tauri dev 启动 | app | 观察启动 toast |
| 2 | 文件树验收 | workspace | 文件树显示 `hajimi-workspace` |
| 3 | 文件操作验收 | app | Day 3 新建 / 重命名 / 删除复验 |
| 4 | 会话 A/B 切换 | app | 确认消息不丢 |
| 5 | 关闭重开 | app | 历史会话仍存在 |
| 6 | 写验收文档 | `docs/debt/UX-FILETREE-SESSION-VERIFY.md` | 命令/截图/结果 |
| 7 | 更新状态 | 当前债务总表 | 成功后 `VERIFY -> CLEARED` 或继续 `OPEN` |

#### 验证命令

```bash
cargo check -p hajimi-desktop
node --check src/interface/web/app.js
cd src/interface/desktop && cargo tauri dev
```

#### Day 7 验收标准

- [ ] 启动无异常 toast
- [ ] 文件树加载成功
- [ ] 新建文件夹成功
- [ ] 重命名、删除成功且删除前有明确确认
- [ ] 会话切换不丢消息
- [ ] 重启后历史会话仍存在
- [ ] receipt 文档存在

---

## Phase 4: Thinking UI & Checkpoint 真闭环（Day 8-10）

> **目标**: 把占位函数和虚拟 diff 变成真实功能。

---

### Day 8: Trace 链路验收 + Checkpoint 数据模型

**预计工时**: 5-7 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 验证 trace_tx 注入 | `main.rs` | 确认 AgentLoop events 可订阅 |
| 2 | 前端 trace 接收测试 | `app.js` | 确认实时卡片来自后端 |
| 3 | 设计 Checkpoint DTO | `main.rs` 或新模块 | id/timestamp/files/diff/metadata |
| 4 | 明确存储位置 | workspace / app data | 不乱写项目根 |
| 5 | 写设计注释 | `docs/debt/THINKING-CHECKPOINT-PLAN.md` | 记录最小模型 |
| 6 | 不急着 restore | 文档 | restore 属高风险，Day 10 再做 |

#### 验证命令

```bash
rg -n "subscribe_agent_trace|trace_tx|agent:trace" src/interface/desktop/src/main.rs
node --check src/interface/web/app.js
cargo check -p hajimi-desktop
```

#### Day 8 验收标准

- [ ] Trace 事件来源明确
- [ ] Checkpoint DTO 草案落地
- [ ] 存储位置明确
- [ ] 没有实现不安全 restore

---

### Day 9: export / compare checkpoint

**预计工时**: 5-7 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 实现 `export_checkpoint` | `main.rs` | 返回真实 JSON/Markdown，不再 `{}` |
| 2 | 实现 `compare_checkpoints` | `main.rs` | 返回 diff summary / JSON |
| 3 | 前端展示 compare 结果 | `app.js` | 最小 UI，不追求漂亮 |
| 4 | Operation Summary 接真实 diff | `app.js` | 先支持当前文件 / 当前 hunk |
| 5 | 错误处理 | both | 找不到 checkpoint 要明确报错 |
| 6 | 写 receipt | `docs/debt/THINKING-CHECKPOINT-VERIFY.md` | 记录示例输出 |

#### 验证命令

```bash
cargo check -p hajimi-desktop
node --check src/interface/web/app.js
rg -n "export_checkpoint|compare_checkpoints" src/interface/desktop/src/main.rs
```

#### Day 9 验收标准

- [ ] `export_checkpoint` 不再返回 `{}` 占位
- [ ] `compare_checkpoints` 不再返回固定 `false`
- [ ] 前端能展示最小 diff/summary
- [ ] 错误路径可读

---

### Day 10: restore checkpoint + Replay 安全闭环

**预计工时**: 5-8 小时  
**风险等级**: 🔴 高

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | restore 加确认 | `main.rs` / `app.js` | 必须用户确认 |
| 2 | restore 前导出 backup | `main.rs` | 防止误覆盖 |
| 3 | restore 使用安全路径 resolver | `main.rs` | 不允许越界恢复 |
| 4 | Replay 绑定 checkpoint | `app.js` | 从真实事件回放 |
| 5 | 失败回滚测试 | 手动 | restore 中断不破坏工作区 |
| 6 | 更新状态文档 | 当前债务总表 | `PARTIAL -> VERIFY` |

#### 验证命令

```bash
cargo check -p hajimi-desktop
node --check src/interface/web/app.js
```

#### Day 10 验收标准

- [ ] restore 前有用户确认
- [ ] restore 前有备份
- [ ] restore 走 workspace 安全路径
- [ ] Replay 使用真实 checkpoint / trace 数据
- [ ] 失败不破坏 workspace

---

## Phase 5: Agent Prompt V2（Day 11-12）

> **目标**: 在安全和 UX 稳住后做质量增强，不抢救火优先级。

---

### Day 11: Prompt Contract 文档化

**预计工时**: 4-6 小时  
**风险等级**: 🟢 低

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 写 Persona 文档 | `docs/agent-prompt-core/AGENT-PERSONA.md` | 角色、安全、边界 |
| 2 | 写 Planner Contract | `PLANNER-PROMPT-CONTRACT.md` | input/output/fallback |
| 3 | 写 Reflector Contract | `REFLECTOR-CONTRACT.md` | root cause / stop-loss |
| 4 | 写 Executor Contract | `EXECUTOR-CONTRACT.md` | toolcall / governance |
| 5 | 写 Tool Manifest Schema | `TOOL-MANIFEST-SCHEMA.md` | 与代码 DTO 对齐 |
| 6 | 更新债务状态 | 当前债务总表 | Agent Prompt 仍保持 `PARTIAL` |

#### 验证命令

```bash
ls docs/agent-prompt-core
rg -n "Planner|Reflector|Tool Manifest|Stop-Loss" docs/agent-prompt-core
```

#### Day 11 验收标准

- [ ] 5 份 Prompt Contract 文档存在
- [ ] 不修改核心代码
- [ ] 文档没有宣称已实现未实现功能
- [ ] 明确 fallback / feature-gate

---

### Day 12: Golden Task Regression

**预计工时**: 5-7 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 建立 golden task 目录 | `tests/agent_prompt_golden/` | 或 docs 测试集 |
| 2 | 定义 5 个 Planner 样例 | JSON/Markdown | 修 bug / 搜索 / 读文件 / 写文件 / ask user |
| 3 | 定义 5 个 Reflector 样例 | JSON/Markdown | success/fail/unknown/stop-loss |
| 4 | 定义 3 个 ToolCall 样例 | JSON/Markdown | safe read / risky write / cannot act |
| 5 | 如已有测试框架则接入 | Rust tests | 没有就先文档化 |
| 6 | 跑 agent-core 测试 | cargo | 不破坏现有代码 |

#### 验证命令

```bash
cargo test -p intelligence-agent-core --lib
find tests/agent_prompt_golden -type f
```

#### Day 12 验收标准

- [ ] Golden cases 存在
- [ ] 覆盖 success/failure/unknown/stop-loss
- [ ] 不强行依赖真实 LLM
- [ ] `cargo test -p intelligence-agent-core --lib` 通过或非本次错误已记录

---

## Phase 6: 前端架构渐进拆分（Day 13-14）

> **目标**: 只拆高风险/高频模块，不重写 IDE。

---

### Day 13: security-dom + workspace 模块

**预计工时**: 5-7 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 新建 `security-dom.js` | `src/interface/web/modules/security-dom.js` | escape / safeText / safeHtml policy |
| 2 | 新建 `workspace.js` | `src/interface/web/modules/workspace.js` | initWorkspace / loadFileTree / create folder |
| 3 | app.js 最小接入 | `app.js` | 不大搬迁 |
| 4 | 保持全局兼容 | app.js | 防止旧调用断 |
| 5 | 检查语法 | JS | 所有模块 node check |
| 6 | 本地 UI smoke test | app | 文件树、新建 / 重命名 / 删除 |

#### 验证命令

```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/security-dom.js
node --check src/interface/web/modules/workspace.js
```

#### Day 13 验收标准

- [ ] 两个模块存在
- [ ] app.js 可运行
- [ ] 文件树功能不回归
- [ ] 新建 / 重命名 / 删除功能不回归

---

### Day 14: sessions + thinking-ui 模块

**预计工时**: 5-8 小时  
**风险等级**: 🟡 中

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|---|---|
| 1 | 新建 `sessions.js` | `modules/sessions.js` | load/save/switch/render session |
| 2 | 新建 `thinking-ui.js` | `modules/thinking-ui.js` | thinking block / parse stream / trace cards |
| 3 | app.js 最小接入 | `app.js` | 保留旧 API 包装 |
| 4 | 本地 smoke test | app | 会话和 thinking 不回归 |
| 5 | 更新架构文档 | `src/ARCHITECTURE.md` 或 docs | 记录前端模块边界 |
| 6 | 更新债务状态 | 当前债务总表 | 前端架构 `OPEN -> PARTIAL` |

#### 验证命令

```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/sessions.js
node --check src/interface/web/modules/thinking-ui.js
cargo check -p hajimi-desktop
```

#### Day 14 验收标准

- [ ] 会话持久化不回归
- [ ] thinking stream 不回归
- [ ] trace UI 不回归
- [ ] 前端架构债可标记为 `PARTIAL`

---

## Phase 7: Closure（Day 15）

> **目标**: 让代码、文档、验证记录三方一致。

---

### Day 15: 清债验证、文档闭环 & Final Commit

**预计工时**: 4-6 小时  
**风险等级**: 🟢 低

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 / 说明 |
|---:|---|---|---|
| 1 | 创建清债总结 | `docs/debt/DEBT-REMEDIATION-CLOSURE-2026-05-xx.md` | 记录每个 batch 的结果 |
| 2 | 更新债务总表 | `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 状态迁移必须有 receipt |
| 3 | 更新 INDEX | `docs/debt/INDEX.md` 或 `src/INDEX.md` | Active / Cleared / Archive |
| 4 | 更新架构文档 | `src/ARCHITECTURE.md` | 安全路径、CSP、checkpoint、前端模块 |
| 5 | 运行完整测试 | workspace | 全部命令 |
| 6 | 检查架构纯洁性 | grep | Engine 不依赖 Interface |
| 7 | 最终 git status | repo | 确认干净或输出待提交清单 |
| 8 | 最终 commit | git | 单独 closure commit |

#### 最终验证命令

```bash
cargo check --workspace
cargo test -p engine-tool-system
cargo test -p intelligence-agent-core --lib
node --check src/interface/web/app.js
Get-ChildItem -LiteralPath src/interface/web/modules -Filter *.js | ForEach-Object { node --check $_.FullName }
rg -n "use interface|use intelligence.*interface" src/engine src/intelligence
git status --short
```

#### Day 15 验收标准

- [ ] `cargo check --workspace` 0 errors，或非本次错误有完整记录
- [ ] `cargo test -p engine-tool-system` 通过
- [ ] `cargo test -p intelligence-agent-core --lib` 通过
- [ ] JS 语法检查通过
- [ ] P0 债务有 receipt 后才标记 `CLEARED`
- [ ] P1 UX 有截图/日志后才标记 `CLEARED`
- [ ] Closure 文档存在
- [ ] git status 清晰

---

## Feature-Gate / 回滚开关建议

| 开关 | 控制范围 | 默认值 | 回滚行为 |
|---|---|---:|---|
| `HAJIMI_SECURE_WORKSPACE_RESOLVER` | 新路径解析器 | `true` | 仅允许本地诊断时临时对比旧 resolver；release / 清债验收不得回到旧 resolver |
| `HAJIMI_TAURI_CSP_ENABLED` | CSP 策略 | `true` | 临时恢复旧 CSP，必须记录原因 |
| `HAJIMI_CHECKPOINT_V1_ENABLED` | checkpoint 真实现 | `true` | 回到只读 timeline，不允许 restore |
| `HAJIMI_AGENT_PROMPT_V2_ENABLED` | Agent Prompt V2 | `false` 初期 | 回到当前 Planner/Reflector 行为 |
| `HAJIMI_FRONTEND_MODULES_ENABLED` | 前端模块拆分 | `true` | 回到 app.js 包装函数路径 |

说明：如果代码里暂时不做 feature-gate，也要在文档里保留“人工回滚方式”。安全修复不建议长期允许回滚到不安全状态。

---

## 工作量统计

| Phase | 天数 | 主要目标 | 预计总工时 |
|---|---:|---|---:|
| Phase 0 | Day 0-1 | 对账与文档落地 | 3-5h |
| Phase 1 | Day 2-4 | P0 安全 + create_dir | 11-17h |
| Phase 2 | Day 5-6 | Tauri CSP / DOM audit | 9-13h |
| Phase 3 | Day 7 | UX 实机验收 | 3-5h |
| Phase 4 | Day 8-10 | Thinking UI / Checkpoint | 15-22h |
| Phase 5 | Day 11-12 | Agent Prompt V2 | 9-13h |
| Phase 6 | Day 13-14 | 前端渐进拆分 | 10-15h |
| Phase 7 | Day 15 | Closure | 4-6h |
| **总计** | **16 天** | **安全优先清债闭环** | **64-96h** |

---

## 每日 Receipt 模板

每一天完成后，在对应文档或 commit message 里贴：

```text
=== DAILY RECEIPT ===
日期：
Day：
目标：
修改文件：
验证命令：
验证结果：
截图/日志路径：
债务状态变化：
未解决问题：
下一步：
=====================
```

---

## 最终清债规则

状态迁移必须遵守：

```text
OPEN -> VERIFY -> CLEARED
PARTIAL -> VERIFY -> CLEARED
UNKNOWN -> OPEN / ARCHIVE
OPEN BY DESIGN -> OPEN / VERIFY / CLEARED
```

不能出现：

```text
OPEN -> CLEARED
UNKNOWN -> CLEARED
```

没有 receipt，就不能清债。  
这条是防止文档再次变成许愿池的防火墙。许愿池可以有，但别拿它当 CI。

---

*Generated on 2026-05-15. Revised on 2026-05-16. This Daily Plan follows the structure of AGENT-PROMPT-CORE-001 execution planning style and maps directly to HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md plus local source/test review.*
