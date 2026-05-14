//! Phase 3 integration tests: cross-module workflows and regression validation.

use agent_core::agent_loop::TraceStepType;
use agent_core::loop_state_machine::LoopStateMachine;
use agent_core::memory_retriever::MemoryRetriever;
use agent_core::resource_monitor::ResourceMonitor;
use agent_core::{AgentOrchestrator, Blackboard, CheckpointManager, LoopState};
use memory::memory_gateway::MemoryGateway;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

fn tm() -> Arc<Mutex<MemoryGateway>> {
    Arc::new(Mutex::new(MemoryGateway::new("it")))
}

#[tokio::test]
async fn test_integration_trace_governance_workflow() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    let mut rx = lp.subscribe_trace().unwrap();
    let h = tokio::spawn(async move { lp.execute_goal("a".to_string(), "test").await });
    let mut evs = vec![];
    for _ in 0..15 {
        match timeout(Duration::from_millis(50), rx.recv()).await {
            Ok(Ok(ev)) => evs.push(ev),
            _ => break,
        }
    }
    assert!(h.await.unwrap().is_ok());
    assert!(!evs.is_empty());
    assert!(evs.iter().any(|e| e.step_type == TraceStepType::Observe));
}

#[tokio::test]
async fn test_integration_checkpoint_monitor_workflow() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let mon = ResourceMonitor::new();
    for i in 0..5 {
        mon.record_iteration();
        mon.record_blackboard_size(i * 10);
        mon.record_success();
        mgr.save(&"cm".to_string(), None, vec![], vec![], &bb)
            .await
            .unwrap();
    }
    assert_eq!(mon.get_metrics().iteration_count, 5);
    assert_eq!(mgr.list(&"cm".to_string()).await.len(), 5);
}

#[tokio::test]
async fn test_integration_pause_resume_with_trace() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    lp.pause();
    assert!(lp.is_paused());
    lp.resume();
    assert!(!lp.is_paused());
    assert!(lp.execute_goal("a".to_string(), "q").await.is_ok());
}

#[tokio::test]
async fn test_integration_resource_monitor_in_loop() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    let before = lp.resource_monitor.get_metrics().iteration_count;
    let _ = lp.execute_goal("m".to_string(), "test").await;
    assert!(lp.resource_monitor.get_metrics().iteration_count > before);
}

#[tokio::test]
async fn test_regression_debt_lines_extraction() {
    let bb = Arc::new(Blackboard::new());
    assert_eq!(
        LoopStateMachine::next_state(LoopState::Idle),
        LoopState::Observing
    );
    assert!(LoopStateMachine::is_terminal(LoopState::Completed));
    assert!(!LoopStateMachine::is_terminal(LoopState::Acting));
    assert_eq!(LoopStateMachine::step_index(LoopState::Acting), 4);
    let retriever = MemoryRetriever::new(bb, None, None);
    match retriever.retrieve("test").await {
        agent_core::memory_retriever::RetrieveOutcome::Error(_) => {}
        _ => panic!("Expected error"),
    }
}

#[tokio::test]
async fn test_regression_trace_event_fields_present() {
    use agent_core::TraceEvent;
    let event = TraceEvent {
        step: LoopState::Planning,
        details: "T".to_string(),
        iteration: 0,
        timestamp: chrono::Utc::now(),
        step_type: TraceStepType::Plan,
        plan_summary: Some("P".to_string()),
        reflection_key_points: vec!["k1".to_string()],
        confidence_score: Some(0.5),
        edit_payload: None,
        operation_summary: None,
        thinking_content: None,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(
        json.contains("plan_summary")
            && json.contains("reflection_key_points")
            && json.contains("confidence_score")
    );
}

#[tokio::test]
async fn test_regression_governance_controls_present() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    lp.pause();
    assert!(lp.is_paused());
    lp.resume();
    assert!(!lp.is_paused());
    assert!(lp.inject_memory("k", "v", &"a".to_string()).await.is_ok());
}

#[tokio::test]
async fn test_regression_checkpoint_enhancements_present() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let chk = mgr
        .save(&"r".to_string(), None, vec![], vec![], &bb)
        .await
        .unwrap();
    assert_eq!(mgr.restore(&chk.id).await.unwrap().id, chk.id);
    assert!(mgr.compare(&chk.id, &chk.id).await.unwrap());
    assert!(!mgr.export(&chk.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn test_invalid_none_checkpoint_compare_fails() {
    assert!(CheckpointManager::new()
        .compare("fake_a", "fake_b")
        .await
        .is_err());
}

#[tokio::test]
async fn test_invalid_empty_plan_update() {
    let lp = AgentOrchestrator::new(tm()).create_agent_loop();
    assert!(lp.update_plan("").await.is_ok());
}

#[tokio::test]
async fn test_integration_metrics_contain_timestamp() {
    assert!(!ResourceMonitor::new().get_metrics().timestamp.is_empty());
}
