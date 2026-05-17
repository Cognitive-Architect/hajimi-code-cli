# HAJIMI-DEBTFIX Day 05 建设性审计报告

> 审计对象：`docs/roadmap/hajimi debtFix/task/Day-05-DOM-Render-Audit-And-Escape.md`  
> 审计官：压力怪  
> 审计日期：2026-05-16  
> 关联阶段：HAJIMI-DEBTFIX Phase Day 05 / `CS-HAJIMI-003` 前置收敛  
> 当前状态：A 级 / Go

---

## 审计背景

### 项目阶段

HAJIMI-DEBTFIX Day 05：DOM 渲染审计 + 高风险 `innerHTML` 修复。目标是在 Day 06 开启 CSP / 收敛 Tauri 暴露面前，先把文件名、Git/工具输出、聊天/模型输出、错误消息等高风险 DOM 注入面降下来，并形成可供后续继续执行的 DOM audit 文档。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `app.js` | `src/interface/web/app.js` | 新增 `safeText()` / `escapeAttr()`，扩展 `escapeHtml()`；修复文件树、Git/Search/Problems、Trace、Checkpoint、Provider/MCP/Audit 等高风险 sink | Engineer | 自动闸门通过 |
| 2 | `SECURITY-DOM-AUDIT.md` | `docs/debt/SECURITY-DOM-AUDIT.md` | 记录 `innerHTML` 扫描、source/sink 分类、恶意样例验证步骤和 deferred DOM 债务 | Engineer | 文档存在 |

### 关键代码片段

```js
// 来自 src/interface/web/app.js:1810-1829
safeText(value) {
  return value == null ? '' : String(value);
},

escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = this.safeText(text);
  return div.innerHTML;
},

escapeAttr(text) {
  return this.safeText(text)
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
},
```

```js
// 来自 src/interface/web/app.js:828-866
folderEl.innerHTML = `
  <span class="tree-icon">
    📁
  </span>
  <span class="tree-label">${this.escapeHtml(node.name)}</span>
`;

fileEl.innerHTML = `
  <span class="tree-icon file-icon" style="color:${iconColor}">
    ${this.getFileIconSvg(node.name)}
  </span>
  <span class="tree-label">${this.escapeHtml(node.name)}</span>
`;
```

```js
// 来自 src/interface/web/app.js:4336-4358
list.innerHTML = checkpoints.map((chk, idx) => `
  <div style="border-bottom:1px solid var(--border);padding:6px 0;">
    <div style="display:flex;justify-content:space-between;">
      <span style="font-weight:bold;">${this.escapeHtml(chk.id || 'chk_' + idx)}</span>
      <span style="color:var(--fg-dim);">${this.escapeHtml(chk.timestamp || '')}</span>
    </div>
    <div style="display:flex;gap:4px;margin-top:4px;">
      <button class="modal-btn secondary btn-secondary checkpoint-restore-btn" data-id="${this.escapeAttr(chk.id || '')}">恢复</button>
      <button class="modal-btn secondary btn-secondary checkpoint-export-btn" data-id="${this.escapeAttr(chk.id || '')}">导出</button>
    </div>
  </div>
`).join('');
```

### 已知限制 / 环境问题

- `npm test -- --runInBand` 在当前环境失败：`jest is not recognized`。本次 Day 05 强制前端闸门以 `node --check src/interface/web/app.js` 为准；Jest 环境缺口需另行处理。
- 本轮未运行真实 Tauri UI smoke；`SECURITY-DOM-AUDIT.md` 已提供恶意文件名、聊天内容、Git/工具输出和 Markdown 链接的手动验证步骤。
- 当前工作区仍包含 Day 02-04 既有修改和 `src/MEMORY.md` 既有改动，不属于 Day 05 审计范围。
- `audit report/` 与 `docs/debt/` 被 `.gitignore` 忽略，后续提交报告和 DOM audit 文档需 `git add -f`。

---

## 质量门禁

