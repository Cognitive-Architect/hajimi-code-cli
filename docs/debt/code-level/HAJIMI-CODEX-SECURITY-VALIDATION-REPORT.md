# HAJIMI Codex Security 风格验证报告

**仓库**: `Cognitive-Architect/hajimi-code-cli`  
**审查基线**: `8ac028b2b3f79f10817523de197760b9f774ed36`  
**最新提交标题**: `fix(frontend): close thinking ui release path`  
**报告日期**: 2026-05-19  
**审查方式**: Codex Security 风格防御审查：Security Scan / Threat Model / Finding Discovery / Attack Path Analysis / Validation / Fix Plan  

---

## 0. 执行结论

当前仓库已经有一批不错的安全基础设施：Tauri CSP 不为空、shell 工具有白名单与元字符过滤、workspace resolver 已经能防路径穿越、Provider API Key 迁移到 OS keyring、Slash Palette 用安全 DOM 渲染、Security Audit Gate 已落地。

但还有几条关键边界没有闭合，尤其是：

1. `execute_tool` 可以直接执行 ToolRegistry 里的工具，但没有看到统一执行 `ToolPermissions` / `requires_confirmation` 的门禁。
2. 通用 `read_file` / `write_file` / `delete_file` / `edit_file` 工具自身没有强制 workspace sandbox，默认 `allowed_paths: None` 时可能被直接用于绝对路径。
3. 前端仍开启 `withGlobalTauri: true`，一旦未来有 XSS，Tauri invoke 面会被放大。
4. `apply_edits` / `preview_edit` 等部分 Tauri 命令绕过 workspace resolver。
5. legacy `run_command` 白名单仍包含 `npx/pnpm/pip/code/cursor` 这类高能力命令。

**总体状态**: `PARTIAL/GATED`  
**建议发布门槛**: 不建议把当前版本暴露给不可信插件、远程内容、多人共享工作区或默认联网自动化场景。若只是本地个人开发使用，可继续迭代，但需要优先修复 High 项。

人话版：现在不是“裸奔”，已经穿了护甲；但有几扇后门的门锁还没上。尤其是工具执行这块，像厨房刀具柜贴了“需要批准”，但前台按钮目前能直接把柜门打开。

---

## 1. 审查范围

### 1.1 覆盖范围

| 区域 | 覆盖内容 |
|---|---|
| Tauri 配置 | CSP、global Tauri API、build 前资源同步 |
| 桌面后端 | workspace resolver、run_command、execute_tool、provider keyring、checkpoint restore、apply_edits |
| Tool System | ShellTool、Read/Write/Delete/Edit 工具、ToolRegistry、ToolPermissions |
| Web 前端 | slash palette、chat command、Markdown 渲染、innerHTML 使用、Tauri invoke 入口 |
| 安全门禁 | `tests/security/security_audit_gate.js` 与 allowlist |

### 1.2 未覆盖范围

| 未覆盖项 | 原因 |
|---|---|
| 动态 exploit / PoC | 本报告只做防御审查，不执行破坏性验证 |
| 完整依赖 CVE 扫描 | 未运行 `npm audit` / `cargo audit` / OSV |
| 全量 crate 逐行审计 | 本轮聚焦高风险入口与安全边界 |
| WebView 实机点击验收 | 未运行本地 Tauri GUI |
| CI 实跑结果 | 未 clone 与执行测试，仅做代码证据审查 |

---

## 2. Threat Model / 威胁模型

### 2.1 保护资产

| 资产 | 风险 |
|---|---|
| 用户本地文件 | 被读取、修改、删除 |
| Provider API Key | 被泄露、导出、错误发送到非预期服务 |
| Git 工作区 | 被自动提交、改写、误删 |
| Tauri invoke 面 | 被 XSS 或恶意前端脚本调用 |
| Agent 工具系统 | 被越权调用高风险工具 |
| Checkpoint / restore | 被滥用为文件写入/删除入口 |

