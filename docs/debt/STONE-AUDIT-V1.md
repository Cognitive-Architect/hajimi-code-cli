# STONE-AUDIT-V1

> 状态: DAY-1-EVIDENCE-CAPTURED
> 目标: 建立史山体检 v1 的本地证据基线
> 禁止: 本轮不修改生产代码，不清债，不重构

## 1. Day 1 证据文件

- `docs/debt/stone-audit-v1-20260603-133316.txt`

## 2. 当前 Top 10 高风险文件

1. `src/interface/web/app.js` (5646 行)
2. `src/interface/web/style.css` (4610 行)
3. `src/interface/web/index.html` (640 行)
4. `src/interface/desktop/src/main.rs` (5079 行)
5. `src/engine/tool-system/src/shell.rs` (361 行)
6. `src/interface/desktop/tauri.conf.json` (36 行)
7. `src/interface/web/modules/inspector.js` (364 行)
8. `src/intelligence/agent-core/context_budget.rs` (1136 行)
9. `src/intelligence/agent-core/long_context_pack.rs` (987 行)
10. `src/intelligence/agent-core/context_receipt.rs` (716 行)

## 3. 当前结论

本轮只完成证据捕获，不宣称任何债务已清偿。所有建议验证命令（`node --check`、`npm run test:security-gate`、`cargo check --workspace`）均在本地通过。

## 4. 下一步

进入 Day 2：DOM 合约与冻结规则。

## 5. Day 2 冻结规则

### app.js

- 禁止新增超过 300 行的新功能逻辑。
- 新增功能必须优先进入 `src/interface/web/modules/`。
- 若必须新增 wrapper，必须写明迁移目标。

### main.rs

- 禁止新增无归属 Tauri command。
- 新 command 必须先归类到 `commands/*` 规划。
- 涉及 keyring / provider / checkpoint / shell 的改动必须单独审查。

### style.css

- 禁止新增无区域归属的全局 class。
- 禁止随手覆盖 `.button` / `.panel` / `.item` 等泛名选择器。
- 新样式必须写入明确分区或规划拆分文件。

### index.html

- 禁止无记录改 DOM ID。
- DOM 结构变动必须同步 `docs/frontend/DOM-CONTRACT.md`。

### shell.rs

- 禁止恢复 `bash` / `sh` / `pwsh` / `powershell` 用户白名单。
- 禁止为体验放开 `rm` / `sudo` / pipe / redirect。

### tauri.conf.json

- 禁止将 `csp` 改回 `null`。
- 禁止将 `withGlobalTauri` 改回 `true`。

## 6. Day 2 状态

> 状态: DAY-2-CONTRACT-AND-FREEZE-RULES-CAPTURED

本轮只建立 DOM 合约与冻结规则，不修改生产代码，不清债，不重构。

下一步进入 Day 3：Verification & Closure。
