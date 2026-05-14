# Hajimi IDE — Agent 开发指南

> **项目**: Hajimi IDE v1 — 本地优先 AI 智能体 IDE  
> **最后更新**: 2026-05-14  
> **架构版本**: v3.9.0（四层分层架构）  
> **许可证**: Apache-2.0  
> **仓库**: https://github.com/Cognitive-Architect/hajimi-code-cli

---

## 项目概述

Hajimi IDE 是一款基于 **Tauri v2** 的桌面端 AI 智能体 IDE，采用 **Rust 后端 + 纯 HTML/CSS/JS 前端** 技术栈。所有核心功能在本地运行，代码与数据不出境。

核心特性包括：
- **文件树浏览** — 真实目录递归，支持右键菜单
- **代码编辑器** — `contenteditable` + 自定义正则语法高亮（18 种语言）
- **集成终端** — Shell 命令执行，白名单参数化安全
- **全局搜索** — 基于 `grep` 的实时搜索 + Tantivy 16 分片全文搜索
- **Git 面板** — 状态、diff、提交、Smart Commit
- **AI 聊天** — 流式响应，多轮上下文，`/` 命令解析
- **MCP 工具调用** — 15+ 工具真实 RPC
- **LSP 集成** — F12 跳转定义、Shift+F12 查找引用、悬停提示
- **Inline 编辑** — Agent 建议 diff 预览 + Accept/Reject/Selective
- **Command Palette** — `@agent refactor`/`review-pr`/`commit` 全局命令
- **Edit History** — 编辑历史时间线 + Session Replay
- **AST 感知** — Tree-sitter + LSP 上下文注入

---

## 技术栈

### 后端（Rust）
| 技术 | 用途 |
|:---|:---|
| Tauri v2 | 桌面应用框架 |
| Tokio | 异步运行时 |
| 40+ 工具系统 | 文件操作、Git、搜索、构建、LSP、MCP |
| Agent Core | 7 步自主循环 + EditApplier + WorkflowOrchestrator |
| LLM 客户端 | Anthropic Claude / OpenAI GPT-4 / Ollama 本地推理 |
| Tantivy | 16 分片全文搜索 + 向量混合搜索 |
| WASM (wasm-bindgen) | HNSW 向量计算加速 |

### 前端（vanilla HTML/CSS/JS）
- **无** React / Vue / Vite / Webpack
- Tauri v2 JS API (`window.__TAURI__`) 调用后端
- 自定义正则语法高亮（18 种语言）
- CSS 变量主题系统（dark / light / high-contrast，跟随系统）
- 前端源码集中在 `src/interface/web/`（`app.js` + `index.html` + `style.css`）

### 基础设施（Node.js / TypeScript）
- 部分基础组件使用 JavaScript/TypeScript（存储路由、限流器、WASM 桥接）
- MCP 服务器使用 TypeScript 实现（`src/interface/mcp-server/`）
- Node.js >= 18.x，`package.json` 中 `"type": "commonjs"`
- 测试使用 Jest（`npm test`）

---

## 项目结构

项目采用**四层分层架构**，硬性约束为：**下层零依赖上层**。

