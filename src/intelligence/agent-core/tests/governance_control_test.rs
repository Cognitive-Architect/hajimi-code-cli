//! Governance control tests: pause/resume, approval level, memory injection, plan update.

use agent_core::{AgentOrchestrator, ApprovalLevel, DefaultGovernance};
use std::sync::Arc;
use tokio::sync::Mutex;
use memory::memory_gateway::MemoryGateway;

fn tm() -> Arc<Mutex<MemoryGateway>> { Arc::new(Mutex::new(MemoryGateway::new("gt"))) }

#[tokio::test]
async fn test_pause_sets_paused_true() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    assert!(!lp.is_paused());
    lp.pause();
    assert!(lp.is_paused());
}

#[tokio::test]
async fn test_resume_sets_paused_false() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    lp.pause();
    lp.resume();
    assert!(!lp.is_paused());
}

#[tokio::test]
async fn test_pause_resume_cycle() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    for _ in 0..3 { lp.pause(); assert!(lp.is_paused()); lp.resume(); assert!(!lp.is_paused()); }
}

#[tokio::test]
async fn test_governance_set_approval_level_auto_to_critical() {
    let mut gov = DefaultGovernance::new();
    assert_eq!(gov.current_approval_level(), ApprovalLevel::Auto);
    gov.set_approval_level(ApprovalLevel::Critical).await.unwrap();
    assert_eq!(gov.current_approval_level(), ApprovalLevel::Critical);
}

#[tokio::test]
async fn test_governance_set_approval_level_advisory() {
    let mut gov = DefaultGovernance::new();
    gov.set_approval_level(ApprovalLevel::Advisory).await.unwrap();
    assert_eq!(gov.current_approval_level(), ApprovalLevel::Advisory);
}

#[tokio::test]
async fn test_governance_set_approval_level_required() {
    let mut gov = DefaultGovernance::new();
    gov.set_approval_level(ApprovalLevel::Required).await.unwrap();
    assert_eq!(gov.current_approval_level(), ApprovalLevel::Required);
}

#[tokio::test]
async fn test_governance_current_level_persists() {
    let mut gov = DefaultGovernance::new();
    for level in [ApprovalLevel::Advisory, ApprovalLevel::Required, ApprovalLevel::Critical, ApprovalLevel::Auto] {
        gov.set_approval_level(level).await.unwrap();
        assert_eq!(gov.current_approval_level(), level);
    }
}

#[tokio::test]
async fn test_inject_memory_writes_to_blackboard() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    assert!(lp.inject_memory("k", "v", &"a1".to_string()).await.is_ok());
}

#[tokio::test]
async fn test_update_plan_creates_goal() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    assert!(lp.update_plan("New plan").await.is_ok());
}

#[tokio::test]
async fn test_invalid_empty_key_inject_handled() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    assert!(lp.inject_memory("", "v", &"a1".to_string()).await.is_ok());
}
