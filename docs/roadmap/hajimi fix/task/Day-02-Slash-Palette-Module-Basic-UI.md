# B-16 Day 2 派单：Slash Palette Module + Basic UI

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: Day 1 Baseline/Contract + B16 Roadmap/Execution Plan
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`
> **执行要求**: 开工前读取 Day 1 产物；若 Day 1 未产出 contract，先补最小 contract 后再实现。

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 2 Slash Palette 独立模块与基础 UI
- **轰炸目标**：新增 `src/interface/web/modules/slash-palette.js`，实现 slash palette V1 状态、过滤、安全 DOM 渲染、打开/关闭/updateQuery 基础能力，并在 `app.js` 做最小接线，`style.css` 增加 `.slash-palette-*` 样式。
- **任务性质**：功能开发 + 前端模块化
- **输入基线**：完整技术背景见模块2。
- **输出要求**：可运行模块 + 基础 UI + 语法检查通过 + 不破坏普通聊天输入。
- **通用铁律**：
  1. **数据诚实**：所有语法检查和 grep 结果必须来自真实命令。
  2. **零占位符**：不得留下空函数、假返回、TODO 冒充完成。
  3. **自动化优先**：至少跑 `node --check app.js` 与 `node --check slash-palette.js`。
  4. **最小必要复杂度**：`app.js` 只做接线，不把模块逻辑塞回大文件。
  5. **债务透明化**：无法验证真实 WebView 时必须声明，不伪关闭。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 新模块、主接线、样式 | `src/interface/web/modules/slash-palette.js`、`src/interface/web/app.js`、`src/interface/web/style.css` | 必须 |
| 现状基线 | 已有 `#aiChatInput` 和 slash command handler；modules 目录下已有安全/工作区/会话/thinking 模块；尚无 slash palette 模块 | `Test-Path src/interface/web/modules/slash-palette.js`；`rg -n "aiChatInput|handleChatCommand|sendChatMessage" src/interface/web` | 必须 |
| 目标结果 | 输入 `/` 时基础面板可由 JS 打开并渲染候选；`/c` 可过滤；Esc/Enter 等高级键盘能力 Day 3 完成 | `node --check src/interface/web/modules/slash-palette.js` + 静态检查 API | 必须 |
| 技术约束 | 模块使用 safe DOM：`createElement`、`textContent`、`appendChild`；禁止用 `innerHTML` 拼候选项；CSS 只新增 `.slash-palette-*` 前缀 | `rg -n "innerHTML|insertAdjacentHTML" src/interface/web/modules/slash-palette.js` 应无命中或只有明确安全注释 | 必须 |
| 风险边界 | 不重写现有 command palette；不改 Rust/Tauri；不改变 shell allow-list；不全量拆 `style.css` | `git diff --stat` | 必须 |
| 测试基线 | 前端语法检查 | `node --check src/interface/web/app.js` | 必须 |
| 文档同步要求 | 若 Day 1 receipt 已存在，追加 Day 2 实现摘要；否则 Day 6 统一补 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 按需 |
| 历史债务 / 相关缺陷 | `AD-007` 主交付；`AD-004` 局部改善；`AD-008` 后续 gate；WebView smoke 不在本日关闭 | debt receipt 或 active debt 文档 | 必须 |

### 探索补充栏

本任务为已知解实现。仅允许围绕 `app.js` 接入点做小范围确认；若接入点不稳定，触发 `ARCH-001` 熔断。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/02
- **角色**：Engineer
- **目标**：新增 slash palette 独立模块和基础 UI，并完成最小接线。
- **输入**：Day 1 contract、模块2路径与命令。
- **依赖关系**：依赖 Day 1 contract；不依赖 Day 3。

### 2）输出交付物

