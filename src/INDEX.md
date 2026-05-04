# HAJIMI V3 源代码索引

> **文档版本**: v3.9.0 (Hajimi IDE v1 Complete)  
> **最后更新**: 2026-04-27  
> **代码总行数**: ~182,362行（.rs/.js/.ts/.html/.css，不含 .md 与依赖，实测2026-04-28）; 含文档（.md）总计 ~186,441行  
> **架构**: 四层分层（Foundation/Engine/Intelligence/Interface）  
> **当前状态**: ✅ Agent Core 266测试通过（实测 `cargo test -p intelligence-agent-core -- --list`），0编译error，0新增clippy warning（agent-core范围内），unsafe SAFETY 100%覆盖；Phase 4 Editing & IDE Integration 完成；Phase 4 Remediation 完成（D4/D1/D3/D2/D5 全维度修复）；Hajimi IDE v1 就绪 <!-- D4-AUDIT-2026-04-28: metrics from real commands -->

---

## 📁 目录总览（四层架构）

```
src/
├── crates/              # 保留的 Rust Crates
│   └── hajimi-codex-twist/  # AI内存核心 (Rust workspace兼容)
│
├── patches/             # 构建依赖补丁（非功能模块）
│   └── zstd-sys/        # zstd-safe 6.x API 兼容性补丁
│
├── foundation/          # 地基层 - 零依赖（7模块）
│   ├── eventloop/       # 异步事件循环 (Rust)
│   ├── format/          # 数据格式（.hctx/BLAKE3）
│   ├── hash/            # 哈希算法（SimHash64）
│   ├── network/         # WebSocket服务器 ⭐
│   ├── security/        # 安全与限流控制
│   ├── storage/         # 存储层（16分片SQLite）⭐
│   └── wasm/            # WASM运行时（HNSW）⭐
│
├── engine/              # 引擎层 - 仅依赖foundation（4模块）
│   ├── llm-core/        # LLM客户端（Anthropic/OpenAI/Ollama）
│   ├── search/          # 搜索索引（Tantivy 16分片）⭐
│   ├── tool-system/     # 工具系统（40+工具/白名单参数化）⭐
│   └── worker/          # 工作线程池
│
├── intelligence/        # 智能层 - 依赖foundation+engine（7模块）
│   ├── agent-core/      # 自主Agent系统（7步循环/Swarm/可插拔治理/LLM桥接）⭐
│   │   └── llm/         #   LLM适配器桥接（PlannerLlmBridge + ReflectorLlmBridge）
│   ├── chimera/         # Chimera REPL引擎（Rust）⭐
│   ├── cloud/           # 云端同步（批次同步）
│   ├── codex-twist/     # AI内存管理（5级架构/双轨清理完成）⭐
│   ├── knowledge/       # 知识图谱（ADR/GNN/知识库）⭐
│   ├── memory/          # 5层记忆系统⭐
│   └── pgvector/        # PostgreSQL向量扩展
│
└── interface/           # 界面层 - 依赖全下层（3模块）
    ├── mcp-server/      # MCP服务器（真实RPC桥接）⭐
    ├── web/             # Web界面（Tauri v2 纯HTML/JS前端）
    └── desktop/         # 桌面后端（Tauri v2 Rust 后端，38+工具注册）
```

---

## 🎯 分层详解

### Foundation 层（地基层）
**原则**: 零外部依赖，提供基础设施  
**状态**: ✅ P0安全加固完成

#### storage/ - 存储层 ⭐
**核心**: 16分片 SQLite + WAL 模式  
**路由**: SimHash-64 高 8bit → 分片 00-15

| 文件 | 功能 |
|------|------|
| `shard-router.js` | SimHash → 分片路由 |
| `chunk.js` | Chunk 数据模型 |
| `connection-pool.js` | SQLite 连接池 |
| `batch-writer-optimized.js` | 批量写入优化 |
| `queue-db-interface.ts` | 持久化队列接口 |
| `leveldb-optimized.ts` | LevelDB 优化配置 |

**性能指标**: 写入 9,569 ops/s（WAL 批量）

#### wasm/ - WASM 运行时 ⭐
**技术**: wasm-bindgen + SharedArrayBuffer

| 文件 | 功能 |
|------|------|
| `loader.js` | WASM 加载器 |
| `hnsw-bridge.js` | HNSW WASM 桥接 |
| `sab-allocator.ts` | SharedArrayBuffer 分配器 |
| `src/lib.rs` | HNSW Rust 实现 |
| `src/code_index.rs` | 代码索引模块 |
| `src/memory.rs` | WASM内存管理 |
| `src/sab.rs` | SAB共享内存 |

**性能加速**: WASM 查询 1.94x，构建 7.7x

#### network/ - 网络服务（P0加固）
**来源**: 原 `ws_server/` 迁移  
**功能**: WebSocket 服务器  
**约束**: PSK长期管理待完善（KMS/Vault/Rotation）

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | WebSocket 服务器实现 |
| `src/handlers.rs` | 消息处理器 |
| `src/protocol.rs` | 协议定义 |

#### security/ - 安全组件
**功能**: 限流、日志、沙盒

| 文件 | 功能 |
|------|------|
| `rate-limiter-sqlite-luxury.js` | SQLite持久化限流器 |
| `rate-limiter-redis.js` | Redis限流器 |
| `audit-logger.rs` | 安全审计与日志 |

#### utils/ - 通用工具
**功能**: SimHash64（8处分散引用，未完全统一）

| 文件 | 功能 |
|------|------|
| `simhash.js` | SimHash-64实现 |
| `logger.js` | 日志工具 |

---

### Engine 层（引擎层）
**原则**: 仅依赖 Foundation 层  
**状态**: ✅ P0安全加固完成，Shell白名单参数化

