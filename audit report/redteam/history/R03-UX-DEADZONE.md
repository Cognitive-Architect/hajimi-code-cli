# R03 UX 死亡地带审计报告：Hajimi V3 用户可及性红队评估

**审计角色**: User Experience Destroyer（ adversarial audit ）  
**审计范围**: 终端 UI、MCP 服务器、VSCode 插件、Web UI、P2P 同步配置、错误提示体系  
**整体风险评级**: **High（高）**

---

## 1. Executive Summary（后果优先）

Hajimi V3 的引擎层囤积了 40+ 工具，但用户能实际触到的接口却像一片沙漠。大量高价值功能（安全审计、ADR 知识检索、TypeRacing 类型预测）被锁在库代码里，没有任何 CLI、MCP、TUI 或 IDE 入口。会导致用户在最关键的安全扫描和知识检索场景中被完全隔绝；会导致通过 MCP 接入的 AI Agent 只能使用不到 8% 的引擎能力；会导致 IDE 用户面对的 60 条命令中 56 条是空壳 stub，点击后仅弹出 `Executing: ${cmd}` 的 toast。整个系统的“功能表面积”与“用户可及表面积”之间存在数量级鸿沟，工程投入被大量浪费在不可见的 dark code 上。

---

## 2. 16-Item Adversarial Checklist（U1–U16）

| ID | 检查项 | 状态 | 后果说明 |
|:---|:---|:---:|:---|
| U1 | ADR 知识库搜索具备可访问入口（CLI/MCP/TUI/IDE） | [❌] | `AdrSearch` 是纯库包装，无任何调用路径。会导致用户无法检索已构建的 ADR 知识图谱。 |
| U2 | `SecurityAuditTool` 暴露于 MCP 服务器 | [❌] | MCP 仅注册 3 个工具，security_audit 不在其中。会导致 AI Agent 无法执行代码密钥/密钥泄露扫描。 |
| U3 | TypeRacing 引擎集成于终端 UI | [❌] | `terminal_adapter.rs` 已实现 Ctrl+Space 触发，但 `InkApp` 未引用该适配器。会导致 LSP 驱动的类型预测对用户完全不可达。 |
| U4 | MCP 服务器工具覆盖率 ≥ 50% | [❌] | 3 / 40+ ≈ 7.5%。会导致 MCP 用户只能使用 Hajimi 极小比例的真实能力。 |
| U5 | VSCode 插件命令具备真实实现（非 stub） | [⚠️] | 60 条命令中 56 条仅弹出 `showInformationMessage`。会导致用户误以为功能可用，实际无任何操作。 |
| U6 | VSCode 插件包含 `openAdr` / `gotoAdr` 命令 | [❌] | `package.json` 与 `CommandRegistry.ts` 中均未出现。会导致 Phase 5 宣称的 IDE 双命令完全缺失。 |
| U7 | 五层记忆系统具备可视化或状态查询 | [❌] | Session→Auto→Dream→Graph→Cloud 无任何仪表盘或 CLI 状态命令。会导致用户对记忆数据所在层级零可见。 |
| U8 | 终端 UI 具备 Memory Monitor 面板 | [❌] | `src/interface/terminal/src/mod.rs` 99 行，仅 body + status 两栏。会导致无法观测记忆水位或同步状态。 |
| U9 | P2P 同步具备一键/二维码配对流 | [❌] | 无 QR 码、无配对向导，仅散落的手动配置。会导致非技术用户无法启用同步。 |
| U10 | P2P 同步配置集中且文档化 | [⚠️] | 配置散落于 `config.js`、`ice-manager.ts`、`turn-client.ts`、`signaling-server.js`。会导致用户必须理解 WebRTC 协议栈才能上线。 |
| U11 | 全局 `help` / `--help` 命令列出所有工具 | [❌] | terminal UI 与 MCP 均未提供统一帮助。会导致用户无从知晓系统拥有 40+ 工具。 |
| U12 | 错误码具备可查文档或 CLI 查询 | [❌] | `E_SIGNALING_INVALID_MESSAGE`、`E_SIGNALING_PEER_NOT_FOUND` 等仅定义在源码中。会导致收到错误后用户无法自助排障。 |
| U13 | MCP 错误返回具备可操作修复建议 | [❌] | MCP handler 返回 `Error: ${err.message}` 纯文本。会导致 AI Agent 与人类用户都得不到修复路径。 |
| U14 | Web UI 功能与 Terminal UI 对等 | [❌] | Web UI 为 Vite+React 骨架，`App.tsx` 仅 850 字节。会导致 Web 用户几乎无任何生产力功能。 |
| U15 | Terminal UI 具备 Sync Status 面板 | [❌] | `InkApp` 无 sync 状态栏或 pane。会导致 P2P 连接状态对用户不可见。 |
| U16 | IDE 覆盖率达到生产力可用水平 | [⚠️] | 仅 4 条快捷命令有真实映射，且 LSP Client 硬编码 `ws://localhost:8080`。会导致 IDE 插件本质为骨架，无法投入生产。 |

