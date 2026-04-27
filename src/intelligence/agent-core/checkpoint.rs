//! Agent Checkpoint: Complete state persistence for recovery.
use crate::blackboard::Blackboard;
use crate::planner::Plan;
use crate::reflector::Reflection;
use crate::swarm::WorkerStatus;
use crate::AgentId;
use chimera_repl::traits::{ReplError, ReplResult};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{info, warn};

const MAX_CHECKPOINTS_PER_AGENT: usize = 100;

/// Complete checkpoint of agent state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String, pub timestamp: chrono::DateTime<chrono::Utc>, pub agent_id: AgentId,
    pub plan: Option<Plan>, pub reflections: Vec<Reflection>,
    pub swarm_workers: Vec<WorkerState>,
    pub blackboard: HashMap<String, crate::blackboard::BlackboardEntry>,
    pub hash: String, pub version: u32,
    #[serde(default)]
    pub goal_progress: Option<f32>,
    #[serde(default)]
    pub key_reflection: Option<String>,
}

/// Worker state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerState { pub worker_id: AgentId, pub status: WorkerStatus, pub assigned_task: Option<String> }

/// Checkpoint manager with MemoryGateway integration.
pub struct CheckpointManager {
    checkpoints: Arc<tokio::sync::RwLock<Vec<Checkpoint>>>,
    memory: Option<Arc<tokio::sync::Mutex<memory::memory_gateway::MemoryGateway>>>,
    resource_monitor: Option<Arc<crate::resource_monitor::ResourceMonitor>>,
}

impl CheckpointManager {
    pub fn new() -> Self { Self { checkpoints: Arc::new(tokio::sync::RwLock::new(Vec::new())), memory: None, resource_monitor: None } }
    pub fn with_memory(mut self, memory: Arc<tokio::sync::Mutex<memory::memory_gateway::MemoryGateway>>) -> Self { self.memory = Some(memory); self }
    pub fn with_resource_monitor(mut self, monitor: Arc<crate::resource_monitor::ResourceMonitor>) -> Self { self.resource_monitor = Some(monitor); self }

    pub async fn save(&self, agent_id: &AgentId, plan: Option<Plan>, reflections: Vec<Reflection>, swarm_workers: Vec<WorkerState>, blackboard: &Blackboard) -> ReplResult<Checkpoint> {
        let mut chk = Checkpoint { id: format!("chk_{}", uuid::Uuid::new_v4()), timestamp: chrono::Utc::now(), agent_id: agent_id.clone(), plan, reflections, swarm_workers, blackboard: blackboard.snapshot().await, hash: String::new(), version: 1, goal_progress: None, key_reflection: None };
        chk.hash = Self::compute_hash(&chk);
        {
            let mut chks = self.checkpoints.write().await;
            chks.push(chk.clone());
            // Prune old checkpoints for this agent
            let agent_count = chks.iter().filter(|c| c.agent_id == *agent_id).count();
            if agent_count > MAX_CHECKPOINTS_PER_AGENT {
                let to_remove = agent_count - MAX_CHECKPOINTS_PER_AGENT;
                let mut removed = 0usize;
                chks.retain(|c| {
                    if c.agent_id == *agent_id && removed < to_remove {
                        removed += 1;
                        false
                    } else {
                        true
                    }
                });
                info!("Pruned {} old checkpoints for agent {}", removed, agent_id);
            }
        }
        if let Some(ref mem) = self.memory {
            if let Ok(json) = serde_json::to_string(&chk) { let _ = mem.lock().await.push_vector(&format!("chk_{}", chk.id), &json); }
        }
        if let Some(ref monitor) = self.resource_monitor {
            let count = self.checkpoints.read().await.iter().filter(|c| c.agent_id == *agent_id).count();
            monitor.record_checkpoint_count(count);
        }
        info!("Checkpoint {} saved", chk.id); Ok(chk)
    }

    pub async fn restore_latest(&self, agent_id: &AgentId) -> ReplResult<Checkpoint> {
        let chks = self.checkpoints.read().await;
        let chk = chks.iter().rfind(|c| c.agent_id == *agent_id).ok_or_else(|| ReplError::Session("No checkpoint".to_string()))?;
        Self::verify_hash(chk)?; Ok(chk.clone())
    }

