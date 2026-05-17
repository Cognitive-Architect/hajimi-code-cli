# B-16 Day 5 派单：Security Audit Gate V1

> **所属批次**: B-16 Slash Palette & Safety Gate
> **任务来源**: B16 Roadmap Phase 3 / AD-008 SecurityAuditTool quality
> **派单生成基线**: branch `v3.8.0-batch-1`, HEAD `ece6cd9b874eecd0c852e3a7a1fd2908e37b86b0`

---

## 【模块1】饱和攻击头部（通用增强版）

- **火力配置**：1 Agent（Engineer）
- **任务名称**：B-16 Day 5 Lightweight Security Audit Gate V1
- **轰炸目标**：新增 `tests/security/security_audit_gate.js`，扫描 Tauri CSP、inline event handler、危险 HTML API、shell allow-list 回归和文件操作绕行风险，并可选新增 allowlist 与 package script。
- **任务性质**：安全门禁 + 测试基础设施
- **输入基线**：完整技术背景见模块2。
- **输出要求**：安全 gate 可运行；严重项 fail；已知历史点 warn/allowlist；receipt 明确覆盖范围和限制。
- **通用铁律**：
  1. **数据诚实**：gate 输出必须来自真实运行。
  2. **零占位符**：不能只写安全说明而没有脚本。
  3. **自动化优先**：安全回归必须可命令执行。
  4. **最小必要复杂度**：V1 固定扫描已知高风险点，不做完整 SAST 平台。
  5. **债务透明化**：Gate V1 是 partial/gated，不是完整安全审计。

---

## 【模块2】输入基线（完整技术背景，零占位符）

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git坐标 | 当前分支 + HEAD SHA | `git branch --show-current` / `git rev-parse HEAD` | 必须 |
| 目标范围 | 安全 gate、allowlist、package script、receipt | `tests/security/security_audit_gate.js`、`tests/security/security_audit_allowlist.json`、`package.json`、`src/interface/desktop/tauri.conf.json`、`src/interface/web/*`、`src/engine/tool-system/src/shell.rs` | 必须 |
| 现状基线 | CSP baseline 已存在但 `withGlobalTauri: true` 仍是已知 warning；前端存在历史 `innerHTML`；shell 用户 allow-list 不应恢复 `bash/sh/pwsh/powershell` | `rg -n "\"csp\"|withGlobalTauri" src/interface/desktop/tauri.conf.json`；`rg -n "innerHTML|insertAdjacentHTML|onclick=|onerror=" src/interface/web`；`rg -n "ALLOWED_COMMANDS|bash|sh|pwsh|powershell" src/engine/tool-system/src/shell.rs` | 必须 |
| 目标结果 | `node tests/security/security_audit_gate.js` 可运行；严重项返回非 0；现有已知项通过 warn 或 allowlist 解释 | gate 输出摘要 | 必须 |
| 技术约束 | `csp: null` fail；用户 shell allow-list 恢复复杂 shell fail；inline event handler fail；`withGlobalTauri: true` warn；历史 `innerHTML` 可 allowlist 但必须 reason | gate 规则和 allowlist | 必须 |
| 风险边界 | 不迁移 Tauri global API；不重写历史 DOM；不恢复复杂 shell；不做 WebView smoke | receipt 说明 | 必须 |
| 测试基线 | slash smoke 与 gate 都应能跑 | `node tests/frontend/day16_slash_palette_smoke.js`；`node tests/security/security_audit_gate.js` | 必须 |
| 文档同步要求 | receipt 更新 AD-008 `PARTIAL/GATED` 建议 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 必须 |
| 历史债务 / 相关缺陷 | `AD-008 SecurityAuditTool quality` 主目标；`AD-002` global API 仍不关闭；`AD-001` shell 限制保持 | active debt / receipt | 必须 |

### 探索补充栏

本任务为已知解实现。唯一可探索项是历史 `innerHTML` 数量与 allowlist 结构；若历史点过多，V1 采用 warn + allowlist，不扩大为全量 DOM 重构。

