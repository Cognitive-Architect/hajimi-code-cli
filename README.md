# Hajimi IDE — Local-First AI Agent IDE

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.78%2B-000000?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-v2-24C8D8?style=flat-square&logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/Frontend-vanilla%20JS%2FCSS-3178c6?style=flat-square" alt="Frontend">
  <img src="https://img.shields.io/badge/架构-v3.9.0-success?style=flat-square" alt="Version">
</p>

<p align="center">
  <strong>本地优先 AI 智能体 IDE，四层分层架构，零外部依赖运行</strong>
</p>

---

## 概述

Hajimi IDE 是一款基于 **Tauri v2** 的桌面端 AI 智能体 IDE，采用 **Rust 后端 + 纯 HTML/CSS/JS 前端** 技术栈。所有核心功能在本地运行，代码与数据不出境。

---

## 核心功能

| 功能 | 状态 | 说明 |
|:---|:---:|:---|
| 文件树 | ✅ | 真实目录递归浏览，支持右键菜单 |
| 代码编辑器 | ✅ | `contenteditable` + 自定义语法高亮（18 种语言） |
| 集成终端 | ✅ | Shell 命令执行，白名单参数化安全 |
| 全局搜索 | ✅ | 基于 `grep` 的实时搜索 + Tantivy 16 分片全文搜索 |
| Git 面板 | ✅ | 状态、diff、提交、Smart Commit |
| AI 聊天 | ✅ | 流式响应，多轮上下文，`/` 命令解析，精确 Token 统计 |
| MCP 工具调用 | ✅ | 15+ 工具真实 RPC |
| LSP 集成 | ✅ | F12 跳转定义、Shift+F12 查找引用、悬停提示 |
| 主题系统 | ✅ | dark / light / high-contrast，跟随系统 |
| 扩展商店 | ✅ | 安装/卸载状态管理 |
| 拖拽调整面板 | ✅ | sidebar / panel 大小持久化 |
| 设置持久化 | ✅ | localStorage 存储 |
| **Inline 编辑** | ✅ | Agent 建议 diff 预览，Accept All / Reject / Selective |
| **Command Palette** | ✅ | `@agent refactor`/`review-pr`/`commit` 全局命令 |
| **Edit History** | ✅ | 编辑历史时间线，Session Replay 回放 |
| **Smart Commit** | ✅ | Conventional commit 启发式 + PR 描述自动生成 |
| **AST 感知** | ✅ | Tree-sitter + LSP 上下文注入，精准编辑建议 |
| **Thinking UI** | ✅ | 可折叠 Thinking 区块、流式状态机、操作摘要条（Scheme C） |
| **精确 Token 统计** | ✅ | tiktoken-rs + usage 字段解析，前端实时显示 + 累计消耗持久化 |
| **Context Window Manager** | ✅ | 优先级分层裁剪（P0–P4），8K token 预算管理，Planner/Reflector 集成 |
| **ACT 多步链式执行** | ✅ | 指纹去重 + 重试机制 + 微反思错误分析，feature-gate 控制 |
| **Reflector V1** | ✅ | 反思优化器 feature-gate，Blackboard Stop-Loss 协议 |

---

## 技术栈

### 后端（Rust）
- **Tauri v2** — 桌面应用框架
- **Tokio** — 异步运行时
- **工具系统** — 40+ 工具实现（文件操作、Git、搜索、构建、LSP、MCP）
- **Agent Core** — 7 步自主循环 + Swarm + 可插拔治理 + LLM 桥接
- **LLM 客户端** — Anthropic Claude / OpenAI GPT-4 / Ollama 本地推理
- **EditApplier** — hunk-level diff，冲突检测，原子写入，真正 undo
- **ContextWindowManager** — P0–P4 优先级分层 token 预算管理
- **Tantivy** — 16 分片全文搜索 + 向量混合搜索
- **WASM (wasm-bindgen)** — HNSW 向量计算加速

### 前端（vanilla HTML/CSS/JS）
- 无 React / Vue / Vite / Webpack
- Tauri v2 JS API (`window.__TAURI__`) 调用后端
- 自定义正则语法高亮（18 种语言）
- CSS 变量主题系统（dark / light / high-contrast）
- 精确 Token 显示（`🔄 xx.x% | ↑ xxxxx | ↓ xxxx`），点击切换累计

---

## 架构

