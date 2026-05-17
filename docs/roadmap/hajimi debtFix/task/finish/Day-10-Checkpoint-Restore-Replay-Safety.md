# Day 10 派单: Restore Checkpoint + Replay 安全闭环

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 10，是 Thinking UI / Checkpoint 修复中风险最高的一天。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: Restore Checkpoint + Replay 安全闭环
- **轰炸目标**: 为 `restore_checkpoint` 实现真实、安全、可回滚的恢复流程，并把 Session Replay 绑定到真实 checkpoint / trace 数据
- **任务性质**: 高风险功能开发 + 安全闭环
- **输入基线**: 完整技术背景见模块2
- **输出要求**: restore 前确认 + restore 前 backup + 安全路径 resolver + replay 真实数据 + 失败不破坏 workspace
- **通用铁律**:
  1. restore 写文件前必须用户确认
  2. restore 前必须导出 backup 或 dry-run plan
  3. restore 每个路径必须走 Day 2 resolver
  4. 失败中断不得留下半恢复且无记录
  5. Replay 必须绑定真实 trace/checkpoint，不使用模拟 timeline

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 前置工单 | Day 8 DTO + Day 9 export/compare | `THINKING-CHECKPOINT-PLAN.md`; `THINKING-CHECKPOINT-VERIFY.md` | 必须 |
| 后端 restore | 当前占位函数 | `src/interface/desktop/src/main.rs:1589` | 必须 |
| 前端 restore 调用 | checkpoint UI restore | `src/interface/web/app.js:4351` | 必须 |
| Replay UI | replay controls and events | `src/interface/web/app.js:4731-4842` | 必须 |
| 安全依赖 | Day 2 resolver | `rg -n "resolve_workspace_path|PathIntent" src/interface/desktop/src/main.rs` | 必须 |
| 验证命令 | Rust/JS 检查 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | 必须 |
| receipt | restore/replay 验证 | `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md` 或 roadmap debt 目录 | 必须 |

### 探索补充栏

本任务高风险。若无法证明 restore 原子性或备份可靠性，必须降级为 dry-run restore preview，并把真实 restore 标记为 `DEBT-THINKING-B10-RESTORE`，不得交付危险写入。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-10/15
- **角色**: Engineer
- **目标**: 让 checkpoint restore/replay 从占位变成安全可审计能力
- **输入**: Day 8/9 checkpoint 数据、`main.rs` restore、`app.js` replay
- **依赖关系**: 依赖 Day 9 export/compare；不得在 Day 8 之前执行

### 2）输出交付物

- **变更文件**:
  - `src/interface/desktop/src/main.rs` 或同层 checkpoint 模块
  - `src/interface/web/app.js`
  - `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md`，或 roadmap debt 目录
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`，如状态同步需要
- **核心修改点**:
  - `restore_checkpoint(id)` 真实读取 checkpoint
  - restore 前生成 backup 或 dry-run plan
  - restore 写入前后端双确认：前端 confirm，后端拒绝未确认参数
  - restore 路径全部走 resolver
  - replay 使用真实 trace/checkpoint event list
  - 失败路径写入 receipt
- **必须包含**:
  - `confirmRestore` 或等价参数
  - backup 路径和恢复策略
  - missing checkpoint / unsafe path / partial failure 的错误处理
  - replay event 数据来源说明
- **禁止包含**:
  - `restore_checkpoint(_id) -> Ok(())`
  - 无确认直接写文件
  - 不备份直接覆盖
  - replay 使用静态示例数据
  - restore 越过 workspace resolver
- **交付证明**:
  - `cargo check -p hajimi-desktop`
  - `node --check src/interface/web/app.js`
  - restore/replay receipt
  - `rg` 检查确认/backup/resolver

### 3）规模与复杂度观察

- **推荐目标**: 先实现可审计的 V1 restore，复杂原子事务可作为后续债务
- **复杂度说明**: 如果涉及多文件恢复和失败回滚，允许声明 `DEBT-COMPLEXITY-B10-001`，但必须保证不会静默破坏 workspace
- **禁止行为**: 因为原子性难就跳过 backup

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | Rust 格式通过 | `cargo fmt -- --check` 或 N/A | 返工 |
| LINT | JS 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| TEST | restore/replay receipt 存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter THINKING-RESTORE-REPLAY-VERIFY.md` | 返工 |
| ARCH | restore 走 resolver | `rg -n "restore_checkpoint|resolve_workspace_path|PathIntent" src/interface/desktop/src/main.rs` | 返工 |
| REAL | restore 不再固定成功 | `rg -n "fn restore_checkpoint|Ok\\(\\(\\)\\)" src/interface/desktop/src/main.rs` 并人工确认 | 返工 |
| DOC | 状态更新有证据 | 债务总表 diff 或 receipt | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `restore_checkpoint` 真实读取 checkpoint | `rg -n "fn restore_checkpoint|load.*checkpoint|read.*checkpoint" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-002 | restore 前确认存在 | `rg -n "confirm|confirmed|restore" src/interface/web/app.js src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-003 | restore 前 backup/dry-run 存在 | `rg -n "backup|dry_run|restore_plan" src/interface/desktop/src/main.rs src/interface/web/app.js` | [ ] |
| FUNC | FUNC-004 | replay 使用真实事件 | `rg -n "replayEvents|traceEvents|checkpoint" src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | restore 路径走 resolver | `rg -n "restore_checkpoint|resolve_workspace_path|PathIntent" src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-002 | missing checkpoint 报错 | receipt 或测试包含 missing id | [ ] |
| CONST | CONST-003 | unsafe path 报错 | receipt 或测试包含越界路径 | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 无确认拒绝 restore | 测试/receipt 证明 confirm false 被拒绝 | [ ] |
| NEG | NEG-002 | 无 backup 不写入 | 测试/receipt 证明 backup 失败时拒绝 | [ ] |
| NEG | NEG-003 | restore 失败不静默成功 | receipt 记录失败路径结果 | [ ] |
| NEG | NEG-004 | 不使用模拟 replay | `rg -n "mock|fake|sample|simulation" src/interface/web/app.js` 并人工确认无新增 | [ ] |
| UX | UX-001 | restore 提示清楚说明风险 | 截图/文案 | [ ] |
| UX | UX-002 | replay 上一条/下一条可用 | 手动 receipt | [ ] |
| E2E | E2E-001 | export/compare/restore/replay 链路可复现 | receipt 步骤 | [ ] |
| High | HIGH-001 | 失败不破坏 workspace | 手动或测试验证 backup/rollback | [ ] |