---

## 【模块3】工单矩阵（通用高压版）

### 1）基础信息

- **工单编号**：B-16/05
- **角色**：Engineer
- **目标**：建立轻量安全审计 gate，覆盖 B16 已知高风险回退点。
- **输入**：模块2目标范围与 Day 4 产物。
- **依赖关系**：可与 Day 4 后并行，但最终 receipt 依赖 Day 4 状态。

### 2）输出交付物

- **变更文件**：
  - `tests/security/security_audit_gate.js`
  - `tests/security/security_audit_allowlist.json`（可选，但推荐）
  - `package.json`（可选新增 `"test:security-gate"`）
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`
- **核心修改点**：
  - 读取 `tauri.conf.json` 检查 `csp` 不为 null。
  - 扫描 `src/interface/web` inline event handler：`onclick=`、`onerror=`、`onload=`、`onmouseover=`。
  - 扫描危险 HTML API：`innerHTML`、`insertAdjacentHTML`，对历史点用 allowlist reason 或 SECURITY 注释处理。
  - 解析 `ALLOWED_COMMANDS`，确认用户 allow-list 不包含复杂 shell。
  - 扫描前端是否绕过 dedicated file commands 通过 shell 做 file ops。
  - 输出 fail/warn summary。
- **必须包含**：
  - 严重项 exit code 非 0。
  - `withGlobalTauri: true` 是 warning，不是本批 fail。
  - allowlist 每项必须包含 path/pattern/reason。
  - receipt 明确 Gate V1 覆盖范围有限。
- **禁止包含**：
  - 修改 Tauri global API 配置作为本日目标。
  - 把所有历史 `innerHTML` 一刀切改坏。
  - 让 gate 永远返回 0。
  - 在 allowlist 中无 reason 放行。
- **交付证明**：
  - `node tests/security/security_audit_gate.js` 输出摘要。
  - 若新增 script，`npm run test:security-gate` 输出摘要。
  - `rg` 验证规则覆盖。

### 3）规模与复杂度观察

- **推荐目标**：脚本自包含，规则表驱动，fail/warn 分离。
- **复杂度说明**：若 allowlist 过长，保留 JSON 并在 receipt 说明后续治理。
- **禁止行为**：实现复杂 AST/SAST 引擎；V1 只做高风险固定扫描。

### 4）自动化质量闸门（强制）

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | gate 语法通过 | `node --check tests/security/security_audit_gate.js` | 返工 |
| TEST | gate 运行通过 | `node tests/security/security_audit_gate.js` | 返工或修规则 |
| REGRESSION | slash smoke 仍通过 | `node tests/frontend/day16_slash_palette_smoke.js` | 返工 |
| FMT | diff 无尾随空格 | `git diff --check -- tests/security package.json docs/debt` | 返工 |
| LINT | gate 非假通过 | `rg -n "process.exitCode|process.exit\\(|fail|warn" tests/security/security_audit_gate.js` | 返工 |
| ARCH | 不迁移 Tauri global API | `git diff -- src/interface/desktop/tauri.conf.json` 为空或仅文档说明 | 返工或说明 |
| REAL | allowlist 有 reason | `rg -n "\"reason\"" tests/security/security_audit_allowlist.json` 如文件存在 | 返工 |
| DOC | receipt 记录 AD-008 partial/gated | `rg -n "AD-008|PARTIAL/GATED|Security Audit Gate" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 返工 |

---

