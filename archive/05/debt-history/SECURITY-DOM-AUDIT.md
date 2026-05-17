# Hajimi DOM Render Security Audit (B-05/15)

> 日期: 2026-05-16  
> 分支: `v3.8.0-batch-1`  
> HEAD: `d697414f42584a0d0c9c85346a6a692e691c4dad`  
> 对应债务: `CS-HAJIMI-003` / Tauri CSP 关闭 + Global Tauri API 开启  
> 范围: `src/interface/web/app.js`

---

## 1. 结论

Day 05 未追求 `innerHTML` 清零，而是优先收敛 CSP 开启前最危险的 DOM 注入面。

本轮已完成:

- 新增 `safeText()` / `escapeAttr()`，并让 `escapeHtml()` 统一处理 `null` / 非字符串输入。
- 文件树文件名、tab/path、搜索结果、Git 文件列表、Problems、Provider/Profile、MCP、Audit Log、Checkpoint、Trace/Edit History 等高风险数据源已改为文本或属性级 escape。
- Checkpoint 列表移除动态 `onclick="..."`，改为 `data-id` + `addEventListener`。
- Markdown 链接 URL 继续只允许 `http://` / `https://` / `mailto:`，并在写入 `href` 前做属性 escape。
- 聊天 / 模型 / 工具输出继续走 `formatText()` / `renderMarkdown()`，两者入口统一 `safeText()`；原始 `<script>` / `<img onerror>` 会先被转义。

剩余 `innerHTML` 主要是静态模板、受控图标、代码高亮、Markdown 子集渲染与若干需要 Day 13-14 模块化时继续收敛的 UI 模板。

---

## 2. 扫描 Receipt

### Git 坐标

```text
git branch --show-current
v3.8.0-batch-1

git rev-parse HEAD
d697414f42584a0d0c9c85346a6a692e691c4dad
```

### `innerHTML` 扫描

```text
rg -n "innerHTML" src/interface/web/app.js
当前命中数: 102
```

分类摘要:

| 区域 | 当前状态 | 说明 |
|---|---|---|
| Inspector / Trace summary | fixed / sanitized | `step/details/plan_summary` 等后端字段已 escape |
| Search / Git / Problems | fixed / sanitized | 文件名、搜索内容、diff、错误消息均文本 escape；关键 `data-*` 改为 `escapeAttr` |
| File tree | fixed | `node.name` 不再直接插入 HTML |
| Terminal / Output | fixed / textContent | 命令、输出和错误默认文本渲染 |
| Chat / Model / Tool output | sanitized | `formatText` / `renderMarkdown` 入口做 `safeText` + HTML escape；模型输出不视为可信 HTML |
| Session / Provider / MCP / Audit | fixed / sanitized | localStorage、配置、后端返回字段进入 HTML 前已 escape；属性使用 `escapeAttr` |
| Checkpoint | fixed | `chk.id` / `timestamp` escaped；动态 inline `onclick` 已移除 |
| Editor syntax highlight | deferred | 先 HTML escape 再插入高亮 span，保留为 `sanitized`，后续模块化时可单独测试 |
| Static UI templates | static | 欢迎页、空状态、按钮图标等无外部输入 |

### Helper 扫描

```text
rg -n "escapeHtml|escapeAttr|safeText|textContent" src/interface/web/app.js
当前命中数: 150
```

---

## 3. 高风险 Source / Sink 表

