use crate::blackboard::Blackboard;
use crate::checkpoint::CheckpointManager;
use crate::governance::{AgentGovernance, DefaultGovernance};
use crate::planner::Planner;
use crate::reflector::Reflector;
use crate::swarm::Supervisor;
use crate::{AgentContext, AgentError};
use crate::agent_loop::AgentLoop;
use crate::edit_applier::EditApplier;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for constructing an AgentLoop.
/// Encapsulates all dependencies to satisfy clippy::too_many_arguments.
/// DEBT-LINES-B03A: Moved from agent_loop.rs to reduce file size.
pub struct AgentLoopConfig {
    pub context: AgentContext,
    pub planner: Arc<Mutex<dyn Planner>>,
    pub reflector: Arc<Mutex<dyn Reflector>>,
    pub governance: Arc<dyn AgentGovernance>,
    pub swarm: Option<Arc<Mutex<Supervisor>>>,
    pub blackboard: Arc<Blackboard>,
    pub checkpoint_mgr: Arc<CheckpointManager>,
    pub memory: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>,
    pub sync_gateway: Option<memory::sync_gateway::SyncGatewayHandle>,
    pub provider_id: Option<String>,
    pub edit_applier: Option<Arc<EditApplier>>,
}

pub struct AgentLoopBuilder {
    context: Option<AgentContext>,
    planner: Option<Arc<Mutex<dyn Planner>>>,
    reflector: Option<Arc<Mutex<dyn Reflector>>>,
    governance: Option<Arc<dyn AgentGovernance>>,
    swarm: Option<Option<Arc<Mutex<Supervisor>>>>,
    blackboard: Option<Arc<Blackboard>>,
    checkpoint_mgr: Option<Arc<CheckpointManager>>,
    memory: Option<Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>>,
    sync_gateway: Option<Option<memory::sync_gateway::SyncGatewayHandle>>,
    provider_id: Option<String>,
    edit_applier: Option<Option<Arc<EditApplier>>>,
}

impl AgentLoopBuilder {
    pub fn new() -> Self {
        Self { context: None, planner: None, reflector: None, governance: None, swarm: Some(None), blackboard: None, checkpoint_mgr: None, memory: Some(None), sync_gateway: Some(None), provider_id: None, edit_applier: Some(None) }
    }
    pub fn with_context(mut self, ctx: AgentContext) -> Self { self.context = Some(ctx); self }
    pub fn with_planner(mut self, p: Arc<Mutex<dyn Planner>>) -> Self { self.planner = Some(p); self }
    pub fn with_reflector(mut self, r: Arc<Mutex<dyn Reflector>>) -> Self { self.reflector = Some(r); self }
    pub fn with_governance(mut self, g: Arc<dyn AgentGovernance>) -> Self { self.governance = Some(g); self }
    pub fn with_swarm(mut self, s: Option<Arc<Mutex<Supervisor>>>) -> Self { self.swarm = Some(s); self }
    pub fn with_blackboard(mut self, bb: Arc<Blackboard>) -> Self { self.blackboard = Some(bb); self }
    pub fn with_checkpoint_mgr(mut self, cp: Arc<CheckpointManager>) -> Self { self.checkpoint_mgr = Some(cp); self }
    pub fn with_memory(mut self, m: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>) -> Self { self.memory = Some(m); self }
    pub fn with_sync_gateway(mut self, sg: Option<memory::sync_gateway::SyncGatewayHandle>) -> Self { self.sync_gateway = Some(sg); self }
    pub fn with_provider_id(mut self, id: Option<String>) -> Self { self.provider_id = id; self }
    pub fn with_edit_applier(mut self, ea: Option<Arc<EditApplier>>) -> Self { self.edit_applier = Some(ea); self }

    pub fn build(self) -> Result<AgentLoop, AgentError> {
        let context = self.context.unwrap_or_default();
        let planner = self.planner.ok_or_else(|| AgentError::Session("Planner is required".to_string()))?;
        let reflector = self.reflector.ok_or_else(|| AgentError::Session("Reflector is required".to_string()))?;
        let governance = self.governance.unwrap_or_else(|| Arc::new(DefaultGovernance::new()));
        let swarm = self.swarm.flatten();
        let blackboard = self.blackboard.unwrap_or_else(|| Arc::new(Blackboard::new()));
        let checkpoint_mgr = self.checkpoint_mgr.unwrap_or_else(|| Arc::new(CheckpointManager::new()));
        let memory = self.memory.flatten();
        let sync_gateway = self.sync_gateway.flatten();
        let edit_applier = self.edit_applier.flatten();
        let _iteration_count = Arc::new(Mutex::new(0));
        let _current_state = Arc::new(Mutex::new(crate::agent_loop::LoopState::Idle));
        let mut agent_loop = AgentLoop::from_components(AgentLoopConfig {
            context, planner, reflector, governance, swarm, blackboard, checkpoint_mgr, memory, sync_gateway, provider_id: self.provider_id, edit_applier: edit_applier.clone(),
        });
        if let Some(ea) = edit_applier {
            agent_loop = agent_loop.with_edit_applier(ea);
        }
        Ok(agent_loop)
    }
}

impl Default for AgentLoopBuilder {
    fn default() -> Self { Self::new() }
}
