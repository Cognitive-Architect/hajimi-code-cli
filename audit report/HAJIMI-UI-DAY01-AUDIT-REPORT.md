# HAJIMI-UI Day 01 建设性审计报告

> 审计对象：`docs/receipts/ui-interaction/` Day 1 Baseline & Scope Lock 交付物
> 审计官：Codex
> 审计日期：2026-05-14
> 关联派单：`docs/roadmap/hajimi design/task/Day-01-Baseline-Scope-Lock.md`

---

## 修复后复审结论

- **评级**: A
- **状态**: Go
- **与自测报告一致性**: 一致
- **复审时间**: 2026-05-15T11:15:00+08:00

Day 1 收尾修复已完成。DOM baseline 已从 125 修正为 128 个唯一 ID，补齐 `backupFileField`、`cancelBackup`、`confirmBackup`；JS binding receipt 已保存 raw `rg -n` 输出；文件规模补齐 `main.rs` 与行数口径；Command Palette、Moved Actions、UI Reference Map 已按 v1.1 表格字段补齐；`before-screenshot.png` 已生成。

复审命令结果：

| 验证项 | 结果 | 证据 |
|---|---:|---|
| DOM baseline 对齐 | PASS | `actual=128 baseline=128` |
| 前端语法检查 | PASS | `node --check src\interface\web\app.js` 退出码 0 |
| Day 1 receipt 文件 | PASS | 9 个文本 receipt + `before-screenshot.png` 均存在 |
| Protected DOM Contract | PASS | 覆盖 `modelSelectBtn` 与备份 modal 三个漏项 |
| Gate 0 | PASS | 可进入 Day 2 |

---

## 原始审计结论（修复前）

- **评级**: C
- **状态**: 有条件 Go，需补证后再进入 Day 2
- **与自测报告一致性**: 部分一致
- **刀刃表通过率**: 7/9 核心项通过，2/9 存在证据缺口
- **自动化闸门通过率**: 4/7 明确通过，3/7 需补证
- **地狱红线触发**: 触发 2 项轻中度红线：数据完整性不足、探索任务过度确定性结论

Day 1 的交付没有修改业务代码，`node --check src/interface/web/app.js` 通过，9 个指定 receipts 文件均存在。这说明执行边界基本守住了。

但多个 receipts 不是原始命令输出，而是人工摘要；`baseline-dom-ids.txt` 漏记 3 个真实 DOM id；`baseline-file-sizes.txt` 未覆盖 `src/interface/desktop/src/main.rs`；`command-palette-capability-audit.md`、`moved-actions-map.md`、`ui-reference-map.md` 未按 v1.1 模板字段落表。因此不建议直接放行 Day 2，应先补齐 Day 1 证据。

---

## 审计背景

### 项目阶段

HAJIMI UI-INTERACTION-CORE Phase 0 Day 1：Baseline & Scope Lock，只读基线、DOM 契约、入口地图。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 审计状态 |
|---:|---|---|---|---|
| 1 | `baseline-git.txt` | `docs/receipts/ui-interaction/baseline-git.txt` | 分支、HEAD、时间戳 | 部分通过，缺 `git status` |
| 2 | `baseline-file-sizes.txt` | `docs/receipts/ui-interaction/baseline-file-sizes.txt` | 前端 3 文件字节数 | 部分通过，缺 `main.rs` 与行数口径 |
| 3 | `baseline-dom-ids.txt` | `docs/receipts/ui-interaction/baseline-dom-ids.txt` | DOM id 清单 | 不通过，漏 3 个 id |
| 4 | `baseline-js-bindings.txt` | `docs/receipts/ui-interaction/baseline-js-bindings.txt` | JS 绑定摘要 | 部分通过，非原始 grep 输出 |
| 5 | `protected-dom-contract.md` | `docs/receipts/ui-interaction/protected-dom-contract.md` | 保护 DOM 契约 | 部分通过，缺 `modelSelectBtn` |
| 6 | `moved-actions-map.md` | `docs/receipts/ui-interaction/moved-actions-map.md` | 动作迁移地图 | 部分通过，字段不符合模板 |
| 7 | `command-palette-capability-audit.md` | `docs/receipts/ui-interaction/command-palette-capability-audit.md` | 命令面板审计 | 部分通过，缺 PASS/FAIL/UNKNOWN 表 |
| 8 | `ui-reference-map.md` | `docs/receipts/ui-interaction/ui-reference-map.md` | UI 对照地图 | 部分通过，缺验收方式与状态 |
| 9 | `day-1-summary.md` | `docs/receipts/ui-interaction/day-1-summary.md` | Day 1 总结 | 部分通过，REAL PASS 结论过满 |

