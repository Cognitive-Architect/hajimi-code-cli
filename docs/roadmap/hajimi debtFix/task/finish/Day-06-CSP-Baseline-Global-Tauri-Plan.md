# Day 06 派单: CSP Baseline + Global Tauri API 迁移计划

> 基于 `集群式开发派单模板.md` 的 ID-59 v3.0 通用增强版格式编写。
> 本工单对应 Daily Plan Day 6，承接 Day 5 DOM audit，继续处理 `CS-HAJIMI-003`。

---

## 【模块1】饱和攻击头部

- **火力配置**: 1 Agent（Engineer）
- **任务名称**: CSP Baseline + Global Tauri API 迁移计划
- **轰炸目标**: 将 `src/interface/desktop/tauri.conf.json` 从 `csp: null` 推进到可运行的基础 CSP，并对 `withGlobalTauri: true` 形成可执行迁移计划
- **任务性质**: 安全加固 + 兼容性验证
- **输入基线**: 完整技术背景见模块2
- **输出要求**: CSP baseline 或明确 blocker + `SECURITY-CSP-VERIFY.md` receipt + 应用基础检查通过
- **通用铁律**:
  1. CSP 调整必须基于 Day 5 DOM audit，不可盲开
  2. 核心功能被 CSP 破坏时必须记录具体报错和阻塞原因
  3. `withGlobalTauri` 如不能立即关闭，必须给出逐函数迁移清单
  4. 不引入前端框架或 bundler
  5. 不允许把 `csp: null` 留下却声称 Tauri 安全面已清偿

---

## 【模块2】输入基线

| 输入项 | 强制要求 | 验证命令 / 证据方式 | 状态 |
|---|---|---|---|
| Git 坐标 | 当前分支 + HEAD | `git branch --show-current`; `git rev-parse HEAD` | 必须 |
| 债务来源 | `CS-HAJIMI-003` 当前 `OPEN / P0-P1` | 债务总表第 4.3 节 | 必须 |
| Tauri 配置 | global API 和 CSP 当前状态 | `src/interface/desktop/tauri.conf.json:13`, `:25` | 必须 |
| Day 5 前置 | DOM audit 文档已存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-DOM-AUDIT.md` | 必须 |
| 前端 Tauri API 使用 | `window.__TAURI__` 或等价调用点 | `rg -n "__TAURI__|tauri\\.core\\.invoke|invoke\\(" src/interface/web/app.js src/interface/web/index.html` | 必须 |
| 推荐 CSP | `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: data:; connect-src 'self' http://127.0.0.1:*` | 写入配置或说明调整原因 | 必须 |
| 验证约束 | Rust + JS 基础检查 | `cargo check -p hajimi-desktop`; `node --check src/interface/web/app.js` | 必须 |
| receipt | CSP 验证文档 | `docs/debt/SECURITY-CSP-VERIFY.md` 或 roadmap debt 目录 | 必须 |

### 探索补充栏

| 项目 | 内容 |
|---|---|
| 已知事实 | `withGlobalTauri: true` 与 `csp: null` 会放大 XSS 后果 |
| 待确认问题 | 当前前端是否依赖 inline script、data/asset 图片、localhost 连接、global Tauri API |
| 预期输出 | 可运行 CSP baseline；如 global API 不能关闭，给出迁移清单 |
| 停止条件 | CSP 不再是 `null`，或 blocker 被完整记录且债务状态保持 `PARTIAL/OPEN` |

---

## 【模块3】工单矩阵

### 1）基础信息

- **工单编号**: B-06/15
- **角色**: Engineer
- **目标**: 降低 Tauri WebView 攻击面，建立 CSP 与 global API 收敛路线
- **输入**: Day 5 audit、`tauri.conf.json`、`app.js` Tauri API 调用点
- **依赖关系**: 依赖 Day 5 完成；不依赖 Day 7+

### 2）输出交付物

- **变更文件**:
  - `src/interface/desktop/tauri.conf.json`
  - `docs/debt/SECURITY-CSP-VERIFY.md`，或 `docs/roadmap/hajimi debtFix/debt/SECURITY-CSP-VERIFY.md`
  - `src/interface/web/app.js`，仅当 CSP 暴露必须修的小问题时
- **核心修改点**:
  - 将 `csp: null` 改为基础 CSP 字符串或对象
  - 运行 Tauri/desktop 基础检查，记录 CSP 报错
  - 扫描 `window.__TAURI__` 使用点，形成迁移清单
  - 如不能关闭 `withGlobalTauri`，保留为 `PARTIAL` 并说明阻塞项
