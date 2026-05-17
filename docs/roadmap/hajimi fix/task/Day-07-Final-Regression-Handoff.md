# B-16 Day 7 派单：Final Regression + Handoff Pack

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: Day 1-6 产物
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 7 最终回归与 Handoff Pack
- **轰炸目标**：运行 B16 最终验证命令，确认 slash palette、Node smoke、security gate、shell allow-list 回归均通过，整理最终 handoff 和用户实机验收步骤，保证可提交、可回滚、可审计。
- **任务性质**：最终验证 + 交接收卷
- **输入基线**：完整技术背景见模块2。
- **输出要求**：最终回归报告 + 完整 handoff + 干净的 diff 范围 + 明确剩余人工验收项。
- **通用铁律**：
  1. **数据诚实**：所有最终验证结果必须来自本日真实命令。
  2. **零占位符**：handoff 不得留 `<待补>`。
  3. **自动化优先**：能跑的全部跑；不能跑的写清原因。
  4. **最小必要复杂度**：Day 7 不新增功能，只做回归和小修。
  5. **债务透明化**：仍未实机验收的项目必须留在 handoff。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | B16 全部变更文件与 receipt | `src/interface/web/modules/slash-palette.js`、`src/interface/web/app.js`、`src/interface/web/style.css`、`tests/frontend/day16_slash_palette_smoke.js`、`tests/security/security_audit_gate.js`、`docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 必须 |
| 现状基线 | Day 1-6 应已完成 slash palette、smoke、security gate、receipt；缺失项必须阻塞或降级 | `Test-Path` 各目标文件 | 必须 |
| 目标结果 | 最终命令通过，handoff 明确自动验证结果、人工验收步骤、剩余债务、回滚方法和建议 commit message | receipt / handoff section | 必须 |
| 技术约束 | Day 7 不做大功能；不伪造 WebView smoke；不关闭 AD-002/003/005；shell allow-list 测试必须跑或记录环境阻塞 | 命令输出摘要 | 必须 |
| 风险边界 | 不迁移 Tauri global API；不恢复复杂 shell；不做 Thinking checkpoint；不新增大依赖 | `git diff --stat` | 必须 |
| 测试基线 | 最终验证命令全量 | `node --check app.js`、`node --check slash-palette.js`、`node tests/frontend/day16_slash_palette_smoke.js`、`node tests/security/security_audit_gate.js`、`cargo test -p engine-tool-system -- test_allow_list` | 必须 |
| 文档同步要求 | receipt 最终更新，必要时 docs/debt/INDEX.md 同步 | `git diff -- docs/debt` | 必须 |
| 历史债务 / 相关缺陷 | AD-002/003/005 仍需用户实机验收；AD-001 仍 `OPEN BY DESIGN` | receipt | 必须 |

### 探索补充栏

本任务为最终验证。若任何核心命令失败，不得“继续收卷为完成”，必须先修小问题或记录阻塞并降低状态。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/07
- **角色**：Engineer
- **目标**：完成 B16 最终回归验证和用户 handoff。
- **输入**：Day 1-6 全部产物。
- **依赖关系**：依赖 Day 1-6。

### 2）输出交付物

- **变更文件**：
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`（最终验证 section）
  - `docs/debt/INDEX.md`（按需）
  - 仅允许对 Day 1-6 代码做小修，不新增新功能文件
- **核心修改点**：
  - 运行最终命令并记录输出摘要。
  - 更新 receipt 最终状态和用户实机验收脚本。
  - 检查 `git diff --stat`，确认无异常大改。
  - 准备建议 commit message。
  - 明确 `docs/debt/active` 或 ignored docs 提交注意事项。
- **必须包含**：
  - 自动验证命令结果。
  - 用户实机验收清单：启动 Tauri、输入 `/`、过滤、上下键、Enter、Esc、普通消息、控制台无 error。
  - 剩余债务：真实 WebView smoke、Tauri global API、Thinking checkpoint 深水区等。
  - 回滚策略。
- **禁止包含**：
  - Day 7 临时新增未测功能。
  - 失败命令被隐藏。
  - 伪造 git 干净状态。
  - 自动关闭需要用户点击的债务。
