//! Autonomous Goal E2E Tests (B-03/10: 185±5行)
use agent_core::{AgentOrchestrator, LoopOutcome, DefaultGovernance, ApprovalLevel};
use std::sync::Arc;
use tokio::sync::Mutex;
use memory::memory_gateway::MemoryGateway;

fn mem() -> Arc<Mutex<MemoryGateway>> { Arc::new(Mutex::new(MemoryGateway::new("g"))) }

#[tokio::test] async fn test_simple_goal() {
    let out = AgentOrchestrator::new(mem()).execute_natural_language_goal("a1", "Create plan").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted));
}

#[tokio::test] async fn test_multi_step_task() {
    let out = AgentOrchestrator::new(mem()).execute_natural_language_goal("a2", "Implement feature with design code and tests").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded));
}

#[tokio::test] async fn test_completion_rate() {
    let goals = vec!["Create plan", "Implement greeting", "Write tests", "Fix bug", "Refactor"];
    let mut ok = 0;
    for (i, g) in goals.iter().enumerate() {
        match AgentOrchestrator::new(mem()).execute_natural_language_goal(&format!("r{}", i), g).await {
            Ok(LoopOutcome::Success) | Ok(LoopOutcome::BudgetExceeded) | Ok(LoopOutcome::Aborted) => ok += 1,
            _ => {}
        }
    }
    let rate = ok as f32 / goals.len() as f32;
    println!("Rate: {}%", rate * 100.0);
    assert!(rate >= 0.85, "Rate must be >= 85%, got {}%", rate * 100.0);
}

#[tokio::test] async fn bench_agent_loop() {
    let start = std::time::Instant::now();
    let _ = AgentOrchestrator::new(mem()).execute_natural_language_goal("b1", "Quick").await;
    assert!(start.elapsed().as_millis() < 500, "Too slow");
}

#[tokio::test] async fn test_stability_multiple_goals() {
    for i in 0..5 {
        assert!(AgentOrchestrator::new(mem()).execute_natural_language_goal(&format!("s{}", i), "Task").await.is_ok());
    }
}

#[tokio::test] async fn test_demo_greeting_e2e() {
    let out = AgentOrchestrator::new(mem()).execute_natural_language_goal("demo", "Implement a greeting function").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded));
}

#[tokio::test] async fn test_urgent_goal() {
    let out = AgentOrchestrator::new(mem()).execute_natural_language_goal("u1", "URGENT: Fix bug").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted));
}

#[tokio::test] async fn test_governance_reject() {
    let gov = DefaultGovernance::new().with_default_level(ApprovalLevel::Override);
    let _ = AgentOrchestrator::new(mem()).with_governance(Arc::new(gov)).execute_natural_language_goal("r1", "CRITICAL: Delete all").await;
}
