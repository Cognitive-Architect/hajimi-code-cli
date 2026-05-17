# B-16 Day 3 派单：Slash Palette Keyboard + Command Integration

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: Day 2 `slash-palette.js` 基础 UI
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 3 输入联动 + 键盘/鼠标选择 + 命令接入
- **轰炸目标**：在 Day 2 基础上实现 `/` 查询过滤、`ArrowUp/ArrowDown` active 切换、`Enter` 选择、`Esc` 关闭、鼠标点击选择，并与现有 `handleChatCommand` 保守对接。
- **任务性质**：功能开发 + 交互回归
- **输入基线**：完整技术背景见模块2。
- **输出要求**：Slash Palette V1 交互闭环 + 普通聊天发送不回归 + disabled 命令不执行。
- **通用铁律**：
  1. **数据诚实**：键盘交互无法真实点击时，必须说明自动化缺口。
  2. **零占位符**：不得留下“后续接入 command handler”的假完成。
  3. **自动化优先**：至少用静态命令验证键盘入口和 handler 接入；Day 4 再补 smoke。
  4. **最小必要复杂度**：只拦截 palette 打开时的有限按键。
  5. **债务透明化**：真实 WebView 点击仍不在本日关闭。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | `slash-palette.js`、`app.js` 中聊天输入事件、命令处理接入 | `src/interface/web/modules/slash-palette.js`、`src/interface/web/app.js` | 必须 |
| 现状基线 | Day 2 已有基础模块、open/close/updateQuery、基础 UI 和样式 | `rg -n "createSlashPalette|open\\(|close\\(|updateQuery" src/interface/web/modules/slash-palette.js` | 必须 |
| 目标结果 | `/` 打开；`/c` 过滤；上下键移动；Enter 选择；Esc 关闭；点击选择；disabled 不执行；普通 Enter 发送不被破坏 | 静态验证 + Day 4 smoke 预留 | 必须 |
| 技术约束 | `Enter` 只在 palette open 且存在 active item 时拦截；`Esc` 不清空输入；中高风险命令优先回填不自动执行 | 代码路径与注释/receipt 说明 | 必须 |
| 风险边界 | 不重写 `sendChatMessage`；不删除 `handleChatCommand`；不实现安全 gate；不改 Rust | `git diff --stat` | 必须 |
| 测试基线 | JS 语法检查 | `node --check src/interface/web/app.js` / `node --check src/interface/web/modules/slash-palette.js` | 必须 |
| 文档同步要求 | receipt 追加 Day 3 交互状态，或 Day 6 统一补 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 按需 |
| 历史债务 / 相关缺陷 | `AD-007` 进入 `IMPLEMENTED/PENDING-UI-SMOKE` 的前置；WebView smoke 仍未完成 | debt receipt | 必须 |

### 探索补充栏

本任务为已知解实现。若 Day 2 模块缺失或 API 与本工单不兼容，先修正模块 API，再做交互。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/03
- **角色**：Engineer
- **目标**：完成 slash palette 的键盘/鼠标交互和命令选择接入。
- **输入**：Day 2 产物、模块2目标范围。
- **依赖关系**：依赖 Day 2。

### 2）输出交付物

