# Day 14 派单: 前端 sessions + thinking-ui 模块渐进拆分

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 14，承接 Day 13，拆分会话持久化与 Thinking UI 高频区域。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: 前端 sessions + thinking-ui 模块渐进拆分
- **轰炸目标**: 新增 `sessions.js` 与 `thinking-ui.js`，将会话持久化、session list、thinking block、trace card、operation summary 的可拆逻辑从 `app.js` 渐进抽离
- **任务性质**: 重构优化 + 回归防护
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 两个模块 + app.js 兼容接入 + 会话和 thinking/trace 不回归 + 文档同步
- **通用铁律**:
  1. 不引入前端框架或 bundler
  2. 保留旧 API 包装，避免旧事件绑定断裂
  3. 会话持久化必须复验关闭重开
  4. Thinking/Trace 必须继续使用真实数据链路
  5. 不把 checkpoint restore/export/compare 逻辑重新写回假实现

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 前置工单 | Day 13 modules 已存在 | `Get-ChildItem src/interface/web/modules` | 必须 |
| 会话区域 | session load/save/switch/render | `src/interface/web/app.js:3340-3422` | 必须 |
| Thinking 区域 | thinking block/trace/operation summary | `src/interface/web/app.js:1920-2041`, `:3077-3286` | 必须 |
| Replay/Checkpoint | Day 8-10 真实链路 | `src/interface/web/app.js:4320-4373`, `:4731-4842` | 必须 |
| 输出模块 | sessions/thinking-ui | `src/interface/web/modules/` | 必须 |
| 文档同步 | 架构变化需记录 | `src/ARCHITECTURE.md`; `src/INDEX.md` | 按需 |
| 验证命令 | JS/Rust 检查 | `node --check ...`; `cargo check -p hajimi-desktop` | 必须 |

### 探索补充栏

本任务的难点是 `app.js` 的 shared state。优先拆 pure rendering/helper 与 persistence function；如状态耦合过强，保留 wrapper 并声明复杂度债，不做危险大搬迁。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-14/15
- **角色**: Engineer
- **目标**: 继续降低 `app.js` 单体风险，同时保护会话和 Thinking UI 回归
- **输入**: Day 13 模块模式、Day 7 session receipt、Day 8-10 thinking/checkpoint 真实链路
- **依赖关系**: 依赖 Day 13；建议在 Day 10 之后执行

### 2）输出交付物

- **变更文件**:
  - `src/interface/web/modules/sessions.js`
  - `src/interface/web/modules/thinking-ui.js`
  - `src/interface/web/app.js`
  - `src/interface/web/index.html`，如需加载模块
  - `src/ARCHITECTURE.md`
  - `src/INDEX.md`
- **核心修改点**:
  - `sessions.js` 负责 load/save/switch/render helper 或 pure state 操作
  - `thinking-ui.js` 负责 thinking block、trace card、operation summary rendering helper
  - `app.js` 保留兼容 wrapper 和旧外部调用
  - 文档记录前端模块边界
- **必须包含**:
  - `node --check` 覆盖 app.js 和四个模块
  - 会话 A/B 切换与关闭重开复验
  - Thinking stream / trace panel / operation summary smoke
  - checkpoint export/compare/restore/replay 不回归
- **禁止包含**:
  - 删除 `hajimi_chat_sessions` 数据兼容
  - 使用模拟 trace/replay 数据
  - 引入 bundler
  - 大改 CSS 或 layout
- **交付证明**:
  - JS checks
  - `cargo check -p hajimi-desktop`
  - 手动 smoke receipt
  - `src/ARCHITECTURE.md` / `src/INDEX.md` diff

### 3）规模与复杂度观察

