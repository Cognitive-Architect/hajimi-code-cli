//! DEBT-LEAK-TEST-PHASE5: Resource leak detection tests for AgentLoop and Supervisor.
//! Uses Arc::weak_count, tokio::time::timeout, and std::mem::drop to verify cleanup.

use agent_core::governance::DefaultGovernance;
use agent_core::ports::WorkerMetrics;
use agent_core::swarm::{Supervisor, TaskAssignment, WorkerResult};
use agent_core::{AgentConfig, AgentRole};
use std::sync::{Arc, Weak};
use tokio::time::{timeout, Duration};

/// Test: AgentLoop normal flow — spawn and stop worker cleans up.
#[tokio::test]
async fn test_worker_cleanup_on_shutdown() {
    let mut supervisor = Supervisor::new(
        Arc::new(DefaultGovernance::new()),
        agent_core::AgentContext::new(),
    );
    let worker_id = supervisor
        .spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t"))
        .await
        .unwrap();
    assert_eq!(supervisor.worker_count(), 1);

    supervisor.stop_worker(&worker_id).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        supervisor.worker_count(),
        0,
        "Worker should be cleaned up after stop"
    );
}

/// Test: Supervisor drop releases all internal Arc handles.
#[tokio::test]
async fn test_supervisor_drop_releases_handles() {
    let metrics_weak: Weak<_>;
    {
        let mut supervisor = Supervisor::new(
            Arc::new(DefaultGovernance::new()),
            agent_core::AgentContext::new(),
        );
        let metrics = supervisor.metrics();
        metrics_weak = Arc::downgrade(&metrics);
        // Spawn and stop a worker to exercise lifecycle
        let id = supervisor
            .spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t"))
            .await
            .unwrap();
        let _ = supervisor.stop_worker(&id).await;
        drop(metrics);
        // supervisor dropped here
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        metrics_weak.upgrade().is_none(),
        "Supervisor metrics leaked after drop"
    );
}

/// Test: Arc weak reference confirms no strong references remain after drop.
#[tokio::test]
async fn test_arc_weak_count_after_drop() {
    let supervisor = Supervisor::new(
        Arc::new(DefaultGovernance::new()),
        agent_core::AgentContext::new(),
    );
    let metrics = supervisor.metrics();
    let weak = Arc::downgrade(&metrics);

    drop(metrics);
    drop(supervisor);

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        weak.upgrade().is_none(),
        "Arc strong_count did not reach zero after drop"
    );
}

/// Test: Worker force-abort via stop_worker leaves no residual handles.
#[tokio::test]
async fn test_worker_abort_cleanup() {
    let mut supervisor = Supervisor::new(
        Arc::new(DefaultGovernance::new()),
        agent_core::AgentContext::new(),
    );
    let worker_id = supervisor
        .spawn_worker(AgentRole::Researcher, AgentConfig::supervisor("t"))
        .await
        .unwrap();

    // Force stop (abort)
    let _ = supervisor.stop_worker(&worker_id).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        supervisor.worker_count(),
        0,
        "Worker should be cleaned up after abort"
    );
}

/// Test: Empty Supervisor drop does not panic.
#[tokio::test]
async fn test_empty_supervisor_drop_no_panic() {
    let supervisor = Supervisor::new(
        Arc::new(DefaultGovernance::new()),
        agent_core::AgentContext::new(),
    );
    drop(supervisor); // Should not panic
}

/// Test: Timeout-guarded leak detection — ensures test completes within budget.
#[tokio::test]
async fn test_leak_detection_within_timeout() {
    let result = timeout(Duration::from_secs(2), async {
        let mut supervisor = Supervisor::new(
            Arc::new(DefaultGovernance::new()),
            agent_core::AgentContext::new(),
        );
        let id = supervisor
            .spawn_worker(AgentRole::Executor, AgentConfig::supervisor("t"))
            .await
            .unwrap();
        supervisor.stop_worker(&id).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(supervisor.worker_count(), 0);
    })
    .await;
    assert!(result.is_ok(), "Leak detection test timed out");
}
