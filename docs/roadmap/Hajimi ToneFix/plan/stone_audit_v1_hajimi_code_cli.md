# Hajimi Code CLI 当前史山体检清单 v1

> 项目：`Cognitive-Architect/hajimi-code-cli`  
> 版本：Stone Audit v1  
> 生成时间：2026-06-03  
> 口径：GitHub 只读抽样 + 既有债务文档 + 关键源码片段核验  
> 注意：本报告不是完整 clone 后的复杂度/覆盖率终审，属于第一轮治理入口清单。

---

## 0. 一句话结论

当前项目不是“纯屎山废墟”，而是：

> **中重度史山候选，但已经进入治理期。**

建议史山指数：

```text
64 / 100
```

等级判断：

| 分数区间 | 等级 | 说明 |
|---:|---|---|
| 0-30 | 健康 | 有少量脏点，但整体可维护 |
| 31-50 | 有债务 | 需要定期打扫 |
| 51-70 | 中度/中重度史山 | 需要专项治理 |
| 71-85 | 重度史山 | 大改高风险，需要治理计划 |
| 86-100 | 祖传禁地 | 先上香，再开 IDE |

本项目当前落在：

```text
51-70：中度/中重度史山，需要治理
```

人话版：  
这不是没人管的垃圾堆，而是一个大体量项目长出了明显技术债，但已经开始打标签、拆模块、补测试、写清债记录。现在最重要的是：**别再让核心巨石文件继续变大。**

---

## 1. 项目总判断

| 项目 | 判断 |
|---|---|
| 当前史山指数 | **64 / 100** |
| 等级 | **中重度史山候选，但已进入治理期** |
| 最危险区域 | `src/interface/web/` + `src/interface/desktop/src/main.rs` |
| 最不建议第一刀动的区域 | `agent-core` 核心 Agent 执行链，除非已有测试护栏 |
| 第一治理目标 | 冻结巨石继续长大，优先拆 Interface 层低风险模块 |

### 1.1 为什么不是轻度

原因主要有三类：

1. 项目体量已经很大，不是单文件小工具。
2. Interface 层存在典型巨石文件：`app.js`、`style.css`、`main.rs`。
3. 技术债文档中已经明确登记过 P0/P1/P2 债务。

### 1.2 为什么也不是祖传禁地

也有不少好消息：

1. 项目有明确四层架构：Foundation / Engine / Intelligence / Interface。
2. Shell 白名单、Tauri CSP、workspace resolver 等安全边界已经做过加固。
3. Agent Core、Tool System、前端 smoke 等已有不少测试与验证记录。
4. 技术债不是没人管，而是已经有 closure、receipt、audit report 等治理痕迹。

---

## 2. Top 10 高风险文件总览

| 排名 | 文件 | 风险等级 | 当前状态 | 处理建议 |
|---:|---|---:|---|---|
| 1 | `src/interface/web/app.js` | 🚨🚨🚨🚨🚨 | 前端总控巨石 | 冻结新增大功能，继续拆模块 |
| 2 | `src/interface/desktop/src/main.rs` | 🚨🚨🚨🚨🚨 | Tauri 后端总控巨石 | 拆 command/provider/agent/checkpoint |
| 3 | `src/interface/web/style.css` | 🚨🚨🚨🚨 | 全局样式大锅炖 | 按区域拆样式边界 |
| 4 | `src/interface/web/index.html` | 🚨🚨🚨🚨 | DOM 合约中心 | 建 DOM ID 合约表 |
| 5 | `src/engine/tool-system/src/shell.rs` | 🚨🚨🚨🚨🚨 | 命令执行边界 | 继续从字符串命令转结构化参数 |
| 6 | `src/interface/desktop/tauri.conf.json` | 🚨🚨🚨🚨 | 安全面总闸门 | 每次改动必须安全审查 |
| 7 | `src/interface/web/modules/inspector.js` | 🚨🚨🚨 | 已抽模块，但动态渲染多 | 加 XSS / 交互 smoke |
| 8 | `src/intelligence/agent-core/context_budget.rs` | 🚨🚨🚨 | Long Context 预算核心 | 拆策略/配置/计算 |
| 9 | `src/intelligence/agent-core/long_context_pack.rs` | 🚨🚨🚨 | 大上下文组包核心 | 强化大文件/排除规则测试 |
| 10 | `src/intelligence/agent-core/context_receipt.rs` | 🚨🚨🚨 | 上下文小票/隐私边界 | 继续做脱敏回归测试 |

---

## 3. 逐项体检