    pub async fn list(&self, agent_id: &AgentId) -> Vec<Checkpoint> {
        let mut list: Vec<_> = self.checkpoints.read().await.iter().filter(|c| c.agent_id == *agent_id).cloned().collect();
        list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        list
    }

    pub async fn restore(&self, checkpoint_id: &str) -> ReplResult<Checkpoint> {
        let chks = self.checkpoints.read().await;
        let chk = chks.iter().find(|c| c.id == checkpoint_id).ok_or_else(|| ReplError::Session(format!("Checkpoint {} not found", checkpoint_id)))?;
        Self::verify_hash(chk)?; Ok(chk.clone())
    }

    pub async fn compare(&self, id_a: &str, id_b: &str) -> ReplResult<bool> {
        let chks = self.checkpoints.read().await;
        let a = chks.iter().find(|c| c.id == id_a).ok_or_else(|| ReplError::Session(format!("Checkpoint {} not found", id_a)))?;
        let b = chks.iter().find(|c| c.id == id_b).ok_or_else(|| ReplError::Session(format!("Checkpoint {} not found", id_b)))?;
        Ok(a.hash == b.hash)
    }

    pub async fn export(&self, checkpoint_id: &str) -> ReplResult<String> {
        let chks = self.checkpoints.read().await;
        let chk = chks.iter().find(|c| c.id == checkpoint_id).ok_or_else(|| ReplError::Session(format!("Checkpoint {} not found", checkpoint_id)))?;
        serde_json::to_string(chk).map_err(|e| ReplError::Session(format!("Serialize error: {}", e)))
    }

    pub async fn export_all(&self, agent_id: &AgentId) -> ReplResult<String> {
        let list = self.list(agent_id).await;
        serde_json::to_string(&list).map_err(|e| ReplError::Session(format!("Serialize error: {}", e)))
    }

    fn compute_hash(chk: &Checkpoint) -> String {
        let mut hasher = DefaultHasher::new();
        chk.timestamp.hash(&mut hasher); chk.agent_id.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn verify_hash(chk: &Checkpoint) -> ReplResult<()> {
        if chk.hash != Self::compute_hash(chk) { warn!("Hash mismatch"); return Err(ReplError::Session("Integrity failed".to_string())); }
        Ok(())
    }

    pub async fn restore_fallback(&self, agent_id: &AgentId) -> ReplResult<Checkpoint> {
        for chk in self.checkpoints.read().await.iter().filter(|c| c.agent_id == *agent_id).rev() { if Self::verify_hash(chk).is_ok() { return Ok(chk.clone()); } }
        Err(ReplError::Session("No valid checkpoint".to_string()))
    }

    pub async fn restore_from_memory(&self, agent_id: &AgentId) -> ReplResult<Checkpoint> {
        if let Some(ref mem) = self.memory {
            let mem_guard = mem.lock().await; let prefix = format!("chk_{}", agent_id);
            for key in mem_guard.session.keys() {
                if key.starts_with(&prefix) {
                    if let Some(entry) = mem_guard.session.get(key) {
                        if let Ok(chk) = serde_json::from_str::<Checkpoint>(&entry.content) {
                            if chk.agent_id == *agent_id && Self::verify_hash(&chk).is_ok() { return Ok(chk); }
                        }
                    }
                }
            }
        }
        Err(ReplError::Session("No checkpoint in memory".to_string()))
    }
}

impl Default for CheckpointManager { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_save_restore() { let mgr = CheckpointManager::new(); let bb = Blackboard::new(); mgr.save(&"a1".to_string(), None, vec![], vec![], &bb).await.unwrap(); assert_eq!(mgr.restore_latest(&"a1".to_string()).await.unwrap().agent_id, "a1"); }
    #[tokio::test] async fn test_list() { let mgr = CheckpointManager::new(); let bb = Blackboard::new(); mgr.save(&"a1".to_string(), None, vec![], vec![], &bb).await.unwrap(); assert_eq!(mgr.list(&"a1".to_string()).await.len(), 1); }
}
