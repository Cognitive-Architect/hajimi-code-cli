# B-16 Day 6 派单：Debt Receipt + Active Status Update

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: Day 1-5 代码与测试产物
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 6 债务 Receipt 与活跃债务状态更新建议
- **轰炸目标**：整理 Day 1-5 的真实变更与验证输出，创建/完善 `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`，并按实际结果给出 active debt 状态更新建议，绝不伪关闭 WebView 相关债务。
- **任务性质**：文档闭环 + 质量证据归档
- **输入基线**：完整技术背景见模块2。
- **输出要求**：一份可审计 receipt，包含分支/HEAD、scope、变更文件、验证命令、债务状态建议、人工验收清单和回滚方法。
- **通用铁律**：
  1. **数据诚实**：命令输出摘要必须来自真实运行。
  2. **零占位符**：不得留下 `<待补>`、`TODO 后续填写`。
  3. **自动化优先**：所有能跑的验证命令必须跑。
  4. **最小必要复杂度**：不在 Day 6 改产品逻辑，除非修正文档暴露的小问题。
  5. **债务透明化**：状态只建议 partial/improved/gated，不夸大关闭。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | B16 receipt、active debt 建议、人工验收清单 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`、`docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md` 或新快照、`docs/debt/INDEX.md`（按需） | 必须 |
| 现状基线 | Day 1-5 应已产生 slash palette、smoke、security gate；如果缺失，必须如实记录缺失而不是补写假结果 | `Test-Path src/interface/web/modules/slash-palette.js`；`Test-Path tests/frontend/day16_slash_palette_smoke.js`；`Test-Path tests/security/security_audit_gate.js` | 必须 |
| 目标结果 | receipt 明确 `AD-007 IMPLEMENTED/PENDING-UI-SMOKE`、`AD-004 PARTIAL/IMPROVED`、`AD-008 PARTIAL/GATED`、`AD-006 DEFERRED/P2-SPEC`、`AD-001 OPEN BY DESIGN`，并明确 `AD-002/003/005` 不关闭 | receipt 内容验证 | 必须 |
| 技术约束 | 不把 Node smoke 写成 WebView smoke；不覆盖旧 active snapshot，优先新建快照或“更新建议” section；文档路径必须真实存在 | `rg -n "Node smoke|WebView|AD-002|AD-003|AD-005" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 必须 |
| 风险边界 | 不做功能开发；不迁移 Tauri；不改 Rust；不归档仍活跃债务 | `git diff --stat` | 必须 |
| 测试基线 | Day 6 必须重新跑关键验证 | `node --check app.js`、`node --check slash-palette.js`、`node tests/frontend/day16_slash_palette_smoke.js`、`node tests/security/security_audit_gate.js` | 必须 |
| 文档同步要求 | receipt 和 active debt 建议完整；必要时更新 `docs/debt/INDEX.md` | `git diff -- docs/debt` | 必须 |
| 历史债务 / 相关缺陷 | `AD-002/003/005` 需要真实 Tauri/WebView 证据，不在本批自动关闭 | receipt | 必须 |

### 探索补充栏

本任务为文档闭环。若 Day 1-5 产物缺失，停止伪闭环，改为“缺失项清单 + 阻塞原因 + 后续补救工单”。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/06
- **角色**：Engineer
- **目标**：完成 B16 debt remediation receipt 和 active debt 状态更新建议。
- **输入**：Day 1-5 产物与验证命令。
- **依赖关系**：依赖 Day 1-5。

### 2）输出交付物

