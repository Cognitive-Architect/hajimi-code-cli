# HAJIMI V3 贡献指南

> **目标**: 帮助开发者快速理解代码结构并参与开发  
> **适用对象**: 核心开发者、代码审查员、功能扩展者  
> **架构版本**: v3.8.0（四层分层架构）  
> **最后更新**: 2026-04-26

---

## 🚀 快速开始

### 环境要求
```bash
# Node.js
node --version  # >= 18.x

# Rust
cargo --version  # >= 1.75

```

### 安装依赖
```bash
# Node.js 依赖 (使用ci而非install，锁定版本)
npm ci

# Rust 依赖
cargo fetch
```

### 运行测试
```bash
# Rust 编译检查（workspace）
cargo check --workspace

# Agent Core单元测试
cargo test -p intelligence-agent-core --lib  # 55 tests passed, 0 failed

# Agent Core E2E测试
cargo test -p intelligence-agent-core

# Shell安全测试 (白名单校验)
cargo test -p engine-tool-system -- test_allow_list
```

---

## 📂 四层架构开发指南

### 架构总览
```
interface/      # 界面层 - 可依赖全下层
    ↓
intelligence/   # 智能层 - 依赖 foundation + engine
    ↓
engine/         # 引擎层 - 仅依赖 foundation
    ↓
foundation/     # 地基层 - 零依赖
```

**核心规则**: 下层禁止依赖上层

### 目录命名规范

#### 分层目录（固定）
| 层级 | 目录 | 模块数 | 说明 |
|:---:|:---|:---:|:---|
| Foundation | `foundation/*/` | 7 | 基础设施模块 |
| Engine | `engine/*/` | 4 | 核心引擎模块 |
| Intelligence | `intelligence/*/` | 8 | 智能模块 |
| Interface | `interface/*/` | 3 | 界面模块 |

#### 文件命名规范
| 类型 | 命名 | 示例 |
|------|------|------|
| 实现文件 | 小写+连字符 | `shard-router.js` |
| 类型定义 | 大驼峰 | `ICrdtEngine.ts` |
| 测试文件 | `*.test.js` | `chunk.test.js` |
| E2E 测试 | `*.e2e.js` | `webrtc-handshake.e2e.js` |
| 基准测试 | `*.bench.js` | `sab-overhead.bench.js` |
| Rust模块 | `*.rs` | `mod.rs`, `lib.rs` |

---

## 🎯 开发工作流

### 1. 添加新功能

**步骤 1: 确定分层**
```
新功能类型 → 对应分层 → 具体目录
├── 存储/网络/安全等基础设施 → foundation/
│   ├── 存储相关 → foundation/storage/
│   ├── 网络相关 → foundation/network/
│   ├── 事件循环 → foundation/eventloop/
│   └── 安全相关 → foundation/security/
│
├── LLM/搜索/工具/线程 → engine/
│   ├── LLM 客户端 → engine/llm-core/
│   ├── 搜索索引 → engine/search/ ⭐
│   ├── 工具系统 → engine/tool-system/ (白名单参数化) ⭐
│   └── 工作线程 → engine/worker/
│
├── AI/记忆/知识/Agent → intelligence/
│   ├── 自主Agent系统 → intelligence/agent-core/ (7步循环/Swarm) ⭐
│   ├── REPL 引擎 → intelligence/chimera/
│   ├── 内存管理 → intelligence/codex-twist/
│   ├── 记忆系统 → intelligence/memory/
│   ├── 知识图谱 → intelligence/knowledge/ ⭐
│
└── UI/接口 → interface/
    ├── MCP 服务器 → interface/mcp-server/ (真实RPC)
    ├── Web 界面 → interface/web/ (Tauri v2 前端)
    └── 桌面后端 → interface/desktop/ (Tauri v2 Rust 后端)
```

**步骤 2: 检查依赖规则**
```rust
// ❌ 错误：foundation 依赖上层
// foundation/foo/src/lib.rs
use engine::tool_system::Tool;  // 禁止！

// ✅ 正确：engine 仅依赖 foundation
// engine/foo/src/lib.rs
use foundation::storage::ShardRouter;  // 允许

// ✅ 正确：intelligence 可依赖 foundation + engine
// intelligence/foo/src/lib.rs
use foundation::db::PgPool;        // 允许
use engine::llm_core::LlmClient;   // 允许
use engine::search::TantivyIndex;  // 允许 ⭐

// ✅ 正确：interface 可依赖全下层
// interface/foo/src/lib.rs
use foundation::storage::*;        // 允许
use engine::tool_system::*;        // 允许
use intelligence::memory::*;       // 允许
use intelligence::knowledge::*;    // 允许 ⭐
```

