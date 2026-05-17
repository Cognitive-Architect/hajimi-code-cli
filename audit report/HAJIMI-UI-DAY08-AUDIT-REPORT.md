# HAJIMI-UI Day 08 建设性审计报告

> 审计对象：`docs/roadmap/hajimi design/task/Day-08-Diff-Trace-Evidence-Panels.md`  
> 审计官：压力怪  
> 审计日期：2026-05-15  
> 关联阶段：HAJIMI-UI-INTERACTION-CORE Phase 4 Day 8

---

## 审计结论

- **评级**: **C级**
- **状态**: **返工**
- **与自测报告一致性**: **严重偏离**
- **核心原因**: Day 8 的 Diff Inspector 实现被后置同名 `renderDiffPreview(container, summary)` 覆盖，`showInspectorTab('diff-preview')` 与 `onEditProposed()` 调用的无参 Diff 渲染实际不会更新右侧 Diff 面板。

---

## 审计背景

### 项目阶段

HAJIMI-UI-INTERACTION-CORE Phase 4 Day 8：Diff Entry + Trace Summary，不做全量 Edit History 合流。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `app.js` | `src/interface/web/app.js` | 新增 Inspector Diff / Trace 渲染逻辑与事件挂载 | Engineer | 自称 `node --check` 通过 |
| 2 | `index.html` | `src/interface/web/index.html` | 右侧 Inspector 已有 Diff / Trace 面板 DOM | Engineer | 未提供 Day 8 新截图 |
| 3 | `day-8-evidence-panels.md` | `docs/receipts/ui-interaction/day-8-evidence-panels.md` | Day 8 回执，声明 Diff / Trace 面板可用 | Engineer | 与代码实际行为不一致 |

### 关键代码片段

```js
// src/interface/web/app.js:185
if (tabId === 'diff-preview') this.renderDiffPreview();
```

```js
// src/interface/web/app.js:249
renderDiffPreview() {
  const container = document.getElementById('inspectorDiffContent');
  // Day 8 期望的 Inspector Diff 渲染实现
}
```

```js
// src/interface/web/app.js:3109
renderDiffPreview(container, summary) {
  if (!container || !summary) return;
  // 旧 Operation Summary 的同名函数，后定义会覆盖 Day 8 函数
}
```

```js
// src/interface/web/app.js:4480-4482
this.currentDiffFile = edit.hunks && edit.hunks.length > 0 ? edit.hunks[0].file_path : null;
this.showEditPanel(edit);
this.renderDiffPreview();
```

---

## 质量门禁

- PASS 已读取 Day 8 工单：`Day-08-Diff-Trace-Evidence-Panels.md`
- PASS 已读取审计模板：`建设性审计模板.md`
- PASS 已读取示例报告：`B-09-AUDIT-REPORT-v3-示例.md`
- PASS 已读取 Day 8 回执：`day-8-evidence-panels.md`
- PASS 已抽查 `app.js` / `index.html`
- PASS 已执行语法闸门：`node --check src/interface/web/app.js`
- FAIL 核心功能验证：Diff Inspector 渲染函数存在同名覆盖

质量门禁结论：允许出报告，但不允许 A/B 放行。

---

## 审计目标

1. **Diff 入口**：点击/触发修改文件后能打开或刷新右侧 Diff Tab？
2. **Diff 最小渲染**：新增/删除/上下文样式可读，失败有空态或旧入口？
3. **Trace 摘要**：Trace Tab 至少显示空态或最近 N 条摘要？
4. **安全边界**：Accept / Reject 不会误触批量覆盖，旧入口未被移除？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| Diff Inspector 入口 | C | `currentDiffFile` 仅设置一次且未被消费；未发现 File Edit Summary 点击打开 Diff Tab 的实现。 |
| Diff Inspector 渲染 | D | Day 8 无参 `renderDiffPreview()` 被后置同名函数覆盖，实际调用不会更新 `#inspectorDiffContent`。 |
| Trace Inspector 摘要 | B | `renderTraceInspector()` 存在，并在 Trace 事件到达时刷新最近 15 条；但未使用 Day 7 的 Inspector guard，且部分字段未 escape。 |
| 旧入口 fallback | B | 旧 `showGitDiff(file)` / `/git diff` 路径仍存在；但 Diff Tab 失败时未提供“打开旧 Diff 入口”的显式 UI。 |
| Accept / Reject 安全 | C | `acceptAllEdits()` 对默认勾选 hunk 批量 apply，无确认；未满足 Day 8 “危险/批量操作必须 confirm”的安全检查。 |
| 回执真实性 | C | 回执声明 Diff 面板 live evidence 与 fail-safe，但代码存在覆盖缺陷且未提供 Day 8 截图证据。 |

