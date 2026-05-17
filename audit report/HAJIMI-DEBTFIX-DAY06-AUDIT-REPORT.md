# HAJIMI-DEBTFIX Day 06 建设性审计报告

> 审计对象：`docs/roadmap/hajimi debtFix/task/Day-06-CSP-Baseline-Global-Tauri-Plan.md`  
> 审计官：压力怪  
> 审计日期：2026-05-16  
> 关联阶段：HAJIMI-DEBTFIX Phase Day 06 / `CS-HAJIMI-003`  
> 当前状态：A 级 / Go（2026-05-16 收尾复审）

> 原始审计结论为 C 级 / 返工；本报告保留原始问题证据，并追加收尾复审结论。

---

## 审计背景

### 项目阶段

HAJIMI-DEBTFIX Day 06：CSP Baseline + Global Tauri API 迁移计划。目标是承接 Day 05 DOM audit，把 `src/interface/desktop/tauri.conf.json` 从 `csp: null` 推进到基础 CSP，并在暂不关闭 `withGlobalTauri` 时给出逐步迁移计划。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `tauri.conf.json` | `src/interface/desktop/tauri.conf.json` | 将 `csp: null` 改为基础 CSP；`withGlobalTauri` 仍为 `true` | Engineer | 编译通过 |
| 2 | `SECURITY-CSP-VERIFY.md` | `docs/debt/SECURITY-CSP-VERIFY.md` | 记录 CSP baseline、Global Tauri API 调用概览、迁移计划和未跑 Tauri dev 的待验项 | Engineer | 文档存在 |
| 3 | `HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | `docs/roadmap/hajimi debtFix/debt/HAJIMI_DEBT_CURRENT_STATUS_2026-05-15.md` | 原始审计发现未同步 Day 06 后的 4.3 当前状态；收尾已改为 `PARTIAL/VERIFY` | Engineer | 已完成 |

### 关键代码片段

```json
// 来自 src/interface/desktop/tauri.conf.json
{
  "app": {
    "withGlobalTauri": true,
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: data:; connect-src 'self' http://127.0.0.1:* http://localhost:*"
    }
  }
}
```

```js
// 原始返工证据，来自 src/interface/web/app.js:1201-1229
container.innerHTML = `
  <div class="editor-view active">
    <div class="welcome-page">
      ...
      <div class="welcome-link" onclick="app.openFile('src/interface/desktop/src/main.rs')">
        📄
        打开 main.rs
      </div>
      <div class="welcome-link" onclick="app.openFolder()">
        📁
        打开文件夹
      </div>
      <div class="welcome-link" onclick="app.cloneRepo()">
        🌿
        克隆仓库
      </div>
      ...
    </div>
  </div>
`;
```

### 已知限制 / 环境问题

- 本审计未启动真实 `cargo tauri dev`。Day 06 receipt 也明确未观察 WebView console，因此运行期 CSP violation 仍需实机确认。
- 当前工作区包含 Day 02-05 既有修改和 `src/MEMORY.md` 既有改动，不属于 Day 06 审计范围。
- `audit report/` 与 `docs/debt/` 被 `.gitignore` 忽略，后续提交报告和 receipt 时需要 `git add -f`。

---

## 质量门禁

- 已读取 Day 06 工单、建设性审计模板、B-09 审计报告示例。
- 已确认 `docs/debt/SECURITY-DOM-AUDIT.md` 和 `docs/debt/SECURITY-CSP-VERIFY.md` 均存在。
- 已抽查 `tauri.conf.json` 的 `withGlobalTauri` 与 `csp`。
- 已扫描 `src/interface/web/app.js` / `index.html` 的 Tauri API 使用和 CSP 相关 inline handler。
- 已执行 `node --check src/interface/web/app.js`、`cargo check -p hajimi-desktop`、`cargo clippy -p hajimi-desktop -- -D warnings`、`git diff --check`。

原始质量门禁满足出报告条件，但不满足 A 级放行条件。收尾复审已处理 blocker 和文档失真项，当前满足 A 级放行条件。

---

## 审计目标

1. CSP 是否从 `null` 变为可追踪 baseline？
2. CSP 是否过宽，或引入远程 CDN / 远程脚本？
3. `withGlobalTauri: true` 是否诚实保留并配套迁移计划？
4. receipt / 债务总表是否记录 CSP blocker 与当前真实状态？

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| CSP baseline | A | `csp: null` 已移除，包含 `default-src 'self'`、`script-src 'self'`、`img-src asset: data:`、本地 `connect-src`；收尾已清除欢迎页 inline `onclick` blocker。 |
| Global Tauri 状态 | A | `withGlobalTauri` 诚实保留为 `true`，receipt 说明 52 个 `__TAURI__` 命中，直接关闭会破坏核心流程。 |
| 迁移计划 | B | receipt 有按 Workspace/FileOps、Git/Search、Chat/LLM、Settings/MCP/Checkpoint/Trace 分组的迁移步骤；但缺少逐调用点级别的替代函数映射。 |
| 自动化闸门 | A | `node --check`、`cargo check -p hajimi-desktop`、`cargo clippy -p hajimi-desktop -- -D warnings` 均通过。 |
| 远程依赖控制 | A | 未发现远程 CDN 脚本；`http://localhost:*` / `127.0.0.1:*` 属 CSP 本地开发连接例外。 |
| CSP blocker 记录 | A | 欢迎页动态 inline `onclick` 已改为 `data-welcome-action` + `addEventListener`；`SECURITY-CSP-VERIFY.md` 已记录静态 blocker 修复和剩余实机 WebView smoke 待验。 |
| 债务文档同步 | A | 债务总表 4.3 已更新为 `PARTIAL/VERIFY`，区分 CSP baseline 已完成与 `withGlobalTauri` 仍待迁移。 |