- **交付证明**：
  - 最终命令输出摘要。
  - `git diff --stat`。
  - `git status --short --ignored docs/debt docs/roadmap/hajimi fix/task` 如需说明 ignored 文件。

### 3）规模与复杂度观察

- **推荐目标**：只整理和小修，不扩大 diff。
- **复杂度说明**：若 security gate 误报或 cargo 测试慢，先定位，必要时记录阻塞。
- **禁止行为**：为了让最终验证过关而放宽安全 gate。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | app.js 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| BUILD-MODULE | slash 模块语法通过 | `node --check src/interface/web/modules/slash-palette.js` | 返工 |
| TEST | slash smoke 通过 | `node tests/frontend/day16_slash_palette_smoke.js` | 返工 |
| SECURITY | gate 通过 | `node tests/security/security_audit_gate.js` | 返工 |
| RUST | shell allow-list 回归通过 | `cargo test -p engine-tool-system -- test_allow_list` | 返工或阻塞声明 |
| FMT | diff 无尾随空格 | `git diff --check` | 返工 |
| ARCH | diff 范围合理 | `git diff --stat` | 异常大改需说明 |
| DOC | receipt 最终完整 | `rg -n "Final|最终|实机验收|回滚|AD-007|AD-008" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 返工 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | Slash 模块存在 | `Test-Path src/interface/web/modules/slash-palette.js` | [ ] |
| FUNC | FUNC-002 | Slash smoke 存在 | `Test-Path tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| FUNC | FUNC-003 | Security gate 存在 | `Test-Path tests/security/security_audit_gate.js` | [ ] |
| FUNC | FUNC-004 | Receipt 存在 | `Test-Path docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| CONST | CONST-001 | app.js 语法通过 | `node --check src/interface/web/app.js` | [ ] |
| CONST | CONST-002 | slash 模块语法通过 | `node --check src/interface/web/modules/slash-palette.js` | [ ] |
| CONST | CONST-003 | slash smoke 通过 | `node tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| CONST | CONST-004 | security gate 通过 | `node tests/security/security_audit_gate.js` | [ ] |
| NEG | NEG-001 | shell allow-list 测试通过 | `cargo test -p engine-tool-system -- test_allow_list` | [ ] |
| NEG | NEG-002 | 无 tail whitespace | `git diff --check` | [ ] |
| NEG | NEG-003 | 无危险 shell 回归 | `rg -n "assert!\\(b.check_allow_list\\(\"bash|assert!\\(b.check_allow_list\\(\"sh|powershell" src/engine/tool-system/src/shell.rs` | [ ] |
| NEG | NEG-004 | WebView 未伪关闭 | `rg -n "WebView|实机验收|未关闭" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| UX | UX-001 | 人工验收脚本完整 | `rg -n "输入 /|/c|ArrowDown|ArrowUp|Enter|Esc|普通消息" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| UX | UX-002 | 回滚方式完整 | `rg -n "回滚|git revert|rollback" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| E2E | E2E-001 | 最终 diff 范围合理 | `git diff --stat` | [ ] |
| High | HIGH-001 | AD 状态矩阵完整 | `rg -n "AD-001|AD-002|AD-003|AD-004|AD-005|AD-006|AD-007|AD-008" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. 任一核心命令失败但写完成，返工。
2. 未跑 shell allow-list 测试且无阻塞说明，返工。
3. Day 7 新增大功能，返工。
4. 放宽 security gate 以通过测试，返工。
5. WebView smoke 伪关闭，返工。
6. `git diff --check` 失败，返工。
7. receipt 缺最终验证摘要，返工。
8. 人工验收清单缺失，返工。
9. 未说明剩余债务，返工。
10. git 状态异常但未说明，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | slash palette 自动验证是否通过？ | [ ] | CF-B16-D07-001 | |
| 约束与回归用例（RG） | shell allow-list 与 security gate 是否通过？ | [ ] | RG-B16-D07-001 | |
| 负面路径用例（NG） | 未验收项是否保留为债务？ | [ ] | NG-B16-D07-001 | |
| 用户体验用例（UX） | 人工点击脚本是否完整？ | [ ] | UX-B16-D07-001 | |
| 端到端关键路径（E2E） | 最终命令是否全跑？ | [ ] | E2E-B16-D07-001 | |
| 高风险场景（High） | 是否避免安全 gate/复杂 shell 回归？ | [ ] | HIGH-B16-D07-001 | |
| 字段完整性 | final handoff 是否有 branch/HEAD/diff/status？ | [ ] | receipt | |
| 需求映射 | AD-001~008 是否全部交代？ | [ ] | receipt | |
| 自测执行 | 是否完整跑过最终验证？ | [ ] | 输出摘要 | |
| 范围边界与债务 | 剩余人工验收是否明确？ | [ ] | receipt | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/07 完成并提交

