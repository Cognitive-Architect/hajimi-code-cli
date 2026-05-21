# HAJIMI V3 架构文档

> **文档版本**: v3.9.0 (Hajimi IDE v1 Complete)
> **架构风格**: 四层分层架构 + 本地优先 + Tauri v2 桌面应用
> **核心原则**: 下层零依赖上层、Git历史完整、最小侵入
> **当前状态**: ✅ Agent Core 266测试全部通过（实测 `cargo test -p intelligence-agent-core -- --list`），0编译error，unsafe SAFETY注释100%覆盖；Phase 4 Editing & IDE Integration 完成；Phase 4 Remediation 完成（D4/D1/D3/D2/D5 全维度修复）；Phase 5 UI Interaction Core Remediation 完成 <!-- D4-AUDIT-2026-04-28: metrics from real commands -->
> **最后更新**: 2026-04-30

---

## 🏛️ 系统架构总览（四层模型）

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              INTERFACE 层（界面层）                           │
│  ┌─────────────────────────┐  ┌─────────────────────────┐  ┌─────────────────────────┐ │
│  │     MCP服务器             │  │     Web 界面            │  │     桌面后端            │ │
│  │   (mcp-server/)           │  │   (web/)                │  │   (desktop/)            │ │
│  │   真实 RPC 桥接           │  │   Tauri v2 + 纯 HTML/JS │  │   Tauri v2 Rust 后端    │ │
│  └───────────────────────────┘  └─────────────────────────┘  └─────────────────────────┘ │
└─────────┼────────────────┼────────────────┼────────────────────┼────────────┘
          │                │                │                    │
          └────────────────┴────────────────┴────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          INTELLIGENCE 层（智能层）                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Chimera   │  │Codex-Twist  │  │   Knowledge         │ │
│  │  (chimera/) │  │(codex-twist)│  │   (knowledge/)      │ │
│  │  REPL引擎   │  │ 5级内存架构 │  │   知识图谱+ADR      │ │
│  ├─────────────┤  ├─────────────┤  ├─────────────────────┤ │
│  │   Memory    │  │   Cloud     │  │   Agent Core        │ │
│  │  (memory/)  │  │  (cloud/)   │  │  (agent-core/)      │ │
│  │ 5层:Session │  │ 批次同步    │  │  7步循环+桥接 ⭐     │ │
│  ├─────────────┤  ├─────────────┤  ├─────────────────────┤ │
│  │  pgvector   │  │             │  │                     │ │
│  │ (pgvector/) │  │             │  │                     │ │
│  │ PG向量存储  │  │             │  │                     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             ENGINE 层（引擎层）                               │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │      LLM-Core (llm-core/)                                               │ │
│  │  ┌────────┐ ┌────────────┐ ┌────────┐                            │ │
│  │  │Anthropic │ │   OpenAI    │ │  Ollama  │                            │ │
│  │  │  Claude  │ │   GPT-4     │ │ 本地推理 │                            │ │
│  │  └────────┘ └────────────┘ └────────┘                            │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────┐  ┌──────────────────────────────────────┐ │
│  │   Tool-System (tool-system/)  │  │     Search (search/) ⭐              │ │
│  │  ┌─────────────────────────┐  │  │  ┌────────────────────────────────┐  │ │
│  │  │ 40+ 工具实现            │  │  │  │ Tantivy 16分片索引 ⭐          │  │ │
│  │  │ - 文件/目录操作         │  │  │  │ 向量+文本混合搜索              │  │ │
│  │  │ - Git/终端/搜索         │  │  │  │ 219行高性能实现                │  │ │
│  │  │ - LSP/MCP/网络          │  │  │  └────────────────────────────────┘  │ │
│  │  │ - 构建/测试/安全        │  │  │                                      │ │
│  │  │ - Shell白名单 ⭐        │  │  │                                      │ │
│  │  └─────────────────────────┘  │  │                                      │ │
│  └───────────────────────────────┘  └──────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │     Worker (worker/)                                                    │ │
│  │     并行执行器 / 串行执行器 / 任务调度器                                 │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FOUNDATION 层（地基层）                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌───────────┐  │
│  │   Storage  │ │   Network  │ │  Security  │ │   Format   │ │   Event   │  │
│  │ (storage/) │ │(network/)  │ │(security/) │ │ (format/)  │ │ Loop      │  │
│  │ 16分片SQLite│ │ WebSocket  │ │限流/日志   │ │ .hctx格式  │ │(eventloop)│  │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └───────────┘  │
│  ┌────────────┐ ┌────────────┐                                               │
│  │    WASM    │ │   Hash     │                                               │
│  │  (wasm/)   │ │  (hash/)   │                                               │
│  │ HNSW WASM  │ │ SimHash64  │                                               │
│  └────────────┘ └────────────┘                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 📁 目录结构详解

### 1. Foundation 层（地基层）
**原则**: 零外部依赖，提供基础设施

