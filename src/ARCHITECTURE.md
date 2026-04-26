# HAJIMI V3 架构文档

> **文档版本**: v3.8.0
> **架构风格**: 四层分层架构 + 本地优先 + Tauri v2 桌面应用
> **核心原则**: 下层零依赖上层、Git历史完整、最小侵入
> **当前状态**: ✅ Agent Core 55测试全部通过，0编译error，unsafe SAFETY注释100%覆盖

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
│  │  Integration│  │  pgvector   │  │                     │ │
│  │(integration)│  │ (pgvector/) │  │                     │ │
│  │ 第三方适配  │  │ PG向量存储  │  │                     │ │
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
│  │   Storage  │ │   Network  │ │     DB     │ │  Security  │ │   Event   │  │
│  │ (storage/) │ │(network/)  │ │   (db/)    │ │(security/) │ │ Loop      │  │
│  │ LevelDB    │ │ WebSocket  │ │PostgreSQL  │ │限流/日志   │ │(eventloop)│  │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └───────────┘  │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌───────────┐  │
│  │   Disk     │ │  Format    │ │    WASM    │ │   Tests    │ │   Utils   │  │
│  │  (disk/)   │ │ (format/)  │ │  (wasm/)   │ │(test/tests)│ │ (utils/)  │  │
│  │ 磁盘管理   │ │ .hctx格式  │ │ HNSW WASM  │ │ 单元/集成  │ │ 通用工具  │  │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └───────────┘  │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────────┐  │
│  │   Bench    │ │ Middleware │ │ Migration  │ │    API     │ │ Compression │  │
│  │ (bench/)   │ │(middleware)│ │(migration/)│ │  (api/)    │ │(compression)│  │
│  │ 性能基准   │ │ 限流中间件 │ │ 数据迁移   │ │ REST API   │ │  压缩算法   │  │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └─────────────┘  │
│  ┌─ scripts ─┐ ┌─ hash ────┐                                                │
│  │(scripts/) │ │  (hash/)  │                                                │
│  │ 工具脚本   │ │ SimHash64 │                                                │
│  └───────────┘ └───────────┘                                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 📁 目录结构详解

### 1. Foundation 层（地基层）
**原则**: 零外部依赖，提供基础设施

| 目录 | 功能 | 关键技术 | 状态 |
|:---|:---|:---|:---:|
| `api/` | REST API 服务器 | Express.js, WebSocket | ✅ 稳定 |
| `bench/` | 性能基准测试 | EVM检测流水线, 压力测试 | ✅ 稳定 |
| `compression/` | 上下文压缩 | micro/auto/compact/mod (Rust) | ✅ 稳定 |
| `db/` | 数据库连接池 | PostgreSQL | ✅ 稳定 |
| `disk/` | 磁盘管理 | ENOSPC处理, 块缓存 | ✅ 稳定 |
| `eventloop/` | 事件循环 | Rust异步运行时 | ✅ 稳定 |
| `format/` | 数据格式 | .hctx 格式, BLAKE3校验 | ✅ 稳定 |
| `hash/` | 哈希算法 | SimHash64, 指纹去重 | ✅ 稳定 |
| `middleware/` | 中间件 | 限流, 错误处理 | ✅ 稳定 |
| `migration/` | 数据迁移 | 版本检测, 迁移脚本 | ✅ 稳定 |
| `network/` | 网络服务 | WebSocket服务器 (原ws_server) | ✅ PSK认证 |
| `scripts/` | 工具脚本 | 构建, 安装, 迁移 | ✅ 安全改造 |
| `security/` | 安全组件 | 限流器, 安全审计与日志 | ✅ 稳定 |
| `storage/` | 存储系统 | LevelDB, 16分片SQLite | ✅ 稳定 |
| `test/` | 单元测试 | 测试工具, Mock | ✅ 稳定 |
| `tests/` | 集成/E2E测试 | WASM, EVM测试 | ✅ 稳定 |
| `utils/` | 通用工具 | SimHash64, Logger | ✅ 8处引用 |
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
| `agent-core/` | 自主Agent系统 | 7步循环, Swarm, 可插拔治理, LLM桥接 ⭐ | ✅ 稳定 |
| `chimera/` | REPL引擎 | ZeroTUI, EventLoop | ✅ 稳定 |
| `cloud/` | 云端同步 | 批次同步 | ✅ 稳定 |
| `codex-twist/` | AI内存管理 | 5级内存架构 | ✅ 双轨清理 |
| `integration/` | 集成模块 | 第三方适配 | ✅ 稳定 |
| `knowledge/` | 知识图谱 | ADR, GNN, 实体关系, SimHash-64 ⭐ | ✅ 稳定 |
| `memory/` | 5层记忆系统 | Session/Auto/Dream/Graph/Cloud | ✅ 稳定 |
| `pgvector/` | PostgreSQL向量 | 向量存储与检索 | ✅ 稳定 |

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
`AgentLoop { blackboard, planner, governance, swarm, tool_registry }`
循环: Observe → Retrieve → Plan → Act → Reflect → Store → Decide。
治理: 5级审批（Auto/Advisory/Required/Critical/Override）。Swarm: Supervisor-Worker 模式。

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
| Agent Core E2E | 90 passed | cargo-discoverable | ✅ |
| Agent Core 编译 | 0 warnings | cargo check | ✅ |

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
├── foundation/          # 地基层 (17模块)
│   ├── api/, bench/, compression/, db/, disk/
│   ├── eventloop/, format/, hash/, middleware/, migration/
│   ├── network/, scripts/, security/, storage/
│   ├── test/, tests/, utils/, wasm/
│   └── ...
├── intelligence/        # 智能层 (8模块)
│   ├── agent-core/      # 自主Agent系统 (7步循环/Swarm/治理/LLM桥接) ⭐
│   ├── chimera/         # REPL引擎
│   ├── cloud/           # 云端同步
│   ├── codex-twist/     # AI内存 (双轨清理完成)
│   ├── integration/     # 集成模块
│   ├── knowledge/       # 知识图谱
│   ├── memory/          # 5层记忆
│   └── pgvector/        # PG向量
└── interface/           # 界面层 (3模块)
    ├── mcp-server/      # MCP服务器 (真实RPC)
    ├── web/             # Web界面 (Tauri v2 前端)
    └── desktop/         # 桌面后端 (Tauri v2 Rust 后端)
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
| ADR-009 | 数据验证机制 (ID-261验证器) | ✅ | tools/data-validator.js |
| ADR-010 | Shell参数化白名单 (消除bash -c) | ✅ | engine/tool-system/shell.rs |
| ADR-011 | Tauri v2 桌面应用架构 | ✅ | src/interface/desktop/ |
| ADR-012 | 工具系统Channel流式传输 | ✅ | engine/tool-system/ |

---

## 🔗 关联文档

| 文档 | 路径 | 说明 |
|:---|:---|:---|
| 源代码索引 | `src/INDEX.md` | 详细文件索引 |
| 贡献指南 | `src/CONTRIBUTING.md` | 开发指南 |
| 质量保障报告 | `audit report/8week/` | 历史质量确认 |
| 技术文档 | `docs/debt/` | 技术约束与限制说明 |
| E2E回归 | `tests/e2e/phase1-5-regression/` | 18个月全周期测试 |

---

*本架构文档与代码同步维护，最后更新于 2026-04-23*