## 【模块3-A】刀刃表（16项，强制命令化）

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | gate 文件存在 | `Test-Path tests/security/security_audit_gate.js` | [ ] |
| FUNC | FUNC-002 | 检查 CSP null | `rg -n "csp|null" tests/security/security_audit_gate.js` | [ ] |
| FUNC | FUNC-003 | 检查 inline event handler | `rg -n "onclick|onerror|onload|onmouseover" tests/security/security_audit_gate.js` | [ ] |
| FUNC | FUNC-004 | 检查 shell allow-list | `rg -n "ALLOWED_COMMANDS|bash|pwsh|powershell|sh" tests/security/security_audit_gate.js` | [ ] |
| CONST | CONST-001 | `withGlobalTauri` 仅 warn | `rg -n "withGlobalTauri|warn" tests/security/security_audit_gate.js` | [ ] |
| CONST | CONST-002 | 危险 HTML API 有规则 | `rg -n "innerHTML|insertAdjacentHTML" tests/security/security_audit_gate.js` | [ ] |
| CONST | CONST-003 | allowlist reason 机制 | `rg -n "allowlist|reason" tests/security/security_audit_gate.js tests/security/security_audit_allowlist.json` | [ ] |
| CONST | CONST-004 | 严重项非 0 退出 | `rg -n "process.exitCode|process.exit\\(1\\)" tests/security/security_audit_gate.js` | [ ] |
| NEG | NEG-001 | gate 不永远 PASS | 代码审查 + `rg -n "failures|errors|exitCode" tests/security/security_audit_gate.js` | [ ] |
| NEG | NEG-002 | 不修改 shell.rs | `git diff -- src/engine/tool-system/src/shell.rs` 为空 | [ ] |
| NEG | NEG-003 | 不关闭 global API 债务 | receipt 中 `rg -n "withGlobalTauri|AD-002|不关闭" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | [ ] |
| NEG | NEG-004 | 不新增外部依赖 | `git diff -- package.json package-lock.json` 仅 script 或为空 | [ ] |
| UX | UX-001 | gate 输出 fail/warn summary | `rg -n "summary|warnings|failures|PASS|FAIL" tests/security/security_audit_gate.js` | [ ] |
| UX | UX-002 | 错误信息指向路径 | `rg -n "path|file|line" tests/security/security_audit_gate.js` | [ ] |
| E2E | E2E-001 | gate 实际运行 | `node tests/security/security_audit_gate.js` 退出码 0（当前基线） | [ ] |
| High | HIGH-001 | shell 复杂命令回归 fail | `rg -n "bash|sh|pwsh|powershell" tests/security/security_audit_gate.js` + 规则说明 | [ ] |

---

## 【模块3-B】地狱红线（10项）

1. gate 永远返回 0，返工。
2. `csp: null` 不 fail，返工。
3. shell allow-list 恢复复杂 shell 不 fail，返工。
4. allowlist 没有 reason，返工。
5. 把 `withGlobalTauri` 当成本批关闭项，返工。
6. 重写大量历史 DOM，返工。
7. 新增依赖但无必要说明，返工。
8. gate 输出没有文件路径，返工。
9. 未运行 gate 却写已通过，返工。
10. 把 Gate V1 写成完整安全审计，返工。

---

## 【模块4】P4 自测轻量检查表 v3.0

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| 核心功能用例（CF） | gate 是否扫描 CSP、DOM、shell？ | [ ] | CF-B16-D05-001 | |
| 约束与回归用例（RG） | 是否覆盖已知安全回退点？ | [ ] | RG-B16-D05-001 | |
| 负面路径用例（NG） | 严重项是否会 fail？ | [ ] | NG-B16-D05-001 | |
| 用户体验用例（UX） | 输出是否能定位文件和原因？ | [ ] | UX-B16-D05-001 | |
| 端到端关键路径（E2E） | gate 是否真实运行通过？ | [ ] | E2E-B16-D05-001 | |
| 高风险场景（High） | 复杂 shell 回归是否 fail？ | [ ] | HIGH-B16-D05-001 | |
| 字段完整性 | allowlist 是否含 reason？ | [ ] | JSON/代码审查 | |
| 需求映射 | 是否映射到 AD-008？ | [ ] | receipt | |
| 自测执行 | 是否跑过 gate 和 slash smoke？ | [ ] | 输出摘要 | |
| 范围边界与债务 | 是否声明 Gate V1 覆盖有限？ | [ ] | receipt | |

---

## 【模块5】收卷格式（强制结构）

```markdown
## 工单 B-16/05 完成并提交

