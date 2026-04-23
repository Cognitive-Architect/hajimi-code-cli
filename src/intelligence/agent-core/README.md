# Agent Core Module

Autonomous multi-agent orchestration system for HAJIMI.

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

## Testing

Run the full test suite:

```bash
cargo test -p intelligence-agent-core
```

Expected: 90+ tests passed, 0 compilation errors.

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