**步骤 3: 实现接口（含P0安全规范）**
```rust
// engine/tool-system/src/my_tool.rs
use async_trait::async_trait;
use crate::tool::{Tool, ToolArgs, ToolOutput, ToolError};

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "My tool description" }
    
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        // P0安全: 如果是shell工具，必须使用白名单参数化
        // 见 engine/tool-system/src/shell.rs 示例
        Ok(ToolOutput::success("done"))
    }
}
```

**步骤 4: 添加测试**
```rust
// engine/tool-system/src/my_tool.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_my_tool() {
        let tool = MyTool;
        let result = tool.execute(ToolArgs::default()).await;
        assert!(result.is_ok());
    }
    
    // P0安全: shell工具必须测试白名单
    #[tokio::test]
    async fn test_shell_allow_list() {
        // 测试允许命令通过
        // 测试禁止命令被拒绝
    }
}
```

**步骤 5: 更新文档**
- 修改 `src/INDEX.md` 添加新模块描述
- 修改 `src/ARCHITECTURE.md` 更新架构图
- 如有接口变更，更新相关 trait 文档
- **建议**: 记录任何TODO/FIXME到技术文档（90天内处理）

---

### 2. 修改现有代码

**检查清单**:
- [ ] 是否违反分层依赖规则（下层依赖上层？）
- [ ] 是否影响 `engine/tool-system/src/mod.rs` Tool Trait
- [ ] 是否影响 `intelligence/knowledge/src/adr_index.rs` 知识库
- [ ] 是否更新相关测试
- [ ] 是否更新文档注释
- [ ] 是否通过 `cargo check --workspace` 或 `tsc --noEmit`
- [ ] **P0安全**: 是否涉及shell执行？（必须使用白名单参数化）
- [ ] **真实性**: 是否有模拟/硬编码？（必须真实实现）

**提交规范**:
```bash
# 格式: <type>(<scope>): <description>
# scope 使用分层前缀: foundation/, engine/, intelligence/, interface/

# 示例:
feat(engine/tool-system): add new file search tool
fix(foundation/storage): resolve shard routing bug
docs(intelligence/knowledge): update 5-tier architecture description
feat(intelligence/knowledge): add ADR pattern extraction
feat(interface/web): improve chat UI streaming
security(engine/tool-system): harden shell with allow-list ⭐
```

---

### 3. 代码审查流程

**触发条件**:
- 新模块超过 100 行
- 修改核心接口（如 Tool Trait）
- 新增跨层依赖
- 修改 foundation 层基础组件
- 新增功能需通过验证

**审查步骤**:
1. 工程师编写自测报告
2. 审查员阅读项目规范了解标准
3. 审查员执行审查，输出审查报告
4. 结论：通过 / 需修改 / 不通过
5. 数据一致性验证 + TODO合规检查

---

## 🔍 代码阅读指南

### 按分层阅读

**1. Foundation 层（基础设施）**:
1. `foundation/storage/shard-router.js` - 16分片路由（最简单核心逻辑）
2. `foundation/wasm/src/lib.rs` - WASM HNSW（Rust/JS 边界）
3. `foundation/security/rate-limiter-sqlite-luxury.js` - 限流器（安全）
4. `foundation/network/src/lib.rs` - WebSocket 服务器（网络）
5. `foundation/eventloop/src/lib.rs` - 事件循环（Rust）

**2. Engine 层（核心引擎）**:
1. `engine/tool-system/src/mod.rs` - Tool Trait（核心接口）
2. `engine/tool-system/src/shell.rs` - **Shell白名单参数化** ⭐ (P0安全)
3. `engine/search/src/tantivy_index.rs` - 搜索索引（219行，Tantivy 16分片）
4. `engine/llm-core/src/anthropic.rs` - LLM 客户端（外部调用）
5. `engine/worker/src/parallel.rs` - 并行执行（并发）

