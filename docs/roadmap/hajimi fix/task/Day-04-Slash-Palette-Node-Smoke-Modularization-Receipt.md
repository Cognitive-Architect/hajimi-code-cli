# B-16 Day 4 派单：Slash Palette Node Smoke + Modularization Receipt

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: Day 2-3 Slash Palette 实现
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 4 Slash Palette Node Smoke + AD-004 模块化证据
- **轰炸目标**：新增 `tests/frontend/day16_slash_palette_smoke.js`，用 mock DOM 验证 slash palette 的打开、过滤、键盘选择、Enter 回调、Esc 关闭、disabled 不执行和 safe DOM rendering，并在 receipt 中记录 AD-004 局部改善。
- **任务性质**：测试补强 + 文档证据
- **输入基线**：完整技术背景见模块2。
- **输出要求**：Node smoke 可重复运行通过 + receipt 明确 Node smoke 不等同 WebView smoke。
- **通用铁律**：
  1. **数据诚实**：测试通过数量和输出必须来自真实命令。
  2. **零占位符**：不得写空 smoke 或只 `console.log(PASS)`。
  3. **自动化优先**：核心交互必须由断言覆盖。
  4. **最小必要复杂度**：测试只覆盖 slash palette 模块，不搭建大型测试框架。
  5. **债务透明化**：真实 Tauri/WebView 点击验收仍不关闭。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | slash palette 模块与前端 smoke 测试 | `src/interface/web/modules/slash-palette.js`、`src/interface/web/app.js`、`tests/frontend/day16_slash_palette_smoke.js` | 必须 |
| 现状基线 | Day 2-3 已实现模块和交互；已有 Day13/Day14 smoke 可参考风格 | `Get-ChildItem tests/frontend`；`node tests/frontend/day13_workspace_modules_smoke.js`；`node tests/frontend/day14_sessions_thinking_modules_smoke.js` | 必须 |
| 目标结果 | 新 smoke 覆盖 `/` 打开、`/c` 过滤、ArrowDown/Up、Enter select、Esc close、disabled、safe DOM | `node tests/frontend/day16_slash_palette_smoke.js` | 必须 |
| 技术约束 | 测试必须真实断言 DOM 状态和回调次数；不能只调用函数不验证；恶意字符串必须以文本呈现，不作为 HTML 执行 | 测试断言与输出摘要 | 必须 |
| 风险边界 | 不改 Tauri；不改 Rust；不扩大 UI 功能；不把测试结果写成 WebView smoke | `git diff --stat` | 必须 |
| 测试基线 | JS 语法检查 + smoke | `node --check src/interface/web/app.js`、`node --check src/interface/web/modules/slash-palette.js`、`node tests/frontend/day16_slash_palette_smoke.js` | 必须 |
| 文档同步要求 | receipt 记录 AD-004 `PARTIAL/IMPROVED`、AD-007 `IMPLEMENTED/PENDING-UI-SMOKE` 建议 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 必须 |
| 历史债务 / 相关缺陷 | `AD-004` 前端模块化只局部改善，不完全关闭；`AD-007` 自动测试通过后仍需用户实机点验收 | receipt | 必须 |

### 探索补充栏

本任务为测试实现。若现有模块不是 CommonJS 可直接 require，允许在 smoke 中使用动态 import 或最小 DOM shim，但必须说明加载方式。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/04
- **角色**：Engineer
- **目标**：为 slash palette V1 建立可重复 Node smoke，并记录模块化改善证据。
- **输入**：Day 2-3 产物、模块2路径。
- **依赖关系**：依赖 Day 2-3。

### 2）输出交付物