```
f:\hajimi-code-cli\
├── Cargo.toml              # Rust workspace 根配置（19 个 workspace members）
├── Cargo.lock              # Rust 依赖锁定
├── package.json            # Node.js 依赖（ioredis, level, sql.js, zod）
├── .mcp.json               # MCP 服务器配置
├── .npmrc                  # NPM 严格模式配置
├── .gitignore              # 忽略规则
├── README.md               # 项目 README
├── AGENTS.md               # 本文件
│
├── src/
│   ├── ARCHITECTURE.md     # 四层架构详解 + ADR 记录
│   ├── CONTRIBUTING.md     # 贡献指南与代码审查流程
│   ├── INDEX.md            # 详细模块索引 + 代码统计
│   ├── MEMORY.md           # 数据诚实性规范与债务基线
│   ├── lib.rs              # workspace 根 lib
│   │
│   ├── foundation/         # 地基层 — 零依赖上层（7 模块）
│   │   ├── eventloop/      # 异步事件循环（Rust）
│   │   ├── format/         # 数据格式（.hctx / BLAKE3）
│   │   ├── hash/           # SimHash64 哈希算法（Rust crate）
│   │   ├── network/        # WebSocket 服务器（Rust）
│   │   ├── security/       # 限流（Token Bucket + SQLite）、审计日志
│   │   ├── storage/        # 16 分片 SQLite 存储（JS/TS）
│   │   └── wasm/           # HNSW WASM 加速（Rust + JS 桥接）
│   │
│   ├── engine/             # 引擎层 — 仅依赖 foundation（4 模块）
│   │   ├── llm-core/       # LLM 客户端（Anthropic / OpenAI / Ollama SSE 流式）
│   │   ├── search/         # Tantivy 16 分片搜索索引
│   │   ├── tool-system/    # 40+ 工具 + ToolRegistry + 白名单参数化 Shell
│   │   └── worker/         # 并行/串行执行器
│   │
│   ├── intelligence/       # 智能层 — 依赖 foundation + engine（7 模块）
│   │   ├── agent-core/     # 7 步自主循环 + Swarm + 可插拔治理 + LLM 桥接
│   │   ├── chimera/        # REPL 引擎（ZeroTUI 架构）
│   │   ├── cloud/          # 云端同步（批次同步）
│   │   ├── codex-twist/    # AI 内存管理（5 级架构）
│   │   ├── knowledge/      # ADR + 知识图谱 + GNN
│   │   ├── memory/         # 5 层记忆系统（Session/Auto/Dream/Graph/Cloud）
│   │   └── pgvector/       # PostgreSQL 向量存储
│   │
│   ├── interface/          # 界面层 — 可依赖全下层（3 模块）
│   │   ├── desktop/        # Tauri v2 Rust 后端（38+ 工具注册）
│   │   ├── web/            # 纯 HTML/CSS/JS 前端
│   │   └── mcp-server/     # MCP 真实 RPC 服务器（15 工具，TypeScript）
│   │
│   ├── crates/             # 保留 Rust Crate（hajimi-codex-twist）
│   ├── patches/            # 构建依赖补丁（zstd-sys API 兼容性修复）
│   ├── integration/        # 集成测试 crate
│   └── meta/               # 项目元数据（ADR 工具）
│
├── tests/                  # Node.js / JS 测试
├── scripts/                # 辅助脚本（count-debt.ps1, count-lines.ps1）
├── docs/                   # 文档（API、审计、技术债务、路线图）
│   ├── API.md              # 接口定义
│   ├── debt/               # 技术约束与限制说明（15 份债务文档）
│   └── self-audit/         # 自测报告
├── benches/                # 基准测试
└── templates/              # 模板文件
```

---

## 构建与运行命令

### 环境要求
```bash
cargo --version  # >= 1.78
node --version   # >= 18.x
```

### 安装依赖
```bash
# Node.js 依赖（使用 ci 锁定版本）
npm ci

# Rust 依赖
cargo fetch
```

### 编译检查
```bash
# 全 workspace 编译检查（最常用，验证分层合规）
cargo check --workspace

# 单 crate 检查
cargo check -p intelligence-agent-core
cargo check -p engine-tool-system
cargo check -p engine-search
```

### 运行桌面应用
```bash
# 开发模式（前端自动从 src/interface/web/ 加载，devUrl: http://localhost:3456）
cd src/interface/desktop && cargo tauri dev

# 构建 release（产出平台原生安装包）
cd src/interface/desktop && cargo tauri build
```

### 格式化代码
```bash
cargo fmt
```

---

## 测试命令

### Rust 测试
```bash
# Agent Core 单元测试（lib 内测试，约 89 个）
cargo test -p intelligence-agent-core --lib

# Agent Core 全部测试（含 E2E，约 266 个测试）
cargo test -p intelligence-agent-core

# Shell 安全白名单测试（P0 安全，必须通过）
cargo test -p engine-tool-system -- test_allow_list

# 其他 crate 测试
cargo test -p engine-search
cargo test -p intelligence-knowledge
cargo test -p intelligence-memory

# Agent Core 稳定性测试（100 轮）
cargo test -p intelligence-agent-core test_stability_100_rounds
```

### Node.js / TypeScript 测试
```bash
# Jest 测试
npm test

# TypeScript 编译检查
npx tsc --noEmit

# 前端语法检查
node --check src/interface/web/app.js

# E2E 回归测试
node tests/e2e/phase1-5-regression/full_chain.test.js
```

### 基准测试
```bash
# WASM 性能对比
node tests/wasm-vs-js-v2.bench.js

# HNSW 召回率基准
cargo test -p intelligence-memory -- bench_hnsw
```

---

## 四层架构与分层依赖规则

```
interface ──────┐
                ├──→ intelligence ────┐
                │                      ├──→ engine ────┐
                │                      │               ├──→ foundation
                │                      │               │
                └──────────────────────┴───────────────┘
```

**硬性约束**（违反将导致 `cargo check --workspace` 失败）：
- **Foundation** 零依赖上层
- **Engine** 仅依赖 Foundation
- **Intelligence** 依赖 Foundation + Engine
- **Interface** 可依赖全下层

```rust
// ❌ 错误示例：foundation 依赖上层
use engine::tool_system::Tool;  // 禁止！

// ✅ 正确：engine 仅依赖 foundation
use foundation::storage::ShardRouter;

// ✅ 正确：intelligence 依赖 foundation + engine
use engine::llm_core::LlmClient;
```