**3. Intelligence 层（智能系统）**:
1. `intelligence/agent-core/agent_loop.rs` - **AgentLoop 7步循环** ⭐（Day 10）
2. `intelligence/agent-core/governance.rs` - **可插拔治理** ⭐（5级审批策略）
3. `intelligence/agent-core/swarm.rs` - **Swarm协调** ⭐（Supervisor-Worker）
4. `intelligence/chimera/chimera-repl/src/repl.rs` - REPL 引擎（核心入口）
5. `intelligence/memory/src/session.rs` - Session 记忆（5层之一）
6. `intelligence/knowledge/src/adr_index.rs` - ADR索引（185行，SimHash-64）
7. `intelligence/knowledge/src/search.rs` - ADR搜索（35行）
8. `intelligence/agent-core/src/llm/bridge.rs` - **LLM 适配器桥接** ⭐（engine-llm-core → planner/reflector，零侵入）

**4. Interface 层（用户界面）**:
1. `interface/mcp-server/server.ts` - MCP 服务器（协议）
2. `interface/mcp-server/capabilities/tools.ts` - **真实RPC工具调用** ⭐
3. `interface/web/app.js` - Tauri v2 Web 前端（聊天界面）
4. `interface/desktop/src/main.rs` - Tauri v2 Rust 后端（38+工具注册）

### 按角色阅读

**新开发者**:
1. 阅读 `src/INDEX.md` - 了解全貌
2. 阅读 `src/ARCHITECTURE.md` - 理解四层架构
3. 从 `foundation/storage/shard-router.js` 开始 - 最简单核心逻辑
4. 看 `engine/tool-system/src/shell.rs` - **P0安全白名单示例**
5. 看 `intelligence/memory/src/session.rs` - 理解记忆系统
6. 看 `src/interface/desktop/src/main.rs` - Tauri 后端命令集成

**代码审查员**:
1. 必读 `Agent prompt/Mike.md` - 项目规范
2. 必读 `Agent prompt/PROJECT-CONTEXT.md` - 项目背景
3. 检查分层依赖：确保无下层依赖上层
4. 重点关注：
   - `engine/tool-system/src/shell.rs` - **P0安全白名单完整性**
   - `foundation/security/rate-limiter-*.js` - 安全策略
   - `intelligence/memory/src/` - 记忆系统一致性
   - `src/interface/desktop/src/main.rs` - Tauri 命令安全（run_command 白名单）

**性能优化者**:
1. 查看 `foundation/wasm/` - WASM 边界跨越性能
2. 查看 `foundation/storage/batch-writer-*.js` - 写入性能
3. 查看 `foundation/wasm/` - WASM 边界跨越
4. 查看 `engine/worker/src/` - 线程池利用率
5. 查看 `engine/search/src/tantivy_index.rs` - 搜索性能

---

## 🔗 关键代码路径

### 查询流程（Query Flow）
```
User Input
    ↓
interface/web/app.js
    ↓
engine/tool-system/src/registry.rs  (ToolRegistry)
    ↓
engine/llm-core/src/anthropic.rs    (LLM API)
    ↓
intelligence/chimera/chimera-repl/src/repl.rs    (REPL 引擎)
    ↓
intelligence/memory/src/session.rs  (记忆存储)
    ↓
foundation/storage/shard-router.js  (持久化)
```

---

## 🐛 调试技巧

### Rust 调试
```bash
# 检查分层依赖（确保无下层依赖上层）
cargo check --workspace

# 详细错误
cargo check 2>&1 | head -50

# 格式化代码
cargo fmt

# 运行特定 crate 测试
cargo test -p engine-tool-system
cargo test -p engine-tool-system -- test_allow_list  # P0安全测试 ⭐
cargo test -p engine-search
cargo test -p intelligence-knowledge
cargo test -p intelligence-memory
cargo test -p intelligence-agent-core  # Day 10: 90 tests passed ⭐

# 查看依赖树
cargo tree -p engine-tool-system
```

### Node.js 调试
```bash
# TypeScript 编译检查
npx tsc --noEmit

# 内存分析
node --max-old-space-size=512 --inspect src/foundation/tests/xxx.test.js

# 性能分析
npx clinic flame -- node src/foundation/tests/bench/xxx.bench.js

# E2E回归
node tests/e2e/phase1-5-regression/full_chain.test.js

# Tauri 桌面应用调试
# 1. cd src/interface/desktop && cargo tauri dev
# 2. 前端自动从 src/interface/web/ 加载
```

### 常见问题

**问题 1**: `Error: Cannot find module './codex-twist'`
- **解决**: 项目使用 workspace 路径，检查 `Cargo.toml` 中 `[workspace.members]`

