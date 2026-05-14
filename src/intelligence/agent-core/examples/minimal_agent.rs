//! Minimal agent example using AgentLoopBuilder.
//! Run with: cargo run --example minimal_agent

use agent_core::{AgentContext, AgentLoopBuilder, AutonomousReflector, HierarchicalPlanner};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mem = Arc::new(Mutex::new(memory::memory_gateway::MemoryGateway::new(
        "minimal",
    )));
    let planner = Arc::new(Mutex::new(HierarchicalPlanner::new(
        mem.clone(),
        AgentContext::new(),
    )));
    let reflector = Arc::new(Mutex::new(AutonomousReflector::new(
        mem.clone(),
        AgentContext::new(),
    )));

    let agent = AgentLoopBuilder::new()
        .with_planner(planner)
        .with_reflector(reflector)
        .build()
        .expect("build agent");

    let outcome = agent.run("agent1".to_string(), "Hello world").await;
    println!("{:?}", outcome);
}