## 3.1 `src/interface/web/app.js`

### 判断

这是当前最像“史山代码核心入口”的文件。

它的问题不是单纯行数大，而是承担了过多职责：UI 状态、DOM 绑定、会话、模型、Diff、Trace、Settings、Tauri 调用等都可能和它产生关系。

### 风险

```text
它既像遥控器，又像电视机，又像电表箱，还顺手管了厨房灯。
```

主要风险：

- 全局 `window.app` 单例过重。
- DOM ID 与事件绑定容易失配。
- 新功能继续堆进去，会让后续拆分成本越来越高。
- 一个局部 UI 改动可能误伤聊天、模型、Diff、Trace、Settings。

### 建议动作

优先拆低风险 UI 模块：

```text
command-palette
slash-palette
provider-config
backup/export UI
audit-log
resource-dashboard
agent-cards
```

原则：

```text
每拆一个模块，都保留 wrapper，跑 smoke，再动下一个。
```

### 止损条件

出现以下情况就停：

```text
1. app.js 新增超过 300 行但没有模块拆分计划
2. 删除/改名 DOM ID 后没有对应 smoke test
3. app.js 同时改 UI + Provider + Agent + Checkpoint
```

---

## 3.2 `src/interface/desktop/src/main.rs`

### 判断

这是桌面后端入口的“万能总控台”。

它承载 Tauri command 注册、LLM 客户端管理、配置、密钥、Agent 控制、Trace、Governance、Checkpoint、Dashboard 等职责，变更半径偏大。

### 风险

```text
桌面后端的中央厨房 + 配电房 + 仓库管理员 + 门卫。
```

风险点：

- 新增 Tauri command 很容易继续堆到这里。
- Provider、Agent、Checkpoint、FS、安全逻辑混在一个文件中，定位成本高。
- 一旦文件继续增长，任何桌面端改动都会变得重。

### 建议动作

拆成：

```text
src/interface/desktop/src/
  main.rs                  # 只保留 tauri::Builder 组装
  state.rs                 # AppState
  registry.rs              # build_registry
  commands/
    fs.rs                  # read/write/list/create/rename/delete
    provider.rs            # provider config / keyring
    agent.rs               # run_agent_task / approval / governance
    checkpoint.rs          # checkpoint export/compare/restore
    shell.rs               # shell bridge / command policy
```

### 止损条件

```text
新 Tauri command 不允许继续直接塞 main.rs。
```

---

## 3.3 `src/interface/web/style.css`

### 判断

这是全局样式层面的史山候选。

CSS 的问题通常不是“会不会报错”，而是“布局莫名其妙变形但没人知道谁影响了谁”。

### 风险

```text
JS 炸了还能报错，CSS 炸了通常是按钮飘了、弹窗透明了、间距阴间了。
```

主要风险：

- 全局 class 容易互相污染。
- 后续 UI 模块拆分后，如果 CSS 不拆，模块仍然不算真正独立。
- 样式覆盖链变长后，很难判断哪个规则在生效。

### 建议动作

短期不一定引入构建工具，可以先做“无构建拆分”：

```text
styles/
  base.css
  layout.css
  sidebar.css
  chat.css
  inspector.css
  settings.css
  trace.css
```

如果暂时不能物理拆文件，也至少在 `style.css` 内建立强制分区：

```css
/* === BASE === */
/* === LAYOUT === */
/* === SIDEBAR === */
/* === CHAT === */
/* === INSPECTOR === */
/* === SETTINGS === */
/* === TRACE === */
```

### 止损条件

```text
新增样式必须写进明确区域；
禁止为了修一个按钮，随手加全局 .button / .panel / .item。
```

---

## 3.4 `src/interface/web/index.html`

### 判断

它是前端 DOM 合约中心。

它不一定最大，但非常关键。大量 JS 绑定依赖这里的 DOM ID、class、data attribute。

### 风险

```text
这文件不是普通 HTML，它是前端所有开关、插座、门牌号的地图。
```

风险点：

- 改一个 ID，JS 可能静默失效。
- DOM 结构移动后，事件绑定可能还在旧地方找元素。
- Settings / Inspector / Command Palette 等区域共用 DOM 合约，回归成本高。

### 建议动作

新增：

```text
docs/frontend/DOM-CONTRACT.md
```

记录：

```text
DOM ID
归属模块
谁在 JS 里绑定它
谁在 CSS 里使用它
对应 smoke test
```

### 止损条件

```text
改 DOM ID 必须同 PR 改 app.js / module / style / smoke。
```

