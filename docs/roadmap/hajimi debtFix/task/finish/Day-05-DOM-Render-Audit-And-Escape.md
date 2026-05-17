# Day 05 派单: DOM 渲染审计 + 高风险 innerHTML 修复

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 5，是 `CS-HAJIMI-003` Tauri 安全面收敛的前置步骤。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: DOM 渲染审计 + 高风险 innerHTML 修复
- **轰炸目标**: 审计 `src/interface/web/app.js` 中 `innerHTML` 使用点，创建 `SECURITY-DOM-AUDIT.md`，优先修复文件名、Git/工具输出、聊天/模型输出、错误消息等高风险来源的 HTML 注入面
- **任务性质**: 安全加固 + 前端质量修复
- **输入基线**: 完整技术背景见模块2
- **输出要求**: DOM audit 文档 + escape/text render helper + 高风险点改造 + `node --check` 通过
- **通用铁律**:
  1. 用户输入、文件名、Git 输出、工具输出、模型输出默认按文本渲染
  2. 能用 `textContent` / DOM API 的地方不用拼 HTML
  3. 允许静态模板保留 `innerHTML`，但必须在 audit 中标注来源为 static
  4. 不引入 React/Vite/Webpack
  5. 不做大规模前端重构，模块拆分留到 Day 13-14

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 记录当前分支和 HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `CS-HAJIMI-003` 当前 `OPEN / P0-P1` | 债务总表第 4.3 节 | 必须 |
| 前端目标 | 纯 HTML/CSS/JS，无 React/Vite/Webpack | `src/interface/web/app.js`, `index.html`, `style.css` | 必须 |
| 当前风险 | `app.js` 存在大量 `innerHTML` | `rg -n "innerHTML" src/interface/web/app.js` | 必须 |
| 高风险区域 | 文件树、搜索/Git diff、terminal、chat、trace、checkpoint | `app.js:815-981`, `:456-637`, `:1746-1899`, `:2933-3144`, `:4320-4362` | 必须 |
| 既有 helper | `escapeHtml`, `formatText`, `renderMarkdown` 等可能已有 | `rg -n "escapeHtml|formatText|renderMarkdown" src/interface/web/app.js` | 必须 |
| 文档输出 | DOM audit 文档 | `docs/debt/SECURITY-DOM-AUDIT.md` 或 roadmap debt 目录 | 必须 |
| 验证约束 | JS 语法必须通过 | `node --check src/interface/web/app.js` | 必须 |

### 探索补充栏

| 项目 | 内容 |
|---|---|
| 已知事实 | Tauri 目前 `csp: null` 且 global API 开启，DOM 注入后果被放大 |
| 待确认问题 | 哪些 `innerHTML` 是静态模板，哪些混入用户/文件/模型/工具输出 |
| 预期输出 | 一份按来源分类的 audit 文档和一批最高风险点修复 |
| 停止条件 | 高风险输入源都有 escape/textContent 策略；剩余 `innerHTML` 有分类与后续任务 |

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-05/15
- **角色**: Engineer
- **目标**: 在开启 CSP 前先降低前端 DOM 注入面
- **输入**: `app.js` innerHTML 扫描、高风险区域、债务总表 4.3
- **依赖关系**: 建议在 Day 2-4 后执行；Day 6 CSP 依赖本日 audit

### 2）输出交付物

- **变更文件**:
  - `src/interface/web/app.js`
  - `docs/debt/SECURITY-DOM-AUDIT.md`，或 `docs/roadmap/hajimi debtFix/debt/SECURITY-DOM-AUDIT.md`
- **核心修改点**:
  - 建立 `safeText` / `escapeHtml` 统一策略，优先复用已有 helper
  - 将文件名、路径、错误消息、Git/工具输出、模型输出中的未 escape 插值修掉
  - 对仍保留的 `innerHTML` 标注为 static / sanitized / needs-follow-up
  - 写恶意样例测试步骤