- **必须包含**:
  - CSP 具体内容
  - console/terminal 报错摘要，若无则写“未观察到阻塞”
  - global API 迁移表：调用点、用途、替代方式、优先级
- **禁止包含**:
  - 为了通过启动直接恢复 `csp: null` 且不记录原因
  - 关闭 global API 但不修前端调用
  - 引入远程 CDN 资源
  - 修改无关 UI
- **交付证明**:
  - `rg -n "withGlobalTauri|csp" src/interface/desktop/tauri.conf.json`
  - `cargo check -p hajimi-desktop`
  - `node --check src/interface/web/app.js`
  - CSP receipt 文档

### 3）规模与复杂度观察

- **推荐目标**: CSP 最小可运行 baseline，不在本日完成 global API 全量迁移
- **复杂度说明**: `withGlobalTauri` 关闭可能需要前端模块化，属于 Day 13-14 或后续安全 batch
- **禁止行为**: 用宽松到等同无 CSP 的策略伪装完成

### 4）自动化质量闸门

| 闸门 | 要求 | 验证命令 | 不通过后果 |
|---|---|---|---|
| BUILD | desktop crate 编译通过 | `cargo check -p hajimi-desktop` | 返工 |
| FMT | JSON 格式合法 | `cargo check -p hajimi-desktop` 或 Tauri 配置解析 | 返工 |
| LINT | JS 语法通过 | `node --check src/interface/web/app.js` | 返工 |
| TEST | CSP receipt 存在 | `Get-ChildItem -LiteralPath docs -Recurse -Filter SECURITY-CSP-VERIFY.md` | 返工 |
| ARCH | 不引入远程依赖 | `rg -n "https://|http://" src/interface/web src/interface/desktop/tauri.conf.json` 并人工确认 connect-src 例外 | 返工 |
| REAL | CSP 不再是 null 或 blocker 明确 | `rg -n "\"csp\"\\s*:\\s*null" src/interface/desktop/tauri.conf.json` | 返工或保留 OPEN |
| DOC | global API 迁移计划存在 | `rg -n "withGlobalTauri|window.__TAURI__|迁移" docs` | 返工 |

---

## 【模块3-A】刀刃表

| 类别 | 检查点ID | 检查目标 | 验证命令 / 证据 | 状态 |
|---|---|---|---|---|
| FUNC | FUNC-001 | CSP 不再默认为 `null` | `rg -n "\"csp\"\\s*:\\s*null|\"csp\"" src/interface/desktop/tauri.conf.json` | [ ] |
| FUNC | FUNC-002 | CSP 包含 `default-src 'self'` | `rg -n "default-src 'self'" src/interface/desktop/tauri.conf.json` | [ ] |
| FUNC | FUNC-003 | CSP 允许必要图片来源 | `rg -n "img-src.*asset:.*data:" src/interface/desktop/tauri.conf.json` | [ ] |
| FUNC | FUNC-004 | CSP 允许必要本地连接 | `rg -n "connect-src.*127\\.0\\.0\\.1" src/interface/desktop/tauri.conf.json` | [ ] |
| CONST | CONST-001 | `withGlobalTauri` 状态已记录 | `rg -n "withGlobalTauri" src/interface/desktop/tauri.conf.json` | [ ] |
| CONST | CONST-002 | global API 调用清单已生成 | `rg -n "__TAURI__|tauri\\.core\\.invoke" src/interface/web/app.js` 输出写入 receipt | [ ] |
| CONST | CONST-003 | Day 5 DOM audit 被引用 | CSP receipt 引用 `SECURITY-DOM-AUDIT.md` | [ ] |
| CONST | CONST-004 | desktop 编译通过 | `cargo check -p hajimi-desktop` | [ ] |
| NEG | NEG-001 | 没有使用远程 CDN 脚本 | `rg -n "script.*https://|cdn" src/interface/web src/interface/desktop/tauri.conf.json` 无新增 | [ ] |
| NEG | NEG-002 | 未用过宽 `default-src *` | `rg -n "default-src \\*" src/interface/desktop/tauri.conf.json` 无命中 | [ ] |
| NEG | NEG-003 | 若 CSP 仍 null，有 blocker | `SECURITY-CSP-VERIFY.md` 中必须有 `BLOCKER` 段落 | [ ] |
| NEG | NEG-004 | 不破坏 JS 语法 | `node --check src/interface/web/app.js` | [ ] |
| UX | UX-001 | 核心界面可启动或有启动 blocker | Tauri dev receipt 或 `cargo check` + blocker | [ ] |
| UX | UX-002 | CSP 报错可读 | receipt 中包含报错/无报错摘要 | [ ] |
| E2E | E2E-001 | Day 5 + Day 6 安全链路闭合 | DOM audit + CSP receipt 两份文档存在 | [ ] |
| High | HIGH-001 | Tauri 安全面状态未伪清偿 | 债务总表状态为 `VERIFY/PARTIAL/OPEN` 且有 receipt | [ ] |