---

## 3.5 `src/engine/tool-system/src/shell.rs`

### 判断

这是安全高压线。

当前已经有严格白名单、元字符过滤、禁止 `rm/sudo` 等高危命令，这比裸奔状态好很多。但凡是“执行命令”，都必须继续按高风险处理。

### 风险

```text
它是家里唯一能开大门、煤气、电闸、仓库的总钥匙。
```

风险点：

- 字符串命令天然比结构化参数更难安全验证。
- 为了体验恢复 pipe、redirect、shell script 会重新扩大攻击面。
- 一旦 shell 边界放松，Agent 工具调用风险会被放大。

### 建议动作

下一阶段从：

```json
{
  "command": "git status"
}
```

逐步改成：

```json
{
  "program": "git",
  "args": ["status"]
}
```

也就是从“把一句话交给 shell 理解”，改成“明确执行哪个程序、传哪些参数”。

### 止损条件

```text
禁止为了体验恢复 pipe / redirect / rm / shell script。
禁止把 bash/sh/pwsh/powershell 加回用户命令白名单。
```

---

## 3.6 `src/interface/desktop/tauri.conf.json`

### 判断

这是小文件，但爆炸半径很大。

它是 Tauri WebView 的安全总闸门，CSP、global Tauri API、build 配置都在这里。

### 风险

```text
它不像厨房垃圾桶，更像总电闸。平时没存在感，一动就是全屋黑。
```

风险点：

- CSP 放宽会扩大 XSS 后果。
- `withGlobalTauri` 如果错误打开，会放大前端漏洞影响。
- 构建路径改动会影响桌面包运行。

### 建议动作

建立规则：

```text
tauri.conf.json 改动必须单独 PR / 单独 commit。
```

每次改动必须附带：

```text
1. 为什么改
2. CSP 是否放宽
3. 是否影响 window.__TAURI__
4. WebView smoke 结果
```

### 止损条件

```text
禁止临时把 csp 改回 null；
禁止临时打开 withGlobalTauri 来“先跑起来”。
```

---

## 3.7 `src/interface/web/modules/inspector.js`

### 判断

这是好消息，也是新风险点。

好消息：它已经从 `app.js` 中被拆出来，说明前端巨石治理已经开始。  
风险点：它仍有大量动态渲染、Diff 展示、Trace 展示、Checkpoint 按钮绑定。

### 风险

```text
这是从大杂物间搬出来的独立工具柜。
工具柜独立了，这是好事；
但里面有刀、有剪子、有打火机，所以还得贴标签。
```

风险点：

- 动态 `innerHTML` 渲染较多。
- Diff / Trace / Checkpoint 内容可能包含外部输入。
- 如果未来加入写入命令，它可能长成第二个 `app.js`。

### 建议动作

补两类测试：

```text
1. 恶意内容渲染测试
   <img src=x onerror=alert(1)>
   <script>alert(1)</script>

2. WebView 手动 smoke
   Inspector Tab 切换
   Context Receipt 刷新
   Diff fallback
   Checkpoint restore/compare 按钮
```

### 止损条件

```text
Inspector 不允许新增写入类 Tauri command。
```

它最好保持“只读观察面板”。

---

## 3.8 `src/intelligence/agent-core/context_budget.rs`

### 判断

这是 Long Context 的预算核心，属于“逻辑复杂但不是屎”的类型。

它负责不同模型上下文预算，例如 Legacy 8K、Fast 128K、Pro 200K、Long 1M 等。

### 风险

```text
上下文塞太多 → 爆 token
上下文塞太少 → Agent 变瞎
预算算错 → 用户以为吃自助餐，实际只能夹一片生菜
```

风险点：

- 预算策略出错会直接影响 Agent 能力。
- 不同 Provider / 模型能力差异大。
- Probe verified / stale / fallback 等状态需要稳定测试。

### 建议动作

拆成：

```text
context_budget/
  mod.rs
  profiles.rs       # Legacy/Fast/Pro/Long
  capability.rs     # ModelContextCaps
  policy.rs         # budget calculation
  tests.rs
```

### 止损条件

```text
任何预算算法改动必须有 golden case。
```

至少覆盖：

```text
8K 模型
128K 模型
200K 模型
1M 模型
probe verified
probe stale
fallback
```

---

## 3.9 `src/intelligence/agent-core/long_context_pack.rs`

### 判断

这是大上下文组包核心，风险主要来自“文件读取 + 大文件策略 + token 估算”。

### 风险

