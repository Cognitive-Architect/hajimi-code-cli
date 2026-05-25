# Hajimi 代码级债务优先级指南

> **更新日期**: 2026-05-24  
> **适用范围**: `docs/debt/code-level/` 下的全部 7 份活跃债务文档  
> **交叉验证**: 已对照 `active/ACTIVE-DEBT-STATUS-2026-05-17.md` 状态矩阵与 `archive/05/debt-history/DEBT-B18-SECURITY-HARDENING-CLOSURE.md` 闭环收据

---

## 优先级定义

| 等级 | 含义 | 通俗解释 |
|:---:|:---|:---|
| **P0** | 🔴 **架构级** — 影响系统安全边界或分层合规，不修就可能"塌房" | 地基有裂缝，不管其他功能多好看，先堵裂缝 |
| **P1** | 🟠 **功能级** — 核心功能跑不通或跑出来的结果不对 | 门能开但锁是坏的，用户会直接感知到问题 |
| **P2** | 🟡 **优化级** — 能用但不够好，体验粗糙或维护痛苦 | 家具摆放不合理，住着不舒服但不影响安全 |
| **P3** | 🟢 **长期级** — 未来有空再做，现在不做也完全不影响使用 | 想在花园种棵树，什么时候种都行 |

---

## P0 — 架构级债务（地基裂缝，优先堵）