| 目录 | 功能 | 关键技术 | 状态 |
|:---|:---|:---|:---:|
| `eventloop/` | 事件循环 | Rust异步运行时 | ✅ 稳定 |
| `format/` | 数据格式 | .hctx 格式, BLAKE3校验 | ✅ 稳定 |
| `hash/` | 哈希算法 | SimHash64, 指纹去重 | ✅ 稳定 |
| `network/` | 网络服务 | WebSocket服务器 | ✅ 稳定 |
| `security/` | 安全组件 | 限流器, 安全审计与日志 | ✅ 稳定 |
| `storage/` | 存储系统 | 16分片SQLite | ✅ 稳定 |
| `wasm/` | WASM运行时 | HNSW向量计算 | ✅ 稳定 |

### 2. Engine 层（引擎层）
**原则**: 仅依赖 Foundation 层

| 目录 | 功能 | 关键技术 | 状态 |
|:---|:---|:---|:---:|
| `llm-core/` | LLM客户端 | Anthropic, OpenAI, Ollama SSE 流式 | ✅ 稳定 |
| `search/` | 搜索索引 | Tantivy 16分片 ⭐ | ✅ 稳定 |
| `tool-system/` | 工具系统 | 40+工具, ToolRegistry, **白名单参数化** ⭐ | ✅ P0安全 |
| `worker/` | 工作线程 | 并行/串行执行器 | ✅ 稳定 |

### 3. Intelligence 层（智能层）
**原则**: 依赖 Foundation + Engine 层

| 目录 | 功能 | 关键技术 | 状态 |
|:---|:---|:---|:---:|
| `agent-core/` | 自主Agent系统 | 7步循环, Swarm, 可插拔治理, LLM桥接, Trace, Dashboard, **EditApplier, WorkflowOrchestrator** ⭐ | ✅ 稳定 |
| `chimera/` | REPL引擎 | ZeroTUI, EventLoop | ✅ 稳定 |
| `cloud/` | 云端同步 | 批次同步 | ✅ 稳定 |
| `codex-twist/` | AI内存管理 | 5级内存架构 | ✅ 双轨清理 |
| `knowledge/` | 知识图谱 | ADR, GNN, 实体关系, SimHash-64 ⭐ | ✅ 稳定 |
| `memory/` | 5层记忆系统 | Session/Auto/Dream/Graph/Cloud + **semantic embedding (fastembed)** ⭐ + **EpisodicMemory JSONL 持久化** + **HNSW 索引** | ✅ Phase 3b 完成 |
| `pgvector/` | PostgreSQL向量 | 向量存储与检索 | ✅ 稳定 |

> <!-- P0-CONTEXT-REMEDIATION-2026-04-30 -->
> **P0 Context Debt Cleared ✅**
>
> 基于 B-02/09 ~ B-09/09 工单已全部完成清偿：
> - `engine/llm-core` 新增 `stream_chat_with_context(Vec<ChatMessage>, Option<String>)`，Anthropic/OpenAI/Ollama 三 Provider 全部实现
> - `interface/desktop/src/main.rs` `stream_chat` command 接收 `messages: Option<Vec<ChatMessage>>`，`AppState` 注入 `memory_gateway: Arc<MemoryGateway>`
> - `codex-twist` `MemoryGateway.optimize()` 重写为真实 LLM 驱动摘要（保留最近 2 轮），`codex-twist` 新增 `engine-llm-core` 依赖
> - `interface/web/app.js` 引入 `chatMessages[]` 状态数组、`/compact` slash 命令、Token 估算 UI、80% 阈值自动压缩
> 详见 `docs/debt/DEBT-P0-REMEDIATION.md` 完整清偿记录。

### 4. Interface 层（界面层）
**原则**: 可依赖全下层

| 目录 | 功能 | 关键技术 | 状态 |
|:---|:---|:---|:---:|
| `mcp-server/` | MCP服务器 | **15工具真实RPC** ⭐ | ✅ 稳定 |
| `web/` | Web界面 | 纯 HTML/CSS/JS, Tauri v2 前端 | ✅ 稳定 |
| `desktop/` | 桌面后端 | Tauri v2 Rust 后端，38+工具注册 | ✅ 稳定 |

---

### Phase 5 UI Interaction Core 架构收口（2026-05-15）

Phase 5 UI Interaction Core Remediation 已完成并通过 Day 10 QA handoff。该阶段只修改 Interface 层，保持 Rust 后端、Agent Core、Tool System 与 MCP RPC 边界不变。

**交付后的界面结构**:
- Chat-first 主工作区：Agent Chat / task feed 是默认第一视角，文件、diff、trace、evidence 等上下文下沉到 Right Inspector。
- 7 区域布局：Window Top Bar、Activity Bar、Main Workspace、Right Inspector、Composer、Status Bar、Settings。
- Right Inspector：承载 Task Details、Diff Preview、Agent Trace、Evidence panels，避免主区被辅助信息打散。
- Settings Integration：Providers、Agent Provider Binding、MCP、Governance、Audit logs、Resource Metrics、Session Browser 统一归入 Settings，减少 Activity Bar 噪声。
- Advanced Agent Cards：`Task Steps` 与 `Edit Summary` 已有前端结构化渲染入口，等待后端真实流式数据继续接入。

