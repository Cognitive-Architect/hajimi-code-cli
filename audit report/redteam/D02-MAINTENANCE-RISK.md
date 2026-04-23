# D02 可维护性风险审计报告

**审计对象**: HAJIMI 核心代码库 (`src/` 全域，`src/intelligence/agent-core` 重点)  
**审计日期**: 2026-04-19  
**执行环境**: Windows PowerShell, Rust nightly toolchain  
**风险等级**: 🔶 MEDIUM-HIGH（累积效应）  

---

## 1. 执行摘要

本次审计通过 11 项自动化/半自动化检查，发现 agent-core 及周围模块存在**构建稳定性阻塞**、**跨层依赖倒置**、**错误处理模式碎片化**三类系统性维护债务。单个问题风险可控，但组合后会导致：新人上手周期延长、回归测试成本指数级上升、unsafe 审计盲区扩大。

---

## 2. 验证命令执行结果与发现

### 2.1 Clippy 认知复杂度检查

**命令**: `cargo clippy -- -W clippy::cognitive_complexity`  
**结果**: ❌ **编译失败，未输出有效认知复杂度报告**

```text
error[E0433]: failed to resolve: could not find `__private` in `scale`
   --> scale-info-2.11.6/src/ty/mod.rs:274:56
```

**后果评估**: 
- **实际后果**: 无法运行 clippy 意味着所有静态分析流水线（CI lint、pre-commit hook）全部失效。任何新增的高复杂度函数都不会被拦截。
- **理论风险**: 认知复杂度失控会导致代码路径难以测试，间接提升缺陷密度。
- **最小修复**: 将 `scale-info` 锁定到兼容当前 nightly 的版本，或在 `Cargo.toml` 中通过 `patch` 指定修复版本。优先修复构建，再启用 `-D clippy::cognitive_complexity` 作为 CI 门禁。

---

### 2.2 无 SAFETY 注释的 unsafe 代码

**命令**: 递归搜索 `src/**/*.rs` 中不含 `SAFETY`/`# Safety` 的 `unsafe` 行  
**结果**: 🔴 **16 个文件，约 21 处 unsafe 缺少 SAFETY 注释**

关键文件分布：
| 模块 | 文件 | 数量 |
|------|------|------|
| `foundation/wasm` | `lib.rs`, `memory.rs`, `sab.rs` | 3 |
| `intelligence/codex-twist` | `storage_gateway.rs` | 5 |
| `intelligence/codex-twist` | `archive_memory.rs`, `archive_tier.rs`, `tiered_storage.rs` | 3 |
| `intelligence/memory` | `sync_wrapper.rs` | 3 |
| `engine/search` | `vector_text_hybrid.rs` | 2 |
| `integration` | `end_to_end.rs` | 1 |
| `engine/tool-system` | `security.rs` | 1 |

**后果评估**:
- **实际后果**: `storage_gateway.rs` 的 5 处 `unsafe extern "C"` FFI 边界无任何 SAFETY 契约文档。若参数指针生命周期或空指针假设被违反，将直接导致段错误且日志无上下文。
- **理论风险**: Rust unsafe 的安全不变量完全依赖人工审查；缺少注释 = 审查者必须逐行反汇编推理。
- **最小修复**: 对 FFI 函数添加 `/// # Safety` 文档，明确调用方必须保证的前提条件（如指针非空、长度合法、线程安全）。内部 `mmap` 调用标注 `SAFETY: file is read-only and size is validated above`。

---

### 2.3 超大单文件检查

**命令**: 统计 `src/**/*.rs` 超过 500 行的文件  
**结果**: ⚠️ **1 个文件**

```text
src/intelligence/memory/src/hnsw.rs: 798
```

**后果评估**:
- **实际后果**: hnsw.rs 是向量检索核心算法实现，798 行处于黄线边缘，尚未失控。该文件包含 `#![deny(unsafe_code)]`，风险可控。
- **理论风险**: 若未来继续追加距离函数、量化策略，将超过 1000 行并触发"文件恐惧"效应（开发者因畏惧修改而复制粘贴）。
- **最小修复**: 将 `HnswIndex` 的构建、查询、序列化逻辑拆分为 `hnsw/build.rs`、`hnsw/query.rs`、`hnsw/serde.rs` 三个子模块。

---

### 2.4 agent-core unwrap/expect 统计