- **变更文件**：
  - `tests/frontend/day16_slash_palette_smoke.js`
  - `src/interface/web/modules/slash-palette.js`（仅为可测性做小修）
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`
- **核心修改点**：
  - 构造 mock DOM 或复用项目已有测试工具。
  - 创建 input/container/button 等必要节点。
  - 构造含 enabled/disabled、low/high risk、恶意 title/description 的 commands。
  - 断言打开、过滤、active index、选择回调、关闭、安全渲染。
  - receipt 记录验证命令与输出摘要。
- **必须包含**：
  - 至少 7 个断言场景：open、filter、ArrowDown/Up、Enter select、Esc close、disabled、safe DOM。
  - 测试失败必须 `process.exit(1)` 或抛异常。
  - PASS 输出必须在断言全部通过后出现。
  - receipt 明确：Node smoke 不等同真实 WebView smoke。
- **禁止包含**：
  - 只打印 PASS 不断言。
  - 用 mock 成功替代模块真实调用。
  - 修改产品代码以适配测试而破坏 UI。
  - 将 `AD-004` 完全关闭。
- **交付证明**：
  - `node tests/frontend/day16_slash_palette_smoke.js` 输出摘要。
  - `node --check` 双文件输出摘要。
  - receipt 中含命令结果。

### 3）规模与复杂度观察

- **推荐目标**：测试文件自包含，少量 helper 如 `assert`、`createMockDom`、`dispatchKey`。
- **复杂度说明**：若 mock DOM 过重，优先缩小测试范围；不要引入完整浏览器自动化。
- **禁止行为**：为了测试而把产品模块改成假环境专用。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 产品 JS 语法通过 | `node --check src/interface/web/modules/slash-palette.js` | 返工 |
| BUILD-APP | app.js 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| TEST | Slash smoke 通过 | `node tests/frontend/day16_slash_palette_smoke.js` | 返工 |
| FMT | diff 无尾随空格 | `git diff --check -- tests/frontend src/interface/web docs/debt` | 返工 |
| LINT | 测试不是假 PASS | `rg -n "assert|throw new Error|process.exit\\(1\\)" tests/frontend/day16_slash_palette_smoke.js` | 返工 |
| ARCH | 新功能仍在模块内 | `rg -n "createSlashPalette" src/interface/web/modules/slash-palette.js src/interface/web/app.js` | 返工 |
| REAL | safe DOM 恶意字符串测试存在 | `rg -n "onerror|<img|textContent|safe" tests/frontend/day16_slash_palette_smoke.js` | 返工 |
| DOC | receipt 记录 Node/WebView 边界 | `rg -n "Node smoke|WebView|AD-004|AD-007" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 返工 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | smoke 文件存在 | `Test-Path tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| FUNC | FUNC-002 | 覆盖 `/` 打开 | `rg -n "open|/" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| FUNC | FUNC-003 | 覆盖 `/c` 过滤 | `rg -n "filter|/c|compact" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| FUNC | FUNC-004 | 覆盖 Enter 选择回调 | `rg -n "Enter|onSelect|selected" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| CONST | CONST-001 | 覆盖 ArrowDown/ArrowUp | `rg -n "ArrowDown|ArrowUp" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| CONST | CONST-002 | 覆盖 Esc 关闭 | `rg -n "Escape|Esc" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| CONST | CONST-003 | 覆盖 disabled 不执行 | `rg -n "disabled|enabled.*false" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| CONST | CONST-004 | 覆盖 safe DOM | `rg -n "onerror|<img|textContent|innerHTML" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| NEG | NEG-001 | 测试失败会抛错/退出 | `rg -n "throw new Error|process.exit\\(1\\)|assert" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| NEG | NEG-002 | 无纯假 PASS | `rg -n "console\\.log\\(.*PASS" tests/frontend/day16_slash_palette_smoke.js` + 断言存在 | [ ] |
| NEG | NEG-003 | 不新增产品危险 DOM | `rg -n "innerHTML|insertAdjacentHTML" src/interface/web/modules/slash-palette.js` 无危险命中 | [ ] |
| NEG | NEG-004 | 不改 Rust/Tauri | `git diff --stat -- src/interface/desktop src/engine` 为空或说明 | [ ] |
| UX | UX-001 | active index 有测试 | `rg -n "active|selected|index" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| UX | UX-002 | 空输入不打开或关闭策略有测试 | `rg -n "empty|blank|close|hidden" tests/frontend/day16_slash_palette_smoke.js` | [ ] |
| E2E | E2E-001 | smoke 真运行通过 | `node tests/frontend/day16_slash_palette_smoke.js` 退出码 0 | [ ] |
| High | HIGH-001 | receipt 不伪装 WebView | `rg -n "不等同.*WebView|Node smoke.*WebView" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. smoke 没有断言只打印 PASS，返工。
2. disabled 不执行未覆盖，返工。
3. safe DOM 恶意字符串未覆盖，返工。
4. `node tests/frontend/day16_slash_palette_smoke.js` 失败仍提交，返工。
5. 为测试引入大型新依赖，返工。
6. 改产品代码为测试假路径，返工。
7. 把 AD-004 完全关闭，返工。
8. 把 Node smoke 写成 WebView smoke，返工。
9. 未记录命令输出摘要，返工。
10. 新增 debt 未声明，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | open/filter/select/close 是否自动验证？ | [ ] | CF-B16-D04-001 | |
| 约束与回归用例（RG） | safe DOM 与模块边界是否覆盖？ | [ ] | RG-B16-D04-001 | |
| 负面路径用例（NG） | disabled/空输入/恶意字符串是否覆盖？ | [ ] | NG-B16-D04-001 | |
| 用户体验用例（UX） | active 状态和 Esc 关闭是否覆盖？ | [ ] | UX-B16-D04-001 | |
| 端到端关键路径（E2E） | smoke 是否真实运行？ | [ ] | E2E-B16-D04-001 | |
| 高风险场景（High） | 是否避免伪 WebView 结论？ | [ ] | HIGH-B16-D04-001 | |
| 字段完整性 | 测试命令和输出摘要是否完整？ | [ ] | receipt | |
| 需求映射 | 是否映射到 AD-007/AD-004？ | [ ] | receipt | |
| 自测执行 | 是否跑过全部命令？ | [ ] | 输出摘要 | |
| 范围边界与债务 | WebView 未验收是否声明？ | [ ] | receipt | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/04 完成并提交

### 提交信息
- Commit: `test(frontend): add slash palette node smoke`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `tests/frontend/day16_slash_palette_smoke.js`
  - `src/interface/web/modules/slash-palette.js`（如有）
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`