- **变更文件**：
  - `src/interface/web/modules/slash-palette.js`
  - `src/interface/web/app.js`
  - `src/interface/web/style.css`（仅按需补 active/disabled/focus 样式）
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`（按需）
- **核心修改点**：
  - 实现 `handleKeyDown(event)`。
  - 实现 `moveActive(delta)`。
  - 实现 `selectActive()`。
  - 实现鼠标点击选择。
  - 实现 `/xxx` 过滤 trigger/title/category。
  - 在 `app.js` 中把输入框 `input/keydown/blur` 与 palette 连接。
  - 选择命令后按风险策略回填或调用现有 command handler。
- **必须包含**：
  - `ArrowDown` / `ArrowUp` 循环移动。
  - `Enter` 选择时 `preventDefault()`，避免同时发送普通消息。
  - `Esc` 关闭且保留输入。
  - disabled command 不触发 `onSelect`。
  - 普通消息在 palette 未打开时走原发送路径。
- **禁止包含**：
  - 将所有 Enter 行为都拦截。
  - 选择任意命令都直接执行高风险操作。
  - 使用 `eval`、动态 Function、HTML inline handler。
  - 删除旧 command palette。
- **交付证明**：
  - `node --check` 双文件通过。
  - `rg` 验证 `ArrowDown|ArrowUp|Escape|Enter|preventDefault|handleKeyDown|selectActive`。
  - `git diff --stat` 证明范围可控。

### 3）规模与复杂度观察

- **推荐目标**：交互状态留在 `slash-palette.js`，`app.js` 只决定何时转发输入事件。
- **复杂度说明**：键盘导航是小型状态机；如 `handleKeyDown` 明显超过 50 行，需说明是否可拆。
- **禁止行为**：为了“通用键盘系统”引入大型抽象。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 模块语法通过 | `node --check src/interface/web/modules/slash-palette.js` | 返工 |
| BUILD-APP | 主文件语法通过 | `node --check src/interface/web/app.js` | 返工 |
| FMT | diff 无尾随空格 | `git diff --check -- src/interface/web` | 返工 |
| LINT | 无 inline handler / eval | `rg -n "onclick=|onkeydown=|eval\\(|new Function" src/interface/web/modules/slash-palette.js src/interface/web/app.js` 无新增危险命中 | 返工 |
| TEST | 静态交互入口存在 | `rg -n "ArrowDown|ArrowUp|Escape|Enter|handleKeyDown|selectActive" src/interface/web` | 返工 |
| ARCH | 接线仍在模块边界内 | `rg -n "moveActive|selectActive" src/interface/web/app.js` 不应出现大量实现逻辑 | 返工或说明 |
| REAL | disabled 不执行 | `rg -n "enabled|disabled" src/interface/web/modules/slash-palette.js` + 代码审查摘要 | 返工 |
| DOC | 记录 WebView 未验收 | receipt 或收卷中明确 | 返工 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | 实现 `handleKeyDown` | `rg -n "handleKeyDown" src/interface/web/modules/slash-palette.js src/interface/web/app.js` | [ ] |
| FUNC | FUNC-002 | 实现上下键导航 | `rg -n "ArrowDown|ArrowUp|moveActive" src/interface/web/modules/slash-palette.js` | [ ] |
| FUNC | FUNC-003 | 实现 Enter 选择 | `rg -n "Enter|selectActive|preventDefault" src/interface/web/modules/slash-palette.js src/interface/web/app.js` | [ ] |
| FUNC | FUNC-004 | 实现 Esc 关闭 | `rg -n "Escape|Esc|close\\(" src/interface/web/modules/slash-palette.js src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | 过滤匹配 trigger/title/category | `rg -n "trigger|title|category|filter" src/interface/web/modules/slash-palette.js` | [ ] |
| CONST | CONST-002 | disabled 不触发执行 | `rg -n "enabled|disabled|return false|return;" src/interface/web/modules/slash-palette.js` | [ ] |
| CONST | CONST-003 | 普通发送入口保留 | `rg -n "sendChatMessage\\(" src/interface/web/app.js` | [ ] |
| CONST | CONST-004 | 不删除旧 command palette | `rg -n "showCommandPalette|renderCommandList" src/interface/web/app.js` | [ ] |
| NEG | NEG-001 | 未新增 inline handler | `rg -n "onclick=|onkeydown=|onerror=|onload=" src/interface/web` 无新增命中 | [ ] |
| NEG | NEG-002 | 未新增 eval | `rg -n "eval\\(|new Function" src/interface/web` 无命中 | [ ] |
| NEG | NEG-003 | Enter 未全局吞掉 | 代码审查摘要 + `rg -n "preventDefault" src/interface/web/app.js src/interface/web/modules/slash-palette.js` | [ ] |
| NEG | NEG-004 | 无 fake 延迟/假成功 | `rg -n "setTimeout|mock|stub|fake" src/interface/web/modules/slash-palette.js` 无未解释命中 | [ ] |
| UX | UX-001 | Active 样式与 aria 状态 | `rg -n "active|selected|aria-selected|role" src/interface/web/modules/slash-palette.js src/interface/web/style.css` | [ ] |
| UX | UX-002 | Esc 保留输入策略有证据 | `rg -n "Escape|close\\(" src/interface/web/modules/slash-palette.js src/interface/web/app.js` + 代码审查摘要 | [ ] |
| E2E | E2E-001 | JS 语法检查通过 | `node --check src/interface/web/app.js; node --check src/interface/web/modules/slash-palette.js` | [ ] |
| High | HIGH-001 | 高风险命令不自动执行策略 | `rg -n "riskLevel|medium|high|fill|回填|enabled" src/interface/web/app.js src/interface/web/modules/slash-palette.js docs/debt` | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. Palette 打开时 Enter 同时发送普通聊天，返工。
2. Palette 未打开时普通 Enter 发送失效，返工。
3. disabled command 可执行，返工。
4. 高风险命令被自动执行且无确认/回填策略，返工。
5. `handleChatCommand` 被删除或破坏，返工。
6. 使用 inline handler 或 eval，返工。
7. 语法检查失败，返工。
8. 大范围重构 `app.js`，返工。
9. WebView 未验收却宣称 UI 完全通过，返工。
10. 新增复杂状态机但未说明，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | `/`、过滤、上下键、Enter、Esc 是否都有实现证据？ | [ ] | CF-B16-D03-001 | |
| 约束与回归用例（RG） | 普通发送和旧 command palette 是否保留？ | [ ] | RG-B16-D03-001 | |
| 负面路径用例（NG） | disabled/无匹配/未知命令是否处理？ | [ ] | NG-B16-D03-001 | |
| 用户体验用例（UX） | 选择态、关闭态、输入保留是否可理解？ | [ ] | UX-B16-D03-001 | |
| 端到端关键路径（E2E） | JS 语法是否通过？ | [ ] | E2E-B16-D03-001 | |
| 高风险场景（High） | 高风险命令是否保守处理？ | [ ] | HIGH-B16-D03-001 | |
| 字段完整性 | command item 字段是否被正确读取？ | [ ] | 代码审查 | |
| 需求映射 | 是否映射到 AD-007？ | [ ] | receipt | |
| 自测执行 | 是否跑完本日命令？ | [ ] | 输出摘要 | |
| 范围边界与债务 | Day 4 smoke 和 WebView 未验收是否声明？ | [ ] | receipt | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/03 完成并提交