---

## 3. Zombie Function Morgue（已实现但无入口的尸体功能）

### 3.1 `src/intelligence/knowledge/src/search.rs` — ADR 检索尸体
```rust
pub struct AdrSearch<'a> { index: &'a KnowledgeGraphIndex }
impl<'a> AdrSearch<'a> {
    pub fn find_by_id(&self, id: &str) -> Option<&str> { ... }
    pub fn find_by_keyword(&self, keyword: &str) -> Vec<&str> { ... }
}
```
- **文件长度**: 42 行。
- **死因**: 这是一个 `KnowledgeGraphIndex` 的纯包装层。没有任何 CLI 子命令、MCP Tool、Terminal UI Pane 或 VSCode Command 调用它。
- **后果**: 会导致用户辛苦构建的 ADR 知识图谱变成只存在于单元测试里的“图书馆里没有门”的废墟。

### 3.2 `src/engine/tool-system/src/security.rs` — 安全审计尸体
```rust
fn name(&self) -> &str { "security_audit" }
```
- **实现能力**: 扫描 AWS Key (`AKIA...`)、GitHub Token (`ghp_...`)、Stripe Key (`sk_live_...`)、私钥头 (`BEGIN PRIVATE KEY`)、以及 Rust 危险模式 (`todo!()`、`.unwrap()`、`panic!()`)。
- **死因**: 虽然在 `mod.rs` 第 140 行被 `pub mod security;` 导出，但 MCP 服务器 `server.ts` 的 `TOOLS` 数组仅有 3 项（`hajimi_search` / `hajimi_add` / `hajimi_stats`），未包含 `security_audit`。
- **后果**: 会导致 AI Agent 与人类用户都丧失了代码密钥泄露扫描能力，留下显著的安全盲区。

### 3.3 `src/intelligence/typeracing/` — 类型竞速尸体
- **引擎**: `engine.rs`（181 行）实现了基于 LSP 的异步类型预测树，支持 `LspHover`、`LspDefinition`、`LspReferences`、`Heuristic`、`Historical` 五种来源。
- **终端适配器**: `terminal_adapter.rs`（259 行）实现了 `Ctrl+Space` 触发键、`↑↓` 导航、`Enter` 选择、`Esc` 取消的完整交互状态机。
- **死因**: `src/interface/terminal/src/mod.rs`（99 行）的 `InkApp` 完全没有引用 `TerminalAdapter`。`handle()` 方法里只有 `q` / `h` / `i` / `:` 四个键位，没有任何代码处理 `Ctrl+Space`。
- **后果**: 会导致 LSP 驱动的智能类型补全对用户完全不可见，大量 LSP 集成工程投入沦为死代码。

---

## 4. MCP Server Desert（3 个工具 vs 40+ 引擎工具）

