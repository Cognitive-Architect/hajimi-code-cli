//! Swarm Coordinator: Supervisor-Worker pattern for multi-agent collaboration.
//! Day 6: Dynamic agent spawning, task delegation, and result aggregation.

use crate::{AgentConfig, AgentId, AgentRole};
use chimera_repl::traits::{ReplError, ReplResult};
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use engine_tool_system::{ToolRegistry, ToolArgs};
use serde_json::json;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{info, warn};

/// Task assignment for Worker agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: String, pub description: String,
    pub assigned_to: AgentId, pub priority: u8,
}

/// Worker execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub task_id: String, pub worker_id: AgentId,
    pub success: bool, pub output: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Worker container holding agent instance and metadata.
pub struct Worker {
    pub config: AgentConfig,
    pub tx: mpsc::Sender<SwarmMessage>,
    pub status: WorkerStatus,
    pub spawn_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkerStatus { Idle, Busy, Crashed, Stopped }

/// Message types for Swarm communication.
#[derive(Debug, Clone)]
pub enum SwarmMessage {
    TaskAssigned(TaskAssignment),
    TaskCompleted(WorkerResult),
    Shutdown,
}

/// Core trait for Swarm coordination.
#[async_trait]
pub trait SwarmCoordinator: Send + Sync {
    /// Delegate task to a Worker agent.
    async fn delegate(&self, task: TaskAssignment) -> ReplResult<()>;
    /// Spawn a new Worker agent.
    async fn spawn_worker(&mut self, role: AgentRole, config: AgentConfig) -> ReplResult<AgentId>;
    /// Stop a Worker agent.
    async fn stop_worker(&mut self, worker_id: &AgentId) -> ReplResult<()>;
    /// Restart a crashed Worker.
    async fn restart_worker(&mut self, worker_id: &AgentId) -> ReplResult<AgentId>;
    /// Aggregate results from all Workers.
    async fn aggregate(&self) -> Vec<WorkerResult>;
    /// Get active worker count.
    fn worker_count(&self) -> usize;
}

/// Supervisor coordinates multiple Worker agents.
pub struct Supervisor {
    workers: Arc<RwLock<HashMap<AgentId, Worker>>>,
    results: Arc<Mutex<Vec<WorkerResult>>>,
    governance: Arc<dyn AgentGovernance>,
    context: crate::AgentContext,
    tool_registry: Option<Arc<Mutex<ToolRegistry>>>,
}

impl Supervisor {
    /// Create new Supervisor with governance.
    pub fn new(governance: Arc<dyn AgentGovernance>, context: crate::AgentContext) -> Self {
        Self { workers: Arc::new(RwLock::new(HashMap::new())), results: Arc::new(Mutex::new(Vec::new())), governance, context, tool_registry: None }
    }
    /// Attach tool registry for worker execution.
    pub fn with_tool_registry(mut self, registry: Arc<Mutex<ToolRegistry>>) -> Self {
        self.tool_registry = Some(registry); self
    }

    /// Check if delegation requires governance approval.
    async fn approve_delegation(&self, task: &TaskAssignment) -> ReplResult<bool> {
        let req = GovernanceRequest {
            requester: "supervisor".to_string(),
            action_type: "delegate_task".to_string(),
            risk_score: task.priority as f32 / 20.0,
            description: task.description.clone(),
            level: if task.priority > 7 { ApprovalLevel::Critical } else { ApprovalLevel::Auto },
        };
        let decision = self.governance.approve(&self.context, &req).await?;
        Ok(matches!(decision, Decision::Approved))
    }

    /// Handle worker crash - isolate failure.
    pub async fn handle_worker_crash(&self, worker_id: &AgentId) {
        warn!("Worker {} crashed, isolating", worker_id);
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) { worker.status = WorkerStatus::Crashed; }
    }

    fn role_str(role: &AgentRole) -> &'static str {
        match role { AgentRole::Coder => "coder", AgentRole::Researcher => "researcher", AgentRole::Critic => "critic", AgentRole::Executor => "executor", _ => "worker" }
    }

    /// Retry a failed task by re-delegating to the same or another worker.
    pub async fn retry_task(&self, task: TaskAssignment) -> ReplResult<()> {
        info!("Retrying task {} for worker {}", task.task_id, task.assigned_to);
        self.delegate(task).await
    }
}

#[async_trait]
impl SwarmCoordinator for Supervisor {
    async fn delegate(&self, task: TaskAssignment) -> ReplResult<()> {
        if !self.approve_delegation(&task).await? {
            return Err(ReplError::Session("Delegation rejected".to_string()));
        }
        let workers = self.workers.read().await;
        if let Some(worker) = workers.get(&task.assigned_to) {
            if worker.status == WorkerStatus::Idle {
                let _ = worker.tx.send(SwarmMessage::TaskAssigned(task)).await;
                return Ok(());
            }
        }
        Err(ReplError::Session("Worker unavailable".to_string()))
    }

