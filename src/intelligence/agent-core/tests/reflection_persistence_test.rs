use agent_core::reflection_persistence::ReflectionPersistence;
use agent_core::reflector::{Reflection, Critique, CritiqueSeverity};
use agent_core::planner::TaskResult;
use memory::memory_gateway::MemoryGateway;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_persist_load_roundtrip() {
    let gw = MemoryGateway::new("test_reflect_roundtrip");
    let rp = ReflectionPersistence::new(Arc::new(Mutex::new(gw)));
    let reflection = Reflection {
        reflection_id: "r1".into(),
        original_goal_id: "g1".into(),
        execution_result: TaskResult { success: true, output: "completed".into(), timestamp: chrono::Utc::now() },
        critique: Critique { success: true, issues: vec![], suggestions: vec!["improve".into()], severity: CritiqueSeverity::Low },
        optimized_plan: None,
        confidence: 0.92,
        timestamp: chrono::Utc::now(),
    };
    rp.persist(&reflection).await.unwrap();
    let loaded = rp.load("r1").await.unwrap();
    assert!(loaded.is_some(), "Expected reflection to be loadable after persist");
    let loaded = loaded.unwrap();
    assert_eq!(loaded.reflection_id, "r1");
    assert_eq!(loaded.original_goal_id, "g1");
    assert_eq!(loaded.confidence, 0.92);
    assert!(loaded.critique.success);
}

#[tokio::test]
async fn test_load_not_found_returns_none() {
    let gw = MemoryGateway::new("test_not_found");
    let rp = ReflectionPersistence::new(Arc::new(Mutex::new(gw)));
    let loaded = rp.load("nonexistent").await.unwrap();
    assert!(loaded.is_none(), "Expected None for non-existent reflection");
}