`src/interface/mcp-server/server.ts`（165 行）中暴露的 MCP 工具：
```typescript
const TOOLS = [
  { name: "hajimi_search", description: "Search LCR for context chunks", ... },
  { name: "hajimi_add",    description: "Add a context chunk to LCR",   ... },
  { name: "hajimi_stats",  description: "Get LCR statistics",           ... },
];
```

而 `src/engine/tool-system/src/mod.rs` 导出的真实工具集群：
- `directory` / `docs` / `download` / `edit` / `parse` / `patch` / `find` / `fs`
- `git` / `grep` / `multi_edit` / `network` / `shell`
- `analyze` / `build` / `graph` / `test` / `lsp` / `mcp`
- `security` / `image_view` / `js_bundle_analyzer` / `rust_doc_generator`

**覆盖率**: 3 / 40+ ≈ **7.5%**。会导致通过 MCP 与 Hajimi 交互的 AI Agent（如 Claude、Kimi、GPT）只能在一个极小的“LCR 存取沙盒”里工作，无法调用文件系统、Git、LSP、构建、测试、安全审计等核心能力。`ffi-bridge/` 目录虽存在，但明显未被充分利用来桥接这 40+ 工具。

---

## 5. Configuration Hell（P2P 同步与五层记忆黑箱）

### 5.1 P2P 同步：配置的地狱拼图
P2P 同步的实现散落在以下文件中：
- `src/engine/p2p-sync/src/config.js` — STUN 服务器、ICE 策略、心跳间隔
- `src/engine/p2p-sync/src/ice-manager.ts` — ICE candidate 管理
- `src/engine/p2p-sync/src/turn-client.ts` — TURN 中继客户端
- `src/engine/p2p-sync/src/signaling-server.js` — WebSocket 信令服务器
- `src/engine/p2p-sync/src/datachannel-manager.js` — WebRTC DataChannel
- `src/engine/p2p-sync/src/yjs-adapter.ts` — Yjs CRDT 适配
- `src/engine/p2p-sync/src/crdt-engine.ts` — CRDT 引擎

`signaling-server.js` 中硬编码 `ws://localhost:8080`，`config.js` 仅列出 Google 公共 STUN。用户若要部署真实环境，必须同时理解 WebRTC、ICE、TURN、STUN、DataChannel、Yjs CRDT 和 Signaling Server 配置。会导致非技术用户根本无法独立完成 P2P 上线；会导致技术用户也需要在 7 个文件之间玩拼图游戏。

### 5.2 5-Tier Memory：完全不可见的黑箱
五层记忆级联（Session → Auto → Dream → Graph → Cloud）是项目核心卖点，但：
- 没有任何 UI 可视化组件；
- 没有任何 `hajimi memory status` 式的 CLI 命令；
- Terminal UI 中没有 Memory Monitor pane；
- Web UI 中没有 dashboard。

**后果**: 会导致用户对记忆系统处于完全的“盲飞”状态，无法判断关键数据存储在哪一层、是否已同步、是否已压缩、是否已丢失。

---

## 6. Error Message Graveyard（通用错误与无码可查）

### 6.1 P2P 错误码：有定义、无文档
`src/engine/p2p-sync/src/signaling-server.js` 定义了：
```javascript
const E_SIGNALING = {
  INVALID_MESSAGE: 'E_SIGNALING_INVALID_MESSAGE',
  INVALID_JSONRPC: 'E_SIGNALING_INVALID_JSONRPC',
  PEER_NOT_FOUND: 'E_SIGNALING_PEER_NOT_FOUND',
  TIMEOUT: 'E_SIGNALING_TIMEOUT',
  CONNECTION_ERROR: 'E_SIGNALING_CONNECTION_ERROR'
};
```
但没有错误码查询文档，也没有 `hajimi error --code E_SIGNALING_PEER_NOT_FOUND` 式的 CLI 查询命令。**后果**: 会导致用户在遭遇 P2P 连接失败时无法进行自助排障，只能阅读源码。