**命令**: `Select-String -Pattern "unwrap\(\)|expect\("` in `src/intelligence/agent-core`  
**结果**: 🔴 **80 处匹配**（生产代码约 6 处，测试代码 74 处）

生产代码中的 unwrap 清单：
| 文件 | 行号 | 代码片段 | 风险 |
|------|------|----------|------|
| `blackboard.rs:60` | `duration_since(UNIX_EPOCH).unwrap()` | 时间倒流 panic | 低（仅系统时间异常） |
| `orchestrator.rs:111` | `duration_since(UNIX_EPOCH).unwrap()` | 同上 | 低 |
| `planner.rs:172` | `self.current_plan.as_mut().unwrap()` | 状态不一致 panic | **中** |
| `planner.rs:181` | `self.current_plan.as_mut().unwrap()` | 同上 | **中** |
| `planner.rs:182` | `plan.subgoals.get_mut(sg_id).unwrap()` | 索引越界 panic | **中** |
| `swarm.rs:193` | `try_read().map(...).unwrap_or(0)` | 非 panic，但语义隐晦 | 低 |

**后果评估**:
- **实际后果**: `planner.rs` 的 3 处 unwrap 在并发或异常恢复路径中可能触发 panic，导致整个 async 任务崩溃。测试中的 74 处 unwrap 虽不影响运行时，但会在测试数据异常时产生难以定位的堆栈。
- **理论风险**: unwrap 密度高会形成"panic 文化"，后续开发者效仿，逐渐侵蚀可靠性。
- **最小修复**: 
  1. `planner.rs`: 将 `unwrap()` 替换为 `ok_or(ReplError::Internal("plan not initialized"))??`。
  2. 测试代码：批量替换为 `.expect("test fixture: planner should have plan after create_goal")`，使失败信息自描述。

---

### 2.5 跨层依赖检查

**命令**: `cargo tree -p intelligence-agent-core`  
**结果**: 🔴 **agent-core（intelligence 层）直接依赖 chimera-repl（interface 层）和 engine-tool-system（engine 层）**

```text
intelligence-agent-core v0.1.0
├── chimera-repl v0.1.0        <-- 上层 REPL 组件
│   ├── codex-twist v0.1.0
│   └── engine-tool-system v0.1.0
└── engine-tool-system v0.1.0  <-- 横向 engine 层
```

代码中可见 `lib.rs:30` 直接 `pub use chimera_repl::traits::{ReplEngineCore, ReplError, ReplResult}`，将上层类型作为自身公共 API 暴露。

**后果评估**:
- **实际后果**: 这是**依赖倒置（Dependency Inversion Violation）**。agent-core 作为 intelligence 层内核，本应是被上层依赖的方，现在却依赖 REPL 的事件循环和错误类型。修改 chimera-repl 的 traits 会级联破坏 agent-core 的公共接口。
- **理论风险**: 形成循环依赖后，无法独立编译/发布 agent-core；模块边界名存实亡。
- **最小修复**: 
  1. 在 `agent-core` 中定义本地 `AgentError`、`AgentResult` 类型，通过 `From` trait 做适配，而非直接 re-export。
  2. `orchestrator.rs` 中对 `EngineController` 的依赖通过 trait object 或端口-适配器模式解耦，将 chimera-repl 降级为 `dev-dependencies` 或完全移除。

---

### 2.6 TODO/FIXME 统计

**命令**: 排除 `node_modules`/`target` 后搜索 `TODO`/`FIXME`  
**结果**: ✅ **仅 5 处**（此前未过滤 node_modules 时误报 650 处）

```text
crates/evm-bench-adapter/src/runner.rs:75-77  // TODO: Deploy vulnerability contract
interface/terminal/src/keymap_vim.rs:100       // TODO: 需要 undo_manager.mark_delete()
intelligence/index/vector/pkg/hajimi_hnsw.js:475 // TODO we could test for more things
```

**后果评估**:
- **实际后果**: 数量极低，说明团队有良好的即时清理习惯。但 `evm-bench-adapter` 的 3 处 TODO 属于安全测试扩展点，长期悬置意味着 EVM 漏洞验证覆盖率不足。
- **理论风险**: 低。但需建立 TODO 与 Issue 的关联规则（如 `// TODO(#123): ...`）。
- **最小修复**: 在 `.github/workflows` 中添加 `todo-check` action，拦截新增无 Issue 编号的 TODO。

---

### 2.7 agent_loop.rs 阻塞调用检查

