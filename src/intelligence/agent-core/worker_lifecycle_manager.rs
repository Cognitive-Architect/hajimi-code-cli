//! DEBT-LINES-B04A: Extracted worker lifecycle management from swarm.rs
//! Handles worker spawning, stopping, restarting, and crash recovery.

use crate::governance::AgentGovernance;
use crate::ports::{WorkerCallback, WorkerMetrics, WorkerResultStatus};
use crate::swarm::{SwarmMessage, Worker, WorkerResult, WorkerStatus};
use crate::{AgentConfig, AgentId, AgentRole};
use chimera_repl::traits::{ReplError, ReplResult};
use engine_tool_system::{ToolArgs, ToolRegistry};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{info, warn};

/// Manages the full lifecycle of Swarm workers.
pub struct WorkerLifecycleManager {
    workers: Arc<RwLock<HashMap<AgentId, Worker>>>,
    results: Arc<Mutex<Vec<WorkerResult>>>,
    retry_counts: Arc<Mutex<HashMap<String, u32>>>,
    restart_counts: Arc<Mutex<HashMap<AgentId, u8>>>,
    #[allow(dead_code)]
    governance: Arc<dyn AgentGovernance>,
    #[allow(dead_code)]
    context: crate::AgentContext,
}

impl WorkerLifecycleManager {
    pub fn new(governance: Arc<dyn AgentGovernance>, context: crate::AgentContext) -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            retry_counts: Arc::new(Mutex::new(HashMap::new())),
            restart_counts: Arc::new(Mutex::new(HashMap::new())),
            governance,
            context,
        }
    }

    pub fn workers(&self) -> Arc<RwLock<HashMap<AgentId, Worker>>> {
        self.workers.clone()
    }
    pub fn results(&self) -> Arc<Mutex<Vec<WorkerResult>>> {
        self.results.clone()
    }
    pub fn retry_counts(&self) -> Arc<Mutex<HashMap<String, u32>>> {
        self.retry_counts.clone()
    }
    pub fn restart_counts(&self) -> Arc<Mutex<HashMap<AgentId, u8>>> {
        self.restart_counts.clone()
    }

    pub fn role_str(role: &AgentRole) -> &'static str {
        match role {
            AgentRole::Coder => "coder",
            AgentRole::Researcher => "researcher",
            AgentRole::Critic => "critic",
            AgentRole::Executor => "executor",
            _ => "worker",
        }
    }

    pub async fn spawn_worker(
        &mut self,
        role: AgentRole,
        mut config: AgentConfig,
        tool_registry: Option<Arc<Mutex<ToolRegistry>>>,
        callback: Option<Arc<dyn WorkerCallback>>,
    ) -> ReplResult<AgentId> {
        let id = format!("{}_{}", Self::role_str(&role), uuid::Uuid::new_v4());
        let id_for_spawn = id.clone();
        config.agent_id = id.clone();
        config.role = role;
        let (tx, mut rx) = mpsc::channel(100);
        let worker = Worker {
            config: config.clone(),
            tx,
            status: WorkerStatus::Idle,
            spawn_time: chrono::Utc::now(),
        };
        self.workers.write().await.insert(id.clone(), worker);
        let workers = self.workers.clone();
        let results = self.results.clone();
        let retry_counts = self.retry_counts.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    SwarmMessage::TaskAssigned(task) => {
                        info!("Worker {} processing {}", id_for_spawn, task.task_id);
                        let tr = tool_registry.clone();
                        let ic = id_for_spawn.clone();
                        let tid = task.task_id.clone();
                        let desc = task.description.clone();
                        let prio = task.priority;
                        let cb = callback.clone();
                        let rs = results.clone();
                        let ws = workers.clone();
                        let rc = retry_counts.clone();
                        tokio::spawn(async move {
                            let (success, output) = if let Some(registry) = tr {
                                let reg = registry.lock().await;
                                if let Some(tool) = reg.get("planning") {
                                    match tool.execute(ToolArgs::from(json!({"action":"create_goal","description":desc,"priority":prio}))).await {
                                        Ok(out) => { info!("Worker {} executed: {}", ic, out.stdout); (true, out.stdout) }
                                        Err(e) => { warn!("Worker {} failed: {}", ic, e.message); (false, e.message) }
                                    }
                                } else {
                                    (false, "Tool not found".to_string())
                                }
                            } else {
                                (false, "No registry".to_string())
                            };
                            let wr = WorkerResult {
                                task_id: tid.clone(),
                                worker_id: ic.clone(),
                                success,
                                output: output.clone(),
                                error: if success { None } else { Some(output.clone()) },
                                status: if success {
                                    WorkerResultStatus::Success
                                } else {
                                    WorkerResultStatus::Failed
                                },
                                metrics: Some(WorkerMetrics::new(0)),
                                timestamp: chrono::Utc::now(),
                            };
                            rs.lock().await.push(wr.clone());
                            if let Some(ref c) = cb {
                                if success {
                                    c.on_success(&tid, &ic, &output, &WorkerMetrics::new(0))
                                        .await;
                                } else {
                                    c.on_failure(&tid, &ic, &output, &WorkerMetrics::new(0))
                                        .await;
                                }
                            }
                            if !success {
                                let mut counts = rc.lock().await;
                                let cnt = counts.entry(tid.clone()).or_insert(0);
                                *cnt += 1;
                                let c = *cnt;
                                drop(counts);
                                if c <= 3 {
                                    let backoff = 100u64 * (1u64 << (c - 1));
                                    warn!(task_id=%tid, attempt=c, backoff_ms=backoff, "Retrying failed task");
                                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff))
                                        .await;
                                } else {
                                    warn!(task_id=%tid, "Retry exhausted after 3 attempts");
                                }
                            }
                            if let Some(w) = ws.write().await.get_mut(&ic) {
                                w.status = WorkerStatus::Idle;
                            }
                        });
                        if let Some(w) = workers.write().await.get_mut(&id_for_spawn) {
                            w.status = WorkerStatus::Busy;
                        }
                    }
                    SwarmMessage::TaskCompleted(result) => {
                        results.lock().await.push(result.clone());
                        if let Some(ref c) = callback {
                            if result.success {
                                c.on_success(
                                    &result.task_id,
                                    &result.worker_id,
                                    &result.output,
                                    &WorkerMetrics::new(0),
                                )
                                .await;
                            } else {
                                c.on_failure(
                                    &result.task_id,
                                    &result.worker_id,
                                    &result.output,
                                    &WorkerMetrics::new(0),
                                )
                                .await;
                            }
                        }
                        if let Some(w) = workers.write().await.get_mut(&id_for_spawn) {
                            w.status = WorkerStatus::Idle;
                        }
                    }
                    SwarmMessage::Shutdown => {
                        if let Some(w) = workers.write().await.get_mut(&id_for_spawn) {
                            w.status = WorkerStatus::Stopped;
                        }
                        break;
                    }
                }
            }
        });
        Ok(id)
    }

    pub async fn stop_worker(&mut self, worker_id: &AgentId) -> ReplResult<()> {
        let tx = {
            self.workers
                .read()
                .await
                .get(worker_id)
                .map(|w| w.tx.clone())
        };
        if let Some(t) = tx {
            let _ = t.send(SwarmMessage::Shutdown).await;
        }
        self.workers.write().await.remove(worker_id);
        Ok(())
    }

    pub async fn restart_worker(
        &mut self,
        worker_id: &AgentId,
        tool_registry: Option<Arc<Mutex<ToolRegistry>>>,
        callback: Option<Arc<dyn WorkerCallback>>,
    ) -> ReplResult<AgentId> {
        let old_config = {
            self.workers
                .read()
                .await
                .get(worker_id)
                .map(|w| w.config.clone())
        };
        let old_count = { self.restart_counts.lock().await.remove(worker_id) };
        if let Some(config) = old_config {
            self.stop_worker(worker_id).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let new_id = self
                .spawn_worker(config.role, config, tool_registry, callback)
                .await?;
            if let Some(c) = old_count {
                *self
                    .restart_counts
                    .lock()
                    .await
                    .entry(new_id.clone())
                    .or_insert(0) += c;
            }
            Ok(new_id)
        } else {
            Err(ReplError::Session("Worker not found".to_string()))
        }
    }

    pub async fn handle_worker_crash(
        &mut self,
        worker_id: &AgentId,
        tool_registry: Option<Arc<Mutex<ToolRegistry>>>,
        callback: Option<Arc<dyn WorkerCallback>>,
    ) {
        warn!("Worker {} crashed, isolating", worker_id);
        if let Some(w) = self.workers.write().await.get_mut(worker_id) {
            w.status = WorkerStatus::Crashed;
        }
        let mut rc = self.restart_counts.lock().await;
        let count = rc.entry(worker_id.clone()).or_insert(0);
        *count += 1;
        let c = *count;
        drop(rc);
        if c <= 3 {
            warn!(worker_id=%worker_id, attempt=c, "Restarting crashed worker");
            let _ = self
                .restart_worker(worker_id, tool_registry, callback)
                .await;
        } else {
            warn!(worker_id=%worker_id, "Worker permanently failed after 3 restarts");
            if let Some(w) = self.workers.write().await.get_mut(worker_id) {
                w.status = WorkerStatus::PermanentlyFailed;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultGovernance;

    #[tokio::test]
    async fn test_lifecycle_manager_spawn_stop() {
        let mut lcm = WorkerLifecycleManager::new(
            Arc::new(DefaultGovernance::new()),
            crate::AgentContext::new(),
        );
        let id = lcm
            .spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t"), None, None)
            .await
            .unwrap();
        assert!(id.contains("coder"));
        assert_eq!(lcm.workers().read().await.len(), 1);
        lcm.stop_worker(&id).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(lcm.workers().read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_lifecycle_manager_restart() {
        let mut lcm = WorkerLifecycleManager::new(
            Arc::new(DefaultGovernance::new()),
            crate::AgentContext::new(),
        );
        let id = lcm
            .spawn_worker(AgentRole::Critic, AgentConfig::supervisor("t"), None, None)
            .await
            .unwrap();
        let nid = lcm.restart_worker(&id, None, None).await.unwrap();
        assert_ne!(id, nid);
        assert_eq!(lcm.workers().read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_lifecycle_manager_crash_recovery() {
        let mut lcm = WorkerLifecycleManager::new(
            Arc::new(DefaultGovernance::new()),
            crate::AgentContext::new(),
        );
        let id = lcm
            .spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t"), None, None)
            .await
            .unwrap();
        lcm.handle_worker_crash(&id, None, None).await;
        assert_eq!(lcm.workers().read().await.len(), 1);
    }
}