整体健康度评级：**C级**。

---

## 关键疑问回答（Q1-Q3）

- **Q1：Diff Tab 是否真的能渲染 Day 8 最小预览？**  
  否。`app.js` 中存在两个同名 `renderDiffPreview` 定义，后定义的 Operation Summary 函数覆盖前定义的 Inspector 函数。`this.renderDiffPreview()` 无参调用实际直接 `return`。

- **Q2：Trace Tab 是否达到最低可用？**  
  基本达到。`renderTraceInspector()` 会在无事件时显示空态，在有事件时展示最近 15 条摘要。但该调用位于 trace subscription 消息处理内，未使用 `withInspectorGuard()`；若 Inspector 渲染异常，仍有影响订阅链路的风险。

- **Q3：Day 8 是否避免了误触批量覆盖？**  
  未达到。Inline Edit 面板的 hunk checkbox 默认勾选，`acceptAllEdits()` 点击后会批量调用 `apply_edits`，没有 confirm。

---

## 验证结果（V1-V8）

| 验证ID | 结果 | 证据 |
|:---|:---:|:---|
| V1 | PASS | `node --check src/interface/web/app.js` 退出码 0。 |
| V2 | PASS | `git diff --check` 仅有 CRLF warning，无 trailing whitespace 错误。 |
| V3 | FAIL | `renderDiffPreview` 定义位于 `app.js:249` 与 `app.js:3109`，存在覆盖。 |
| V4 | FAIL | `this.renderDiffPreview()` 无参调用位于 `app.js:185` 与 `app.js:4482`，会命中后置同名函数并因参数缺失直接返回。 |
| V5 | FAIL | `currentDiffFile` 仅在 `app.js:4480` 设置，未发现用于选择文件、打开 Tab 或渲染旧入口。 |
| V6 | PASS | `renderTraceInspector()` 位于 `app.js:292`，Trace 事件到达时 `app.js:1797` 调用。 |
| V7 | FAIL | `acceptAllEdits()` 位于 `app.js:4531`，未发现批量 apply 前的 `confirm()`。 |
| V8 | FAIL | `ui-reference-map.md` 未新增 Day 8 截图或 Day 8 acceptance；`day-8-evidence-panels.md` 未记录覆盖风险。 |

可复现命令：

```powershell
node --check src\interface\web\app.js
git diff --check
rg -n "renderDiffPreview\s*\(" src\interface\web\app.js
rg -n "currentDiffFile" src\interface\web\app.js src\interface\web\index.html
rg -n "acceptAllEdits|confirm\(" src\interface\web\app.js
```

用于定位同名覆盖的静态验证：

```powershell
@'
const fs = require('fs');
const src = fs.readFileSync('src/interface/web/app.js', 'utf8');
const defs = [...src.matchAll(/^\s{2}renderDiffPreview\s*\(/gm)]
  .map(m => src.slice(0, m.index).split(/\r?\n/).length);
console.log(JSON.stringify(defs));
process.exit(defs.length === 1 ? 0 : 1);
'@ | node -
```

实际输出：`[249,3109]`，退出码 1。

---

## 问题与建议

### 短期必须修复

1. 将 Day 8 Inspector Diff 函数改名为 `renderInspectorDiffPreview()`，并把 `showInspectorTab('diff-preview')`、`onEditProposed()` 调用点改为新函数。
2. 保留旧 Operation Summary `renderDiffPreview(container, summary)` 或将其改名为 `renderOperationDiffPreview()`，避免同名覆盖。
3. File Edit Summary 或 `onEditProposed()` 需要明确打开/刷新 `Diff 预览` Tab，或在 Diff Tab 空态中提供旧 `showGitDiff(file)` 入口。
4. `acceptAllEdits()` 在应用多个 hunk 或默认全选 hunk 时增加确认。