    async fn spawn_worker(&mut self, role: AgentRole, mut config: AgentConfig) -> ReplResult<AgentId> {
        let id = format!("{}_{}", Supervisor::role_str(&role), uuid::Uuid::new_v4());
        let id_for_spawn = id.clone();
        config.agent_id = id.clone(); config.role = role;
        let (tx, mut rx) = mpsc::channel(100);
        let worker = Worker { config: config.clone(), tx, status: WorkerStatus::Idle, spawn_time: chrono::Utc::now() };
        self.workers.write().await.insert(id.clone(), worker);
        let workers = self.workers.clone();
        let results = self.results.clone();
        let tool_registry = self.tool_registry.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    SwarmMessage::TaskAssigned(task) => {
                        info!("Worker {} processing {}", id_for_spawn, task.task_id);
                        let tool_registry = tool_registry.clone();
                        let id_clone = id_for_spawn.clone();
                        let task_id = task.task_id.clone();
                        let desc = task.description.clone();
                        let prio = task.priority;
                        tokio::spawn({
                            let workers = workers.clone();
                            let results = results.clone();
                            async move {
                                let (success, output) = if let Some(registry) = tool_registry {
                                    let reg: tokio::sync::MutexGuard<'_, ToolRegistry> = registry.lock().await;
                                    if let Some(tool) = reg.get("planning") {
                                        let args = ToolArgs::from(json!({
                                            "action": "create_goal",
                                            "description": desc,
                                            "priority": prio
                                        }));
                                        match tool.execute(args).await {
                                            Ok(out) => { info!("Worker {} executed: {}", id_clone, out.stdout); (true, out.stdout) }
                                            Err(e) => { warn!("Worker {} failed: {}", id_clone, e.message); (false, e.message) }
                                        }
                                    } else { (false, "Tool not found".to_string()) }
                                } else { (false, "No registry".to_string()) };
                                let mut r = results.lock().await;
                                r.push(WorkerResult { task_id, worker_id: id_clone.clone(), success, output, timestamp: chrono::Utc::now() });
                                let mut w = workers.write().await;
                                if let Some(worker) = w.get_mut(&id_clone) { worker.status = WorkerStatus::Idle; }
                            }
                        });
                        let mut w = workers.write().await;
                        if let Some(worker) = w.get_mut(&id_for_spawn) {
                            worker.status = WorkerStatus::Busy;
                        }
                    }
                    SwarmMessage::TaskCompleted(result) => {
                        let mut r = results.lock().await;
                        r.push(result);
                        let mut w = workers.write().await;
                        if let Some(worker) = w.get_mut(&id_for_spawn) { worker.status = WorkerStatus::Idle; }
                    }
                    SwarmMessage::Shutdown => {
                        let mut w = workers.write().await;
                        if let Some(worker) = w.get_mut(&id_for_spawn) { worker.status = WorkerStatus::Stopped; }
                        break;
                    }
                }
            }
        });
        Ok(id)
    }

    async fn stop_worker(&mut self, worker_id: &AgentId) -> ReplResult<()> {
        let workers = self.workers.read().await;
        if let Some(worker) = workers.get(worker_id) { let _ = worker.tx.send(SwarmMessage::Shutdown).await; }
        drop(workers); self.workers.write().await.remove(worker_id); Ok(())
    }

    async fn restart_worker(&mut self, worker_id: &AgentId) -> ReplResult<AgentId> {
        let old_config = { self.workers.read().await.get(worker_id).map(|w| w.config.clone()) };
        if let Some(config) = old_config {
            self.stop_worker(worker_id).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            self.spawn_worker(config.role, config).await
        } else { Err(ReplError::Session("Worker not found".to_string())) }
    }

    async fn aggregate(&self) -> Vec<WorkerResult> { self.results.lock().await.clone() }
    fn worker_count(&self) -> usize { self.workers.try_read().map(|w| w.len()).unwrap_or(0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultGovernance;

    fn supervisor() -> Supervisor {
        Supervisor::new(Arc::new(DefaultGovernance::new()), crate::AgentContext::new())
    }

    #[tokio::test]
    async fn test_supervisor_spawn_worker() {
        let mut s = supervisor();
        let id = s.spawn_worker(AgentRole::Coder, AgentConfig::supervisor("test")).await.unwrap();
        assert!(id.contains("coder")); assert_eq!(s.worker_count(), 1);
    }

    #[tokio::test]
    async fn test_supervisor_stop_worker() {
        let mut s = supervisor();
        let id = s.spawn_worker(AgentRole::Researcher, AgentConfig::supervisor("test")).await.unwrap();
        s.stop_worker(&id).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(s.worker_count(), 0);
    }

    #[tokio::test]
    async fn test_supervisor_restart_worker() {
        let mut s = supervisor();
        let id = s.spawn_worker(AgentRole::Critic, AgentConfig::supervisor("test")).await.unwrap();
        let new_id = s.restart_worker(&id).await.unwrap();
        assert_ne!(id, new_id); assert_eq!(s.worker_count(), 1);
    }

    #[tokio::test]
    async fn test_delegate_task() {
        let mut s = supervisor();
        let worker_id = s.spawn_worker(AgentRole::Executor, AgentConfig::supervisor("test")).await.unwrap();
        let task = TaskAssignment { task_id: "t1".to_string(), description: "Test".to_string(), assigned_to: worker_id, priority: 5 };
        assert!(s.delegate(task).await.is_ok());
    }

    #[tokio::test]
    async fn test_worker_crash_isolation() {
        let mut s = supervisor();
        let id = s.spawn_worker(AgentRole::Coder, AgentConfig::supervisor("test")).await.unwrap();
        s.handle_worker_crash(&id).await;
        assert_eq!(s.workers.read().await.get(&id).unwrap().status, WorkerStatus::Crashed);
    }

    #[tokio::test]
    async fn test_swarm_e2e() {
        let mut s = supervisor();
        let cfg = AgentConfig::supervisor("test");
        let coder = s.spawn_worker(AgentRole::Coder, cfg.clone()).await.unwrap();
        let researcher = s.spawn_worker(AgentRole::Researcher, cfg.clone()).await.unwrap();
        assert_eq!(s.worker_count(), 2);
        assert!(s.delegate(TaskAssignment { task_id: "code".to_string(), description: "Write".to_string(), assigned_to: coder, priority: 5 }).await.is_ok());
        assert!(s.delegate(TaskAssignment { task_id: "research".to_string(), description: "Search".to_string(), assigned_to: researcher, priority: 3 }).await.is_ok());
    }
}
