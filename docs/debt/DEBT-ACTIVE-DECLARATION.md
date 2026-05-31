# DEBT-ACTIVE-DECLARATION — Hajimi 活跃技术约束声明

> **ID**: `DEBT-ACTIVE-DECLARATION`
> **更新时间**: 2026-05-31
> **状态**: 🟡 **ACTIVE**（持续维护中）
> **分支**: `feature/toolfix-deepseek-schema`
> **当前 HEAD**: `516e1fb2d53820cd3de8a3011d2d77094adddf58`

---

## 1. 活跃债务矩阵（Active Debt Matrix）

以下债务为当前代码基线中**仍未完全清偿**的技术约束，按优先级排序：

| ID | 债务名称 | 状态 | 优先级 | 来源文档 | 当前真相 |
|:---|:---|:---|:---:|:---|:---|
| AD-001 | Shell 功能降级 | `OPEN BY DESIGN` | P2 | `SHELL-FEATURE-DEBT-002.md` | 管道、重定向、变量、子 Shell 等复杂功能在设计层面保持禁用，直到沙箱、审计、超时和审批控制全部完善 |
| AD-002 | Tauri global API 迁移 | `IMPLEMENTED / PENDING-UI-SMOKE` | P1 | `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | `withGlobalTauri: false` 已加固，高风险 API 已集中至 `tauri-bridge.js`，仅缺真实 WebView GUI 验证 |
| AD-003 | Tauri GUI/WebView 真机验证阻塞 | `ACTIVE BLOCKED / MANUAL` | P1 | `DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md` | 启动、文件树、会话、恶意 DOM 样本、文件操作点击、Slash Palette 交互需真实 Tauri 窗口证据 |
| AD-004 | 前端模块化 | `PARTIAL / IMPROVED` | P2 | `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | `security-dom`、`workspace`、`sessions`、`thinking-ui`、`slash-palette`、`inspector`、`settings-panel` 已提取并通过 Node smoke，剩余二级模块待分解 |
| AD-005 | Thinking UI 与 Checkpoint 深度 | `IMPLEMENTED / PENDING-UI-SMOKE` | P1/P2 | `DEBT-THINKING-UI.md` | SSE 分块与畸形 Token 的流式解析器已完成 TDD；Checkpoint 管理已集成；Trace/Checkpoint 深度待 WebView 验证 |
| AD-006 | Agent Prompt 产品化 | `PARTIAL / IMPROVED` | P2 | `archive/05/debt-history/DEBT-AGENT-PROMPT-001.md` | Agent Persona、上下文窗口管理器、Tool Manifest、Prompt Golden 已就绪；运行时一致性、更广泛策略集成、产品评分待完善 |
| AD-007 | Slash 命令建议面板 | `IMPLEMENTED / PENDING-UI-SMOKE` | P1/P2 | `archive/05/debt-history/02-slash-command-palette.md` | Slash Palette V1 已实现并通过 Node smoke，仅缺真实 WebView 交互证据 |
| AD-008 | SecurityAuditTool 质量 | `IMPLEMENTED / GATED` | P2 | `tests/security/security_audit_gate.js` | 安全审计门 V1 已通过反回归门；AST 级语法扫描、更窄 Allowlist 精度、更广 Sink 覆盖待补充 |
| AD-009 | Agent Skills V0 集成 | `PARTIAL` | P2 | `DEBT-AGENT-SKILLS-V0.md` | V0a（Manifest/Registry/Router/Planner-Reflector 注入/输出评估）和 V0b（受限运行时权限）已完成；V0c（Blackboard Receipts 和 Templates）部分完成；Graph Memory、Cloud Sync、Interface list-validate 延期 |
| AD-011 | Agent Governance UI 审批等待 | `IMPLEMENTED / PENDING-UI-SMOKE` | P1 | `DEBT-AGENT-GOVERNANCE-UI-WAITING.md` | Required/Critical 级别需用户干预；Oneshot 阻塞 Tokio 线程；动态毛玻璃 UI Modal 就绪；待物理 WebView 验证 |
| AD-012 | Agent Checkpoint Diff 预览 UI | `EXPLORED / PARTIAL-UI` | P1 | `DEBT-AGENT-CHECKPOINT-DIFF-UI.md` | Trace Event 不含细粒度原始文件 Diff 或物理内容快照；通过 Premium Checkpoint ID Badge 链接和诚实用户警告缓解 |
| AD-015 | Agent LLM-Native 审批等待卡死 | `CODE-LEVEL FIXED + RELEASE PACKAGED / PENDING-REAL-WEBVIEW-SMOKE` | P0 | `DEBT-AGENT-LLM-NATIVE-APPROVAL-HANG.md` | 已在代码层修复只读工具 Critical 误判、重复审批等待、审批无超时和 Trace 不可见问题，并完成 Tauri release 重新打包；完整关闭前仍需真实 WebView `/agent` smoke |

