# Codex Security 工作流演示：审查 `Cognitive-Architect/hajimi-code-cli`

> 版本：2026-05-13  
> 范围：公开 GitHub 仓库静态阅读 + 本地 PoC 验证  
> 说明：这不是 OpenAI Codex Security 云端产品的真实扫描结果，而是按 Codex Security 的公开工作流做的一次“同款流程模拟”：威胁建模 → 代码扫描 → 漏洞候选 → 沙箱验证 → 修复建议 → 复测计划。

---

## 0. 一句话结论

`hajimi-code-cli` 的主要安全风险不在“普通网页 bug”，而在 **AI/前端能不能通过工具系统碰到本机命令、文件系统、API Key、Git 仓库**。

这类项目的安全审查重点应该放在：

1. 命令执行是否真的被限制住；
2. workspace 文件读写是否真的出不去；
3. Tauri 前端如果出现 XSS，能不能直接调用后端危险能力；
4. AI agent / MCP 工具调用有没有权限闸门；
5. 修复以后有没有回归测试证明“这次真堵上了”。

人话版：  
这个项目像一个“带 AI 的本地 IDE”。IDE 本来就能读文件、跑命令、改代码，所以安全边界一定要非常清楚。否则 AI 或前端一旦被绕一下，就可能从“帮我写代码”变成“帮攻击者操作我电脑”——这就不是小感冒，是厨房着火。

---

## 1. 本次模拟的 Codex Security 六步法

### 1.1 建立扫描上下文

先搞清楚仓库是什么、能碰什么、谁能输入东西。

本仓库核心信息：

- 项目类型：local-first Tauri v2 桌面 AI IDE；
- 后端：Rust/Tauri；
- 前端：HTML/CSS/JavaScript；
- 能力：文件树、编辑器、终端、grep、git、AI chat、MCP、LSP、智能提交、AST 上下文；
- 安全相关能力：shell 白名单、API Key 存 OS Keyring、权限配置、workspace 限制；
- 高风险模块：`tool-system`、`desktop`、`mcp-server`、`security`、`storage`、`network`。

---

### 1.2 威胁建模

#### 资产：我们要保护什么？

| 资产 | 为什么重要 |
|---|---|
| 本地源代码 | 被删、被改、被泄露都很麻烦 |
| API Key / provider 配置 | 泄露后可能产生账单、数据泄露 |
| OS Keyring 里的密钥 | 这是钥匙串，不是普通配置 |
| workspace 外的文件 | IDE 不应该随便写到外面 |
| Git 仓库历史 | 恶意提交/自动提交会污染项目 |
| 本地命令执行能力 | 一旦失控，风险直接拉满 |
| MCP 本地上下文数据库 | 可能有提示词、历史上下文、项目资料 |

#### 入口：攻击者可能从哪里进来？

| 入口 | 典型输入 |
|---|---|
| Tauri 前端 UI | 文件名、搜索结果、聊天内容、git 输出、错误信息 |
| AI Agent 工具调用 | shell、read/write file、git、grep、workflow |
| MCP Server | 外部 MCP client 传入 tool 参数 |
| Provider 配置 | base URL、model name、API key |
| Git 仓库内容 | 恶意文件名、symlink、脚本、README 内容 |
| Terminal / Shell | 用户或 AI 要执行的命令 |

#### 信任边界：哪些地方必须重点查？

| 边界 | 风险 |
|---|---|
| Web 前端 → Tauri 后端 | 前端一旦 XSS，可能调用后端命令 |
| AI Agent → Tool Registry | AI 不能默认等于可信用户 |
| Workspace → 本机文件系统 | 路径校验和 symlink 是经典翻车点 |
| MCP Client → MCP Tools | 外部调用必须做输入校验和权限控制 |
| Provider 网络 → 本地配置 | base URL 和 key 管理要防泄露/滥用 |

---

## 2. 扫描计划

### 2.1 先扫“能炸锅”的地方

优先级不是“哪个文件最新”，而是“哪个能力最危险”。

