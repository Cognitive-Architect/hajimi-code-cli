# Day 09 派单: Checkpoint Export / Compare + Operation Diff

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 9，处理 checkpoint `export_checkpoint` / `compare_checkpoints` 占位与 Operation Summary 虚拟 diff。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: Checkpoint Export / Compare + Operation Diff
- **轰炸目标**: 将 `export_checkpoint` 从固定 `{}` 改为真实 JSON/Markdown 输出，将 `compare_checkpoints` 从固定 `false` 改为可展示 diff summary，并让前端最小展示真实 compare/export 结果
- **任务性质**: 功能开发 + 半成品债务修复
- **输入基线**: 完整技术背景见模块2
- **输出要求**: export/compare 真实现 + 前端最小展示 + `THINKING-CHECKPOINT-VERIFY.md`
- **通用铁律**:
  1. 禁止固定 `{}`、固定 `false`、硬编码成功
  2. 找不到 checkpoint 必须返回明确错误
  3. diff summary 必须来自真实 checkpoint/file/git 数据
  4. Operation Summary 不能继续只展示虚拟摘要而无数据来源
  5. Day 9 不实现 restore 写入

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 前置工单 | Day 8 DTO/存储计划 | `docs/debt/THINKING-CHECKPOINT-PLAN.md` 或 roadmap debt 文档 | 必须 |
| 后端占位 | checkpoint commands | `src/interface/desktop/src/main.rs:1594-1599` | 必须 |
| 前端 checkpoint UI | list/export/restore 调用 | `src/interface/web/app.js:4320-4373` | 必须 |
| Operation Summary | diff preview 当前逻辑 | `src/interface/web/app.js:3144-3286` | 必须 |
| 相关数据 | agent-core checkpoint 能力 | `src/intelligence/agent-core/checkpoint.rs` | 必须 |
| 验证命令 | Rust/JS 检查 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | 必须 |
| receipt | Checkpoint 验证文档 | `docs/debt/THINKING-CHECKPOINT-VERIFY.md` 或 roadmap debt 目录 | 必须 |

### 探索补充栏

本任务为已知解实现，但 diff 数据来源可能有两种：checkpoint snapshot 对比或 git/file diff。若 Day 8 未落地存储，必须先完成可持久读取的最小 checkpoint store，不得继续返回占位值。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-09/15
- **角色**: Engineer
- **目标**: 让 checkpoint export/compare 产出真实可审计数据
- **输入**: Day 8 DTO、`main.rs` checkpoint commands、`app.js` checkpoint UI 与 Operation Summary
- **依赖关系**: 依赖 Day 8；Day 10 restore 依赖本日

### 2）输出交付物

- **变更文件**:
  - `src/interface/desktop/src/main.rs` 或同层 checkpoint 模块
  - `src/interface/web/app.js`
  - `docs/debt/THINKING-CHECKPOINT-VERIFY.md`，或 roadmap debt 目录
- **核心修改点**:
  - 实现 `export_checkpoint(id)`，支持单个 id 和必要时 `all`
  - 实现 `compare_checkpoints(id_a, id_b)`，返回 diff summary JSON，而非 bool
  - 前端展示 export/compare 错误与结果
  - Operation Summary diff preview 接真实 summary 或明确降级到“无 diff 数据”
- **必须包含**:
  - export JSON 包含 id/timestamp/files/metadata
  - compare JSON 包含 added/modified/deleted 或等价字段
  - 找不到 checkpoint 的错误路径
  - receipt 中包含一份示例 JSON
- **禁止包含**:
  - 固定 `{}` / 固定 `false`
  - 使用随机数据伪造 diff
  - Day 9 修改工作区文件进行 restore
  - 前端吞掉后端错误
- **交付证明**:
  - `rg -n "export_checkpoint|compare_checkpoints" src/interface/desktop/src/main.rs`
  - `cargo check -p hajimi-desktop`
  - `node --check src/interface/web/app.js`
  - verify 文档中的示例输出

### 3）规模与复杂度观察