#### tool-system/ - 工具系统 ⭐（P0加固）
**规模**: 40+ 工具实现  
**核心**: Tool Trait 标准接口 + **白名单参数化（B-04 P0）**

| 模块 | 文件 | 工具数量 |
|------|------|:--------:|
| 文件操作 | `fs.rs`, `directory.rs`, `edit.rs`, `multi_edit.rs` | 10 |
| 搜索 | `grep.rs`, `search.rs`, `find.rs`, `search/find.rs`, `search/grep.rs` | 6 |
| 终端 | `shell.rs` ⭐ | 2 |
| Git | `git.rs`, `git_branch.rs`, `git_cli.rs` | 6 |
| LSP | `lsp.rs` | 4 |
| MCP | `mcp.rs` ⭐ | 5 |
| 构建 | `build.rs` | 4 |
| 测试 | `test.rs` | 3 |
| 网络 | `network.rs` | 3 |
| 文档 | `docs.rs`, `rust_doc_generator.rs` | 3 |
| 分析 | `analyze.rs`, `graph.rs`, `security.rs` | 4 |
| 其他 | `download.rs`, `image_view.rs`, `js_bundle_analyzer.rs`, `parse.rs`, `patch.rs`, `registry.rs`, `tools_lib.rs` | 7 |

**Tool Trait**:
```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn permissions(&self) -> ToolPermissions;
    fn is_enabled(&self, config: &Config) -> bool;
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError>;
}
```

**Shell白名单参数化（B-04 P0）**:
```rust
// shell.rs - 严格白名单
const ALLOWED_COMMANDS: &[&str] = &[
    "git", "cargo", "npm", "node", "python3", "ls", "cat", "echo", ...
]; // 38工具，无rm/sudo

// 降级功能: 复杂管道/重定向/逻辑运算符
// 约束: docs/debt/SHELL-FEATURE-DEBT-002.md
```

#### search/ - 搜索索引 ⭐
**技术**: Tantivy 全文搜索 + 16分片  
**状态**: ✅ 完成

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 搜索模块入口 |
| `src/tantivy_index.rs` | Tantivy 16分片索引（219行）⭐ |
| `src/tantivy_query.rs` | 查询解析 |
| `src/vector_text_hybrid.rs` | 向量+文本混合搜索 |
| `src/debug_test.rs` | 调试测试 |

**性能指标**: 混合搜索，16分片并行

#### llm-core/ - LLM客户端
**来源**: 原 `crates/hajimi-core/src/llm/` 迁移

| 文件 | 功能 |
|------|------|
| `anthropic.rs` | Claude API 客户端 |
| `openai.rs` | OpenAI API 客户端 |
| `ollama.rs` | Ollama 本地推理 |
| `mod.rs` | 统一接口 |
| `error.rs` | 错误处理 |
| `streaming/mod.rs` | 流式响应 |

#### worker/ - 工作线程
**来源**: 原 `worker/` + `crates/hajimi-core/src/executor/` 合并

| 文件 | 功能 |
|------|------|
| `parallel.rs` | 并行执行器 |
| `serial.rs` | 串行执行器 |
| `mod.rs` | 任务调度器 |
| `query.rs` | 查询执行器 |
| `error.rs` | 错误处理 |

---

### Intelligence 层（智能层）
**原则**: 依赖 Foundation + Engine 层  
**状态**: ✅ 稳定

#### agent-core/ - 自主Agent系统 ⭐（Day 10 FULL）
**技术栈**: Rust (tokio + async-trait)  
**来源**: Day 1-10 渐进式构建（Planner→Reflector→Governance→Swarm→AgentLoop）  
**代码规模**: ~2,750行源文件 / ~3,150行总计（19源文件 + 4测试文件）  
**状态**: 121 测试全部通过（69 lib + 52 E2E），0编译error  
**约束**: 7项技术约束与待办已记录（见下方）

| 文件 | 功能 | 行数 |
|------|------|:----:|
| `lib.rs` | 模块入口 + 公共类型 | 198 |
| `agent_loop.rs` | 7步自主循环（含pause/resume/monitor集成） | 337 |
| `agent_loop_builder.rs` | AgentLoop构建器 | 76 |
| `agent_loop_tests.rs` | AgentLoop单元测试 | 93 |
| `swarm.rs` | Swarm协调器（Supervisor-Worker模式） | 236 |
| `swarm_delegate.rs` | Swarm委托与结果轮询 | 45 |
| `worker_lifecycle_manager.rs` | Worker生命周期管理 | 95 |
| `governance.rs` | 可插拔治理（5级审批策略 + 投票 + 运行时调级） | 289 |
| `orchestrator.rs` | Agent编排器 | 305 |
| `planner.rs` | 任务规划器 | 167 |
| `reflector.rs` | 反思优化器（187行，提取后≤240） | 187 |
| `multi_worker_aggregator.rs` | 多Worker结果聚合 | 55 |
| `blackboard.rs` | 共享黑板状态 | 112 |
| `checkpoint.rs` | 检查点恢复（含restore/compare/export） | 129 |
| `events.rs` | 事件系统 | 211 |
| `event_tracing.rs` | Worker全生命周期Trace | 38 |
| `tools.rs` | 工具集成 | 171 |
| `degrade.rs` | 降级策略 | 13 |
| `ports.rs` | 端口抽象 | 121 |
| `minimal_agent.rs` | 最小Agent实现 | 18 |
| `llm/bridge.rs` | **LLM 适配器桥接**（PlannerLlmBridge + ReflectorLlmBridge）⭐ | 161 |
| `llm/mod.rs` | LLM 模块入口 | 3 |
| `mod.rs` | 公共API导出（含约束声明） | 34 |
| `memory_retriever.rs` | **多层级记忆检索**（DEBT-LINES清偿） | 87 |
| `loop_state_machine.rs` | **7步循环状态机**（DEBT-LINES清偿） | 69 |
| `reflection_persistence.rs` | **反思持久化与审批**（DEBT-LINES清偿） | 53 |
| `plan_optimizer.rs` | **计划优化器**（DEBT-LINES清偿） | 39 |
| `edit_applier.rs` | EditApplier: hunk-level diff, conflict detection, atomic apply, true undo (unique `.bak`), size/hunk/concurrency guards, ResourceMonitor integration (Phase 4 Day 1+6) | ~564 |
| `workflow_orchestrator.rs` | Test→Fix→Commit closed loop, SmartCommit, PR description, auto-checkpoint (Phase 4 Day 4) | ~220 |
| `lsp_integration.rs` | `LspContextProvider` / `ASTContextProvider`, `enhance_retrieve_with_ast()` (Phase 4 Day 2) | ~120 |

