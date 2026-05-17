# B-16 Day 1 派单：Baseline Audit + Slash Contract

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: `docs/roadmap/hajimi fix/plan/B16-SLASH-PALETTE-SAFETY-GATE-ROADMAP.md` + `B16-SLASH-PALETTE-SAFETY-GATE-EXECUTION-PLAN.md`
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`
> **执行要求**: 执行 agent 必须在开工前重新记录自己的 branch 与 HEAD，不得把本派单生成基线冒充为完成基线。

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 1 Baseline Audit + SlashCommandItem V1 Contract
- **轰炸目标**：只读审计 `src/interface/web/app.js`、`index.html`、`modules/*`、`style.css` 与 `src/engine/tool-system/src/shell.rs`，定位 slash command 入口、聊天输入入口、模块接入方式和安全 gate 扫描点，并输出 SlashCommandItem V1 契约草案。
- **任务性质**：探索调研 + 契约设计 + 文档基线
- **输入基线**：完整技术背景见模块2，禁止只写“见 roadmap”。
- **输出要求**：Baseline notes + SlashCommandItem V1 contract + 安全扫描点清单 + 后续实现边界。
- **通用铁律**：
  1. **数据诚实**：所有命令命中数量、测试结果、warning 数必须来自真实输出。
  2. **零占位符**：不能只写“已检查”，必须给出命令、路径、行号或输出摘要。
  3. **自动化优先**：能用 `rg` / `node --check` 验证的，必须运行。
  4. **最小必要复杂度**：Day 1 不修改核心业务逻辑，不做 UI 实现。
  5. **债务透明化**：未知项必须记录为待确认，不得伪装为已闭环。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 执行时当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | slash command、聊天输入、command palette、前端模块、安全扫描入口 | `src/interface/web/app.js`、`src/interface/web/index.html`、`src/interface/web/modules/*`、`src/interface/web/style.css`、`src/engine/tool-system/src/shell.rs` | 必须 |
| 现状基线 | `index.html` 已有 `#aiChatInput`、`#commandPalette`；`app.js` 已有 `handleChatCommand`、`sendChatMessage`、`showCommandPalette`；`modules` 下已有 `security-dom.js`、`workspace.js`、`sessions.js`、`thinking-ui.js`，尚无 `slash-palette.js` | `rg -n "aiChatInput|commandPalette|handleChatCommand|sendChatMessage|showCommandPalette" src/interface/web` | 必须 |
| 目标结果 | 明确 `/` 输入建议面板 V1 的数据结构、触发条件、过滤规则、选择规则和安全渲染约束 | 新增或更新 `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 的 Baseline/Contract 草稿，或独立 `docs/debt/B16-BASELINE-NOTES.md` | 必须 |
| 技术约束 | 本日不写业务实现；不得重构 `app.js`；不得关闭 `AD-002/003/005`；不得恢复复杂 shell；后续 UI 渲染必须用 safe DOM API | 文档中显式列出 Non-Scope | 必须 |
| 风险边界 | 禁止修改 `src/interface/desktop/tauri.conf.json`、Rust command、Thinking checkpoint、Tauri global API 迁移 | `git diff --stat` 证明本日无核心业务改动或仅新增文档 | 必须 |
| 测试基线 | JS 语法检查至少通过 | `node --check src/interface/web/app.js` | 必须 |
| 文档同步要求 | 输出 Day 1 基线证据，供 Day 2-7 使用 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 或 `docs/debt/B16-BASELINE-NOTES.md` | 必须 |
| 历史债务 / 相关缺陷 | `AD-007` 是主目标；`AD-004` 局部推进；`AD-008` 建 gate；`AD-001` 保持限制；`AD-002/003/005` 不处理 | `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md` 如存在；否则 `docs/debt/INDEX.md` 与相关 debt 文档 | 必须 |

### 探索补充栏

| 项目 | 内容 |
|---|---|
| 已知事实 | 当前已有聊天输入 `#aiChatInput`、全局 command palette 和 slash command 处理函数；尚无独立 slash palette 模块。 |
| 待确认问题 | 1. slash 命令清单应复用 `handleChatCommand` 还是新增 registry。2. 选择命令后应直接执行还是回填。3. `app.js` 当前输入事件能否低风险接入。4. 安全 gate 对历史 `innerHTML` 应 warn 还是 fail。 |
| 预期输出 | 可供 Day 2 直接实现的 SlashCommandItem V1 shape、模块 API 建议、接入点清单、安全扫描点清单。 |
| 停止条件 | 已定位核心入口、记录证据、定义 contract，并明确 Day 2 可以在哪些文件内开始实现。 |

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/01
- **角色**：Engineer
- **目标**：完成 B16 baseline audit，形成 Slash Palette V1 实现契约和安全 gate V1 扫描清单。
- **输入**：模块2全部路径与命令。
- **依赖关系**：无前置开发依赖；依赖当前 repo 可读。

### 2）输出交付物

- **变更文件**：
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`（可新建草稿或追加 Baseline/Contract）
  - 或 `docs/debt/B16-BASELINE-NOTES.md`（若团队决定 receipt Day 6 再合并）
- **核心修改点**：
  - 记录 Git 坐标、入口文件、关键函数、相关行号。
  - 定义 `SlashCommandItem` V1 shape。
  - 明确 `/` 打开、过滤、关闭、选择、disabled 行为。
  - 明确 Day 2 新模块 API 轮廓。
  - 明确 Security Gate V1 的 fail/warn 扫描项。
- **必须包含**：
  - `SlashCommandItem` 示例对象，字段至少包含 `id`、`trigger`、`title`、`description`、`category`、`riskLevel`、`enabled`。
  - `Non-Scope`：不关闭 `AD-002/003/005`，不恢复复杂 shell，不全量重构 `app.js/style.css`。
  - 安全渲染约束：后续 slash palette 禁止拼接 HTML，优先 `createElement` + `textContent`。
  - 后续验证命令清单。
- **禁止包含**：
  - 修改业务逻辑来“顺手实现” slash palette。
  - 删除或改写现有 `handleChatCommand`。
  - 把 Node smoke 或静态审计写成真实 WebView smoke。
  - 恢复 `bash/sh/pwsh/powershell` 到用户 shell allow-list。
- **交付证明**：
  - `rg` 输出摘要。
  - `node --check src/interface/web/app.js` 输出摘要。
  - `git diff --stat` 输出摘要。

### 3）规模与复杂度观察

- **推荐目标**：只新增/更新 1 份文档；如必须拆分，最多 2 份文档。
- **复杂度说明**：Day 1 是探索任务，不允许引入复杂状态机或业务代码。
- **禁止行为**：为了显得完成度高而提前开发 UI。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 前端主文件语法可解析 | `node --check src/interface/web/app.js` | 返工 |
| FMT | 文档无明显尾随空格 | `git diff --check -- docs/debt` | 返工或说明 |
| LINT | N/A，Day 1 不改 JS 业务 | N/A + 原因：只读审计/文档 | - |
| TEST | 基线命令真实执行 | `rg -n "handleChatCommand|sendChatMessage|showCommandPalette|aiChatInput" src/interface/web` | 返工 |
| ARCH | 不改分层、不改 Rust 业务 | `git diff --stat` 中不应出现 Rust/Tauri 核心业务文件 | 返工 |
| REAL | 不伪造 WebView smoke | 文档中必须出现“Node/静态审计不等同 WebView smoke” | 返工 |
| DOC | Baseline/Contract 文档存在 | `Test-Path docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 或 `Test-Path docs/debt/B16-BASELINE-NOTES.md` | 返工 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | 定位聊天输入框 DOM | `rg -n "aiChatInput" src/interface/web/index.html src/interface/web/app.js` | [ ] |
| FUNC | FUNC-002 | 定位发送消息入口 | `rg -n "sendChatMessage|chatInput.addEventListener" src/interface/web/app.js` | [ ] |
| FUNC | FUNC-003 | 定位 slash command 处理入口 | `rg -n "handleChatCommand|Handle slash commands|Unknown command" src/interface/web/app.js` | [ ] |
| FUNC | FUNC-004 | 定位现有 command palette 可复用边界 | `rg -n "showCommandPalette|renderCommandList|commandPalette" src/interface/web/app.js src/interface/web/index.html` | [ ] |
| CONST | CONST-001 | 确认现有模块目录 | `Get-ChildItem src/interface/web/modules -File` | [ ] |
| CONST | CONST-002 | 确认尚无 slash palette 模块 | `Test-Path src/interface/web/modules/slash-palette.js` 返回 `False` | [ ] |
| CONST | CONST-003 | 明确 B16 Non-Scope | 文档中 `rg -n "AD-002|AD-003|AD-005|Non-Scope" docs/debt` | [ ] |
| CONST | CONST-004 | 明确 safe DOM contract | 文档中 `rg -n "textContent|createElement|innerHTML" docs/debt` | [ ] |
| NEG | NEG-001 | 记录危险 DOM 历史点 | `rg -n "innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=|onload=" src/interface/web` | [ ] |
| NEG | NEG-002 | 记录 shell allow-list 风险点 | `rg -n "ALLOWED_COMMANDS|bash|sh|pwsh|powershell" src/engine/tool-system/src/shell.rs` | [ ] |
| NEG | NEG-003 | 不修改核心业务代码 | `git diff --stat` 只显示文档变更，或说明例外 | [ ] |
| NEG | NEG-004 | 不声明 WebView smoke 完成 | `rg -n "不等同.*WebView|Node.*不等同" docs/debt` | [ ] |
| UX | UX-001 | 定义 `/` 打开与过滤行为 | 文档中 `rg -n "打开|过滤|/c|Arrow" docs/debt` | [ ] |
| UX | UX-002 | 定义选择后策略 | 文档中 `rg -n "回填|直接执行|disabled" docs/debt` | [ ] |
| E2E | E2E-001 | 前端主文件语法检查 | `node --check src/interface/web/app.js` 退出码 0 | [ ] |
| High | HIGH-001 | 明确复杂 shell 不恢复 | 文档中 `rg -n "复杂 shell|OPEN BY DESIGN|不恢复" docs/debt` | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. 只写“已检查”但无命令输出摘要，返工。
2. 未记录执行时 branch/HEAD，返工。
3. 把本日探索写成完整功能交付，返工。
4. 未定位 `handleChatCommand` 或未说明未找到原因，返工。
5. 未定义 `SlashCommandItem` V1 shape，返工。
6. 删除或重写 `app.js` 业务逻辑，返工。
7. 伪关闭 `AD-002/003/005`，返工。
8. 恢复复杂 shell 或弱化 allow-list，返工。
9. 未声明 Node/静态验证与 WebView smoke 的边界，返工。
10. 产生新增 debt 但未声明，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | 是否找到 slash 与聊天输入主路径？ | [ ] | CF-B16-D01-001 | |
| 约束与回归用例（RG） | 是否明确不改 Tauri/WebView 债务？ | [ ] | RG-B16-D01-001 | |
| 负面路径用例（NG） | 是否列出危险 DOM 与 shell 回归扫描点？ | [ ] | NG-B16-D01-001 | |
| 用户体验用例（UX） | 是否定义 `/`、`/c`、上下键、Esc、Enter 行为？ | [ ] | UX-B16-D01-001 | |
| 端到端关键路径（E2E） | 是否跑过 `node --check app.js`？ | [ ] | E2E-B16-D01-001 | |
| 高风险场景（High） | 是否声明不恢复复杂 shell？ | [ ] | HIGH-B16-D01-001 | |
| 字段完整性 | Contract 字段是否完整？ | [ ] | 文档审查 | |
| 需求映射 | 每条结论是否映射到 AD-007/004/008？ | [ ] | 文档审查 | |
| 自测执行 | 是否完整跑过所有 Day 1 命令？ | [ ] | 命令输出摘要 | |
| 范围边界与债务 | 未覆盖项是否清楚标记？ | [ ] | 文档审查 | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/01 完成并提交

### 提交信息
- Commit: `docs(debt): record b16 baseline and slash contract`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 或 `docs/debt/B16-BASELINE-NOTES.md`

### 本轮目标与实际结果
- 目标: 完成 B16 slash palette baseline audit 与 V1 contract。
- 实际完成: `<列出已定位入口、contract、安全扫描点>`
- 未完成/不在范围: 未实现 UI；未关闭 AD-002/003/005；未做 WebView smoke。

### 关键决策记录
- DECISION-001: `<SlashCommandItem 字段选择>` - `<原因>`
- DECISION-002: `<选择后直接执行/回填策略>` - `<原因>`

### 自动化质量检查报告
```bash
git branch --show-current
git rev-parse HEAD
rg -n "handleChatCommand|sendChatMessage|showCommandPalette|aiChatInput" src/interface/web
rg -n "innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=|onload=" src/interface/web
node --check src/interface/web/app.js
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
- DEBT-TEST-B16-D01: 无 WebView smoke，本日为静态/Node 基线。
- DEBT-SCOPE-B16-D01: AD-002/003/005 不在本轮范围。

### 风险与回滚点
- 主要风险: 后续实现误把 command palette 和 slash palette 耦合过深。
- 回滚方式: `git revert <commit>` 或删除本日新增文档。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 找不到稳定输入入口或现有输入逻辑高度耦合 | 暂停实现，只输出接入方案与风险 | Day 2 先做最小接线 spike |
| QUALITY-001 | `node --check app.js` 失败 | 不继续派生实现，先记录失败并定位原因 | 返工 |
| TEST-001 | 无法运行 Node | 记录环境错误，改用静态 `rg` 证据并声明 `DEBT-TEST` | 有条件交付 |
| SCOPE-001 | 发现任务需要 Tauri WebView 才能判断 | 标记为用户实机验收项 | 不在 Day 1 关闭 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 1 Baseline Audit + Slash Contract** 通用高压任务！

### 技术背景
当前 B-16 批次目标是实现 `/` 命令建议面板 V1，局部推进前端模块化，并为安全回归建立轻量 gate。Day 1 只做基线审计和契约设计，不开发 UI，不关闭需要真实 Tauri/WebView 证据的债务。

### 关键约束
- 必须重新记录执行时 branch/HEAD。
- 必须定位 `#aiChatInput`、`sendChatMessage`、`handleChatCommand`、现有 command palette。
- 必须定义 `SlashCommandItem` V1 shape。
- 不得修改核心业务逻辑。

### 质量红线
- 10 项地狱红线生效。
- 刀刃表 16 项必须命令化。
- Node/静态验证不得冒充 WebView smoke。

### 工单并行矩阵
- B-16/01 Engineer：Baseline Audit + SlashCommandItem V1 Contract

### 验收铁律
- `node --check src/interface/web/app.js` 必须通过或如实记录失败。
- `rg` 必须覆盖 slash、command palette、危险 DOM、shell allow-list。
- 文档必须明确 Day 2 可实现的模块 API 与接入点。

### 收卷要求
- 附自动化质量检查摘要。
- 附刀刃表摘要。
- 诚实声明未覆盖 WebView smoke。

Ouroboros 闭环启动，B-16 Day 1，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
rg -n "slash|command|palette|handleChatCommand|showCommandPalette" src/interface/web tests docs
rg -n "aiChatInput|sendChatMessage|chatInput.addEventListener" src/interface/web/app.js src/interface/web/index.html
rg -n "innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=|onload=" src/interface/web
rg -n "ALLOWED_COMMANDS|bash|pwsh|powershell|sh" src/engine/tool-system/src/shell.rs
node --check src/interface/web/app.js
git diff --stat
git diff --check -- docs/debt
```