- 已读取 Day 05 工单、建设性审计模板、B-09 审计报告示例。
- 已确认 `docs/debt/SECURITY-DOM-AUDIT.md` 存在。
- 已抽查 `src/interface/web/app.js` 中文件树、搜索/Git、Problems、Trace、Chat/Markdown、Checkpoint、Provider/MCP/Audit 等渲染路径。
- 已执行 `node --check src/interface/web/app.js`、`cargo check -p hajimi-desktop`、`git diff --check`。
- 已执行 `rg -n "innerHTML"` 并核对命中数为 102，与 audit 文档一致。
- 已执行 `rg -n "React|Vue|Vite|webpack" src/interface/web package.json`，无命中。
- 已检查 audit 文档包含 `fixed/sanitized/static/deferred` 分类、`DEBT-DOM-B05-001` 和 NEG-001~NEG-004 恶意样例步骤。

质量门禁满足出报告条件。

---

## 审计目标

1. DOM audit 文档是否存在，并记录 `innerHTML` 清单、source/sink/status？
2. 文件名、Git/工具输出、聊天/模型输出、错误消息等高风险来源是否已 escape 或文本渲染？
3. 剩余 `innerHTML` 是否被分类为 static / sanitized / deferred，而不是声称清零？
4. Day 06 CSP 是否能基于本日 audit 继续执行？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| DOM audit 文档 | A | `docs/debt/SECURITY-DOM-AUDIT.md` 存在，包含扫描 receipt、分类摘要、高风险表、恶意样例步骤和 deferred 债务。 |
| 高风险 sink 修复 | A | 文件树、搜索/Git/Problems、Trace、Checkpoint、Provider/MCP/Audit 等路径已用 `escapeHtml` / `escapeAttr` / `textContent` 收敛。 |
| Chat / Model 输出策略 | A | `formatText()` / `renderMarkdown()` 入口统一 `safeText()` + HTML escape；模型输出未被当作可信 HTML。 |
| 属性注入防护 | A | `data-*`、`option value`、按钮 id/path 等已大量改为 `escapeAttr`，checkpoint 移除动态 inline `onclick`。 |
| 剩余 innerHTML 诚实度 | A | 未声称清零；102 处命中按 fixed/sanitized/static/deferred 分类，`DEBT-DOM-B05-001` 明确后续范围。 |
| 自动化闸门 | A- | `node --check`、`cargo check -p hajimi-desktop`、`git diff --check` 通过；`npm test` 因当前环境缺少 `jest` 无法作为有效闸门。 |
| 范围控制 | A | 未引入 React/Vue/Vite/Webpack，未做大规模前端重构。 |

整体健康度评级：A 级。Day 05 达到“高风险 sink 真实收敛 + 剩余风险可追踪”的目标。

---

## 关键疑问回答（Q1-Q3）

**Q1：是否只改了代码、没有 audit 文档？**

否。`docs/debt/SECURITY-DOM-AUDIT.md` 已存在，并记录 `innerHTML` 命中数 102、helper 命中数 150、source/sink 表、NEG-001~NEG-004 恶意样例步骤和 `DEBT-DOM-B05-001` deferred 清单。

**Q2：模型输出 / 工具输出是否仍被当作可信 HTML？**

未发现。聊天流式响应走 `responseDiv.innerHTML = this.formatText(...)`，`formatText()` 入口先做 HTML escape；Thinking / Markdown 走 `renderMarkdown()`，同样先 escape，再只恢复受控 Markdown 子集，并对链接 URL 做 allow-list 和属性 escape。Git diff、工具输出、文件预览也按代码块或逐行文本 escape。

**Q3：剩余 `innerHTML` 是否影响 Day 6 CSP 执行？**

不阻塞 Day 6。剩余 `innerHTML` 主要是静态模板、受控图标、已 escape 的 syntax highlight / Markdown 子集，以及被声明为 deferred 的 UI 模板。audit 文档已经为 CSP 前置收敛提供了 source/sink/status 基线。