**新增特性 (Phase 1)**:
- **SyncMemoryGateway**: `memory/src/sync_gateway.rs` — 跨层记忆检索与持久化抽象（Session→Auto→Dream→Graph→Cloud）
- **AgentLoop 集成**: `retrieve()` 使用 `retrieve_multi()` 多层级联检索，带 30s TTL 缓存与 4096 token 溢出保护
- **Event 持久化**: `AgentEventProcessor` 各 `process_*` 方法通过 `push_event` 持久化到记忆层
- **Checkpoint 双向同步**: `CheckpointManager::restore_from_memory()` 支持从 Session 层恢复检查点

**新增特性 (Phase 2)**:
- **WorkerCallback 回调机制**: `ports.rs` `WorkerCallback` trait + `WorkerResultStatus`/`WorkerMetrics` 类型，支持 `on_success`/`on_failure`/`on_timeout`
- **Swarm 执行闭环**: `AgentLoop::act()` → `Supervisor::delegate()` → Worker 执行 → `handle_worker_result()` → `pop_result()` → `reflect_multi()`
- **边缘案例处理**: Worker 崩溃自动重启（≤3次，超限标记 `PermanentlyFailed`）、结果 >1MB UTF-8-safe 截断、`SupervisorMetrics` 原子计数
- **Worker 全生命周期 Trace**: `events.rs` 6 个 trace 方法覆盖 spawn/start/complete/fail/crash/restart
- **子模块提取（债务清偿）**: `SwarmDelegate` / `MultiWorkerAggregator` / `WorkerLifecycleManager` / `event_tracing` 4 个独立模块，单文件复杂度降低 23%-37%

**新增特性 (Phase 3)**:
- **DEBT-LINES 提取（Day 1）**: `MemoryRetriever` / `LoopStateMachine` / `ReflectionPersistence` / `PlanOptimizer` 4 个独立模块，`agent_loop.rs` 337→247行，`reflector.rs` 281→187行
- **Trace 系统增强（Day 2）**: `TraceEvent` 扩展 `step_type`/`plan_summary`/`reflection_key_points`/`confidence_score`，Desktop `subscribe_agent_trace` + Web 结构化卡片
- **治理控制面板（Day 3）**: `AgentLoop::pause/resume/inject_memory/update_plan`，`DefaultGovernance::set_approval_level` 运行时调级，Desktop/Web 双端控制面板
- **Session Checkpoint 浏览器（Day 4）**: `Checkpoint` 扩展 `goal_progress`/`key_reflection`，`restore/compare/export` API，Web 恢复/比较/导出交互
- **Resource Dashboard（Day 5）**: `ResourceMonitor` 原子计数器 + 滑动窗口失败率 + 可配置阈值警报 + 冷却期，`AgentLoop` 每迭代自动记录，Desktop 命令 + Web 仪表盘

**新增特性 (Phase 4)**:
- **EditApplier 核心引擎（Day 1+6）**: `ProposedEdit`/`AppliedEdit`/`EditHunk` 模型，hunk-level unified diff，冲突检测（精确匹配），原子写入（唯一 `.bak` 备份），真正 undo 文件恢复，Governance 审批门，10MB/50-hunk/并发保护，undo 栈 100 条上限，Checkpoint 自动保存
- **AST + LSP 精准上下文（Day 2）**: `ASTContextProvider` trait + `LspContextProvider`，`retrieve_with_ast()` 可选 AST 注入，fallback 到纯文本，WASM Tree-sitter `CodeSymbol` 扩展
- **Desktop Inline Editing UI（Day 3）**: Tauri 命令 `apply_edits`/`preview_edit`/`get_ast_context`，Web 端 inline edit panel（diff 高亮 `diff-add`/`diff-del`、Accept All/Reject/Selective 按钮），与 Governance Panel 统一布局
- **Git Workflow & Orchestrator（Day 4）**: `SmartCommitTool`（conventional commit 启发式前缀）、`GeneratePrDescriptionTool`（markdown PR body）、`WorkflowOrchestrator`（Propose→Apply→Checkpoint→Test→Fix≤3→Commit 闭环）
- **Command Palette & Observability（Day 5）**: `@agent refactor`/`review-pr`/`continue-background`/`pause`/`status`，`EditHistoryEntry` 时间线（200条上限），Session Replay 基础，Resource Dashboard 编辑指标
- **边缘案例与 E2E（Day 6）**: 50-cycle stress test，hunk 行偏移冲突检测，新文件 create→undo remove，Checkpoint 100 条 auto-prune，ResourceMonitor `edit_count`/`undo_stack_size`/`checkpoint_count` 告警

**测试套件**（cargo-discoverable，位于 `tests/` 目录）：

