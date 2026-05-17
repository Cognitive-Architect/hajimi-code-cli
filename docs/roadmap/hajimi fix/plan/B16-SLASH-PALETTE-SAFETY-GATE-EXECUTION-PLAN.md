# B-16 Slash Palette + Safety Gate 执行计划 — Day 1~7 每日细化

> **文档版本**: 1.0
> **所属 Roadmap**: `docs/roadmap/B16-SLASH-PALETTE-SAFETY-GATE-ROADMAP.md`
> **前置条件**: `docs/debt/ACTIVE-DEBT-STATUS-2026-05-17.md` 已作为当前债务基线
> **最后更新**: 2026-05-17
> **批次目标**: 排除需要真实 Tauri/WebView 证据的债务后，优先处理 `AD-007`、局部推进 `AD-004`、为 `AD-008` 建立轻量自动门禁。

---

## 已确认的基线

| Debt | 状态 | 本计划处理结论 |
|:---|:---:|:---|
| `AD-001 Shell feature downgrade` | `OPEN BY DESIGN` | 不恢复复杂 shell；只保留现有限制和文档说明 |
| `AD-002 Tauri global API migration` | `PARTIAL/VERIFY` | 不处理；需要真实 WebView smoke |
| `AD-003 Tauri GUI/WebView smoke blocker` | `ACTIVE BLOCKED` | 不处理；由用户后续实机验收 |
| `AD-004 Frontend modularization` | `PARTIAL/P2` | 随 `AD-007` 局部推进，新功能必须模块化 |
| `AD-005 Thinking UI and checkpoint depth` | `PARTIAL/VERIFY` | 不处理；需要真实 WebView smoke |
| `AD-006 Agent Prompt productization` | `PARTIAL/P2` | 不进入主开发；只更新后续规格/冻结说明 |
| `AD-007 Slash command suggestion panel` | `OPEN` | 本批主任务，交付 Slash Palette V1 |
| `AD-008 SecurityAuditTool quality` | `OPEN/P2` | 本批建立轻量 gate，先 partial/gated，不直接吹成完整安全系统 |

**当前验证基线建议**:

```bash
node --check src/interface/web/app.js
node tests/frontend/day13_workspace_modules_smoke.js
node tests/frontend/day14_sessions_thinking_modules_smoke.js
cargo test -p engine-tool-system -- test_allow_list
```

---

## Phase 0: Baseline Audit（Day 1）

> **目标**: 找到 slash command 解析入口、输入框事件入口、现有模块加载方式、安全风险扫描点。先看路，再开车。

---

### Day 1: Baseline Audit + Slash Contract

**预计工时**: 3-5 小时
**风险等级**: 🟢 低（只读为主，最多新增草稿文档）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 搜索 slash / command 相关入口 | `src/interface/web/app.js`、`src/interface/web/modules/*` | `rg "slash|command|palette|handleChatCommand|showCommandPalette" src/interface/web tests docs` |
| 2 | 搜索输入框事件入口 | `app.js` | 定位 `input`、`keydown`、`submit`、`sendChatMessage` 相关逻辑 |
| 3 | 搜索前端模块加载方式 | `src/interface/web` | 确认项目当前是 ES module、全局对象还是 script 拼接 |
| 4 | 搜索安全风险 DOM 写法 | `src/interface/web` | `rg "innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=" src/interface/web` |
| 5 | 搜索 shell allow-list | `src/engine/tool-system/src/shell.rs` | 确认 `bash/sh/pwsh/powershell` 没回到用户 allow-list |
| 6 | 定义 SlashCommandItem V1 | receipt 或代码注释 | 见下方 V1 shape |
| 7 | 确定 B16 Non-Scope | receipt | 明确不处理 `AD-002/003/005`，不恢复复杂 shell，不做大重构 |

#### SlashCommandItem V1 建议

```js
{
  id: "compact",
  trigger: "/compact",
  title: "Compact context",
  description: "Compress current chat context",
  category: "context",
  riskLevel: "low",
  enabled: true
}
```

#### 验证命令

