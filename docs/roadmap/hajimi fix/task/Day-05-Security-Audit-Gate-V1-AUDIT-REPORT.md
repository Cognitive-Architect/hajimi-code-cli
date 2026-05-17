# B-16 Day 5 建设性审计报告

> 审计对象: `Day-05-Security-Audit-Gate-V1.md`
> 审计官: Codex
> 审计日期: 2026-05-17
> 关联派单: B-16/05 Security Audit Gate V1

---

## 审计背景

### 项目阶段

B-16 Slash Palette & Safety Gate：在 slash palette 完成 Node smoke 后，建立轻量 Security Audit Gate V1，用自动化方式守住 CSP、DOM、安全 shell allow-list 与前端 file-op 绕行等已知高风险回退点。

### 交付物清单

| 序号 | 文件名 | 路径 | 内容摘要 | 交付者 | 自检结果 |
|---:|---|---|---|---|---|
| 1 | `security_audit_gate.js` | `tests/security/security_audit_gate.js` | 轻量安全 gate，扫描 CSP、inline handler、危险 HTML API、shell allow-list、file-op shell 绕行 | Engineer | 语法与运行通过 |
| 2 | `security_audit_allowlist.json` | `tests/security/security_audit_allowlist.json` | 历史危险 HTML API allowlist，包含 path/pattern/reason | Engineer | reason 机制存在 |
| 3 | `package.json` | `package.json` | 新增 `test:security-gate` 脚本 | Engineer | 脚本运行通过 |
| 4 | `DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 追加 Day 5 收卷、验证摘要、AD-008/AD-002/AD-001 状态 | Engineer | 与实现一致 |

### 关键代码片段

```javascript
// 来自 tests/security/security_audit_gate.js
function scanTauriConfig() {
  const raw = readText(tauriConfigPath);
  const config = JSON.parse(raw);
  const csp = config.app?.security?.csp;
  if (csp === null) {
    addFailure('tauri-csp-null', tauriConfigPath, findLine(raw, '"csp"'), 'Tauri CSP must not be null');
  }
  if (config.app?.withGlobalTauri === true) {
    addWarning('tauri-global-api', tauriConfigPath, findLine(raw, 'withGlobalTauri'), 'withGlobalTauri remains AD-002 debt and is warning-only in Gate V1');
  }
}
```

```javascript
// 来自 tests/security/security_audit_gate.js
function scanShellAllowList() {
  const raw = readText(shellPath);
  const block = raw.match(/const\s+ALLOWED_COMMANDS:[\s\S]*?=\s*&\[(?<body>[\s\S]*?)\];/);
  if (!block) {
    addFailure('shell-allow-list-missing', shellPath, 21, 'ALLOWED_COMMANDS block not found');
    return;
  }

  const commands = Array.from(block.groups.body.matchAll(/"([^"]+)"/g)).map(match => match[1]);
  const forbiddenShells = ['bash', 'sh', 'pwsh', 'powershell'];
  for (const shell of forbiddenShells) {
    if (commands.includes(shell)) {
      addFailure('shell-complex-shell-allowlist', shellPath, findLine(raw, `"${shell}"`), `ALLOWED_COMMANDS must not include ${shell}`);
    }
  }
}
```

### 已知限制 / 环境问题

- Gate V1 是文本模式轻量 gate，不是完整 SAST。
- `withGlobalTauri: true` 仍是 AD-002 债务，本日仅 warning，不关闭。
- 历史 `innerHTML` 通过 allowlist 降级为 warning；当前 allowlist 以文件 + pattern 粒度为主，后续应进一步收窄。

---

## 质量门禁

- 已读取 Day 5 工单、建设性审计模板、B-09 审计报告示例。
- 已读取新增 gate、allowlist、package script、Day 5 receipt。
- 已执行 `node --check tests/security/security_audit_gate.js`。
- 已执行 `node tests/security/security_audit_gate.js`。
- 已执行 `npm run test:security-gate`。
- 已执行 `node tests/frontend/day16_slash_palette_smoke.js`。
- 已执行规则反查、CSP/shell 现状反查、diff 范围和格式检查。

质量门禁全部满足，允许出报告。

---

## 审计目标

1. Gate 真实性：是否真实扫描项目文件，严重项是否有非 0 退出路径？
2. 规则覆盖：是否覆盖 CSP null、inline handler、危险 HTML API、shell allow-list、file-op shell 绕行？
3. 范围控制：是否未迁移 Tauri global API、未改 Rust shell、未新增依赖锁变动？
4. 文档诚实性：是否把 AD-008 标为 `PARTIAL/GATED`，并说明 Gate V1 覆盖有限？

---

## 审计结论

- 评级: A级
- 状态: Go
- 与自测报告一致性: 一致，附 1 个 P2 精度建议
- v3.0 刀刃表通过率: 16/16
- v3.0 自动化闸门通过率: 8/8
- v3.0 地狱红线触发: 否

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| Gate 可运行性 | A | `node tests/security/security_audit_gate.js` 与 `npm run test:security-gate` 均通过。 |
| Fail/Warn 分离 | A | CSP null、inline handler、slash-palette dangerous HTML、shell allow-list 回归等为 failure；`withGlobalTauri` 为 warning。 |
| 规则覆盖 | A | 覆盖工单要求的 CSP、DOM、shell allow-list、file-op shell 绕行。 |
| Allowlist 机制 | A- | JSON 每项包含 path/pattern/reason；粒度偏宽，建议后续收窄。 |
| 范围控制 | A | 未改 Tauri 配置、未改 Rust shell；`package.json` 仅新增脚本，`package-lock.json` 无 diff。 |
| 文档诚实性 | A | Receipt 明确 `AD-008 PARTIAL/GATED`、`AD-002 still open`、`AD-001 still open by design`。 |

整体健康度评级: A级。

---

## 关键疑问回答（Q1-Q3）

- Q1: gate 是否会永远返回 0？
  结论: 不会。脚本维护 `failures`，严重项进入 `addFailure`，`printSummary()` 在 failures 非空时设置 `process.exitCode = 1`。

- Q2: shell 复杂命令是否会被回归放回用户 allow-list？
  结论: 当前不会。gate 解析 `ALLOWED_COMMANDS` 只检查用户 allow-list，实际列表不含 `bash`、`sh`、`pwsh`、`powershell`。

- Q3: 是否把历史债务伪关闭？
  结论: 没有。`withGlobalTauri: true` 仍为 warning，receipt 明确 `AD-002` 仍 open；`AD-008` 是 `PARTIAL/GATED`，不是完整安全审计。

---

## 验证结果（V1-V16）

| 验证ID | 结果 | 证据 |
|:---|:---:|:---|
| V1 | 通过 | `git branch --show-current` -> `v3.8.0-batch-1` |
| V2 | 通过 | `git rev-parse HEAD` -> `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0` |
| V3 | 通过 | `node --check tests/security/security_audit_gate.js` 无输出 |
| V4 | 通过 | `node tests/security/security_audit_gate.js` -> `failures: 0`, `warnings: 105`, `PASS` |
| V5 | 通过 | `npm run test:security-gate` -> `failures: 0`, `warnings: 105`, `PASS` |
| V6 | 通过 | `node tests/frontend/day16_slash_palette_smoke.js` -> `PASS (8 scenarios)` |
| V7 | 通过 | `rg "csp|null|withGlobalTauri|onclick|...|reason" tests/security/...` 命中规则与 allowlist reason |
| V8 | 通过 | `rg '"csp"\|withGlobalTauri' src/interface/desktop/tauri.conf.json` 显示 CSP 非 null、global API 仍 true |
| V9 | 通过 | `ALLOWED_COMMANDS` 当前只含 git/cargo/npm/node/python3 等允许项，不含复杂 shell |
| V10 | 通过 | `rg "onclick=|onerror=|onload=|onmouseover=" src/interface/web` 无命中 |
| V11 | 通过 | `rg "innerHTML|insertAdjacentHTML" src/interface/web/modules/slash-palette.js` 无命中 |
| V12 | 通过 | `git diff -- src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs package-lock.json` 无输出 |
| V13 | 通过 | `git diff -- package.json` 仅新增 `test:security-gate` 脚本 |
| V14 | 通过 | `git diff --check -- tests/security package.json docs/debt` 退出码 0，仅 CRLF warning |
| V15 | 通过 | `Select-String ... tests/security ... '[ \t]+$'` 无尾随空格 |
| V16 | 通过 | `rg "AD-008|PARTIAL/GATED|AD-002|AD-001" docs/debt/...` 命中状态声明 |

---

## 刀刃表摘要

| 类别 | 通过情况 | 说明 |
|:---|:---:|:---|
| FUNC | 4/4 | gate 存在并覆盖 CSP、inline handler、shell allow-list。 |
| CONST | 4/4 | `withGlobalTauri` warning、危险 HTML API、allowlist reason、严重项 exit code 均实现。 |
| NEG | 4/4 | gate 非假通过；未改 shell.rs；未关闭 global API；未新增依赖。 |
| UX | 2/2 | 输出有 summary、fail/warn、file:line。 |
| E2E | 1/1 | gate 实际运行通过。 |
| High | 1/1 | 复杂 shell allow-list 回归会 fail。 |

---

## 问题与建议

- 短期: 无阻断项，允许进入 Day 6。
- 中期: P2 建议把 `security_audit_allowlist.json` 从文件级 `innerHTML` pattern 收窄到更精确的历史点，例如 line/hash/snippet，避免未来在同一文件新增危险 HTML API 时也只变成 warning。
- 长期: Gate V1 后续可补最小 mutation test 或 fixture test，用临时样例证明 CSP null、inline handler、slash-palette innerHTML、shell allow-list 回归都会触发非 0。

## 压力怪评语

"还行吧"（A级，安全门终于能自动跑了；allowlist 还可以更锋利一点。）

## 归档建议

- 审计报告归档: `docs/roadmap/hajimi fix/task/Day-05-Security-Audit-Gate-V1-AUDIT-REPORT.md`
- 关联状态: B-16/05 Go，允许进入 Day 6。
