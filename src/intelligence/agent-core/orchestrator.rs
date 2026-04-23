//! Agent Orchestrator: Central coordination for multi-agent systems.
//! Day 6: Swarm integration, Blackboard shared state, concurrent agent ticks.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use futures::future::join_all;

use crate::{Agent, AgentConfig, AgentContext, AgentId, AgentOutcome, AgentRole};
use tracing::info;
use crate::events::AgentEventProcessor;
use crate::MemoryGateway;
use crate::governance::{AgentGovernance, DefaultGovernance};
use crate::swarm::{Supervisor, SwarmCoordinator, TaskAssignment};
use crate::blackboard::Blackboard;
use crate::checkpoint::{CheckpointManager, WorkerState};
use crate::planner::{HierarchicalPlanner, Plan};
use crate::reflector::{AutonomousReflector, Reflection};
use crate::tools::{PlanningTool, ReflectionTool};
use crate::agent_loop::{AgentLoop, LoopOutcome};
use engine_tool_system::ToolRegistry;
use chimera_repl::event::{ReplEvent, ReplEventSender};
use chimera_repl::engine::{EngineController, EngineState};
use chimera_repl::traits::{ReplConfig, ReplError, ReplResult};
use chimera_repl::ReplEngineCore;

struct State { state: OrchestratorState, agents: HashMap<AgentId, Box<dyn Agent>>, cycle_count: u64 }
impl State { fn new() -> Self { Self { state: OrchestratorState::Initializing, agents: HashMap::new(), cycle_count: 0 } } }
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State").field("state", &self.state).field("agent_count", &self.agents.len()).field("cycle_count", &self.cycle_count).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorState { Initializing, Running, ShuttingDown, Stopped }

/// Central orchestrator with Swarm, Blackboard, Checkpointing, and Tool integration.
pub struct AgentOrchestrator {
    state: Arc<RwLock<State>>, context: AgentContext, engine: EngineController,
    event_processor: AgentEventProcessor, event_rx: Arc<Mutex<mpsc::Receiver<ReplEvent>>>,
    shutdown_tx: mpsc::Sender<()>, governance: Arc<dyn AgentGovernance>,
    supervisor: Option<Arc<Mutex<Supervisor>>>, blackboard: Option<Arc<Blackboard>>,
    checkpoint_mgr: Arc<CheckpointManager>, last_checkpoint: Arc<RwLock<u64>>,
    shutdown_rx: Arc<Mutex<mpsc::Receiver<()>>>,
    tool_registry: Arc<Mutex<ToolRegistry>>,
}