- **变更文件**：
  - `src/interface/web/modules/slash-palette.js`
  - `src/interface/web/app.js`
  - `src/interface/web/style.css`
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`（按需追加）
- **核心修改点**：
  - 导出 `createSlashPalette(options)`。
  - 模块内部维护 `isOpen`、`query`、`items`、`filteredItems`、`activeIndex`。
  - 实现 `open(query)`、`close(reason)`、`updateQuery(query)`、`isOpen()`。
  - 实现 safe render：候选项 trigger/title/description/category 用 `textContent`。
  - `app.js` 初始化 palette，提供 `getCommands` 与 `onSelect` 占位接线。
  - `style.css` 添加基础定位、列表、active、disabled、empty 样式。
- **必须包含**：
  - API：`createSlashPalette({ inputEl, containerEl, getCommands, onSelect, onOpen, onClose })`。
  - 基础 command item 字段兼容 Day 1 `SlashCommandItem`。
  - Killswitch：若项目已有 flag，使用 `window.__HAJIMI_FLAGS__.slashPaletteEnabled !== false`；若没有，不硬造大型配置系统，只做局部默认开启判断。
  - 无匹配候选时显示安全空状态或隐藏，策略需在代码/receipt 中说明。
- **禁止包含**：
  - 在 `slash-palette.js` 中使用 `innerHTML` 拼候选项。
  - 大范围重构 `sendChatMessage` 或 `handleChatCommand`。
  - 新增依赖、打包器、框架。
  - 把 disabled 命令设置为可执行。
- **交付证明**：
  - `node --check src/interface/web/modules/slash-palette.js`
  - `node --check src/interface/web/app.js`
  - `rg -n "createSlashPalette|textContent|slash-palette" src/interface/web`
  - `git diff --stat`

### 3）规模与复杂度观察

- **推荐目标**：`slash-palette.js` 单一职责；复杂 DOM 构建拆成少量私有函数即可。
- **复杂度说明**：本日只做基础 UI，不实现完整键盘状态机；如键盘逻辑提前实现，必须说明为何不等 Day 3。
- **禁止行为**：为了模块化而抽离无关 chat 逻辑。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | JS 语法通过 | `node --check src/interface/web/modules/slash-palette.js` | 返工 |
| BUILD-APP | 主前端文件语法通过 | `node --check src/interface/web/app.js` | 返工 |
| FMT | diff 无尾随空格 | `git diff --check -- src/interface/web` | 返工 |
| LINT | 无新增危险 HTML 拼接 | `rg -n "innerHTML|insertAdjacentHTML" src/interface/web/modules/slash-palette.js` 无候选拼接命中 | 返工 |
| TEST | Day 2 静态证据完整 | `rg -n "createSlashPalette|open\\(|close\\(|updateQuery" src/interface/web/modules/slash-palette.js` | 返工 |
| ARCH | 新功能进入模块 | `Test-Path src/interface/web/modules/slash-palette.js` 返回 true | 返工 |
| REAL | 不用假实现 | `rg -n "TODO|stub|mock|setTimeout" src/interface/web/modules/slash-palette.js` 无交付性假实现 | 返工或声明 |
| DOC | receipt 按需更新 | `rg -n "Day 2|slash-palette" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 或声明 Day 6 统一更新 | 返工或声明 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `slash-palette.js` 文件存在 | `Test-Path src/interface/web/modules/slash-palette.js` | [ ] |
| FUNC | FUNC-002 | 导出 `createSlashPalette` | `rg -n "export function createSlashPalette|module.exports.*createSlashPalette" src/interface/web/modules/slash-palette.js` | [ ] |
| FUNC | FUNC-003 | 实现 `open/close/updateQuery` | `rg -n "open\\(|close\\(|updateQuery\\(" src/interface/web/modules/slash-palette.js` | [ ] |
| FUNC | FUNC-004 | `app.js` 接入新模块 | `rg -n "createSlashPalette|slashPalette" src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | 渲染使用 `textContent` | `rg -n "textContent" src/interface/web/modules/slash-palette.js` | [ ] |
| CONST | CONST-002 | 候选项不使用 `innerHTML` 拼接 | `rg -n "innerHTML|insertAdjacentHTML" src/interface/web/modules/slash-palette.js` 无危险命中 | [ ] |
| CONST | CONST-003 | CSS 使用 `.slash-palette-*` 前缀 | `rg -n "\\.slash-palette" src/interface/web/style.css` | [ ] |
| CONST | CONST-004 | 不新增外部依赖 | `git diff -- package.json package-lock.json` 为空或说明 | [ ] |
| NEG | NEG-001 | 普通聊天发送入口未被删除 | `rg -n "sendChatMessage\\(|chatSendBtn.*sendChatMessage" src/interface/web/app.js` | [ ] |
| NEG | NEG-002 | 现有 `handleChatCommand` 未被删除 | `rg -n "async handleChatCommand|handleChatCommand\\(" src/interface/web/app.js` | [ ] |
| NEG | NEG-003 | disabled 字段被识别 | `rg -n "enabled|disabled" src/interface/web/modules/slash-palette.js` | [ ] |
| NEG | NEG-004 | 无交付性假实现 | `rg -n "TODO|stub|mock|setTimeout" src/interface/web/modules/slash-palette.js` 无未解释命中 | [ ] |
| UX | UX-001 | 面板有 active 样式 | `rg -n "active|selected" src/interface/web/modules/slash-palette.js src/interface/web/style.css` | [ ] |
| UX | UX-002 | 面板有空状态或关闭策略 | `rg -n "empty|no results|无匹配|close\\(" src/interface/web/modules/slash-palette.js` | [ ] |
| E2E | E2E-001 | 前端语法检查通过 | `node --check src/interface/web/app.js; node --check src/interface/web/modules/slash-palette.js` | [ ] |
| High | HIGH-001 | 本日不改 Rust shell allow-list | `git diff -- src/engine/tool-system/src/shell.rs` 为空 | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. 新模块不存在却声称完成，返工。
2. 使用 `innerHTML` 拼候选项，返工。
3. `app.js` 大面积重写聊天流程，返工。
4. 删除或破坏 `handleChatCommand`，返工。
5. 新增 React/Vue/Vite/Webpack 等依赖，返工。
6. disabled 命令仍可执行，返工。
7. 语法检查失败仍提交，返工。
8. 把基础 UI 写成 WebView 已验收，返工。
9. 未说明 Day 3 剩余键盘能力，返工。
10. 新增债务未声明，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | `/` 面板基础模块是否可创建和打开？ | [ ] | CF-B16-D02-001 | |
| 约束与回归用例（RG） | 旧聊天发送和 slash command handler 是否保留？ | [ ] | RG-B16-D02-001 | |
| 负面路径用例（NG） | disabled/空结果是否有处理？ | [ ] | NG-B16-D02-001 | |
| 用户体验用例（UX） | 面板是否有基础样式、active、empty？ | [ ] | UX-B16-D02-001 | |
| 端到端关键路径（E2E） | `app.js` 与模块语法是否通过？ | [ ] | E2E-B16-D02-001 | |
| 高风险场景（High） | 是否没有危险 HTML 拼接？ | [ ] | HIGH-B16-D02-001 | |
| 字段完整性 | API 与 item shape 是否完整？ | [ ] | 文档/代码审查 | |
| 需求映射 | 是否映射到 AD-007/AD-004？ | [ ] | receipt | |
| 自测执行 | 是否跑完本日命令？ | [ ] | 输出摘要 | |
| 范围边界与债务 | WebView 未验收是否声明？ | [ ] | receipt | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/02 完成并提交

### 提交信息
- Commit: `feat(frontend): add slash palette module and basic ui`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `src/interface/web/modules/slash-palette.js`
  - `src/interface/web/app.js`
  - `src/interface/web/style.css`
  - `<receipt 如有>`

### 本轮目标与实际结果
- 目标: 新增 slash palette 模块与基础 UI。
- 实际完成: `<列出 open/close/updateQuery/render/app.js 接线>`
- 未完成/不在范围: Day 3 完成完整键盘/鼠标选择；未做 WebView smoke。

### 关键决策记录
- DECISION-001: `<模块 API>` - `<原因>`
- DECISION-002: `<空状态/过滤策略>` - `<原因>`

### 自动化质量检查报告
```bash
node --check src/interface/web/modules/slash-palette.js
node --check src/interface/web/app.js
rg -n "createSlashPalette|textContent|slash-palette" src/interface/web
git diff --stat
git diff --check -- src/interface/web
```

### 债务声明
- DEBT-TEST-B16-D02: 未进行真实 Tauri/WebView 点击验收。
- DEBT-SCOPE-B16-D02: 键盘/鼠标完整选择留给 Day 3。

### 风险与回滚点
- 主要风险: app.js 接线影响普通聊天输入。
- 回滚方式: 删除 `slash-palette.js`，回退 `app.js` 与 `style.css` 中 slash palette 相关 diff。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | `app.js` 接入需要重写大段聊天逻辑 | 停止实现，只保留模块并记录接线风险 | Day 3/后续拆小步 |
| QUALITY-001 | `node --check` 失败 | 先修语法，不继续加功能 | 返工 |
| SECURITY-001 | 为渲染候选项必须使用 `innerHTML` | 停止，重写为 safe DOM | 返工 |
| TEST-001 | 无法在 Node 下检查模块语法 | 记录环境问题，并提供静态证据 | 有条件交付 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 2 Slash Palette Module + Basic UI** 通用高压任务！

### 技术背景
B-16 主目标是补上 `/` 命令建议面板。Day 2 要把新能力放入独立模块，完成基础显示与安全渲染，避免继续扩大 `app.js`。

### 关键约束
- 新增 `src/interface/web/modules/slash-palette.js`。
- `app.js` 只做轻量初始化与事件接线。
- 渲染候选项必须使用 safe DOM。
- 不新增框架或依赖。

### 质量红线
- 10 项地狱红线生效。
- `innerHTML` 候选拼接直接返工。
- Node 语法检查必须通过。

### 工单并行矩阵
- B-16/02 Engineer：Slash Palette 独立模块 + 基础 UI

### 验收铁律
- `node --check src/interface/web/modules/slash-palette.js` 通过。
- `node --check src/interface/web/app.js` 通过。
- `rg "textContent"` 能证明 safe DOM 渲染。

### 收卷要求
- 附质量检查摘要。
- 附刀刃表摘要。
- 诚实声明 Day 3 剩余交互能力与 WebView 未验收。

Ouroboros 闭环启动，B-16 Day 2，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
node --check src/interface/web/modules/slash-palette.js
node --check src/interface/web/app.js
rg -n "createSlashPalette|open\\(|close\\(|updateQuery|textContent|slash-palette" src/interface/web
rg -n "innerHTML|insertAdjacentHTML" src/interface/web/modules/slash-palette.js
git diff --stat
git diff --check -- src/interface/web
```
