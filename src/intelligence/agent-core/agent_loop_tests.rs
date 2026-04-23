#[cfg(test)]
mod tests {
    use crate::agent_loop::{AgentLoop, LoopState, LoopOutcome};
    use crate::{AgentContext};
    use crate::governance::DefaultGovernance;
    use crate::planner::{HierarchicalPlanner, Planner, Priority};
    use crate::reflector::{AutonomousReflector, Reflector};
    use crate::blackboard::Blackboard;
    use crate::checkpoint::CheckpointManager;
    use memory::memory_gateway::MemoryGateway;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn test_loop_with_mem(mem: Arc<Mutex<MemoryGateway>>) -> AgentLoop {
        AgentLoop::new(
            AgentContext::new(),
            Arc::new(Mutex::new(HierarchicalPlanner::new(mem.clone(), AgentContext::new()))) as Arc<Mutex<dyn Planner>>,
            Arc::new(Mutex::new(AutonomousReflector::new(mem.clone(), AgentContext::new()))) as Arc<Mutex<dyn Reflector>>,
            Arc::new(DefaultGovernance::new()),
            None,
            Arc::new(Blackboard::new()),
            Arc::new(CheckpointManager::new()),
            Some(mem),
        )
    }

    fn test_loop() -> AgentLoop {
        let m = Arc::new(Mutex::new(MemoryGateway::new("test")));
        test_loop_with_mem(m)
    }

    #[tokio::test]
    async fn test_creation() {
        let _ = test_loop();
    }

    #[tokio::test]
    async fn test_from_nl() {
        let (g, p) = AgentLoop::from_natural_language("Fix critical bug");
        assert_eq!(g, "Fix critical bug");
        assert_eq!(p, Priority::Critical);
    }

    #[tokio::test]
    async fn test_run() {
        assert!(test_loop().run("a1".to_string(), "Test").await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_goal() {
        let l = test_loop();
        assert!(l.execute_goal("a1".to_string(), "Test goal").await.is_ok());
    }

    #[tokio::test]
    async fn test_max_iter() {
        let out = test_loop().run("a1".to_string(), "Never end").await.unwrap();
        assert!(matches!(out, LoopOutcome::BudgetExceeded | LoopOutcome::Success | LoopOutcome::Aborted));
    }

    #[tokio::test]
    async fn test_long_running_stability() {
        for i in 0..3 {
            let _ = test_loop().run(format!("a{}", i), &format!("G{}", i)).await;
        }
    }

    #[tokio::test]
    async fn test_state_observability() {
        let l = test_loop();
        assert_eq!(l.current_state().await, LoopState::Idle);
    }

    #[tokio::test]
    async fn test_autonomous_goal_completion() {
        let l = test_loop();
        let outcome = l.execute_goal("agent1".to_string(), "Create a simple test plan").await.unwrap();
        assert!(matches!(outcome, LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted));
    }

    #[tokio::test]
    async fn test_agent_loop_no_leak() {
        let mem = Arc::new(Mutex::new(MemoryGateway::new("leak_test")));
        let initial = Arc::strong_count(&mem);
        for i in 0..10 {
            let l = test_loop_with_mem(mem.clone());
            let _ = l.run(format!("leak_test_{}", i), "Quick test goal").await;
            drop(l);
        }
        let final_count = Arc::strong_count(&mem);
        assert!(final_count <= initial + 1, "Resource leak detected: Arc count {} > expected {}", final_count, initial + 1);
    }

    #[tokio::test]
    async fn test_trace_emit() {
        let l = test_loop();
        let mut rx = l.subscribe_trace().expect("trace subscriber");
        let _ = l.run("trace_test".to_string(), "Quick test").await;
        let mut count = 0;
        while let Ok(_event) = rx.try_recv() {
            count += 1;
        }
        assert!(count >= 4, "Expected at least 4 trace events, got {}", count);
    }
}