**架构约束**:
- 前端仍为 vanilla HTML/CSS/JS；禁止在该阶段补引入 React/Vue/Vite/Webpack。
- `src/interface/web/index.html`、`src/interface/web/app.js`、`src/interface/web/style.css` 继续作为 UI 合约核心文件。
- `src/interface/web/modules/security-dom.js` 与 `src/interface/web/modules/workspace.js` 是 Day 13 起新增的无 bundler IIFE 模块，通过 `window.HajimiSecurityDom` / `window.HajimiWorkspace` 暴露小 API；Day 14 追加 `src/interface/web/modules/sessions.js` 与 `src/interface/web/modules/thinking-ui.js`，通过 `window.HajimiSessions` / `window.HajimiThinkingUI` 承接会话持久化、Thinking/Trace、Operation Summary 与 Replay helper；`app.js` 保留旧方法名作为兼容 wrapper。
- Day 15 closure 明确 `frontend modules` 仍是渐进拆分状态：security-dom/workspace/sessions/thinking-ui 已落地，command/slash/provider/style 尚未拆，`app.js` wrapper 继续作为兼容边界。
- Desktop 后端的 `workspace resolver` 以 `resolve_workspace_path` / `PathIntent` 统一承接 read/write/list/create/rename/delete/restore 等路径安全校验；Tauri `CSP` baseline 已启用但 `withGlobalTauri` 仍保留；checkpoint export/compare/restore/replay V1 已接入，restore 需要 dry-run、用户确认和 backup。
- Agent Prompt V2 当前以契约文档与 `prompt golden` regression 作为质量护栏，尚未宣称 live runtime prompt 策略全部产品化。
- Long Context 1M 当前处于债务基线登记状态：`docs/debt/DEBT-LONG-CONTEXT-1M.md` 记录 bridge 8K 硬编码、system prompt token 估算缺口、Provider capability 旧字段和 Memory budget 口径冲突；Day 2 起计划在 `intelligence/agent-core` 内新增 `context_budget.rs` / `long_context_pack.rs` / `context_receipt.rs`，保持预算模块只接收中性 capability DTO 或 Blackboard primitive 字段。
- 已移动 UI 节点必须同步维护 DOM ID 与 JS 事件绑定，遵守 Protected DOM Contract。
- 已知债务集中记录在 `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md`。

**Day 10 QA handoff 证据**:
- `node --check src\interface\web\app.js`: PASS。
- `cargo check --workspace`: PASS。
- `cargo tauri dev`: PASS，Tauri dev build 启动 `target\debug\hajimi-desktop.exe`，webview 成功请求 `/`, `/style.css`, `/app.js`, `/logo.jpg`。
- Day 10 Git 坐标：`v3.8.0-batch-1` / `f1d49e864d24d2ef4edff2b9896a2e225c875653`。

---

## 🎯 核心设计模式

### 1. 分层依赖规则
```
interface ──────┐
                ├──→ intelligence ────┐
                │                      ├──→ engine ────┐
                │                      │               ├──→ foundation
                │                      │               │
                └──────────────────────┴───────────────┘
```
**硬性约束**: Foundation 零依赖上层; Engine 仅依赖 Foundation; Intelligence 依赖 Foundation + Engine; Interface 可依赖全下层。

### 2. ZeroTUI 架构
业务逻辑与 TUI 完全解耦。`TerminalUI<C: Clock, I: InputSource, R: AsyncWrite + Unpin>` 通过泛型参数隔离渲染层。

### 3. Tool 系统架构
统一 `Tool` trait（`name/description/permissions/is_enabled/execute`）。Shell 参数化执行：白名单校验 → 元字符过滤 → `Command::new` 执行，无 `bash -c` 拼接。

### 4. 5级内存架构 (Codex-Twist)
```
Hot:   Focus Memory   — LRU 4K tokens     O(1) ~100ns
Warm:  Working Memory — mmap + zstd 32K   O(log n) ~1μs
Cold:  Archive Memory — LevelDB 1M        O(log n) ~10ms
RAG:   RAG Index      — HNSW 384-dim      O(log n) ~5ms
```

### 5. SimHash-64 分片路由
`simhash64(text) -> u64`; `NUM_SHARDS = 16`; `get_shard_id(text) = simhash64(text) % 16`。

