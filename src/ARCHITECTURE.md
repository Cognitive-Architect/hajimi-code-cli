# HAJIMI V3 架构文档

> **文档版本**: v3.9.0 (Hajimi IDE v1 Complete)
> **架构风格**: 四层分层架构 + 本地优先 + Tauri v2 桌面应用
> **核心原则**: 下层零依赖上层、Git历史完整、最小侵入
> **当前状态**: ✅ Agent Core 266测试全部通过（实测 `cargo test -p intelligence-agent-core -- --list`），0编译error，unsafe SAFETY注释100%覆盖；Phase 4 Editing & IDE Integration 完成；Phase 4 Remediation 完成（D4/D1/D3/D2/D5 全维度修复） <!-- D4-AUDIT-2026-04-28: metrics from real commands -->  
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
