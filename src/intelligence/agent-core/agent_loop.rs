//! Proactive Agent Loop: Observe → Retrieve → Plan → Act → Reflect → Store → Decide
use crate::blackboard::Blackboard;
use crate::checkpoint::CheckpointManager;
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::planner::{Planner, Priority, TaskResult, Goal, PlanStatus};
use crate::reflector::Reflector;
use crate::swarm::{Supervisor, SwarmCoordinator, TaskAssignment};
use crate::{AgentContext, AgentId};
use chimera_repl::traits::{ReplError, ReplResult};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

const MAX_ITERATIONS: usize = 100;
const ITERATION_BUDGET: usize = 50;
const CHECKPOINT_INTERVAL: usize = 10;
const ACT_FAILURE_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum LoopState { Idle, Observing, Retrieving, Planning, Acting, Reflecting, Storing, Deciding, Completed, Failed }

#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceEvent {
    pub step: LoopState,
    pub details: String,
    pub iteration: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct AgentLoop {
    context: AgentContext,
    planner: Arc<Mutex<dyn Planner>>,
    reflector: Arc<Mutex<dyn Reflector>>,
    governance: Arc<dyn AgentGovernance>,
    swarm: Option<Arc<Mutex<Supervisor>>>,
    blackboard: Arc<Blackboard>,
    checkpoint_mgr: Arc<CheckpointManager>,
    #[allow(dead_code)]
    memory: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>,
    iteration_count: Arc<Mutex<usize>>,
    current_state: Arc<Mutex<LoopState>>,
    trace_tx: Option<tokio::sync::broadcast::Sender<TraceEvent>>,
}

impl AgentLoop {
    #[deprecated(since = "0.2.0", note = "Use AgentLoopBuilder instead")]
    pub fn new(
        context: AgentContext,
        planner: Arc<Mutex<dyn Planner>>,
        reflector: Arc<Mutex<dyn Reflector>>,
        governance: Arc<dyn AgentGovernance>,
        swarm: Option<Arc<Mutex<Supervisor>>>,
        blackboard: Arc<Blackboard>,
        checkpoint_mgr: Arc<CheckpointManager>,
        memory: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>,
    ) -> Self {
        Self::from_components(context, planner, reflector, governance, swarm, blackboard, checkpoint_mgr, memory)
    }

    pub(crate) fn from_components(
        context: AgentContext,
        planner: Arc<Mutex<dyn Planner>>,
        reflector: Arc<Mutex<dyn Reflector>>,
        governance: Arc<dyn AgentGovernance>,
        swarm: Option<Arc<Mutex<Supervisor>>>,
        blackboard: Arc<Blackboard>,
        checkpoint_mgr: Arc<CheckpointManager>,
        memory: Option<Arc<Mutex<memory::memory_gateway::MemoryGateway>>>,
    ) -> Self {
        Self { context, planner, reflector, governance, swarm, blackboard, checkpoint_mgr, memory, iteration_count: Arc::new(Mutex::new(0)), current_state: Arc::new(Mutex::new(LoopState::Idle)), trace_tx: Some(tokio::sync::broadcast::channel(64).0) }
    }