| 优先级 | 模块 | 审查原因 |
|---|---|---|
| P0 | shell / command runner | 可以执行本机命令 |
| P0 | file read/write/delete | 可以改本地文件 |
| P0 | Tauri invoke 暴露面 | 前端能调用后端 |
| P1 | API key / provider config | 密钥泄露风险 |
| P1 | MCP server | 外部工具入口 |
| P1 | Git tools | 自动 add/commit/push 风险 |
| P2 | SecurityAuditTool | 看它是否只是“正则扫描器” |
| P2 | CSP / innerHTML | XSS 到本机能力的跳板 |

### 2.2 每个发现必须带四件套

以后让 AI 审代码时，要求它每个安全发现都必须输出：

1. **攻击路径**：攻击者从哪里进来，怎么走到危险点；
2. **代码证据**：具体文件、函数、逻辑；
3. **验证证据**：PoC、测试输出、日志、失败/成功结果；
4. **修复建议**：最好是最小补丁 + 回归测试。

没有这四件套，就不要叫“漏洞确认”，只能叫“可疑点”。

---

## 3. 本次发现摘要

| ID | 标题 | 严重度 | 状态 | 核心原因 |
|---|---:|---:|---:|---|
| CS-HAJIMI-001 | Shell 白名单可被嵌套 shell 绕过 | 高 | 已验证 | 允许 `bash/sh/pwsh`，且只校验第一段命令 |
| CS-HAJIMI-002 | workspace 路径校验可能被 symlink 新文件写入绕过 | 高 | 已验证 | 目标文件不存在时使用未 canonicalize 的路径兜底 |
| CS-HAJIMI-003 | Tauri 关闭 CSP 且启用 global Tauri API，XSS 后果被放大 | 中-高 | 静态确认，未做 XSS 复现 | 前端一旦注入脚本，后端能力暴露面更大 |
| CS-HAJIMI-004 | 前端调用 `mkdir/mv/rm -rf`，但后端通用命令白名单不允许 | 中 | 静态确认 | 这是功能错配，也会诱导未来粗暴放开危险命令 |
| CS-HAJIMI-005 | 内置 SecurityAuditTool 偏正则扫描，覆盖不够 | 低-中 | 静态确认 | 能抓 secret/panic/unwrap，但不是完整语义安全扫描 |

---

## 4. 详细发现

---

### CS-HAJIMI-001：Shell 白名单可被嵌套 shell 绕过

**严重度：高**  
**状态：已验证**  
**影响范围：`src/engine/tool-system/src/shell.rs`**

#### 现象

代码看起来做了白名单：

- 只允许 `git`、`cargo`、`npm`、`node`、`python`、`ls` 等命令；
- 禁止 `;`、`&`、`|`、反引号、`$`、括号等 shell 元字符；
- 声称禁止 `rm/sudo` 等危险命令。

但问题是：白名单里包含了 `bash`、`sh`、`pwsh`、`powershell` 这类 shell 解释器。

如果校验逻辑只看第一个 token，那么：

```bash
bash -c 'touch /tmp/hajimi_shell_bypass_marker'
```

第一段是 `bash`，通过白名单。  
内部真实执行的是 `touch /tmp/...`，这个命令其实绕开了“只允许第一段”的限制。

人话版：  
你家门口保安说“只有员工能进”，结果员工可以带一整个施工队进去，而且保安不查施工队。那这白名单就有点像纸糊的，帅是帅，挡不住人。

#### 本地验证结果

用同等逻辑模拟 `check_allow_list` 和执行方式：

```text
command: bash -c 'touch /tmp/hajimi_shell_bypass_marker'
allowlist_result: True allowed
exit_code: 0
marker_created: True
```

这说明该命令能通过白名单并成功执行内部非白名单命令。

#### 缓解因素

当前 `ShellTool` 默认权限是需要确认/拒绝，不是完全静默执行。  
所以这个问题的实际利用通常还依赖：

- 用户是否批准了 AI 的 shell 执行；
- AI 是否能诱导用户确认；
- 该工具是否在某些 workflow 中被自动授权。

#### 修复建议

**最低成本修法：**

1. 从 `ALLOWED_COMMANDS` 移除：
   - `bash`
   - `sh`
   - `pwsh`
   - `powershell`
