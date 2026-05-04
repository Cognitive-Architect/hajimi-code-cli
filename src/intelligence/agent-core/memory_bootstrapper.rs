use crate::agent_loop::AgentLoop;
use crate::agent_loop_builder::AgentLoopBuilder;
use crate::blackboard::Blackboard;
use crate::checkpoint::{Checkpoint, CheckpointManager};
use crate::{AgentContext, AgentError};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Result of loading project memory, carrying the shared gateway and restored checkpoint state.
pub struct BootstrapResult {
    pub gateway: Arc<Mutex<memory::memory_gateway::MemoryGateway>>,
    pub checkpoint_mgr: CheckpointManager,
    pub summary: String,
}

/// # Safety: MemoryBootstrapper coordinates initialization order to prevent race conditions
/// between Checkpoint restore and MemoryGateway tier enablement. All async operations
/// are serialized within each bootstrap call.
pub struct MemoryBootstrapper {
    project_id: String,
    device_id: String,
    agent_id: String,
}

impl MemoryBootstrapper {
    pub fn new(project_id: &str, device_id: &str, agent_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            device_id: device_id.to_string(),
            agent_id: agent_id.to_string(),
        }
    }

    /// Initialize MemoryGateway, enable Auto/Graph/Dream tiers, restore latest Checkpoint,
    /// and generate a human-readable project memory summary.
    pub async fn load_project_memory(&self) -> Result<BootstrapResult, AgentError> {
        let mut gateway =
            memory::memory_gateway::MemoryGateway::new_with_project(&self.device_id, Some(&self.project_id));
        let _ = gateway.enable_auto(&self.project_id);
        gateway.enable_graph(&self.project_id);
        let _ = gateway.enable_dream(&self.project_id);
        let gateway_arc = Arc::new(Mutex::new(gateway));
        let checkpoint_mgr = CheckpointManager::new().with_memory(gateway_arc.clone());
        let checkpoint = checkpoint_mgr
            .restore_latest_from_disk(&self.project_id, &self.agent_id)
            .await
            .ok();
        let summary = Self::generate_summary(&checkpoint);
        Ok(BootstrapResult {
            gateway: gateway_arc,
            checkpoint_mgr,
            summary,
        })
    }

    /// Build a fully configured AgentLoop using the shared MemoryGateway from load_project_memory.
    /// Injects project_memory_summary into the Blackboard before returning.
    pub async fn build_agent_loop_with_memory(&self) -> Result<AgentLoop, AgentError> {
        let result = self.load_project_memory().await?;
        let blackboard = Arc::new(Blackboard::new());
        blackboard
            .write("project_memory_summary", &result.summary, &self.agent_id)
            .await;
        let context = AgentContext::new();
        let planner = Arc::new(Mutex::new(crate::planner::HierarchicalPlanner::new(
            result.gateway.clone(),
            context.clone(),
        )));
        let reflector = Arc::new(Mutex::new(crate::reflector::AutonomousReflector::new(
            result.gateway.clone(),
            context.clone(),
        )));
        AgentLoopBuilder::production_ready(&self.device_id)
            .with_context(context)
            .with_planner(planner)
            .with_reflector(reflector)
            .with_memory(Some(result.gateway.clone()))
            .with_blackboard(blackboard)
            .with_checkpoint_mgr(Arc::new(result.checkpoint_mgr))
            .build()
    }

    fn generate_summary(checkpoint: &Option<Checkpoint>) -> String {
        match checkpoint {
            Some(cp) => {
                let plan_summary = cp.plan.as_ref().map(|_| "active").unwrap_or("none");
                let reflection_count = cp.reflections.len();
                let goal_progress = cp
                    .goal_progress
                    .map(|p| format!("{:.0}%", p * 100.0))
                    .unwrap_or_else(|| "N/A".to_string());
                format!(
                    "plan_summary={}; reflections={}; goal_progress={}",
                    plan_summary, reflection_count, goal_progress
                )
            }
            None => "No checkpoint available".to_string(),
        }
    }
}
