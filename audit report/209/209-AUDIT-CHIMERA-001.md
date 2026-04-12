# 209-AUDIT-CHIMERA-001 建设性审计报告

**审计日期**: 2026-03-29  
**审计官**: 审计喵（本地代码侦察模式）  
**审计范围**: Chimera 奇美拉架构验证（Codex CLI ↔ Hajimi v3.8.0 双向桥接可行性）  
**审计链**: PROGRESS-AUDIT-002(S+级) → 本审计（架构验证态）

---

## 审计结论

| 项目 | 结果 |
|:---|:---|
| **评级** | **B级**（良好，有小障碍，可控） |
| **状态** | **有条件 Go**（KP-001需剥离工作，KP-003技术路径需调整） |
| **核心发现** | Codex TUI与UI强耦合需剥离；Hajimi napi-rs非C ABI需桥接层；事件循环兼容 |

**关键判定**:
- **KP-001**: ⚠️ **B级** - TUI与终端强耦合，需手动剥离REPL逻辑（预估2-3天）
- **KP-002**: ✅ **A级** - 上下文纯内存+协议流，可直接替换为Hajimi FFI
- **KP-003**: ⚠️ **B级** - napi-rs非C ABI，需手写Rust→Rust桥接（预估1-2天）
- **KP-004**: ✅ **A级** - tokio与napi-rs async兼容，无事件循环冲突

---

## 进度报告（KP分项评级）

| KP | 评级 | 关键证据 | 影响 |
|:---|:---:|:---|:---|
| **KP-001** REPL可剥离性 | **B** | `tui/src/app.rs:100+` 强依赖`crossterm`/`ratatui` | 需重构剥离，2-3天 |
| **KP-002** 存储机制 | **A** | `protocol/src/items.rs:22` `TurnItem`纯内存，无SQLite | 直接替换，零迁移成本 |
| **KP-003** FFI绑定 | **B** | `ffi.rs:2` 使用`#[napi]`非`extern "C"` | 需Rust→Rust桥接层，1-2天 |
| **KP-004** 事件循环 | **A** | `ffi.rs:255` `pub async fn`支持tokio | 直接使用，无冲突 |

---

## 关键疑问回答（Q1-Q4）

### Q1（KP-001）：REPL主循环入口在哪里？是否与UI强耦合？

**审计发现**:

**文件**: `codex-twist/codex-rs/tui/src/app.rs` (Line 1-100+)

```rust
// App结构体强依赖TUI框架
pub struct App {
    chat_widget: ChatWidget,           // ratatui组件
    tui: Tui,                          // 终端UI实例
    app_event_tx: AppEventSender,      // 事件通道
    // ... 100+行TUI状态
}

// 主事件循环（Line 500+）
impl App {
    async fn run_event_loop(&mut self) -> Result<()> {
        // 使用crossterm读取键盘/鼠标输入
        // 使用ratatui渲染界面
        // 与终端强耦合
    }
}
```

**耦合度分析**:
- ✅ **好消息**: Codex有清晰的协议层（`codex_protocol::TurnItem`），业务逻辑与UI分离
- ⚠️ **坏消息**: `App`结构体直接实例化`ChatWidget`/`Tui`，无`trait IOHandler`注入点
- ⚠️ **剥离成本**: 需手动提取`App`中的业务逻辑到`ChimeraRepl`结构体

**结论**: ⚠️ **B级** - 需2-3天重构剥离REPL逻辑

---

### Q2（KP-002）：上下文存储机制是什么？能否被Hajimi替换？

**审计发现**:

**文件**: `codex-twist/codex-rs/protocol/src/items.rs` (Line 22)

```rust
// TurnItem: 纯内存枚举，无持久化逻辑
#[derive(Debug, Clone, Deserialize, Serialize, TS, JsonSchema)]
#[serde(tag = "type")]
pub enum TurnItem {
    UserMessage(UserMessageItem),
    AgentMessage(AgentMessageItem),
    Plan(PlanItem),
    // ...
}

// ThreadEventStore: 内存缓冲（app.rs内部）
#[derive(Debug)]
struct ThreadEventStore {
    buffer: VecDeque<Event>,  // 纯内存
    // ...
}
```