2. 不要用 `bash -c <整段命令>` 执行用户输入命令；
3. 把命令拆成：
   - 程序名；
   - 参数数组；
   - 明确的 cwd；
   - 明确的 env；
4. 每个命令做子命令级别限制，例如：
   - `git status` 可以；
   - `git push --force` 需要确认；
   - `npm install` 需要确认；
   - `curl http://... | sh` 永远不可以。

#### 回归测试

新增测试用例：

```text
bash -c 'touch /tmp/x'        => 必须拒绝
sh -c 'touch /tmp/x'          => 必须拒绝
pwsh -Command "..."           => 必须拒绝
git status                    => 可以允许
cargo check                   => 可以允许
rm -rf .                      => 必须拒绝
```

---

### CS-HAJIMI-002：workspace 路径校验可能被 symlink 新文件写入绕过

**严重度：高**  
**状态：已验证**  
**影响范围：`src/interface/desktop/src/main.rs`**

#### 现象

桌面端路径校验逻辑大致是：

1. 拒绝 `..`；
2. 把相对路径拼到 workspace 根目录；
3. 尝试 `canonicalize`；
4. 如果目标不存在，fallback 到未解析的 joined path；
5. 判断结果是否 `starts_with(workspace)`。

这个逻辑对“已经存在的真实文件”比较有效。  
但对“symlink 目录下面的新文件”会有问题：

```text
workspace/
  link -> /tmp/outside/
```

如果写入：

```text
workspace/link/new.txt
```

`new.txt` 不存在，canonicalize 失败，于是校验 fallback 到：

```text
workspace/link/new.txt
```

它看起来还在 workspace 内。  
但实际写入时，系统会顺着 symlink，把文件写到：

```text
/tmp/outside/new.txt
```

人话版：  
门卫只看地址写着“小区 3 栋”，但没发现 3 栋门口其实有个传送门，走进去直接到隔壁小区。地址看起来没出界，脚已经出界了。

#### 本地验证结果

我用同等逻辑做了 PoC：

```text
validator_returned: /tmp/.../hajimi-workspace/link/new.txt
starts_with_base: True
outside_file_exists: True
outside_file_content: written outside via symlink parent
```

这说明：校验认为路径在 workspace 里，但实际文件写到了 workspace 外面。

#### 攻击条件

攻击者需要能让 workspace 中出现 symlink。常见来源包括：

- 用户打开了包含 symlink 的项目；
- Git 仓库里包含 symlink；
- 某个工具/脚本在 workspace 内创建了 symlink；
- 用户或 AI 创建了 symlink 后再调用写文件能力。

#### 修复建议

**核心原则：不要只 canonicalize 目标文件，要检查路径的每一段。**

建议：

1. 对写入新文件，先 canonicalize 父目录；
2. 父目录必须已经存在并且在 workspace 内；
3. 拒绝路径中任意 symlink 组件；
4. Linux 上可考虑 `openat2` / `RESOLVE_BENEATH` / `NO_SYMLINKS`；
5. Rust 跨平台可考虑 `cap-std` 这类 capability-based filesystem 库；
6. 对 `write_file`、`create_dir_all`、`rename`、`delete` 都统一用一个安全路径解析函数。

#### 回归测试

```text
workspace/link -> /tmp/outside
write_file("link/new.txt") => 必须拒绝
read_file("link/existing.txt") => 必须拒绝
delete_file("link/target.txt") => 必须拒绝
rename("safe.txt", "link/x.txt") => 必须拒绝
```

---

### CS-HAJIMI-003：Tauri 关闭 CSP 且启用 global Tauri API，XSS 后果被放大

**严重度：中-高**  
**状态：静态确认，未复现 XSS**  
**影响范围：`src/interface/desktop/src-tauri/tauri.conf.json` 与前端 JS**

#### 现象

Tauri 配置中：

```json
{
  "app": {
    "withGlobalTauri": true
  },
  "security": {
    "csp": null
  }
}
```

这两个配置叠加后，风险模型是：

1. 如果前端出现 XSS；
2. 注入脚本可能更容易访问 Tauri invoke 能力；
3. 然后调用后端命令、文件、工具相关接口；
4. 最终从“前端小洞”升级成“本机能力洞”。