```
┌─────────────────────────────────────────┐
│  Interface 层（界面层）                   │
│  ├── desktop/     Tauri v2 Rust 后端    │
│  ├── web/         纯 HTML/CSS/JS 前端   │
│  └── mcp-server/  MCP 真实 RPC          │
├─────────────────────────────────────────┤
│  Intelligence 层（智能层）               │
│  ├── agent-core/  7步循环 + Swarm       │
│  │   ├── ContextWindowManager  ⭐新增   │
│  │   ├── ActExecutor（多步链式）⭐新增  │
│  │   ├── Reflector V1          ⭐新增   │
│  │   └── llm/    PlannerBridge + ReflectorBridge │
│  ├── chimera/     REPL 引擎             │
│  ├── memory/      5 层记忆系统          │
│  ├── knowledge/   ADR + 知识图谱        │
│  ├── codex-twist/ AI 内存管理           │
│  ├── cloud/       云端同步              │
│  └── pgvector/    PG 向量存储           │
├─────────────────────────────────────────┤
│  Engine 层（引擎层）                     │
│  ├── llm-core/    LLM 客户端            │
│  ├── search/      Tantivy 16 分片搜索   │
│  ├── tool-system/ 40+ 工具 + 白名单     │
│  └── worker/      并行/串行执行器       │
├─────────────────────────────────────────┤
│  Foundation 层（地基层）                 │
│  ├── storage/     16 分片 SQLite        │
│  ├── network/     WebSocket 服务器      │
│  ├── wasm/        HNSW WASM 加速        │
│  ├── security/    限流 + 审计日志       │
│  ├── eventloop/   异步事件循环          │
│  ├── format/      .hctx 数据格式        │
│  └── hash/        SimHash64             │
└─────────────────────────────────────────┘
```

**分层规则**: Foundation 零依赖上层 → Engine 仅依赖 Foundation → Intelligence 依赖 Foundation + Engine → Interface 可依赖全下层。

---

## 快速开始

### 环境要求
```bash
cargo --version  # >= 1.78
node --version   # >= 18.x
```

### 安装依赖
```bash
npm ci
cargo fetch
```

### 运行桌面应用
```bash
# 开发模式
cd src/interface/desktop && cargo tauri dev

# 构建
cd src/interface/desktop && cargo tauri build
```

### 运行测试
```bash
# Rust workspace 编译检查
cargo check --workspace

# Agent Core 单元测试
cargo test -p intelligence-agent-core --lib

# Agent Core 全部测试（含 E2E）
cargo test -p intelligence-agent-core

# Shell 安全测试（P0 必须通过）
cargo test -p engine-tool-system -- test_allow_list

# 前端语法检查
node --check src/interface/web/app.js
```

---

## 目录结构

```
src/
├── foundation/     # 地基层 — 7 模块
├── engine/         # 引擎层 — 4 模块
├── intelligence/   # 智能层 — 7 模块
│   └── agent-core/ # Agent 系统（含 ContextWindowManager / ActExecutor / Reflector V1）
├── interface/      # 界面层 — 3 模块
│   ├── desktop/    # Tauri v2 Rust 后端
│   ├── web/        # 纯 HTML/CSS/JS 前端
│   └── mcp-server/ # MCP 真实 RPC
├── patches/        # 构建依赖补丁 (zstd-sys)
├── crates/         # 保留 Rust Crate (hajimi-codex-twist)
├── integration/    # 集成测试 crate
├── meta/           # 项目元数据
└── lib.rs          # 根 lib
```

---

## 代码统计

| 语言 | 文件数 | 行数 | 主要分布 |
|:---|:---:|:---:|:---|
| Rust | 234 | ~48,492 | engine/, intelligence/, foundation/ |
| JavaScript | 20 | ~15,296 | interface/web/ |
| TypeScript | 28 | ~2,263 | interface/mcp-server/, foundation/ |
| HTML | 1 | ~1,563 | interface/web/ |
| CSS | 1 | ~7,322 | interface/web/ |
| **总计** | **284** | **~74,936** | - |

> 统计口径：仅 `src/` 目录，排除 `target/`、`node_modules/`、`dist/`；含注释与空行；实测 2026-05-15。

---

## 关键里程碑（Phase 1–5）