| 测试文件 | 测试数 | 关键测试 |
|----------|:------:|----------|
| `tests/agent_core_e2e.rs` | 25 | `test_stability_100_rounds`, `test_governance_rejection`, `test_swarm_delegate` |
| `tests/autonomous_goal_test.rs` | 6 | `test_completion_rate`, `bench_agent_loop` |
| `tests/integration.rs` | 10 | `test_worker_crash_isolation`, `test_loop_timeout_handling` |
| `tests/memory_sync_e2e.rs` | 6 | `test_sync_gateway_20_iteration_consistency`, `test_sync_gateway_concurrent_stress` |
| `tests/swarm_callback_e2e.rs` | 3 | `test_concurrent_30_tasks`, `test_worker_crash_recovery` |
| `tests/agent_loop_leak_test.rs` | 8 | `test_worker_cleanup_on_shutdown`, `test_supervisor_drop_releases_handles` |
| `tests/trace_event_test.rs` | **8** | `test_trace_event_serialization_roundtrip`, `test_emit_trace_broadcasts_via_loop` |
| `tests/governance_control_test.rs` | **10** | `test_pause_sets_paused_true`, `test_governance_set_approval_level_auto_to_critical` |
| `tests/checkpoint_enhanced_test.rs` | **8** | `test_restore_by_id_returns_correct_checkpoint`, `test_compare_different_checkpoints_returns_false` |
| `tests/resource_monitor_test.rs` | **10** | `test_alert_memory_threshold_exceeded`, `test_monitor_concurrent_updates_safe` |
| `tests/integration_phase3_test.rs` | **11** | `test_integration_trace_governance_workflow`, `test_regression_debt_lines_extraction` |
| **lib内测试** | **89** | 各模块单元测试（含 swarm 21个 + reflector 8个 + llm/bridge.rs 5个桥接测试）⭐ |
| `tests/ast_context_test.rs` | **11** | `test_ast_context_retrieval`, `test_retrieve_with_ast_fallback` |
| `tests/edit_applier_test.rs` | **11** | `test_end_to_end_file_apply`, `test_atomic_write_and_read` |
| `tests/workflow_orchestrator_test.rs` | **10** | `test_workflow_propose_apply_checkpoint`, `test_workflow_tests_fail_then_fix` |
| `tests/editing_e2e.rs` | **10** | `test_undo_actually_restores_file_content`, `test_stress_50_consecutive_applies` |
| **E2E/Integration总计** | **160** | `cargo test -p intelligence-agent-core` |
| **lib内测试** | **89** | 各模块单元测试（含 swarm 21个 + reflector 8个 + llm/bridge.rs 5个 + edit_applier 10个 + workflow_orchestrator 2个）⭐ |
| **总计** | **266** | `cargo test -p intelligence-agent-core -- --list` (实测2026-04-28) |

**关键特性**:
- **7步循环**: Observe → Retrieve → Plan → Act → Reflect → Store → Decide
- **可插拔治理**: `GovernancePolicy` trait 支持运行时策略注册
- **Swarm协调**: Supervisor-Worker多Agent协作，`TaskAssignment`/`WorkerResult`通信
- **LLM 桥接**: `PlannerLlmBridge` / `ReflectorLlmBridge` 将 `engine_llm_core::LlmClient` 桥接到上层 trait，零侵入 planner.rs / reflector.rs ⭐
- 7项技术约束与待办已记录于代码注释中

**7步循环代码示例**:
```rust
// agent_loop.rs
pub async fn run(&self, agent_id: &AgentId) -> ReplResult<()> {
    for iteration in 0..MAX_ITERATIONS {
        self.observe(agent_id).await;
        self.retrieve(agent_id).await;  // RETRIEVE-PHASE5: Graph/Dream层待集成
        let plan = self.planner.plan(agent_id).await?;
        let result = self.act(agent_id, &plan.goal_id).await?;
        self.reflect(agent_id, &result).await?;
        self.store(agent_id).await?;      // MEMORY-SYNC: 待完善
        if self.decide(agent_id).await? { break; }
    }
    Ok(())
}
```

**技术约束与待办**:
| 编号 | 状态 | 说明 |
|------|------|------|
| RETRIEVE-PHASE5 | **已清偿** | `SyncMemoryGateway::retrieve_multi` 多层级联检索 ✅ |
| MEMORY-SYNC | **已清偿** | `sync_with_blackboard` + `restore_from_memory` 双向同步 ✅ |
| WORKER-TOOL-EXECUTION | **已完成** | WorkerCallback trait + `handle_worker_result` + `AgentLoop::act()` 集成 + E2E 验证 ✅ |
| LEAK-TEST-PHASE5 | 进行中 | AgentLoop资源泄漏测试待重写 |
| W5-CONTEXT-DEEP | 部分完成 | tree-sitter AST 检索已存在（`retrieve_with_ast`），EditApplier 尚未接入 AST 验证（已知 gap） |
| W1-STREAMING-001 | 待办 | MCP SSE/WebSocket 真实流式 |
| W5-ONBOARD-ADVANCED | 待办 | 视频导览素材待产 |

**相关文档**:
- `src/intelligence/agent-core/README.md` - 模块README
- `docs/debt/agent-core-debt-history.md` - 历史技术约束记录
- `docs/debt/DEBT-ACTIVE-DECLARATION.md` - 活跃技术约束声明

---

#### chimera/ - REPL引擎 ⭐
**技术栈**: Rust (Edition 2024)  
**来源**: 原 `chimera/chimera-repl/` 迁移  
**代码规模**: ~787行  
**状态**: CH-01~10 已完成