### 已知限制/环境问题

- 本次审计未启动 Tauri UI，因此未产生截图验证。
- Day 1 工单文件本身要求 9 个 receipts，不包含 `before-screenshot.png`；但上层 roadmap v1.1 要求截图或 UNKNOWN 说明。该项按“上层规范缺口”记录，不作为单独 D 级阻断。

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 范围控制 | A | `git status --short` 为空，未发现业务代码修改 |
| 自动化语法检查 | A | `node --check src/interface/web/app.js` 退出码 0 |
| 交付物存在性 | A | Day 1 工单列出的 9 个 receipts 全部存在 |
| DOM 基线完整性 | C | 当前唯一 DOM id 为 128 个，baseline 记录 125 个，漏 `backupFileField`、`cancelBackup`、`confirmBackup` |
| JS 绑定证据质量 | C | 当前 grep 命中 349 行，但 receipt 仅为摘要，无法复现完整绑定边界 |
| Protected DOM Contract | B | 覆盖核心 Chat/FileTree/CommandPalette，但缺 Day 1 模板点名的 `modelSelectBtn` |
| Command Palette 审计 | C | 列出命令注册表，但未回答 category、Settings、Inspector Tab、危险确认等审计项 |
| Moved Actions Map | C | 有初版清单，但缺“是否危险 / 是否确认弹窗 / 验收方式 / 回滚入口”等必需字段 |
| UI Reference Map | C | 有区域映射，但缺“实现选择器 / 验收方式 / PASS/FAIL/UNKNOWN 状态” |

**整体健康度评级**：C 级。边界守住了，但证据链不够完整，不满足 v1.1 Gate 0 的“receipts 齐全且可复现”标准。

---

## 关键疑问回答（Q1-Q3）

### Q1：Day 1 是否真的保持只读，没有修改业务代码？

**结论**：是。

**证据**：

```powershell
git status --short
```

输出为空。当前工作区没有 `src/` 或其他未提交改动。

### Q2：DOM baseline 是否完整、可作为 Day 2 的 Protected Contract 输入？

**结论**：否，需要补齐。

**证据**：

```powershell
(rg -o 'id="[^"]*"' src\interface\web\index.html | Sort-Object -Unique | Measure-Object).Count
```

实测当前唯一 DOM id 为 **128**。

```powershell
(Select-String -Path "docs\receipts\ui-interaction\baseline-dom-ids.txt" -Pattern '^- ' | Measure-Object).Count
```

receipt 中记录为 **125**。

缺失项：

```text
backupFileField
cancelBackup
confirmBackup
```

其中 `cancelBackup`、`confirmBackup`、`backupFileField` 均被 `src/interface/web/app.js` 绑定使用。

### Q3：Command Palette 能力审计能否支撑 Day 6 的入口迁移决策？

**结论**：不能直接支撑。

`command-palette-capability-audit.md` 只列出了当前命令和一个“fully functional”的总结，没有按 roadmap 要求逐项标注：

- commands 注册表：PASS/FAIL/UNKNOWN
- 搜索：PASS/FAIL/UNKNOWN
- category：PASS/FAIL/UNKNOWN
- 快捷键：PASS/FAIL/UNKNOWN
- 打开 Settings：PASS/FAIL/UNKNOWN
- 打开 Inspector Tab：PASS/FAIL/UNKNOWN
- 危险操作 confirm：PASS/FAIL/UNKNOWN

审计结论过满，属于“可用性描述”，不是“可迁移决策证据”。

---

## 验证结果（V1-V10）