| Source | Sink | 风险 | 处理状态 | 验证方式 |
|---|---|---|---|---|
| 文件名 / 目录名 `node.name` | 文件树 `folderEl.innerHTML` / `fileEl.innerHTML` | 恶意文件名可注入 HTML | fixed | `node.name` 改为 `this.escapeHtml(node.name)` |
| 文件路径 `tab.id` / breadcrumb path | tab / breadcrumb `data-file` / `data-path` | 引号打断属性并注入事件 | fixed | 属性改为 `this.escapeAttr(...)` |
| 搜索输出 `grep` | search result item + `data-file` | 搜索内容或文件名注入 | fixed | 文本用 `escapeHtml`，属性用 `escapeAttr` |
| Git status / diff 输出 | Git 文件列表 / diff panel | Git 文件名或 diff 内容注入 | fixed | 文件属性 `escapeAttr`，diff 每行 `escapeHtml` |
| Terminal / build 输出 | terminal / output panel | 工具输出注入 | fixed | 输出逐行 `textContent` |
| Problems 输出 | problem item + `data-file` | 编译器消息或路径注入 | fixed | 消息文本 `escapeHtml`，属性 `escapeAttr` |
| Agent trace event | trace card / inspector | 后端事件字段注入 | fixed | `step/details/plan_summary/iteration` 统一 escape |
| Chat / model response | `responseDiv.innerHTML` | 模型输出被当作可信 HTML | sanitized | `formatText()` 先 escape HTML，再只允许小型 markdown 子集 |
| Thinking content | thinking markdown block | 模型思考内容注入 | sanitized | `renderMarkdown()` 先 escape HTML，链接 URL allow-list + attribute escape |
| MCP / provider / profile config | settings lists / options | 配置字段注入 HTML 或属性 | fixed | 文本 `escapeHtml`，option/data 属性 `escapeAttr` |
| Audit logs | audit table rows | 后端日志字段注入 | fixed | provider/model/status/time 均 escape |
| Checkpoint id | checkpoint buttons | `onclick` 拼接 id 可注入 JS | fixed | 移除 inline onclick，改为 `data-id` + listener |
| Extensions catalog | extension list | catalog 字段注入 | fixed / mostly static | id/text 已 escape；`iconColor` 仍属静态 catalog 约束 |

---

## 4. 恶意样例验证步骤

这些步骤用于 Tauri dev 或打包应用手动验收。预期均为“只显示文本，不执行脚本，不弹窗”。

### NEG-001 恶意文件名

```text
样例: <img src=x onerror=alert(1)>.txt
路径: workspace 内创建该文件后刷新文件树
预期: 文件树显示字面量文件名，不执行 onerror
覆盖: renderTreeNode -> escapeHtml(node.name)
```

### NEG-002 恶意聊天内容

```text
样例: <script>alert(1)</script>
路径: 在 AI 聊天输入该文本，或让模型返回该文本
预期: 消息区显示字面量标签，不执行 script
覆盖: addChatMessage / streamChat -> formatText / renderMarkdown
```

### NEG-003 恶意 Git / 工具输出

```text
样例文件名: "bad\" onmouseover=\"alert(1).txt"
路径: 让 git status / search / problems 面板展示该文件名
预期: 文件名显示为文本，data-file 属性不被打断
覆盖: renderGitFiles / renderSearchResults / renderProblems -> escapeAttr
```

### NEG-004 恶意 Markdown 链接

```text
样例: [x](https://example.com/" onmouseover="alert(1))
路径: thinking 或 markdown chat response
预期: href 属性被转义，不产生 onmouseover 属性
覆盖: renderMarkdown -> sanitizeUrl + escapeAttr
```

---

## 5. Deferred 债务

### `DEBT-DOM-B05-001`

本轮保留以下 sink，原因是它们需要更大范围 UI 模块化或专门测试，不属于 Day 05 的最小安全修复范围:

| Sink | 状态 | 原因 | 后续建议 |
|---|---|---|---|
| `highlightCode()` + editor `innerHTML` | sanitized / deferred | 当前先 escape HTML 再注入 syntax span，但正则高亮本身应有独立 fuzz 测试 | Day 13 拆 `security-dom.js` 后补 malicious code fixture |
| `renderMarkdown()` | sanitized / deferred | 已先 escape + URL allow-list，但仍是手写 Markdown 子集 | Day 13/14 抽成 helper 并补样例测试 |
| 大量静态 `innerHTML` UI 模板 | static / deferred | 为保持范围不失控，本日不做 DOM API 大重写 | 前端模块拆分时逐块收敛 |
| `extension-icon` inline color | static catalog / deferred | 当前扩展列表为内置 catalog；若未来支持第三方扩展元数据，需要 CSS 值 allow-list | 扩展系统开放前补 color sanitizer |

---

## 6. 质量检查

```text
node --check src/interface/web/app.js
结果: 通过

cargo check -p hajimi-desktop
结果: 通过

git diff --check
结果: 通过；仅输出当前工作区既有 CRLF 提示

rg -n "React|Vue|Vite|webpack" src/interface/web package.json
结果: 无命中
```

未执行真实 Tauri UI smoke；本文件已给出恶意文件名、聊天内容、Git/工具输出的手动验收步骤，供 Day 06 CSP 和后续实机验证继续使用。

---

## 7. 回滚点

```text
git restore src/interface/web/app.js docs/debt/SECURITY-DOM-AUDIT.md
```