### 本轮目标与实际结果
- 目标: 用 Node smoke 验证 slash palette V1 行为。
- 实际完成: `<列出通过场景>`
- 未完成/不在范围: 真实 Tauri/WebView 点击验收。

### 关键决策记录
- DECISION-001: `<mock DOM 方案>` - `<原因>`
- DECISION-002: `<safe DOM 测试方式>` - `<原因>`

### 自动化质量检查报告
```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
git diff --stat
git diff --check -- tests/frontend src/interface/web docs/debt
```

### 债务声明
- DEBT-UI-B16-D04: Node smoke 不能替代真实 WebView smoke。
- DEBT-SCOPE-B16-D04: AD-004 仅局部改善，不完全关闭。

### 风险与回滚点
- 主要风险: mock DOM 与真实浏览器行为存在差异。
- 回滚方式: 删除 `tests/frontend/day16_slash_palette_smoke.js` 并回退可测性小修。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| TEST-001 | 模块无法在 Node 环境加载 | 为模块增加不影响浏览器的导出兼容，或声明测试债务 | 有条件交付 |
| QUALITY-001 | smoke flaky 或依赖真实计时 | 改为同步断言，不使用计时等待 | 返工 |
| SECURITY-001 | safe DOM 测试发现 HTML 被执行/解析 | 先修产品渲染 | 返工 |
| SCOPE-001 | 需要真实 WebView 才能判断 | 写入人工验收清单，不伪关闭 | 有条件交付 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 4 Slash Palette Node Smoke + Modularization Receipt** 通用高压任务！

### 技术背景
Day 2-3 已实现 slash palette 模块和交互。Day 4 要把核心 JS 行为变成可重复自动验证，并记录 AD-004 的局部模块化改善。

### 关键约束
- smoke 必须真实断言。
- 必须覆盖 safe DOM 与 disabled。
- 不把 Node smoke 冒充 WebView smoke。
- 不完全关闭 AD-004。

### 质量红线
- 10 项地狱红线生效。
- 假 PASS 直接返工。
- smoke 失败不得提交。

### 工单并行矩阵
- B-16/04 Engineer：Slash Palette Node Smoke + 模块化 receipt

### 验收铁律
- `node tests/frontend/day16_slash_palette_smoke.js` 必须通过。
- `node --check` 双文件通过。
- receipt 必须包含 Node/WebView 边界。

### 收卷要求
- 附自动化质量检查摘要。
- 附刀刃表摘要。
- 诚实声明 AD-004/AD-007 状态。

Ouroboros 闭环启动，B-16 Day 4，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
rg -n "assert|throw new Error|process.exit\\(1\\)" tests/frontend/day16_slash_palette_smoke.js
rg -n "Node smoke|WebView|AD-004|AD-007" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
git diff --stat
git diff --check -- tests/frontend src/interface/web docs/debt
```