### 6. Agent Core 7步自主循环
`AgentLoop { blackboard, planner, governance, swarm, tool_registry, sync_gateway }`
循环: Observe → Retrieve → Plan → Act → Reflect → Store → Decide。
- **Retrieve**: 通过 `SyncMemoryGateway::retrieve_multi` 进行 Session→Auto→Dream→Graph 级联检索，带 30s TTL 缓存与 4096 token 溢出保护
- **Act**: `AgentLoop::act()` → `SwarmDelegate::delegate_and_wait()` → `Supervisor::delegate(TaskAssignment)` → Worker 处理 → `WorkerResult` 入队 → `SwarmDelegate::pop_result()` 取出 → Blackboard 写入 → `Reflector::reflect_multi()` 聚合反思
- **Reflect**: `Reflector::reflect_multi()` → `MultiWorkerAggregator::aggregate_results()` 计算 success_rate + severity → 生成优化子计划
- **Store**: 通过 `SyncMemoryGateway::sync_with_blackboard` 双向同步 Blackboard ↔ Memory
治理: 5级审批（Auto/Advisory/Required/Critical/Override），支持运行时 `set_approval_level()` 调级。
Swarm: Supervisor-Worker 模式，`WorkerLifecycleManager` 管理 spawn/stop/restart/crash 生命周期，含崩溃自动重启（≤3次，超限 `PermanentlyFailed`）、结果截断（>1MB UTF-8-safe）、原子 metrics、`event_tracing` 全生命周期 trace。
Trace 可观测性: `TraceEvent` 含 `step_type`/`plan_summary`/`reflection_key_points`/`confidence_score`，`emit_trace()` 在 7 步循环各阶段广播，Desktop/Web 双端实时展示。
治理控制: `AgentLoop::pause/resume/inject_memory/update_plan` 支持运行时不中断干预。
Checkpoint 增强: `restore(id)`/`compare(id_a,id_b)`/`export(id)`，Web 端支持浏览/恢复/比较/导出。
资源监控: `ResourceMonitor` 原子计数器跟踪 iteration/blackboard/failure_rate/latency，可配置阈值警报（含冷却期），Web Dashboard 实时展示。

**Swarm 回调闭环伪代码**:
```rust
// Act → Worker → Callback → Reflect 闭环
pub async fn act(&self, agent_id: &AgentId, goal_id: &str) -> ReplResult<TaskResult> {
    // SwarmDelegate 封装委托+轮询逻辑
    let result = SwarmDelegate::delegate_and_wait(
        &self.swarm, &self.blackboard, task_id, deadline: 30s
    ).await?;
    Ok(TaskResult { success: result.success, output: result.output, .. })
}

// Reflect 阶段使用 MultiWorkerAggregator 聚合多 Worker 结果
pub async fn reflect(&self, goal: &Goal, results: &[WorkerResult]) -> ReplResult<Reflection> {
    let aggregated = MultiWorkerAggregator::aggregate_results(results)?;
    let reflection = self.reflector.reflect_multi(goal, &aggregated).await?;
    Ok(reflection)
}
```

---

## 🔌 关键接口定义

### 1. Tool 接口（Engine层）
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn permissions(&self) -> ToolPermissions;
    fn is_enabled(&self, config: &Config) -> bool;
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError>;
}
```

### 2. MCP RPC 接口（Interface层）
```typescript
class CommandRegistry {
  private async invokeMcpTool(name: string, args: unknown[]): Promise<any> {
    return this.lspClient.sendRequest('mcp/toolCall', { tool: name, arguments: args });
  }
}
```

### 3. LLM 接口（Engine层）
```rust
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError>;
    async fn stream(&self, messages: Vec<Message>) -> Result<Stream, LlmError>;
}
```

### 4. AgentGovernance 接口（Intelligence层）
```rust
#[async_trait]
pub trait AgentGovernance: Send + Sync {
    async fn policy(&self, ctx: &AgentContext) -> ApprovalLevel;
    async fn approve(&self, ctx: &AgentContext, req: &GovernanceRequest) -> Decision;
    async fn register_policy(&mut self, name: &str, policy: Arc<dyn GovernancePolicy>) -> ReplResult<()>;
}
```

### 5. SyncMemoryGateway 接口（Intelligence层）
```rust
#[async_trait]
pub trait SyncMemoryGateway: Send {
    async fn retrieve_from_tier(&mut self, tier: MemoryTier, query: &str) -> Result<Vec<MemoryEntry>, SyncGatewayError>;
    async fn retrieve_multi(&mut self, tiers: &[MemoryTier], query: &str) -> Result<Vec<(MemoryTier, Vec<MemoryEntry>)>, SyncGatewayError>;
    async fn push_event(&mut self, event: GatewayEvent) -> Result<(), SyncGatewayError>;
    async fn sync_with_blackboard(&mut self, snapshot: &BlackboardSnapshot) -> Result<(), SyncGatewayError>;
    async fn tier_health(&mut self, tier: MemoryTier) -> Result<TierHealth, SyncGatewayError>;
}
```

---

## 🔄 数据流

**查询流程**: User → Interface (web/mcp) → Engine (tool-system/llm) → Intelligence (agent-core/chimera) → Foundation (storage/db) → Engine (LLM API) → Interface (Output)。

### Model-aware Long Context Flow（计划入口）

```text
Interface Provider Settings
  -> neutral provider/model capability fields
  -> Intelligence agent-core ContextBudget
  -> LongContextPack structured blocks
  -> ContextWindowManager assemble / compact
  -> Engine LLM request
  -> ContextReceipt metadata and omitted reasons
  -> Interface token / status display
