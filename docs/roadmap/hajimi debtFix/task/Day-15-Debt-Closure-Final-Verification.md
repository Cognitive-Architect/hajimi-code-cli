# Day 15 派单: 清债验证、文档闭环与 Final Commit

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 15，是 15 天清债执行的最终收口。

---

## 【模块1】饱和攻击头部

- **火力配置**: 2 Agent（Architect + Engineer）
- **任务名称**: 清债验证、文档闭环与 Final Commit
- **轰炸目标**: 汇总 Day 1-14 的代码、文档、receipt、测试结果，更新债务总表、`src/INDEX.md`、`src/ARCHITECTURE.md`，形成最终 closure 文档，并确保 git 状态和验证矩阵清晰
- **任务性质**: 收口验证 + 文档同步 + 发布准备
- **输入基线**: 完整技术背景见模块2
- **输出要求**: Closure 文档 + 债务状态矩阵 + 最终验证命令摘要 + 文档同步 + commit/push 准备清单
- **通用铁律**:
  1. 没有 receipt 的债务不得标 `CLEARED`
  2. 状态迁移必须遵守 `OPEN -> VERIFY -> CLEARED`
  3. P0/P1 安全/UX 必须有命令或实机证据
  4. `src/INDEX.md` 与 `src/ARCHITECTURE.md` 必须反映真实架构变更
  5. `docs/` 被 ignore 时，提交必须显式处理 `git add -f`

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD`; `git status --short --ignored` | 必须 |
| 债务总表 | 当前状态源 | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 必须 |
| Roadmap/Daily Plan | 批次和日计划 | `docs/roadmap/hajimi debtFix/plan/*.md` | 必须 |
| Day receipts | Day 1-14 所有 receipt | `Get-ChildItem -Recurse docs -Include *VERIFY*.md,*AUDIT*.md,*PLAN*.md,*CLOSURE*.md` | 必须 |
| 核心验证命令 | Rust/JS/test | `cargo check --workspace`; `cargo test -p engine-tool-system`; `cargo test -p intelligence-agent-core --lib`; `node --check src/interface/web/app.js` | 必须 |
| 前端模块 | Day 13-14 modules | `Get-ChildItem src/interface/web/modules -Filter *.js` | 必须 |
| 文档同步 | 架构/索引 | `src/ARCHITECTURE.md`; `src/INDEX.md` | 必须 |
| Closure 输出 | 最终文档 | `docs/debt/DEBT-REMEDIATION-CLOSURE-2026-05-xx.md` 或 roadmap debt 目录 | 必须 |

### 探索补充栏

本任务不是继续开发功能。若发现未闭合项，应该诚实降级状态并开 debt，而不是临时补大功能。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-15/15
- **角色**: Architect / Engineer
- **目标**: 让代码、文档、测试、receipt 四方一致
- **输入**: Day 1-14 全部产物、债务总表、源码、验证命令
- **依赖关系**: 依赖 Day 1-14 收卷

### 2）输出交付物

- **变更文件**:
  - `docs/debt/DEBT-REMEDIATION-CLOSURE-2026-05-xx.md`，或 `docs/roadmap/hajimi debtFix/debt/DEBT-REMEDIATION-CLOSURE-2026-05-xx.md`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`
  - `src/INDEX.md`
  - `src/ARCHITECTURE.md`
  - `src/MEMORY.md`，仅当债务基线规则变化
- **核心修改点**:
  - 创建最终清债总结
  - 更新每个债务状态及 receipt 链接
  - 更新前端模块、安全路径、CSP、checkpoint、prompt golden 等架构/索引说明
  - 运行最终验证矩阵
  - 输出 git 提交清单，包含 docs ignore 处理方式
- **必须包含**:
  - 每个 `CLEARED` 状态对应命令或 receipt
  - 每个未完成项对应 `DEBT-*`
  - 完整验证命令摘要
  - 最终 `git status --short --ignored` 摘要
- **禁止包含**:
  - 无证据清债
  - 删除历史 receipt
  - 将外部/环境失败写成通过
  - 大改代码
- **交付证明**:
  - closure 文档
  - 最终验证命令输出摘要
  - `git diff --check`
  - `git status --short --ignored`

### 3）规模与复杂度观察

- **推荐目标**: 收口而非继续开发；只做文档同步与小修
- **复杂度说明**: 如最终验证失败且非小问题，停止清债，写 blocker
- **禁止行为**: 为了 final commit 临时掩盖失败

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | workspace 编译通过 | `cargo check --workspace` | 返工或 blocker |
| FMT | Rust 格式通过 | `cargo fmt -- --check` | 返工或 blocker |
| LINT | JS 语法全部通过 | `node --check src/interface/web/app.js`; modules check | 返工 |
| TEST | 核心测试通过 | `cargo test -p engine-tool-system`; `cargo test -p intelligence-agent-core --lib` | 返工或 blocker |
| ARCH | 分层无明显违规 | `rg -n "use interface|interface::" src/engine src/intelligence` | 返工 |
| REAL | 状态迁移有证据 | closure 矩阵逐项核对 receipt | 返工 |
| DOC | ARCHITECTURE/INDEX 同步 | `rg -n "workspace resolver|CSP|checkpoint|modules|golden" src/ARCHITECTURE.md src/INDEX.md` | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | Closure 文档存在 | `Get-ChildItem -Recurse docs -Filter "DEBT-REMEDIATION-CLOSURE-*.md"` | [ ] |
| FUNC | FUNC-002 | 债务总表状态更新 | `rg -n "CS-HAJIMI-001|CS-HAJIMI-002|CS-HAJIMI-003|CS-HAJIMI-004|DEBT-THINKING" "docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md"` | [ ] |
| FUNC | FUNC-003 | `src/ARCHITECTURE.md` 同步 | `rg -n "workspace resolver|CSP|checkpoint|frontend modules" src/ARCHITECTURE.md` | [ ] |
| FUNC | FUNC-004 | `src/INDEX.md` 同步 | `rg -n "modules|security-dom|checkpoint|prompt golden|workspace resolver" src/INDEX.md` | [ ] |
| CONST | CONST-001 | workspace check 通过 | `cargo check --workspace` | [ ] |
| CONST | CONST-002 | engine tool-system 测试通过 | `cargo test -p engine-tool-system` | [ ] |
| CONST | CONST-003 | agent-core lib 测试通过 | `cargo test -p intelligence-agent-core --lib` | [ ] |
| CONST | CONST-004 | JS checks 通过 | `node --check src/interface/web/app.js`; modules checks | [ ] |
| NEG | NEG-001 | 无证据清债被拒绝 | closure 每个 `CLEARED` 有 receipt | [ ] |
| NEG | NEG-002 | 未完成项有 debt | `rg -n "DEBT-" docs/debt "docs/roadmap/hajimi debtFix/debt"` | [ ] |
| NEG | NEG-003 | 分层违规扫描 | `rg -n "use interface|interface::" src/engine src/intelligence` | [ ] |
| NEG | NEG-004 | docs ignore 风险处理 | `git status --short --ignored docs` + 收卷说明 `git add -f` | [ ] |
| UX | UX-001 | UX receipt 链接完整 | closure 引用 `UX-FILETREE-SESSION-VERIFY.md` | [ ] |
| UX | UX-002 | Thinking/Checkpoint receipt 链接完整 | closure 引用 checkpoint verify 文档 | [ ] |
| E2E | E2E-001 | 从安全到 UX 的最终矩阵完整 | closure 中有 final verification matrix | [ ] |
| High | HIGH-001 | P0 债务状态诚实 | Shell/workspace/Tauri 均有命令或 blocker | [ ] |

---

## 【模块3-B】地狱红线

1. 无 receipt 标 `CLEARED`，返工
2. 最终验证失败却提交“完成”，返工
3. 忘记更新 `src/INDEX.md` / `src/ARCHITECTURE.md`，返工
4. 删除历史债务证据，返工
5. `docs/` ignore 未说明，返工
6. 把 blocker 写成通过，返工
7. 混入新功能开发，返工
8. 未检查分层约束，返工
9. 未列出未完成债务，返工
10. git 状态不清晰，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 所有闭环文档是否存在 | [ ] | FUNC-001~004 | |
| RG | Day 1-14 产物是否引用 | [ ] | UX-001~002 | |
| NG | 未完成项是否留 debt | [ ] | NEG-001~002 | |
| UX | UX/Thinking receipt 是否完整 | [ ] | UX-001~002 | |
| E2E | 最终验证矩阵是否完整 | [ ] | E2E-001 | |
| High | P0 状态是否诚实 | [ ] | HIGH-001 | |
| 字段完整性 | closure 是否含命令/结果/路径 | [ ] | Closure 文档 | |
| 需求映射 | 是否映射三份计划文档 | [ ] | 输入基线 | |
| 自测执行 | 是否跑完整命令 | [ ] | 质量闸门 | |
| 范围边界与债务 | 未完成是否不开假票 | [ ] | `DEBT-*` | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-15/15 完成并提交

### 提交信息
- Commit: `docs(debt): close remediation batch with verification matrix`
- 分支: `<实际分支>`
- 变更文件:
  - `docs/debt/DEBT-REMEDIATION-CLOSURE-2026-05-xx.md`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`
  - `src/ARCHITECTURE.md`
  - `src/INDEX.md`

### 本轮目标与实际结果
- 目标: 清债收口和最终验证
- 实际完成: `<列出状态矩阵、验证命令、文档同步>`
- 未完成/不在范围: `<列出保留债务>`

### 自动化质量检查报告
- `cargo check --workspace`: `<摘要>`
- `cargo test -p engine-tool-system`: `<摘要>`
- `cargo test -p intelligence-agent-core --lib`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- `git diff --check`: `<摘要>`
- `git status --short --ignored`: `<摘要>`

### 债务声明
- `<所有未 CLEARED 项>`

### 风险与回滚点
- 主要风险: 文档状态与实际代码再次漂移
- 回滚方式: 回退 closure 和状态文档；不删除 receipt
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| QUALITY-001 | 完整验证失败且非小问题 | 停止 final commit，写 blocker | 不清债 |
| DOC-001 | receipt 缺失 | 状态保持 `VERIFY/PARTIAL`，补 debt | 不标 CLEARED |
| ARCH-001 | 分层违规 | 暂停收口，修分层或记录 blocker | 不提交 final |
| GIT-001 | docs 被 ignore | 使用 `git add -f` 或写提交说明 | 防止漏提交 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 15 清债验证、文档闭环与 Final Commit**。

### 关键约束
- 无 receipt 不清债
- 最终验证命令必须真实执行
- `src/INDEX.md` 和 `src/ARCHITECTURE.md` 必须同步
- docs ignore 必须处理

### 验收铁律
- Closure 文档存在
- 最终验证矩阵完整
- P0/P1 状态诚实
- git 状态清晰，提交范围明确

闭环启动，Day 15，执行。
