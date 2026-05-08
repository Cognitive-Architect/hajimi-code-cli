//! TraceEvent tests: serialization roundtrip, enriched fields, emit_trace broadcasting.

use agent_core::{AgentOrchestrator, LoopState, TraceEvent};
use agent_core::agent_loop::TraceStepType;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use memory::memory_gateway::MemoryGateway;

fn tm() -> Arc<Mutex<MemoryGateway>> { Arc::new(Mutex::new(MemoryGateway::new("tt"))) }

fn sample_event() -> TraceEvent {
    TraceEvent { step: LoopState::Planning, details: "Test".to_string(), iteration: 1, timestamp: chrono::Utc::now(), step_type: TraceStepType::Plan, plan_summary: Some("S".to_string()), reflection_key_points: vec!["p1".to_string()], confidence_score: Some(0.85), edit_payload: None, operation_summary: None, thinking_content: None }
}

#[tokio::test]
async fn test_trace_event_serialization_roundtrip() {
    let event = sample_event();
    let json = serde_json::to_string(&event).unwrap();
    let de: TraceEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(de.step, LoopState::Planning);
    assert_eq!(de.iteration, 1);
    assert_eq!(de.step_type, TraceStepType::Plan);
    assert_eq!(de.confidence_score, Some(0.85));
}

#[tokio::test]
async fn test_trace_event_loop_state_serde() {
    for state in [LoopState::Idle, LoopState::Observing, LoopState::Completed] {
        let de: LoopState = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(de, state);
    }
}

#[tokio::test]
async fn test_trace_event_trace_step_type_serde() {
    for st in [TraceStepType::Observe, TraceStepType::Plan, TraceStepType::Act, TraceStepType::Other] {
        let de: TraceStepType = serde_json::from_str(&serde_json::to_string(&st).unwrap()).unwrap();
        assert_eq!(de, st);
    }
}

#[tokio::test]
async fn test_trace_event_confidence_out_of_range_preserved() {
    let mut event = sample_event();
    event.confidence_score = Some(1.5);
    let de: TraceEvent = serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
    assert_eq!(de.confidence_score, Some(1.5));
}

#[tokio::test]
async fn test_trace_event_empty_reflection_key_points() {
    let mut event = sample_event();
    event.reflection_key_points = vec![];
    let de: TraceEvent = serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
    assert!(de.reflection_key_points.is_empty());
}

#[tokio::test]
async fn test_emit_trace_broadcasts_via_loop() {
    let orch = AgentOrchestrator::new(tm());
    let lp = orch.create_agent_loop();
    let mut rx = lp.subscribe_trace().unwrap();
    let handle = tokio::spawn(async move { lp.execute_goal("a".to_string(), "test").await });
    let mut events = vec![];
    for _ in 0..20 { match timeout(Duration::from_millis(40), rx.recv()).await { Ok(Ok(ev)) => events.push(ev), _ => break } }
    assert!(handle.await.unwrap().is_ok());
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.step_type == TraceStepType::Plan));
}

#[tokio::test]
async fn test_emit_trace_enriched_fields_preserved() {
    let event = TraceEvent { step: LoopState::Acting, details: "A".to_string(), iteration: 5, timestamp: chrono::Utc::now(), step_type: TraceStepType::Act, plan_summary: Some("Fix".to_string()), reflection_key_points: vec!["k1".to_string(), "k2".to_string()], confidence_score: Some(0.92), edit_payload: None, operation_summary: None, thinking_content: None };
    let de: TraceEvent = serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
    assert_eq!(de.plan_summary, Some("Fix".to_string()));
    assert_eq!(de.reflection_key_points.len(), 2);
    assert_eq!(de.confidence_score, Some(0.92));
}

#[tokio::test]
async fn test_trace_event_large_summary_handled() {
    let mut event = sample_event();
    event.plan_summary = Some("x".repeat(20000));
    let de: TraceEvent = serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
    assert_eq!(de.plan_summary.as_ref().unwrap().len(), 20000);
}