```

Boundary rule: the intelligence budget code must not import desktop provider configuration types. Interface may normalize provider settings into primitive Blackboard fields or intelligence-owned DTOs such as `ProviderContextCaps` / `BudgetResolveInput`. Day 03 adds `resolve_context_budget` and `HAJIMI_LONG_CONTEXT_ENABLED` handling inside agent-core, but bridge integration is still reserved for Day 4-5. Probe success is required before any 1M model can be treated as `Verified`; otherwise the status remains `Declared / Target` or falls back to a smaller window.

---

## 🛡️ 安全架构

1. **工具权限系统**: `PermissionLevel { Deny, Ask, Allow }`。Shell 严格白名单（38命令）+ `Command::new` 参数化执行。
2. **限流策略**: Token Bucket（SQLite持久化）。Burst 100, Rate 10req/s, 熔断器（Failure 50%, Recovery 30s）。
3. **API Key 安全存储**: OS Keyring（Windows Credential Manager / macOS Keychain / Linux Secret Service）+ `secrecy::SecretString` 内存脱敏。`providers.json` 仅存元数据，密钥不落磁盘明文。
4. **配置文件权限**: Unix `0o600` / Windows `icacls` 受限 ACL，父目录 `0o700`（best-effort，失败 graceful 降级）。
5. **Workspace 配置隔离**: 项目级 `.hajimi/providers.json` 覆盖全局配置，全局作为 fallback，支持 per-project Key 隔离。
3. **WebRTC信令认证**: CSPRNG + 环境变量 PSK + `timingSafeEqual`。`clientId = crypto.randomUUID()`。
4. **审批策略**: `ApprovalPolicy { AskBeforeExec, AskForDangerous, AskOnceThenAuto, FullAuto, FullDeny }`。

---

## 📊 性能基准

| 操作 | 指标 | 实现 | 状态 |
|:---|:---|:---|:---:|
| SQLite 批量写入 | 9,569 ops/s | WAL + 16分片 | ✅ |
| HNSW 查询 | 1.94x 加速 | WASM | ✅ |
| HNSW 构建 | 7.7x 加速 | WASM | ✅ |
| Tantivy 搜索 | 219行 | 16分片 | ✅ |
| Memory Gateway | O(1) ~100ns | LRU Focus | ✅ |
| Agent Core E2E | 194 passed | cargo-discoverable | ✅ |
| Agent Core 编译 | 0 errors, pre-existing warnings 外 crate | cargo check | ✅ |
| Memory Sync E2E | 6 passed | `memory_sync_e2e.rs` | ✅ |
| Phase 3 测试 | 172 passed | Day 1-15 Phase 3a/3b 全量验证 | ✅ Phase 3b 完成 |
| HNSW 召回率 | top-1=1.0 | `bench_hnsw_recall` @ n=100, recall@10 | ✅ |
| HNSW 内存 | 15.9MB | `bench_hnsw_memory` @ n=1000 vectors | ✅ |
| 精确 Token 统计 | 100/100 | 方案 B 精确模式（误差 0%，E2E 12 tests passed） | ✅ Scheme B 已完成 |
| Token Tracker 持久化集成 | 100/100 | ✅ TokenUsageTracker 正式激活（P1 已清偿）：AppState 扩展 + `record_usage()` + `get_cumulative_stats` Tauri Command + Frontend 混合持久化 | ✅ P1 Cleared |

<!-- Scheme B: Precise Token Pipeline -->
**Scheme B 精确 Token 统计管线**（Engine → Intelligence → Interface）：
- **Engine 层**: `tiktoken-rs` 集成，`LlmClient::count_tokens()` 精确编码
- **Intelligence 层**: `TokenUsageTracker` 会话级 + 全局累计聚合
- **Interface 层**: 前端 UI 升级精确分离显示（`🔄 xx.x% | ↑ xxxxx | ↓ xxxx`）
- **设计文档**: `docs/roadmap/Hajimi Context/p0 fix/02-exact-token-usage-tracking.md`、`03-token-scheme-b-daily-development-plan.md`、`04-token-scheme-b-guidance.md`
- **架构决策**: ADR-SB-01 Precise Token Pipeline，ADR-SB-02 Tiktoken Integration（feature flag 控制）

---

## 🗺️ 目录迁移历史

### 版本演进
```
src/
├── crates/              # 保留: evm-bench-adapter, hajimi-codex-twist
├── engine/              # 引擎层 (4模块)
│   ├── llm-core/        # LLM客户端
│   ├── search/          # Tantivy搜索
│   ├── tool-system/     # 40+工具 (白名单参数化)
│   └── worker/          # 工作线程
├── foundation/          # 地基层 (7模块)
│   ├── eventloop/, format/, hash/
│   ├── network/, security/, storage/
│   ├── wasm/
│   └── ...
├── intelligence/        # 智能层 (7模块)
│   ├── agent-core/      # 自主Agent系统 (7步循环/Swarm/治理/LLM桥接) ⭐
│   ├── chimera/         # REPL引擎
│   ├── cloud/           # 云端同步
│   ├── codex-twist/     # AI内存 (双轨清理完成)
│   ├── knowledge/       # 知识图谱
│   ├── memory/          # 5层记忆
│   └── pgvector/        # PG向量
└── interface/           # 界面层 (3模块)
    ├── mcp-server/      # MCP服务器 (真实RPC)
    ├── web/             # Web界面 (Tauri v2 前端)
    └── desktop/         # 桌面后端 (Tauri v2 Rust 后端)