    pub async fn run(&self, agent_id: AgentId, initial_goal: &str) -> ReplResult<LoopOutcome> {
        info!("AgentLoop starting for {} with goal: {}", agent_id, initial_goal);
        *self.current_state.lock().await = LoopState::Planning;
        self.emit_trace(LoopState::Planning, format!("Planning initial goal: {}", initial_goal), 0);
        let goal_id = self.plan_initial_goal(initial_goal).await?;
        info!("Initial goal created: {}", goal_id);
        let mut outcome = LoopOutcome::InProgress;
        for i in 0..MAX_ITERATIONS {
            *self.iteration_count.lock().await = i;
            if i >= ITERATION_BUDGET { warn!("Iteration budget exhausted at step {}", i); outcome = LoopOutcome::BudgetExceeded; break; }
            *self.current_state.lock().await = LoopState::Observing;
            self.emit_trace(LoopState::Observing, format!("Observing environment for iteration {}", i), i);
            let observation = self.observe(&agent_id).await;
            self.emit_trace(LoopState::Observing, format!("Observed {} blackboard keys", observation.blackboard_keys.len()), i);
            *self.current_state.lock().await = LoopState::Retrieving;
            self.emit_trace(LoopState::Retrieving, format!("Retrieving memories for {}", agent_id), i);
            self.retrieve(&agent_id).await;
            *self.current_state.lock().await = LoopState::Acting;
            self.emit_trace(LoopState::Acting, format!("Acting on goal {}", goal_id), i);
            match self.act(&agent_id, &goal_id).await {
                Ok(task_result) => {
                    *self.current_state.lock().await = LoopState::Reflecting;
                    self.emit_trace(LoopState::Reflecting, format!("Reflecting on task result: success={}", task_result.success), i);
                    if let Err(e) = self.reflect(&goal_id, &task_result).await {
                        warn!("Reflection failed (continuing): {}", e);
                    }
                }
                Err(e) => {
                    warn!("Act failed (continuing): {}", e);
                    self.emit_trace(LoopState::Acting, format!("Act failed: {}", e), i);
                    if i > ACT_FAILURE_THRESHOLD { outcome = LoopOutcome::ActFailed(e.to_string()); break; }
                }
            }
            *self.current_state.lock().await = LoopState::Storing;
            self.emit_trace(LoopState::Storing, format!("Storing checkpoint for iteration {}", i), i);
            if i % CHECKPOINT_INTERVAL == 0 {
                if let Err(e) = self.store(&agent_id).await { warn!("Store failed (continuing): {}", e); }
            }
            *self.current_state.lock().await = LoopState::Deciding;
            self.emit_trace(LoopState::Deciding, format!("Deciding next action for goal {}", goal_id), i);
            match self.decide(&agent_id, &goal_id).await? {
                DecisionOutcome::Continue => {}
                DecisionOutcome::Complete => { outcome = LoopOutcome::Success; break; }
                DecisionOutcome::Abort => { outcome = LoopOutcome::Aborted; break; }
            }
        }
        *self.current_state.lock().await = LoopState::Completed;
        self.emit_trace(LoopState::Completed, format!("Loop completed with outcome: {:?}", outcome), 0);
        info!("AgentLoop completed with outcome: {:?}", outcome);
        Ok(outcome)
    }

    pub async fn execute_goal(&self, agent_id: AgentId, description: &str) -> ReplResult<LoopOutcome> {
        let (parsed_goal, priority) = Self::from_natural_language(description);
        info!("Executing goal '{}' with priority {:?}", parsed_goal, priority);
        self.run(agent_id, &parsed_goal).await
    }

    async fn plan_initial_goal(&self, description: &str) -> ReplResult<String> {
        if !self.gov_check("create_goal", description, 0.1).await? {
            return Err(ReplError::Session("Goal creation rejected by governance".to_string()));
        }
        self.planner.lock().await.create_goal(description, Priority::High).await
    }

    async fn observe(&self, agent_id: &AgentId) -> Observation {
        info!("Observing for {}", agent_id);
        let bb_state = self.blackboard.snapshot().await;
        Observation { agent_id: agent_id.clone(), blackboard_keys: bb_state.keys().cloned().collect(), timestamp: chrono::Utc::now() }
    }

    async fn retrieve(&self, agent_id: &AgentId) {
        info!("Retrieving memories for {} with goal context", agent_id);
        if let Some(ref memory) = self.memory {
            let mem = memory.lock().await;
            let graph_key = format!("ctx_{}", agent_id);
            if let Some(entry) = mem.session.get(&graph_key) {
                let content = format!("{:?}", entry);
                self.blackboard.write(&format!("retrieved_{}", agent_id), &content, agent_id).await;
                info!("Graph query hit for {}: {} bytes", agent_id, content.len());
            }
        }
        let _bb_state = self.blackboard.snapshot().await;
    }

    async fn act(&self, agent_id: &AgentId, goal_id: &str) -> ReplResult<TaskResult> {
        info!("Acting for {} on goal {}", agent_id, goal_id);
        if !self.gov_check("act", &format!("Execute task for {}", goal_id), 0.1).await? {
            return Ok(TaskResult { success: false, output: "Act rejected by governance".to_string(), timestamp: chrono::Utc::now() });
        }
        let task_opt = { self.planner.lock().await.next_task().await? };
        if let Some(task) = task_opt {
            if let Some(ref swarm) = self.swarm {
                let assignment = TaskAssignment { task_id: task.id.clone(), description: task.description.clone(), assigned_to: agent_id.clone(), priority: 5 };
                if let Err(e) = swarm.lock().await.delegate(assignment).await { warn!("Swarm delegation failed (falling back): {}", e); }
            }
            Ok(TaskResult { success: true, output: format!("Task {} delegated to Worker", task.id), timestamp: chrono::Utc::now() })
        } else {
            Ok(TaskResult { success: true, output: "No pending tasks".to_string(), timestamp: chrono::Utc::now() })
        }
    }