| 验证ID | 结果 | 证据 |
|:---|:---:|:---|
| V1 Git HEAD | 通过 | `git rev-parse HEAD` = `d9b99b858b1121ba700d71c72428f11b93073bcd`，与 receipt 一致 |
| V2 当前分支 | 通过 | `git branch --show-current` = `v3.8.0-batch-1`，与 receipt 一致 |
| V3 工作区只读 | 通过 | `git status --short` 输出为空 |
| V4 文件存在性 | 通过 | 9 个 Day 1 receipts 均存在 |
| V5 前端语法 | 通过 | `node --check src/interface/web/app.js` 退出码 0 |
| V6 文件规模 | 部分通过 | 前端 3 文件字节数一致；缺 `main.rs`，且未按上层计划记录 `wc -l` |
| V7 DOM id 基线 | 不通过 | 当前 128 个唯一 id，receipt 125 个，漏 3 个 |
| V8 JS 绑定基线 | 部分通过 | receipt 覆盖主链路摘要，但当前 grep 命中 349 行，未保存原始行号输出 |
| V9 Command Palette 审计 | 部分通过 | 命令列表与 `app.js` 注册表基本一致，但缺能力矩阵 |
| V10 UI Reference Map | 部分通过 | 有区域映射，但缺选择器验收与状态列 |

---

## 问题与建议

### 短期修复（进入 Day 2 前必须完成）

1. 重新生成 `baseline-dom-ids.txt`，保存完整原始清单，并修正总数为 128。
2. 将 `backupFileField`、`cancelBackup`、`confirmBackup` 加入 DOM baseline；如认为不是 P0 保护项，也必须在说明中解释原因。
3. 补充 `baseline-git.txt` 的 `git status --short` 输出。
4. 补充 `baseline-file-sizes.txt` 的 `src/interface/desktop/src/main.rs`，并明确“字节数”还是“行数”口径。
5. 将 `baseline-js-bindings.txt` 改为包含原始 `rg -n "getElementById|querySelector|dataset\.view|invoke\(" src/interface/web/app.js` 输出，或另存 `baseline-js-bindings-raw.txt`。
6. 在 `protected-dom-contract.md` 加入 `modelSelectBtn`，满足 Day 1 模板至少覆盖 Chat 输入、发送、模型选择、文件树的要求。
7. 按模板重写 `moved-actions-map.md` 字段：功能、原入口、新入口、是否危险、是否确认弹窗、验收方式、回滚入口。
8. 按模板重写 `command-palette-capability-audit.md`，逐项给 PASS/FAIL/UNKNOWN 与证据。
9. 按模板重写 `ui-reference-map.md`，加入实现选择器、验收方式、状态。
10. 如无法截图，新增 `before-screenshot.md` 或在 `day-1-summary.md` 写明 `before-screenshot.png = UNKNOWN`、原因、下一步。

### 中期建议

- 给 Day 1 receipts 增加一个 `verification-commands.md`，集中记录所有复跑命令和结果，避免摘要文件承担原始证据职责。
- 后续 Day 2/3 改 DOM 前，用脚本比对 Protected DOM Contract 中的 id 是否仍存在。

### 长期建议

- 把 DOM id 和 JS selector 审计做成可重复脚本，例如 `scripts/audit-ui-dom.ps1`，减少人工摘要漏项。

---

## 落地可执行路径

### C 级补证路径

条件：业务代码未改、语法检查通过、但 receipts 证据链不完整。

路径：

1. 不进入 Day 2。
2. 只修改 `docs/receipts/ui-interaction/*`。
3. 复跑 V1-V10。
4. 当 V7/V8/V9/V10 全部达到“通过”或诚实 `UNKNOWN` 后，再将 Gate 0 标记为通过。

### 升级为 B 级的标准

- DOM id 基线完整。
- JS 绑定有原始 grep 行号证据。
- 三个模板文件字段符合 roadmap。
- 不要求截图必须存在，但必须有 UNKNOWN 说明。

### 升级为 A 级的标准

- 在 B 级基础上，补齐 `before-screenshot.png` 或通过浏览器/手工方式提供真实截图证据。

---

## 压力怪评语

"哈？！边界守住了，但收据没写清。现在不是返工业务代码，是返工证据。别急着 Day 2 搬墙，先把 Day 1 的标签贴全。"

---

## 归档建议

- 审计报告归档：`audit report/HAJIMI-UI-DAY01-AUDIT-REPORT.md`
- 关联状态：HAJIMI-UI Day 01
- 下一步唯一动作：补齐 Day 1 receipts 后复审 Gate 0
