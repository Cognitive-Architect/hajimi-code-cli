//! Swarm Coordinator: Supervisor-Worker pattern for multi-agent collaboration.
//! Day 6: Dynamic agent spawning, task delegation, and result aggregation.
//! Phase 2 Day 2: Callback registration, result dispatch, retry with backoff.
//! DEBT-LINES-B04A: Worker lifecycle management extracted to worker_lifecycle_manager.rs.

use crate::{AgentConfig, AgentId, AgentRole};
use crate::ports::{WorkerCallback, WorkerMetrics, WorkerResultStatus};
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::worker_lifecycle_manager::WorkerLifecycleManager;
use chimera_repl::traits::{ReplError, ReplResult};
use engine_tool_system::ToolRegistry;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
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
    pub error: Option<String>,
    pub status: WorkerResultStatus,
    pub metrics: Option<WorkerMetrics>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WorkerResult {
    pub fn success(task_id: impl Into<String>, worker_id: AgentId, output: impl Into<String>, metrics: WorkerMetrics) -> Self {
        let ts = metrics.timestamp;
        Self { task_id: task_id.into(), worker_id, success: true, output: output.into(), error: None, status: WorkerResultStatus::Success, metrics: Some(metrics), timestamp: ts }
    }
    pub fn failure(task_id: impl Into<String>, worker_id: AgentId, error: impl Into<String>, metrics: WorkerMetrics) -> Self {
        let ts = metrics.timestamp;
        Self { task_id: task_id.into(), worker_id, success: false, output: String::new(), error: Some(error.into()), status: WorkerResultStatus::Failed, metrics: Some(metrics), timestamp: ts }
    }
}

/// Worker container holding agent instance and metadata.
pub struct Worker {
    pub config: AgentConfig,
    pub tx: tokio::sync::mpsc::Sender<SwarmMessage>,
    pub status: WorkerStatus,
    pub spawn_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkerStatus { Idle, Busy, Crashed, Stopped, PermanentlyFailed }

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
    async fn delegate(&self, task: TaskAssignment) -> ReplResult<()>;
    async fn spawn_worker(&mut self, role: AgentRole, config: AgentConfig) -> ReplResult<AgentId>;
    async fn stop_worker(&mut self, worker_id: &AgentId) -> ReplResult<()>;
    async fn restart_worker(&mut self, worker_id: &AgentId) -> ReplResult<AgentId>;
    async fn aggregate(&self) -> Vec<WorkerResult>;
    fn worker_count(&self) -> usize;
}

/// Supervisor coordinates multiple Worker agents.
pub struct SupervisorMetrics {
    pub total_tasks: AtomicUsize,
    pub successful_tasks: AtomicUsize,
    pub failed_tasks: AtomicUsize,
    pub total_execution_time_ms: AtomicU64,
    pub callback_latency_ms: AtomicU64,
}

impl Default for SupervisorMetrics {
    fn default() -> Self { Self::new() }
}

impl SupervisorMetrics {
    pub fn new() -> Self { Self { total_tasks: AtomicUsize::new(0), successful_tasks: AtomicUsize::new(0), failed_tasks: AtomicUsize::new(0), total_execution_time_ms: AtomicU64::new(0), callback_latency_ms: AtomicU64::new(0) } }
}

pub struct Supervisor {
    lifecycle: WorkerLifecycleManager,
    governance: Arc<dyn AgentGovernance>,
    context: crate::AgentContext,
    tool_registry: Option<Arc<Mutex<ToolRegistry>>>,
    callback: Option<Arc<dyn WorkerCallback>>,
    metrics: Arc<SupervisorMetrics>,
    max_concurrent_tasks: usize,
}

impl Supervisor {
    pub fn new(governance: Arc<dyn AgentGovernance>, context: crate::AgentContext) -> Self {
        let lifecycle = WorkerLifecycleManager::new(governance.clone(), context.clone());
        Self { lifecycle, governance, context, tool_registry: None, callback: None, metrics: Arc::new(SupervisorMetrics::new()), max_concurrent_tasks: 100 }
    }
    pub fn with_max_concurrent_tasks(mut self, max: usize) -> Self { self.max_concurrent_tasks = max; self }
    pub fn with_tool_registry(mut self, registry: Arc<Mutex<ToolRegistry>>) -> Self { self.tool_registry = Some(registry); self }
    pub fn register_callback(&mut self, callback: Arc<dyn WorkerCallback>) { self.callback = Some(callback); }
    pub fn set_callback(&mut self, callback: Arc<dyn WorkerCallback>) { self.callback = Some(callback); }