```bash
rg "slash|command|palette|handleChatCommand|showCommandPalette" src/interface/web tests docs
rg "innerHTML|outerHTML|insertAdjacentHTML|onclick=|onerror=" src/interface/web
rg "ALLOWED_COMMANDS|bash|pwsh|powershell" src/engine/tool-system/src/shell.rs
node --check src/interface/web/app.js
```

#### Day 1 验收标准

- [ ] 找到现有 slash command 解析入口，或确认需要新增 registry。
- [ ] 找到聊天输入框事件入口。
- [ ] 确认新模块如何被 `app.js` 引入。
- [ ] 输出 SlashCommandItem V1 shape。
- [ ] 明确安全 gate V1 的扫描项。
- [ ] 没有修改核心业务逻辑。

---

## Phase 1: Slash Palette UI V1（Day 2-3）

> **目标**: 实现用户输入 `/` 时出现命令建议面板，并保证普通输入不受影响。

---

### Day 2: 新增 `slash-palette.js` 模块 + 基础 UI

**预计工时**: 4-6 小时
**风险等级**: 🟡 中（前端交互新增，但可模块化隔离）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 新建 slash palette 模块 | `src/interface/web/modules/slash-palette.js` | 导出 `createSlashPalette(options)` |
| 2 | 定义内部状态 | `slash-palette.js` | `isOpen`、`query`、`items`、`filteredItems`、`activeIndex` |
| 3 | 实现 safe render | `slash-palette.js` | 使用 `document.createElement` + `textContent`，禁止拼接 HTML |
| 4 | 实现 `open(query)` | `slash-palette.js` | 显示面板并渲染过滤结果 |
| 5 | 实现 `close(reason)` | `slash-palette.js` | 隐藏面板，重置 activeIndex |
| 6 | 实现 `updateQuery(query)` | `slash-palette.js` | 根据 `/xxx` 更新过滤结果 |
| 7 | 添加基础样式 | `src/interface/web/style.css` | 使用 `.slash-palette-*` 前缀，避免污染 |
| 8 | 在 app.js 轻量初始化 | `src/interface/web/app.js` | 只传入 input、commands、onSelect，不把业务逻辑塞回模块 |

#### 建议模块 API

```js
export function createSlashPalette({
  inputEl,
  containerEl,
  getCommands,
  onSelect,
  onOpen,
  onClose
}) {
  return {
    open,
    close,
    updateQuery,
    handleKeyDown,
    isOpen
  };
}
```

#### 验证命令

```bash
node --check src/interface/web/modules/slash-palette.js
node --check src/interface/web/app.js
grep "createSlashPalette" src/interface/web/app.js
grep "textContent" src/interface/web/modules/slash-palette.js
```

#### Day 2 验收标准

- [ ] `slash-palette.js` 存在并通过语法检查。
- [ ] 输入 `/` 可以打开面板。
- [ ] 面板至少显示 trigger + title + description。
- [ ] 渲染使用 safe DOM，不直接拼 HTML。
- [ ] `app.js` 只做接线，不出现大面积重构。

---

### Day 3: 输入联动 + 键盘/鼠标选择

**预计工时**: 4-6 小时
**风险等级**: 🟡 中（容易误伤 Enter 发送消息逻辑）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 监听 input 变化 | `app.js` | 当前输入 token 以 `/` 开头时调用 `palette.open/updateQuery` |
| 2 | 实现过滤规则 | `slash-palette.js` | `/c` 匹配 trigger/title/category |
| 3 | 实现 `ArrowDown` / `ArrowUp` | `slash-palette.js` | 改变 `activeIndex`，循环选择 |
| 4 | 实现 `Enter` 选择 | `slash-palette.js` + `app.js` | palette open 时拦截 Enter，选择 active item |
| 5 | 实现 `Esc` 关闭 | `slash-palette.js` | 不清空用户输入 |
| 6 | 实现鼠标 click 选择 | `slash-palette.js` | disabled item 不可执行 |
| 7 | 接入现有 command handler | `app.js` | 优先复用已有 `/compact`、`/help` 等命令处理逻辑 |
| 8 | 保护普通消息发送 | `app.js` | palette 未打开时，Enter 走旧路径 |

#### 选择后动作策略