### 2.2 攻击入口

| 入口 | 风险等级 | 说明 |
|---|---:|---|
| `/tool <name> <json>` | 高 | 可直接触发 `execute_tool` |
| `window.__TAURI__` | 高 | `withGlobalTauri: true` 时前端全局可见 |
| `innerHTML` 渲染 | 中 | 多数已 escape，但历史债仍多 |
| Provider config 导入/导出 | 中 | 涉及 API Key 与任意 file_path 输入 |
| `run_command` | 中 | 有白名单，但命令能力较大 |
| MCP invoke/init | 中 | 取决于 MCP 服务可信度 |
| Checkpoint restore | 中 | 已有 dry-run/confirm/backup，但仍是写文件入口 |

---

## 3. Positive Controls / 已验证的安全正项

| 控制项 | 状态 | 说明 |
|---|---|---|
| CSP 不为 null | PASS | Tauri CSP 存在，`script-src 'self'` 有基础防护 |
| Slash Palette 安全 DOM | PASS | 使用 `createElement` / `textContent` / `appendChild`，没有 `innerHTML` |
| ShellTool 白名单 | PASS | 工具层 shell 阻止复杂 shell 与元字符 |
| Dedicated workspace file commands | PASS | `read_file/write_file/list_dir/create_dir/rename/delete` 使用 `resolve_workspace_path` |
| Provider keyring | PASS | API key 跳过 JSON 序列化，迁移到 OS keyring |
| Provider config 文件权限 | PASS | Unix 下写 `0600`，父目录 `0700` |
| Security Audit Gate | PASS/PARTIAL | 已有 `test:security-gate`，但覆盖仍偏文本扫描 |
| URL sanitizer | PASS | Markdown link 只允许 `http/https/mailto` |

---

## 4. Findings Summary

| ID | 严重性 | 标题 | 状态 | 优先级 |
|---|---:|---|---|---:|
| F-001 | High | `execute_tool` 未统一执行 ToolPermissions / confirmation | Confirmed | P0 |
| F-002 | High | 通用文件工具可绕过 workspace sandbox | Confirmed | P0 |
| F-003 | Medium-High | `withGlobalTauri: true` 放大 XSS 后果 | Confirmed / Existing Debt | P1 |
| F-004 | Medium | legacy `run_command` 白名单包含高能力命令 | Confirmed | P1 |
| F-005 | Medium | `apply_edits` / `preview_edit` 绕过 workspace resolver | Confirmed | P1 |
| F-006 | Low-Medium | workspace provider config path 信任前端传参 | Confirmed | P2 |

---

## 5. Finding Details

### F-001: `execute_tool` 未统一执行 ToolPermissions / confirmation

**Severity**: High  
**Status**: Confirmed  
**Component**: `src/interface/desktop/src/main.rs`, `src/engine/tool-system`  

#### 证据摘要

- 前端 `/tool <name> <json_args>` 会直接调用 `invoke('execute_tool', { name: toolName, args })`。
- 后端 `execute_tool` 只从 registry 取工具并直接 `tool.execute(args).await`。
- `ToolPermissions` 与 `Tool::permissions()` 已存在，但没有看到 `execute_tool` 在执行前检查 `default_level` / `requires_confirmation`。
- `ToolRegistry` 只提供 `register/get/list`，本身不做权限执行。

#### 攻击路径分析

1. 攻击者通过前端 slash command、恶意 prompt 诱导用户、或未来 XSS 调用 `/tool`。
2. `/tool` 进入 `execute_tool`。
3. `execute_tool` 直接执行目标工具。
4. 如果目标工具是 `write_file`、`delete_file`、`edit_file`、`web_search`、`fetch_url` 等高能力工具，权限元信息不会自动拦截。

#### 影响

- 高风险工具可能被无确认执行。
- 与 F-002 叠加后，可能造成本地文件读写删除。
- 与 F-003 叠加后，未来 XSS 可直接调用 Tauri 工具面。

