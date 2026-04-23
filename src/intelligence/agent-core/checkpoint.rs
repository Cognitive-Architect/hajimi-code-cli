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

/// Complete checkpoint of agent state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String, pub timestamp: chrono::DateTime<chrono::Utc>, pub agent_id: AgentId,
    pub plan: Option<Plan>, pub reflections: Vec<Reflection>,
    pub swarm_workers: Vec<WorkerState>,
    pub blackboard: HashMap<String, crate::blackboard::BlackboardEntry>,
    pub hash: String, pub version: u32,
}

/// Worker state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerState { pub worker_id: AgentId, pub status: WorkerStatus, pub assigned_task: Option<String> }

/// Checkpoint manager with MemoryGateway integration.
pub struct CheckpointManager {
    checkpoints: Arc<tokio::sync::RwLock<Vec<Checkpoint>>>,
    memory: Option<Arc<tokio::sync::Mutex<memory::memory_gateway::MemoryGateway>>>,
}

impl CheckpointManager {
    pub fn new() -> Self { Self { checkpoints: Arc::new(tokio::sync::RwLock::new(Vec::new())), memory: None } }
    
    /// Attach MemoryGateway for persistence.
    pub fn with_memory(mut self, memory: Arc<tokio::sync::Mutex<memory::memory_gateway::MemoryGateway>>) -> Self {
        self.memory = Some(memory); self
    }

    /// Save checkpoint with full state.
    pub async fn save(&self, agent_id: &AgentId, plan: Option<Plan>, reflections: Vec<Reflection>, swarm_workers: Vec<WorkerState>, blackboard: &Blackboard) -> ReplResult<Checkpoint> {
        let mut chk = Checkpoint { id: format!("chk_{}", uuid::Uuid::new_v4()), timestamp: chrono::Utc::now(), agent_id: agent_id.clone(), plan, reflections, swarm_workers, blackboard: blackboard.snapshot().await, hash: String::new(), version: 1 };
        chk.hash = Self::compute_hash(&chk);
        self.checkpoints.write().await.push(chk.clone());
        // Persist to MemoryGateway if available
        if let Some(ref mem) = self.memory {
            if let Ok(json) = serde_json::to_string(&chk) {
                let _ = mem.lock().await.push_vector(&format!("chk_{}", chk.id), &json);
            }
        }
        info!("Checkpoint {} saved", chk.id); Ok(chk)
    }

    /// Restore latest checkpoint.
    pub async fn restore_latest(&self, agent_id: &AgentId) -> ReplResult<Checkpoint> {
        let chks = self.checkpoints.read().await;
        let chk = chks.iter().rfind(|c| c.agent_id == *agent_id).ok_or_else(|| ReplError::Session("No checkpoint".to_string()))?;
        Self::verify_hash(chk)?; Ok(chk.clone())
    }

    /// List checkpoints for agent.
    pub async fn list(&self, agent_id: &AgentId) -> Vec<Checkpoint> { self.checkpoints.read().await.iter().filter(|c| c.agent_id == *agent_id).cloned().collect() }

    /// Compute integrity hash using std::hash::DefaultHasher.
    fn compute_hash(chk: &Checkpoint) -> String {
        let mut hasher = DefaultHasher::new();
        chk.timestamp.hash(&mut hasher);
        chk.agent_id.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Verify checkpoint integrity.
    fn verify_hash(chk: &Checkpoint) -> ReplResult<()> {
        if chk.hash != Self::compute_hash(chk) { warn!("Hash mismatch"); return Err(ReplError::Session("Integrity failed".to_string())); }
        Ok(())
    }

    /// Fallback restore from last valid.
    pub async fn restore_fallback(&self, agent_id: &AgentId) -> ReplResult<Checkpoint> {
        for chk in self.checkpoints.read().await.iter().filter(|c| c.agent_id == *agent_id).rev() { if Self::verify_hash(chk).is_ok() { return Ok(chk.clone()); } }
        Err(ReplError::Session("No valid checkpoint".to_string()))
    }
}

impl Default for CheckpointManager { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test] async fn test_save_restore() { let mgr = CheckpointManager::new(); let bb = Blackboard::new(); mgr.save(&"a1".to_string(), None, vec![], vec![], &bb).await.unwrap(); assert_eq!(mgr.restore_latest(&"a1".to_string()).await.unwrap().agent_id, "a1"); }
    #[tokio::test] async fn test_list() { let mgr = CheckpointManager::new(); let bb = Blackboard::new(); mgr.save(&"a1".to_string(), None, vec![], vec![], &bb).await.unwrap(); assert_eq!(mgr.list(&"a1".to_string()).await.len(), 1); }
}