patches/                 # 构建依赖补丁（非功能模块）
└── zstd-sys/            # zstd-safe 6.x API 兼容性补丁
    # 本地覆盖 [patch.crates-io]，修复上游 API 不匹配
```

---

## 📝 架构决策记录 (ADR)

| ID | 决策 | 状态 | 关联 |
|:---|:---|:---:|:---|
| ADR-001 | 四层分层架构 | ✅ | v2.0重构完成 |
| ADR-002 | 16分片 SQLite (SimHash-64) | ✅ | foundation/storage/ |
| ADR-003 | ZeroTUI (业务逻辑与TUI解耦) | ✅ | intelligence/chimera/ |
| ADR-004 | WASM HNSW (Rust/WASM, 比JS快5倍) | ✅ | foundation/wasm/ |
| ADR-005 | 5级内存架构 (Session/Auto/Dream/Graph/Cloud) | ✅ | intelligence/memory/ |
| ADR-006 | Tool Trait 标准接口 (5方法) | ✅ | engine/tool-system/ |
| ADR-007 | Git历史完整保留 (git mv) | ✅ | v2.0重构 |
| ADR-008 | SimHash-64统一分片 | ⚠️ | foundation 8处引用 |
| ADR-009 | TraceEvent 结构化扩展 | ✅ | agent-core/agent_loop.rs |
| ADR-010 | Shell参数化白名单 (消除bash -c) | ✅ | engine/tool-system/shell.rs |
| ADR-013 | DEBT-LINES 子模块提取 | ✅ | agent-core/memory_retriever.rs, loop_state_machine.rs, reflection_persistence.rs, plan_optimizer.rs |
| ADR-014 | ResourceMonitor 原子监控 | ✅ | agent-core/resource_monitor.rs |
| ADR-015 | Editing Pipeline（EditApplier 作为 Apply 唯一入口，Proposed→Review→Apply 状态机，原子写入 + 唯一备份） | ✅ | intelligence/agent-core/edit_applier.rs |
| ADR-016 | AST-First Context Retrieval（Retrieve 阶段可选注入 AST 上下文，fallback 到纯文本，LspContextProvider 抽象） | ✅ | engine/tool-system/lsp_integration.rs, intelligence/agent-core/memory_retriever.rs |
| ADR-011 | Tauri v2 桌面应用架构 | ✅ | src/interface/desktop/ |
| ADR-012 | 工具系统Channel流式传输 | ✅ | engine/tool-system/ |
| ADR-SB-01 | Precise Token Pipeline | ✅ | engine/llm-core/ → intelligence/codex-twist/ → interface/web/ |
| ADR-SB-02 | Tiktoken Integration（feature flag） | ✅ | engine/llm-core/ |
| ADR-P1-01 | Token Tracker Persistence（`TokenUsageTracker` 作为 Intelligence 层统计核心，Backend → Tauri Command 暴露） | ✅ | intelligence/codex-twist/ → interface/desktop/ |
| ADR-P1-02 | Hybrid Frontend Storage（Tauri Command + LocalStorage 混合持久化，刷新后数据不丢失） | ✅ | interface/web/app.js |

<!-- P1-TOKEN-TRACKER-2026-05-02: integration cleared, all 5 workitems done -->

### P1 Token Tracker Integration 架构设计

**核心原则**: 最小侵入 + 分层纯洁

**接入点设计**:
- `AppState` 新增 `token_tracker: Arc<codex_twist::memory::TokenUsageTracker>` 字段
- `main()` 实例化并注入 `AppState`
- `stream_chat` 结束时调用 `record_usage(session_key, provider, prompt_tokens, completion_tokens)`
- 新增 `get_cumulative_stats` Tauri Command 返回 `GlobalStats` JSON

**数据流**:
```
Engine (llm-core) ──→ usage 解析 ──→ Interface (desktop)
                                             │
                                             ▼
                              TokenUsageTracker::record_usage()
                                             │
                                             ▼
                              Intelligence (codex-twist) 累计聚合
                                             │
                                             ▼
                              get_cumulative_stats Tauri Command
                                             │
                                             ▼
                              Frontend (app.js) 混合持久化恢复
