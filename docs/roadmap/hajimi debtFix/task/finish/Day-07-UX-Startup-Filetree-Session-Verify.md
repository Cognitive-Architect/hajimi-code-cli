# Day 07 派单: 启动 / 文件树 / 会话持久化实机验收

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 7，用 receipt 闭合 `DEBT-UX-AGENT-001` 的 `VERIFY` 状态。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: 启动 / 文件树 / 会话持久化实机验收
- **轰炸目标**: 本地启动 Tauri 应用，验证启动 toast、文件树加载、Day 3 文件操作、新会话/切换/重启恢复，并将结果写入 `UX-FILETREE-SESSION-VERIFY.md`
- **任务性质**: QA 验收 + 小修复
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 本地验收 receipt + 必要小修 + 债务状态基于证据迁移
- **通用铁律**:
  1. 没有实机 receipt 不得把 UX 债标记为 `CLEARED`
  2. 本日只允许启动、文件树、会话持久化相关小修
  3. 必须复验 Day 3 新建/重命名/删除
  4. 会话持久化必须覆盖关闭重开
  5. 如果 Tauri dev 无法启动，必须记录环境和错误，不伪造通过

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `DEBT-UX-AGENT-001` 当前 `VERIFY / P1` | 债务总表第 6.1 节 | 必须 |
| 前端启动路径 | `initWorkspace`, `loadFileTree`, session 初始化 | `src/interface/web/app.js:66`, `:815-861`, `:3340-3422` | 必须 |
| 后端 workspace | `get_current_workspace`, file commands | `rg -n "get_current_workspace|read_file|list_dir|create_dir|rename_path|delete_path" src/interface/desktop/src/main.rs` | 必须 |
| 文件操作前置 | Day 3 专用 commands 已接入 | `rg -n "create_dir|rename_path|delete_path" src/interface/web/app.js src/interface/desktop/src/main.rs` | 必须 |
| 验证命令 | Rust/JS 基础检查 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | 必须 |
| 实机命令 | Tauri dev 启动 | `cd src/interface/desktop; cargo tauri dev` | 必须或记录 blocker |
| receipt | UX 验收文档 | `docs/debt/UX-FILETREE-SESSION-VERIFY.md` 或 roadmap debt 目录 | 必须 |

### 探索补充栏

本任务以验收为主。若发现启动/会话 bug，只做最小修复；如果问题超出本日范围，写入 `DEBT-UX-B07-*`，不得扩大到前端模块化重构。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-07/15
- **角色**: Engineer
- **目标**: 将 UX `VERIFY` 从“代码看着有”变为“本地有证据”
- **输入**: `app.js` workspace/session 相关函数、Day 3 file ops、债务总表 6.1
- **依赖关系**: 依赖 Day 3 文件操作；建议在 Day 6 后执行

### 2）输出交付物

- **变更文件**:
  - `docs/debt/UX-FILETREE-SESSION-VERIFY.md`，或 `docs/roadmap/hajimi debtFix/debt/UX-FILETREE-SESSION-VERIFY.md`
  - `src/interface/web/app.js`，仅当验收失败需要小修
  - `src/interface/desktop/src/main.rs`，仅当 workspace 启动路径需要小修
- **核心修改点**:
  - 记录启动无异常 toast / 异常 toast
  - 验证文件树显示 workspace
  - 验证新建文件夹、重命名、删除
  - 验证会话 A/B 切换和关闭重开持久化
  - 根据结果更新债务状态为 `CLEARED` 或保持 `VERIFY/OPEN`
- **必须包含**:
  - 命令输出摘要
  - 截图路径或文字复现步骤与结果
  - localStorage key `hajimi_chat_sessions` 的结果说明
- **禁止包含**:
  - 无实测截图/日志就清债
  - 借 UX 验收重构 `app.js`
  - 跳过文件操作复验
  - 把 Tauri 启动失败写成“环境问题，不影响通过”
- **交付证明**:
  - UX receipt 文档
  - `cargo check -p hajimi-desktop`
  - `node --check src/interface/web/app.js`

### 3）规模与复杂度观察

