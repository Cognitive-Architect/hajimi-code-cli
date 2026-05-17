# Day 01 派单: 债务状态复核 + Signaling PSK 归档候选确认

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 `HAJIMI_DEBT_REMEDIATION_DAILY_PLAN_2026-05-15.md` 的 Day 1，并吸收 Day 0 的本地 audit 前置动作。
> 目标不是修代码，而是为 Day 2-4 的 P0/P1 修复建立可信输入基线。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Architect）
- **任务名称**: 债务状态复核 + Signaling PSK 归档候选确认
- **轰炸目标**: 对 `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 中 P0/P1 债务重新跑本地证据，确认 Shell、workspace symlink、Tauri CSP/global API、文件操作错配、Signaling PSK 五类状态没有漂移
- **任务性质**: 探索调研 + 文档对账
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 本地 audit receipt + 债务状态差异清单 + 如有冲突则修正债务总表
- **通用铁律**:
  1. 所有状态判断必须来自本地命令输出或源码定位
  2. 不允许把 `OPEN` 或 `PARTIAL` 直接改成 `CLEARED`
  3. 不允许因为“文档写了已修”而跳过源码复核
  4. Signaling PSK 只能在确认没有 active runtime 后保持 `ARCHIVE CANDIDATE`
  5. 本日不修改功能代码

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 记录当前分支和 HEAD | `git branch --show-current`; `git rev-parse HEAD`; `git status --short` | 必须 |
| 债务总表 | 当前状态快照 | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 必须 |
| Roadmap | 批次优先级 | `docs/roadmap/hajimi debtFix/plan/HAJIMI_DEBT_REMEDIATION_ROADMAP_2026-05-15.md` | 必须 |
| Daily Plan | Day 0-15 执行顺序 | `docs/roadmap/hajimi debtFix/plan/HAJIMI_DEBT_REMEDIATION_DAILY_PLAN_2026-05-15.md` | 必须 |
| Shell 债 | `shell.rs` 当前仍允许 shell 解释器 | `src/engine/tool-system/src/shell.rs:20-40`, `:320-331` | 必须 |
| workspace 债 | `validate_path_within_workspace` fallback 风险 | `src/interface/desktop/src/main.rs:166-227` | 必须 |
| Tauri 债 | global API 与 CSP 配置 | `src/interface/desktop/tauri.conf.json:13`, `:25` | 必须 |
| 文件操作错配 | 前端 `mkdir/mv/rm` 调用与后端白名单不一致 | `src/interface/web/app.js:786`, `:1018`, `:1038`; `src/interface/desktop/src/main.rs:229` | 必须 |
| Signaling PSK | 当前为 `ARCHIVE CANDIDATE` | `rg -n "WebRTC|signaling|psk|pre-shared|KMS|Vault" src Cargo.toml package.json` | 必须 |
| 文档 ignore 风险 | `docs/` 可能被 `.gitignore` 忽略 | `rg -n "^docs/|audit report" .gitignore`; `git status --short --ignored docs/roadmap` | 必须 |

### 探索补充栏

| 项目 | 内容 |
|---|---|
| 已知事实 | 债务总表已经把文件操作错配扩展到 `mkdir/mv/rm`，Signaling PSK 已收窄为 `ARCHIVE CANDIDATE` |
| 待确认问题 | 本地源码是否已被其他 agent 修改；Signaling 是否出现新的 active runtime；Day 2-4 是否仍按 P0/P1 顺序执行 |
| 预期输出 | 一份本地 audit receipt，必要时更新债务总表，不做功能代码变更 |
| 停止条件 | 五类债务状态与本地源码一致，或差异已写入债务总表 |

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-01/15
- **角色**: Architect
- **目标**: 复核 P0/P1 债务当前事实，给 Day 2-4 安全修复建立不可争议的输入
- **输入**: 模块2全部输入项
- **依赖关系**: 无前置代码依赖；必须先于 Day 2 执行

### 2）输出交付物

- **变更文件**:
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`，仅当本地证据和现有表述冲突时修改
  - `docs/roadmap/hajimi debtFix/debt/local-debt-audit-YYYYMMDD-HHMMSS.txt`
