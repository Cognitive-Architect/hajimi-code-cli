# HAJIMI V3 源代码索引

> **文档版本**: v3.8.0  
> **最后更新**: 2026-04-23  
> **代码总行数**: ~42,100行自有代码（不含依赖，含agent-core ~2,540行源文件 / ~2,920行总计）  
> **架构**: 四层分层（Foundation/Engine/Intelligence/Interface）  
> **当前状态**: ✅ Agent Core 55测试通过，0编译error，unsafe SAFETY 100%覆盖

---

## 📁 目录总览（四层架构）

```
src/
├── crates/              # 保留的 Rust Crates（根目录）
│   ├── evm-bench-adapter/   # EVM检测适配器
│   └── hajimi-codex-twist/  # AI内存核心 (Rust workspace兼容)
│
├── foundation/          # 地基层 - 零依赖（18模块）
│   ├── api/             # REST API 服务器
│   ├── bench/           # 性能基准测试
│   ├── compression/     # 上下文压缩（micro/auto/compact/mod）
│   ├── db/              # PostgreSQL连接池
│   ├── disk/            # 磁盘管理（ENOSPC处理）
│   ├── eventloop/       # 异步事件循环 (Rust)
│   ├── format/          # 数据格式（.hctx/BLAKE3）
│   ├── hash/            # 哈希算法（SimHash64）
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
├── engine/              # 引擎层 - 仅依赖foundation（4模块）
│   ├── llm-core/        # LLM客户端（Anthropic/OpenAI/Ollama）
│   ├── search/          # 搜索索引（Tantivy 16分片）⭐
│   ├── tool-system/     # 工具系统（40+工具/白名单参数化）⭐
│   └── worker/          # 工作线程池
│
├── intelligence/        # 智能层 - 依赖foundation+engine（8模块）
│   ├── agent-core/      # 自主Agent系统（7步循环/Swarm/可插拔治理/LLM桥接）⭐
│   │   └── llm/         #   LLM适配器桥接（PlannerLlmBridge + ReflectorLlmBridge）
│   ├── chimera/         # Chimera REPL引擎（Rust）⭐
│   ├── cloud/           # 云端同步（批次同步）
│   ├── codex-twist/     # AI内存管理（5级架构/双轨清理完成）⭐
│   ├── integration/     # 集成模块
│   ├── knowledge/       # 知识图谱（ADR/GNN/知识库）⭐
│   ├── memory/          # 5层记忆系统⭐
│   ├── pgvector/        # PostgreSQL向量扩展
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
**代码规模**: ~2,540行源文件 / ~2,920行总计（19源文件 + 3测试文件）  
**状态**: 98 测试全部通过（55 lib + 43 E2E），0编译warning  
**约束**: 7项技术约束与待办已记录（见下方）

| 文件 | 功能 | 行数 |
|------|------|:----:|
| `lib.rs` | 模块入口 + 公共类型 | 180 |
| `agent_loop.rs` | 7步自主循环 | 224 |
| `agent_loop_builder.rs` | AgentLoop构建器 | 51 |
| `agent_loop_tests.rs` | AgentLoop单元测试 | 93 |
| `swarm.rs` | Swarm协调器（Supervisor-Worker模式） | 251 |
| `governance.rs` | 可插拔治理（5级审批策略 + 投票机制） | 239 |
| `orchestrator.rs` | Agent编排器 | 305 |
| `planner.rs` | 任务规划器 | 167 |
| `reflector.rs` | 反思优化器 | 277 |
| `blackboard.rs` | 共享黑板状态 | 112 |
| `checkpoint.rs` | 检查点恢复 | 83 |
| `events.rs` | 事件系统 | 115 |
| `tools.rs` | 工具集成 | 171 |
| `degrade.rs` | 降级策略 | 13 |
| `ports.rs` | 端口抽象 | 46 |
| `minimal_agent.rs` | 最小Agent实现 | 18 |
| `llm/bridge.rs` | **LLM 适配器桥接**（PlannerLlmBridge + ReflectorLlmBridge）⭐ | 161 |
| `llm/mod.rs` | LLM 模块入口 | 3 |
| `mod.rs` | 公共API导出（含约束声明） | 34 |

**测试套件**（cargo-discoverable，位于 `tests/` 目录）：

| 测试文件 | 测试数 | 关键测试 |
|----------|:------:|----------|
| `tests/agent_core_e2e.rs` | 25 | `test_stability_100_rounds`, `test_governance_rejection`, `test_swarm_delegate` |
| `tests/autonomous_goal_test.rs` | 8 | `test_completion_rate`, `bench_agent_loop` |
| `tests/integration.rs` | 10 | `test_worker_crash_isolation`, `test_loop_timeout_handling` |
| **lib内测试** | **55** | 各模块单元测试（含 llm/bridge.rs 5个桥接测试）⭐ |
| **E2E总计** | **43** | `cargo test -p intelligence-agent-core` |

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
| RETRIEVE-PHASE5 | 进行中 | Graph/Dream层记忆检索待全面集成 |
| WORKER-TOOL-EXECUTION | 进行中 | Worker执行结果回调机制待完善 |
| LEAK-TEST-PHASE5 | 进行中 | AgentLoop资源泄漏测试待重写 |
| W5-CONTEXT-DEEP | 待办 | tree-sitter AST 感知上下文 |
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
| Foundation | 18 | TS/JS/Rust | 稳定 ✅ |
| Engine | 4 | TS/Rust | P0安全 ✅ |
| Intelligence | 8 | Rust/TS | 稳定 ✅ |
| Interface | 3 | TS/Rust | 稳定 ✅ |
| **总计** | **33** | - | **v3.8** |

**按语言统计**:
| 语言 | 文件数 | 行数 | 主要分布 |
|:---|:---:|:---:|:---|
| Rust | 193 | ~20,906 | engine/, intelligence/, foundation/wasm/ |
| JavaScript | 82 | ~17,899 | foundation/, interface/ |
| TypeScript | 42 | ~3,295 | foundation/, interface/ |
| TSX | 0 | 0 | - |
| **总计** | **317** | **~42,100** | - |

**TODO统计**:
| 范围 | 当前数量 | 说明 |
|:---|:---:|:---|
| src目录 | 10 | 持续维护中 |
| engine核心层 | 0 | - |
| intelligence层 | 5 | 4项约束声明 |
| agent-core | 4活跃 | 历史总数13项，9项已处理 |

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
5. `foundation/compression/mod.rs` - 压缩模块

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
| **E2E回归** | `tests/e2e/phase1-5-regression/full_chain.test.js` |
| **技术约束文档** | `docs/debt/DEBT-P0-001.md` |
| **技术约束文档** | `docs/debt/SHELL-FEATURE-DEBT-002.md` |
| **历史约束记录** | `docs/debt/agent-core-debt-history.md` |
| **活跃约束声明** | `docs/debt/DEBT-ACTIVE-DECLARATION.md` |


---

## 🗺️ 目录迁移对照

| 原路径 | 新路径 | 层级 |
|:---|:---|:---:|
| `api/` | `foundation/api/` | Foundation |
| `compression/` | `foundation/compression/` | Foundation |
| `db/` | `foundation/db/` | Foundation |
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

| E2E回归套件 | `tests/e2e/phase1-5-regression/` | 18个月全周期测试 |

---

## 📈 项目改进指标

| 指标 | 改进前 | 当前 | 改进率 |
|:---|:---:|:---:|:---:|
| setTimeout模拟 | 1 | 0 | 100% |
| 硬编码返回值 | 1 | 0 | 100% |
| Shell bash -c | 1 | 0 | 100% |
| 综合状态 | 波动 | **稳定** | - |

---

*本索引文档与代码同步维护，最后更新于 2026-04-23*