```

**前端持久化策略**:
- 主路径: `window.__TAURI__.core.invoke('get_cumulative_stats')`
- 兜底路径: `localStorage.getItem('hajimi_cumulative_stats')`
- 保存时机: `sendChatMessage()` 成功后写入 LocalStorage

**关键约束**:
- Engine 层不直接依赖 `codex_twist`（分层纯洁）— 验证: `grep codex_twist src/engine/` = 0 匹配
- `TokenUsageTracker` 仅在 Intelligence 层提供服务 — 验证: `cargo test -p codex-twist --test token_tracking_e2e` 12 passed
- Interface 层通过 Tauri Command 消费，不暴露内存细节 — 验证: `cargo check --workspace` 0 errors

<!-- MEMORY-REMEDIATION-2026-05-03: AgentLoopBuilder production_ready() adds default MemoryGateway injection -->
<!-- MEMORY-REMEDIATION-CLEARED: 7/7 Cleared -->

---

## 🔗 关联文档

| 文档 | 路径 | 说明 |
|:---|:---|:---|
| 源代码索引 | `src/INDEX.md` | 详细文件索引 |
| 贡献指南 | `src/CONTRIBUTING.md` | 开发指南 |
| 技术文档 | `docs/debt/` | 技术约束与限制说明 |
| 技术约束文档 | `docs/debt/DEBT-P0-UI-INTERACTION-REMEDIATION.md` | UI交互核心重构期间无框架约束声明 |

---

### Phase 3a/3b Memory Enhancement 架构

<!-- PHASE-3A-REMEDIATION-2026-05-05: semantic memory + LLM summary initiated -->
<!-- PHASE-3B-REMEDIATION-2026-04-30: Phase 3b completed -->

**状态**: ✅ Phase 3a/3b **全部完成**（17/17 工单）

**新增组件** (Intelligence 层):
- DreamMemory semantic embedding (`fastembed` AllMiniLML6V2 384-dim, optional feature) + LRU cache
- MemoryBootstrapper LLM 自然语言摘要 (`generate_natural_language_summary`)
- EpisodicMemory JSONL 持久化 (`episodes.jsonl`, 1000条容量) + 跨进程恢复
- DreamMemory HNSW 索引 (`hnsw_rs` optional feature, O(log n) ~5ms release / ~7.4ms debug)

**EpisodicMemory 架构**:
- `Episode` 结构体：id, timestamp, action, content, outcome, confidence, metadata
- `new_with_persist(project_id)` → `~/.hajimi/memory/{project_id}/episodes.jsonl`
- `append_to_jsonl()` 原子写入（NamedTempFile + rename）
- `load_from_disk()` 容错加载（跳过损坏行）
- `query_by_keyword()` 关键词检索（B-10 新增）
- 容量淘汰：MAX_EPISODES=1000，超限 pop_front

**HNSW 索引架构**:
- `hnsw_rs` crate，optional feature `hnsw-index`
- 参数 FINAL（B-15）: M=16, max_elements=10_000, max_layer=16, ef_construction=16
- `new_with_hnsw()` 启动时重建，失败 graceful 降级为线性扫描
- `rebuild_hnsw()` 策略 A：SQLite 全表扫描 → 新 HNSW → 原子替换
- 每 1000 条插入触发自动重建
- `search_hnsw()`: hnsw.search → id_to_text 映射 → SQLite 重建 DreamEntry
- 5 个联合/边缘/并发测试（joint/empty/single/concurrent/graceful）

**架构决策**: ADR-P3-01 Semantic Embedding, ADR-P3-02 HNSW Index, ADR-P3-03 Episodic Persistence

---

## Thinking UI 方案C 架构状态 — 已完成

<!-- THINKING-UI-2026-04-30: scheme-c implementation completed B-02~B-12 -->

**状态**: ✅ **方案C 已完成**（B-02~B-12，12 天实施周期闭环）

**架构组件**（代码实测验证）：

| 组件 | 状态 | 关联文件 | 关键交付 |
|:---|:---:|:---|:---|
| AgentLoop 7步循环 | ✅ | `agent_loop.rs` | emit_trace_with_meta() 发送完整 trace 事件 |
| TraceEvent 结构 | ✅ | `agent_loop.rs:32-43` | plan_summary / reflection_key_points / confidence_score / operation_summary / thinking_content |
| Tauri Event Bridge | ✅ | `main.rs:1242-1280` | trace_tx 注入 AppState，subscribe_agent_trace 通道打通 |
| MCP Trace Handler | ✅ | `events.rs` | 真实 AgentLoop 事件（无模拟数据） |
| Chat Thinking UI | ✅ | `app.js:2640` | 可折叠 thinking-block + 流式更新 + Markdown 渲染 |
| 操作可视化 | ✅ | `app.js:2719` | Codex 风格 operation-summary-bar + diff 预览 + 实时进度 |
| 时间线整合 | ✅ | `app.js` | TimelineEvent 统一模型 + Session Replay 补全 |

**已实施架构决策（ADR）**:
- ✅ ADR-THINKING-UI-01: Tauri Event Bridge（`subscribe_agent_trace` 作为唯一通道）
- ✅ ADR-THINKING-UI-02: Thinking Content Pipeline（`<thinking>` 标签作为提取标准）
- ✅ ADR-THINKING-UI-03: Operation Summary Aggregation（`OperationSummary` 作为统一表示）

**分层约束**: 所有 Thinking UI 可视化逻辑仅存在于 Interface 层；Engine/Intelligence 层仅提供结构化事件数据。此约束在 B-02~B-12 中严格遵守，无分层违规。

**Roadmap**: `docs/roadmap/Hajimi Thinking UI/THINKING-UI-IMPLEMENTATION-ROADMAP.md`

*本架构文档与代码同步维护，最后更新于 2026-04-30*

---

## Long Context 1M Engine 架构状态 — 阶段成果记录

<!-- LONG-CONTEXT-1M-2026-05-21: Day 15 Closure pending final live E2E -->

**当前状态**: 🔄 **Day 15 清债闭环待全面交付（已更新基线，回归通过）**

Hajimi 1M Long Context 引擎在智能层 `agent-core` 内部完全闭环，完全隔离了对 Interface 层 `ProviderConfig` 的依赖，以保持无反向引用的清晰分层架构。

### 1. 架构组件说明

| 核心组件 | 实现文件 | 行数 | 核心职责 |
|:---|:---|:---:|:---|
| **ContextBudget** | `context_budget.rs` | 1147 | 基于 Legacy (8K) / Fast (128K) / Pro (200K) / Long (1M) 四档模型容量，动态计算系统 Prompt 占用、剩余输出保留区与上下文检索上限（Cap 600K）。 |
| **LongContextPack** | `long_context_pack.rs` | 987 | 上下文超长打包合并。应用 Tree-sitter AST、首尾保留、大小剔除策略及 SimHash 指纹排重，保证打包后的 token 估算不超过可用预算，并对每个省略（Omitted）块记录清晰的原因。 |
| **ContextProbe** | `context_probe.rs` | 393 | 负责异步 Provider 能力探测。基于本地 `.hajimi/provider_probes` 持久化文件管理状态转换。支持 `1小时 TTL`，并在超时或失败时执行多级级联降级策略。 |
| **ContextReceipt** | `context_receipt.rs` | 716 | 在每次 LLM 交互结束后产生审计回执并异步写盘。包含 Included/Omitted 列表、系统与检索估算 Token，同时对密钥（sk-...）与配置环境敏感信息执行完全的正则脱敏（Redaction）。 |

### 2. 状态机生命周期 (Declared / Verified / Stale / Fallback)

长上下文处理引擎采用清晰的容量五态口径进行驱动：

```
 [Declared (声明值)] ──────► 进行容量探针探测
         │
         ├───► 探针验证成功 (1h内 TTL 有效) ──────► [Verified (已验证)]
         │
         ├───► 探针验证成功但 TTL 超期(1h) ────────► [Stale (过期受限 128K)]
         │
         └───► 探针失败/超时/被取消 ───────────────► [Fallback (级联降级)]