### 提交信息
- Commit: `feat(frontend): add slash palette v1 and lightweight security gate`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `src/interface/web/modules/slash-palette.js`
  - `src/interface/web/app.js`
  - `src/interface/web/style.css`
  - `tests/frontend/day16_slash_palette_smoke.js`
  - `tests/security/security_audit_gate.js`
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`

### 本轮目标与实际结果
- 目标: 完成 B16 最终回归与 handoff。
- 实际完成: `<最终命令结果>`
- 未完成/不在范围: `<真实 WebView smoke / AD-002 / AD-003 / AD-005 等>`

### 关键决策记录
- DECISION-001: `<最终状态建议>` - `<原因>`
- DECISION-002: `<剩余人工验收处理>` - `<原因>`

### 自动化质量检查报告
```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
node tests/security/security_audit_gate.js
cargo test -p engine-tool-system -- test_allow_list
git diff --check
git diff --stat
```

### 刀刃表摘要
| 类别 | 覆盖数 | 关键证据 |
|:---|:---:|:---|
| FUNC | 4/4 | |
| CONST | 4/4 | |
| NEG | 4/4 | |
| UX | 2/2 | |
| E2E | 1/1 | |
| High | 1/1 | |

### 债务声明
- DEBT-UI-B16-D07: 真实 Tauri/WebView 点击验收仍需用户执行。
- DEBT-SCOPE-B16-D07: AD-002/003/005 不在本批自动关闭范围。

### 风险与回滚点
- 主要风险: 真实 WebView 与 Node smoke 行为存在差异。
- 回滚方式: `git revert <commit>`，或回退 slash palette、smoke、security gate、receipt 相关文件。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| QUALITY-001 | 任一核心命令失败 | 停止收卷，修复或降级状态 | 返工/阻塞 |
| RUST-001 | `cargo test` 因环境慢或锁失败 | 重试一次；仍失败则记录完整错误和未验证风险 | 有条件交付 |
| SECURITY-001 | security gate 失败 | 先修安全问题，不放宽规则 | 返工 |
| DOC-001 | receipt 与实际命令不一致 | 修正文档，不改写事实 | 返工 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 7 Final Regression + Handoff Pack** 通用高压任务！

### 技术背景
B-16 已完成 slash palette、Node smoke、security gate 和 debt receipt。Day 7 要做最终回归，确保后续可以提交、推送或交给用户进行真实 Tauri/WebView 验收。

### 关键约束
- 不新增大功能。
- 所有最终验证命令必须真实运行。
- WebView 未验收必须保留为人工验收项。
- 不放宽 security gate。

### 质量红线
- 10 项地狱红线生效。
- 失败命令不得隐藏。
- 最终 handoff 不得含占位符。

### 工单并行矩阵
- B-16/07 Engineer：Final Regression + Handoff Pack

### 验收铁律
- `node --check` 双文件通过。
- slash smoke 通过。
- security gate 通过。
- shell allow-list 测试通过或明确阻塞。
- receipt/handoff 完整。

### 收卷要求
- 附自动化质量检查摘要。
- 附刀刃表摘要。
- 附剩余人工验收清单和回滚方式。

Ouroboros 闭环启动，B-16 Day 7，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
node tests/security/security_audit_gate.js
cargo test -p engine-tool-system -- test_allow_list
rg -n "AD-001|AD-002|AD-003|AD-004|AD-005|AD-006|AD-007|AD-008" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
rg -n "输入 /|/c|ArrowDown|ArrowUp|Enter|Esc|普通消息|WebView" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
git diff --check
git diff --stat
git status --short --ignored docs/debt "docs/roadmap/hajimi fix/task"
```