### 提交信息
- Commit: `feat(frontend): wire slash palette keyboard and command selection`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `src/interface/web/modules/slash-palette.js`
  - `src/interface/web/app.js`
  - `src/interface/web/style.css`
  - `<receipt 如有>`

### 本轮目标与实际结果
- 目标: 完成 slash palette 键盘/鼠标交互和命令接入。
- 实际完成: `<列出 Arrow/Enter/Esc/click/filter/onSelect>`
- 未完成/不在范围: Day 4 补 Node smoke；真实 WebView 点击仍需用户验收。

### 关键决策记录
- DECISION-001: `<Enter 选择与普通发送隔离策略>` - `<原因>`
- DECISION-002: `<高风险命令回填策略>` - `<原因>`

### 自动化质量检查报告
```bash
node --check src/interface/web/modules/slash-palette.js
node --check src/interface/web/app.js
rg -n "ArrowDown|ArrowUp|Escape|Enter|handleKeyDown|selectActive" src/interface/web
git diff --stat
git diff --check -- src/interface/web
```

### 债务声明
- DEBT-TEST-B16-D03: 尚未新增 Node smoke，Day 4 覆盖。
- DEBT-UI-B16-D03: 尚未真实 Tauri/WebView 点击验收。

### 风险与回滚点
- 主要风险: 键盘拦截影响普通聊天发送。
- 回滚方式: 回退 `app.js` 输入接线和 `slash-palette.js` 键盘选择相关 diff。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 普通发送逻辑与 palette keydown 无法安全隔离 | 停止接入 Enter，只保留选择回填并声明风险 | 返工或拆分 |
| QUALITY-001 | `node --check` 失败 | 先修语法 | 返工 |
| UX-001 | 选择命令导致不可逆操作 | 改为只回填，不自动执行 | 返工 |
| TEST-001 | 无法证明 disabled 不执行 | Day 4 必须补自动测试；本日声明 `DEBT-TEST` | 有条件交付 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 3 Slash Palette Keyboard + Command Integration** 通用高压任务！

### 技术背景
Day 2 已新增 slash palette 基础模块。Day 3 要让它可用：过滤、键盘导航、选择、关闭、鼠标点击，并确保普通聊天输入不回归。

### 关键约束
- 只在 palette open 时拦截有限按键。
- disabled command 不执行。
- 高风险命令优先回填。
- 不删除旧 command palette 和 `handleChatCommand`。

### 质量红线
- 10 项地狱红线生效。
- Enter 双触发直接返工。
- WebView 未验收不得伪关闭。

### 工单并行矩阵
- B-16/03 Engineer：Slash Palette 键盘/鼠标选择 + 命令接入

### 验收铁律
- `node --check` 双文件通过。
- `rg` 能证明键盘入口、选择函数和 disabled 策略存在。
- 普通发送入口仍存在。

### 收卷要求
- 附自动化质量检查摘要。
- 附刀刃表摘要。
- 声明 Day 4 smoke 与 WebView 验收边界。

Ouroboros 闭环启动，B-16 Day 3，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
node --check src/interface/web/modules/slash-palette.js
node --check src/interface/web/app.js
rg -n "ArrowDown|ArrowUp|Escape|Enter|handleKeyDown|moveActive|selectActive|preventDefault" src/interface/web
rg -n "sendChatMessage\\(|handleChatCommand\\(|showCommandPalette" src/interface/web/app.js
rg -n "onclick=|onkeydown=|eval\\(|new Function" src/interface/web
git diff --stat
git diff --check -- src/interface/web
```