整体健康度评级：A 级。CSP baseline、静态 blocker 清理、receipt、债务总表与自动化验证已经形成闭环；`withGlobalTauri` 仍按后续迁移债保留，不能视为 `CS-HAJIMI-003` 完全清偿。

---

## 关键疑问回答（Q1-Q3）

**Q1：CSP 是否仍为 `null`？**

否。`tauri.conf.json` 已改为基础 CSP 字符串，`rg -n '"csp"\s*:\s*null'` 无命中。这部分满足 Day 06 主目标。

**Q2：`withGlobalTauri` 是否被伪装成已关闭？**

否。配置仍为 `withGlobalTauri: true`，receipt 明确说明当前 `app.js` 有 52 个 `__TAURI__` 调用，直接关闭会中断文件树、Git、聊天、Provider、MCP、Checkpoint、Trace 等流程。这是诚实状态。

**Q3：CSP baseline 是否可以 Go？**

可以。`script-src 'self'` 不允许 inline event handler，原始审计发现的 `renderWelcome()` 4 个 `onclick="app...."` 已在收尾阶段改为事件监听绑定，receipt 也已记录该静态 blocker 的处理结果。剩余风险是未启动 `cargo tauri dev` 观察 WebView console，作为实机 smoke 待验项保留。

---

## 验证结果（V1-V15）

