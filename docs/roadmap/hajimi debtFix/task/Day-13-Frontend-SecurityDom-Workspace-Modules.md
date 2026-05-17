# Day 13 派单: 前端 security-dom + workspace 模块渐进拆分

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 13，开始处理前端 `app.js/style.css` 单体化债，但只拆安全渲染与 workspace 高频区。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: 前端 security-dom + workspace 模块渐进拆分
- **轰炸目标**: 在 `src/interface/web/modules/` 下新增 `security-dom.js` 和 `workspace.js`，将 escape/safe render helper 与 workspace/file tree/file ops 高频逻辑从 `app.js` 渐进抽离
- **任务性质**: 重构优化 + 回归防护
- **输入基线**: 完整技术背景见模块2
- **输出要求**: 两个无 bundler 模块 + app.js 最小接入 + 文件树/文件操作不回归
- **通用铁律**:
  1. 纯 HTML/CSS/JS，不引入 React/Vite/Webpack
  2. 模块采用浏览器可直接加载的方式，推荐 IIFE 挂到 `window.Hajimi*`
  3. 不一次性重写 `app.js`
  4. 必须保留旧调用兼容层
  5. 文件树、新建、重命名、删除必须 smoke test

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | 前端架构债 `OPEN / P2` | 债务总表第 11.1 节 | 必须 |
| 技术栈约束 | vanilla HTML/CSS/JS，无 bundler | AGENTS.md 与 `src/interface/web/` | 必须 |
| Day 5 前置 | security helper 与 DOM audit | `SECURITY-DOM-AUDIT.md`; `rg -n "escapeHtml|safeText" src/interface/web/app.js` | 必须 |
| Day 3 前置 | file ops 专用 commands | `rg -n "create_dir|rename_path|delete_path" src/interface/web/app.js src/interface/desktop/src/main.rs` | 必须 |
| 目标函数 | workspace/file tree/file ops | `src/interface/web/app.js:786-1038` | 必须 |
| 输出目录 | modules 目录 | `src/interface/web/modules/` | 必须 |
| 文档同步 | 架构变化需更新 `src/ARCHITECTURE.md` 或 receipt | `src/ARCHITECTURE.md` | 按需 |

### 探索补充栏

本任务是渐进拆分，不是重写。若发现 `index.html` 当前 script 加载方式不支持模块，优先使用普通 `<script>` 加载 IIFE，而不是引入构建工具。

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-13/15
- **角色**: Engineer
- **目标**: 把安全渲染 helper 与 workspace 高频逻辑拆到小模块，降低后续改动风险
- **输入**: Day 5 DOM audit、Day 3 file ops、`app.js` workspace 区域
- **依赖关系**: 依赖 Day 3 和 Day 5；建议在 Day 12 后执行

### 2）输出交付物

- **变更文件**:
  - `src/interface/web/modules/security-dom.js`
  - `src/interface/web/modules/workspace.js`
  - `src/interface/web/app.js`
  - `src/interface/web/index.html`，如需加载模块
  - `src/ARCHITECTURE.md` 或 Day receipt，说明前端模块边界
- **核心修改点**:
  - `security-dom.js` 提供 `escapeHtml`, `safeText`, `setSafeHtml` 或等价 API
  - `workspace.js` 承接 workspace init/file tree render/file ops helper 的最小可拆部分
  - `app.js` 保留兼容 wrapper，避免旧事件绑定断裂
  - `index.html` 以普通 script 顺序加载模块，若需要
- **必须包含**:
  - 每个模块 `node --check`
  - 文件树、新建、重命名、删除 smoke test
  - 模块 API 挂载方式说明
- **禁止包含**:
  - ESM/bundler 改造导致 Tauri 加载断裂，除非已完整验证
  - 大规模改 CSS
  - 删除 `window.app` 兼容入口
  - 重写会话或 thinking UI，留给 Day 14
- **交付证明**:
  - `node --check src/interface/web/app.js`
  - `node --check src/interface/web/modules/security-dom.js`
  - `node --check src/interface/web/modules/workspace.js`
  - `cargo check -p hajimi-desktop`

### 3）规模与复杂度观察