注意：  
我这次没有证明当前前端已经存在可利用 XSS。  
但这个配置会让未来任何 XSS 的后果明显变大。

人话版：  
不是说现在厨房已经着火，而是你把防火门拆了，煤气阀还放在客厅。以后真有火星，后果会更难看。

#### 修复建议

1. 关闭 `withGlobalTauri`；
2. 前端通过模块化 import 调用 Tauri API；
3. 开启 CSP，例如：
   - `default-src 'self'`
   - `script-src 'self'`
   - `style-src 'self' 'unsafe-inline'`
   - `img-src 'self' asset: data:`
   - `connect-src 'self' http://127.0.0.1:*`
4. 所有 `innerHTML` 统一审查，能用 DOM API 就别拼 HTML；
5. 对聊天内容、搜索结果、文件名、Git 输出、错误信息全部保持 escape。

#### 回归测试

```text
恶意文件名：<img src=x onerror=alert(1)>
恶意 git 输出：包含 HTML 标签
恶意 chat message：包含 script/img/onerror
预期：页面只展示文本，不执行脚本
```

---

### CS-HAJIMI-004：前端调用 `mkdir/mv/rm -rf`，但后端通用命令白名单不允许

**严重度：中**  
**状态：静态确认**  
**影响范围：`src/interface/desktop/src/app.js` 与 `main.rs`**

#### 现象

前端里存在类似：

```javascript
invoke('run_command', { cmd: 'mkdir', ... })
invoke('run_command', { cmd: 'mv', ... })
invoke('run_command', { cmd: 'rm', args: ['-rf', path] })
```

但桌面端后端的命令白名单看起来并不包含 `mkdir`、`mv`、`rm`。

这首先是功能 bug：对应功能可能跑不通。  
但更大的安全风险是：未来为了“让功能能跑”，可能有人把 `rm/mv/mkdir` 一股脑加进通用命令白名单。尤其是 `rm -rf`，这玩意别随便开门，开了就是请蟑螂吃自助餐。

#### 正确修法

不要通过通用 shell 跑文件操作。  
应该做专门的 Tauri 命令：

```text
create_folder(path)
rename_path(old_path, new_path)
delete_path(path, recursive: bool)
```

每个命令都必须：

1. 先通过 workspace 安全路径解析；
2. 禁止 symlink 逃逸；
3. 删除前做确认；
4. 递归删除只允许 workspace 内；
5. 记录审计日志。

---

### CS-HAJIMI-005：内置 SecurityAuditTool 偏正则扫描，覆盖不够

**严重度：低-中**  
**状态：静态确认**  
**影响范围：`src/foundation/security/src/security.rs`**

#### 现象

内置 `SecurityAuditTool` 会扫描：

- AWS key；
- GitHub token；
- Stripe key；
- private key；
- `todo!`；
- `unwrap()`；
- `panic!`。

这对基础 hygiene 有用，但它不等于完整安全扫描。  
它抓得住“地上有香蕉皮”，抓不住“厨房煤气管路设计有问题”。

#### 建议增强

把它当作第一层扫描，而不是最终答案。

建议组合：

| 工具 | 用途 |
|---|---|
| gitleaks | secret 扫描 |
| cargo audit | Rust 依赖漏洞 |
| npm audit / pnpm audit | JS 依赖漏洞 |
| semgrep | 代码模式安全扫描 |
| cargo clippy | Rust 代码质量 |
| 自定义测试 | shell/path/Tauri 权限回归 |

---

## 5. 建议的修复顺序

### P0：先堵住本机能力逃逸

1. 修 `ShellTool`：
   - 移除 shell 解释器；
   - 不使用 `bash -c`；
   - 加绕过测试。
2. 修 workspace 路径解析：
   - 拒绝 symlink；
   - canonicalize 父目录；
   - 加 symlink 写入/读取/删除测试。

### P1：降低前端到后端的爆炸半径

3. 开启 Tauri CSP；
4. 关闭 global Tauri API；
5. 审查所有 `innerHTML`。

### P2：把扫描流程产品化

6. 增强 `SecurityAuditTool`；
7. CI 加：
   - cargo test；
   - cargo clippy；
   - cargo audit；
   - npm/pnpm audit；
   - gitleaks；
   - semgrep；