### 中期建议

- 给 `renderTraceInspector()` 加 `withInspectorGuard()` 包装，避免 Inspector 渲染异常影响 Trace 订阅。
- `renderTraceInspector()` 对 `ev.step`、`ev.iteration` 等展示字段统一 `escapeHtml()`。
- 在 `ui-reference-map.md` 增加 Day 8 acceptance 和截图证据。

### 长期建议

- 建立前端静态检查：禁止对象字面量内出现重复方法名，尤其是 `app.js` 这种大型单对象文件。

---

## 落地可执行路径

C级返工路径：

1. 拆分两个 `renderDiffPreview` 命名，保证 Day 8 Inspector Diff 与旧 Operation Summary Diff 各自有独立函数。
2. 增加一个最小 Node harness，验证 `onEditProposed()` 后 `#inspectorDiffContent` 包含文件名、删除行、新增行。
3. 增加一个静态检查，验证 `renderDiffPreview` 或新命名函数无重复定义。
4. 补充 Day 8 回执与 `ui-reference-map.md`，记录功能范围、降级路径、截图证据。

---

## 压力怪评语

**"哈？！"**（C级）

Trace 摘要像是走到门口了，但 Diff 预览在门口撞上了同名函数覆盖。语法没炸，不代表功能活着。Day 8 先返工，修完后再复审。

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-UI-DAY08-AUDIT-REPORT.md`
- 关联状态：HAJIMI-UI Day 8
- 当前结论：**返工后复审**

---

## 修复后复审结论（2026-05-15）

- **评级**: **A级**
- **状态**: **Go**
- **与修复后回执一致性**: **一致**
- **复审结论**: Day 8 已完成 A 级收尾。Diff Inspector 与 Operation Summary Diff 已拆分命名，消除了同名方法覆盖；`onEditProposed()` 与 Operation Summary 入口均能打开右侧 Diff 证据面板；Trace 摘要通过 guard 更新；批量 apply 前已确认。

### 复审修复项

| 原问题 | 修复结果 | 证据 |
|:---|:---:|:---|
| `renderDiffPreview()` 同名覆盖 | PASS | `renderInspectorDiffPreview()` / `renderDiffPreview()` / `renderOperationDiffPreview()` 各 1 个定义 |
| `onEditProposed()` 无法更新 Diff Inspector | PASS | `onEditProposed()` 调用 `openDiffPreview(this.currentDiffFile)` |
| Operation Summary 无 Diff 入口 | PASS | `.operation-summary-diff-entry` 打开 Inspector Diff tab |
| Trace 更新未隔离失败 | PASS | Trace subscription 调用 `safeRenderTraceInspector()` |
| Trace 字段未统一转义 | PASS | `step` / `iteration` / `details` 注入前使用 `escapeHtml()` |
| 批量 Apply 无确认 | PASS | `acceptAllEdits()` 在 `apply_edits` 前执行 confirm |
| 回执缺少修复后事实 | PASS | `day-8-evidence-panels.md` 与 `ui-reference-map.md` 已更新 |

### 复审验证结果（RV1-RV8）

| 验证ID | 结果 | 证据 |
|:---|:---:|:---|
| RV1 | PASS | `node --check src/interface/web/app.js` 退出码 0 |
| RV2 | PASS | `git diff --check` 仅 CRLF warning |
| RV3 | PASS | 静态 harness 确认 3 个 Diff renderer 命名无重复定义 |
| RV4 | PASS | 行为 harness 确认 `onEditProposed()` 渲染文件名、summary、old/new lines |
| RV5 | PASS | 行为 harness 确认 Trace 摘要转义 `<Observe>` / `<scan>` |
| RV6 | PASS | 行为 harness 确认 Operation Summary 的 `Diff 预览` 打开右侧 Inspector |
| RV7 | PASS | 静态 harness 确认 `confirm()` 位于 `apply_edits` 之前 |
| RV8 | PASS | Chrome headless 生成 Day 8 非空截图 |

### 复审评语

**"还行吧"**（A级）

这次把“语法活着但功能没接上”的洞补实了。Diff 入口、Trace 摘要、安全确认和证据文档都闭上了，Day 8 可以进入后续 Day 9。
