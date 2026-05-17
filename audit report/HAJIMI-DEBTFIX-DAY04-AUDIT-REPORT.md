# HAJIMI-DEBTFIX Day 04 建设性审计报告

> 审计对象：`docs/roadmap/hajimi debtFix/task/Day-04-Shell-Allowlist-Hardening.md`  
> 审计官：压力怪  
> 审计日期：2026-05-16  
> 关联阶段：HAJIMI-DEBTFIX Phase Day 04  
> 当前状态：A 级 / Go（2026-05-16 收尾复审）

---

## 审计背景

### 项目阶段

HAJIMI-DEBTFIX Day 04：ShellTool 用户 allow-list 收紧。目标是处理 `CS-HAJIMI-001` 中用户可直接调用 `bash/sh/pwsh/powershell` 的 P0 风险，同时保留内部跨平台 shell wrapper，不恢复复杂 shell 功能。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `shell.rs` | `src/engine/tool-system/src/shell.rs` | 从 `ALLOWED_COMMANDS` 移除 `bash/sh/pwsh/powershell`，更新 `test_allow_list` 断言 | Engineer | 部分通过 |
| 2 | `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 4.1 追加 Day 04 修复与验证，但总览矩阵仍保留旧 `OPEN` 描述 | Engineer | 部分同步 |

### 关键代码片段

```rust
// 来自 src/engine/tool-system/src/shell.rs:20-42
const ALLOWED_COMMANDS: &[&str] = &[
    "git",
    "cargo",
    "npm",
    "node",
    "python3",
    "ls",
    "cat",
    "echo",
    "pwd",
    "which",
    "forge",
    "cast",
    "anvil",
    "slither",
    "rustc",
    "clippy-driver",
    "curl",
    "wget",
    "tar",
    "unzip",
    "make",
];
```

```rust
// 来自 src/engine/tool-system/src/shell.rs:155-159
// PowerShell specific metachar checks (less aggressive as -Command is used)
if trimmed.contains(['&', ';', '`']) {
    return Err(ToolError {
        message: "Dangerous PowerShell metacharacters not permitted.".to_string(),
        kind: ToolErrorKind::PermissionDenied,
    });
}
```

```rust
// 来自 src/engine/tool-system/src/shell.rs:248-255
let (shell, mut sargs) = self.executor.shell_cmd();
if shell == "bash" {
    sargs.push(a.command.clone());
} else {
    sargs.push(format!(
        "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;{}",
        a.command
    ));
}
```

### 已知限制 / 环境问题

- Windows 下 cargo 写 `target/debug` 时可能遇到沙箱 `拒绝访问`，本审计已按流程提权重跑关键 Cargo 命令。
- 当前工作区存在既有 `src/MEMORY.md`、Day 03 文件操作相关改动，不属于 Day 04 审计范围。
- `audit report/` 与 `docs/roadmap/` 被 `.gitignore` 忽略，后续提交审计报告或债务文档需 `git add -f`。

---

## 质量门禁

- 已读取 Day 04 工单、建设性审计模板、B-09 审计报告示例。
- 已抽查 `src/engine/tool-system/src/shell.rs` 的 allow-list、`BashExecutor`、`PowerShellExecutor`、执行入口和单元测试。
- 已抽查债务总表 `CS-HAJIMI-001` 总览和 4.1 详情。
- 已执行 `cargo fmt -- --check`、`cargo check -p engine-tool-system`、`cargo clippy -p engine-tool-system -- -D warnings`、`cargo test -p engine-tool-system -- test_allow_list`、`cargo test -p engine-tool-system`、`cargo check --workspace`。
- 已验证 `ALLOWED_COMMANDS` 不再包含 `bash/sh/pwsh/powershell`。
- 已验证 `test_allow_list` 对四个 shell 解释器有拒绝断言。

质量门禁满足出报告条件，但不满足 A 级放行条件。

---

## 审计目标

1. Shell 解释器是否从用户 allow-list 移除？
2. 内部 shell wrapper 是否保留且未暴露为用户白名单？
3. metachar 拦截是否在实际平台执行器中仍可靠？
4. 测试与债务文档是否足以支撑 `CS-HAJIMI-001 -> VERIFY`？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 解释器移除 | A | `ALLOWED_COMMANDS` 已移除 `bash/sh/pwsh/powershell`，用户首 token 直接调用这些解释器会被拒绝。 |
| 内部 wrapper 保留 | A | `BashExecutor::shell_cmd()` 和 `PowerShellExecutor::shell_cmd()` 仍作为内部执行器存在，未被重新加入用户 allow-list。 |
| 回归测试覆盖 | B | `test_allow_list` 已覆盖 `powershell/pwsh/bash/sh` 拒绝、`git/cargo/ls` 允许和 Bash metachar；但未覆盖 PowerShell metachar 绕过。 |
| 自动化闸门 | A | `fmt/check/clippy/test/workspace check` 均通过。 |
| 安全完整性 | C | Windows 实际执行器仍允许 `| $ ( ) < >` 等 PowerShell 可解释元字符；`git status | ...` 这类命令会通过 first-token allow-list。 |
| 文档同步 | C | 4.1 详情写 `VERIFY` 且追加 Day 04 修复，但总览矩阵仍写 `CS-HAJIMI-001` 为 `OPEN` 并描述旧状态；4.1 的“当前代码观察”也仍写 allow-list 包含四个解释器。 |

整体健康度评级：C 级。核心方向正确，但 Windows PowerShell 分支仍有 allow-list 后门形态，且债务文档状态链不一致。

---

## 关键疑问回答（Q1-Q3）

**Q1：用户命令是否还能直接以 `bash/sh/pwsh/powershell` 作为 program？**

结论：按当前 `check_allow_list()` 的 first-token 规则，不能。`ALLOWED_COMMANDS` 不再包含四个 shell 解释器，`test_allow_list` 也把这四类调用改为 `is_err()`。

**Q2：复杂 shell 能力是否被意外恢复？**

结论：在 Bash 分支基本没有恢复，但 Windows PowerShell 分支仍存在风险。`PowerShellExecutor::check_allow_list()` 只拒绝 `& ; \``，没有拒绝 `| $ ( ) { } < >`。由于执行入口会把用户命令拼入 `powershell -Command`，形如 `git status | <后续命令>` 的输入会先以允许的 `git` 通过白名单，再由 PowerShell 解释管道。这个行为和工单“metachar 拦截必须保持”“不恢复复杂 shell 功能”的约束冲突。

