# Day 12 派单: Agent Prompt Golden Task Regression

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 12，为 Day 11 契约建立不依赖真实 LLM 的 golden regression。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: Agent Prompt Golden Task Regression
- **轰炸目标**: 建立 `tests/agent_prompt_golden/` golden case 集，覆盖 Planner、Reflector、ToolCall 的 success/failure/unknown/stop-loss 场景，并尽量接入现有 agent-core 单元测试
- **任务性质**: 测试建设 + 质量增强
- **输入基线**: 完整技术背景见模块2
- **输出要求**: Golden cases + 可执行或可人工复核的 regression harness + agent-core 测试不破坏
- **通用铁律**:
  1. Golden cases 不依赖真实 LLM 或网络
  2. 不使用 mock 成功掩盖 schema 错误
  3. 每个 case 必须映射 Day 11 契约
  4. 如果暂不能接入自动测试，必须提供可复核文档和 `DEBT-TEST`
  5. 不改 prompt 运行逻辑来迎合样例

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 前置文档 | Day 11 五份契约 | `Get-ChildItem docs/agent-prompt-core -Filter *.md` | 必须 |
| Planner DTO | schema 与测试参考 | `src/intelligence/agent-core/planner_dto.rs` | 必须 |
| Reflector DTO | schema 与测试参考 | `src/intelligence/agent-core/reflector_dto.rs` | 必须 |
| Tool Manifest | 工具过滤与 schema | `src/intelligence/agent-core/tool_manifest.rs` | 必须 |
| ActExecutor | ToolCall 与黑板协议 | `src/intelligence/agent-core/act_executor.rs` | 必须 |
| 输出目录 | golden cases | `tests/agent_prompt_golden/` | 必须 |
| 验证命令 | agent-core 测试 | `cargo test -p intelligence-agent-core --lib` | 必须或记录 blocker |

### 探索补充栏

如果现有测试结构不适合直接读取 `tests/agent_prompt_golden/`，允许先以 JSON/Markdown case + Rust 单元测试内联样例形式交付，但必须声明如何后续接入统一 harness。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-12/15
- **角色**: Engineer
- **目标**: 用 golden cases 固定 Agent Prompt V2 的最低行为边界
- **输入**: Day 11 契约、planner/reflector/tool manifest/act executor 当前代码
- **依赖关系**: 依赖 Day 11；不依赖 Day 13+

### 2）输出交付物

- **变更文件**:
  - `tests/agent_prompt_golden/README.md`
  - `tests/agent_prompt_golden/planner/*.json` 或 `.md`
  - `tests/agent_prompt_golden/reflector/*.json` 或 `.md`
  - `tests/agent_prompt_golden/toolcall/*.json` 或 `.md`
  - `src/intelligence/agent-core/*`，仅当接入无网络单元测试需要
- **核心修改点**:
  - Planner 至少 5 个样例：修 bug、搜索、读文件、写文件、ask user
  - Reflector 至少 5 个样例：success、failure、unknown、retry、stop-loss
  - ToolCall 至少 3 个样例：safe read、risky write、cannot act
  - 如接入测试，覆盖 schema deserialize/validation，不调用真实 LLM
- **必须包含**:
  - 每个 case 有输入、期望输出、契约映射、失败原因
  - README 说明如何运行验证
  - `cargo test -p intelligence-agent-core --lib` 输出摘要
- **禁止包含**:
  - 真实 LLM API 调用
  - 网络依赖
  - snapshot 中硬编码“所有通过”而不校验字段
  - 大幅改 planner/reflector runtime
- **交付证明**:
  - `Get-ChildItem -Recurse tests/agent_prompt_golden`
  - `cargo test -p intelligence-agent-core --lib`
  - `rg -n "stop-loss|unknown|ask_user|safe_read|risky_write" tests/agent_prompt_golden`

### 3）规模与复杂度观察