- **推荐目标**: 每个模块暴露少量函数，app.js 只做调用转发
- **复杂度说明**: 如 workspace 拆分牵涉大量 this 状态，先拆 pure helper，声明 `DEBT-COMPLEXITY-B13-001`
- **禁止行为**: 为拆而拆，造成交互回归

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | JS 语法通过 | `node --check src/interface/web/app.js`; module checks | 返工 |
| LINT | 不引入框架/bundler | `rg -n "React|Vue|Vite|webpack|import .* from" src/interface/web package.json` | 返工 |
| TEST | 文件树/file ops smoke | 手动 receipt 或脚本说明 | 返工 |
| ARCH | 模块边界记录 | `rg -n "modules/security-dom|modules/workspace|frontend modules" src/ARCHITECTURE.md docs` | 返工或声明 |
| REAL | 旧功能不回归 | Day 7/Day 3 smoke 复验 | 返工 |
| DOC | 架构变化有记录 | `src/ARCHITECTURE.md` diff 或 receipt | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | `security-dom.js` 存在 | `Test-Path src/interface/web/modules/security-dom.js` | [ ] |
| FUNC | FUNC-002 | `workspace.js` 存在 | `Test-Path src/interface/web/modules/workspace.js` | [ ] |
| FUNC | FUNC-003 | `app.js` 接入模块 | `rg -n "HajimiSecurityDom|HajimiWorkspace|security-dom|workspace" src/interface/web/app.js src/interface/web/index.html` | [ ] |
| FUNC | FUNC-004 | 旧 wrapper 保留 | `rg -n "createNewFolder|renameFile|deleteFile|loadFileTree" src/interface/web/app.js` | [ ] |
| CONST | CONST-001 | 不引入 bundler | `rg -n "Vite|webpack|React|Vue" src/interface/web package.json` 无新增 | [ ] |
| CONST | CONST-002 | 模块语法检查通过 | `node --check src/interface/web/modules/security-dom.js`; `node --check src/interface/web/modules/workspace.js` | [ ] |
| CONST | CONST-003 | app.js 语法检查通过 | `node --check src/interface/web/app.js` | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 未删除 `window.app` 入口 | `rg -n "window\\.app|new HajimiApp" src/interface/web/app.js` | [ ] |
| NEG | NEG-002 | 未改会话/thinking 大块逻辑 | `git diff -- src/interface/web/app.js` 人工确认范围 | [ ] |
| NEG | NEG-003 | 不使用 fake success | `rg -n "mock|fake|simulation|setTimeout" src/interface/web/modules src/interface/web/app.js` 无新增假实现 | [ ] |
| NEG | NEG-004 | 文件操作仍用专用 commands | `rg -n "create_dir|rename_path|delete_path|cmd: 'mkdir'|cmd: 'mv'|cmd: 'rm'" src/interface/web/app.js src/interface/web/modules` | [ ] |
| UX | UX-001 | 文件树 smoke 通过 | 手动 receipt | [ ] |
| UX | UX-002 | 新建/重命名/删除 smoke 通过 | 手动 receipt | [ ] |
| E2E | E2E-001 | Tauri 前端加载不报模块错误 | Tauri dev receipt 或 blocker | [ ] |
| High | HIGH-001 | 前端架构债推进到 `PARTIAL` 有证据 | 债务总表或 receipt | [ ] |

---

## 【模块3-B】地狱红线

1. 引入 React/Vite/Webpack，返工
2. 删除 `window.app` 兼容入口，返工
3. 只新建空模块不接入，返工
4. 文件树或文件操作回归，返工
5. `node --check` 任一文件失败，返工
6. 用假成功替代真实 invoke，返工
7. 混入会话/thinking 大重构，返工
8. 忘记加载模块导致运行时 undefined，返工
9. 架构变化不记录，返工
10. 状态直接标 `CLEARED`，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | 两个模块是否存在并接入 | [ ] | FUNC-001~004 | |
| RG | 文件树/file ops 是否不回归 | [ ] | UX-001~002 | |
| NG | 是否未引入 bundler/假实现 | [ ] | NEG-003, CONST-001 | |
| UX | 主交互是否 smoke | [ ] | UX-001~002 | |
| E2E | Tauri 加载是否验证 | [ ] | E2E-001 | |
| High | 架构债状态是否诚实推进 | [ ] | HIGH-001 | |
| 字段完整性 | 模块 API 是否说明 | [ ] | 文档/receipt | |
| 需求映射 | 是否映射前端架构债 | [ ] | 债务总表 11.1 | |
| 自测执行 | 是否跑所有 JS checks | [ ] | CONST-002~003 | |
| 范围边界与债务 | 会话/thinking 是否留给 Day 14 | [ ] | NEG-002 | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-13/15 完成并提交

### 提交信息
- Commit: `refactor(interface/web): split security dom and workspace helpers`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/web/modules/security-dom.js`
  - `src/interface/web/modules/workspace.js`
  - `src/interface/web/app.js`
  - `src/interface/web/index.html`
  - `src/ARCHITECTURE.md`

### 本轮目标与实际结果
- 目标: 渐进拆分 security-dom + workspace
- 实际完成: `<列出模块 API 和接入点>`
- 未完成/不在范围: sessions/thinking-ui 属 Day 14

### 自动化质量检查报告
- `node --check src/interface/web/app.js`: `<摘要>`
- `node --check modules/security-dom.js`: `<摘要>`
- `node --check modules/workspace.js`: `<摘要>`
- `cargo check -p hajimi-desktop`: `<摘要>`

### 债务声明
- `DEBT-COMPLEXITY-B13-001`: `<如 app.js 状态耦合未完全拆>`

### 风险与回滚点
- 主要风险: script 加载顺序导致模块 undefined
- 回滚方式: 回退 modules、app.js、index.html 本日 diff
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 模块化需要改为 type=module 且影响全局 | 暂停 ESM，使用 IIFE/window 挂载 | 避免大迁移 |
| QUALITY-001 | 文件树回归 | 回退 workspace 拆分，仅保留 security-dom | 有条件交付 |
| TEST-001 | 无法实机 smoke | 记录 blocker，自动检查必须全过 | 有条件交付 |
| COMPLEXITY-001 | app.js this 状态耦合太重 | 只拆 pure helper，声明复杂度债 | 不硬拆 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 13 前端 security-dom + workspace 模块渐进拆分**。

### 关键约束
- vanilla JS，无 bundler
- 保留兼容 wrapper
- 文件树/file ops 不回归
- 只拆 security-dom 与 workspace

### 验收铁律
- 两个模块存在且 `node --check` 通过
- app.js 通过
- Tauri/文件树 smoke 有证据
- 架构债最多推进到 `PARTIAL`

闭环启动，Day 13，执行。