---

## 验证结果（V1-V13）

| 验证ID | 命令 | 结果 | 证据 |
|:---|:---|:---:|:---|
| V1 | `git branch --show-current` | PASS | `v3.8.0-batch-1` |
| V2 | `git rev-parse HEAD` | PASS | `d697414f42584a0d0c9c85346a6a692e691c4dad` |
| V3 | `Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-DOM-AUDIT.md` | PASS | `F:\hajimi-code-cli\docs\debt\SECURITY-DOM-AUDIT.md` |
| V4 | `rg -n "innerHTML" src/interface/web/app.js` | PASS | 102 命中，与 audit 文档一致 |
| V5 | `rg -n "escapeHtml|escapeAttr|safeText|textContent" src/interface/web/app.js` | PASS | 150 命中 |
| V6 | `node --check src/interface/web/app.js` | PASS | 退出码 0 |
| V7 | `cargo check -p hajimi-desktop` | PASS | 退出码 0 |
| V8 | `git diff --check` | PASS | 仅 CRLF 提示，无 whitespace error |
| V9 | `rg -n "React|Vue|Vite|webpack" src/interface/web package.json` | PASS | 无命中 |
| V10 | `rg -n "onclick=|onerror=|javascript:" src/interface/web/app.js` | PASS | 仅欢迎页静态 `onclick` 和 `sanitizeUrl` 注释命中；无动态 checkpoint onclick |
| V11 | `rg -n "NEG-001|NEG-002|NEG-003|NEG-004" docs/debt/SECURITY-DOM-AUDIT.md` | PASS | 恶意文件名、聊天、Git/工具输出、Markdown 链接步骤存在 |
| V12 | `rg -n "DEBT-DOM-B05-001|deferred|static" docs/debt/SECURITY-DOM-AUDIT.md` | PASS | deferred sink 清单存在 |
| V13 | `npm test -- --runInBand` | N/A | 当前环境缺少 `jest`，命令失败为 `'jest' is not recognized` |

---

## 问题与建议

### 短期

- Day 06 可以基于 `docs/debt/SECURITY-DOM-AUDIT.md` 继续推进 CSP / global API 收敛。
- 做一次真实 Tauri UI smoke：按 NEG-001~NEG-004 输入恶意文件名、聊天内容、Git/工具输出和 Markdown 链接，确认只显示文本。

### 中期

- Day 13/14 将 `formatText()`、`renderMarkdown()`、`escapeHtml()`、`escapeAttr()` 抽成独立安全 helper，并补可执行 fixture 测试。
- 为 `highlightCode()` 增加恶意代码片段 fuzz / fixture 测试，确认正则高亮不会破坏已 escape 文本。

### 长期

- 如果扩展系统未来支持第三方 catalog，`extension-icon` 的 inline color 应从 `escapeAttr` 升级为 CSS 值 allow-list。
- 修复当前 Node/Jest 环境缺口，让 `npm test` 能重新成为有效前端/JS 闸门。

---

## 评级结论

- 评级：A 级
- 状态：Go
- 与自测报告一致性：一致
- 地狱红线触发：否
- 是否需要返工：否

---

## 压力怪评语

“这次没有假装把 `innerHTML` 清零，反而是好事。高风险入口该 escape 的 escape，该转属性的转属性，剩下的写进 deferred 债务。安全工作最怕许愿，这份至少是能接着往 Day 06 CSP 走的真账。”

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-DEBTFIX-DAY05-AUDIT-REPORT.md`
- DOM audit 文档归档：`docs/debt/SECURITY-DOM-AUDIT.md`
- 关联状态：HAJIMI-DEBTFIX Day 05 / `CS-HAJIMI-003` 前置收敛
- 下一步建议：进入 Day 06 CSP / Tauri global API 收敛；`CS-HAJIMI-003` 整体仍应保持 `OPEN`，直到 CSP 与 global API 本体完成。