- **变更文件**：
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`
  - `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md` 或新 `ACTIVE-DEBT-STATUS-YYYY-MM-DD.md`（按项目习惯）
  - `docs/debt/INDEX.md`（按需）
- **核心修改点**：
  - 记录分支、HEAD、日期、范围。
  - 列出 Day 1-5 变更文件。
  - 写入验证命令与输出摘要。
  - 给出每个相关 AD 状态建议。
  - 写入用户实机验收清单。
  - 写入回滚方式。
- **必须包含**：
  - `AD-007`: `IMPLEMENTED/PENDING-UI-SMOKE` 或按实际结果更保守。
  - `AD-004`: `PARTIAL/IMPROVED`，不得关闭。
  - `AD-008`: `PARTIAL/GATED`，不得写成完整安全系统。
  - `AD-002/003/005`: 明确未关闭，需真实 Tauri/WebView。
  - `AD-001`: `OPEN BY DESIGN`。
  - `AD-006`: `DEFERRED/P2-SPEC`。
- **禁止包含**：
  - 未运行命令却写 PASS。
  - 用旧输出冒充本日输出。
  - 删除活跃债务文档。
  - 把人工验收项写成已完成。
- **交付证明**：
  - receipt 中有命令输出摘要。
  - `rg` 能查到 AD 状态。
  - `git diff --check -- docs/debt` 通过。

### 3）规模与复杂度观察

- **推荐目标**：receipt 结构清楚，状态矩阵优先。
- **复杂度说明**：如 active snapshot 路径受 `.gitignore` 影响，收卷中说明提交时需 `git add -f`。
- **禁止行为**：为“文档好看”改写事实。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 前端语法通过 | `node --check src/interface/web/app.js` | 返工或记录阻塞 |
| BUILD-MODULE | slash 模块语法通过 | `node --check src/interface/web/modules/slash-palette.js` | 返工或记录阻塞 |
| TEST | slash smoke 通过 | `node tests/frontend/day16_slash_palette_smoke.js` | 返工或记录阻塞 |
| SECURITY | gate 通过 | `node tests/security/security_audit_gate.js` | 返工或记录阻塞 |
| FMT | 文档 diff 无尾随空格 | `git diff --check -- docs/debt` | 返工 |
| LINT | N/A，文档任务 | N/A + 原因：无代码 lint 变更 | - |
| REAL | receipt 不伪造 WebView | `rg -n "Node smoke|WebView|不等同|未关闭" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 返工 |
| DOC | AD 状态完整 | `rg -n "AD-001|AD-002|AD-003|AD-004|AD-005|AD-006|AD-007|AD-008" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 返工 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | receipt 存在 | `Test-Path docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| FUNC | FUNC-002 | 记录分支与 HEAD | `rg -n "branch|HEAD|分支" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| FUNC | FUNC-003 | 记录变更文件清单 | `rg -n "slash-palette|day16_slash|security_audit_gate" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| FUNC | FUNC-004 | 记录验证命令 | `rg -n "node --check|node tests/frontend|node tests/security" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| CONST | CONST-001 | AD-007 状态建议 | `rg -n "AD-007|IMPLEMENTED|PENDING-UI-SMOKE" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| CONST | CONST-002 | AD-004 仅 partial/improved | `rg -n "AD-004|PARTIAL/IMPROVED" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| CONST | CONST-003 | AD-008 仅 partial/gated | `rg -n "AD-008|PARTIAL/GATED" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| CONST | CONST-004 | AD-002/003/005 不关闭 | `rg -n "AD-002|AD-003|AD-005|未关闭|不关闭" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| NEG | NEG-001 | 不伪造 WebView smoke | `rg -n "Node smoke.*不等同|WebView.*未" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| NEG | NEG-002 | 无占位符 | `rg -n "<待补>|TODO|TBD|待填写" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 无命中 | [ ] |
| NEG | NEG-003 | 文档格式无尾随空格 | `git diff --check -- docs/debt` | [ ] |
| NEG | NEG-004 | 不改产品代码 | `git diff --stat -- src tests package.json` 为空或只含必要小修并说明 | [ ] |
| UX | UX-001 | 人工验收清单存在 | `rg -n "实机验收|人工验收|Tauri|输入 /|ArrowDown|Esc" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| UX | UX-002 | 回滚方式存在 | `rg -n "回滚|revert|rollback" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| E2E | E2E-001 | 关键命令重跑通过 | `node tests/frontend/day16_slash_palette_smoke.js; node tests/security/security_audit_gate.js` | [ ] |
| High | HIGH-001 | active debt 状态不夸大 | 文档审查 + `rg -n "完整安全审计|WebView smoke 已通过" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 无误导命中 | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. receipt 缺失，返工。
2. 未记录分支/HEAD，返工。
3. 命令未运行却写 PASS，返工。
4. Node smoke 冒充 WebView smoke，返工。
5. AD-004/008 被完全关闭，返工。
6. AD-002/003/005 被伪关闭，返工。
7. 文档含 `<待补>` / TBD，返工。
8. 删除活跃债务源文档，返工。
9. Day 6 大量改产品代码，返工。
10. 新增债务未声明，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | receipt 是否覆盖 Day 1-5 产物？ | [ ] | CF-B16-D06-001 | |
| 约束与回归用例（RG） | AD 状态是否诚实？ | [ ] | RG-B16-D06-001 | |
| 负面路径用例（NG） | 缺失/失败项是否记录？ | [ ] | NG-B16-D06-001 | |
| 用户体验用例（UX） | 人工验收清单是否可执行？ | [ ] | UX-B16-D06-001 | |
| 端到端关键路径（E2E） | 关键自动命令是否重跑？ | [ ] | E2E-B16-D06-001 | |
| 高风险场景（High） | 是否避免伪关闭 WebView 债务？ | [ ] | HIGH-B16-D06-001 | |
| 字段完整性 | 分支/HEAD/scope/验证/回滚是否齐全？ | [ ] | receipt 审查 | |
| 需求映射 | 是否映射到 AD-001~008？ | [ ] | receipt | |
| 自测执行 | 是否完整跑过本日命令？ | [ ] | 输出摘要 | |
| 范围边界与债务 | 未覆盖项是否明确声明？ | [ ] | receipt | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/06 完成并提交