- **推荐目标**: 验收优先，小修限于启动路径和会话状态
- **复杂度说明**: 若会话持久化需要重写数据模型，触发 `DEBT-COMPLEXITY-B07-001`，拆到后续
- **禁止行为**: 用清空 localStorage 掩盖会话恢复 bug

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | 若改 Rust 则格式通过 | `cargo fmt -- --check` 或 N/A | 返工 |
| LINT | JS 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| TEST | 实机 receipt 存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter UX-FILETREE-SESSION-VERIFY.md` | 返工 |
| ARCH | 不做前端大重构 | `git diff --stat` 人工确认范围 | 返工 |
| REAL | 关闭重开真实验证 | receipt 中必须有 restart step | 返工 |
| DOC | 债务状态更新有证据 | 债务总表 diff 或 receipt | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | Tauri dev 能启动或 blocker 记录 | `cd src/interface/desktop; cargo tauri dev` receipt | [ ] |
| FUNC | FUNC-002 | 文件树加载成功 | 截图/日志显示 workspace 文件树 | [ ] |
| FUNC | FUNC-003 | 会话 A/B 可切换 | receipt 记录 A/B 消息切换结果 | [ ] |
| FUNC | FUNC-004 | 关闭重开后会话保留 | receipt 记录 restart 后 localStorage/session list | [ ] |
| CONST | CONST-001 | Day 3 新建文件夹复验 | 手动步骤 + 结果 | [ ] |
| CONST | CONST-002 | Day 3 重命名复验 | 手动步骤 + 结果 | [ ] |
| CONST | CONST-003 | Day 3 删除确认复验 | 手动步骤 + 结果 | [ ] |
| CONST | CONST-004 | JS 语法通过 | `node --check src/interface/web/app.js` | [ ] |
| NEG | NEG-001 | 启动失败不伪通过 | 若失败，receipt 有错误日志和状态保持 `OPEN/VERIFY` | [ ] |
| NEG | NEG-002 | 文件树失败 toast 可读 | 如触发失败，记录 toast 文案 | [ ] |
| NEG | NEG-003 | 会话损坏数据可恢复或记录 | 手动写异常 localStorage 或记录未覆盖 debt | [ ] |
| NEG | NEG-004 | 删除无确认不得通过 | receipt 证明删除前确认 | [ ] |
| UX | UX-001 | 启动无异常 toast | 截图/日志 | [ ] |
| UX | UX-002 | 会话列表用户可理解 | 截图/文字说明 | [ ] |
| E2E | E2E-001 | 从启动到会话重启完整链路 | receipt 串联步骤 1-6 | [ ] |
| High | HIGH-001 | UX 状态迁移有证据 | 债务总表或 receipt 写明 `VERIFY -> CLEARED/OPEN` 原因 | [ ] |

---

## 【模块3-B】地狱红线

1. 没有 Tauri 实机验证或 blocker 却清债，返工
2. 跳过关闭重开，会话持久化不算通过，返工
3. 跳过 Day 3 文件操作复验，返工
4. 为了通过验收清空历史数据，返工
5. 大规模重构 `app.js`，返工
6. `node --check` 失败仍收卷，返工
7. `cargo check -p hajimi-desktop` 失败无说明，返工
8. 截图/日志路径缺失，返工
9. 状态直接 `VERIFY -> CLEARED` 但 receipt 不完整，返工
10. 混入 Thinking/Checkpoint 修复，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 启动、文件树、会话是否覆盖 | [ ] | FUNC-001~004 | |
| RG | Day 3 文件操作是否复验 | [ ] | CONST-001~003 | |
| NG | 失败路径是否诚实记录 | [ ] | NEG-001~003 | |
| UX | 主路径是否有截图/日志 | [ ] | UX-001~002 | |
| E2E | 启动到重启是否完整 | [ ] | E2E-001 | |
| High | 状态迁移是否有证据 | [ ] | HIGH-001 | |
| 字段完整性 | receipt 是否含日期、分支、命令 | [ ] | 文档 | |
| 需求映射 | 是否映射 `DEBT-UX-AGENT-001` | [ ] | 债务总表 6.1 | |
| 自测执行 | 是否实机跑过 | [ ] | Tauri dev receipt | |
| 范围边界与债务 | 未修问题是否成债 | [ ] | `DEBT-UX-B07-*` | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-07/15 完成并提交

### 提交信息
- Commit: `test(interface/desktop): record ux startup and session verification`
- 分支: `<实际分支>`
- 变更文件:
  - `docs/debt/UX-FILETREE-SESSION-VERIFY.md`
  - `<小修文件，如有>`

### 本轮目标与实际结果
- 目标: 验收启动/文件树/会话持久化
- 实际完成: `<列出通过/失败步骤>`
- 未完成/不在范围: Thinking UI/Checkpoint 属 Day 8-10

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- `cargo tauri dev`: `<启动摘要或 blocker>`

### 债务声明
- `DEBT-UX-B07-001`: `<如有未通过项>`

### 风险与回滚点
- 主要风险: 实机环境差异导致验收不稳定
- 回滚方式: 仅回退小修；receipt 保留历史证据
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| QUALITY-001 | Tauri dev 启动失败 | 记录完整错误，不做伪验收 | 状态保持 `VERIFY/OPEN` |
| TEST-001 | 无 GUI 环境 | 运行可用命令，标记实机待验 | 有条件交付 |
| ARCH-001 | 修复需要前端架构拆分 | 停止扩修，登记到 Day 13-14 | 不扩大本日 |
| DATA-001 | localStorage 数据异常 | 导出异常样本，做最小兼容或登记债务 | 避免数据丢失 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 07 启动 / 文件树 / 会话持久化实机验收**。

### 关键约束
- 无 receipt 不清债
- 必须覆盖关闭重开
- 必须复验新建/重命名/删除
- 小修不得变成重构

### 验收铁律
- UX receipt 存在
- Rust/JS 检查通过
- 启动到重启链路完整
- 状态迁移诚实

闭环启动，Day 07，执行。