#### 修复建议

1. 在 `execute_tool` 中统一执行权限门禁：
   - `Deny` 默认拒绝；
   - `Ask` / `requires_confirmation=true` 要求前端提供一次性 confirmation token；
   - `Allow` 仅允许低风险只读工具。
2. 为工具增加 risk level，至少分为 `read-only / write / destructive / network / credential`。
3. `/tool` 默认只允许低风险工具，高风险工具必须二次确认。
4. 在 ToolRegistry 外包一层 `ToolExecutionGate`，不要让 UI 直接拿 registry 执行。

#### 建议验证

```bash
rg -n "async fn execute_tool|permissions\(|requires_confirmation|default_level" src/interface/desktop/src/main.rs src/engine/tool-system/src
cargo test -p interface-desktop -- execute_tool_permission
```

---

### F-002: 通用文件工具可绕过 workspace sandbox

**Severity**: High  
**Status**: Confirmed  
**Component**: `src/engine/tool-system/src/fs.rs`, `edit.rs`, `main.rs`  

#### 证据摘要

- Dedicated Tauri file commands 使用 `resolve_workspace_path`，会 canonicalize 并检查目标是否在 workspace 内。
- 通用 `ReadFileTool` / `WriteFileTool` / `DeleteFileTool` 默认 `allowed_paths: None`，只拒绝 `..` 或根目录删除。
- `EditFileTool` 直接用传入 path 读写，没有 workspace resolver。
- 这些工具均注册在 desktop registry，并可由 `execute_tool` 调用。

#### 攻击路径分析

1. 调用 `execute_tool` 选择 `read_file` / `write_file` / `delete_file` / `edit_file`。
2. 传入绝对路径或非 workspace 路径。
3. 工具自身没有强制 workspace root。
4. 在应用用户权限范围内读写删除文件。

#### 影响

- 可能读写删除 workspace 外文件。
- 破坏 dedicated command 已建立的 workspace sandbox。
- 与 `/tool` 和 `withGlobalTauri` 组合后风险更高。

#### 修复建议

1. 不要在 desktop registry 中暴露 unrestricted `ReadFileTool/WriteFileTool/DeleteFileTool/EditFileTool`。
2. 改为注册 workspace-bound 版本，构造时注入 canonical workspace root。
3. 修改 `validate_path`：当 UI/desktop 模式下 `allowed_paths` 为 `None` 时应拒绝，而不是默认为无限制。
4. `EditFileTool` 增加 canonical workspace root 检查。
5. 对 absolute path 添加负向测试。

#### 建议验证

```bash
cargo test -p engine-tool-system -- fs_workspace_escape
cargo test -p interface-desktop -- execute_tool_rejects_absolute_paths
```

---

### F-003: `withGlobalTauri: true` 放大 XSS 后果

**Severity**: Medium-High  
**Status**: Confirmed / Existing Debt  
**Component**: Tauri config + Web frontend  

#### 证据摘要

- `withGlobalTauri` 当前为 `true`。
- CSP 已存在，不是空 CSP。
- Security Gate 将 `withGlobalTauri` 作为 warning-only。
- 前端仍有多处 legacy `innerHTML`，虽然大量使用 `escapeHtml`、`formatText`、`renderMarkdown` 和 URL sanitizer，但 allowlist 明确承认它们是历史债。

#### 攻击路径分析

1. 如果未来某个渲染点漏 escape，产生 DOM XSS。
2. 因为 `withGlobalTauri: true`，攻击脚本可访问 `window.__TAURI__`。
3. 攻击脚本可调用 `execute_tool`、`run_command`、provider backup/import/export、checkpoint restore 等高能力命令。
4. 与 F-001/F-002 叠加会升级为本地文件/命令面风险。

#### 影响

- 单个前端 XSS 的影响不再只是 UI 污染，而是可能触达本地能力。

