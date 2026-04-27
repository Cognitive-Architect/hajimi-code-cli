//! DEBT-LINES-B03A: Extracted Swarm delegation logic from agent_loop.rs::act()
//! Encapsulates worker discovery, task delegation, result polling, and timeout handling.

use crate::blackboard::Blackboard;
use crate::planner::Task;
use crate::swarm::{Supervisor, SwarmCoordinator, TaskAssignment, WorkerResult};
use chimera_repl::traits::{ReplError, ReplResult};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Coordinates task delegation to Swarm workers and result retrieval.
pub struct SwarmDelegate;

impl SwarmDelegate {
    /// Attempt to delegate a task to an idle worker and wait for completion.
    /// Returns `Some(Result)` if delegation was attempted, `None` if no idle worker available.
    pub async fn try_delegate(
        swarm: &Arc<Mutex<Supervisor>>,
        blackboard: &Arc<Blackboard>,
        agent_id: &str,
        task: &Task,
    ) -> Option<ReplResult<crate::planner::TaskResult>> {
        let worker_id = swarm.lock().await.find_idle_worker().await?;

        let assignment = TaskAssignment {
            task_id: task.id.clone(),
            description: task.description.clone(),
            assigned_to: worker_id,
            priority: 5,
        };
        swarm.lock().await.delegate(assignment).await.ok()?;

        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(30);
        let mut result = None;
        while result.is_none() && tokio::time::Instant::now() < deadline {
            result = swarm.lock().await.pop_result(&task.id).await;
            if result.is_none() {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }

        Some(match result {
            Some(r) => {
                let _ = blackboard.write(&format!("worker_result_{}", task.id), &r.output, agent_id).await;
                if r.success {
                    Ok(crate::planner::TaskResult { success: true, output: r.output, timestamp: chrono::Utc::now() })
                } else {
                    Err(ReplError::Session(format!("Worker failed: {}", r.output)))
                }
            }
            None => Err(ReplError::Session("Act timeout after 30s".to_string())),
        })
    }

    /// Pop a result for a specific task_id from the swarm results queue.
    pub async fn pop_result(swarm: &Arc<Mutex<Supervisor>>, task_id: &str) -> Option<WorkerResult> {
        swarm.lock().await.pop_result(task_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultGovernance;
    use crate::swarm::{Supervisor, TaskAssignment, WorkerResult};
    use crate::ports::WorkerMetrics;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_swarm_delegate_pop_result_empty() {
        let supervisor = Supervisor::new(Arc::new(DefaultGovernance::new()), crate::AgentContext::new());
        let swarm = Arc::new(Mutex::new(supervisor));
        let result = SwarmDelegate::pop_result(&swarm, "nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_swarm_delegate_no_idle_worker() {
        let supervisor = Supervisor::new(Arc::new(DefaultGovernance::new()), crate::AgentContext::new());
        let swarm = Arc::new(Mutex::new(supervisor));
        let blackboard = Arc::new(Blackboard::new());
        let task = Task { id: "t1".to_string(), parent_subgoal: "sg1".to_string(), description: "Test".to_string(), tool_calls: vec![], status: crate::planner::PlanStatus::Pending, result: None };
        let result = SwarmDelegate::try_delegate(&swarm, &blackboard, "agent-1", &task).await;
        assert!(result.is_none(), "Expected None when no idle worker available");
    }
}