- **推荐目标**: 保守拆 helper，保留 app state ownership
- **复杂度说明**: 如果 `thinking-ui.js` 需要大量 DOM state，声明 `DEBT-COMPLEXITY-B14-001`，保留部分在 app.js
- **禁止行为**: 用模块拆分破坏 Day 8-10 真实数据链路

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | 所有 JS 语法通过 | `node --check src/interface/web/app.js`; `Get-ChildItem src/interface/web/modules -Filter *.js | ForEach-Object { node --check $_.FullName }` | 返工 |
| LINT | 不引入 bundler/framework | `rg -n "React|Vue|Vite|webpack|import .* from" src/interface/web package.json` | 返工 |
| TEST | 会话和 thinking smoke | receipt 文档或收卷报告 | 返工 |
| ARCH | 文档同步 | `rg -n "sessions.js|thinking-ui.js|frontend modules" src/ARCHITECTURE.md src/INDEX.md` | 返工 |
| REAL | 不使用模拟数据 | `rg -n "mock|fake|simulation|sample" src/interface/web/modules src/interface/web/app.js` 并人工确认无新增假实现 | 返工 |
| DOC | 债务状态诚实 | 债务总表或 receipt | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `sessions.js` 存在 | `Test-Path src/interface/web/modules/sessions.js` | [ ] |
| FUNC | FUNC-002 | `thinking-ui.js` 存在 | `Test-Path src/interface/web/modules/thinking-ui.js` | [ ] |
| FUNC | FUNC-003 | app.js 接入 sessions | `rg -n "HajimiSessions|sessions.js|loadChatSessions|saveChatSessions" src/interface/web/app.js src/interface/web/index.html` | [ ] |
| FUNC | FUNC-004 | app.js 接入 thinking-ui | `rg -n "HajimiThinkingUI|thinking-ui.js|traceEvents|OperationSummary" src/interface/web/app.js src/interface/web/index.html` | [ ] |
| CONST | CONST-001 | 所有模块语法通过 | `Get-ChildItem src/interface/web/modules -Filter *.js | ForEach-Object { node --check $_.FullName }` | [ ] |
| CONST | CONST-002 | app.js 语法通过 | `node --check src/interface/web/app.js` | [ ] |
| CONST | CONST-003 | 文档同步前端模块边界 | `rg -n "security-dom|workspace|sessions|thinking-ui" src/ARCHITECTURE.md src/INDEX.md` | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 未删除 localStorage key | `rg -n "hajimi_chat_sessions" src/interface/web/app.js src/interface/web/modules` | [ ] |
| NEG | NEG-002 | 未使用模拟 trace/replay | `rg -n "mock|fake|simulation" src/interface/web/app.js src/interface/web/modules` 无新增 | [ ] |
| NEG | NEG-003 | checkpoint 真实接口未回退 | `rg -n "export_checkpoint|compare_checkpoints|restore_checkpoint" src/interface/web/app.js src/interface/web/modules` | [ ] |
| NEG | NEG-004 | 不引入 framework/bundler | `rg -n "React|Vue|Vite|webpack" src/interface/web package.json` 无新增 | [ ] |
| UX | UX-001 | 会话关闭重开 smoke | receipt | [ ] |
| UX | UX-002 | Thinking/Trace/Operation Summary smoke | receipt | [ ] |
| E2E | E2E-001 | 从会话到 trace/replay 关键路径 | receipt | [ ] |
| High | HIGH-001 | 前端架构债状态推进有证据 | 债务总表或 receipt 标 `PARTIAL` | [ ] |

---

## 【模块3-B】地狱红线

1. 引入 bundler/framework，返工
2. 模块空壳未接入，返工
3. 会话持久化回归，返工
4. Thinking/Trace 使用模拟数据，返工
5. checkpoint 功能回退到占位，返工
6. `node --check` 任一文件失败，返工
7. 忘记更新 `src/ARCHITECTURE.md` / `src/INDEX.md`，返工或声明债务
8. 删除旧 API wrapper，返工
9. 大改 layout/CSS，返工
10. 架构债直接 `CLEARED`，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | sessions/thinking 模块是否存在接入 | [ ] | FUNC-001~004 | |
| RG | 会话和 checkpoint 是否不回归 | [ ] | NEG-001, NEG-003 | |
| NG | 模拟数据和 bundler 是否拒绝 | [ ] | NEG-002, NEG-004 | |
| UX | 会话/thinking smoke 是否完成 | [ ] | UX-001~002 | |
| E2E | 关键路径是否串联 | [ ] | E2E-001 | |
| High | 架构债状态是否诚实 | [ ] | HIGH-001 | |
| 字段完整性 | 模块 API 是否文档化 | [ ] | ARCHITECTURE/INDEX | |
| 需求映射 | 是否映射前端架构债和 Thinking 债 | [ ] | 债务总表 | |
| 自测执行 | JS checks 是否全跑 | [ ] | CONST-001~002 | |
| 范围边界与债务 | 未拆完的状态是否声明 | [ ] | debt 声明 | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-14/15 完成并提交

### 提交信息
- Commit: `refactor(interface/web): split sessions and thinking ui helpers`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/web/modules/sessions.js`
  - `src/interface/web/modules/thinking-ui.js`
  - `src/interface/web/app.js`
  - `src/interface/web/index.html`
  - `src/ARCHITECTURE.md`
  - `src/INDEX.md`

### 本轮目标与实际结果
- 目标: 渐进拆分 sessions + thinking-ui
- 实际完成: `<列出模块 API、接入点、smoke 结果>`
- 未完成/不在范围: 完整前端架构重构不在本日

### 自动化质量检查报告
- `node --check app.js`: `<摘要>`
- `node --check modules/*.js`: `<摘要>`
- `cargo check -p hajimi-desktop`: `<摘要>`
- 会话/thinking smoke: `<摘要>`

### 债务声明
- `DEBT-COMPLEXITY-B14-001`: `<如 app.js 仍保留状态耦合>`

### 风险与回滚点
- 主要风险: 模块加载顺序或 state ownership 回归
- 回滚方式: 回退 modules、app.js、index.html、文档 diff
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| QUALITY-001 | 会话恢复失败 | 回退 sessions 拆分，保留文档债 | 有条件交付 |
| QUALITY-002 | trace/replay 回归 | 回退 thinking-ui 拆分 | 不破坏 Day 8-10 |
| ARCH-001 | 状态耦合过强 | 只拆 pure helper，声明复杂度债 | 不硬拆 |
| TEST-001 | 无法实机验证 | 自动检查必须全过，手动待验写 debt | 有条件交付 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 14 前端 sessions + thinking-ui 模块渐进拆分**。

### 关键约束
- 保持 vanilla JS
- 不破坏会话持久化
- 不破坏 trace/checkpoint/replay 真实链路
- 更新架构和索引文档

### 验收铁律
- 两个模块存在且语法检查通过
- app.js 通过
- 会话/thinking smoke 有证据
- 前端架构债最多推进到 `PARTIAL`

闭环启动，Day 14，执行。