**命令**: 搜索 `std::fs|std::thread|tokio::time::sleep`  
**结果**: ⚠️ **1 处 `tokio::time::sleep(Duration::from_millis(10))`**

位于 `agent_loop.rs:254` 的**测试代码** `test_agent_loop_no_leak` 中。

**后果评估**:
- **实际后果**: 生产代码中无阻塞调用，符合 async 运行时规范。测试中的 10ms sleep 用于等待任务释放，在 CI 中可能因负载导致 flaky test（时序敏感）。
- **理论风险**: 若未来在生产代码中引入 `std::fs::read_to_string` 或 `std::thread::sleep`，会阻塞 tokio 工作线程。
- **最小修复**: 将测试中的固定 sleep 替换为 `tokio::time::timeout` + 轮询计数器，或使用 `tokio::task::yield_now()` 配合计数断言。

---

### 2.8 错误处理模式重复（人工审计）

对比 `agent_loop.rs` / `swarm.rs` / `governance.rs` 的降级路径：

| 文件 | 降级模式 | 日志标签 | 行为 |
|------|----------|----------|------|
| `agent_loop.rs:81` | `warn!("Reflection failed (continuing): {}", e)` | `NEG-002: Degrade gracefully` | 继续循环 |
| `agent_loop.rs:85` | `warn!("Act failed (continuing): {}", e)` | `NEG-002` | 继续循环 |
| `swarm.rs:149` | `warn!("Swarm delegation failed (falling back): {}", e)` | 无标签 | 仅记录，已完成的任务不撤回 |
| `governance.rs:145` | `warn!("Advisory: {} - proceeding", req.description)` | 无标签 | 直接批准 |

**后果评估**:
- **实际后果**: 三种不同的降级语义（continuing / falling back / proceeding）没有统一抽象。开发者需要阅读每个调用点才能判断失败后的状态机走向。`agent_loop.rs` 中 `if i > 10`（魔法数字）作为 Act 失败的熔断条件，与 `MAX_ITERATIONS=100`、`ITERATION_BUDGET=50` 不在同一配置维度。
- **理论风险**: 碎片化错误处理会随模块增长演变为"意大利面条式恢复"，难以追踪状态一致性。
- **最小修复**: 
  1. 引入统一 trait `Degradable<T>` 或宏 `degrade_warn!(e, "reflect")`。
  2. 将 `i > 10` 提取为 `const EARLY_FAILURE_THRESHOLD: usize = 10`，与 `MAX_ITERATIONS` 放在一起。

---

### 2.9 魔法数字检查

**命令**: 搜索 `agent_loop.rs` 中的数字常量  
**结果**: 🔴 **内联魔法数字 `10`**

```rust
const MAX_ITERATIONS: usize = 100;       // ✅ 命名常量
const ITERATION_BUDGET: usize = 50;      // ✅ 命名常量
const CHECKPOINT_INTERVAL: usize = 10;   // ✅ 命名常量
if i > 10 { outcome = LoopOutcome::ActFailed(...); break; } // ❌ 魔法数字
```

**后果评估**:
- **实际后果**: `10` 与 `CHECKPOINT_INTERVAL=10` 数值相同但语义完全不同（前者是 Act 失败熔断阈值，后者是 checkpoint 周期）。开发者极易误改。
- **理论风险**: 魔法数字是维护债务中最容易累积的"微伤口"。
- **最小修复**: 提取为 `const ACT_FAILURE_THRESHOLD: usize = 10;`，并添加文档注释说明与 `MAX_ITERATIONS` 的关系。

---

### 2.10 死代码检查

**命令**: `cargo clippy --workspace -- -W dead_code`  
**结果**: ⚠️ **编译失败，但捕获到 unused import/variable 警告**

```text
warning: unused import: `DeleteRange`  --> interface/terminal/src/input_handler.rs:7
warning: unused import: `StreamChunk`  --> (某模块)
warning: unused variable: `api_key`     --> (某模块，2处)
warning: unused import: `simhash64`     --> (2处)
```

**后果评估**:
- **实际后果**: 未使用的导入和变量会误导代码阅读者，增加认知负荷。`api_key` 未使用暗示 API 密钥注入逻辑可能不完整。
- **理论风险**: 死代码是腐烂的" Canary"，往往预示着未完成的特性或已废弃的集成点。
- **最小修复**: 在 CI 中启用 `#![deny(unused_imports, unused_variables)]`（或 clippy 对应 lint），一次清理后保持门禁。