### P0-1: 安全审计工具的扫描覆盖面仍偏弱
> 📄 来源: [HAJIMI-CODEX-SECURITY-VALIDATION-REPORT.md](file:///f:/hajimi-code-cli/docs/debt/code-level/HAJIMI-CODEX-SECURITY-VALIDATION-REPORT.md) (F-002, F-005, F-006)

| 子项 | 当前状态 | 说明 |
|:---|:---:|:---|
| 通用文件工具 workspace 沙箱 | ⚠️ 部分修复 | B18 已为 desktop registry 注入了 `with_allowed_paths`，但 `EditFileTool` 的 canonical workspace 检查需进一步验证覆盖完整性 |
| `apply_edits` / `preview_edit` 路径校验 | ⚠️ 部分修复 | B18 已将 `preview_edit` 接入 `resolve_workspace_path`，但需要负向测试（传入 workspace 外绝对路径时必须拒绝） |
| Provider config workspace_path 信任前端 | ⚠️ 未修复 | 后端仍接受前端传入的 `workspace_path` 拼接配置文件路径，应改为后端维护 canonical workspace，不信任前端传参 |

**为什么是 P0**：这些都是安全沙箱边界问题。如果工具系统允许跨 workspace 读写文件，等于给了 Agent 或前端脚本突破沙箱的能力，是架构级的安全红线。

**建议行动**：
1. 为 `EditFileTool` 补充 canonical workspace root 检查
2. 为 `apply_edits` / `preview_edit` 添加路径逃逸的负向测试用例
3. Provider config 的 workspace_path 由后端 canonicalize 后再使用

---

### P0-2: Security Gate 反回退规则已补全
> 📄 来源: [HAJIMI-SECURITY-RESIDUAL-FIX-GUIDE.md](file:///f:/hajimi-code-cli/docs/debt/code-level/HAJIMI-SECURITY-RESIDUAL-FIX-GUIDE.md) (R-004)

| 子项 | 当前状态 | 说明 |
|:---|:---:|:---|
| `execute_tool` 权限门禁回退检测 | ✅ 已有 | Gate 会检测 `enforce_tool_permissions` 是否存在 |
| `withGlobalTauri=true` 回退检测 | ✅ 已有 | Gate 对 `withGlobalTauri=true` 会 fail |
| `run_command` 裸暴露回退检测 | ✅ 已有 | Gate 检测 `run_command` 是否仍在 invoke_handler |
| 文件工具 `with_allowed_paths` 回退检测 | ✅ 已补 | 如果有人把 desktop file tools 改回 `new()` 无约束版本，Gate 会报错 |
| inline edit resolver 回退检测 | ✅ 已补 | `apply_edits` 去掉 `resolve_workspace_path` 后 Gate 会捕获 |
| Provider config workspace 校验检测 | ✅ 已补 | 各个 provider 相关的 command 必须通过 `trusted_workspace_path` 等校验才可通过 |

**为什么是 P0**：Security Gate 是防止安全修复被意外回滚的最后一道防线。如果关键修复点没有对应的回退检测规则，未来任何一次重构都可能静默地把安全边界打开。

**建议行动**：
1. 在 `tests/security/security_audit_gate.js` 中新增 `desktop-file-tools-workspace-bound` 规则 (已完成)
2. 新增 `inline-edit-resolver` 规则，确保 `apply_edits` 始终走 `resolve_workspace_path` (已完成)
3. 新增 `desktop-provider-workspace-security` 规则，确保 provider 命令进行 workspace 路径安全校验 (已完成)

---

## P1 — 功能级债务（核心功能有缺口）

### P1-1: 前端模块化仍然不完整
> 📄 来源: [DEBT-P0-UI-INTERACTION-REMEDIATION.md](file:///f:/hajimi-code-cli/docs/debt/code-level/DEBT-P0-UI-INTERACTION-REMEDIATION.md) / AD-002, AD-004

| 子项 | 当前状态 | 说明 |
|:---|:---:|:---|
| `app.js` 行数膨胀 | ⚠️ ~5000 行 | 所有 UI 逻辑堆在单文件中，事件绑定与 DOM 查找脆弱，任何 UI 改动都有回归风险 |
| `style.css` 行数膨胀 | ⚠️ ~3200 行 | 样式层叠覆盖难以维护 |
| 已拆出的模块 | ✅ 部分完成 | `security-dom.js`、`workspace.js`、`sessions.js`、`thinking-ui.js`、`slash-palette.js`、`tauri-bridge.js` 已独立 |
| 待拆出的模块 | ❌ 未开始 | `settings.js`、`inspector.js`、`agent-cards.js`、`command-palette.js`、`provider.js` |

**为什么是 P1**：`app.js` 单体膨胀已经到了"改一行可能炸十处"的临界点。每次新增功能都在增加回归风险，这不是"体验不好"的问题，而是"继续加功能会越来越难"的功能级瓶颈。

**建议行动**：
1. 按优先级依次拆出 `settings.js` → `inspector.js` → `agent-cards.js`
2. 每次拆分后必须跑 `node --check` + `day13_workspace_modules_smoke.js`
3. 禁止在 `app.js` 中再堆砌超过 500 行的新功能逻辑

---

### P1-2: Thinking UI 流式解析存在跨 Chunk 截断缺陷
> 📄 来源: [DEBT-THINKING-UI.md](file:///f:/hajimi-code-cli/docs/debt/code-level/DEBT-THINKING-UI.md) / AD-005

| 子项 | 债务ID | 当前状态 | 说明 |
|:---|:---|:---:|:---|
| `parseThinkingStream` 跨 chunk 标签截断 | DEBT-B09-001 | ⚠️ 未修 | 如果 `<thinking>` 标签恰好被切割在两个 SSE chunk 之间，解析器会丢失该思考块，用户看不到 Agent 的推理过程 |
| `streamChat` 与 `addThinking` 短暂双 div | DEBT-B09-002 | ⚠️ 未修 | 流式响应时，思考内容可能短暂出现两个重复 DOM 节点，体验闪烁 |
| `TokenEvent` 未被后端 provider 使用 | DEBT-B09-003 | ⚠️ 未修 | 定义了 Token 级事件但后端 LLM 客户端还未实际发送，前端无法做到 Token 级思考动画 |

**为什么是 P1**：Thinking UI 是 Hajimi IDE 最核心的差异化体验之一——让用户看到 Agent 是"怎么想的"。跨 chunk 截断会直接导致推理过程丢失，这是功能层面的正确性问题。

**建议行动**：
1. 在 `parseThinkingStream` 中维护一个跨 chunk 的 buffer，处理标签被截断的情况
2. 解决双 div 闪烁问题（在创建新 thinking div 前先检查是否已存在未关闭的）

---

### P1-3: 1M 长上下文探针仍为 Gated 状态
> 📄 来源: [DEBT-LONG-CONTEXT-1M.md](file:///f:/hajimi-code-cli/docs/debt/DEBT-LONG-CONTEXT-1M.md)

| 子项 | 当前状态 | 说明 |
|:---|:---:|:---|
| 动态预算解析引擎 | ✅ 已完成 | `resolve_context_budget` 纯引擎已就位 |
| Bridge 8K 硬编码消除 | ✅ 已完成 | Planner/Reflector bridge 已动态化 |
| Provider 能力字段扩展 | ✅ 已完成 | camelCase `maxContextTokens` 等已注入 Blackboard |
| Memory 检索预算动态分配 | ✅ 已完成 | Focus/Working/Archive 三层按比例分配 |
| **真实 Provider Probe** | ⚠️ Gated | 已实现默认关闭的 Gated 真实探针通道、显式 ProbeResult cache API 及降级梯度；真实 provider receipt 仍为 pending。 |
| Context 小票 token 估算 | ⚠️ 估算值 | 小票中的 token 数是词法估算，不是 LLM 服务端实际计费值 |

**为什么是 P1**：1M 上下文是 Hajimi 对接高端模型的关键能力。目前预算引擎、Bridge、Memory 分配都已就位，但"最后一公里"——真实探针还没接上。这意味着系统无法确认某个 Provider 是否真正支持 1M 窗口，只能盲目信任配置声明。

**建议行动**：
1. 实现 `RealProviderProbe`，通过实际 API 调用验证 Provider 的上下文窗口容量（已以 Gated 形式实现 `ProviderProbeClient` 与 `run_probe` 通道）
2. ProbeResult 显式 save/load API 与 TTL/fallback 测试已覆盖；真实 provider receipt 仍 pending
3. 探针失败时触发 Fallback 降级（900K → 512K → 256K → 128K → 32K）（已完成，预算解析级联降级已整合）

---

## P2 — 优化级债务（能用但不够好）

### P2-1: Agent Skills V0c 部分完成，图记忆与云同步延后
> 📄 来源: [DEBT-AGENT-SKILLS-V0.md](file:///f:/hajimi-code-cli/docs/debt/code-level/DEBT-AGENT-SKILLS-V0.md) / AD-009

| 子项 | 状态 |
|:---|:---:|
| V0a 路由/加载/打分/注入/评估 | ✅ 已清除 |
| V0b 运行时沙盒/ActExecutor 过滤 | ✅ 已清除 |
| V0c Blackboard 执行小票 | ✅ 已完成 |
| V0c 图记忆 (Graph Memory) | 🔜 延后至 V1 |
| V0c 云端同步 (Cloud Sync) | 🔜 延后至 V1 |
| V0c Interface 列表展示/校验 | 🔜 延后至 V1 |
| 2 字符滑动窗口误触发风险 (V0A-002) | ⚠️ 已知风险 |
| Legacy 简单 prompt 路径不注入技能指令 (V0A-003) | ⚠️ 已知局限 |
| CI/Clippy 未完整回填 (B12-CI-CLIPPY) | ⚠️ 已知局限 |

**为什么是 P2**：Skills V0 的核心管道（打分→路由→加载→注入→过滤→评估→小票）已经完整运转。剩余的图记忆和云同步属于"让技能系统更强大"的增量优化，不影响当前使用。

---

### P2-2: Thinking UI 虚拟 Diff 与规则推理
> 📄 来源: [DEBT-THINKING-UI.md](file:///f:/hajimi-code-cli/docs/debt/code-level/DEBT-THINKING-UI.md)

| 子项 | 债务ID | 说明 |
|:---|:---|:---|
| Diff 预览是虚拟 diff，不是真实 git diff | DEBT-B11-001 | 用户看到的代码变更对比是模拟的，未接入真实的 git diff 算子 |
| `renderMarkdown` 不支持表格和嵌套列表 | DEBT-B08-002 | 轻量 Markdown 解析器功能有限，复杂格式会降级为纯文本 |
| AgentLoop 真实事件仅在 Tauri WebView 可用 | DEBT-B04-001 | 非 WebView 环境下拿不到真实事件流 |
| `stream_chat_with_context` 未使用 | DEBT-B08-001 | 定义了但未接入，存在死代码 |
| TimelineEvent 未绑定后端 Checkpoint | DEBT-B12-001 | 时间线组件是纯前端状态，没有与后端持久化的 Checkpoint 对齐 |

**为什么是 P2**：这些都是"体验可以更好"的优化项。虚拟 diff 不影响 Agent 的实际执行能力，Markdown 表格不影响核心对话流，但修好它们会让产品质感上一个台阶。

---

### P2-3: 安全审计门禁精度需提升
> 📄 来源: [HAJIMI-CODEX-SECURITY-VALIDATION-REPORT.md](file:///f:/hajimi-code-cli/docs/debt/code-level/HAJIMI-CODEX-SECURITY-VALIDATION-REPORT.md) (F-003 残余) / AD-008

| 子项 | 状态 | 说明 |
|:---|:---:|:---|
| `withGlobalTauri` 已关闭 | ✅ | B18 已设为 `false` |
| `__TAURI__` 散落调用已集中 | ✅ | 高风险路径已迁移到 `tauri-bridge.js` |
| legacy `innerHTML` allowlist 仍有 108 项 | ⚠️ | 历史 DOM 危险 API 用法被登记为 warning，需逐步清理 |
| Security Gate 只做正则匹配 | ⚠️ | 当前门禁是文本扫描级别，不具备 AST 级语法分析能力 |

**为什么是 P2**：安全门禁已经能防住主要回退风险，但 108 项 allowlist 意味着还有大量历史 `innerHTML` 需要逐步替换为安全 DOM API。这是渐进式清理工作，不是紧急修复。

---

## P3 — 长期级债务（什么时候做都行）

### P3-1: Shell 高级功能恢复
> 📄 来源: [SHELL-FEATURE-DEBT-002.md](file:///f:/hajimi-code-cli/docs/debt/code-level/SHELL-FEATURE-DEBT-002.md) / AD-001

当前被主动禁用的高级 Shell 功能：

| 被禁用的功能 | 恢复前提 |
|:---|:---|
| 管道 `\|` | Rust-side shlex AST 解析器 + 多 Command 链式组装 |
| 重定向 `>` `>>` | 输出目标 workspace 沙箱校验 |
| 命令替换 `$(cmd)` | 子命令白名单 + 递归深度限制 |
| 逻辑运算符 `&&` `\|\|` `;` | 每段命令独立权限校验 |
| 后台运行 `&` `nohup` | 进程生命周期管理 + 超时 + 资源限制 |
| 环境变量展开 | 敏感变量过滤（API Key 等） |

**为什么是 P3**：这是一个 **设计驱动的主动降级**，不是 bug。当前白名单模式已经能满足日常开发的 `git`/`cargo`/`npm`/`node` 需求。恢复高级功能需要引入完整的 Shell AST 解析器和沙箱隔离，工程量大且优先级低于产品功能迭代。

---

### P3-2: Thinking UI 推理生成 LLM 化
> 📄 来源: [DEBT-THINKING-UI.md](file:///f:/hajimi-code-cli/docs/debt/code-level/DEBT-THINKING-UI.md)

| 子项 | 债务ID | 说明 |
|:---|:---|:---|
| 理由生成基于规则匹配，非 LLM | DEBT-B11-002 | 当前 Agent 操作理由是模板填充，不是 LLM 动态生成 |
| Replay 只读回放 | DEBT-B12-002 | 时间线回放只能看，不能从某个历史节点重新分支执行 |

**为什么是 P3**：这两项都是"锦上添花"的能力。规则模板生成的理由已经够用；历史分支重放是极高级的 IDE 功能，可以作为 v2 的差异化特性来规划。

---

### P3-3: Agent Prompt 产品化深水区
> 📄 参考: AD-006（来源文档已归档）

| 子项 | 状态 | 说明 |
|:---|:---:|:---|
| Agent Persona 人格设定 | ✅ 已完成 | |
| Context Window Manager | ✅ 已完成 | |
| Tool Manifest Schema | ✅ 已完成 | |
| Prompt Golden Tests | ✅ 已完成 (6 cases) | |
| 运行时一致性验证 | 🔜 未开始 | 确保 Persona + 工具清单在不同模型/配置下的行为一致 |
| 产品级评分体系 | 🔜 未开始 | 超越确定性 Golden Case 的动态评估 |

**为什么是 P3**：Prompt 工程的基础设施（Persona、DTO 契约、Golden Tests）已经完备。"产品化"意味着在更多真实场景下验证和调优，这是一个持续迭代的过程，不是一次性修复。

---

## 📊 总览：一张表看清全部

| 等级 | 编号 | 债务名称 | 当前状态 | 预估工作量 |
|:---:|:---|:---|:---:|:---:|
| 🔴 P0 | P0-1 | 安全沙箱边界补全（文件工具/编辑器/Provider） | ⚠️ 部分修复 | 1-2 天 |
| 🔴 P0 | P0-2 | Security Gate 反回退规则补全 | ⚠️ 部分覆盖 | 0.5 天 |
| 🟠 P1 | P1-1 | 前端 `app.js` 模块化拆分 | ⚠️ 部分拆出 | 3-5 天 |
| 🟠 P1 | P1-2 | Thinking UI 跨 chunk 流式解析修复 | ❌ 未修 | 1 天 |
| 🟠 P1 | P1-3 | 1M 长上下文真实 Provider Probe | ⚠️ Gated | 2-3 天 |
| 🟡 P2 | P2-1 | Agent Skills V0c 图记忆/云同步 | 🔜 延后 | 5+ 天 |
| 🟡 P2 | P2-2 | Thinking UI 虚拟 Diff / Markdown 增强 | ⚠️ 基础版 | 2-3 天 |
| 🟡 P2 | P2-3 | 安全审计门禁精度提升 (innerHTML 清理) | ⚠️ 108 项 | 3-5 天 |
| 🟢 P3 | P3-1 | Shell 高级功能恢复（管道/重定向/子命令） | 🔒 设计冻结 | 10+ 天 |
| 🟢 P3 | P3-2 | Thinking UI 推理 LLM 化 / 历史分支重放 | 🔜 规划中 | 5+ 天 |
| 🟢 P3 | P3-3 | Agent Prompt 产品化深水区 | 🔜 持续迭代 | 持续 |

---

## 🎯 推荐执行顺序

```
第 1 步: P0-1 + P0-2 （安全边界补全 + Gate 规则，约 2 天）
    ↓
第 2 步: P1-2 （Thinking UI 流式解析修复，约 1 天）
    ↓
第 3 步: P1-3 （1M 真实 Provider Probe，约 2-3 天）
    ↓
第 4 步: P1-1 （前端模块化拆分，可分批进行）
    ↓
第 5 步: P2 级别（根据产品需要灵活安排）
    ↓
长期:   P3 级别（融入日常迭代，不设 deadline）
```

> [!TIP]
> P0 必须在任何新功能开发之前完成。P1 可以穿插在功能迭代中逐步消化。P2 和 P3 不需要专门排期，融入日常开发节奏即可。