---

## 【模块3-B】地狱红线

1. restore 无确认写文件，返工
2. restore 无 backup/dry-run，返工
3. restore 不走 resolver，返工
4. `restore_checkpoint` 仍固定 `Ok(())`，返工
5. replay 使用模拟数据，返工
6. missing checkpoint 返回成功，返工
7. unsafe path 未拒绝，返工
8. `node --check` 失败，返工
9. `cargo check -p hajimi-desktop` 失败无说明，返工
10. 状态直接 `PARTIAL -> CLEARED` 但无完整 receipt，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | restore/replay 主路径是否可用 | [ ] | FUNC-001~004 | |
| RG | Day 8/9 契约是否保持 | [ ] | E2E-001 | |
| NG | 无确认、无 backup、unsafe path 是否拒绝 | [ ] | NEG-001~003 | |
| UX | restore 风险提示是否清楚 | [ ] | UX-001~002 | |
| E2E | 完整链路是否可复现 | [ ] | E2E-001 | |
| High | workspace 是否不被破坏 | [ ] | HIGH-001 | |
| 字段完整性 | receipt 是否含成功/失败路径 | [ ] | 文档 | |
| 需求映射 | 是否映射 `DEBT-THINKING-UI` | [ ] | 债务总表 7.1 | |
| 自测执行 | 是否跑 Rust/JS 检查 | [ ] | 质量闸门 | |
| 范围边界与债务 | 原子 restore 未完成是否声明 | [ ] | debt 声明 | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-10/15 完成并提交

### 提交信息
- Commit: `feat(interface/desktop): add safe checkpoint restore and replay binding`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/desktop/src/main.rs`
  - `src/interface/web/app.js`
  - `docs/debt/THINKING-RESTORE-REPLAY-VERIFY.md`

### 本轮目标与实际结果
- 目标: restore/replay 安全闭环
- 实际完成: `<列出确认、backup、resolver、replay 数据源>`
- 未完成/不在范围: `<如原子事务或 richer replay 延后，写明>`

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- restore/replay 手动测试: `<摘要>`

### 债务声明
- `DEBT-THINKING-B10-001`: `<如 restore 原子性/复杂 diff 延后>`

### 风险与回滚点
- 主要风险: restore 误覆盖用户文件
- 回滚方式: 使用本日 backup 恢复；代码回退用 `git restore src/interface/desktop/src/main.rs src/interface/web/app.js`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| SAFETY-001 | 不能保证 backup | 禁用写入 restore，仅交付 dry-run preview | 状态保持 `PARTIAL` |
| SAFETY-002 | resolver 无法覆盖某类路径 | 拒绝该路径 restore，写入 debt | 不越界 |
| QUALITY-001 | 多文件 restore 失败处理复杂 | 先实现单文件/小集合 V1，声明债务 | 有条件交付 |
| TEST-001 | GUI replay 无法手测 | 提供事件数据和 DOM 状态日志，标记实机待验 | 有条件交付 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 10 Restore Checkpoint + Replay 安全闭环**。

### 关键约束
- restore 前确认
- restore 前 backup 或 dry-run
- restore 路径走 resolver
- replay 使用真实事件

### 验收铁律
- Rust/JS 检查通过
- 成功和失败路径都有 receipt
- workspace 不被静默破坏
- Thinking UI 状态迁移诚实

闭环启动，Day 10，执行。