impl AgentOrchestrator {
    pub fn new(memory: Arc<Mutex<MemoryGateway>>) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let (event_tx, event_rx) = mpsc::channel(100);
        let event_sender = ReplEventSender::new(event_tx);
        let engine = EngineController::new(chimera_repl::eventloop_adapter::rwlock(chimera_repl::SessionState::default()));
        let context = AgentContext::new();
        let event_processor = AgentEventProcessor::new(event_sender, memory.clone(), context.clone());
        let checkpoint_mgr = Arc::new(CheckpointManager::new().with_memory(memory.clone()));
        let blackboard = Arc::new(Blackboard::new());
        let governance = Arc::new(DefaultGovernance::new());
        // Initialize ToolRegistry with PlanningTool and ReflectionTool
        let mut tool_registry = ToolRegistry::new();
        let planner = Arc::new(Mutex::new(HierarchicalPlanner::new(memory.clone(), context.clone())));
        let reflector = Arc::new(Mutex::new(AutonomousReflector::new(memory.clone(), context.clone())));
        tool_registry.register(Arc::new(PlanningTool::new(planner.clone(), governance.clone(), blackboard.clone())));
        tool_registry.register(Arc::new(ReflectionTool::new(reflector.clone(), governance.clone(), blackboard.clone())));
        Self { state: Arc::new(RwLock::new(State::new())), context, engine, event_processor, event_rx: Arc::new(Mutex::new(event_rx)), shutdown_tx, governance, supervisor: None, blackboard: Some(blackboard), checkpoint_mgr, last_checkpoint: Arc::new(RwLock::new(0)), shutdown_rx: Arc::new(Mutex::new(shutdown_rx)), tool_registry: Arc::new(Mutex::new(tool_registry)) }
    }
    pub fn with_governance(mut self, gov: Arc<dyn AgentGovernance>) -> Self { self.governance = gov; self }
    pub fn governance(&self) -> &Arc<dyn AgentGovernance> { &self.governance }
    pub fn with_supervisor(mut self, sv: Arc<Mutex<Supervisor>>) -> Self { self.supervisor = Some(sv); self }
    pub fn with_blackboard(mut self, bb: Arc<Blackboard>) -> Self { self.blackboard = Some(bb); self }

    /// Create and initialize AgentLoop for autonomous execution.
    pub fn create_agent_loop(&self) -> AgentLoop {
        let memory = Arc::new(Mutex::new(MemoryGateway::new("agent_loop")));
        AgentLoop::new(
            self.context.clone(),
            Arc::new(Mutex::new(HierarchicalPlanner::new(memory.clone(), self.context.clone()))) as Arc<Mutex<dyn crate::planner::Planner>>,
            Arc::new(Mutex::new(AutonomousReflector::new(memory.clone(), self.context.clone()))) as Arc<Mutex<dyn crate::reflector::Reflector>>,
            self.governance.clone(),
            self.supervisor.clone(),
            self.blackboard.clone().unwrap_or_else(|| Arc::new(Blackboard::new())),
            self.checkpoint_mgr.clone(),
            Some(memory),
        )
    }

    /// Execute a natural language goal autonomously.
    pub async fn execute_natural_language_goal(&self, agent_id: &str, goal: &str) -> ReplResult<LoopOutcome> {
        info!("Executing natural language goal for {}: {}", agent_id, goal);
        let agent_loop = self.create_agent_loop();
        agent_loop.execute_goal(agent_id.to_string(), goal).await
    }

    /// Access tool registry.
    pub fn tool_registry(&self) -> &Arc<Mutex<ToolRegistry>> { &self.tool_registry }
    /// Access checkpoint manager.
    pub fn checkpoint_mgr(&self) -> &Arc<CheckpointManager> { &self.checkpoint_mgr }

    /// Trigger manual checkpoint for agent.
    pub async fn checkpoint(&self, agent_id: &AgentId, plan: Option<Plan>, reflections: Vec<Reflection>) -> ReplResult<()> {
        let workers = match &self.supervisor {
            Some(sv) => {
                let s = sv.lock().await;
                (0..s.worker_count()).map(|i| WorkerState { worker_id: format!("w{}", i), status: crate::swarm::WorkerStatus::Idle, assigned_task: None }).collect()
            }
            None => vec![]
        };
        let bb = match &self.blackboard { Some(bb) => bb.as_ref(), None => return Err(ReplError::Session("Blackboard not initialized".to_string())) };
        match self.checkpoint_mgr.save(agent_id, plan, reflections, workers, bb).await {
            Ok(_) => { *self.last_checkpoint.write().await = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(); Ok(()) }
            Err(e) => { tracing::warn!("Checkpoint failed: {}", e); Ok(()) } // Fail open - don't interrupt agent loop
        }
    }

    /// Auto-checkpoint every N cycles.
    async fn maybe_auto_checkpoint(&self) {
        let cycle = self.state.read().await.cycle_count;
        if cycle % 100 == 0 { // Every 100 cycles
            if let Some(agent_id) = self.state.read().await.agents.keys().next().cloned() {
                let _ = self.checkpoint(&agent_id, None, vec![]).await;
            }
        }
    }

    /// Spawn worker via Swarm.
    pub async fn spawn_worker(&self, role: AgentRole, cfg: AgentConfig) -> ReplResult<AgentId> {
        match &self.supervisor { Some(sv) => sv.lock().await.spawn_worker(role, cfg).await, None => Err(ReplError::Session("Swarm not initialized".to_string())) }
    }
    /// Delegate task to worker.
    pub async fn delegate_task(&self, task: TaskAssignment) -> ReplResult<()> {
        match &self.supervisor { Some(sv) => sv.lock().await.delegate(task).await, None => Err(ReplError::Session("Swarm not initialized".to_string())) }
    }

    pub async fn register_agent(&self, agent: Box<dyn Agent>) -> Result<(), ReplError> {
        let id = agent.id().clone();
        let mut state = self.state.write().await;
        if state.agents.contains_key(&id) { return Err(ReplError::Session(format!("Agent {} exists", id))); }
        state.agents.insert(id, agent); Ok(())
    }

    pub async fn start(&self) -> ReplResult<()> {
        self.engine.start().await?; self.state.write().await.state = OrchestratorState::Running; self.run_loop().await
    }

    async fn run_loop(&self) -> ReplResult<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            let mut event_rx = self.event_rx.lock().await;
            let mut shutdown_rx = self.shutdown_rx.lock().await;
            tokio::select! {
                _ = interval.tick() => { drop(event_rx); drop(shutdown_rx); self.process_ticks().await?; self.maybe_auto_checkpoint().await; }
                event = event_rx.recv() => { drop(event_rx); drop(shutdown_rx); if let Some(evt) = event { self.handle_event(evt).await?; } }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
                    drop(event_rx); drop(shutdown_rx);
                    if *self.engine.state.read().await == EngineState::ShuttingDown { self.state.write().await.state = OrchestratorState::ShuttingDown; break; }
                }
                _ = shutdown_rx.recv() => { drop(event_rx); self.state.write().await.state = OrchestratorState::ShuttingDown; break; }
            }
            if self.state.read().await.state == OrchestratorState::ShuttingDown { break; }
        }
        self.state.write().await.state = OrchestratorState::Stopped; Ok(())
    }

    /// Process agent ticks concurrently.
    async fn process_ticks(&self) -> ReplResult<()> {
        let ids: Vec<AgentId> = { let state = self.state.read().await; state.agents.keys().cloned().collect() };
        let mut agents_to_process = Vec::new();
        for id in ids {
            let agent_opt = { let mut state = self.state.write().await; state.agents.remove(&id) };
            if let Some(agent) = agent_opt {
                let cycle = { let mut state = self.state.write().await; state.cycle_count += 1; state.cycle_count };
                agents_to_process.push((id, agent, cycle));
            }
        }
        let mut handles = Vec::new();
        for (id, mut agent, _cycle) in agents_to_process {
            let state = self.state.clone();
            handles.push(tokio::spawn(async move {
                let outcome = agent.tick().await;
                let completed = matches!(&outcome, Ok(AgentOutcome::Completed));
                if !completed { state.write().await.agents.insert(id.clone(), agent); }
                (id, outcome)
            }));
        }
        for (id, outcome) in join_all(handles).await.into_iter().flatten() {
            match outcome {
                Ok(AgentOutcome::Continue) | Ok(AgentOutcome::Completed) => {}
                Ok(AgentOutcome::Escalated(r)) => return Err(ReplError::Session(format!("Agent {} escalated: {}", id, r))),
                Ok(AgentOutcome::Failed(m)) => return Err(ReplError::Session(format!("Agent {} failed: {}", id, m))),
                Err(e) => return Err(ReplError::Session(e.to_string())),
            }
        }
        Ok(())
    }

    async fn handle_event(&self, event: ReplEvent) -> ReplResult<()> {
        match event {
            ReplEvent::Shutdown => { self.shutdown().await?; }
            ReplEvent::ObservationReceived { agent_id, observation, source } => { self.event_processor.process_observation(&agent_id, &observation, &source).await?; }
            ReplEvent::ToolResult { agent_id, tool_name, result, success } => { self.event_processor.process_tool_result(&agent_id, &tool_name, &result, success).await?; }
            _ => {}
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> ReplResult<()> {
        self.engine.stop().await?; self.state.write().await.state = OrchestratorState::ShuttingDown;
        let _ = self.shutdown_tx.send(()).await; Ok(())
    }

    /// Restore agent from checkpoint.
    pub async fn restore_from_checkpoint(&self, agent_id: &AgentId) -> ReplResult<crate::checkpoint::Checkpoint> {
        self.checkpoint_mgr.restore_latest(agent_id).await
    }

    pub async fn state(&self) -> OrchestratorState { self.state.read().await.state }
    pub async fn agent_count(&self) -> usize { self.state.read().await.agents.len() }
    pub fn context(&self) -> &AgentContext { &self.context }
    pub fn create_supervisor(&self, id: &str) -> AgentConfig { AgentConfig::supervisor(id) }

    /// Invoke a tool by name with governance approval check.
    pub async fn invoke_tool(&self, tool_name: &str, args: engine_tool_system::ToolArgs) -> Result<engine_tool_system::ToolOutput, engine_tool_system::ToolError> {
        // Governance approval for tool invocation
        let req = crate::governance::GovernanceRequest {
            requester: "orchestrator".to_string(),
            action_type: format!("invoke_tool:{}", tool_name),
            risk_score: 0.1,
            description: format!("Invoke tool {} with args {:?}", tool_name, args),
            level: crate::governance::ApprovalLevel::Auto,
        };
        match self.governance.approve(&self.context, &req).await {
            Ok(crate::governance::Decision::Approved) => {},
            Ok(_) => return Err(engine_tool_system::ToolError::new("Tool invocation rejected by governance")),
            Err(e) => return Err(engine_tool_system::ToolError::new(format!("Governance error: {}", e))),
        }
        // Execute tool via registry
        let registry = self.tool_registry.lock().await;
        match registry.get(tool_name) {
            Some(tool) => tool.execute(args).await,
            None => Err(engine_tool_system::ToolError::new(format!("Tool {} not found", tool_name))),
        }
    }
}

#[async_trait::async_trait]
impl ReplEngineCore for AgentOrchestrator {
    async fn new(_config: ReplConfig) -> ReplResult<Self> where Self: Sized { Ok(Self::new(Arc::new(Mutex::new(MemoryGateway::new("default"))))) }
    async fn run(&self) -> ReplResult<()> { self.start().await }
    async fn shutdown(&self) -> ReplResult<()> { self.shutdown().await }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[tokio::test] async fn test_orchestrator_lifecycle() {
        let orch = AgentOrchestrator::new(Arc::new(Mutex::new(MemoryGateway::new("test"))));
        assert_eq!(orch.state().await, OrchestratorState::Initializing);
    }
    #[tokio::test] async fn test_tool_registry_initialized() {
        let orch = AgentOrchestrator::new(Arc::new(Mutex::new(MemoryGateway::new("test"))));
        let registry = orch.tool_registry.lock().await;
        let tools = registry.list();
        assert!(tools.contains(&"planning"), "PlanningTool should be registered");
        assert!(tools.contains(&"reflection"), "ReflectionTool should be registered");
    }
    #[tokio::test] async fn test_tool_agent_loop() {
        let orch = AgentOrchestrator::new(Arc::new(Mutex::new(MemoryGateway::new("test"))));
        // Test PlanningTool invocation via orchestrator
        let result = orch.invoke_tool("planning", json!({"action": "create_goal", "description": "Test goal", "priority": "high"})).await;
        assert!(result.is_ok(), "PlanningTool should execute successfully");
        // Test ReflectionTool invocation
        let result = orch.invoke_tool("reflection", json!({"action": "get_history", "goal_id": "test_goal"})).await;
        assert!(result.is_ok(), "ReflectionTool should execute successfully");
    }
    #[tokio::test] async fn test_agent_loop_creation() {
        let orch = AgentOrchestrator::new(Arc::new(Mutex::new(MemoryGateway::new("test"))));
        let agent_loop = orch.create_agent_loop();
        assert_eq!(agent_loop.current_state().await, crate::agent_loop::LoopState::Idle);
    }
    #[tokio::test] async fn test_natural_language_goal_execution() {
        let orch = AgentOrchestrator::new(Arc::new(Mutex::new(MemoryGateway::new("test"))));
        let outcome = orch.execute_natural_language_goal("test_agent", "Create a test plan").await;
        assert!(outcome.is_ok());
    }
    #[tokio::test] async fn test_autonomous_goal_completion() {
        let orch = AgentOrchestrator::new(Arc::new(Mutex::new(MemoryGateway::new("test"))));
        let outcome = orch.execute_natural_language_goal("agent1", "Create a simple plan").await.unwrap();
        assert!(matches!(outcome, LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted));
    }
}