- **推荐目标**: 最小可审计 export/compare，不追求完整时间旅行 UI
- **复杂度说明**: 如果 compare 需要复杂 diff 算法，可先用文件列表/哈希/内容摘要实现 V1，并声明后续 richer diff
- **禁止行为**: 为了展示 UI 而生成假 diff

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | Rust 格式通过 | `cargo fmt -- --check` 或 N/A | 返工 |
| LINT | JS 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| TEST | export/compare 结果可复现 | 单测或 receipt 示例 JSON | 返工 |
| ARCH | 不破坏分层 | `cargo check --workspace` 或说明外部错误 | 返工 |
| REAL | 占位值消失 | `rg -n "Ok\\(\"\\{\\}\"\\)|Ok\\(false\\)" src/interface/desktop/src/main.rs` | 返工 |
| DOC | verify 文档存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter THINKING-CHECKPOINT-VERIFY.md` | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `export_checkpoint` 返回真实数据 | `rg -n "fn export_checkpoint|export_checkpoint" src/interface/desktop/src/main.rs` + 示例 JSON | [ ] |
| FUNC | FUNC-002 | `compare_checkpoints` 返回 diff summary | `rg -n "fn compare_checkpoints|Diff|modified|added|deleted" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-003 | 前端 export 调用可下载真实 JSON | `rg -n "export_checkpoint|checkpoint_.*json" src/interface/web/app.js` | [ ] |
| FUNC | FUNC-004 | 前端 compare 展示最小结果 | `rg -n "compare_checkpoints|checkpoint.*compare|diff" src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | 找不到 checkpoint 明确报错 | 单测或 receipt 记录 missing id | [ ] |
| CONST | CONST-002 | export 包含 metadata | 示例 JSON 包含 `metadata` | [ ] |
| CONST | CONST-003 | compare 包含变更分类 | 示例 JSON 包含 added/modified/deleted 或等价字段 | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 固定 `{}` 消失 | `rg -n "Ok\\(\"\\{\\}\"\\)" src/interface/desktop/src/main.rs` 无命中 | [ ] |
| NEG | NEG-002 | 固定 `false` 消失 | `rg -n "Ok\\(false\\)" src/interface/desktop/src/main.rs` 无命中 | [ ] |
| NEG | NEG-003 | 不执行 restore | `git diff -- src/interface/desktop/src/main.rs` 人工确认无 restore 写入逻辑 | [ ] |
| NEG | NEG-004 | 错误不被前端吞掉 | `rg -n "catch|showToast|checkpoint.*失败" src/interface/web/app.js` | [ ] |
| UX | UX-001 | export 失败提示可读 | 手动/receipt | [ ] |
| UX | UX-002 | compare 结果可读 | 手动/receipt | [ ] |
| E2E | E2E-001 | 创建/导出/比较链路可复现 | receipt 步骤 | [ ] |
| High | HIGH-001 | Operation Summary 不再伪 diff | `rg -n "renderOperationDiffPreview|operation_summary" src/interface/web/app.js` + 数据来源说明 | [ ] |

---

## 【模块3-B】地狱红线

1. `export_checkpoint` 仍固定 `{}`，返工
2. `compare_checkpoints` 仍固定 `false`，返工
3. 生成随机/假 diff，返工
4. 找不到 checkpoint 返回成功，返工
5. Day 9 实现 restore 写文件，返工
6. 前端下载空 JSON 却显示成功，返工
7. `node --check` 失败，返工
8. `cargo check -p hajimi-desktop` 失败无说明，返工
9. verify 文档没有示例输出，返工
10. Operation Summary 继续无数据来源却宣称真实，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | export/compare 是否真实现 | [ ] | FUNC-001~004 | |
| RG | 占位返回是否消失 | [ ] | NEG-001~002 | |
| NG | missing id 是否覆盖 | [ ] | CONST-001 | |
| UX | 前端结果/错误是否可读 | [ ] | UX-001~002 | |
| E2E | 链路是否可复现 | [ ] | E2E-001 | |
| High | diff 来源是否真实 | [ ] | HIGH-001 | |
| 字段完整性 | 示例 JSON 是否完整 | [ ] | verify 文档 | |
| 需求映射 | 是否映射 `DEBT-THINKING-UI` | [ ] | 债务总表 7.1 | |
| 自测执行 | 是否跑 Rust/JS 检查 | [ ] | 质量闸门 | |
| 范围边界与债务 | restore 是否延后 | [ ] | NEG-003 | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-09/15 完成并提交

### 提交信息
- Commit: `feat(interface/desktop): implement checkpoint export and compare`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/desktop/src/main.rs`
  - `src/interface/web/app.js`
  - `docs/debt/THINKING-CHECKPOINT-VERIFY.md`

### 本轮目标与实际结果
- 目标: export/compare 真实现
- 实际完成: `<列出接口、返回格式、示例输出>`
- 未完成/不在范围: restore/replay 属 Day 10

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- `rg -n "Ok(\"{}\")|Ok(false)" ...`: `<摘要>`

### 债务声明
- `DEBT-THINKING-B09-001`: `<如 diff 粒度仅为 V1 summary，写后续 richer diff>`

### 风险与回滚点
- 主要风险: checkpoint 数据格式后续变更
- 回滚方式: `git restore src/interface/desktop/src/main.rs src/interface/web/app.js`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | Day 8 DTO 不足以 compare | 补 DTO 版本字段，不做假 diff | 返工 |
| QUALITY-001 | export/compare 编译失败 | 停止前端 UI，先修后端 | 返工 |
| TEST-001 | 无现成 checkpoint 样本 | 构造最小真实样本文件，不用随机假数据 | 有条件交付 |
| SAFETY-001 | 需要读取越界路径 | 必须走 Day 2 resolver 或拒绝 | 不交付越界能力 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 09 Checkpoint Export / Compare + Operation Diff**。

### 关键约束
- 不允许 `{}` / `false` 假实现
- 不实现 restore
- diff 必须有真实数据来源
- verify 文档必须含示例 JSON

### 验收铁律
- export/compare 命令真实返回
- Rust/JS 检查通过
- 前端错误可读
- Operation Summary 不再伪称真实 diff

闭环启动，Day 09，执行。