#### 修复建议

1. 将 `withGlobalTauri` 迁移为关闭状态，改用显式 scoped import/API bridge。
2. 将 Security Gate 中 `withGlobalTauri` 从 warning 升级为 P1 gate，至少在 release build fail。
3. 新增规则：新增 `innerHTML` 默认为 fail，只有 `security-dom` 和 vetted renderer 可豁免。
4. 为 Tauri commands 加 capability allowlist，不让任意前端调用全部命令。

---

### F-004: legacy `run_command` 白名单包含高能力命令

**Severity**: Medium  
**Status**: Confirmed  
**Component**: `src/interface/desktop/src/main.rs`  

#### 证据摘要

- legacy `run_command` 使用 `Command::new(cmd).args(args)`，避免 shell 拼接，这是好事。
- 但 allow-list 包含 `npx`、`pnpm`、`pip`、`pip3`、`code`、`cursor`。
- `run_command` 已暴露到 Tauri invoke handler。

#### 攻击路径分析

1. 前端或未来 XSS 直接 invoke `run_command`。
2. 选择 `npx` / `pip` / `pnpm` 等包管理/执行命令。
3. 即使没有 shell injection，也可能触发网络下载、包执行、环境污染或打开外部程序。

#### 影响

- 不一定是直接漏洞，但攻击面过宽。
- 对本地 IDE 来说，包管理器与外部编辑器属于高能力动作，不应无确认。

#### 修复建议

1. 将 `npx/pnpm/pip/pip3/code/cursor` 从默认 allow-list 移出。
2. 如果保留，必须加 confirmation + timeout + cwd workspace + network policy。
3. 对 `run_command` 只保留 build/test/status 类命令。
4. 最终尽量废弃 legacy `run_command`，改走 ToolRegistry + governance gate。

---

### F-005: `apply_edits` / `preview_edit` 绕过 workspace resolver

**Severity**: Medium  
**Status**: Confirmed  
**Component**: `src/interface/desktop/src/main.rs`  

#### 证据摘要

- `apply_edits` 直接调用 registry 中的 `edit_file` 工具，把前端 path 原样传入。
- `preview_edit` 直接 `std::fs::read_to_string(&path)`。
- 两者均在 Tauri invoke handler 暴露。

#### 攻击路径分析

1. 前端调用 `apply_edits` 或 `preview_edit`。
2. 传入 workspace 外路径。
3. `preview_edit` 读文件，`apply_edits` 编辑文件。
4. 未经过 `resolve_workspace_path` 的 canonical workspace 检查。

#### 影响

- 文件读/写边界不一致。
- 破坏 dedicated file commands 的安全模型。

#### 修复建议

1. `apply_edits` 与 `preview_edit` 调用前先 `resolve_workspace_path(path, base_dir, ExistingFile)`。
2. `preview_edit` 应禁止空 `old_string`，限制最大读取大小。
3. `apply_edits` 应要求 dry-run preview + confirmation token 再写入。
4. 添加路径逃逸测试。

---

### F-006: workspace provider config path 信任前端传参

**Severity**: Low-Medium  
**Status**: Confirmed  
**Component**: Provider config  

#### 证据摘要

- workspace provider config 使用 `workspace_path` 拼接 `.hajimi/providers.json`。
- `add_provider_config` / `update_provider_config` / `delete_provider_config` 接收前端传入的 `workspace_path`。
- API key 本身会清空并进 keyring，这是好设计。

#### 攻击路径分析

1. 前端传入任意 writable directory 作为 workspacePath。
2. 后端将配置写入该路径下 `.hajimi/providers.json`。
3. 虽然不是任意文件路径写入，但会破坏“workspace 由后端决定”的边界。

#### 影响

- 低到中等风险，取决于 app 是否允许用户切换任意工作区。
- 主要是边界不一致和后续维护风险。

#### 修复建议

