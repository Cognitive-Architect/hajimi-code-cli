# Day 04 派单: ShellTool 用户白名单收紧

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 4，处理 `CS-HAJIMI-001` Shell 白名单绕过风险。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: ShellTool 用户白名单收紧
- **轰炸目标**: 从 `src/engine/tool-system/src/shell.rs` 的用户命令 allow-list 中移除 `bash/sh/pwsh/powershell`，并补充拒绝测试，同时保留内部外层执行器能力
- **任务性质**: 安全修复 + 回归测试
- **输入基线**: 完整技术背景见模块2
- **输出要求**: shell 解释器用户命令被拒绝 + 低风险命令保持可用 + `cargo test -p engine-tool-system -- test_allow_list` 通过
- **通用铁律**:
  1. 用户输入命令不允许以 shell 解释器作为 program
  2. 内部执行器仍可根据平台选择外层 shell，不作为用户白名单
  3. 不扩大 `run_command` 或 ShellTool 权限
  4. metachar 拦截必须保持
  5. 测试必须从“允许 powershell/pwsh”改为“拒绝 powershell/pwsh”

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 记录当前分支和 HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `CS-HAJIMI-001` 当前 `OPEN / P0` | 债务总表第 4.1 节 | 必须 |
| 目标文件 | ShellTool 实现和测试 | `src/engine/tool-system/src/shell.rs:20-40`, `:62-153`, `:320-331` | 必须 |
| 当前缺陷 | `ALLOWED_COMMANDS` 包含 `bash/sh/pwsh/powershell` | `rg -n "\"bash\"|\"sh\"|\"pwsh\"|\"powershell\"" src/engine/tool-system/src/shell.rs` | 必须 |
| 当前测试 | `powershell/pwsh` 当前被断言允许 | `rg -n "powershell -Command|pwsh -Command|test_allow_list" src/engine/tool-system/src/shell.rs` | 必须 |
| 保留行为 | `git`, `cargo`, `npm`, `node`, `python3`, `ls`, `cat`, `echo`, `pwd` 等低风险命令继续 allow | `cargo test -p engine-tool-system -- test_allow_list` | 必须 |
| 技术约束 | Engine 层不能依赖 Interface | `cargo check -p engine-tool-system` | 必须 |
| 文档同步 | 状态最多 `OPEN -> VERIFY` | 债务总表或 Day receipt | 按需 |

### 探索补充栏

本任务为已知解实现。不得把“Shell 功能降级债”在本日恢复；复杂 shell 能力恢复受 `SHELL-FEATURE-DEBT-002` 约束，必须等安全 sandbox 方案。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-04/15
- **角色**: Engineer
- **目标**: 让 `bash/sh/pwsh/powershell` 作为用户命令全部被 allow-list 拒绝
- **输入**: `shell.rs` allow-list、executor、`test_allow_list`
- **依赖关系**: 可与 Day 3 串行执行；不得依赖 Day 5+

### 2）输出交付物