**问题 2**: `cargo check` 失败，分层依赖错误
- **解决**: 检查是否在下层（如 foundation）引用了上层（如 engine）的模块

**问题 3**: `error: no matching package named 'hajimi-core'`
- **解决**: `hajimi-core` 已拆分，改用 `engine-tool-system` 或 `intelligence-chimera`



**问题 6**: Shell工具执行被拒绝
- **解决**: 检查命令是否在白名单中（见 `engine/tool-system/src/shell.rs:18-22`）
- 降级功能清单: `docs/debt/SHELL-FEATURE-DEBT-002.md`

---

## 📈 性能优化指南

### 1. 向量检索优化
```javascript
// 使用批量搜索（减少 WASM 边界跨越）
const results = index.searchBatch(queries, queryCount, k);

// 使用零拷贝（避免内存分配）
const results = index.searchBatchZeroCopy(float32Array, dim, k);
```

### 2. 存储优化
```javascript
// 使用批量写入
const batchWriter = new BatchWriter(db);
await batchWriter.write(chunks);

// 使用 WAL 模式
const db = new SQLite(':memory:', { wal: true });

// 使用16分片路由
const shard = shardRouter.route(key); // SimHash-64 高 8bit
```

### 3. Tantivy搜索优化
```rust
// engine/search/src/tantivy_index.rs
// 16分片并行搜索
let results = (0..NUM_SHARDS)
    .into_par_iter()
    .map(|shard| search_shard(shard, query))
    .flatten()
    .collect::<Vec<_>>();
```

### 5. Tool 系统优化
```rust
// 使用并行执行（独立工具）
use engine::worker::ParallelExecutor;

let executor = ParallelExecutor::new();
executor.spawn_batch(tools).await;

// 使用串行执行（依赖工具）
use engine::worker::SerialExecutor;

let executor = SerialExecutor::new();
executor.chain(tools).await;
```

### 6. 知识库ADR索引优化
```rust
// 使用SimHash-64预分片
let shard_id = get_shard_id(&adr.id);  // O(1)
shards[shard_id].push(adr);            // O(1)

// 关键词索引
keyword_index.entry(keyword)
    .or_insert_with(Vec::new)
    .push(adr.id);
```

---

## 📝 文档维护

### 必须同步更新的文档
| 变更类型 | 更新文档 |
|----------|----------|
| 新增模块 | `src/INDEX.md` |
| 架构变更 | `src/ARCHITECTURE.md` |
| 接口变更 | 相关接口注释 + `src/INDEX.md` |
| 分层移动 | `src/INDEX.md` 迁移对照表 |
| 性能数据 | `src/ARCHITECTURE.md` 性能表格 |
| 质量报告 | `audit report/XX/` |
| **P0安全变更** | `docs/debt/` 技术约束文档更新 |

### 文档模板

**模块描述模板**:
```markdown
### module-name/
**分层**: Engine  
**技术栈**: Rust  
**代码规模**: ~XXX行  
**状态**: 稳定/开发中/已废弃

| 文件 | 功能 |
|------|------|
| `src/lib.rs` | 模块入口 |
| `src/xxx.rs` | 一句话描述 |

**依赖**:
- 上层: intelligence/, interface/
- 下层: foundation/

**关键特性**:
- 特性 1
- 特性 2

**已知限制** (如适用):
- 降级功能: docs/debt/XXX.md
- TODO/FIXME: Y个 (90天内处理)
```

---

## 🎓 学习资源