| 验证ID | 命令 | 结果 | 证据 |
|:---|:---|:---:|:---|
| V1 | `git branch --show-current` | PASS | `v3.8.0-batch-1` |
| V2 | `git rev-parse HEAD` | PASS | `d697414f42584a0d0c9c85346a6a692e691c4dad` |
| V3 | `rg -n 'withGlobalTauri\|csp' src/interface/desktop/tauri.conf.json` | PASS | `withGlobalTauri: true`；CSP baseline 字符串存在 |
| V4 | `rg -n '"csp"\s*:\s*null' src/interface/desktop/tauri.conf.json` | PASS | 无命中 |
| V5 | `rg -n "default-src 'self'" src/interface/desktop/tauri.conf.json` | PASS | 命中 CSP baseline |
| V6 | `rg -n "img-src.*asset:.*data:" src/interface/desktop/tauri.conf.json` | PASS | 命中 |
| V7 | `rg -n "connect-src.*127\.0\.0\.1" src/interface/desktop/tauri.conf.json` | PASS | 命中 |
| V8 | `Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-DOM-AUDIT.md` | PASS | Day 5 DOM audit 存在 |
| V9 | `Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-CSP-VERIFY.md` | PASS | CSP receipt 存在 |
| V10 | `rg -n "__TAURI__" src/interface/web/app.js` | PASS | 52 命中，与 receipt 一致 |
| V11 | `node --check src/interface/web/app.js` | PASS | 退出码 0 |
| V12 | `cargo check -p hajimi-desktop` | PASS | 退出码 0 |
| V13 | `cargo clippy -p hajimi-desktop -- -D warnings` | PASS | 退出码 0 |
| V14 | `rg -n 'default-src \*|script.*https://|cdn|<script[^>]*https' src/interface/web src/interface/desktop/tauri.conf.json` | PASS | 未发现远程脚本 / CDN / `default-src *` |
| V15 | `rg -n 'onclick=' src/interface/web src/interface/desktop/tauri.conf.json` | PASS | 收尾复审无命中；欢迎页改为 `data-welcome-action` + `addEventListener` |

---

## 问题与建议

### 已完成收尾

1. 已修复 `renderWelcome()` 的 inline `onclick`：
   - 将欢迎页按钮改为 `data-welcome-action` / `data-path`。
   - `container.innerHTML = ...` 后使用 `querySelectorAll(...).forEach(addEventListener)` 绑定。
   - 避免为解决问题在 CSP 中加入 `script-src 'unsafe-inline'`。
2. 已更新 `docs/debt/SECURITY-CSP-VERIFY.md`：
   - 记录 inline handler blocker 已修复。
   - 将运行期表述改为“未运行 WebView；静态 blocker 已处理；实机 CSP violation 待补验”。
3. 已更新债务总表 4.3：
   - `csp` 当前不再是 `null`。
   - `withGlobalTauri` 仍为 `true`。
   - 状态从 `OPEN` 改为 `PARTIAL/VERIFY`，不标 `CLEARED`。

### 建议补强

- 将 `withGlobalTauri` 迁移计划从区域级补到函数级：调用点、命令名、替代模块函数、优先级。
- Day 06 已在 receipt 中列出静态 CSP 风险扫描；后续仍建议补真实 `cargo tauri dev` WebView console smoke。
- Day 07 前补一轮 Tauri dev smoke，检查 WebView console 是否出现 CSP violation。

---

## 评级结论

- 评级：A 级
- 状态：Go
- 与自测报告一致性：一致，已追加收尾修正
- 地狱红线触发：否，原始 blocker 已修复并入档
- 是否需要返工：不需要；后续仅保留 WebView smoke 与 Global Tauri API 迁移债

---

## 压力怪评语

“这次收尾把关键点补齐了：CSP baseline 没回退，inline handler 没靠放宽策略糊过去，债务总表也承认 `withGlobalTauri` 还没关。Day 06 可以 A 级收卷，但别把它误读成 CSP/global API 整条债已经清零。”

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-DEBTFIX-DAY06-AUDIT-REPORT.md`
- CSP receipt：`docs/debt/SECURITY-CSP-VERIFY.md`
- 关联状态：HAJIMI-DEBTFIX Day 06 / `CS-HAJIMI-003`
- 下一步建议：进入 Day 07 前补一次真实 `cargo tauri dev` WebView smoke；Global Tauri API 迁移按 `docs/debt/SECURITY-CSP-VERIFY.md` 继续拆分推进。
