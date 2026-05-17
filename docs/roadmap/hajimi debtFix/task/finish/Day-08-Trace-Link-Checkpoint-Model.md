# Day 08 派单: Trace 链路验收 + Checkpoint 数据模型

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 8，是 `DEBT-THINKING-UI` 从 `PARTIAL` 走向真实闭环的第一步。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Architect + Engineer 合并执行）
- **任务名称**: Trace 链路验收 + Checkpoint 数据模型
- **轰炸目标**: 验证 `trace_tx` 到前端 `agent:trace` 的真实链路，设计并落地最小 Checkpoint DTO/存储位置，不实现高风险 restore
- **任务性质**: 探索调研 + 最小功能开发
- **输入基线**: 完整技术背景见模块2
- **输出要求**: Trace 链路 receipt + Checkpoint DTO 草案/代码 + `THINKING-CHECKPOINT-PLAN.md`
- **通用铁律**:
  1. 不允许继续使用“看起来像 trace”的模拟数据伪装真实链路
  2. Checkpoint DTO 必须能支撑 Day 9 export/compare
  3. 存储位置不得随意写项目根目录
  4. Day 8 不实现 restore
  5. 所有未知必须写入计划文档，不硬编码成功

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `DEBT-THINKING-UI` 当前 `PARTIAL / P1-P2` | 债务总表第 7.1 节 | 必须 |
| 后端 trace | `subscribe_agent_trace` 与 `trace_tx` | `src/interface/desktop/src/main.rs:1479`, `:1864-1908` | 必须 |
| 前端 trace | `agent:trace`, trace panel, operation summary | `src/interface/web/app.js:1920-2041`, `:3144-3286` | 必须 |
| checkpoint 占位 | restore/export/compare 当前占位 | `src/interface/desktop/src/main.rs:1589-1599` | 必须 |
| Agent checkpoint | intelligence 层已有 checkpoint 能力 | `src/intelligence/agent-core/checkpoint.rs`; `src/intelligence/agent-core/workflow_orchestrator.rs:118-146` | 必须 |
| 验证命令 | Rust/JS 基础检查 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | 必须 |
| 文档输出 | Checkpoint 计划 | `docs/debt/THINKING-CHECKPOINT-PLAN.md` 或 roadmap debt 目录 | 必须 |

### 探索补充栏

| 项目 | 内容 |
|---|---|
| 已知事实 | trace 注入已出现，checkpoint desktop commands 仍有占位 |
| 待确认问题 | 前端是否收到真实后端事件；checkpoint 数据源选 desktop app data 还是 agent-core checkpoint；diff 数据粒度 |
| 预期输出 | Trace 验收证据、DTO 字段定义、存储位置和 Day 9/10 接口契约 |
| 停止条件 | Day 9 可以按 DTO 实现 export/compare；restore 风险被明确延后 |

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-08/15
- **角色**: Architect / Engineer
- **目标**: 让 Thinking UI 的 trace/checkpoint 有真实数据契约
- **输入**: `main.rs` trace/checkpoint、`app.js` trace/replay、agent-core checkpoint
- **依赖关系**: 建议在 Day 7 UX 验收后执行；Day 9 依赖本 DTO

### 2）输出交付物

- **变更文件**:
  - `src/interface/desktop/src/main.rs` 或新增同层模块
  - `docs/debt/THINKING-CHECKPOINT-PLAN.md`，或 roadmap debt 目录
  - `src/interface/web/app.js`，仅当 trace 接收需要最小修复
- **核心修改点**:
  - 验证 `subscribe_agent_trace` 是否发出真实事件
  - 定义 Checkpoint DTO，建议字段：`id`, `timestamp`, `label`, `files`, `diff_summary`, `trace_event_ids`, `metadata`
  - 明确 checkpoint 存储位置：workspace `.hajimi/checkpoints` 或 Tauri app data，必须写明理由
  - 为 Day 9 `export_checkpoint` / `compare_checkpoints` 定义返回格式
- **必须包含**:
  - trace receipt：事件来源、触发方式、前端接收结果
  - DTO 字段说明与 JSON 示例
  - restore 风险说明：Day 8 不实现 restore
- **禁止包含**:
  - `restore_checkpoint` 假实现继续返回 `Ok(())` 后声称完成
  - `export_checkpoint` 返回固定 `{}` 作为新实现
  - `compare_checkpoints` 返回固定 `false` 作为新实现
  - 存储到项目根目录无说明
- **交付证明**:
  - `rg -n "subscribe_agent_trace|trace_tx|agent:trace" ...`
  - `cargo check -p hajimi-desktop`
  - `node --check src/interface/web/app.js`
  - `THINKING-CHECKPOINT-PLAN.md`

### 3）规模与复杂度观察