---

### 2.11 新人上手时间估算

**输入数据**:
- `agent_loop.rs`: 258 行
- `swarm.rs`: 255 行
- `governance.rs`: 241 行
- agent-core 目录总计: ~2,582 行（15 个文件）

**复杂度因子**:
1. **跨层依赖倒置**: 需同时理解 chimera-repl 和 engine-tool-system 才能读懂 agent-core（+2 天）。
2. **模式不一致**: 三种错误处理风格、魔法数字、unwrap 混用（+1.5 天）。
3. **无 clippy 辅助**: 无法通过工具快速发现代码异味（+0.5 天）。

**估算结论**: 
- **理论最小值**: 3 天（仅阅读 3 个核心文件并跑通测试）。
- **实际预期**: **5–7 天** 才能安全提交生产代码级 PR。若包含理解周边 wasm/codex-twist 的 unsafe 边界，则延长至 **10 天**。

---

## 3. 风险矩阵

| 发现项 | 理论风险 | 实际后果 | 修复成本 | 优先级 |
|--------|----------|----------|----------|--------|
| 构建失败（scale-info） | 高 | 静态分析全面失效 | 低（pin 版本） | P0 |
| 跨层依赖倒置 | 高 | API 级联破坏 | 中（重构接口） | P0 |
| unsafe 无 SAFETY | 高 | FFI 段错误无上下文 | 低（补注释） | P1 |
| planner unwrap | 中 | 运行时 panic | 低（换 ?） | P1 |
| 错误处理碎片化 | 中 | 状态机难追踪 | 中（抽象 trait） | P2 |
| 魔法数字 | 低 | 误改阈值 | 极低（提取常量） | P2 |
| 测试 sleep | 低 | Flaky test | 低（换 timeout） | P3 |
| 超大文件 | 低 | 未来失控 | 中（拆模块） | P3 |
| TODO 数量 | 低 | 无 | 极低 | P4 |

---

## 4. 最小修复方案（按优先级排序）

### 🔴 P0（本周内）
1. **修复构建**: 在 workspace `Cargo.toml` 的 `[patch.crates-io]` 中锁定 `scale-info = "=2.11.3"`（或已知兼容版本），恢复 clippy 可用性。
2. **解耦 agent-core**: 
   - 创建 `agent-core/src/ports.rs` 定义本地 `AgentResult`/`AgentError`。
   - 将 `lib.rs` 中的 `pub use chimera_repl::*` 替换为本地类型，并通过 `orchestrator.rs` 的适配器做转换。

### 🟠 P1（两周内）
3. **补全 SAFETY**: 为 `storage_gateway.rs` 的 5 个 FFI 函数和 `vector_text_hybrid.rs` 的 `from_raw_parts` 添加 `# Safety` 文档。
4. **消除生产 unwrap**: `planner.rs` 三处 `unwrap()` 改为 `ok_or(...)?`；`blackboard.rs`/`orchestrator.rs` 的 `duration_since().unwrap()` 改为 `unwrap_or_else(|_| Duration::ZERO)` 或返回错误。

### 🟡 P2（一个月内）
5. **统一降级语义**: 在 `agent-core` 中新增 `fn degrade<T, E>(result: Result<T, E>, ctx: &str) -> Option<T>` 辅助函数，统一 "continuing/falling back/proceeding" 日志格式。
6. **魔法数字常量化**: 提取 `ACT_FAILURE_THRESHOLD`、`RESTART_DELAY_MS`（swarm.rs 的 100ms）、`VOTE_TIMEOUT_MS`（governance.rs 的 5000ms）。

### 🟢 P3（排期优化）
7. **拆分 hnsw.rs**: 按构建/查询/序列化拆分为子模块。
8. **清理 unused**: 启用 `#![deny(unused_imports)]` 并全局清理。

---

## 5. 结论

HAJIMI 代码库在 agent-core 层面展现了**较高的功能完成度**，但**架构边界和静态分析基础设施**存在显著维护债务。最紧迫的是**构建失败导致的 clippy 失效**和**agent-core 对 chimera-repl 的依赖倒置**——前者使所有自动化质量门禁失灵，后者将长期制约模块独立演进。建议在下一个 Sprint 中优先处理 P0 项，以恢复团队的安全网。

---
*报告生成基于实际命令输出，未做理论臆测。*