### 核心技术
- **SimHash**: [Google SimHash 论文](https://static.googleusercontent.com/media/research.google.com/zh-CN//pubs/archive/33026.pdf)
- **HNSW**: [Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs](https://arxiv.org/abs/1603.09320)
- **Tantivy**: [Tantivy Documentation](https://tantivy-search.github.io/)
- **MCP**: [Model Context Protocol Spec](https://modelcontextprotocol.io/)

### 项目特定
- **质量保障历史**: 见 `audit report/`
- **技术白皮书**: `README-PHASE5-FINAL.md`
- **技术约束文档**: `docs/debt/DEBT-P0-001.md`, `docs/debt/SHELL-FEATURE-DEBT-002.md`

---

## ✅ 贡献检查清单

提交 PR 前请确认：

- [ ] **分层合规**: 无下层依赖上层
- [ ] **编译通过**: `cargo check --workspace` 0 errors
- [ ] **测试通过**: 新增功能有测试覆盖
- [ ] **unsafe合规** ⭐:
  - [ ] 所有`unsafe`块前有`/// # Safety`注释
  - [ ] SAFETY注释说明前提条件（指针有效/长度正确/生命周期）
  - [ ] 不修改unsafe块实际逻辑，仅添加注释
- [ ] **文档更新**: `INDEX.md` 和 `ARCHITECTURE.md` 已同步
- [ ] **提交规范**: `<type>(<layer>/<scope>): <description>`
- [ ] **历史完整**: 使用 `git mv` 移动文件（如适用）
- [ ] **P0安全合规** ⭐:
  - [ ] Shell工具: 白名单参数化（无bash -c）
  - [ ] 网络服务: CSPRNG（无Math.random）
  - [ ] 加密操作: timingSafeEqual（防时序攻击）
- [ ] **代码真实性** ⭐:
  - [ ] 无`setTimeout`模拟延迟
  - [ ] 无硬编码"成功"返回值
  - [ ] 无`mock`/`simulation`字样（测试除外）
  - [ ] 真实RPC调用（非包装转发）
  - [ ] 测试必须cargo-discoverable（放在 `tests/` 或 `#[cfg(test)]` 内）
  - [ ] 无虚构测试结果（必须实际运行验证）
  - [ ] 自测报告必须真实且独立存在
- [ ] **TODO管理** ⭐:
  - [ ] 新增TODO必须带deadline (<90天)
  - [ ] 或转换为技术约束文档
  - [ ] src目录TODO总数 ≤ 20

## 📋 文档准确性校验规范

> **原则**: 所有文档中的量化数据必须与代码实际状态一致，禁止估算替代实测。

### 校验清单（每次更新文档时执行）

**1. 代码行数核对**
```bash
# 统计agent-core实际行数
find src/intelligence/agent-core -name '*.rs' | xargs wc -l
# 源文件（不含tests/）
find src/intelligence/agent-core -name '*.rs' ! -path '*/tests/*' | xargs wc -l
# 测试文件
find src/intelligence/agent-core/tests -name '*.rs' | xargs wc -l
```

**2. 测试数量核对**
```bash
# 实际运行测试并记录结果
cargo test -p intelligence-agent-core --quiet 2>&1 | tee test-output.txt
```

**3. 编译warning核对**
```bash
# agent-core模块自身warning（排除其他crate）
cargo check -p intelligence-agent-core 2>&1 | grep -c "warning:"
```

### 代码-文档同步规则

| 数据项 | 文档位置 | 校验频率 | 允许偏差 |
|:---|:---|:---|:---|
| 代码行数 | INDEX.md | 每次大版本更新 | <5% |
| 测试通过数 | INDEX.md / CONTRIBUTING.md | 每次发布 | 0（必须完全匹配） |
| 编译warning | 自测报告 | 每次提交 | 0（必须完全匹配） |

---

## 🎯 项目当前状态

| 指标 | 值 | 状态 |
|:---|:---:|:---:|
| 代码总行数 | ~42,948 | ✅ |
| Rust文件数 | 181 | ✅ |
| src目录TODO | 10 | ✅ |
| unsafe SAFETY覆盖率 | 100% (13/13) | ✅ |
| E2E测试 | Agent Core | ✅ |
| Agent Core测试 (lib) | 55 passed | ✅ |
| Agent Core测试 (E2E) | 90 passed | ✅ |
| Agent Core编译 | 0 warnings | ✅ |
| P0安全 | 4/4完成 | ✅ |

**近期改进成果**:
- ✅ TODO从 1,292 降至 10
- ✅ setTimeout模拟延迟已移除
- ✅ 硬编码返回值已移除

- ✅ Shell白名单参数化 完成
- ✅ Agent Core单元测试 完成（55测试/0failed）
- ✅ engine-tool-system warning清零 完成
- ✅ unsafe SAFETY注释100%覆盖 完成
- ✅ VSCode Sidebar 56→7对齐 完成

**已知限制与待办**:
- WebRTC PSK长期管理（KMS/Vault/Rotation）
- Shell复杂功能降级清单（管道/重定向/逻辑运算符）
- Graph/Dream层记忆检索待集成
- Worker执行结果回调机制待完善
- AgentLoop资源泄漏测试待重写
- tree-sitter AST 感知上下文
- MCP SSE/WebSocket 真实流式
- 视频导览

**后续待办** 🚀

---

*本指南与代码同步维护，最后更新于 2026-04-26*