- **变更文件**:
  - `src/engine/tool-system/src/shell.rs`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`，如状态同步需要
- **核心修改点**:
  - 从 `ALLOWED_COMMANDS` 移除 `bash`, `sh`, `pwsh`, `powershell`
  - 将 `powershell -Command Get-Process` 和 `pwsh -Command Write-Host test` 测试改为 `is_err()`
  - 新增 `bash script.sh`、`sh script.sh` 拒绝测试
  - 保留内部 `DefaultShellExecutor::shell()` 外层选择逻辑
- **必须包含**:
  - `git status`, `cargo check`, `ls -la` 等允许命令仍通过
  - `echo ; rm -rf /` 等 metachar 仍被拒绝
  - 错误信息不泄露额外敏感信息
- **禁止包含**:
  - 删除 metachar 检查
  - 删除内部平台 shell wrapper
  - 为了测试通过扩大白名单
  - 修改 Interface 层文件
- **交付证明**:
  - `cargo test -p engine-tool-system -- test_allow_list`
  - `cargo test -p engine-tool-system`
  - `cargo check --workspace` 或说明外部非本次错误

### 3）规模与复杂度观察

- **推荐目标**: 最小 diff，集中改 allow-list 和测试
- **复杂度说明**: 本日不做结构化 `program + args[]` 中期方案，只收紧当前 allow-list
- **禁止行为**: 以大重构替代 P0 快速修复

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | engine tool-system 编译通过 | `cargo check -p engine-tool-system` | 返工 |
| FMT | Rust 格式通过 | `cargo fmt -- --check` | 返工 |
| LINT | 不新增 warning | `cargo clippy -p engine-tool-system -- -D warnings` 或 `N/A + 原因` | 返工或声明债务 |
| TEST | allow-list 测试通过 | `cargo test -p engine-tool-system -- test_allow_list` | 返工 |
| ARCH | Engine 不依赖 Interface | `cargo check -p engine-tool-system` | 返工 |
| REAL | shell 解释器拒绝 | 测试断言 + `rg` 白名单检查 | 返工 |
| DOC | 状态更新有 receipt | 债务总表 diff 或 Day receipt | 返工或声明无文档改动 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | allow-list 不含 `bash` | `rg -n "\"bash\"" src/engine/tool-system/src/shell.rs` 并确认不是 `ALLOWED_COMMANDS` 项 | [ ] |
| FUNC | FUNC-002 | allow-list 不含 `sh` | `rg -n "\"sh\"" src/engine/tool-system/src/shell.rs` 并确认不是 `ALLOWED_COMMANDS` 项 | [ ] |
| FUNC | FUNC-003 | allow-list 不含 `pwsh` | `rg -n "\"pwsh\"" src/engine/tool-system/src/shell.rs` 并确认不是 `ALLOWED_COMMANDS` 项 | [ ] |
| FUNC | FUNC-004 | allow-list 不含 `powershell` | `rg -n "\"powershell\"" src/engine/tool-system/src/shell.rs` 并确认不是 `ALLOWED_COMMANDS` 项 | [ ] |
| CONST | CONST-001 | 低风险命令仍允许 | `cargo test -p engine-tool-system -- test_allow_list` | [ ] |
| CONST | CONST-002 | metachar 拦截仍有效 | `rg -n "metachar|contains\\(';|echo ; rm" src/engine/tool-system/src/shell.rs` | [ ] |
| CONST | CONST-003 | 外层 shell wrapper 保留 | `rg -n "DefaultShellExecutor|fn shell\\(|powershell|bash" src/engine/tool-system/src/shell.rs` 并人工确认仅内部使用 | [ ] |
| CONST | CONST-004 | engine crate 编译通过 | `cargo check -p engine-tool-system` | [ ] |
| NEG | NEG-001 | `bash script.sh` 被拒绝 | 测试断言或 `test_allow_list` 输出 | [ ] |
| NEG | NEG-002 | `sh script.sh` 被拒绝 | 测试断言或 `test_allow_list` 输出 | [ ] |
| NEG | NEG-003 | `pwsh -Command` 被拒绝 | 测试断言或 `test_allow_list` 输出 | [ ] |
| NEG | NEG-004 | `powershell -Command` 被拒绝 | 测试断言或 `test_allow_list` 输出 | [ ] |
| UX | UX-001 | 错误信息说明命令不允许 | 单测或手动输出摘要 | [ ] |
| UX | UX-002 | `git status` 仍允许 | `test_allow_list` 中保留断言 | [ ] |
| E2E | E2E-001 | workspace 编译或说明外部错误 | `cargo check --workspace` | [ ] |
| High | HIGH-001 | 未恢复复杂 shell 功能 | `rg -n "bash -c|powershell -Command" src/engine/tool-system/src/shell.rs` 并确认只在内部 executor | [ ] |

---

## 【模块3-B】地狱红线

1. `ALLOWED_COMMANDS` 仍含 shell 解释器，返工
2. 测试仍断言 `powershell/pwsh` 可用，返工
3. 删除 metachar 检查，返工
4. 删除内部执行器导致命令完全不可执行且未修复，返工
5. 将 `rm` 或高风险命令加入 allow-list，返工
6. 不跑 `test_allow_list`，返工
7. `cargo check -p engine-tool-system` 失败仍收卷，返工
8. 借本日恢复管道/重定向功能，返工
9. 修改 Interface 层绕过 ShellTool，返工
10. 状态直接 `CLEARED` 但无完整 workspace 验证，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 四个 shell 解释器是否拒绝 | [ ] | NEG-001~004 | |
| RG | 低风险命令是否保留 | [ ] | CONST-001, UX-002 | |
| NG | metachar 是否仍拒绝 | [ ] | CONST-002 | |
| UX | 错误信息是否可读 | [ ] | UX-001 | |
| E2E | workspace 编译是否检查 | [ ] | E2E-001 | |
| High | 是否未恢复复杂 shell | [ ] | HIGH-001 | |
| 字段完整性 | 测试输出是否记录 | [ ] | 收卷报告 | |
| 需求映射 | 是否映射 `CS-HAJIMI-001` | [ ] | 债务总表 4.1 | |
| 自测执行 | 是否跑过 cargo test | [ ] | TEST 闸门 | |
| 范围边界与债务 | 是否未扩到 Shell 功能恢复 | [ ] | git diff | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-04/15 完成并提交

### 提交信息
- Commit: `security(engine/tool-system): reject shell interpreters in user allow-list`
- 分支: `<实际分支>`
- 变更文件:
  - `src/engine/tool-system/src/shell.rs`

### 本轮目标与实际结果
- 目标: 用户命令拒绝 `bash/sh/pwsh/powershell`
- 实际完成: `<列出 allow-list 和测试变化>`
- 未完成/不在范围: 结构化 program+args 和 sandbox 属中期方案

### 自动化质量检查报告
- `cargo test -p engine-tool-system -- test_allow_list`: `<摘要>`
- `cargo test -p engine-tool-system`: `<摘要>`
- `cargo check -p engine-tool-system`: `<摘要>`
- `cargo check --workspace`: `<摘要或非本次错误>`

### 债务声明
- `SHELL-FEATURE-DEBT-002`: 复杂 shell 功能继续受限，属安全设计，不在本日恢复

### 风险与回滚点
- 主要风险: 用户体验下降，部分脚本命令不能直接跑
- 回滚方式: 安全修复不建议回滚；紧急回滚用 `git revert <commit>` 并重开 P0 债务
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 测试暴露内部执行器和用户 allow-list 混在一起 | 暂停扩展，只拆最小函数区分二者 | 返工 |
| QUALITY-001 | engine-tool-system 全量测试失败 | 先定位是否本次修改导致 | 返工或记录外部错误 |
| TEST-001 | Windows shell 行为导致测试不稳定 | 测 `check_allow_list` 纯函数，不测真实 shell 执行 | 有条件交付 |
| SAFETY-001 | 有人要求重新允许 shell 解释器 | 拒绝本日变更，记录为产品决策待审 | 不放行 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 04 ShellTool 用户白名单收紧**。

### 关键约束
- 用户命令拒绝 `bash/sh/pwsh/powershell`
- 保留内部执行器 wrapper
- 不恢复复杂 shell 功能
- 不扩大任何白名单

### 验收铁律
- `cargo test -p engine-tool-system -- test_allow_list` 通过
- `cargo check -p engine-tool-system` 通过
- 四个 shell 解释器拒绝测试存在
- `git status` 等低风险命令仍允许

闭环启动，Day 04，执行。