### 提交信息
- Commit: `docs(debt): record b16 slash palette and safety gate receipt`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`
  - `<active snapshot 或 INDEX 如有>`

### 本轮目标与实际结果
- 目标: 完成 B16 receipt 和 active debt 更新建议。
- 实际完成: `<列出状态矩阵和验证摘要>`
- 未完成/不在范围: WebView smoke；Tauri global API 迁移；Thinking checkpoint 深层债务。

### 关键决策记录
- DECISION-001: `<active status 更新方式>` - `<原因>`
- DECISION-002: `<AD-007/004/008 状态建议>` - `<原因>`

### 自动化质量检查报告
```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
node tests/security/security_audit_gate.js
git diff --check -- docs/debt
```

### 债务声明
- DEBT-UI-B16-D06: WebView smoke 未完成，需用户实机验收。
- DEBT-SCOPE-B16-D06: AD-002/003/005 不在本批自动关闭范围。

### 风险与回滚点
- 主要风险: 文档状态过度乐观。
- 回滚方式: 回退 receipt/active snapshot 文档。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| DOC-001 | Day 1-5 产物缺失 | 不伪造 receipt，改写缺失清单和阻塞说明 | 阻塞闭环 |
| QUALITY-001 | 关键命令失败 | 记录失败，状态降级，不写 PASS | 有条件交付 |
| SCOPE-001 | 发现需 WebView 才能判断 | 写入人工验收，不关闭债务 | 保留债务 |
| GIT-001 | `docs/debt/active` 被 ignore | 收卷说明提交需 `git add -f` | 提交注意 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 6 Debt Receipt + Active Status Update** 通用高压任务！

### 技术背景
Day 1-5 已围绕 slash palette、Node smoke 和 security gate 形成代码与测试产物。Day 6 的目标是做诚实文档闭环，给后续用户实机验收和最终回归提供依据。

### 关键约束
- 命令输出必须真实。
- Node smoke 不等同 WebView smoke。
- AD-004/008 不完全关闭。
- AD-002/003/005 不关闭。

### 质量红线
- 10 项地狱红线生效。
- 文档不得含占位符。
- 状态不得夸大。

### 工单并行矩阵
- B-16/06 Engineer：Debt Receipt + Active Status Update

### 验收铁律
- receipt 存在且 AD-001~008 状态完整。
- 关键命令重跑并记录摘要。
- 人工 Tauri/WebView 验收清单存在。

### 收卷要求
- 附自动化质量检查摘要。
- 附刀刃表摘要。
- 诚实声明所有未关闭债务。

Ouroboros 闭环启动，B-16 Day 6，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
node tests/security/security_audit_gate.js
rg -n "AD-001|AD-002|AD-003|AD-004|AD-005|AD-006|AD-007|AD-008" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
rg -n "Node smoke|WebView|实机验收|回滚" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
git diff --stat
git diff --check -- docs/debt
```
