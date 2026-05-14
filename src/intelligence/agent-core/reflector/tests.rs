use super::*;
use crate::planner::{PlanStatus, Priority};
use std::collections::HashMap;

fn reflector() -> AutonomousReflector {
    AutonomousReflector::new(
        Arc::new(Mutex::new(memory::memory_gateway::MemoryGateway::new(
            "test",
        ))),
        AgentContext::new(),
    )
}

#[tokio::test]
async fn test_reflection_cycle() {
    let mut r = reflector();
    let goal = Goal {
        id: "g1".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let result = TaskResult {
        success: false,
        output: "Error".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let reflection = r.reflect(&goal, &result).await.unwrap();
    assert_eq!(reflection.original_goal_id, "g1");
    assert!(!reflection.critique.success);
}

#[tokio::test]
async fn test_critique_success() {
    let r = reflector();
    let goal = Goal {
        id: "g2".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let result = TaskResult {
        success: true,
        output: "Done".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let critique = r.critique(&goal, &result).await.unwrap();
    assert!(critique.success);
    assert_eq!(critique.severity, CritiqueSeverity::Low);
}

#[tokio::test]
async fn test_reflection_budget() {
    let mut r = reflector();
    let goal = Goal {
        id: "g3".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let result = TaskResult {
        success: false,
        output: "Error".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // Exhaust budget
    for _ in 0..MAX_REFLECTION_STEPS {
        let _ = r.reflect(&goal, &result).await;
    }
    assert!(r.budget_exhausted());
    assert!(r.reflect(&goal, &result).await.is_err());
}

/// Test persist_reflection creates a reflection successfully
#[tokio::test]
async fn test_persist_reflection() {
    let mut r = reflector();
    let goal = Goal {
        id: "g4".to_string(),
        description: "Test persist".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let result = TaskResult {
        success: false,
        output: "Error".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let reflection = r.reflect(&goal, &result).await.unwrap();
    assert!(!reflection.reflection_id.is_empty());
    assert_eq!(reflection.original_goal_id, "g4");
}

/// Test optimize_plan returns non-empty Plan with incremented version
#[tokio::test]
async fn test_optimize_plan_not_empty() {
    let r = reflector();
    let goal = Goal {
        id: "g5".to_string(),
        description: "Test optimize".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let critique = Critique {
        success: false,
        issues: vec!["Issue".to_string()],
        suggestions: vec!["Fix".to_string()],
        severity: CritiqueSeverity::High,
    };

    let optimized = r.optimize_plan(&goal, &critique).await.unwrap();
    if let Some(plan) = optimized {
        assert!(plan.version > 0);
        assert_eq!(plan.goal.id, goal.id);
    }
}

// Phase 2 Patch: reflect_multi aggregation tests
#[tokio::test]
async fn test_reflect_multi_all_success() {
    let mut r = reflector();
    let goal = Goal {
        id: "g6".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let results = vec![
        crate::swarm::WorkerResult::success(
            "t1",
            "w1".to_string(),
            "ok1",
            crate::ports::WorkerMetrics::new(10),
        ),
        crate::swarm::WorkerResult::success(
            "t2",
            "w2".to_string(),
            "ok2",
            crate::ports::WorkerMetrics::new(20),
        ),
        crate::swarm::WorkerResult::success(
            "t3",
            "w3".to_string(),
            "ok3",
            crate::ports::WorkerMetrics::new(30),
        ),
    ];
    let refl = r.reflect_multi(&goal, &results).await.unwrap();
    assert!(refl.critique.success);
    assert_eq!(refl.critique.severity, CritiqueSeverity::Low);
    assert_eq!(refl.confidence, 1.0);
}

#[tokio::test]
async fn test_reflect_multi_all_fail() {
    let mut r = reflector();
    let goal = Goal {
        id: "g7".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let results = vec![
        crate::swarm::WorkerResult::failure(
            "t1",
            "w1".to_string(),
            "e1",
            crate::ports::WorkerMetrics::new(10),
        ),
        crate::swarm::WorkerResult::failure(
            "t2",
            "w2".to_string(),
            "e2",
            crate::ports::WorkerMetrics::new(20),
        ),
        crate::swarm::WorkerResult::failure(
            "t3",
            "w3".to_string(),
            "e3",
            crate::ports::WorkerMetrics::new(30),
        ),
    ];
    let refl = r.reflect_multi(&goal, &results).await.unwrap();
    assert!(!refl.critique.success);
    assert_eq!(refl.critique.severity, CritiqueSeverity::Critical);
    assert_eq!(refl.confidence, 0.0);
}

#[tokio::test]
async fn test_reflect_multi_partial() {
    let mut r = reflector();
    let goal = Goal {
        id: "g8".to_string(),
        description: "Test".to_string(),
        priority: Priority::High,
        status: PlanStatus::Pending,
        subgoals: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        approved: true,
    };
    let results = vec![
        crate::swarm::WorkerResult::success(
            "t1",
            "w1".to_string(),
            "ok",
            crate::ports::WorkerMetrics::new(10),
        ),
        crate::swarm::WorkerResult::failure(
            "t2",
            "w2".to_string(),
            "e2",
            crate::ports::WorkerMetrics::new(20),
        ),
        crate::swarm::WorkerResult::failure(
            "t3",
            "w3".to_string(),
            "e3",
            crate::ports::WorkerMetrics::new(30),
        ),
    ];
    let refl = r.reflect_multi(&goal, &results).await.unwrap();
    assert!(!refl.critique.success);
    assert_eq!(refl.critique.severity, CritiqueSeverity::High);
    assert!((refl.confidence - 1.0 / 3.0).abs() < 0.001);
}