---

## 代码风格规范

### 文件命名
| 类型 | 命名风格 | 示例 |
|------|----------|------|
| Rust 实现文件 | snake_case | `agent_loop.rs`, `shell.rs` |
| JS/TS 实现文件 | 小写 + 连字符 | `shard-router.js` |
| 类型定义 | 大驼峰 | `ICrdtEngine.ts` |
| 测试文件 | `*.test.js` / `*_test.rs` | `chunk.test.js` |
| Rust 模块入口 | `mod.rs` 或 `lib.rs` | — |

### 提交规范
```
<类型>(<分层>/<作用域>): <描述>

示例：
feat(engine/tool-system): add new file search tool
fix(foundation/storage): resolve shard routing bug
security(engine/tool-system): harden shell with allow-list
docs(intelligence/knowledge): update architecture description
```

### `unsafe` 规范
- 所有 `unsafe` 块前必须有 `/// # Safety` 注释
- SAFETY 注释必须说明前提条件（指针有效 / 长度正确 / 生命周期）
- **禁止**修改 `unsafe` 块实际逻辑，仅允许添加注释

---

## 核心设计模式

### 1. Tool Trait 标准接口（`src/engine/tool-system/src/mod.rs`）
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
所有 40+ 工具实现此 trait。工具注册在 `src/interface/desktop/src/main.rs` 的 `build_registry()` 函数中，共注册 38+ 工具。

### 2. Agent Core 7 步循环（`src/intelligence/agent-core/agent_loop.rs`）
```
Observe → Retrieve → Plan → Act → Reflect → Store → Decide
```
- **Retrieve**: `SyncMemoryGateway::retrieve_multi` 多层级联检索（Session→Auto→Dream→Graph），带 30s TTL 缓存与 4096 token 溢出保护
- **Act**: `SwarmDelegate::delegate_and_wait()` → Supervisor-Worker 执行
- **Reflect**: `MultiWorkerAggregator` 聚合多 Worker 结果
- **Store**: `sync_with_blackboard` 双向同步 Blackboard ↔ Memory
- **治理**: 5 级审批（Auto/Advisory/Required/Critical/Override），运行时 `set_approval_level()` 调级

### 3. 5 级内存架构（Codex-Twist）
```
Hot:   Focus Memory   — LRU 4K tokens     O(1) ~100ns
Warm:  Working Memory — mmap + zstd 32K   O(log n) ~1μs
Cold:  Archive Memory — LevelDB 1M        O(log n) ~10ms
RAG:   RAG Index      — HNSW 384-dim      O(log n) ~5ms
```