**Q3：债务状态是否可以从 `OPEN` 推进到 `VERIFY`？**

结论：暂不建议。解释器移除这部分可以进入 `VERIFY`，但 `CS-HAJIMI-001` 是 P0 Shell 边界债；在 PowerShell metachar 缺口修复前，整体债务仍不应被视为可验证完成。同时文档总览仍写 `OPEN`，详情写 `VERIFY`，需要同步。

---

## 验证结果（V1-V14）

| 验证ID | 命令 | 结果 | 证据 |
|:---|:---|:---:|:---|
| V1 | `git branch --show-current` | PASS | `v3.8.0-batch-1` |
| V2 | `git rev-parse HEAD` | PASS | `d697414f42584a0d0c9c85346a6a692e691c4dad` |
| V3 | `rg -n 'const ALLOWED_COMMANDS\|"bash"\|"sh"\|"pwsh"\|"powershell"' src/engine/tool-system/src/shell.rs` | PASS | 四个解释器不在 `ALLOWED_COMMANDS` 中；仅内部 wrapper / 测试出现 |
| V4 | `rg -n "bash script\|sh script\|powershell -Command\|pwsh -Command" src/engine/tool-system/src/shell.rs` | PASS | 四个拒绝断言存在于 `test_allow_list` |
| V5 | `rg -n "git status\|cargo check\|echo ; rm" src/engine/tool-system/src/shell.rs` | PASS | 低风险允许与 metachar 拒绝断言存在 |
| V6 | `cargo test -p engine-tool-system -- test_allow_list` | PASS | 1 passed；lib test 有既有 unused import warning |
| V7 | `cargo test -p engine-tool-system` | PASS | 73 passed；lib test 有既有 unused import warning |
| V8 | `cargo check -p engine-tool-system` | PASS | 退出码 0 |
| V9 | `cargo clippy -p engine-tool-system -- -D warnings` | PASS | 退出码 0 |
| V10 | `cargo fmt -- --check` | PASS | 退出码 0 |
| V11 | `cargo check --workspace` | PASS | 退出码 0 |
| V12 | `rg -n "contains\\(\\[" src/engine/tool-system/src/shell.rs` | FAIL | Bash 拒绝 `; & | \` $ ( ) { } < >`，PowerShell 仅拒绝 `& ; \`` |
| V13 | `rg -n "PowerShell specific metachar" src/engine/tool-system/src/shell.rs` | FAIL | 注释明确写 “less aggressive as -Command is used”，但 `-Command` 正是解释风险来源 |
| V14 | `rg -n "CS-HAJIMI-001" docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | FAIL | 总览矩阵为 `OPEN`，4.1 详情为 `VERIFY`，状态冲突 |

---

## 问题与建议

### 必须返工

1. 收紧 `PowerShellExecutor::check_allow_list()` 的 metachar 检查。最低要求：与 Bash 分支共享同一组 forbidden chars，覆盖 `; & | \` $ ( ) { } < >`。
2. 为 PowerShell 分支补拒绝测试，例如：