1. 后端维护 canonical current workspace，不接受前端 workspacePath 作为信任根。
2. 如果必须接收，先 canonicalize 并确认等于当前 workspace 或在 workspace allow-list 内。
3. 写 provider config 前用统一 workspace resolver。

---

## 6. Validation Matrix

| 验证点 | 结果 | 证据 |
|---|---|---|
| CSP 不为空 | PASS | Tauri config 有 CSP 字符串 |
| `withGlobalTauri` 关闭 | FAIL/WARN | 当前仍为 true |
| ShellTool 拒绝复杂 shell | PASS | shell.rs 测试覆盖 bash/sh/pwsh/powershell 拒绝 |
| ShellTool 拒绝元字符 | PASS | FORBIDDEN_METACHARS + tests |
| Dedicated workspace file commands | PASS | resolve_workspace_path 使用 canonical workspace 检查 |
| Generic file tools workspace-bound | FAIL | fs/edit 工具默认不强制 workspace root |
| Tool permission enforcement | FAIL | execute_tool 未见权限/确认检查 |
| API key JSON 明文存储 | PASS | skip_serializing + keyring migration |
| Slash Palette DOM XSS | PASS | 使用安全 DOM API |
| Legacy innerHTML 全清理 | FAIL/PARTIAL | allowlist 仍保留多处 legacy innerHTML |
| Security Gate 存在 | PASS/PARTIAL | test:security-gate 存在，覆盖仍有限 |

---

## 7. Recommended Fix Order

### P0: 立即修

1. 为 `execute_tool` 加统一权限门禁。
2. 禁止 UI 直接执行 destructive/write/network/credential tools。
3. 把 generic file/edit tools 改成 workspace-bound。
4. `apply_edits` / `preview_edit` 接入 workspace resolver。

### P1: 下一批修

1. 关闭或约束 `withGlobalTauri`。
2. 缩小 legacy `run_command` allow-list。
3. Security Gate 增加：`execute_tool` permission enforcement 检测、generic fs absolute path 回归测试。
4. 新增 release-mode gate：global Tauri API 不允许 release 打包。

### P2: 后续治理

1. 逐步减少 legacy `innerHTML` allowlist。
2. Provider workspace path 后端化。
3. dependency audit 接入 CI。
4. 生成 Security Workflow Report 模板并接入 `/security scan`。

---

## 8. Suggested Regression Tests

```bash
# Existing gate
npm run test:security-gate

# New tests to add
cargo test -p interface-desktop -- execute_tool_denies_requires_confirmation_without_token
cargo test -p interface-desktop -- execute_tool_rejects_absolute_file_tool_path
cargo test -p interface-desktop -- apply_edits_rejects_workspace_escape
cargo test -p interface-desktop -- preview_edit_rejects_workspace_escape
cargo test -p engine-tool-system -- fs_tools_reject_absolute_when_allowed_paths_missing
cargo test -p engine-tool-system -- delete_file_rejects_outside_workspace

# Suggested static checks
rg -n "async fn execute_tool|tool\.execute\(|permissions\(|requires_confirmation" src/interface/desktop/src/main.rs src/engine/tool-system/src
rg -n "std::fs::read_to_string\(&path\)|edit_file.*path|preview_edit|apply_edits" src/interface/desktop/src/main.rs
rg -n "withGlobalTauri|innerHTML|insertAdjacentHTML" src/interface/desktop/tauri.conf.json src/interface/web
```

---

## 9. Report Verdict

**结论**: 安全地基方向正确，但核心执行边界未闭合。  
**上线建议**: 本地个人开发可继续使用；不建议用于不可信前端内容、插件生态、远程任务执行、多用户场景。  
**下一步**: 先做 P0 修复：`execute_tool` permission gate + generic file tools workspace-bound。  

人话版：现在像家里门锁、防盗窗、烟雾报警器都有了，但工具柜钥匙放在前台抽屉里。第一件事不是再买一个摄像头，而是先把工具柜锁上。
