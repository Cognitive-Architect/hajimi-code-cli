# HAJIMI V3 源代码索引

> **文档版本**: v3.8.0-batch-1 (Phase 7 Debt Clearance + DEBT-LLM-CLIENT 清偿 / B+→A-级评级)  
> **最后更新**: 2026-04-23  
> **代码总行数**: ~65,482行自有代码（不含依赖，含agent-core ~2,760行源文件 / ~3,184行总计）  
> **架构**: 四层分层（Foundation/Engine/Intelligence/Interface）  
> **Phase 7状态**: ✅ Agent Core 55测试通过，0编译error，unsafe SAFETY 100%覆盖

---

## 📁 目录总览（v3.1 四层架构 + 债务清偿后）

```
src/
├── crates/              # 保留的 Rust Crates（根目录）
│   ├── evm-bench-adapter/   # EVM检测适配器
│   └── hajimi-codex-twist/  # AI内存核心 (Rust workspace兼容)
│
├── foundation/          # 地基层 - 零依赖（17模块）
│   ├── api/             # REST API 服务器
│   ├── bench/           # 性能基准测试
│   ├── compression/     # 上下文压缩（micro/auto/compact/mod）
│   ├── db/              # PostgreSQL连接池
│   ├── disk/            # 磁盘管理（ENOSPC处理）
│   ├── eventloop/       # 异步事件循环 (Rust)
│   ├── format/          # 数据格式（.hctx/BLAKE3）
│   ├── middleware/      # Express/Koa中间件
│   ├── migration/       # 数据库迁移工具
│   ├── network/         # WebSocket服务器（PSK认证）⭐
│   ├── scripts/         # 构建/安装脚本（SHA256校验）⭐
│   ├── security/        # 安全与限流控制
│   ├── storage/         # 存储层（16分片SQLite）⭐
│   ├── test/            # 单元测试辅助
│   ├── tests/           # 测试套件（E2E/集成）
│   ├── utils/           # 通用工具（SimHash64）
│   └── wasm/            # WASM运行时（HNSW）⭐
│
├── engine/              # 引擎层 - 仅依赖foundation（5模块）
│   ├── llm-core/        # LLM客户端（Anthropic/OpenAI/Ollama）
│   ├── p2p-sync/        # P2P同步引擎（WebRTC/CRDT/PSK认证）⭐
│   ├── search/          # 搜索索引（Tantivy 16分片）⭐
│   ├── tool-system/     # 工具系统（40+工具/白名单参数化）⭐
│   └── worker/          # 工作线程池
│
├── intelligence/        # 智能层 - 依赖foundation+engine（11模块）
│   ├── agent-core/      # 自主Agent系统（7步循环/Swarm/可插拔治理/LLM桥接）⭐
│   │   └── llm/         #   LLM适配器桥接（PlannerLlmBridge + ReflectorLlmBridge）
│   ├── chimera/         # Chimera REPL引擎（Rust）⭐
│   ├── cloud/           # 云端同步（批次同步）
│   ├── codex-twist/     # AI内存管理（5级架构/双轨清理完成）⭐
│   ├── index/           # [已归档] 向量索引（HNSW+Tantivy）
│   ├── integration/     # 集成模块
│   ├── knowledge/       # 知识图谱（ADR/GNN/知识库）⭐
│   ├── memory/          # 5层记忆系统⭐
│   ├── onnx/            # [已归档] ONNX推理引擎
│   ├── pgvector/        # PostgreSQL向量扩展
│   └── typeracing/      # [已归档] LSP驱动类型预测引擎
│
└── interface/           # 界面层 - 依赖全下层（4模块）
    ├── cli/             # [已归档] CLI工具
    ├── mcp-server/      # MCP服务器（真实RPC桥接）⭐
    ├── terminal/        # [已归档] 终端UI
    └── vscode/          # [已归档] VSCode插件
```

---

## 🎯 分层详解（债务清偿后状态）

### Foundation 层（地基层）
**原则**: 零外部依赖，提供基础设施  
**债务状态**: ✅ P0安全加固完成，TODO清收完成

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
**债务**: DEBT-P0-001 (PSK长期管理)

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | WebSocket 服务器实现 |
| `src/handlers.rs` | 消息处理器 |
| `src/protocol.rs` | 协议定义 |