- **必须包含**:
  - `rg -n "innerHTML"` 输出计入 audit
  - 高风险项至少包含 source、sink、风险、处理状态、验证方式
  - 恶意文件名和恶意聊天内容的手动验证步骤
- **禁止包含**:
  - 引入前端框架或 bundler
  - 用 `setTimeout` 模拟安全验证
  - 把模型输出当可信 HTML
  - 为了消除 `innerHTML` 大面积重写 unrelated UI
- **交付证明**:
  - `node --check src/interface/web/app.js`
  - `rg -n "innerHTML" src/interface/web/app.js` 修复前后摘要
  - SECURITY-DOM-AUDIT 文档路径

### 3）规模与复杂度观察

- **推荐目标**: 本日只修高风险 sink，不追求 `innerHTML` 清零
- **复杂度说明**: Markdown 渲染若依赖 HTML，需要明确 sanitized 边界；不能假装天然安全
- **禁止行为**: 以大规模 UI 重写替代安全审计

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | 前端语法通过 | `node --check src/interface/web/app.js` | 返工 |
| FMT | JS 项目无统一格式器则 N/A | N/A，手动保持局部风格 | 说明原因 |
| LINT | 如有 lint 则运行 | `npm test` 或 `N/A + 原因` | 返工或声明 |
| TEST | 恶意输入有手动步骤 | audit 文档中记录 `<img src=x onerror=alert(1)>` 和 `<script>alert(1)</script>` | 返工 |
| ARCH | 不引入框架 | `rg -n "React|Vue|Vite|webpack" src/interface/web package.json` | 返工 |
| REAL | 高风险 sink 不再直接拼未 escape 输入 | audit 表 + diff | 返工 |
| DOC | audit 文档存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-DOM-AUDIT.md` | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `innerHTML` 清单已生成 | `rg -n "innerHTML" src/interface/web/app.js` 输出写入 audit | [ ] |
| FUNC | FUNC-002 | 高风险 source 分类完成 | audit 文档包含 file/git/tool/model/error/static 分类 | [ ] |
| FUNC | FUNC-003 | escape/text helper 明确 | `rg -n "escapeHtml|safeText|textContent" src/interface/web/app.js` | [ ] |
| FUNC | FUNC-004 | 文件树文件名渲染已处理 | `rg -n "fileEl.innerHTML|folderEl.innerHTML|escapeHtml" src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | Git/diff 输出默认 escaped | `rg -n "diffContent|escapeHtml|colored" src/interface/web/app.js` 并人工确认 | [ ] |
| CONST | CONST-002 | chat/model 输出策略明确 | `rg -n "formatText|renderMarkdown|responseDiv.innerHTML|escapeHtml" src/interface/web/app.js` | [ ] |
| CONST | CONST-003 | 错误消息不直接注入 | `rg -n "showToast|搜索失败|扫描失败|escapeHtml" src/interface/web/app.js` | [ ] |
| CONST | CONST-004 | `node --check` 通过 | `node --check src/interface/web/app.js` | [ ] |
| NEG | NEG-001 | 恶意文件名不执行 JS | 手动 receipt: `<img src=x onerror=alert(1)>` 只显示文本 | [ ] |
| NEG | NEG-002 | 恶意聊天内容不执行 JS | 手动 receipt: `<script>alert(1)</script>` 只显示文本 | [ ] |
| NEG | NEG-003 | 恶意 Git/工具输出不执行 JS | 手动或构造输出 receipt | [ ] |
| NEG | NEG-004 | 未引入框架/bundler | `rg -n "React|Vue|Vite|webpack" src/interface/web package.json` 无新增 | [ ] |
| UX | UX-001 | UI 文本仍可读 | 手动 smoke test 或截图路径 | [ ] |
| UX | UX-002 | 错误提示仍可理解 | 手动触发失败路径或 audit 示例 | [ ] |
| E2E | E2E-001 | Tauri dev 前置检查通过 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | [ ] |
| High | HIGH-001 | CSP 开启前高风险 sink 有处理状态 | audit 中每个 high risk sink 有 `fixed/deferred + reason` | [ ] |