8. 每次修安全问题都要附验证 receipt。

---

## 6. 以后让 AI 审这个仓库的标准提示词

你可以把下面这段直接丢给 AI：

```text
你现在模拟 Codex Security，对当前 GitHub 仓库做安全审查。

必须按以下流程输出：

1. 仓库安全画像
   - 项目类型
   - 高风险能力
   - 关键资产
   - 外部入口
   - 信任边界

2. 威胁模型
   - 谁可能攻击
   - 从哪里输入
   - 能碰到什么敏感资源
   - 最坏结果是什么

3. 扫描计划
   - 优先审查 shell/command runner
   - 优先审查 file read/write/delete/path validation
   - 优先审查 Tauri invoke / frontend XSS / CSP
   - 优先审查 MCP tools
   - 优先审查 API key/provider config
   - 优先审查 git workflow 自动化

4. 漏洞发现
   每个发现必须包含：
   - ID
   - 严重度
   - 状态：已验证 / 静态确认 / 未验证猜测
   - 攻击路径
   - 代码证据：文件、函数、关键逻辑
   - 验证证据：PoC、测试命令、输出
   - 影响
   - 缓解因素
   - 最小修复建议
   - 回归测试

5. 验证要求
   - 不允许把“可能有问题”写成“已确认漏洞”
   - 没有 PoC 或测试输出的，只能标为“未验证”
   - 每个高危发现都要给可复现步骤
   - 每个修复建议都要给回归测试

6. 输出格式
   - 先给摘要表
   - 再给详细发现
   - 再给修复优先级
   - 最后给 TODO checklist
```

---

## 7. 每次安全审查的 Markdown 模板

```markdown
# Security Review Report

## Scope

- Repository:
- Commit:
- Date:
- Reviewer:
- Method:
- Limitations:

## Threat Model

### Assets

| Asset | Why it matters |
|---|---|

### Entry Points

| Entry point | Input |
|---|---|

### Trust Boundaries

| Boundary | Risk |
|---|---|

## Findings Summary

| ID | Title | Severity | Status |
|---|---|---|---|

## Findings

### FINDING-ID: Title

- Severity:
- Status:
- Affected files:
- Attack path:
- Evidence:
- Validation:
- Impact:
- Mitigations:
- Recommended fix:
- Regression tests:

## Patch Plan

| Priority | Task | Owner | Done when |
|---|---|---|---|

## Revalidation

| Test | Expected result | Actual result |
|---|---|---|
```

---

## 8. 最小 CI 安全门禁建议

如果以后要放进 GitHub Actions，可以先做这个级别：

```yaml
name: security-check

on:
  pull_request:
  push:
    branches: [ main ]

jobs:
  rust-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo clippy --workspace --all-targets -- -D warnings

  secret-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: gitleaks/gitleaks-action@v2

  js-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm audit --audit-level=high
```

注意：  
这个只是入门门禁。真正要做强，还要加 Semgrep、自定义 symlink 回归测试、自定义 shell bypass 回归测试。

---

## 9. 人工验收清单

修复后，不要只看“代码改了”。要看这些证据：

```text
[ ] bash -c 绕过测试失败，即被拒绝
[ ] sh -c 绕过测试失败，即被拒绝
[ ] workspace/link -> outside 写入被拒绝
[ ] read/delete/rename symlink 路径都被拒绝
[ ] Tauri CSP 不再是 null
[ ] withGlobalTauri 不再是 true
[ ] 前端恶意 HTML 只显示文本，不执行
[ ] 删除/重命名/新建文件不走通用 run_command
[ ] CI 跑过 cargo test/clippy/security scan
[ ] 每个高危修复都有测试 receipt
```

---

## 10. 最后判断

这次最值得优先修的不是“扫描器多不多”，而是两个边界：

1. **Shell 白名单边界**：不能让 shell 解释器自己成为绕过工具；
2. **Workspace 文件边界**：不能让 symlink 把写入传送到 workspace 外。

这两个修完后，项目的本地安全底座会稳很多。  
后面再做 CSP、MCP 权限、CI 安全扫描，就不是救火，是装修加固。