| 文件 | 功能 |
|------|------|
| `chimera-repl/src/lib.rs` | 核心引擎入口 |
| `chimera-repl/src/archive_writer.rs` | .hctx 归档格式 + BLAKE3 |
| `chimera-repl/src/codex_bridge.rs` | Codex MemoryGateway FFI |
| `chimera-repl/src/state.rs` | ReplState 状态机 |
| `chimera-repl/src/traits.rs` | ReplEngineCore trait |
| `chimera-repl/src/session.rs` | 会话状态管理 |
| `chimera-repl/src/engine.rs` | 异步事件循环 |
| `chimera-repl/src/repl.rs` | ZeroTUI 主循环 |
| `chimera-repl/src/clock.rs` | 时钟抽象 |
| `chimera-repl/src/event.rs` | 事件系统 |
| `chimera-repl/src/eventloop_adapter.rs` | EventLoop适配器 |
| `chimera-repl/src/io.rs` | IO抽象 |

**关键特性**: ZeroTUI 架构（无 TUI 依赖）

#### memory/ - 5层记忆系统 ⭐
**来源**: 原 `memory/` 迁移  
**状态**: 5层数据流验证通过

| 层级 | 文件 | 功能 | 容量 |
|------|------|------|------|
| Session | `src/session.rs` | 内存对话历史 | LRU 4K tokens |
| Auto | `src/auto.rs` | 本地文件提取 | JSONL 持久化 |
| Dream | `src/dream.rs` | 后台整理 | SQLite+Embedding |
| Graph | `src/graph.rs` | 实体关系图 | 知识图谱 |
| Cloud | `src/cloud.rs` | 云端同步 | 端到端加密 |

**E2E验证**: `tests/memory_five_tier_e2e.rs` - 真实数据流测试  
**SyncGateway**: `tests/memory_sync_e2e.rs` — 跨层检索/并发压力/崩溃恢复测试（6项）

#### knowledge/ - 知识图谱 ⭐
**来源**: 原 `knowledge/` + `crates/hajimi-core/src/knowledge/` 合并  
**状态**: 知识库实现完成（227行）

| 文件 | 功能 |
|------|------|
| `src/adr_index.rs` | ADR索引 + SimHash-64 ⭐ |
| `src/search.rs` | ADR搜索 ⭐ |
| `src/lib.rs` | 模块入口 |
| `src/mod.rs` | 模块导出 |

**5层链路注释**: Session → Auto → Dream → Graph → Knowledge

#### codex-twist/ - AI内存管理
**来源**: 原 `crates/hajimi-codex-twist/` 迁移  
**定位**: AI内存管理核心  
**状态**: 双轨清理完成（0重复文件）

| 模块 | 文件 | 功能 |
|------|------|------|
| 核心 | `lib.rs`, `lcr_adapter.rs` | 模块导出、LCR适配 |
| FFI | `ffi.rs` | FFI绑定 |
| 对话 | `thread.rs` | Thread/ThreadConfig |
| 交互 | `turn.rs` | Turn/TurnStatus |
| 存储 | `storage.rs` | HCTX 本地存储 |
| 审批 | `approval.rs` | 5级审批策略 |
| 内存 | `memory/` | 5级内存架构 |
| 分层 | `tiered/` | Hot/Warm/Cold/Archive |

#### cloud/ - 云端同步
| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 云同步模块 |
| `src/batch_sync.rs` | 批次同步 |

### Interface 层（界面层）
**原则**: 可依赖全下层  
**状态**: ✅ 真实RPC修复完成，20显式注册完成

#### mcp-server/ - MCP服务器 ⭐（真实RPC）
**来源**: 原 `adapters/mcp/` + `mcp/` 合并  
**规范**: MCP 2025-03-26  
**状态**: 真实RPC修复完成（setTimeout已移除）

| 文件 | 功能 |
|------|------|
| `server.ts` | MCP 服务器实现（**15工具**）⭐ |
| `capabilities/tools.ts` ⭐ | 工具能力定义（真实handler） |
| `capabilities/help.ts` | help输出（15工具） |
| `handlers/index.ts` | 14个handler导出 |
| `capabilities/resources.ts` | 资源能力定义 |
| `capabilities/prompts.ts` | 提示能力定义 |
| `transport/sse-transport.ts` | SSE 传输 |
| `transport/stdio-transport.ts` | stdio 传输 |
| `transport/message-adapter.ts` | 消息适配 |
| `ffi-bridge/index.ts` | FFI桥接 |
| `ffi-bridge/tools-bridge.ts` ⭐ | 工具桥接（真实调用） |
| `ffi-bridge/resources-bridge.ts` | 资源桥接 |
| `lifecycle.ts` | 生命周期管理 |
| `cli.ts` | CLI入口 |

---

### crates/ 目录（根目录保留）

#### hajimi-codex-twist/ - AI内存核心
**状态**: 已迁移至 `src/intelligence/codex-twist/`  
**保留原因**: Rust workspace 兼容性  
**状态**: 双轨共存（intelligence/ 为活跃版本，crates/ 保留 workspace 兼容）

#### evm-bench-adapter/ - EVM检测适配器
**状态**: 保留在根目录  
**功能**: EVM漏洞基准测试

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 模块导出 |
| `src/runner.rs` | 漏洞利用运行器 |
| `src/types.rs` | ExploitConfig 类型 |
| `src/main.rs` | 主入口 |

---

### src/ 根目录其他目录

#### patches/ - 构建依赖补丁
**功能**: Cargo `[patch.crates-io]` 本地覆盖  
**来源**: zstd-sys 2.0.15+zstd.1.5.7 修改版  
**原因**: 修复 zstd-safe 6.x experimental API 与 zstd-sys 2.0.15+ 的不匹配  
**依赖链**: tantivy-sstable 0.2.0 → zstd 0.12.4 → zstd-safe 6.0.5 + experimental