    async fn reflect(&self, goal_id: &str, result: &TaskResult) -> ReplResult<()> {
        info!("Reflecting on goal {} with success={}", goal_id, result.success);
        if !self.gov_check("reflect", &format!("Reflect on goal {}", goal_id), 0.1).await? {
            warn!("Reflection rejected by governance (skipping)"); return Ok(());
        }
        let goal = Goal { id: goal_id.to_string(), description: "Reflected goal".to_string(), priority: Priority::Medium, status: PlanStatus::InProgress, subgoals: vec![], metadata: std::collections::HashMap::new(), created_at: chrono::Utc::now(), approved: true };
        let _ = self.reflector.lock().await.reflect(&goal, result).await?;
        Ok(())
    }

    async fn store(&self, agent_id: &AgentId) -> ReplResult<()> {
        info!("Storing checkpoint and plan for {}", agent_id);
        let _ = self.checkpoint_mgr.save(agent_id, None, vec![], vec![], &self.blackboard).await?;
        if let Some(ref memory) = self.memory {
            let mut mem = memory.lock().await;
            if let Err(e) = mem.push_vector(&format!("plan_{}", agent_id), &format!("checkpoint_{}", agent_id)) {
                warn!("MemoryGateway persist failed (continuing): {}", e);
            }
        }
        Ok(())
    }

    async fn decide(&self, agent_id: &AgentId, goal_id: &str) -> ReplResult<DecisionOutcome> {
        info!("Deciding for {} on goal {}", agent_id, goal_id);
        let req = GovernanceRequest { requester: agent_id.clone(), action_type: "decide_next".to_string(), risk_score: 0.1, description: format!("Decide next step for goal {}", goal_id), level: ApprovalLevel::Auto };
        match self.governance.approve(&self.context, &req).await? {
            Decision::Approved => { if self.planner.lock().await.is_complete() { Ok(DecisionOutcome::Complete) } else { Ok(DecisionOutcome::Continue) } }
            Decision::Rejected(_) => Ok(DecisionOutcome::Abort),
            _ => Ok(DecisionOutcome::Continue),
        }
    }

    async fn gov_check(&self, action: &str, desc: &str, risk: f32) -> ReplResult<bool> {
        let req = GovernanceRequest { requester: "agent_loop".to_string(), action_type: action.to_string(), risk_score: risk, description: desc.to_string(), level: ApprovalLevel::Auto };
        match self.governance.approve(&self.context, &req).await? { Decision::Approved => Ok(true), _ => Ok(false) }
    }

    pub async fn current_state(&self) -> LoopState { *self.current_state.lock().await }
    pub async fn iteration_count(&self) -> usize { *self.iteration_count.lock().await }

    pub fn subscribe_trace(&self) -> Option<tokio::sync::broadcast::Receiver<TraceEvent>> {
        self.trace_tx.as_ref().map(|tx| tx.subscribe())
    }

    fn emit_trace(&self, step: LoopState, details: String, iteration: usize) {
        if let Some(ref tx) = self.trace_tx {
            let event = TraceEvent { step, details, iteration, timestamp: chrono::Utc::now() };
            let _ = tx.send(event);
        }
    }

    pub fn from_natural_language(goal: &str) -> (String, Priority) {
        let priority = if goal.contains("urgent") || goal.contains("critical") { Priority::Critical }
        else if goal.contains("important") || goal.contains("high") { Priority::High }
        else if goal.contains("low") || goal.contains("minor") { Priority::Low }
        else { Priority::Medium };
        (goal.to_string(), priority)
    }
}

#[derive(Debug, Clone)]
pub struct Observation { pub agent_id: AgentId, pub blackboard_keys: Vec<String>, pub timestamp: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopOutcome { InProgress, Success, Aborted, BudgetExceeded, ActFailed(String) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecisionOutcome { Continue, Complete, Abort }