| 命令风险 | V1 动作 |
|:---|:---|
| low | 可直接执行或回填，按现有项目习惯 |
| medium/high | 只回填，不自动执行 |
| disabled | 不执行，显示不可用状态 |
| unknown | 关闭面板并保留输入，不调用 handler |

#### 验证命令

```bash
node --check src/interface/web/modules/slash-palette.js
node --check src/interface/web/app.js
grep "ArrowDown\|ArrowUp\|Escape\|Enter" src/interface/web/modules/slash-palette.js src/interface/web/app.js
```

#### Day 3 验收标准

- [ ] `/` 打开面板。
- [ ] `/c` 可以过滤候选项。
- [ ] 上下键可以移动选择。
- [ ] `Enter` 选择命令时不会同时发送普通聊天。
- [ ] `Esc` 关闭面板且保留输入。
- [ ] 鼠标点击可选择。
- [ ] disabled 命令不会执行。
- [ ] 普通消息发送逻辑不被破坏。

---

## Phase 2: Smoke Test + Modularization Receipt（Day 4）

> **目标**: 让 slash palette 的 JS 行为可自动验证，并留下 AD-004 局部改善证据。

---

### Day 4: Slash Palette Node Smoke

**预计工时**: 3-5 小时
**风险等级**: 🟢 低（测试为主）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 新增 smoke 测试 | `tests/frontend/day16_slash_palette_smoke.js` | 模仿 Day13/Day14 smoke 风格 |
| 2 | 构造 mock DOM | 测试文件 | 创建 input/container/button 节点 |
| 3 | 测试 `/` 打开 | 测试文件 | 调用 input/update 后断言 panel visible |
| 4 | 测试 query 过滤 | 测试文件 | `/c` 只显示匹配项 |
| 5 | 测试键盘导航 | 测试文件 | ArrowDown/Up 改变 active |
| 6 | 测试 Enter 选择 | 测试文件 | onSelect 被调用一次 |
| 7 | 测试 Esc 关闭 | 测试文件 | panel hidden |
| 8 | 测试安全渲染 | 测试文件 | 注入 `<img onerror=...>` 文案，断言不作为 HTML 执行 |
| 9 | 更新 receipt 草稿 | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 记录 Node smoke 覆盖范围 |

#### 验证命令

```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
```

#### Day 4 验收标准

- [ ] `day16_slash_palette_smoke.js` 通过。
- [ ] smoke 覆盖打开、过滤、选择、关闭。
- [ ] smoke 覆盖 safe DOM rendering。
- [ ] receipt 明确：Node smoke 不等同真实 WebView smoke。
- [ ] AD-004 可记录为 slash palette 相关模块化改善。

---

## Phase 3: Security Audit Gate V1（Day 5）

> **目标**: 为 AD-008 增加轻量自动门禁，先覆盖最容易回归的大坑。

---

### Day 5: Security Gate 脚本 + 可配置 allowlist

**预计工时**: 4-6 小时
**风险等级**: 🟡 中（误报会影响开发体验）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 新建 security gate | `tests/security/security_audit_gate.js` | Node 脚本，读取固定文件并扫描风险模式 |
| 2 | 检查 Tauri CSP | `src/interface/desktop/tauri.conf.json` | `"csp": null` 直接 fail |
| 3 | 检查 inline event handler | `src/interface/web` | `onclick=`、`onerror=`、`onload=` 等新增风险直接 fail |
| 4 | 检查危险 HTML API | `src/interface/web` | `innerHTML` / `insertAdjacentHTML` 若无 `SECURITY:` 注释则 warn/fail |
| 5 | 检查 shell allow-list | `src/engine/tool-system/src/shell.rs` | 用户 allow-list 不得包含 `bash/sh/pwsh/powershell` |
| 6 | 增加 allowlist | `tests/security/security_audit_allowlist.json` 可选 | 对历史遗留点要求写 reason |
| 7 | 增加 package script | `package.json` 可选 | `"test:security-gate": "node tests/security/security_audit_gate.js"` |
| 8 | 记录 AD-008 receipt | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 明确只是 Gate V1，不是完整安全审计 |

#### Gate V1 fail/warn 建议