    /// Handle a worker result: store, notify callback, retry if failed, update metrics.
    pub async fn handle_worker_result(&self, result: WorkerResult) {
        let tid = result.task_id.clone();
        let wid = result.worker_id.clone();
        self.metrics.total_tasks.fetch_add(1, Ordering::Relaxed);
        if result.success { self.metrics.successful_tasks.fetch_add(1, Ordering::Relaxed); }
        else { self.metrics.failed_tasks.fetch_add(1, Ordering::Relaxed); }
        if let Some(ref m) = result.metrics { self.metrics.total_execution_time_ms.fetch_add(m.execution_time_ms, Ordering::Relaxed); }
        let truncated = Self::truncate_output(&result.output, 1_000_000);
        let mut r = result.clone();
        if truncated.len() < result.output.len() { r.output = truncated; r.error = Some("truncated".to_string()); }
        self.lifecycle.results().lock().await.push(r.clone());
        let cb_start = std::time::Instant::now();
        if let Some(ref cb) = self.callback {
            if r.success { cb.on_success(&tid, &wid, &r.output, &WorkerMetrics::new(0)).await; }
            else { cb.on_failure(&tid, &wid, &r.output, &WorkerMetrics::new(0)).await; }
        }
        self.metrics.callback_latency_ms.fetch_add(cb_start.elapsed().as_millis() as u64, Ordering::Relaxed);
        if !r.success {
            let retry_counts = self.lifecycle.retry_counts();
            let mut rc = retry_counts.lock().await;
            let count = rc.entry(tid.clone()).or_insert(0);
            *count += 1;
            let c = *count;
            drop(rc);
            if c <= 3 {
                let backoff = 100u64 * (1u64 << (c - 1));
                warn!(task_id=%tid, attempt=c, backoff_ms=backoff, "Retrying failed task");
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff)).await;
            } else {
                warn!(task_id=%tid, "Retry exhausted after 3 attempts");
            }
        } else {
            self.lifecycle.retry_counts().lock().await.remove(&tid);
        }
    }

    fn truncate_output(output: &str, max_size: usize) -> String {
        if output.len() <= max_size { return output.to_string(); }
        if output.is_empty() { return String::new(); }
        let mut end = max_size;
        while end > 0 && !output.is_char_boundary(end) { end -= 1; }
        format!("{}...[truncated]", &output[..end])
    }

    async fn approve_delegation(&self, task: &TaskAssignment) -> ReplResult<bool> {
        let req = GovernanceRequest { requester: "supervisor".to_string(), action_type: "delegate_task".to_string(), risk_score: task.priority as f32 / 20.0, description: task.description.clone(), level: if task.priority > 7 { ApprovalLevel::Critical } else { ApprovalLevel::Auto } };
        Ok(matches!(self.governance.approve(&self.context, &req).await?, Decision::Approved))
    }

    pub async fn spawn_worker(&mut self, role: AgentRole, config: AgentConfig) -> ReplResult<AgentId> {
        self.lifecycle.spawn_worker(role, config, self.tool_registry.clone(), self.callback.clone()).await
    }

    pub async fn stop_worker(&mut self, worker_id: &AgentId) -> ReplResult<()> {
        self.lifecycle.stop_worker(worker_id).await
    }

    pub async fn restart_worker(&mut self, worker_id: &AgentId) -> ReplResult<AgentId> {
        self.lifecycle.restart_worker(worker_id, self.tool_registry.clone(), self.callback.clone()).await
    }

    pub async fn handle_worker_crash(&mut self, worker_id: &AgentId) {
        self.lifecycle.handle_worker_crash(worker_id, self.tool_registry.clone(), self.callback.clone()).await;
    }

    pub fn metrics(&self) -> Arc<SupervisorMetrics> { self.metrics.clone() }

    pub fn worker_count(&self) -> usize { self.lifecycle.workers().try_read().map(|w| w.len()).unwrap_or(0) }

    pub async fn retry_task(&self, task: TaskAssignment) -> ReplResult<()> {
        info!("Retrying task {} for worker {}", task.task_id, task.assigned_to);
        self.delegate(task).await
    }

    /// Pop a result for a specific task_id from the results queue.
    pub async fn pop_result(&self, task_id: &str) -> Option<WorkerResult> {
        let results_arc = self.lifecycle.results();
        let mut results = results_arc.lock().await;
        results.iter().position(|r| r.task_id == task_id).map(|pos| results.remove(pos))
    }

    /// Find an idle worker and return its ID.
    pub async fn find_idle_worker(&self) -> Option<AgentId> {
        let workers_arc = self.lifecycle.workers();
        let workers = workers_arc.read().await;
        workers.iter().find(|(_, w)| w.status == WorkerStatus::Idle).map(|(id, _)| id.clone())
    }
}

#[async_trait]
impl SwarmCoordinator for Supervisor {
    async fn delegate(&self, task: TaskAssignment) -> ReplResult<()> {
        if !self.approve_delegation(&task).await? { return Err(ReplError::Session("Delegation rejected".to_string())); }
        // Detect duplicate task_id already in-flight
        if self.lifecycle.results().lock().await.iter().any(|r| r.task_id == task.task_id) {
            return Err(ReplError::Session(format!("Task {} already assigned or completed", task.task_id)));
        }
        if let Some(worker) = self.lifecycle.workers().read().await.get(&task.assigned_to) {
            if worker.status == WorkerStatus::Idle { let _ = worker.tx.send(SwarmMessage::TaskAssigned(task)).await; return Ok(()); }
        }
        Err(ReplError::Session("Worker unavailable".to_string()))
    }

    async fn spawn_worker(&mut self, role: AgentRole, config: AgentConfig) -> ReplResult<AgentId> {
        self.lifecycle.spawn_worker(role, config, self.tool_registry.clone(), self.callback.clone()).await
    }

    async fn stop_worker(&mut self, worker_id: &AgentId) -> ReplResult<()> {
        self.lifecycle.stop_worker(worker_id).await
    }

    async fn restart_worker(&mut self, worker_id: &AgentId) -> ReplResult<AgentId> {
        self.lifecycle.restart_worker(worker_id, self.tool_registry.clone(), self.callback.clone()).await
    }

    async fn aggregate(&self) -> Vec<WorkerResult> { self.lifecycle.results().lock().await.clone() }
    fn worker_count(&self) -> usize { self.lifecycle.workers().try_read().map(|w| w.len()).unwrap_or(0) }
}

#[cfg(test)]
mod tests;