- **推荐目标**: cases 可读、schema 可验证，harness 简单
- **复杂度说明**: 若接入 Rust 测试需要较多 infra，先交付 case + README + `DEBT-TEST-B12-001`
- **禁止行为**: 为了让 golden 通过而改弱 schema 校验

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | agent-core 编译/测试通过 | `cargo test -p intelligence-agent-core --lib` | 返工或记录外部错误 |
| FMT | Rust 格式通过，如改 Rust | `cargo fmt -- --check` 或 N/A | 返工 |
| LINT | JSON/Markdown 可搜索 | `rg -n "contract|expected|schema_version" tests/agent_prompt_golden` | 返工 |
| TEST | cases 数量达标 | `Get-ChildItem -Recurse tests/agent_prompt_golden -File` | 返工 |
| ARCH | 不依赖真实 LLM | `rg -n "OPENAI|ANTHROPIC|api_key|http" tests/agent_prompt_golden src/intelligence/agent-core` 并人工确认无新增 | 返工 |
| REAL | schema 字段真实校验 | 测试或 README 说明校验方式 | 返工 |
| DOC | README 存在 | `Test-Path tests/agent_prompt_golden/README.md` | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | Planner 5 cases | `Get-ChildItem tests/agent_prompt_golden/planner -File` 数量 >= 5 | [ ] |
| FUNC | FUNC-002 | Reflector 5 cases | `Get-ChildItem tests/agent_prompt_golden/reflector -File` 数量 >= 5 | [ ] |
| FUNC | FUNC-003 | ToolCall 3 cases | `Get-ChildItem tests/agent_prompt_golden/toolcall -File` 数量 >= 3 | [ ] |
| FUNC | FUNC-004 | README 说明运行方式 | `rg -n "Run|运行|cargo test|验证" tests/agent_prompt_golden/README.md` | [ ] |
| CONST | CONST-001 | Planner schema version 出现 | `rg -n "PlannerSubgoalPlanV1" tests/agent_prompt_golden` | [ ] |
| CONST | CONST-002 | Reflector schema version 出现 | `rg -n "ReflectorCritiqueV1" tests/agent_prompt_golden` | [ ] |
| CONST | CONST-003 | stop-loss case 存在 | `rg -n "stop-loss|stop_loss|Stop-Loss" tests/agent_prompt_golden` | [ ] |
| CONST | CONST-004 | agent-core 测试通过 | `cargo test -p intelligence-agent-core --lib` | [ ] |
| NEG | NEG-001 | 不依赖真实 LLM | `rg -n "api_key|OPENAI|ANTHROPIC|http" tests/agent_prompt_golden` 无真实调用 | [ ] |
| NEG | NEG-002 | unknown/failure case 存在 | `rg -n "unknown|failure|失败" tests/agent_prompt_golden` | [ ] |
| NEG | NEG-003 | risky write case 存在 | `rg -n "risky_write|risky write|write" tests/agent_prompt_golden/toolcall` | [ ] |
| NEG | NEG-004 | cannot act case 存在 | `rg -n "cannot_act|cannot act|无法执行" tests/agent_prompt_golden/toolcall` | [ ] |
| UX | UX-001 | case 可读 | README 说明 case 字段 | [ ] |
| UX | UX-002 | 契约映射明确 | `rg -n "AGENT-PERSONA|PLANNER-PROMPT-CONTRACT|REFLECTOR-CONTRACT|EXECUTOR-CONTRACT" tests/agent_prompt_golden` | [ ] |
| E2E | E2E-001 | 至少一条 harness 或手动验证闭环 | README 或 Rust test 输出 | [ ] |
| High | HIGH-001 | 不弱化运行时 schema | `git diff -- src/intelligence/agent-core` 人工确认 | [ ] |

---

## 【模块3-B】地狱红线

1. Golden cases 依赖真实 LLM，返工
2. case 数量不达标且无 debt，返工
3. 不覆盖 failure/unknown/stop-loss，返工
4. 没有 README，返工
5. 为了测试通过弱化 DTO 校验，返工
6. `cargo test -p intelligence-agent-core --lib` 失败无说明，返工
7. case 没有映射 Day 11 契约，返工
8. 网络/API key 字段进入测试执行路径，返工
9. 只写样例不说明如何验证，返工
10. Prompt 债务直接标 `CLEARED`，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | Planner/Reflector/ToolCall cases 是否齐全 | [ ] | FUNC-001~003 | |
| RG | 是否不破坏 agent-core 测试 | [ ] | CONST-004 | |
| NG | unknown/failure/stop-loss 是否覆盖 | [ ] | NEG-002, CONST-003 | |
| UX | cases 是否可读可维护 | [ ] | UX-001~002 | |
| E2E | 是否有运行方式 | [ ] | E2E-001 | |
| High | 是否无真实 LLM/network | [ ] | NEG-001 | |
| 字段完整性 | case 是否有 input/expected/mapping | [ ] | README | |
| 需求映射 | 是否映射 Agent Prompt 债 | [ ] | Day 11 契约 | |
| 自测执行 | 是否跑测试或写 blocker | [ ] | BUILD 闸门 | |
| 范围边界与债务 | harness 未完成是否声明 | [ ] | `DEBT-TEST` | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-12/15 完成并提交

### 提交信息
- Commit: `test(intelligence/agent-core): add prompt golden regression cases`
- 分支: `<实际分支>`
- 变更文件:
  - `tests/agent_prompt_golden/**`
  - `<Rust test 文件，如有>`

### 本轮目标与实际结果
- 目标: 建立 Prompt Golden Regression
- 实际完成: `<列出 case 数量和 harness 状态>`
- 未完成/不在范围: 真实 LLM eval 不在本日

### 自动化质量检查报告
- `cargo test -p intelligence-agent-core --lib`: `<摘要>`
- `Get-ChildItem -Recurse tests/agent_prompt_golden`: `<摘要>`
- `rg schema/stop-loss`: `<摘要>`

### 债务声明
- `DEBT-TEST-B12-001`: `<如未接入自动 harness>`

### 风险与回滚点
- 主要风险: golden cases 与 runtime 演进不同步
- 回滚方式: 回退 `tests/agent_prompt_golden/**` 和本日测试改动
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| TEST-001 | 无法接入 Rust harness | 交付 case + README + `DEBT-TEST` | 有条件交付 |
| QUALITY-001 | agent-core 测试非本次失败 | 记录失败测试，不扩修 | 有条件交付 |
| ARCH-001 | cases 需要 runtime 改造才成立 | 修改 case 或标 future，不改 runtime | 防止伪契约 |
| REAL-001 | 样例字段和 DTO 不匹配 | 以 DTO 为准修样例 | 返工 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 12 Agent Prompt Golden Task Regression**。

### 关键约束
- 不依赖真实 LLM
- cases 必须映射 Day 11 契约
- 覆盖 success/failure/unknown/stop-loss
- 不弱化运行时 schema

### 验收铁律
- Planner >=5，Reflector >=5，ToolCall >=3
- README 存在
- agent-core 测试通过或 blocker 诚实
- Prompt 债保持质量增强状态，不伪清偿

闭环启动，Day 12，执行。