| 检查项 | V1 行为 |
|:---|:---:|
| `csp: null` | fail |
| 用户 allow-list 恢复 shell | fail |
| inline event handler | fail |
| 未标注 `innerHTML` | warn 或 fail，取决于现有历史数量 |
| `withGlobalTauri: true` | warn，不在本批解决 |

#### 验证命令

```bash
node tests/security/security_audit_gate.js
```

#### Day 5 验收标准

- [ ] Security gate 能稳定运行。
- [ ] 严重项能返回非 0 exit code。
- [ ] 已知历史点如果被 allowlist，必须有 reason。
- [ ] 不把 `withGlobalTauri: true` 当成本批失败项。
- [ ] AD-008 可标记为 `PARTIAL/GATED`，不建议直接关闭。

---

## Phase 4: Docs Closure（Day 6）

> **目标**: 文档闭环，避免“代码做了但债务状态没同步”。

---

### Day 6: Debt Receipt + Active Status 更新建议

**预计工时**: 2-4 小时
**风险等级**: 🟢 低（文档为主）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 创建 B16 receipt | `docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | 记录分支、HEAD、scope、变更文件、验证命令 |
| 2 | 更新 active debt 建议 | `docs/debt/ACTIVE-DEBT-STATUS-2026-05-17.md` 或新快照 | 不建议直接覆盖旧快照，优先新建 `ACTIVE-DEBT-STATUS-YYYY-MM-DD.md` |
| 3 | 记录 AD-007 状态 | debt receipt | `IMPLEMENTED/PENDING-UI-SMOKE` 或按实际验收结果 |
| 4 | 记录 AD-004 状态 | debt receipt | `PARTIAL/IMPROVED`，因为只拆了 slash palette |
| 5 | 记录 AD-008 状态 | debt receipt | `PARTIAL/GATED`，因为 gate V1 覆盖有限 |
| 6 | 记录 AD-006 状态 | debt receipt | `DEFERRED/P2-SPEC`，不进入本批开发 |
| 7 | 记录 AD-001 状态 | debt receipt | `OPEN BY DESIGN`，保持限制 |
| 8 | 增加用户实机验收 TODO | debt receipt | slash palette UI 点击验证清单 |

#### 验证命令

```bash
grep -n "AD-007\|AD-004\|AD-008" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
grep -n "WebView\|Node smoke" docs/debt/DEBT-B16-SLASH-SAFETY-REMEDIATION.md
```

#### Day 6 验收标准

- [ ] B16 receipt 存在。
- [ ] receipt 中记录所有验证命令和结果。
- [ ] 没有把 Node smoke 写成 WebView smoke。
- [ ] 明确列出仍需用户实机验收的项目。
- [ ] 债务状态更新建议清楚，但不过度关闭。

---

## Phase 5: Final Regression & Handoff（Day 7）

> **目标**: 完成最终验证，给用户一份能亲自点验收的接管包。

---

### Day 7: 最终回归 + Handoff Pack

**预计工时**: 3-4 小时
**风险等级**: 🟢 低（验证和整理）

#### 任务清单

| # | 任务 | 目标文件 | 代码细节 |
|---:|---|:---|:---|
| 1 | 运行前端语法检查 | `app.js` + `slash-palette.js` | `node --check ...` |
| 2 | 运行 slash smoke | `tests/frontend/day16_slash_palette_smoke.js` | 确认 JS 行为可重复 |
| 3 | 运行 security gate | `tests/security/security_audit_gate.js` | 确认高风险回退点被扫描 |
| 4 | 运行 shell allow-list 回归 | `engine-tool-system` | `cargo test -p engine-tool-system -- test_allow_list` |
| 5 | 生成最终手动验收清单 | receipt | 给用户 Tauri/WebView 点击步骤 |
| 6 | 检查 git diff | 全仓库 | 确认没有意外大重构 |
| 7 | 准备 commit message | git | 见下方建议 |

#### 最终验证命令

```bash
node --check src/interface/web/app.js
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
node tests/security/security_audit_gate.js
cargo test -p engine-tool-system -- test_allow_list
git diff --stat
```

#### Day 7 验收标准

- [ ] 前端 JS 语法检查通过。
- [ ] Slash Palette smoke 通过。
- [ ] Security gate 通过。
- [ ] Shell allow-list 测试通过。
- [ ] `git diff --stat` 没有明显异常大改。
- [ ] B16 receipt 完整。
- [ ] 用户实机验收清单已写入 receipt。

---

## 最终回归验收清单

- [ ] `node --check src/interface/web/app.js` **0 errors**
- [ ] `node --check src/interface/web/modules/slash-palette.js` **0 errors**
- [ ] `node tests/frontend/day16_slash_palette_smoke.js` **PASS**
- [ ] `node tests/security/security_audit_gate.js` **PASS**
- [ ] `cargo test -p engine-tool-system -- test_allow_list` **PASS**
- [ ] Slash Palette 输入 `/` 能打开
- [ ] Slash Palette 输入 `/c` 能过滤
- [ ] `ArrowUp/ArrowDown` 能选择
- [ ] `Enter` 能选择命令
- [ ] `Esc` 能关闭
- [ ] disabled command 不执行
- [ ] 安全 gate 不允许 `csp: null`
- [ ] 安全 gate 不允许 shell allow-list 恢复复杂 shell
- [ ] `DEBT-B16-SLASH-SAFETY-REMEDIATION.md` 记录完整
- [ ] `AD-002/003/005` 未被伪关闭
- [ ] git status 干净，最终 commit 符合规范

---

## Kill Switch / Gate 汇总

| 开关 | 控制范围 | 默认值 | 回滚行为 |
|:---|:---|:---:|:---|
| `window.__HAJIMI_FLAGS__.slashPaletteEnabled` 或等价前端 flag | Slash Palette UI | `true` | 回到旧输入框行为 |
| `SECURITY_AUDIT_GATE_STRICT` | Security Gate 严格模式 | `false` 初期可 report-only | 只输出 warn，不阻塞开发 |
| `HAJIMI_COMPLEX_SHELL_ENABLED` | 不建议新增 | `false` | 继续保持复杂 shell 禁用 |

> 注意：如果项目当前没有统一 feature flag 体系，不要为了本批次硬造大型配置系统。V1 可以用最小 killswitch 或文档化回滚替代。

---

## 工作量统计

| Phase | 天数 | 新建文件 | 修改文件 | 预计总工时 |
|:---|:---:|:---:|:---:|:---:|
| Phase 0 | Day 1 | 0-1 | 0 | 3-5h |
| Phase 1 | Day 2-3 | 1 | 2 | 8-12h |
| Phase 2 | Day 4 | 1 | 1 | 3-5h |
| Phase 3 | Day 5 | 1-2 | 0-1 | 4-6h |
| Phase 4 | Day 6 | 1 | 0-1 | 2-4h |
| Phase 5 | Day 7 | 0 | 1 | 3-4h |
| **总计** | **7 天** | **4-6** | **3-6** | **23-36h** |

---

## 建议 Commit Message

```text
feat(frontend): add slash palette v1 and lightweight security gate

- add slash-palette module with safe DOM rendering
- wire slash command suggestions into chat input
- add Node smoke coverage for slash palette behavior
- add lightweight security audit gate for known regressions
- document B16 debt remediation receipt
```

---

## 用户实机验收脚本（人工）

> 这部分 Day 7 之后由用户在真实 Tauri/WebView 窗口里亲自确认。

```text
1. 启动桌面 App。
2. 聚焦聊天输入框。
3. 输入 / ，确认建议面板出现。
4. 输入 /c ，确认候选项过滤。
5. 按 ArrowDown / ArrowUp，确认高亮移动。
6. 按 Enter，确认命令被选择。
7. 再输入 / 后按 Esc，确认面板关闭且输入不被误删。
8. 输入普通消息并发送，确认旧聊天行为没坏。
9. 打开 DevTools / 日志，确认无明显 error。
10. 截图或保存日志，贴回 B16 debt receipt。
```

---

*本执行计划与 B16 roadmap 同步维护。每完成一天，请在对应 Day 的验收清单中打勾，并把验证命令输出贴进 debt receipt。*