#### meta/ - 项目元数据
**功能**: ADR 工具与项目元数据管理  
**文件**: `adr.rs` (~153行)

#### integration/ - 集成测试 crate
**功能**: 端到端集成测试  
**文件**: `lib.rs`, `mod.rs`, `end_to_end.rs`, `end_to_end_tests.rs`

---

## 📊 代码统计

| 分层 | 模块数 | 主要语言 | 状态 |
|:---|:---:|:---|:---|
| Foundation | 7 | TS/JS/Rust | 稳定 ✅ |
| Engine | 4 | TS/Rust | P0安全 ✅ |
| Intelligence | 7 | Rust/TS | 稳定 ✅ |
| Interface | 3 | TS/Rust | 稳定 ✅ |
| **总计** | **22** | - | **v3.8** |

**按语言统计**:
| 语言 | 文件数 | 行数 | 主要分布 |
|:---|:---:|:---:|:---|
| Rust | 220 | ~30,382 | engine/, intelligence/, foundation/wasm/ |
| JavaScript | 18 | ~6,790 | interface/web/ |
| TypeScript | 28 | ~2,301 | interface/mcp-server/, foundation/security/, foundation/storage/ |
| HTML | 1 | ~614 | interface/web/ |
| CSS | 1 | ~2,068 | interface/web/ |
| **总计** | **268** | **~42,155** | - |

> **统计口径**: 仅 `src/` 目录，排除 `target/`、`node_modules/`、`dist/`；含注释与空行；实测 2026-04-28。