#### security/ - 安全组件
**功能**: 限流、审计、沙盒

| 文件 | 功能 |
|------|------|
| `rate-limiter-sqlite-luxury.js` | SQLite持久化限流器 |
| `rate-limiter-redis.js` | Redis限流器 |
| `audit-logger.rs` | 审计日志 |

#### utils/ - 通用工具
**功能**: SimHash64（8处分散引用，未完全统一）

| 文件 | 功能 |
|------|------|
| `simhash.js` | SimHash-64实现 |
| `logger.js` | 日志工具 |

---

### Engine 层（引擎层）
**原则**: 仅依赖 Foundation 层  
**债务状态**: ✅ P0安全加固完成，Shell白名单参数化

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
// 债务: docs/debt/SHELL-FEATURE-DEBT-002.md
```

#### search/ - 搜索索引 ⭐ (Phase 5)
**技术**: Tantivy 全文搜索 + 16分片  
**债务状态**: ✅ 完成

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 搜索模块入口 |
| `src/tantivy_index.rs` | Tantivy 16分片索引（219行）⭐ |
| `src/tantivy_query.rs` | 查询解析 |
| `src/vector_text_hybrid.rs` | 向量+文本混合搜索 |
| `src/debug_test.rs` | 调试测试 |

**性能指标**: 混合搜索，16分片并行

#### p2p-sync/ - P2P同步引擎 ⭐（P0加固）
**来源**: 原 `p2p/` + `sync/` 合并  
**技术**: TypeScript + WebRTC DataChannel  
**协议**: ICEv2 (RFC 8445) + Yjs CRDT + **PSK认证（B-03 P0）**

| 文件 | 功能 |
|------|------|
| `sync-engine.ts` | 同步生命周期管理 |
| `sync-engine.ts` | 同步引擎接口（141行，B-02提取后） |
| `progress-bar.ts` ⭐ | 进度条模块（57行，从sync-engine提取） |
| `crdt-engine.ts` | Yjs CRDT 引擎封装 |
| `datachannel-manager.js` | DataChannel 管理 |
| `signaling-server.js` ⭐ | WebSocket 信令服务器（PSK认证） |
| `signaling-client.js` | WebRTC 客户端 |
| `ice-manager.ts` | ICE 候选管理 |
| `ice-v2-client.ts` | ICEv2 客户端 |
| `turn-client.ts` | TURN 客户端 |
| `dvv-manager.ts` | DVV 版本向量管理 |
| `latency-monitor.ts` | 延迟监控 |
| `memory-monitor.ts` | 内存监控 |
| `yjs-adapter.ts` | Yjs 适配器 |

**PSK认证（B-03 P0）**:
```javascript
const crypto = require('crypto');
const clientId = crypto.randomUUID(); // CSPRNG，替换Math.random()
const psk = process.env.HAJIMI_SIGNALING_PSK; // 环境变量
// timingSafeEqual验证
```

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
**债务状态**: ✅ TypeRacing还魂完成，TODO清收完成

#### agent-core/ - 自主Agent系统 ⭐（Day 10 FULL）
**技术栈**: Rust (tokio + async-trait)  
**来源**: Day 1-10 渐进式构建（Planner→Reflector→Governance→Swarm→AgentLoop）  
**代码规模**: ~2,760行源文件 / ~3,184行总计（16源文件 + 3测试文件）  
**状态**: A级审计通过，55 lib测试 + 90 E2E测试全部通过，0编译warning  
**DEBT**: 4项Phase 5债务 + 3项Phase 7延续债务诚实申报（目标≤8）

| 文件 | 功能 | 行数 |
|------|------|:----:|
| `lib.rs` | 模块入口 + 公共类型 | 192 |
| `agent_loop.rs` | 7步自主循环（Observe→Retrieve→Plan→Act→Reflect→Store→Decide） | 242 |
| `agent_loop_builder.rs` | AgentLoop构建器 | 53 |
| `swarm.rs` | Swarm协调器（Supervisor-Worker模式） | 277 |
| `governance.rs` | 可插拔治理（5级审批策略 + 投票机制） | 271 |
| `orchestrator.rs` | Agent编排器 | 293 |
| `planner.rs` | 任务规划器 | 182 |
| `reflector.rs` | 反思优化器 | 314 |
| `blackboard.rs` | 共享黑板状态 | 136 |
| `checkpoint.rs` | 检查点恢复 | 95 |
| `events.rs` | 事件系统 | 127 |
| `tools.rs` | 工具集成 | 179 |
| `degrade.rs` | 降级策略 | 15 |
| `ports.rs` | 端口抽象 | 53 |
| `llm/bridge.rs` | **LLM 适配器桥接**（PlannerLlmBridge + ReflectorLlmBridge）⭐ | 164 |
| `llm/mod.rs` | LLM 模块入口 | 4 |
| `mod.rs` | 公共API导出（含DEBT声明） | 36 |

**测试套件**（cargo-discoverable，位于 `tests/` 目录）：

| 测试文件 | 测试数 | 关键测试 |
|----------|:------:|----------|
| `tests/agent_core_e2e.rs` | 25 | `test_stability_100_rounds`, `test_governance_rejection`, `test_swarm_delegate` |
| `tests/autonomous_goal_test.rs` | 8 | `test_completion_rate`, `bench_agent_loop` |
| `tests/integration.rs` | 10 | `test_worker_crash_isolation`, `test_loop_timeout_handling` |
| **lib内测试** | **55** | 各模块单元测试（含 llm/bridge.rs 5个桥接测试）⭐ |
| **E2E总计** | **90** | `cargo test -p intelligence-agent-core` |

**关键特性**:
- **7步循环**: Observe → Retrieve → Plan → Act → Reflect → Store → Decide
- **可插拔治理**: `GovernancePolicy` trait 支持运行时策略注册
- **Swarm协调**: Supervisor-Worker多Agent协作，`TaskAssignment`/`WorkerResult`通信
- **LLM 桥接**: `PlannerLlmBridge` / `ReflectorLlmBridge` 将 `engine_llm_core::LlmClient` 桥接到上层 trait，零侵入 planner.rs / reflector.rs ⭐
- **诚实性**: 4项 `DEBT-XXX-PHASE5` 注释 + 3项 Phase 7 延续债务（非虚构，非阻塞）

**7步循环代码示例**:
```rust
// agent_loop.rs
pub async fn run(&self, agent_id: &AgentId) -> ReplResult<()> {
    for iteration in 0..MAX_ITERATIONS {
        self.observe(agent_id).await;
        self.retrieve(agent_id).await;  // DEBT-RETRIEVE-PHASE5
        let plan = self.planner.plan(agent_id).await?;
        let result = self.act(agent_id, &plan.goal_id).await?;
        self.reflect(agent_id, &result).await?;
        self.store(agent_id).await?;      // DEBT-MEMORY-SYNC
        if self.decide(agent_id).await? { break; }
    }
    Ok(())
}
```

**DEBT Summary**:
| DEBT | 状态 | 说明 |
|------|------|------|
| DEBT-RETRIEVE-PHASE5 | 活跃 | Graph/Dream层记忆检索待全面集成 |
| DEBT-WORKER-TOOL-EXECUTION | 活跃 | Worker执行结果回调机制待完善 |
| DEBT-LEAK-TEST-PHASE5 | 活跃 | AgentLoop资源泄漏测试待重写 |
| DEBT-W5-CONTEXT-DEEP | 延续Phase 8 | tree-sitter AST 感知上下文 |
| DEBT-W1-STREAMING-001 | 延续Phase 8 | MCP SSE/WebSocket 真实流式 |
| DEBT-W5-ONBOARD-ADVANCED | 延续Phase 8 | 视频导览素材待产 |

**相关文档**:
- `src/intelligence/agent-core/README.md` - 模块README
- `docs/debt/agent-core-debt-history.md` - 9项已清偿债务
- `docs/debt/DEBT-ACTIVE-DECLARATION.md` - 4项活跃债务声明

---

#### chimera/ - REPL引擎 ⭐
**技术栈**: Rust (Edition 2024)  
**来源**: 原 `chimera/chimera-repl/` 迁移  
**代码规模**: ~787行  
**状态**: CH-01~10 已完成

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 核心引擎入口 |
| `src/archive_writer.rs` | .hctx 归档格式 + BLAKE3 |
| `src/codex_bridge.rs` | Codex MemoryGateway FFI |
| `src/state.rs` | ReplState 状态机 |
| `src/traits.rs` | ReplEngineCore trait |
| `src/session.rs` | 会话状态管理 |
| `src/engine.rs` | 异步事件循环 |
| `src/repl.rs` | ZeroTUI 主循环 |
| `src/clock.rs` | 时钟抽象 |
| `src/event.rs` | 事件系统 |
| `src/eventloop_adapter.rs` | EventLoop适配器 |
| `src/io.rs` | IO抽象 |

**关键特性**: ZeroTUI 架构（无 TUI 依赖）

#### memory/ - 5层记忆系统 ⭐
**来源**: 原 `memory/` 迁移  
**Phase 5**: 5层数据流验证通过

| 层级 | 文件 | 功能 | 容量 |
|------|------|------|------|
| Session | `src/session.rs` | 内存对话历史 | LRU 4K tokens |
| Auto | `src/auto.rs` | 本地文件提取 | JSONL 持久化 |
| Dream | `src/dream.rs` | 后台整理 | SQLite+Embedding |
| Graph | `src/graph.rs` | 实体关系图 | 知识图谱 |
| Cloud | `src/cloud.rs` | 云端同步 | 端到端加密 |

**E2E验证**: `tests/memory_five_tier_e2e.rs` - 真实数据流测试

#### knowledge/ - 知识图谱 ⭐
**来源**: 原 `knowledge/` + `crates/hajimi-core/src/knowledge/` 合并  
**Phase 5 Month 4**: 知识库实现完成（227行）

| 文件 | 功能 |
|------|------|
| `src/adr_index.rs` | ADR索引 + SimHash-64（185行）⭐ |
| `src/search.rs` | ADR搜索（35行）⭐ |
| `src/mod.rs` | 模块导出（7行）⭐ |
| `src/graph/mod.rs` | 知识图谱核心 |
| `src/graph/db.rs` | 图数据库接口 |
| `src/graph/gnn_impl.rs` | GNN 实现 |
| `src/graph/attention.rs` | 注意力机制 |
| `src/graph/traversal.rs` | 图遍历 |
| `src/core_adr/` | ADR 架构决策记录模块 |
| `src/core_adapters/` | 核心适配器 |
| `src/core_relations/` | 核心关系提取 |

**5层链路注释**: Session → Auto → Dream → Graph → Knowledge

#### typeracing/ - 类型预测引擎 ⭐（Week 6还魂）
**Phase 5**: LSP驱动的智能代码补全  
**依赖**: `engine/tool-system` LSP工具  
**触发**: Ctrl+Space

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 模块导出 |
| `src/engine.rs` | 类型预测引擎 |
| `src/algorithm.rs` | 预测算法 |
| `src/terminal_adapter.rs` ⭐ | 终端适配器（Ctrl+Space触发） |

**核心算法**: 
- `calculate_weighted_confidence`: 加权置信度
- `rank_predictions`: 预测排序
- `select_top_k`: Top-K选择

**TerminalAdapter**:
```rust
pub struct TerminalAdapter {
    engine: Arc<Mutex<Engine>>,
    state: AdapterState,
    last_predictions: Vec<PredictionNode>,
}
// Ctrl+Space触发 spawn_predict()
```

#### index/ - 向量索引 ⭐
**来源**: 原 `index/` + `vector/` 合并

| 文件 | 功能 |
|------|------|
| `src/tantivy.rs` | 全文索引（Tantivy）|
| `src/pgvector.rs` | PG向量索引 |
| `src/unified.rs` | 统一检索接口 |
| `src/batch_compute.rs` | 批量计算 |
| `src/mod.rs` | 模块入口 |
| `vector/*.js` | HNSW JS/WASM 实现 |
| `vector/hnsw-core.js` | HNSW核心 |
| `vector/hnsw-index-wasm-v3.js` | WASM索引v3 |
| `vector/wasm-loader.js` | WASM加载器 |

#### codex-twist/ - AI内存管理
**来源**: 原 `crates/hajimi-codex-twist/` 迁移  
**定位**: OpenAI Codex 本地优先移植版  
**债务**: 双轨清理完成（0重复文件）

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

#### cloud/ - 云端同步 (Phase 5)
| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 云同步模块 |
| `src/batch_sync.rs` | 批次同步 |

#### onnx/ - ONNX推理
| 文件 | 功能 |
|------|------|
| `mod.rs` | 模块入口 |
| `adapter.rs` | 适配器 |
| `real_inference.rs` | 真实推理 |

---

### Interface 层（界面层）
**原则**: 可依赖全下层  
**债务状态**: ✅ Week 9真实RPC修复完成，20显式注册完成

#### terminal/ - 终端UI ⭐（TypeRacing集成）
**来源**: 原 `crates/hajimi-core/src/ui/terminal/` 迁移  
**技术**: Ink + React

| 文件 | 功能 |
|------|------|
| `src/mod.rs` | 终端UI主模块（TypeRacing集成） |
| `src/layout.rs` | 布局管理 |
| `src/pane.rs` | Pane 组件 |
| `src/pane_manager.rs` | Pane 管理器 |
| `src/pane_layout.rs` | Pane 布局 |
| `src/pane_utils.rs` | Pane 工具 |
| `src/keymap_emacs.rs` | Emacs 快捷键 |
| `src/keymap_vim.rs` | Vim 快捷键 |
| `src/theme.rs` | 主题系统 |
| `src/animation.rs` | 动画效果 |
| `src/virtual_list.rs` | 虚拟列表 |
| `src/input_handler.rs` | 输入处理（Ctrl+Space触发） |
| `src/config.rs` | 配置管理 |
| `src/config_utils.rs` | 配置工具 |

#### mcp-server/ - MCP服务器 ⭐（真实RPC）
**来源**: 原 `adapters/mcp/` + `mcp/` 合并  
**规范**: MCP 2025-03-26  
**债务**: Week 9真实RPC修复完成（setTimeout清零）

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

#### vscode/ - VSCode插件 ⭐（20显式注册）
**债务**: Week 9修复完成（真实RPC桥接）

| 文件 | 功能 |
|------|------|
| `extension.ts` | 插件入口 |
| `src/registry/CommandRegistry.ts` ⭐ | 命令注册（7命令止血+真实RPC） |
| `src/clients/LspClient.ts` ⭐ | LSP客户端（WebSocket） |
| `src/managers/TreeViewManager.ts` | 树形视图管理器 |
| `package.json` | 7命令定义（Week 6止血完成） |

**CommandRegistry真实RPC（Week 9）**:
```typescript
// 真实RPC调用（非setTimeout模拟）
private async invokeMcpTool(toolName: string, args: unknown[] = []): Promise<unknown> {
  const result = await this.lspClient.sendCustomRequest<unknown>('mcp/toolCall', {
    tool: toolName,
    arguments: args
  });
  return result; // 真实结果透传
}

// 7个真实命令显式注册（Week 6止血：从64→7）
this.registerCommand(CommandId.RUN_TESTS, async () => {
  return this.invokeMcpTool('run_tests');
});
// ... 共7个命令（4 built-in + 3 MCP）
```

#### cli/ - CLI工具
| 文件 | 功能 |
|------|------|
| `vector-debug.js` | 向量调试工具 |

---

### crates/ 目录（根目录保留）

#### hajimi-codex-twist/ - AI内存核心
**状态**: 已迁移至 `src/intelligence/codex-twist/`  
**保留原因**: Rust workspace 兼容性  
**债务**: 双轨清理完成（0文件）

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

## 📊 代码统计（8周债务清偿后）

| 分层 | 模块数 | 主要语言 | 状态 |
|:---|:---:|:---|:---|
| Foundation | 17 | TS/JS/Rust | 稳定 ✅ |
| Engine | 5 | TS/Rust | P0安全 ✅ |
| Intelligence | 11 | Rust/TS | Day 10 A级 ✅ |
| Interface | 4 | TS/Rust | 真实RPC ✅ |
| **总计** | **37** | - | **v3.2** |

**按语言统计**:
| 语言 | 文件数 | 行数 | 主要分布 |
|:---|:---:|:---:|:---|
| Rust | 220 | ~25,428 | engine/, intelligence/, foundation/wasm/ |
| JavaScript | 131 | ~26,917 | foundation/, interface/, engine/p2p-sync/ |
| TypeScript | 114 | ~10,853 | foundation/, interface/, engine/p2p-sync/ |
| TSX | 23 | ~2,284 | interface/ |
| **总计** | **488** | **~65,482** | - |

**TODO清收统计**:
| 范围 | 清偿前 | 清偿后 | 清收率 |
|:---|:---:|:---:|:---:|
| src目录 | 1,292 | 10 | 99.2% |
| engine核心层 | - | 0 | - |
| intelligence层 | - | 5 | 4项Phase 5诚实申报 |
| agent-core DEBT | 13有记录 | 4活跃 | 69.2% |
|  | *(历史总数22含早期未归档条目)* |  |  |

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

intelligence/index/
├── foundation/wasm (WASM HNSW)
└── intelligence/memory (向量存储)

intelligence/typeracing/
└── engine/tool-system (LSP工具)

intelligence/knowledge/
├── intelligence/memory (5层记忆)
└── engine/search (Tantivy索引)

engine/p2p-sync/
├── Yjs (CRDT)
├── @koush/wrtc (WebRTC)
└── ws (WebSocket)

engine/tool-system/
└── foundation/storage (持久化)

interface/mcp-server/
└── engine/tool-system (工具调用)

interface/vscode/
├── LspClient (WebSocket)
└── CommandRegistry (真实RPC桥接)
```

---

## 📝 如何阅读代码

### 按分层阅读

**1. Foundation 层（基础设施）**:
1. `foundation/storage/shard-router.js` - 16分片路由
2. `foundation/wasm/src/lib.rs` - WASM HNSW
3. `foundation/security/rate-limiter-sqlite-luxury.js` - 限流器
4. `foundation/network/src/lib.rs` - WebSocket 服务器
5. `foundation/compression/mod.rs` - 压缩模块

**2. Engine 层（核心引擎）**:
1. `engine/tool-system/src/mod.rs` - Tool Trait 定义
2. `engine/tool-system/src/shell.rs` - **Shell白名单参数化** ⭐
3. `engine/search/src/tantivy_index.rs` - 搜索索引（219行）⭐
4. `engine/p2p-sync/src/signaling-server.js` - **WebRTC PSK认证** ⭐
5. `engine/llm-core/src/anthropic.rs` - LLM 客户端
6. `engine/worker/src/parallel.rs` - 并行执行

**3. Intelligence 层（智能系统）**:
1. `intelligence/chimera/src/repl.rs` - REPL 引擎
2. `intelligence/memory/src/session.rs` - Session 记忆
3. `intelligence/knowledge/src/adr_index.rs` - ADR索引（185行）⭐
4. `intelligence/typeracing/src/terminal_adapter.rs` - **TypeRacing** ⭐
5. `intelligence/index/src/tantivy.rs` - 全文索引

**4. Interface 层（用户界面）**:
1. `interface/terminal/src/mod.rs` - 终端UI
2. `interface/mcp-server/server.ts` - MCP 服务器
3. `interface/vscode/src/registry/CommandRegistry.ts` - **真实RPC** ⭐
4. `interface/vscode/extension.ts` - VSCode 插件

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
| **P2P 同步** | `src/engine/p2p-sync/src/sync-engine.ts` |
| **P2P PSK认证** | `src/engine/p2p-sync/src/signaling-server.js` |
| **搜索索引** | `src/engine/search/src/tantivy_index.rs` |
| **终端 UI** | `src/interface/terminal/src/mod.rs` |
| **MCP 服务器** | `src/interface/mcp-server/server.ts` |
| **真实RPC** | `src/interface/vscode/src/registry/CommandRegistry.ts` |
| **限流器** | `src/foundation/security/rate-limiter-sqlite-luxury.js` |
| **向量索引** | `src/intelligence/index/src/tantivy.rs` |
| **HNSW WASM** | `src/foundation/wasm/src/lib.rs` |
| **存储路由** | `src/foundation/storage/shard-router.js` |
| **TypeRacing** | `src/intelligence/typeracing/src/terminal_adapter.rs` |
| **Agent Core E2E** | `src/intelligence/agent-core/tests/agent_core_e2e.rs` |
| **Agent Core 治理** | `src/intelligence/agent-core/governance.rs` |
| **Agent Core 循环** | `src/intelligence/agent-core/agent_loop.rs` |
| **E2E回归** | `tests/e2e/phase1-5-regression/full_chain.test.js` |
| **债务文档** | `docs/debt/DEBT-P0-001.md` |
| **债务文档** | `docs/debt/SHELL-FEATURE-DEBT-002.md` |
| **Day 10审计** | `docs/self-audit/DAY10-AGENT-CORE-SELF-AUDIT-001.md` |
| **债务历史** | `docs/debt/agent-core-debt-history.md` |
| **活跃债务** | `docs/debt/DEBT-ACTIVE-DECLARATION.md` |
| **8周审计** | `audit report/8week/HAJIMI-8WEEK-DEBT-CLEARANCE-AUDIT.md` |

---

## 🗺️ 目录迁移对照

| 原路径 (v1.x) | 新路径 (v3.1) | 层级 |
|:---|:---|:---:|
| `api/` | `foundation/api/` | Foundation |
| `compression/` | `foundation/compression/` | Foundation |
| `db/` | `foundation/db/` | Foundation |
| `network/` | `foundation/network/` | Foundation |
| `wasm/` | `foundation/wasm/` | Foundation |
| `llm-core/` | `engine/llm-core/` | Engine |
| `p2p-sync/` | `engine/p2p-sync/` | Engine |
| `search/` | `engine/search/` | Engine |
| `tool-system/` | `engine/tool-system/` | Engine |
| `chimera/` | `intelligence/chimera/` | Intelligence |
| `codex-twist/` | `intelligence/codex-twist/` | Intelligence |
| `index/` | `intelligence/index/` | Intelligence |
| `knowledge/` | `intelligence/knowledge/` | Intelligence |
| `memory/` | `intelligence/memory/` | Intelligence |
| `onnx/` | `intelligence/onnx/` | Intelligence |
| `typeracing/` | `intelligence/typeracing/` | Intelligence |
| `mcp-server/` | `interface/mcp-server/` | Interface |
| `terminal/` | `interface/terminal/` | Interface |
| `vscode/` | `interface/vscode/` | Interface |

---

## 🎯 8周债务清偿资产

| 资产 | 路径 | 说明 |
|:---|:---|:---|
| 8周审计报告 | `audit report/8week/HAJIMI-8WEEK-DEBT-CLEARANCE-AUDIT.md` | A-级评级确认 |
| DEBT-PHASE2审计 | `audit report/DEBT-PHASE2-CONSTRUCTIVE-AUDIT-REPORT.md` | B+级，2项返工 |
| DEBT-PHASE2-REWORK审计 | `audit report/DEBT-PHASE2-REWORK-CONSTRUCTIVE-AUDIT-REPORT.md` | A-级，Go |
| Week 9审计 | `audit report/week9/WEEK9-TRUE-RPC-AUDIT-004.md` | 真实跃升证据 |
| Day 10审计 | `docs/self-audit/DAY10-AGENT-CORE-SELF-AUDIT-001.md` | A级评级确认 |
| 债务历史 | `docs/debt/agent-core-debt-history.md` | 9项已清偿债务 |
| 活跃债务 | `docs/debt/DEBT-ACTIVE-DECLARATION.md` | 4项Phase 5诚实申报 |
| 债务文档 | `docs/debt/DEBT-P0-001.md` | PSK长期管理债务 |
| 债务文档 | `docs/debt/SHELL-FEATURE-DEBT-002.md` | Shell降级功能清单 |
| TODO清收日志 | `TODO-CLEARANCE-LOG.txt` | 清收记录 |
| E2E回归套件 | `tests/e2e/phase1-5-regression/` | 18个月全周期测试 |

---

## 📈 债务清偿关键指标

| 指标 | 清偿前 | 清偿后 | 清收率 |
|:---|:---:|:---:|:---:|
| TODO/FIXME (src) | 1,292 | 10 | 99.2% |
| setTimeout模拟 | 1 | 0 | 100% |
| 硬编码返回值 | 1 | 0 | 100% |
| WebRTC Math.random | 1 | 0 | 100% |
| Shell bash -c | 1 | 0 | 100% |
| codex-twist双轨 | 重复文件 | 0文件 | 100% |
| Phase 7综合 | B+/NoGo | **B+→A-** (返工后Go) | - |
| DEBT-LLM-CLIENT | 未清偿 | **A-清偿** | 164行桥接 |
| 综合评级 | C/D波动 | **A-** | - |

---

*本索引文档与代码同步维护，最后更新于 2026-04-23 (v3.8.0-batch-1 - Phase 7 Debt Clearance + DEBT-LLM-CLIENT 清偿完成)*
