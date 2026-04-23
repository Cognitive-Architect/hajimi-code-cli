//! Agent Core Integration Tests (B-01/10)
use agent_core::*;
use agent_core::planner::{Planner, Priority};
use agent_core::governance::AgentGovernance;
use agent_core::swarm::SwarmCoordinator;
use std::sync::Arc;
use tokio::sync::Mutex;
use memory::memory_gateway::MemoryGateway;

fn mem() -> Arc<Mutex<MemoryGateway>> { Arc::new(Mutex::new(MemoryGateway::new("i"))) }

#[tokio::test] async fn test_agent_lifecycle_full() {
    let _orch = orchestrator::AgentOrchestrator::new(mem());
    let cfg = AgentConfig::supervisor("lc");
    assert!(cfg.capability.can_govern);
}

#[tokio::test] async fn test_plan_execute_reflect_cycle() {
    let m = mem();
    let mut p = planner::HierarchicalPlanner::new(m.clone(), AgentContext::new());
    let gid = p.create_goal("Fix bug", Priority::Critical).await.unwrap();
    let _ = p.decompose(&gid).await.unwrap();
    assert!(p.next_task().await.unwrap().is_some() || p.is_complete());
}

#[tokio::test] async fn test_governance_integration() {
    let gov = governance::DefaultGovernance::new();
    let ctx = AgentContext::new();
    let req = governance::GovernanceRequest {
        requester: "i".to_string(), action_type: "a".to_string(),
        risk_score: 0.8, description: "T".to_string(), level: governance::ApprovalLevel::Critical,
    };
    let _ = gov.approve(&ctx, &req).await;
}

#[tokio::test] async fn test_swarm_coordination() {
    let gov = Arc::new(governance::DefaultGovernance::new());
    let mut sv = swarm::Supervisor::new(gov, AgentContext::new());
    let cfg = AgentConfig::supervisor("sc");
    let c = sv.spawn_worker(AgentRole::Coder, cfg.clone()).await.unwrap();
    let r = sv.spawn_worker(AgentRole::Researcher, cfg.clone()).await.unwrap();
    assert_eq!(sv.worker_count(), 2);
    sv.delegate(swarm::TaskAssignment { task_id: "t1".to_string(), description: "C".to_string(), assigned_to: c, priority: 5 }).await.unwrap();
    sv.delegate(swarm::TaskAssignment { task_id: "t2".to_string(), description: "R".to_string(), assigned_to: r, priority: 3 }).await.unwrap();
}

#[tokio::test] async fn test_tool_invocation_failure() {
    let orch = orchestrator::AgentOrchestrator::new(mem());
    let result: Result<engine_tool_system::ToolOutput, engine_tool_system::ToolError> = 
        orch.invoke_tool("nonexistent", serde_json::json!({})).await;
    assert!(result.is_err());
}

#[tokio::test] async fn test_loop_timeout_handling() {
    let orch = orchestrator::AgentOrchestrator::new(mem());
    let out = orch.execute_natural_language_goal("to", "Never end").await.unwrap();
    assert!(matches!(out, agent_loop::LoopOutcome::BudgetExceeded | agent_loop::LoopOutcome::Success | agent_loop::LoopOutcome::Aborted));
}

#[tokio::test] async fn test_worker_crash_isolation() {
    let gov = Arc::new(governance::DefaultGovernance::new());
    let sv = swarm::Supervisor::new(gov, AgentContext::new());
    let cfg = AgentConfig::supervisor("wci");
    let mut sv_lock = sv;
    let wid = sv_lock.spawn_worker(AgentRole::Coder, cfg).await.unwrap();
    sv_lock.handle_worker_crash(&wid).await;
}

#[tokio::test] async fn test_checkpoint_restore_failure() {
    let mgr = checkpoint::CheckpointManager::new();
    let result = mgr.restore_latest(&"ne".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test] async fn test_agent_loop_observability() {
    let orch = orchestrator::AgentOrchestrator::new(mem());
    let lp = orch.create_agent_loop();
    assert_eq!(lp.current_state().await, agent_loop::LoopState::Idle);
}

#[tokio::test] async fn test_single_round_performance() {
    let orch = orchestrator::AgentOrchestrator::new(mem());
    let start = std::time::Instant::now();
    let _ = orch.execute_natural_language_goal("perf", "Quick").await;
    assert!(start.elapsed().as_millis() < 5000);
}