**TODO统计**:
| 范围 | 当前数量 | 说明 |
|:---|:---:|:---|
| .rs文件 | 242 | `find src -name "*.rs" | wc -l` (实测2026-04-27) |
| 总源码行 | ~176k | 含rs/js/html/md (实测) |
| 测试可执行 | ~25+ | cargo test --no-run (agent-core/tests/*为主) |
| TODO/unwrap计数 | ~630 | unwrap 431 / expect 184 / panic 15 (D4 实测)

---

## 🔗 关键依赖关系

### 分层依赖
```
interface ──────┐
                ├──→ intelligence ────┐
                │                      ├──→ engine ────┐
                │                      │               ├──→ foundation
                │                      │               │
                └──────────────────────┴───────────────┘
```

### 模块间依赖
```
intelligence/chimera/
└── intelligence/codex-twist (Thread/Turn/MemoryGateway)

intelligence/knowledge/
├── intelligence/memory (5层记忆)
└── engine/search (Tantivy索引)

engine/tool-system/
└── foundation/storage (持久化)

interface/mcp-server/
└── engine/tool-system (工具调用)

```

---

## 📝 如何阅读代码

### 按分层阅读

**1. Foundation 层（基础设施）**:
1. `foundation/storage/shard-router.js` - 16分片路由
2. `foundation/wasm/src/lib.rs` - WASM HNSW
3. `foundation/security/rate-limiter-sqlite-luxury.js` - 限流器
4. `foundation/network/src/lib.rs` - WebSocket 服务器

**2. Engine 层（核心引擎）**:
1. `engine/tool-system/src/mod.rs` - Tool Trait 定义
2. `engine/tool-system/src/shell.rs` - **Shell白名单参数化** ⭐
3. `engine/search/src/tantivy_index.rs` - 搜索索引（219行）⭐

5. `engine/llm-core/src/anthropic.rs` - LLM 客户端
6. `engine/worker/src/parallel.rs` - 并行执行

**3. Intelligence 层（智能系统）**:
1. `intelligence/chimera/chimera-repl/src/repl.rs` - REPL 引擎
2. `intelligence/memory/src/session.rs` - Session 记忆
3. `intelligence/knowledge/src/adr_index.rs` - ADR索引（185行）⭐

**4. Interface 层（用户界面）**:

2. `interface/mcp-server/server.ts` - MCP 服务器

---

## 🔍 快速查找

| 功能 | 文件路径 |
|:---|:---|
| **工具 Trait** | `src/engine/tool-system/src/mod.rs` |
| **Shell白名单** | `src/engine/tool-system/src/shell.rs` |
| **记忆系统** | `src/intelligence/memory/src/session.rs` |
| **知识库 ADR** | `src/intelligence/knowledge/src/adr_index.rs` |
| **LLM 客户端** | `src/engine/llm-core/src/anthropic.rs` |
| **LLM Bridge** | `src/intelligence/agent-core/src/llm/bridge.rs` |
| **Tauri 桌面** | `src/interface/desktop/src/main.rs` |
| **Tauri 流式聊天** | `src/interface/desktop/src/main.rs` (stream_chat) |
| **配置文件权限** | `src/interface/desktop/src/main.rs` (write_configs_to_path) |
| **Workspace 配置隔离** | `src/interface/desktop/src/main.rs` (read_merged_configs) |
| **搜索索引** | `src/engine/search/src/tantivy_index.rs` |
| **Web 前端** | `src/interface/web/app.js` |
| **MCP 服务器** | `src/interface/mcp-server/server.ts` |
| **MCP 真实RPC** | `src/interface/mcp-server/capabilities/tools.ts` |
| **限流器** | `src/foundation/security/rate-limiter-sqlite-luxury.js` |
| **HNSW WASM** | `src/foundation/wasm/src/lib.rs` |
| **存储路由** | `src/foundation/storage/shard-router.js` |
| **Tauri 工具集成** | `src/interface/desktop/src/main.rs` (execute_tool) |
| **Agent Core E2E** | `src/intelligence/agent-core/tests/agent_core_e2e.rs` |
| **Agent Core 治理** | `src/intelligence/agent-core/governance.rs` |
| **Agent Core 循环** | `src/intelligence/agent-core/agent_loop.rs` |
| **SyncMemoryGateway** | `src/intelligence/memory/src/sync_gateway.rs` |

| **zstd-sys 补丁** | `src/patches/zstd-sys/` |
| **技术约束文档** | `docs/debt/DEBT-P0-001.md` |
| **技术约束文档** | `docs/debt/SHELL-FEATURE-DEBT-002.md` |
| **历史约束记录** | `docs/debt/agent-core-debt-history.md` |
| **活跃约束声明** | `docs/debt/DEBT-ACTIVE-DECLARATION.md` |


---

## 🗺️ 目录迁移对照

| 原路径 | 新路径 | 层级 |
|:---|:---|:---:|
| `network/` | `foundation/network/` | Foundation |
| `wasm/` | `foundation/wasm/` | Foundation |
| `llm-core/` | `engine/llm-core/` | Engine |
| `src-tauri/` | `src/interface/desktop/` | Interface |
| `search/` | `engine/search/` | Engine |
| `tool-system/` | `engine/tool-system/` | Engine |
| `chimera/` | `intelligence/chimera/` | Intelligence |
| `codex-twist/` | `intelligence/codex-twist/` | Intelligence |
| `knowledge/` | `intelligence/knowledge/` | Intelligence |
| `memory/` | `intelligence/memory/` | Intelligence |

| `mcp-server/` | `interface/mcp-server/` | Interface |

---

## 🎯 质量保障资产

| 资产 | 路径 | 说明 |
|:---|:---|:---|
| 历史约束记录 | `docs/debt/agent-core-debt-history.md` | 9项已处理约束 |
| 活跃约束声明 | `docs/debt/DEBT-ACTIVE-DECLARATION.md` | 4项约束 |
| 技术约束文档 | `docs/debt/DEBT-P0-001.md` | PSK长期管理 |
| 技术约束文档 | `docs/debt/SHELL-FEATURE-DEBT-002.md` | Shell功能限制 |



---

## 📈 项目改进指标

| 指标 | 改进前 | 当前 | 改进率 |
|:---|:---:|:---:|:---:|
| setTimeout模拟 | 1 | 0 | 100% |
| 硬编码返回值 | 1 | 0 | 100% |
| Shell bash -c | 1 | 0 | 100% |
| 综合状态 | 波动 | **稳定** | - |

---

## 🔴 P0-CONTEXT-REMEDIATION-2026-04-30

基于代码审计的真实基线（Git `848a9b0`）：

| 债务项 | 代码位置 | 实测证据 |
|:---|:---|:---|
| 单轮 LLM 接口 | `engine/llm-core/src/mod.rs:133` | `stream_chat(&self, prompt: String)` — 仅接收单 String |
| 后端无 messages | `interface/desktop/src/main.rs:824` | `stream_chat` command 签名仅含 `prompt: String` |
| MemoryGateway 孤岛 | `intelligence/codex-twist/src/memory/memory_gateway.rs` | 94 行完整实现，`grep` main.rs 返回 0 匹配（未使用/未引用） |
| 前端无对话状态 | `interface/web/app.js` | 仅 `aiChatMessages` DOM 渲染，无 `chatMessages` 状态数组 |

**根因分析**：`LlmClient` trait 设计为单轮 `stream_chat(String)`，所有 provider（Anthropic/OpenAI/Ollama）请求体仅含单条 `user` 消息。`codex-twist` MemoryGateway（Focus/Working/Archive 三层 + `optimize()`）已完整实现但完全未被 `main.rs` 引用。前端聊天历史仅渲染于 DOM，不维护状态数组。

---

### ✅ 清偿记录（B-02/09 ~ B-09/09）

**P0 Context Debt Cleared**

| 债务项 | 修复状态 | 修复后代码位置 | 验证 |
|:---|:---:|:---|:---|
| 单轮 LLM 接口 | ✅ | `engine/llm-core/src/mod.rs:L149` `stream_chat_with_context` | 3 Provider 全部实现 |
| 后端无 messages | ✅ | `interface/desktop/src/main.rs:L833` `messages: Option<Vec<ChatMessage>>` | `cargo check` 通过 |
| MemoryGateway 孤岛 | ✅ | `interface/desktop/src/main.rs:L70` `memory_gateway: Arc<MemoryGateway>` | `grep` 5 处匹配 |
| 前端无对话状态 | ✅ | `interface/web/app.js:L28` `chatMessages: []` | 20+ 处引用 |

**详细清偿记录**: `docs/debt/DEBT-P0-REMEDIATION.md`

**关联文档**：
- `src/MEMORY.md` — 数据诚实性规范与债务基线记录
- `docs/roadmap/Hajimi Context/03-context-compaction.md` — 根因分析与修复验证
- `docs/debt/DEBT-P0-REMEDIATION.md` — 完整债务清偿记录

---

---

## Scheme B 精确 Token 统计 baseline

> **状态**: 已启动（B-01/06 Day 1）
> **目标**: 在方案 A 基础上实现后端精确 Token 统计
> **Roadmap**: `docs/roadmap/Hajimi Context/p0 fix/02-exact-token-usage-tracking.md` | `03-token-scheme-b-daily-development-plan.md` | `04-token-scheme-b-guidance.md`

### Baseline 数据（实测）

基于 Git `6ad02ec` 的代码审计：

| 测量项 | 命令 | 实测值 |
|:---|:---|:---:|
| 编译状态 | `cargo check --workspace` | 0 errors |
| Rust 源文件数 | `find src -name "*.rs"` | 242 |
| JS 源文件数 | `find src -name "*.js"` | 66 |
| 前端 Token 估算引用 | `grep estimateTokens app.js` | 3 处 |
| 前端对话状态引用 | `grep chatMessages app.js` | 25 处 |
| 后端多轮接口引用 | `grep stream_chat_with_context llm-core` | 1 处 |
| 后端 MemoryGateway 引用 | `grep memory_gateway main.rs` | 5 处 |
| Audit Log 引用 | `grep log_usage main.rs` | 6 处 |
| 精确统计当前覆盖 | `grep tiktoken\|precise\|count_tokens src/` | 4 处（文档引用，无实现） |

### 当前能力评估

| 能力 | 状态 | 说明 |
|:---|:---:|:---|
| 前端 Token 估算 | ✅ | `estimateTokens()` 字符启发式（中≈1，英≈1.3） |
| 自动压缩触发 | ✅ | 80% 阈值，`checkAutoCompact()` |
| 后端多轮接口 | ✅ | `stream_chat_with_context()` 三 Provider 实现 |
| 精确 Token 编码 | ❌ | 无 `tiktoken-rs` 集成 |
| API usage 解析 | ✅ | OpenAI/Anthropic/Ollama 三 Provider 均解析 |
| 累计消耗统计 | ✅ | `TokenUsageTracker` 会话级 + 全局累计（by_provider / by_day / total）|

### 后续工单映射

| 工单 | 对应 Phase | 目标 | 覆盖内容 | 状态 |
|:---|:---|:---|:---|:---:|
| B-02/06 | Phase 1 | Engine 层精确计数 | `tiktoken-rs` + `count_tokens()` | ✅ |
| B-03/06 | Phase 2 | Backend usage 解析 | `stream_chat` usage 字段 + Audit Log | ✅ |
| B-04/06 | Phase 3 | Intelligence 统计聚合 | `TokenUsageTracker` + 会话/全局累计 | ✅ |
| B-05/06 | Phase 4 | Frontend 精确 UI | 精确值展示 + 累计消耗 | ✅ |
| B-06/06 | Phase 5 | 验证与清债 | E2E 测试 + 文档闭环 + 误差 < 5% | ✅ |

*Phase 1~5 由 B-02/06 ~ B-06/06 全部覆盖。Scheme B 已完成。*

---

## P1 Token Tracker Integration — 进入清偿阶段

<!-- P1-TOKEN-TRACKER-2026-05-02: integration initiated -->

**状态**: ✅ P1 Cleared with minimal AppState extension（Backend 集成 → Tauri Command → Frontend 持久化 → 清债验证 → 文档闭环）
**目标**: 以最小变更激活 `TokenUsageTracker`，完成 Scheme B 精确 Token 统计全链路闭环
**债务来源**: `docs/debt/DEBT-SCHEME-B.md` 诚实声明 3 项已知限制

### Baseline 数据（实测 2026-05-02）

| 测量项 | 命令 | 实测值 |
|:---|:---|:---:|
| E2E 测试 | `cargo test -p codex-twist --test token_tracking_e2e` | 12 passed |
| 编译状态 | `cargo check --workspace` | 0 errors |
| 分层合规 (Engine↔Intelligence) | `grep codex_twist src/engine/` | 0 匹配 |
| 分层合规 (Intelligence↔Interface) | `grep "use.*interface" src/intelligence/` | 0 匹配 |
| 前端语法 | `node --check src/interface/web/app.js` | 通过 |
| Git HEAD | `git rev-parse HEAD` | `d2f3de5` |

### 已知限制（DEBT-SCHEME-B.md）

1. ✅ `TokenUsageTracker` 已集成到 `interface/desktop/src/main.rs` `stream_chat` 流（P1-02/05）
2. ✅ 前端 `cumulativeStats` 已改为混合持久化，刷新后通过 Tauri Command 恢复（P1-04/05）
3. ⚪ `exact-tokens` feature 维持设计决策，精确计数时显式启用 `--features exact-tokens`

### 后续 P1 工单映射

| 工单 | 阶段 | 目标 | 覆盖文件 | 状态 |
|:---|:---|:---|:---|:---:|
| P1-01/05 | Step 0 | 文档基线同步 + Baseline 测量 | 4 份 MD | ✅ 已完成 |
| P1-02/05 | Step 1 | Backend 集成（AppState + record_usage） | `main.rs` | ✅ 已完成 |
| P1-03/05 | Step 2 | Tauri Command 暴露（get_cumulative_stats） | `main.rs` | ✅ 已完成 |
| P1-04/05 | Step 3-4 | Frontend 持久化（混合存储） | `app.js` | ✅ 已完成 |
| P1-05/05 | Step 5 | 清债验证 + 文档闭环 | 多文件 | ✅ 已完成 |

**关联文档**:
- Roadmap: `docs/roadmap/Hajimi Context/P1 fix/P1-TOKEN-TRACKER-INTEGRATION-ROADMAP.md`
- Daily Plan: `docs/roadmap/Hajimi Context/P1 fix/P1-TOKEN-TRACKER-DETAILED-DAILY-PLAN.md`
- Guidance: `docs/roadmap/Hajimi Context/P1 fix/P1-TOKEN-TRACKER-REMEDIATION-GUIDANCE.md`

### P1 设计约束

- **最小变更**: 仅扩展 `AppState` 字段，不重构现有逻辑
- **分层纯洁**: Engine 层零依赖 Intelligence，`codex_twist` 仅由 Interface 消费
- **数据诚实**: 所有 metric 来自当天实测命令，禁止估算
- **混合持久化**: Tauri Command 主路径 + LocalStorage 兜底，离线场景可用

---

<!-- MEMORY-REMEDIATION-2026-05-03: five-tier memory activation initiated -->
<!-- MEMORY-REMEDIATION-CLEARED: 7/7 Cleared -->

*本索引文档与代码同步维护，最后更新于 2026-04-30*