---

## 【模块3-B】地狱红线

1. `csp: null` 仍存在且无 blocker，返工
2. 设置 `default-src *` 或过宽策略，返工
3. 关闭 global API 但前端调用全断，返工
4. 声称已关闭 global API 但配置仍 true，返工
5. CSP 报错不记录，返工
6. 引入远程 CDN，返工
7. 跳过 Day 5 audit 直接改 CSP，返工
8. `node --check` 失败仍收卷，返工
9. `cargo check -p hajimi-desktop` 失败且不说明，返工
10. 把 `CS-HAJIMI-003` 直接标为 `CLEARED`，返工

---

## 【模块4】P4 自测轻量检查表

| 检查点 | 自检问题 | 覆盖情况 | 相关用例ID / 命令 | 备注 |
|---|---|---|---|---|
| CF | CSP baseline 是否落地 | [ ] | FUNC-001~004 | |
| RG | Day 5 audit 是否被承接 | [ ] | CONST-003 | |
| NG | 宽松策略是否被拒绝 | [ ] | NEG-001~002 | |
| UX | 应用启动或 blocker 是否记录 | [ ] | UX-001~002 | |
| E2E | DOM audit + CSP receipt 是否闭合 | [ ] | E2E-001 | |
| High | 安全面状态是否诚实 | [ ] | HIGH-001 | |
| 字段完整性 | 迁移表是否含调用点/替代方式 | [ ] | receipt | |
| 需求映射 | 是否映射 `CS-HAJIMI-003` | [ ] | 债务总表 4.3 | |
| 自测执行 | 是否跑 Rust/JS 检查 | [ ] | 质量闸门 | |
| 范围边界与债务 | global API 未关是否声明 | [ ] | `PARTIAL`/blocker | |

---

## 【模块5】收卷格式

```markdown
## 工单 B-06/15 完成并提交

### 提交信息
- Commit: `security(interface/desktop): add tauri csp baseline`
- 分支: `<实际分支>`
- 变更文件:
  - `src/interface/desktop/tauri.conf.json`
  - `docs/debt/SECURITY-CSP-VERIFY.md`

### 本轮目标与实际结果
- 目标: CSP baseline + global API 迁移计划
- 实际完成: `<列出 CSP、验证结果、global API 状态>`
- 未完成/不在范围: global API 全量关闭如未完成，写阻塞清单

### 自动化质量检查报告
- `cargo check -p hajimi-desktop`: `<摘要>`
- `node --check src/interface/web/app.js`: `<摘要>`
- `rg -n "withGlobalTauri|csp" ...`: `<摘要>`

### 债务声明
- `DEBT-CSP-B06-001`: `<若 global API 未关闭，列出迁移任务>`

### 风险与回滚点
- 主要风险: CSP 误伤资源加载
- 回滚方式: 不建议回到 `csp: null`；必要时收窄到具体 source 并记录
```

---

## 【模块6】技术熔断预案

| 熔断ID | 触发条件 | 动作 | 后果 |
|---|---|---|---|
| ARCH-001 | 关闭 global API 需要改造大量前端调用 | 保留 true，写迁移计划 | 状态保持 `PARTIAL` |
| QUALITY-001 | CSP 导致应用无法启动 | 记录具体 directive，最小放宽并复测 | 返工 |
| TEST-001 | 无法运行 Tauri dev | 用 `cargo check` + 配置检查 + 手动待验 debt | 有条件交付 |
| SAFETY-001 | 为启动而恢复 `csp: null` | 停止收卷，重开 blocker | 不交付 |

---

## 【模块7】派单口令

启动饱和攻击集群，执行 **Day 06 CSP Baseline + Global Tauri API 迁移计划**。

### 关键约束
- CSP 不得再空白放行，除非有明确 blocker
- global API 未关闭必须有迁移计划
- 不引入远程 CDN
- 不跳过 Day 5 DOM audit

### 验收铁律
- CSP receipt 存在
- Rust/JS 基础检查通过
- CSP 配置可追踪
- `CS-HAJIMI-003` 不得无证据清偿

闭环启动，Day 06，执行。