```rust
assert!(ps.check_allow_list("git status | Get-Process").is_err());
assert!(ps.check_allow_list("git status > out.txt").is_err());
assert!(ps.check_allow_list("echo $(Get-Process)").is_err());
```

3. 债务总表同步：如果修复后验证通过，`CS-HAJIMI-001` 总览与 4.1 详情都写 `VERIFY`；如果暂不修复，二者都应保持 `OPEN/P0` 并明确 PowerShell metachar 缺口。

### 建议补强

- 将 forbidden metachar 提取为共享常量，避免 Bash / PowerShell 分支漂移。
- 为错误信息补断言，确认解释器拒绝和 metachar 拒绝都返回 `PermissionDenied`，且不泄露额外环境信息。
- 保留 `SHELL-FEATURE-DEBT-002` 为 `OPEN BY DESIGN`，不要在 Day 04 恢复管道、重定向或 subshell。

---

## 评级结论

- 评级：C 级
- 状态：返工
- 与自测报告一致性：部分一致
- 地狱红线触发：是，实际 Windows ShellTool 仍存在 metachar 绕过面
- 是否需要返工：需要

---

## 压力怪评语

“方向对了，门也拆掉了几把钥匙，但 Windows 这边还留着管道口。`powershell -Command` 不是因为叫内部 wrapper 就自动安全，用户字符串一旦带着 `|` 进去，白名单就只管了第一个词。把 PowerShell 分支的 metachar 拦截补齐，这天才算真正收口。”

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-DEBTFIX-DAY04-AUDIT-REPORT.md`
- 关联状态：HAJIMI-DEBTFIX Day 04 / `CS-HAJIMI-001`
- 下一步建议：原始 C 级审计要求先返工 Day 04；2026-05-16 收尾复审已完成返工，当前可进入 Day 05。

---

## 收尾复审结论（2026-05-16）

### 复审范围

本次收尾针对原 C 级审计中的两项阻塞问题进行复核：

1. `PowerShellExecutor` 只拦 `& ; \``，未覆盖管道、重定向、变量替换、命令替换等 shell 元字符。
2. 债务总表中 `CS-HAJIMI-001` 总览与 4.1 详情状态冲突，且详情仍保留旧“当前仍包含解释器”的描述。

### 修复结果

| 原问题 | 收尾处理 | 结果 |
|:---|:---|:---:|
| PowerShell metachar 拦截弱于 Bash | 新增共享 `FORBIDDEN_METACHARS` 常量，Bash / PowerShell 均拒绝分号、管道、重定向、变量替换、命令替换等元字符 | PASS |
| 缺少 PowerShell 绕过测试 | `test_allow_list` 新增 `git status \| Get-Process`、`git status > out.txt`、`echo $(Get-Process)` 拒绝断言，并保留 `git status` 允许断言 | PASS |
| 债务文档状态冲突 | `CS-HAJIMI-001` 总览改为 `VERIFY`；4.1 改写为原始缺陷、Day 4 修复、验证证据、剩余实机 smoke 条件 | PASS |

### 复审验证

| 验证项 | 结果 |
|:---|:---:|
| `cargo fmt -- --check` | PASS |
| `cargo check -p engine-tool-system` | PASS |
| `cargo clippy -p engine-tool-system -- -D warnings` | PASS |
| `cargo test -p engine-tool-system -- test_allow_list` | PASS |
| `cargo test -p engine-tool-system` | PASS，73 passed |
| `cargo check --workspace` | PASS |
| `rg -n 'const ALLOWED_COMMANDS\|"bash"\|"sh"\|"pwsh"\|"powershell"' src/engine/tool-system/src/shell.rs` | PASS，解释器仅存在于内部 wrapper / 测试，不在用户 allow-list |
| `rg -n "git status \\\| Get-Process\|git status > out.txt\|echo \\$\\(Get-Process\\)" src/engine/tool-system/src/shell.rs` | PASS，PowerShell metachar 拒绝断言存在 |
| `git diff --check` | PASS，仅出现既有 CRLF 提示，无 whitespace error |

说明：Cargo 命令在默认沙箱内可能遇到 Windows `target/debug` 写入 `拒绝访问`，已按流程提权复跑并通过。

### 最终评级

- 评级：A 级
- 状态：Go
- 债务状态建议：`CS-HAJIMI-001` 保持 `VERIFY`
- 剩余关闭条件：补一轮真实 Tauri dev / ShellTool smoke，确认用户调用 `bash`、`pwsh`、PowerShell 管道命令均返回 allow-list / metachar 拒绝

Day 04 当前已满足进入下一日工单的 A 级基线。