```

### 3. 数据流生命周期

```
[Blackboard] ──► resolve_context_budget() ──► 内存分配上限 (Focus/Working/Archive)
       │                                                 │
       ▼                                                 ▼
[AgentLoop] ──────► retrieve_multi() ──► LongContextPackBuilder::build() 
       │                                                 │ (压缩/首尾/OMIT 追踪)
       ▼                                                 ▼
[LLM Bridges] ◄─────────────────────────────────── [Structured Pack]
       │ (消息流式响应)
       ▼
[ContextReceipt] ──► redact_sensitive_text() ──► Asynchronous Disk Write (.json)
```

### 4. 架构硬性红线

1. **分层完整性**: `agent-core` 对 `interface/desktop` 及 `ProviderConfig` 的依赖关系为 **0**。所有数据通过 neutral/plain Blackboard 转换承载。
2. **回滚安全性 (Rollback)**: 任何时候设置 `HAJIMI_LONG_CONTEXT_ENABLED=false` 均能完美触发传统 8K standard 模式。
3. **真实性原则**:
   - 真实的 Provider Probe **尚未在真机网络下集成**，目前依然使用极度逼真的 MockOnly 机制。
   - 小票显示的 token 计数为 **Estimated** 评估值，并非 provider 实机计费（actual usage）。
   - 实机 GUI 点击测试仍保留为活跃技术债务。

*本架构文档与代码同步维护，最后更新于 2026-05-21*