**存储机制**:
- ✅ **无SQLite**: 搜索全代码库，无`rusqlite`依赖
- ✅ **无本地文件**: 上下文仅存内存，云端同步通过OpenAI API
- ✅ **协议驱动**: `TurnItem`可序列化为JSON，直接映射到Hajimi的Turn结构

**替换路径**:
```rust
// Codex原流程: UserInput → TurnItem → Event → OpenAI API
// Chimera流程: UserInput → TurnItem → Hajimi FFI memory_put → Archive
```

**结论**: ✅ **A级** - 纯内存存储，直接替换为零迁移成本

---

### Q3（KP-003）：Hajimi FFI导出方式？Rust如何接入？

**审计发现**:

**文件**: `crates/hajimi-codex-twist/src/ffi.rs` (Line 1-5)

```rust
//! FFI绑定层 - napi-rs实现
#![cfg(feature = "napi")]
use napi::bindgen_prelude::*;
use napi_derive::napi;

// 导出示例（Line 111）
#[napi]
pub fn create_thread(name: String, ...) -> Result<ThreadHandle> {
    // ...
}
```

**绑定方式分析**:
- ⚠️ **非C ABI**: 使用`#[napi]`宏（Node-API），非`extern "C"`
- ⚠️ **无.h头文件**: napi-rs生成TypeScript定义，无C头文件
- ✅ **Rust→Rust可行**: 可直接依赖`hajimi-codex-twist` crate，绕过FFI

**推荐桥接方案**:
```rust
// 方案A: 直接crate依赖（推荐）
// chimera/Cargo.toml
[dependencies]
hajimi-codex-twist = { path = "../crates/hajimi-codex-twist" }

// chimera/src/main.rs
use hajimi_codex_twist::memory::MemoryGateway;
// 直接使用，无需FFI转换
```

**结论**: ⚠️ **B级** - 需调整技术路径（crate依赖替代C FFI），工作量1-2天

---

### Q4（KP-004）：`thread_turn`同步/异步？事件循环如何协同？

**审计发现**:

**文件**: `crates/hajimi-codex-twist/src/ffi.rs` (Line 123, 140, 255)

```rust
// create_turn: 同步导出（napi-rs内部处理）
#[napi]
pub fn create_turn(thread_id: String, prompt: String) -> Result<TurnHandle> {
    // ...
}

// memory_put: 异步导出
#[napi]
pub async fn memory_put(...) -> Result<()> {
    // ...
}

// memory_get: 异步导出
#[napi]
pub async fn memory_get(...) -> Result<Option<String>> {
    // ...
}
```

**事件循环兼容性**:
- ✅ **tokio兼容**: napi-rs底层使用tokio runtime，与Codex一致
- ✅ **async/await**: 可直接在Codex的async上下文中调用
- ✅ **无阻塞风险**: `spawn_blocking`已用于CPU密集型操作（zstd压缩）

**协同方案**:
```rust
// Codex的tokio runtime中直接调用
async fn handle_user_input(input: String) {
    // 无需spawn_blocking，napi-rs自动处理
    let turn = create_turn(thread_id, input).await?;
    // ...
}
```

**结论**: ✅ **A级** - tokio与napi-rs兼容，无事件循环冲突

---

## 验证结果（V1-V4）

| 验证ID | 验证内容 | 结果 | 证据 |
|:---|:---|:---:|:---|
| **V1** | REPL主循环入口 | ✅ | `tui/src/app.rs:100+` `struct App` + `run_event_loop` |
| **V2** | Context定义 | ✅ | `protocol/src/items.rs:22` `pub enum TurnItem` |
| **V3** | FFI导出方式 | ✅ | `ffi.rs:2` `#[napi]`宏，非`extern "C"` |
| **V4** | 异步语义 | ✅ | `ffi.rs:255` `pub async fn memory_put` |

---

## 问题与建议

### 短期（立即处理）

**Issue 1: KP-001 REPL剥离（P0关键路径）**

**方案**: 创建`chimera-repl` crate，提取Codex业务逻辑

```rust
// chimera-repl/src/lib.rs
pub struct ChimeraRepl {
    memory_gateway: MemoryGateway,  // Hajimi
    event_tx: mpsc::Sender<ReplEvent>,
}

impl ChimeraRepl {
    pub async fn run(&mut self, input: impl AsyncRead, output: impl AsyncWrite) -> Result<()> {
        // 无TUI依赖，纯业务逻辑
    }
}
```