### 提交信息
- Commit: `test(security): add lightweight audit gate for b16 regressions`
- 分支: `<执行时分支>`
- HEAD: `<执行时HEAD>`
- 变更文件:
  - `tests/security/security_audit_gate.js`
  - `tests/security/security_audit_allowlist.json`（如有）
  - `package.json`（如有）
  - `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md`

### 本轮目标与实际结果
- 目标: 建立 Security Audit Gate V1。
- 实际完成: `<列出 fail/warn 规则>`
- 未完成/不在范围: Tauri global API 迁移；完整 SAST；WebView smoke。

### 关键决策记录
- DECISION-001: `<fail/warn 分级>` - `<原因>`
- DECISION-002: `<allowlist 结构>` - `<原因>`

### 自动化质量检查报告
```bash
node --check tests/security/security_audit_gate.js
node tests/security/security_audit_gate.js
node tests/frontend/day16_slash_palette_smoke.js
git diff --stat
git diff --check -- tests/security package.json docs/debt
```

### 债务声明
- DEBT-SECURITY-B16-D05: Gate V1 只覆盖已知高风险模式，不替代完整安全审计。
- DEBT-SCOPE-B16-D05: `withGlobalTauri: true` 仍为 AD-002 后续债务。

### 风险与回滚点
- 主要风险: gate 误报影响开发体验。
- 回滚方式: 删除 gate script/package script，保留 receipt 中债务说明。
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| QUALITY-001 | gate 误报过多无法通过 | 降级为 warn + allowlist reason，记录债务 | 有条件交付 |
| SECURITY-001 | 发现真实严重回归 | 先修回归，不放宽 gate | 返工 |
| SCOPE-001 | 规则需要 AST/SAST 才能可靠判断 | 保持 V1 文本扫描，声明覆盖限制 | 后续债务 |
| TEST-001 | Node 环境无法运行 | 记录环境失败，不能标记 AD-008 partial/gated | 返工或阻塞 |

---

## 【模块7】派单口令（通用版）

启动饱和攻击集群，执行 **B-16 Day 5 Security Audit Gate V1** 通用高压任务！

### 技术背景
B-16 除 slash palette 外，还要推进 AD-008：建立轻量自动安全门禁，防止 CSP、DOM、shell allow-list 等已知安全债回归。

### 关键约束
- 严重项必须 fail。
- `withGlobalTauri: true` 只 warn。
- allowlist 必须有 reason。
- Gate V1 不等同完整安全审计。

### 质量红线
- 10 项地狱红线生效。
- 永远 PASS 的 gate 直接返工。
- 安全债不得伪关闭。

### 工单并行矩阵
- B-16/05 Engineer：Security Audit Gate V1

### 验收铁律
- `node tests/security/security_audit_gate.js` 必须运行。
- 严重项 fail 机制必须有代码证据。
- receipt 必须写 AD-008 `PARTIAL/GATED` 建议和限制。

### 收卷要求
- 附自动化质量检查摘要。
- 附刀刃表摘要。
- 诚实声明 gate 覆盖范围。

Ouroboros 闭环启动，B-16 Day 5，执行。

---

## 【模块8】通用验证命令库（本工单适用）

```bash
git branch --show-current
git rev-parse HEAD
node --check tests/security/security_audit_gate.js
node tests/security/security_audit_gate.js
node tests/frontend/day16_slash_palette_smoke.js
rg -n "\"csp\"|withGlobalTauri" src/interface/desktop/tauri.conf.json
rg -n "innerHTML|insertAdjacentHTML|onclick=|onerror=|onload=" src/interface/web
rg -n "ALLOWED_COMMANDS|bash|sh|pwsh|powershell" src/engine/tool-system/src/shell.rs
git diff --stat
git diff --check -- tests/security package.json docs/debt
```