| 阶段 | 内容 | 状态 |
|:---|:---|:---:|
| Phase 1 | SyncMemoryGateway + AgentLoop 多层检索集成 | ✅ |
| Phase 2 | WorkerCallback 回调 + Swarm 执行闭环 + E2E 验证 | ✅ |
| Phase 3a/3b | EpisodicMemory JSONL 持久化 + fastembed + HNSW 索引 | ✅ |
| Phase 4 | EditApplier + AST 上下文 + Desktop Inline UI + Git Workflow | ✅ |
| **P0 Context** | 多轮 LLM 接口 + 前端对话状态 + MemoryGateway 激活 | ✅ |
| **Scheme B** | tiktoken-rs 精确 Token 统计，三 Provider 全覆盖 | ✅ |
| **P1 Token Tracker** | TokenUsageTracker 全链路集成 + 前端持久化 | ✅ |
| **Thinking UI** | Scheme C 完成：流式状态机 + 操作摘要条 + Diff 预览 | ✅ |
| **Phase 5 Day 1** | ContextWindowManager + PlannerLlmBridge 集成 + ReflectorLlmBridge 集成 | ✅ |
| **Phase 5 Day 2** | ActExecutor 多步链式 + 指纹去重 + 重试机制 + `is_act_toolcall_v1_enabled` | ✅ |
| **Phase 5 Day 3** | Reflector V1 feature-gate + Blackboard Stop-Loss + DTO 验证测试 | ✅ |

---

## 关键文档

| 文档 | 路径 | 说明 |
|:---|:---|:---|
| 架构文档 | `src/ARCHITECTURE.md` | 四层架构详解 + ADR 记录 |
| 贡献指南 | `src/CONTRIBUTING.md` | 开发规范与代码审查流程 |
| 源代码索引 | `src/INDEX.md` | 详细模块索引 + 代码统计 |
| 债务索引 | `docs/debt/INDEX.md` | 活跃债务 + 历史清偿记录 |
| API 文档 | `docs/API.md` | 接口定义 |
| P0 安全债务 | `docs/debt/DEBT-P0-001.md` | WebRTC PSK 长期管理 |
| Shell 功能降级 | `docs/debt/SHELL-FEATURE-DEBT-002.md` | 管道/重定向限制声明 |
| Slash 命令说明 | `docs/debt/02-slash-command-palette.md` | 可用命令列表与面板实现方案 |

---

## 配置模型（Providers Sidebar）

Hajimi IDE 通过 **Providers Sidebar** 管理 LLM 提供商配置，支持多 Provider 切换与自定义 OpenAI-Compatible 端点。

### 打开 Providers Sidebar

1. 点击左侧边栏 **⚙️ 设置** 图标，或按 `Ctrl+Shift+P` 打开 Command Palette 输入 `/providers`
2. 在 Sidebar 中选择 **Providers** 标签页

### 添加 Provider（3 步完成）

1. **点击「+ 添加自定义 Provider」**
2. **填写字段**：
   - **名称**：自定义显示名称（如 `My-GPT-4`）
   - **Base URL**：API 端点地址（如 `https://api.openai.com/v1`）
   - **API Key**：从提供商后台获取的密钥
   - **默认模型**：聊天时默认使用的模型 ID（如 `gpt-4`）
3. **点击「验证」**按钮 — 后端将在 5 秒内尝试连通性检测，成功后自动保存

### 内置 Provider

| Provider | 类型 | 说明 |
|:---|:---|:---|
| `ollama` | 本地 | 本地 Ollama 推理，无需 API Key |
| `anthropic` | 云端 | Claude 系列模型 |
| `openai` | 云端 | GPT 系列模型 |
| **自定义** | OpenAI-Compatible | 任意兼容 OpenAI API 格式的端点 |

### 安全提示

- **API Key 存储**：所有密钥通过操作系统钥匙串（Windows Credential Manager / macOS Keychain / Linux Secret Service）加密存储，明文不落磁盘
- **配置文件隔离**：Workspace 级配置存储在 `<workspace>/.hajimi/providers.json`（仅元数据，无密钥），可覆盖全局设置

---

## 安全

- **Shell 白名单**: 38 个允许命令，参数化执行，无 `bash -c`，无元字符注入
- **API Key 存储**: OS Keyring + `secrecy::SecretString` 内存脱敏
- **限流**: Token Bucket，SQLite 持久化，Burst 100 / Rate 10 req/s
- **配置文件权限**: Unix `0o600` / Windows 受限 ACL
- **治理策略**: 5 级审批（Auto / Advisory / Required / Critical / Override）
- **WebRTC PSK**: CSPRNG + timingSafeEqual（详见 `docs/debt/DEBT-P0-001.md`）

---

## 许可证

Apache-2.0

---

*最后更新: 2026-05-15 — Hajimi IDE v1 (Phase 1–5 Day 3) 活跃开发中 🚀*