---

## 【模块3-B】地狱红线

1. 没有 audit 文档，只改代码，返工
2. 声称 XSS 已清零但仍有未分类 `innerHTML`，返工
3. 把模型输出当可信 HTML，返工
4. 引入 React/Vite/Webpack，返工
5. 大规模重写 UI 导致范围失控，返工
6. `node --check` 失败仍收卷，返工
7. 用黑名单字符串过滤替代 HTML escape，返工
8. 漏掉文件名、Git/工具输出、聊天输出任一高风险来源，返工
9. 未记录无法修的 sink，返工
10. Day 6 CSP 无法基于本日 audit 执行，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 高风险 sink 是否修复或标注 | [ ] | FUNC-001~004 | |
| RG | 旧 UI 是否未明显回归 | [ ] | UX-001 | |
| NG | 恶意输入是否覆盖 | [ ] | NEG-001~003 | |
| UX | 错误提示是否仍可读 | [ ] | UX-002 | |
| E2E | Tauri/JS 检查是否通过 | [ ] | E2E-001 | |
| High | CSP 前置风险是否收敛 | [ ] | HIGH-001 | |
| 字段完整性 | audit 是否含 source/sink/status | [ ] | audit 文档 | |
| 需求映射 | 是否映射 `CS-HAJIMI-003` | [ ] | 债务总表 4.3 | |
| 自测执行 | 是否做过恶意样例测试 | [ ] | 手动 receipt | |
| 范围边界与债务 | 未修项是否声明 | [ ] | audit `deferred` | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-05/15 完成并提交

### 提交信息
- Commit: `security(interface/web): audit and harden high-risk dom rendering`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/web/app.js`
  - `docs/debt/SECURITY-DOM-AUDIT.md`

### 本轮目标与实际结果
- 目标: DOM 渲染审计并修高风险 innerHTML
- 实际完成: `<列出 fixed sink 和 deferred sink>`
- 未完成/不在范围: CSP 配置属 Day 6；模块拆分属 Day 13-14

### 自动化质量检查报告
- `rg -n "innerHTML" src/interface/web/app.js`: `<数量与分类摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- 恶意文件名/聊天/Git 输出手动测试: `<摘要或截图路径>`

### 债务声明
- `DEBT-DOM-B05-001`: `<列出 deferred sink 与原因>`

### 风险与回滚点
- 主要风险: 修改渲染路径导致 UI 文本展示变化
- 回滚方式: `git restore src/interface/web/app.js docs/debt/SECURITY-DOM-AUDIT.md`
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 发现需要模块化才能安全修复 | 只抽最小 helper，模块化留 Day 13 | 避免范围失控 |
| QUALITY-001 | `node --check` 失败 | 停止审计扩展，先修语法 | 返工 |
| TEST-001 | 无法构造恶意 Git 输出 | 用文件名和聊天内容完成手动验证，并记录 Git 待补 | 有条件交付 |
| SAFETY-001 | Markdown 渲染无法证明安全 | 降级为纯文本或声明 blocker | 不伪装安全 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 05 DOM 渲染审计 + 高风险 innerHTML 修复**。

### 关键约束
- 不引入框架或 bundler
- 不追求 `innerHTML` 清零，追求高风险 sink 真实收敛
- 所有 deferred sink 必须写入 audit
- Day 6 CSP 必须能接着本日文档执行

### 验收铁律
- `SECURITY-DOM-AUDIT.md` 存在
- `node --check src/interface/web/app.js` 通过
- 恶意输入样例不执行 JS
- 高风险 sink 有 fixed/deferred 状态

闭环启动，Day 05，执行。