- **核心修改点**:
  - 记录 Shell 白名单、workspace resolver、Tauri CSP、文件操作错配、Signaling 搜索结果
  - 明确 Day 2-4 的修复前确认清单
- **必须包含**:
  - 当前分支、HEAD、工作区状态
  - `rg` 命令输出摘要
  - 如果修改债务总表，必须说明修改原因和证据
- **禁止包含**:
  - 功能代码修改
  - `OPEN -> CLEARED`
  - 没有证据的“已修复”表述
- **交付证明**:
  - audit 文件路径
  - `git diff -- docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` 摘要，若无修改则说明无差异

### 3）规模与复杂度观察

- **推荐目标**: 只做证据收集与文档修正，不引入实现复杂度
- **复杂度说明**: 如 Signaling 搜索命中过多，需要按 active runtime / tests / docs 分类，不得一概归档
- **禁止行为**: 为了推进进度而跳过本地命令

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 本日不改代码 | `git diff -- src` 应为空或只包含用户已有改动 | 返工 |
| FMT | Markdown 不要求格式器 | N/A，文档人工检查 | 说明原因 |
| LINT | 不新增功能代码 warning | N/A，本日不改代码 | 说明原因 |
| TEST | 本地证据命令完成 | 模块3-A全部命令 | 返工 |
| ARCH | 状态迁移符合规则 | `rg -n "OPEN -> CLEARED|UNKNOWN -> CLEARED" "docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md"` 应无新增 | 返工 |
| REAL | 不允许伪证据 | audit 文件必须包含命令输出摘要 | 返工 |
| DOC | 如有冲突则更新债务总表 | `git diff -- "docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md"` | 返工或声明无差异 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | 当前 Git 坐标已记录 | `git branch --show-current`; `git rev-parse HEAD` | [ ] |
| FUNC | FUNC-002 | Shell 解释器白名单状态已确认 | `rg -n "ALLOWED_COMMANDS|\"bash\"|\"sh\"|\"pwsh\"|\"powershell\"" src/engine/tool-system/src/shell.rs` | [ ] |
| FUNC | FUNC-003 | workspace resolver 风险已确认 | `rg -n "validate_path_within_workspace|canonicalize|unwrap_or\\(resolved\\)" src/interface/desktop/src/main.rs` | [ ] |
| FUNC | FUNC-004 | 文件操作错配覆盖 `mkdir/mv/rm` | `rg -n "createNewFolder|renameFile|deleteFile|cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | Tauri CSP/global API 已确认 | `rg -n "withGlobalTauri|csp" src/interface/desktop/tauri.conf.json` | [ ] |
| CONST | CONST-002 | Signaling PSK active runtime 已分类 | `rg -n "WebRTC|signaling|psk|pre-shared|KMS|Vault" src Cargo.toml package.json` | [ ] |
| CONST | CONST-003 | 文档 ignore 风险已记录 | `git status --short --ignored docs/roadmap` | [ ] |
| CONST | CONST-004 | 状态迁移规则未破坏 | 人工检查债务总表状态迁移段落 | [ ] |
| NEG | NEG-001 | 未修改功能代码 | `git diff -- src` | [ ] |
| NEG | NEG-002 | 未将未验证项标记 CLEARED | `rg -n "CLEARED" "docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md"` 并人工核对 receipt | [ ] |
| NEG | NEG-003 | 未把 `mkdir/mv/rm` 加入后端白名单 | `rg -n "mkdir|mv|rm" src/interface/desktop/src/main.rs src/engine/tool-system/src/shell.rs` | [ ] |
| NEG | NEG-004 | Signaling 文档命中未误判为 active | 命中列表按 source/test/docs/runtime 分类 | [ ] |
| UX | UX-001 | Day 2-4 输入清单可执行 | audit 中列出 Day 2-4 必修文件 | [ ] |
| UX | UX-002 | 错误或差异说明可读 | audit 中有“冲突/无冲突”结论 | [ ] |
| E2E | E2E-001 | 本地 audit 文件存在 | `Get-ChildItem -LiteralPath "docs/roadmap/hajimi debtFix/debt" -Filter "local-debt-audit-*.txt"` | [ ] |
| High | HIGH-001 | P0/P1 状态没有凭空降级 | 债务总表 diff 或“无 diff”说明 | [ ] |

---

## 【模块3-B】地狱红线

1. 只引用文档不跑源码命令，返工
2. 没有 audit 文件却声称完成，返工
3. 把 `ARCHIVE CANDIDATE` 写成 `ARCHIVE` 但没有 owner 确认，返工
4. 将 `OPEN` 直接改成 `CLEARED`，返工
5. 修改功能代码，返工
6. 忽略 `docs/` 被 `.gitignore` 忽略的事实，返工
7. 漏掉 `renameFile/deleteFile` 的 `mv/rm` 错配，返工
8. 用 `grep` 依赖 Unix 环境而不提供 Windows 可执行替代，返工
9. 状态判断没有对应命令输出摘要，返工
10. 未给 Day 2 明确修复入口，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 五类债务是否全部复核 | [ ] | FUNC-001~004 | |
| RG | 是否覆盖 2026-05-16 修订口径 | [ ] | 文件操作 `mkdir/mv/rm` | |
| NG | 是否防止错误清债 | [ ] | NEG-002 | |
| UX | 后续 agent 是否能直接接 Day 2 | [ ] | UX-001 | |
| E2E | audit 文件是否存在 | [ ] | E2E-001 | |
| High | Signaling 是否未被误归档 | [ ] | HIGH-001 | |
| 字段完整性 | 是否记录分支、HEAD、命令 | [ ] | Git 坐标命令 | |
| 需求映射 | 是否映射债务总表与 Daily Plan | [ ] | 文档路径 | |
| 自测执行 | 是否实际跑过命令 | [ ] | audit receipt | |
| 范围边界与债务 | 是否声明本日不改代码 | [ ] | `git diff -- src` | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-01/15 完成并提交

### 提交信息
- Commit: `docs(debt): record day 1 local debt recheck`
- 分支: `<实际分支>`
- 变更文件:
  - `docs/roadmap/hajimi debtFix/debt/local-debt-audit-YYYYMMDD-HHMMSS.txt`
  - `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md`（如有）

### 本轮目标与实际结果
- 目标: 复核 P0/P1 债务状态
- 实际完成: `<列出已确认项>`
- 未完成/不在范围: 功能代码修复不在 Day 1

### 自动化质量检查报告
- `git branch --show-current`: `<摘要>`
- `git rev-parse HEAD`: `<摘要>`
- `rg ... shell.rs`: `<摘要>`
- `rg ... main.rs`: `<摘要>`
- `rg ... app.js`: `<摘要>`
- `rg ... tauri.conf.json`: `<摘要>`
- `rg ... WebRTC/signaling`: `<摘要>`

### 债务声明
- `DEBT-DOC-B01-001`: 如 docs 被 ignore，提交时需 `git add -f`

### 风险与回滚点
- 主要风险: 状态判断失真会污染后续 14 天工单
- 回滚方式: 回退本日文档 diff，不涉及功能代码
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 发现本地源码与三份计划文档大面积冲突 | 暂停 Day 2，先修订计划文档 | 不进入代码修复 |
| QUALITY-001 | audit 命令无法执行 | 记录失败命令和环境，不伪造结果 | 有条件收卷 |
| TEST-001 | `rg` 不可用 | 使用 PowerShell `Select-String` 替代并记录 | 有条件收卷 |
| DOC-001 | docs ignore 导致状态不可见 | 收卷中注明 `git add -f` 要求 | 不影响代码 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 01 债务状态复核 + Signaling PSK 归档候选确认**。

### 关键约束
- 本日不改功能代码
- 所有状态必须有命令证据
- Signaling PSK 只能保持 `ARCHIVE CANDIDATE`，除非 owner 明确确认归档
- 文件操作错配必须覆盖 `createNewFolder/renameFile/deleteFile`

### 验收铁律
- audit receipt 存在
- P0/P1 状态与本地源码一致
- 没有未验证清债
- Day 2 输入清单清晰

闭环启动，Day 01，执行。