### 6.2 MCP 错误：无操作建议
`server.ts` 的错误处理：
```typescript
return {
  content: [{ type: "text", text: `Error: ${err instanceof Error ? err.message : "Unknown"}` }],
  isError: true
};
```
**后果**: 会导致 AI Agent 和人类用户收到的都是无上下文的纯文本错误，没有任何“请检查路径是否存在”或“请确认 LSP 服务已启动”之类的可操作修复建议。

### 6.3 无全局帮助系统
`src/engine/tool-system/src/mod.rs` 导出了 40+ 工具，但终端 UI 和 MCP 服务器均未提供统一的 `--help` 或 `help` 命令来枚举它们。**后果**: 会导致用户根本不知道系统里藏了哪些工具，功能发现成本极高。

---

## 7. Consequence Matrix（发现 → 用户影响 → 生产力损失）

| 发现 | 用户影响 | 生产力损失 |
|:---|:---|:---|
| ADR 搜索无入口 | 无法检索已录入的架构决策 | 知识管理投资归零 |
| SecurityAuditTool 未暴露 | AI/人均无法扫描密钥泄露 | 安全风险与返工成本激增 |
| TypeRacing 未接入 TUI | 无智能类型预测可用 | LSP 集成开发投入完全浪费 |
| MCP 仅 3 个工具 | AI Agent 能力被锁死在 LCR | 外部自动化场景无法落地 |
| VSCode 56/60 命令为 stub | 用户点击后无实际效果 | IDE 用户体验崩塌，信任丧失 |
| 无 openAdr / gotoAdr | 无法在 IDE 中浏览 ADR | Phase 5 卖点沦为虚假宣传 |
| 5-Tier Memory 不可见 | 不知数据存于何处 | 调试与数据恢复效率趋近于零 |
| P2P 配置分散且无向导 | 非技术用户无法启用同步 | 协作功能 adoption 率极低 |
| 无错误码文档 | 出现故障只能读源码 | 平均排障时间成倍增加 |
| Web UI 为骨架 | Web 端用户无任何生产力 | 多平台战略名存实亡 |

---

## 8. Minimal Fix Recommendations（低成本 UX 改进）

1. **僵尸功能还魂**: 在 MCP `server.ts` 的 `TOOLS` 数组中立即注册 `security_audit` 和 `hajimi_adr_search`（包装 `AdrSearch`），并添加对应的 `CallToolRequestSchema` handler。成本：1–2 小时。
2. **TypeRacing 接线**: 在 `InkApp::handle()` 中集成 `TerminalAdapter::is_trigger_key` 与 `spawn_predict` 调用。成本：2–4 小时。
3. **MCP 工具扩容**: 通过 `ffi-bridge` 或子进程调用将 `engine/tool-system` 的 `ReadFileTool`、`GrepTool`、`GitStatusTool`、`RunTestsTool` 桥接到 MCP，把工具数从 3 提升到至少 15。成本：1–2 天。
4. **VSCode stub 止血**: 要么移除 `CommandRegistry.ts` 中未实现的 56 条命令的注册，要么为它们提供最小 viable 实现（如调用 Hajimi CLI 子进程）。成本：半天。
5. **错误提示升级**: 在 MCP handler 的 catch 块中根据异常类型返回结构化 JSON，包含 `error_code`、`suggestion`、`docs_link` 字段。成本：2 小时。
6. **Memory 状态 CLI**: 添加 `hajimi memory status` 子命令，打印五层记忆的条目数与最后更新时间。成本：半天。

---

## 9. Overall Risk Rating

**High（高）**

Hajimi V3 的用户可及性与其工程复杂度严重脱节。大量高价值功能被锁在库层、MCP 桥接残缺、IDE 插件充斥空壳命令、P2P 与记忆系统对用户完全不可见。这不仅是“体验差”的问题，而是**功能无法被用户触达**的系统性失败。若不及时填补这些 UX Dead Zone，项目将面临工程投入大量沉没而用户 adoption 趋零的双重危机。
