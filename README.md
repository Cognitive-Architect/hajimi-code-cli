# Hajimi IDE — Local-First AI Agent IDE

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.78%2B-000000?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Tauri-v2-24C8D8?style=flat-square&logo=tauri" alt="Tauri">
  <img src="https://img.shields.io/badge/Frontend-vanilla%20JS%2FCSS-3178c6?style=flat-square" alt="Frontend">
</p>

<p align="center">
  <strong>本地优先 AI 智能体 IDE，四层分层架构，零外部依赖运行</strong>
</p>

---

## 概述

Hajimi IDE 是一款基于 **Tauri v2** 的桌面端 AI 智能体 IDE，采用 **Rust 后端 + 纯 HTML/CSS/JS 前端** 技术栈。所有核心功能在本地运行，代码与数据不出境。

## 核心功能

| 功能 | 状态 | 说明 |
|:---|:---:|:---|
| 文件树 | ✅ | 真实目录递归浏览，支持右键菜单 |
| 代码编辑器 | ✅ | `contenteditable` + 自定义语法高亮（18种语言） |
| 集成终端 | ✅ | Shell 命令执行，白名单参数化安全 |
| 全局搜索 | ✅ | 基于 `grep` 的实时搜索 |
| Git 面板 | ✅ | 状态、diff、提交 |
| AI 聊天 | ✅ | 流式响应，上下文文件管理，`/` 命令解析 |
| MCP 工具调用 | ✅ | 15+ 工具真实 RPC |
| LSP 集成 | ✅ | F12 跳转定义、Shift+F12 查找引用、悬停提示 |
| 主题系统 | ✅ | dark / light / high-contrast，跟随系统 |
| 扩展商店 | ✅ | 安装/卸载状态管理 |
| 拖拽调整面板 | ✅ | sidebar / panel 大小持久化 |
| 设置持久化 | ✅ | localStorage 存储 |

## 技术栈

### 后端（Rust）
- **Tauri v2** — 桌面应用框架
- **Tokio** — 异步运行时
- **工具系统** — 40+ 工具实现（文件操作、Git、搜索、构建、LSP、MCP）
- **Agent Core** — 7步自主循环（Observe→Retrieve→Plan→Act→Reflect→Store→Decide）
- **LLM 客户端** — Anthropic Claude / OpenAI GPT-4 / Ollama 本地推理

### 前端（vanilla HTML/CSS/JS）
- 无 React / Vue / Vite / Webpack
- Tauri v2 JS API (`window.__TAURI__`) 调用后端
- 自定义正则语法高亮（18种语言）
- CSS 变量主题系统

## 架构

```
┌─────────────────────────────────────────┐
│  Interface 层（界面层）                   │
│  ├── desktop/  Tauri v2 Rust 后端        │
│  ├── web/      纯 HTML/CSS/JS 前端       │
│  └── mcp-server/  MCP 真实 RPC           │
├─────────────────────────────────────────┤
│  Intelligence 层（智能层）                │
│  ├── agent-core/   7步自主循环 + Swarm   │
│  ├── chimera/      REPL 引擎             │
│  ├── memory/       5层记忆系统           │
│  ├── knowledge/    ADR + 知识图谱        │
│  ├── codex-twist/  AI 内存管理           │
│  ├── cloud/        云端同步              │
│  ├── integration/  第三方适配            │
│  └── pgvector/     PG 向量存储           │
├─────────────────────────────────────────┤
│  Engine 层（引擎层）                      │
│  ├── llm-core/     LLM 客户端            │
│  ├── search/       Tantivy 16分片搜索    │
│  ├── tool-system/  40+ 工具 + 白名单     │
│  └── worker/       并行/串行执行器       │
├─────────────────────────────────────────┤
│  Foundation 层（地基层）                  │
│  ├── storage/      16分片 SQLite         │
│  ├── network/      WebSocket 服务器      │
│  ├── wasm/         HNSW WASM 加速        │
│  ├── security/     限流 + 审计日志       │
│  ├── compression/  上下文压缩            │
│  ├── api/          REST API 服务器       │
│  ├── db/           PostgreSQL 连接池     │
│  ├── eventloop/    异步事件循环          │
│  ├── format/       .hctx 数据格式        │
│  ├── hash/         SimHash64             │
│  ├── middleware/   Express/Koa 中间件    │
│  ├── migration/    数据库迁移            │
│  ├── scripts/      构建/安装脚本         │
│  ├── test/         单元测试辅助          │
│  ├── tests/        集成/E2E 测试         │
│  ├── utils/        通用工具              │
│  └── disk/         磁盘管理              │
└─────────────────────────────────────────┘
```

**分层规则**: Foundation 零依赖上层 → Engine 仅依赖 Foundation → Intelligence 依赖 Foundation + Engine → Interface 可依赖全下层。

## 快速开始

### 环境要求
```bash
# Rust
cargo --version  # >= 1.78

# Node.js
node --version   # >= 18.x
```

### 安装依赖
```bash
# Node.js 依赖
npm ci

# Rust 依赖
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

# Agent Core 单元测试（55 passed）
cargo test -p intelligence-agent-core --lib

# Agent Core E2E 测试（43 passed）
cargo test -p intelligence-agent-core

# Shell 安全测试
cargo test -p engine-tool-system -- test_allow_list

# 前端语法检查
node --check src/interface/web/app.js
```

## 目录结构

```
src/
├── foundation/     # 地基层 - 18模块
├── engine/         # 引擎层 - 4模块
├── intelligence/   # 智能层 - 8模块
├── interface/      # 界面层 - 3模块
│   ├── desktop/    # Tauri v2 Rust 后端
│   ├── web/        # 纯 HTML/CSS/JS 前端
│   └── mcp-server/ # MCP 真实 RPC
├── crates/         # 保留 Rust Crates
├── integration/    # 集成测试 crate
├── meta/           # 项目元数据
└── lib.rs          # 根 lib
```

## 代码统计

| 语言 | 文件数 | 行数 | 主要分布 |
|:---|:---:|:---:|:---|
| Rust | 193 | ~20,906 | engine/, intelligence/, foundation/wasm/ |
| JavaScript | 82 | ~17,899 | foundation/, interface/ |
| TypeScript | 42 | ~3,295 | foundation/, interface/ |
| **总计** | **317** | **~42,100** | - |

## 关键文档

| 文档 | 路径 | 说明 |
|:---|:---|:---|
| 架构文档 | `src/ARCHITECTURE.md` | 四层架构详解 |
| 贡献指南 | `src/CONTRIBUTING.md` | 开发规范与流程 |
| 源代码索引 | `src/INDEX.md` | 详细模块索引 |
| API 文档 | `docs/API.md` | 接口定义 |

## 安全

- **Shell 白名单**: 38 个允许命令，参数化执行，无 `bash -c`
- **API Key 存储**: OS Keyring + `secrecy::SecretString` 内存脱敏
- **限流**: Token Bucket，SQLite 持久化
- **配置文件权限**: Unix `0o600` / Windows 受限 ACL

## 许可证

Apache-2.0

---

*最后更新: 2026-04-23*