```text
这文件像给 Agent 打包行李：
带少了，Agent 出门没裤子；
带多了，行李箱爆炸；
带错了，可能把银行卡密码和客户资料一起塞进去。
```

风险点：

- 大文件读取策略如果不严，会导致 token 爆炸。
- `node_modules`、`target`、`dist` 等目录必须被排除。
- 二进制文件、密钥文件、环境变量文件不能混进上下文。

### 建议动作

优先补测试：

```text
1. 超大文件必须 skip 或 head/tail
2. node_modules / target / dist 必须排除
3. 二进制文件不能塞上下文
4. 敏感文件名必须被拒绝或脱敏
5. 组包结果必须有 receipt
```

### 止损条件

```text
禁止 Full 读取未知大文件作为默认行为。
```

---

## 3.10 `src/intelligence/agent-core/context_receipt.rs`

### 判断

这是隐私边界文件，行数不是最吓人，但责任很重。

它负责记录上下文小票：记录哪些内容被放入上下文、哪些被省略、预算怎么分配，但不应该记录 API key、完整 prompt、完整文件正文等敏感信息。

### 风险

```text
本来想给用户一张购物小票，
结果把银行卡密码、身份证号、厨房监控录像一起打印出来。
```

风险点：

- receipt 如果记录过多，会变成隐私泄露源。
- 脱敏规则如果覆盖不足，会泄露 token / secret / api_key。
- summary 长度限制如果失效，会把大量内容写入小票。

### 建议动作

继续保持独立，并补：

```text
redaction golden tests
receipt size limit tests
no full file body tests
no API key tests
```

### 止损条件

```text
receipt 里禁止出现完整 prompt、完整文件正文、环境变量值、API key。
```

---

# 4. 当前免死金牌区

## 4.1 `src/patches/zstd-sys/`

这类文件看起来可能很吓人，但不建议作为第一治理对象。

原因：它更像上游补丁 / bindgen 类代码，不是手写业务史山。

处理建议：

```text
不要拿它当第一治理对象。
```

## 4.2 `src/intelligence/agent-core/` 整体

Agent Core 里确实有较大的文件，比如 `context_budget.rs`、`long_context_pack.rs`。  
但它整体有模块化、测试和文档，不能简单按“文件大”判死刑。

处理建议：

```text
只做局部拆分，不做大动脉手术。
```

---

# 5. 第一阶段治理路线

## Phase A：只读对账，不改代码

目标：先把“哪里危险”钉死。

```text
1. 统计 Top 30 大文件
2. 统计 TODO/FIXME/DEBT
3. 统计 innerHTML / unwrap / panic
4. 统计没有 smoke 的 UI 模块
5. 生成 docs/debt/STONE-AUDIT-V1.md
```

## Phase B：冻结巨石增长

规则：

```text
app.js 不新增大功能
main.rs 不新增 command
style.css 不新增无归属全局样式
index.html 不改 DOM ID，除非同步测试
```

## Phase C：从低风险模块继续拆

优先顺序：

```text
1. command-palette
2. slash-palette
3. audit-log
4. resource-dashboard
5. provider-config 只读展示部分
```

暂时不要拆：

```text
Agent streaming
checkpoint restore
provider keyring
shell execution
```

这些属于高爆区，不适合第一刀。

---

# 6. 本地一键体检命令

## 6.1 Windows PowerShell

```powershell
$Out = "docs/debt/stone-audit-v1-$(Get-Date -Format yyyyMMdd-HHmmss).txt"
New-Item -ItemType Directory -Force docs/debt | Out-Null

"# Stone Audit V1" | Out-File $Out -Encoding utf8

"`n## Git" | Out-File $Out -Append -Encoding utf8
git branch --show-current | Out-File $Out -Append -Encoding utf8
git rev-parse HEAD | Out-File $Out -Append -Encoding utf8
git status --short | Out-File $Out -Append -Encoding utf8

"`n## High-risk file line counts" | Out-File $Out -Append -Encoding utf8
$files = @(
  "src/interface/web/app.js",
  "src/interface/web/style.css",
  "src/interface/web/index.html",
  "src/interface/desktop/src/main.rs",
  "src/engine/tool-system/src/shell.rs",
  "src/interface/desktop/tauri.conf.json",
  "src/interface/web/modules/inspector.js",
  "src/intelligence/agent-core/context_budget.rs",
  "src/intelligence/agent-core/long_context_pack.rs",
  "src/intelligence/agent-core/context_receipt.rs"
)

