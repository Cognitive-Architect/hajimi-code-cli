# Agent Core Module

Autonomous multi-agent orchestration system for HAJIMI.

## 职责

- **7 步自主循环**：`AgentLoop` 实现 `Observe → Retrieve → Plan → Act → Reflect → Store → Decide` 的主动式执行循环，最大迭代 100 次，预算 50 次，每 10 次迭代自动 checkpoint
- **Swarm 协调**：`Supervisor` 基于 Supervisor-Worker 模式管理多智能体协作，支持动态 spawn/stop/restart Worker、任务委托（`delegate`）、结果聚合（`aggregate`）、指数退避重试（最多 3 次）
- **可插拔治理**：`DefaultGovernance` 实现 5 级审批策略（`Auto / Advisory / Required / Critical / Override`），基于风险评分自动升级；支持自定义 `GovernancePolicy` 注册、投票机制、用户反馈记录
- **EditApplier**：可靠的 hunk 级编辑管线，实现 `ProposedEdit → Review → Apply/Reject`，包含冲突检测、原子写入（临时文件 + rename）、备份与撤销栈（最大 100 条）、并发编辑保护、文件大小限制（10MB）和 hunk 数量限制（50）
- **资源监控**：`ResourceMonitor` 记录迭代次数、成功率、Blackboard 大小、编辑次数、撤销栈深度等指标
- **Trace 可观测性**：`TraceEvent` 广播通道，覆盖全部 7 步状态转移与编辑事件（`EditProposed` / `EditApplied` / `EditRejected`），支持富化 trace（plan_summary、reflection_key_points、confidence_score）

## Quick Start

```rust
use agent_core::{AgentLoopBuilder, DefaultGovernance, HierarchicalPlanner, AutonomousReflector, AgentContext, Blackboard, CheckpointManager};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mem = Arc::new(Mutex::new(memory::memory_gateway::MemoryGateway::new("demo")));
    let planner = Arc::new(Mutex::new(HierarchicalPlanner::new(mem.clone(), AgentContext::new())));
    let reflector = Arc::new(Mutex::new(AutonomousReflector::new(mem.clone(), AgentContext::new())));

    let agent = AgentLoopBuilder::new()
        .with_planner(planner)
        .with_reflector(reflector)
        .build()
        .expect("build agent");

    let outcome = agent.run("agent1".to_string(), "Hello world").await;
    println!("{:?}", outcome);
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent_core = { path = "../intelligence/agent-core" }
tokio = { version = "1", features = ["full"] }
```

## 依赖

```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["rt", "sync", "time"] }
tracing = "0.1"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"

# Internal dependencies
chimera-repl = { path = "../chimera/chimera-repl" }
memory = { path = "../memory" }
engine-tool-system = { path = "../../engine/tool-system" }
engine-llm-core = { path = "../../engine/llm-core" }
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│              AgentOrchestrator                  │
│  ┌────────┐ ┌────────┐ ┌──────────┐ ┌────────┐ │
│  │Planner │ │Reflector│ │Governance│ │ Swarm  │ │
│  └────┬───┘ └────┬───┘ └────┬─────┘ └───┬────┘ │
│       └──────────┴──────────┴───────────┘       │
│                    │                            │
│              ┌─────┴─────┐                      │
│              │ AgentLoop │ ← 7-step cycle      │
│              └─────┬─────┘                      │
│  ┌─────────────────┼──────────────────────┐    │
│  │ Blackboard(Shared)  Checkpoint(Persist)│    │
│  └─────────────────┴──────────────────────┘    │
└─────────────────────────────────────────────────┘
```

## 7-Step Loop

1. Observe → 2. Retrieve → 3. Plan → 4. Act → 5. Reflect → 6. Store → 7. Decide

## Builder API

Use `AgentLoopBuilder` to construct an `AgentLoop` with chainable methods and defaults:

```rust
let agent = AgentLoopBuilder::new()
    .with_context(AgentContext::new())
    .with_planner(planner)
    .with_reflector(reflector)
    .with_governance(Arc::new(DefaultGovernance::new()))
    .with_blackboard(Arc::new(Blackboard::new()))
    .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
    .build()?;
```

Required fields: `planner`, `reflector`.  
Optional fields default to `DefaultGovernance`, empty `Blackboard`, `CheckpointManager::new()`, and `None` for swarm/memory.

## Features

- **Autonomous Loop**: 7-step proactive execution cycle with observability
- **Swarm Coordination**: Supervisor-Worker multi-agent collaboration
- **Pluggable Governance**: 5-level approval strategy (Auto → Override)
- **Builder Pattern**: Chainable configuration with sensible defaults
- **Memory Integration**: 5-tier memory gateway (Session/Auto/Dream/Graph/Cloud)
- **Checkpoint Recovery**: Full state persistence and restore

## Modules

| Module | Description |
|--------|-------------|
| `agent_loop` | Core 7-step autonomous execution loop |
| `planner` | Hierarchical goal/task planning |
| `reflector` | Execution critique and optimization |
| `governance` | Policy-driven approval engine |
| `swarm` | Multi-agent coordination |
| `blackboard` | Shared inter-agent state |
| `checkpoint` | State persistence and recovery |
| `edit_applier` | Hunk-level edit pipeline with conflict detection and undo |

## Governance (Pluggable)

5-Level strategy: Auto, Advisory, Required, Critical, Override

```rust
struct MyPolicy;
impl GovernancePolicy for MyPolicy {
    async fn evaluate(&self, ctx: &AgentContext, req: &GovernanceRequest) -> Option<Decision> {
        if req.risk_score > 0.8 { Some(Decision::Rejected("Too risky".to_string())) } else { None }
    }
}
gov.register_policy("custom", Arc::new(MyPolicy), "admin_test", PermissionLevel::Admin).await?;
```

## 测试

运行 Agent Core 全部测试（含 E2E，约 249 个测试）：

```bash
cargo test -p intelligence-agent-core
```

稳定性测试（100 轮）：

```bash
cargo test -p intelligence-agent-core test_stability_100_rounds
```

预期：249+ 测试通过，0 编译错误。

## Advanced Usage

Configure with Swarm and custom governance:

```rust
let agent = AgentLoopBuilder::new()
    .with_planner(planner)
    .with_reflector(reflector)
    .with_governance(Arc::new(DefaultGovernance::new().with_default_level(ApprovalLevel::Required)))
    .with_swarm(Some(Arc::new(Mutex::new(Supervisor::new(gov, ctx)))))
    .with_memory(Some(mem))
    .build()?;
```

## DEBT Summary (Day 10)

| DEBT | Status | Phase | Description |
|------|--------|-------|-------------|
| DEBT-RETRIEVE-PHASE5 | Active | 5 | Session层已集成，Graph/Dream层检索待全面集成（部分实现） |
| DEBT-WORKER-TOOL-EXECUTION | Active | 5 | Worker执行结果回调机制待完善 |
| DEBT-MEMORY-SYNC | Active | 5 | push_vector已调用，完整事件/Plan持久化待启用（部分实现） |
| DEBT-LEAK-TEST-PHASE5 | Active | 5 | AgentLoop资源泄漏测试待重写 |
| Others | CLEARED | - | 已清偿债务见docs/debt/agent-core-debt-history.md |

**Total Active: 4** (Target ≤ 8)

## API

```rust
let orch = AgentOrchestrator::new(memory);
let outcome = orch.execute_natural_language_goal("agent1", "Implement feature").await?;
```