### 4. Shell 白名单参数化（`src/engine/tool-system/src/shell.rs`）
严格白名单：`git`, `cargo`, `npm`, `node`, `python3`, `ls`, `cat`, `echo`, `pwd`, `rustc`, `bash`, `sh`, `pwsh`, `powershell`, `curl`, `wget`, `tar`, `unzip`, `make` 等。
- **禁止** `rm`, `sudo` 等危险命令
- 使用 `Command::new` 参数化执行，**禁止** `bash -c` 拼接
- 元字符过滤：拒绝 `;`, `&`, `|`, `` ` ``, `$`, `(`, `)`, `{`, `}`, `<`, `>`

---

## 安全规范（P0 级别）

### 1. Shell 白名单参数化
- 相关文件：`src/engine/tool-system/src/shell.rs`
- 降级功能清单（管道/重定向/逻辑运算符限制）：`docs/debt/SHELL-FEATURE-DEBT-002.md`

### 2. API Key 安全存储
- OS Keyring 存储（Windows Credential Manager / macOS Keychain / Linux Secret Service）
- `secrecy::SecretString` 内存脱敏
- `providers.json` 仅存元数据，密钥不落磁盘明文
- Workspace 级配置隔离：`<workspace>/.hajimi/providers.json` 覆盖全局

### 3. 配置文件权限
- Unix `0o600` / Windows `icacls` 受限 ACL

### 4. 限流与熔断
- Token Bucket 算法，SQLite 持久化
- Burst 100, Rate 10 req/s
- 熔断器：Failure 50% 触发，Recovery 30s

### 5. 网络安全
- WebRTC 信令认证：CSPRNG + 环境变量 PSK + `timingSafeEqual`
- **禁止**使用 `Math.random()` 生成安全敏感随机数

### 6. 审批策略
- 5 级：`AskBeforeExec` / `AskForDangerous` / `AskOnceThenAuto` / `FullAuto` / `FullDeny`

---

## 关键文件速查

| 功能 | 路径 |
|------|------|
| 工具 Trait 定义 | `src/engine/tool-system/src/mod.rs` |
| Shell 白名单 | `src/engine/tool-system/src/shell.rs` |
| Agent 7 步循环 | `src/intelligence/agent-core/agent_loop.rs` |
| 可插拔治理 | `src/intelligence/agent-core/governance.rs` |
| Swarm 协调 | `src/intelligence/agent-core/swarm.rs` |
| LLM 桥接 | `src/intelligence/agent-core/llm/` |
| EditApplier | `src/intelligence/agent-core/edit_applier.rs` |
| WorkflowOrchestrator | `src/intelligence/agent-core/workflow_orchestrator.rs` |
| Tauri 桌面后端（入口） | `src/interface/desktop/src/main.rs` |
| Tauri 配置 | `src/interface/desktop/tauri.conf.json` |
| Web 前端 | `src/interface/web/app.js` |
| 前端 HTML | `src/interface/web/index.html` |
| 前端样式 | `src/interface/web/style.css` |
| MCP 服务器 | `src/interface/mcp-server/server.ts` |
| Tantivy 搜索索引 | `src/engine/search/src/tantivy_index.rs` |
| LLM 客户端（Anthropic） | `src/engine/llm-core/src/anthropic.rs` |
| LLM 客户端（OpenAI） | `src/engine/llm-core/src/openai.rs` |
| LLM 客户端（Ollama） | `src/engine/llm-core/src/ollama.rs` |
| 存储分片路由 | `src/foundation/storage/shard-router.js` |
| HNSW WASM | `src/foundation/wasm/src/lib.rs` |
| 限流器 | `src/foundation/security/rate-limiter-sqlite-luxury.js` |

---

## Workspace 成员（Cargo.toml）

根 `Cargo.toml` 定义了 19 个 workspace members：
- `src/foundation/hash`
- `src/foundation/eventloop`
- `src/foundation/network`
- `src/foundation/wasm`
- `src/engine/llm-core`
- `src/engine/tool-system`
- `src/engine/worker`
- `src/engine/search`
- `src/intelligence/chimera/chimera-repl`
- `src/intelligence/codex-twist`
- `src/intelligence/memory`
- `src/intelligence/pgvector`
- `src/intelligence/cloud`
- `src/intelligence/knowledge`
- `src/intelligence/agent-core`
- `src/integration`
- `src/interface/desktop`

所有 workspace 依赖在根 `Cargo.toml` 的 `[workspace.dependencies]` 中精确锁定版本（使用 `=` 前缀防止 API 漂移）。

`[patch.crates-io]` 中有一个本地补丁：`zstd-sys`（修复 zstd-safe 6.x experimental API 不匹配）。

---

## 文档索引

| 文档 | 路径 | 说明 |
|------|------|------|
| 架构文档 | `src/ARCHITECTURE.md` | 四层架构详解 + ADR-001~016 |
| 贡献指南 | `src/CONTRIBUTING.md` | 开发规范、代码审查流程 |
| 源代码索引 | `src/INDEX.md` | 详细模块索引 + 代码统计 |
| API 文档 | `docs/API.md` | 接口定义 |
| 活跃技术约束 | `docs/debt/DEBT-ACTIVE-DECLARATION.md` | 4 项活跃约束声明 |
| Shell 功能降级 | `docs/debt/SHELL-FEATURE-DEBT-002.md` | 管道/重定向/逻辑运算符限制 |
| P0 技术约束 | `docs/debt/DEBT-P0-001.md` | PSK 长期管理等 |

---

## 贡献检查清单

提交 PR 前请确认：

- [ ] **分层合规**: 无下层依赖上层
- [ ] **编译通过**: `cargo check --workspace` 0 errors
- [ ] **测试通过**: 新增功能有测试覆盖（cargo-discoverable）
- [ ] **unsafe 合规**: 所有 `unsafe` 块前有 `/// # Safety` 注释
- [ ] **文档更新**: `src/INDEX.md` 和 `src/ARCHITECTURE.md` 已同步
- [ ] **提交规范**: `<type>(<layer>/<scope>): <description>`
- [ ] **P0 安全合规**:
  - [ ] Shell 工具: 白名单参数化（无 `bash -c`）
  - [ ] 网络服务: CSPRNG（无 `Math.random`）
  - [ ] 加密操作: `timingSafeEqual`（防时序攻击）
- [ ] **代码真实性**:
  - [ ] 无 `setTimeout` 模拟延迟
  - [ ] 无硬编码"成功"返回值
  - [ ] 无 `mock`/`simulation` 字样（测试除外）
  - [ ] 真实 RPC 调用（非包装转发）
- [ ] **TODO 管理**: 新增 TODO 必须带 deadline（<90 天），src 目录 TODO 总数 ≤ 20

---

*本文件与代码同步维护。如有架构变更、新增模块或安全策略调整，请务必同步更新本文档。*