- **推荐目标**: 先定清 DTO 与 trace 证据，不把 Day 9/10 全部吃掉
- **复杂度说明**: 如果接入 agent-core checkpoint 需要跨层改造，先记录方案，不做半成品桥接
- **禁止行为**: 用模拟 trace 或随机 checkpoint 数据凑 UI

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | Rust 格式通过 | `cargo fmt -- --check` 或 N/A | 返工 |
| LINT | JS 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| TEST | Trace/DTO receipt 存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter THINKING-CHECKPOINT-PLAN.md` | 返工 |
| ARCH | 存储位置符合 local-first | 计划文档写明 workspace/app data 选择 | 返工 |
| REAL | 不新增假 checkpoint 成功 | `rg -n "Ok\\(\\(\\)\\)|Ok\\(\"\\{\\}\"\\)|Ok\\(false\\)" src/interface/desktop/src/main.rs` 并确认占位未被宣称完成 | 返工 |
| DOC | Day 9/10 契约清晰 | 计划文档包含 export/compare/restore 分节 | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `subscribe_agent_trace` 入口已确认 | `rg -n "subscribe_agent_trace|trace_tx" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-002 | 前端 `agent:trace` 接收已确认 | `rg -n "agent:trace|traceEvents|renderTrace" src/interface/web/app.js` | [ ] |
| FUNC | FUNC-003 | Checkpoint DTO 字段落地 | `rg -n "Checkpoint.*Dto|CheckpointRecord|checkpoint" src/interface/desktop/src/main.rs docs` | [ ] |
| FUNC | FUNC-004 | 存储位置已明确 | `rg -n "checkpoint.*store|\\.hajimi|app data|AppData" docs src/interface/desktop/src/main.rs` | [ ] |
| CONST | CONST-001 | Day 9 export 契约明确 | `rg -n "export_checkpoint" docs` | [ ] |
| CONST | CONST-002 | Day 9 compare 契约明确 | `rg -n "compare_checkpoints" docs` | [ ] |
| CONST | CONST-003 | Day 10 restore 风险明确 | `rg -n "restore_checkpoint|确认|backup|dry-run" docs` | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 不实现不安全 restore | `git diff -- src/interface/desktop/src/main.rs` 人工确认 restore 未被假完成 | [ ] |
| NEG | NEG-002 | 不新增固定 `{}` export | `rg -n "Ok\\(\"\\{\\}\"\\)" src/interface/desktop/src/main.rs` | [ ] |
| NEG | NEG-003 | 不新增固定 `false` compare | `rg -n "Ok\\(false\\)" src/interface/desktop/src/main.rs` | [ ] |
| NEG | NEG-004 | 不写项目根目录 | 计划文档说明存储路径 | [ ] |
| UX | UX-001 | trace panel 有真实来源说明 | receipt 记录真实事件或 blocker | [ ] |
| UX | UX-002 | checkpoint label/timestamp 可展示 | DTO 示例包含 label/timestamp | [ ] |
| E2E | E2E-001 | Trace 到前端链路有证据 | 触发 agent 后事件截图/日志 | [ ] |
| High | HIGH-001 | Restore 明确延后 | 计划文档写“Day 8 不 restore” | [ ] |

---

## 【模块3-B】地狱红线

1. 用模拟 trace 当真实事件，返工
2. Checkpoint DTO 没有存储位置，返工
3. 继续固定 `{}`/`false` 却声称完成 checkpoint，返工
4. Day 8 实现 restore，返工
5. 不写计划文档，返工
6. 跳过前端 trace 接收验证，返工
7. `cargo check -p hajimi-desktop` 失败无说明，返工
8. `node --check` 失败仍收卷，返工
9. 跨层依赖破坏四层架构，返工
10. Day 9 无法基于 DTO 继续，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | trace 和 DTO 是否明确 | [ ] | FUNC-001~004 | |
| RG | 占位函数是否未被伪清偿 | [ ] | NEG-001~003 | |
| NG | restore 风险是否阻断 | [ ] | HIGH-001 | |
| UX | 前端展示字段是否够用 | [ ] | UX-001~002 | |
| E2E | trace 链路是否有证据 | [ ] | E2E-001 | |
| High | 存储/restore 是否安全 | [ ] | CONST-003, NEG-004 | |
| 字段完整性 | DTO 是否有 JSON 示例 | [ ] | 文档 | |
| 需求映射 | 是否映射 `DEBT-THINKING-UI` | [ ] | 债务总表 7.1 | |
| 自测执行 | 是否跑 Rust/JS 检查 | [ ] | 质量闸门 | |
| 范围边界与债务 | Day 9/10 未做项是否声明 | [ ] | 计划文档 | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-08/15 完成并提交

### 提交信息
- Commit: `feat(interface/desktop): define checkpoint model and trace validation plan`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/desktop/src/main.rs`
  - `docs/debt/THINKING-CHECKPOINT-PLAN.md`
  - `<app.js 如有>`

### 本轮目标与实际结果
- 目标: 验证 trace 链路并定义 checkpoint DTO
- 实际完成: `<列出 trace 证据、DTO、存储选择>`
- 未完成/不在范围: export/compare 属 Day 9；restore/replay 属 Day 10

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- `rg trace/checkpoint`: `<摘要>`

### 债务声明
- `DEBT-THINKING-B08-001`: `<如 trace 实机未验，写 blocker>`

### 风险与回滚点
- 主要风险: DTO 过早固化导致 Day 9/10 重做
- 回滚方式: 回退 DTO 代码和计划文档，保留债务 `PARTIAL`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | agent-core checkpoint 不能直接给 desktop 用 | 记录适配层方案，不硬接 | Day 9 先做 desktop-local |
| QUALITY-001 | trace 事件无真实来源 | 停止 UI 美化，先修链路或记录 blocker | 不清债 |
| TEST-001 | 无法触发真实 agent trace | 用订阅通道单测/日志替代，并写手动待验 | 有条件交付 |
| SAFETY-001 | restore 被要求提前实现 | 拒绝提前实现，写入 Day 10 | 保持安全边界 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 08 Trace 链路验收 + Checkpoint 数据模型**。

### 关键约束
- 不做 restore
- 不造假 trace
- DTO 要支撑 Day 9/10
- 存储位置必须明确

### 验收铁律
- `THINKING-CHECKPOINT-PLAN.md` 存在
- Rust/JS 检查通过
- trace 链路有证据或 blocker
- checkpoint 占位不得被伪清偿

闭环启动，Day 08，执行。