**工作量**: 2-3天  
**优先级**: P0（阻塞P1-P3）

---

### 中期（P1-P2）

**Issue 2: KP-003 Rust→Rust桥接（技术路径调整）**

**推荐方案**: 放弃C FFI，改用crate依赖

```toml
# chimera/Cargo.toml
[dependencies]
codex_protocol = { path = "../codex-twist/codex-rs/protocol" }
hajimi_codex_twist = { path = "../crates/hajimi-codex-twist" }
```

**优势**:
- 零FFI开销
- 类型安全（编译期检查）
- 直接调用`MemoryGateway`方法

**工作量**: 1-2天重构依赖

---

### 长期（P3）

**Issue 3: P2P同步觉醒（可选）**

一旦基础桥接完成，可启用Hajimi的Yjs同步：
```rust
// 对话状态序列化为.hctx
let context = thread.to_hctx();
// 通过Yjs CRDT同步到多设备
yjs_doc.update(context);
```

---

## 落地可执行路径

### 修订路线图（基于审计结果）

| 阶段 | 目标 | 产物 | 依赖 |
|:---|:---|:---|:---|
| **P0: REPL剥离**（3-4天） | 提取Codex业务逻辑，移除TUI依赖 | `chimera-repl` crate | KP-001 |
| **P1: Crate桥接**（1-2天） | 改用Rust→Rust直接依赖 | `chimera`可执行文件 | KP-003调整 |
| **P2: 记忆嫁接**（3-5天） | 替换为Hajimi FFI调用 | 对话自动落盘Archive | KP-002 |
| **P3: P2P觉醒**（5-7天） | 接入Yjs同步 | 跨设备对话续接 | P2完成 |

**总工期**: 12-18天（vs 原规划10-14天，+2-4天用于KP-001剥离）

---

## 压力怪评语（B级认证版）

> **"还行吧，有点耦合，但可解。"**（B级）

> "Codex的TUI是个'胖客户端'——业务逻辑和界面搅在一起，在`app.rs`里养了100多行TUI状态。
> 
> 但好消息是：
> - 上下文纯内存，没搞SQLite那种硬绑定（KP-002 A级）
> - 事件循环tokio和napi-rs天然兼容（KP-004 A级）
> - 协议层`TurnItem`设计清晰，可复用
> 
> **坏消息就一个**：Hajimi用的是napi-rs（Node-API），不是C FFI。想直接`extern "C"`调用？门儿没有。
> 
> **解决方案**：别折腾FFI了，直接crate依赖，`use hajimi_codex_twist::memory::MemoryGateway`，完事儿。
> 
> **B级，有条件Go！** 先把REPL从TUI里剥出来，后面就是按部就班的嫁接工程。

---

## 归档建议

- **审计报告归档**: `audit report/209/209-AUDIT-CHIMERA-001.md` ✅ 本文件
- **关联状态**: 
  - PROGRESS-AUDIT-002（S+级代码认证）
  - ID-209（Chimera架构验证态）
- **建议行动**:
  1. ⚠️ **立即**: 启动P0 REPL剥离（关键路径）
  2. ✅ **本周**: 调整KP-003技术路径（crate依赖替代C FFI）
  3. ✅ **下一步**: P1完成后进入P2记忆嫁接

---

## 审计链连续性

```
PROGRESS-AUDIT-002(S+级，Hajimi代码完成)
    ↓
209-AUDIT-CHIMERA-001(B级，Codex↔Hajimi桥接验证) ← 当前
    ↓
修订路线图（+2-4天用于REPL剥离）
    ↓
210-P0-REPL-STRIP（开发派单）
    ↓
Chimera v0.1 MVP
```

**Ouroboros第25次迭代，Codex CLI实地侦察完成，4大卡点全部回答，B级认证，有条件Go！** ☝️🐍♾️🔍🐱

---

*审计完成时间: 2026-03-29*  
*审计官: 审计喵*  
*标准: ID-175建设性审计模板*  
*评级: B级（有条件Go，REPL剥离后升A级）*