---

## 2. 本次迁移新增 / 变更债务

| ID | 债务名称 | 状态 | 优先级 | 来源文档 | 说明 |
|:---|:---|:---|:---:|:---|:---|
| — | `DEBT-AGENT-LOOP-LLM-NO-OP` | ✅ **CLOSED** | P0 | `DEBT-AGENT-LOOP-LLM-NO-OP.md` | Day 1-7 已修复执行链路，Agent Core 真实 ToolCall 闭环就绪 |
| — | `DEBT-AGENT-CHINESE-I18N` | ✅ **CLOSED** | P0 | `DEBT-AGENT-CHINESE-I18N.md` | Day 8-23 已彻底割除关键词篡改逻辑，Raw User Intent 直达 LLM |
| DEBT-LLM-NATIVE-TEST-MOCK-PROBE | LLM-Native ContextProbe 实机测试债务 | 🟡 **OPEN** | P2 | `DEBT-AGENT-LLM-NATIVE-MIGRATION.md` §4 | 长上下文探针在离线/CI 中仍为 MockOnly，未经过公有云高并发实机测试 |
| DEBT-MCP-SCHEMA-001 | MCP Schema 嵌套可读性债务 | 🟡 **OPEN** | P2 | `DEBT-AGENT-LLM-NATIVE-MIGRATION.md` §4 | 多层嵌套 JSON-Schema 可读性待优化，未来复杂多参数工具可维护性风险 |

---

## 3. 已归档 / 已清偿债务（Closed / Archived）

| ID | 债务名称 | 归档日期 | 归档依据 |
|:---|:---|:---|:---|
| AD-010 | Agent UI 集成 | 2026-05-25 | 代码和自动化层面已关闭；`DEBT-AGENT-UI-INTEGRATION.md`、`DEBT-AGENT-UI-REMEDIATION.md` 标记 CLOSED |
| AD-013 | Agent Loop LLM No-Op | 2026-05-28 | `DEBT-AGENT-LOOP-LLM-NO-OP.md` 标记 CLOSED；`DEBT-AGENT-LLM-NATIVE-MIGRATION.md` 终局归档 |
| AD-014 | Agent 中文意图篡改 | 2026-05-30 | `DEBT-AGENT-CHINESE-I18N.md` 标记 CLOSED；`DEBT-AGENT-LLM-NATIVE-MIGRATION.md` 终局归档 |

---

## 4. 手动验证债务（Manual Verification Debt — 不计入未处理计数）

以下项目**不属于未处理实现债务**，但关闭前必须提供真实 Tauri/WebView 证据：

| 领域 | 所需证据 |
|:---|:---|
| 启动 / 文件树 / 会话 | Tauri 窗口正常打开；无启动错误 Toast；工作区树渲染；会话 A/B 切换和重启持久化工作 |
| 工作区文件操作 | 真实窗口点击创建文件夹、重命名、删除、刷新；后端日志显示专用命令而非 Shell 文件操作 |
| 安全 DOM 样本 | 恶意文本样本在真实 WebView 中安全渲染，无脚本执行、无 Console 错误 |
| Slash Palette V1 | 输入 `/`、过滤 `/c`、使用 ArrowUp/ArrowDown、Enter、Esc、非 Slash 正常发送，检查 Console/后端日志 |
| Thinking UI / Checkpoint | 真实 WebView Trace/Checkpoint UX 在 UI 关闭前需验证 |

---

## 5. 更新日志

| 日期 | 操作 | 操作人 |
|:---|:---|:---|
| 2026-05-17 | 初始创建，DebtFix V5 Day 1-11 收卷 | Architect |
| 2026-05-25 | 修订注入 DebtFix V5 最终状态 | Architect |
| 2026-05-30 | LLM-Native Migration 终局归档：新增 CLOSED 债务、新增遗留微小债务、同步 AD-013/AD-014 归档 | Architect |
| 2026-05-31 | 新增 AD-015：记录 `/agent` DeepSeek 工具调用后卡在本地治理审批等待的 P0 运行时债务 | Codex |
| 2026-05-31 | AD-015 代码级修复：只读工具低风险分类、审批 30s 超时、单一审批点、LLM-Native Trace 转发 | Codex |
| 2026-05-31 | AD-015 打包回执：Tauri release 产出 exe、msi、nsis 安装包，等待真实 WebView `/agent` smoke | Codex |

---

*活跃约束声明持续维护中。任何新增、变更或关闭债务必须同步更新本文件。*