foreach ($f in $files) {
  if (Test-Path $f) {
    $count = (Get-Content $f).Count
    "$count`t$f" | Out-File $Out -Append -Encoding utf8
  } else {
    "MISSING`t$f" | Out-File $Out -Append -Encoding utf8
  }
}

"`n## Risk keywords" | Out-File $Out -Append -Encoding utf8
Select-String -Path $files -Pattern "TODO|FIXME|DEBT|innerHTML|unwrap\(|panic!|withGlobalTauri|csp|null" -ErrorAction SilentlyContinue |
  ForEach-Object { "$($_.Path):$($_.LineNumber): $($_.Line.Trim())" } |
  Out-File $Out -Append -Encoding utf8

"`n## Suggested checks" | Out-File $Out -Append -Encoding utf8
"node --check src/interface/web/app.js" | Out-File $Out -Append -Encoding utf8
"npm run test:security-gate" | Out-File $Out -Append -Encoding utf8
"cargo check --workspace" | Out-File $Out -Append -Encoding utf8

Write-Host "Wrote $Out"
```

## 6.2 macOS / Linux / Git Bash

```bash
mkdir -p docs/debt
OUT="docs/debt/stone-audit-v1-$(date +%Y%m%d-%H%M%S).txt"

{
  echo "# Stone Audit V1"

  echo
  echo "## Git"
  git branch --show-current
  git rev-parse HEAD
  git status --short

  echo
  echo "## High-risk file line counts"
  for f in \
    "src/interface/web/app.js" \
    "src/interface/web/style.css" \
    "src/interface/web/index.html" \
    "src/interface/desktop/src/main.rs" \
    "src/engine/tool-system/src/shell.rs" \
    "src/interface/desktop/tauri.conf.json" \
    "src/interface/web/modules/inspector.js" \
    "src/intelligence/agent-core/context_budget.rs" \
    "src/intelligence/agent-core/long_context_pack.rs" \
    "src/intelligence/agent-core/context_receipt.rs"
  do
    if [ -f "$f" ]; then
      wc -l "$f"
    else
      echo "MISSING $f"
    fi
  done

  echo
  echo "## Risk keywords"
  grep -RInE "TODO|FIXME|DEBT|innerHTML|unwrap\(|panic!|withGlobalTauri|csp|null" \
    src/interface/web/app.js \
    src/interface/web/style.css \
    src/interface/web/index.html \
    src/interface/desktop/src/main.rs \
    src/engine/tool-system/src/shell.rs \
    src/interface/desktop/tauri.conf.json \
    src/interface/web/modules/inspector.js \
    src/intelligence/agent-core/context_budget.rs \
    src/intelligence/agent-core/long_context_pack.rs \
    src/intelligence/agent-core/context_receipt.rs 2>/dev/null || true

  echo
  echo "## Suggested checks"
  echo "node --check src/interface/web/app.js"
  echo "npm run test:security-gate"
  echo "cargo check --workspace"
} > "$OUT"

echo "Wrote $OUT"
```

---

# 7. 最终处方

## 7.1 立即做

```text
1. 生成本地 stone-audit-v1 文件
2. 冻结 app.js / main.rs / style.css 的无规划增长
3. 为 DOM ID 建合约表
4. 继续拆 app.js 里低风险 UI 模块
5. 补真实 Tauri WebView manual smoke
```

## 7.2 暂时别做

```text
1. 不要全量重构前端
2. 不要立刻上 React/Vue/Vite
3. 不要大改 Agent Core
4. 不要放宽 shell 白名单
5. 不要把 CSP 改松
```

## 7.3 总结

这项目不是没救的史山。  
它更像：

> **大体量项目已经长出技术债，但已经开始打标签、拆模块、补测试、写清债记录。现在最关键的是别再让核心巨石继续变大。**

换成人话：

```text
厨房已经很大，锅也很多，确实有几个柜子乱到离谱。
但不是黑暗料理作坊。
现在要做的是：别继续往万能抽屉里塞东西，先给刀、锅、煤气阀分区。
```

---

# 8. 后续建议版本

建议后续继续做两份文档：

```text
STONE-AUDIT-V2-FULL-SCAN.md
```

用于完整 clone 后统计：

```text
Top 30 大文件
复杂度 Top 30 函数
TODO/FIXME/DEBT 真实数量
innerHTML / unsafe / unwrap / panic 统计
测试覆盖缺口
大文件与仓库体积分析
```

以及：

```text
FRONTEND-DOM-CONTRACT.md
```

用于锁住前端 DOM 合约，防止 `index.html / app.js / style.css / modules` 四方漂移。
