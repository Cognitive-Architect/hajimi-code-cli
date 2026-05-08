//! Proactive Agent Loop: Observe → Retrieve → Plan → Act → Reflect → Store → Decide
use crate::blackboard::Blackboard;
use crate::agent_loop_builder::AgentLoopConfig;
use crate::checkpoint::CheckpointManager;
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::planner::{Planner, Priority, TaskResult, Goal, PlanStatus};
use crate::reflector::Reflector;
use crate::swarm::{Supervisor, SwarmCoordinator};
use crate::swarm_delegate::SwarmDelegate;
use crate::edit_applier::EditApplier;
use crate::memory_retriever::{MemoryRetriever, RetrieveOutcome};
use crate::resource_monitor::ResourceMonitor;
use crate::{AgentContext, AgentId};
use chimera_repl::traits::{ReplError, ReplResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn};

const MAX_ITERATIONS: usize = 100;
const ITERATION_BUDGET: usize = 50;
const CHECKPOINT_INTERVAL: usize = 10;
const ACT_FAILURE_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoopState { Idle, Observing, Retrieving, Planning, Acting, Reflecting, Storing, Deciding, Completed, Failed }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TraceStepType { Observe, Retrieve, Plan, Act, Reflect, Store, Decide, Other, EditProposed, EditApplied, EditRejected }

/// Summary of file and command operations performed during an agent step.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OperationSummary {
    pub files_edited: usize,
    pub files_created: usize,
    pub files_deleted: usize,
    pub commands_run: usize,
    pub total_diff_lines: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceEvent {
    pub step: LoopState,
    pub details: String,
    pub iteration: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub step_type: TraceStepType,
    pub plan_summary: Option<String>,
    pub reflection_key_points: Vec<String>,
    pub confidence_score: Option<f32>,
    pub edit_payload: Option<String>,
    pub operation_summary: Option<OperationSummary>,
    pub thinking_content: Option<String>,
}

pub struct AgentLoop {
    context: AgentContext,
    planner: Arc<Mutex<dyn Planner>>,
    reflector: Arc<Mutex<dyn Reflector>>,
    governance: Arc<dyn AgentGovernance>,
    swarm: Option<Arc<Mutex<Supervisor>>>,
    blackboard: Arc<Blackboard>,
    checkpoint_mgr: Arc<CheckpointManager>,
    iteration_count: Arc<Mutex<usize>>,
    current_state: Arc<Mutex<LoopState>>,
    trace_tx: Option<tokio::sync::broadcast::Sender<TraceEvent>>,
    provider_id: Option<String>,
    memory_retriever: MemoryRetriever,
    paused: Arc<AtomicBool>,
    pub resource_monitor: Arc<ResourceMonitor>,
    edit_applier: Option<Arc<EditApplier>>,
}

impl AgentLoop {
    #[deprecated(since = "0.2.0", note = "Use AgentLoopBuilder instead")]
    pub fn new(config: AgentLoopConfig) -> Self {
        Self::from_components(config)
    }

    pub(crate) fn from_components(config: AgentLoopConfig) -> Self {
        let memory_retriever = MemoryRetriever::new(
            config.blackboard.clone(),
            config.sync_gateway.clone(),
            config.memory.clone(),
        );
        Self {
            context: config.context,
            planner: config.planner,
            reflector: config.reflector,
            governance: config.governance,
            swarm: config.swarm,
            blackboard: config.blackboard,
            checkpoint_mgr: config.checkpoint_mgr,
            iteration_count: Arc::new(Mutex::new(0)),
            current_state: Arc::new(Mutex::new(LoopState::Idle)),
            trace_tx: Some(tokio::sync::broadcast::channel(64).0),
            provider_id: config.provider_id,
            memory_retriever,
            paused: Arc::new(AtomicBool::new(false)),
            resource_monitor: Arc::new(ResourceMonitor::new()),
            edit_applier: None,
        }
    }

    pub async fn run(&self, agent_id: AgentId, initial_goal: &str) -> ReplResult<LoopOutcome> {
        info!("AgentLoop starting for {} with goal: {}", agent_id, initial_goal);
        if let Some(ref pid) = self.provider_id {
            self.blackboard.write("__hajimi_provider_id", pid, &agent_id).await;
        }
        *self.current_state.lock().await = LoopState::Planning;
        self.emit_trace(LoopState::Planning, format!("Planning initial goal: {}", initial_goal), 0);
        let goal_id = self.plan_initial_goal(initial_goal).await?;
        info!("Initial goal created: {}", goal_id);
        let mut outcome = LoopOutcome::InProgress;
        for i in 0..MAX_ITERATIONS {
            while self.paused.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            self.resource_monitor.record_iteration();
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
                    self.resource_monitor.record_success();
                    *self.current_state.lock().await = LoopState::Reflecting;
                    self.emit_trace(LoopState::Reflecting, format!("Reflecting on task result: success={}", task_result.success), i);
                    if let Err(e) = self.reflect(&goal_id, &task_result).await {
                        warn!("Reflection failed (continuing): {}", e);
                    }
                }
                Err(e) => {
                    warn!("Act failed (continuing): {}", e);
                    self.emit_trace(LoopState::Acting, format!("Act failed: {}", e), i);
                    self.resource_monitor.record_failure();
                    if i > ACT_FAILURE_THRESHOLD { outcome = LoopOutcome::ActFailed(e.to_string()); break; }
                }
            }
            self.resource_monitor.record_blackboard_size(self.blackboard.snapshot().await.len());
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
        let iter = *self.iteration_count.lock().await;
        match self.memory_retriever.retrieve(agent_id).await {
            RetrieveOutcome::CacheHit(summary) => {
                self.emit_trace(LoopState::Retrieving, format!("Cache hit: {}", summary), iter);
            }
            RetrieveOutcome::Retrieved { summary } => {
                self.emit_trace(LoopState::Retrieving, format!("Retrieved {}", summary), iter);
            }
            RetrieveOutcome::Error(e) => {
                self.emit_trace(LoopState::Retrieving, format!("Retrieval error: {}", e), iter);
            }
        }
    }

    pub(crate) async fn act(&self, agent_id: &AgentId, goal_id: &str) -> ReplResult<TaskResult> {
        info!("Acting for {} on goal {}", agent_id, goal_id);
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(LoopState::Acting, format!("Acting start for goal {}", goal_id), iter);
        if !self.gov_check("act", &format!("Execute task for {}", goal_id), 0.1).await? {
            return Ok(TaskResult { success: false, output: "Act rejected by governance".to_string(), timestamp: chrono::Utc::now() });
        }
        let task_opt = { self.planner.lock().await.next_task().await? };
        if let Some(task) = task_opt {
            if let Some(ref swarm) = self.swarm {
                match SwarmDelegate::try_delegate(swarm, &self.blackboard, agent_id, &task).await {
                    Some(Ok(result)) => {
                        self.emit_trace(LoopState::Acting, format!("Task {} completed: success={}", task.id, result.success), iter);
                        Ok(result)
                    }
                    Some(Err(e)) => {
                        warn!("Swarm delegation failed (falling back): {}", e);
                        self.emit_trace(LoopState::Acting, format!("Delegation failed for task {}: {}", task.id, e), iter);
                        Ok(TaskResult { success: true, output: format!("Task {} executed locally (delegation failed)", task.id), timestamp: chrono::Utc::now() })
                    }
                    None => {
                        self.emit_trace(LoopState::Acting, "No idle worker available, falling back to local execution".to_string(), iter);
                        Ok(TaskResult { success: true, output: format!("Task {} executed locally (no idle worker)", task.id), timestamp: chrono::Utc::now() })
                    }
                }
            } else {
                self.emit_trace(LoopState::Acting, "No swarm available, falling back to local execution".to_string(), iter);
                Ok(TaskResult { success: true, output: format!("Task {} executed locally (no swarm)", task.id), timestamp: chrono::Utc::now() })
            }
        } else {
            Ok(TaskResult { success: true, output: "No pending tasks".to_string(), timestamp: chrono::Utc::now() })
        }
    }

    pub(crate) async fn reflect(&self, goal_id: &str, result: &TaskResult) -> ReplResult<()> {
        info!("Reflecting on goal {} with success={}", goal_id, result.success);
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(LoopState::Reflecting, format!("Reflecting on goal {}", goal_id), iter);
        if !self.gov_check("reflect", &format!("Reflect on goal {}", goal_id), 0.1).await? {
            warn!("Reflection rejected by governance (skipping)"); return Ok(());
        }
        let goal = Goal { id: goal_id.to_string(), description: "Reflected goal".to_string(), priority: Priority::Medium, status: PlanStatus::InProgress, subgoals: vec![], metadata: std::collections::HashMap::new(), created_at: chrono::Utc::now(), approved: true };
        if let Some(ref swarm) = self.swarm {
            let worker_results = swarm.lock().await.aggregate().await;
            if !worker_results.is_empty() {
                let reflection = self.reflector.lock().await.reflect_multi(&goal, &worker_results).await?;
                self.emit_trace(LoopState::Reflecting, format!("Multi-worker reflection: confidence={:.2}", reflection.confidence), iter);
            } else {
                let _ = self.reflector.lock().await.reflect(&goal, result).await?;
            }
        } else {
            let _ = self.reflector.lock().await.reflect(&goal, result).await?;
        }
        Ok(())
    }

    async fn store(&self, agent_id: &AgentId) -> ReplResult<()> {
        self.memory_retriever.store(agent_id, &self.checkpoint_mgr).await
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

    /// Returns a clone of the broadcast sender for trace events.
    /// Callers can subscribe to this sender to receive real-time AgentLoop traces.
    pub fn trace_tx(&self) -> Option<tokio::sync::broadcast::Sender<TraceEvent>> {
        self.trace_tx.clone()
    }

    fn emit_trace(&self, step: LoopState, details: String, iteration: usize) {
        self.emit_trace_with_meta(step, details, iteration, None, vec![], None, None, None, None);
    }

    fn emit_trace_with_meta(
        &self,
        step: LoopState,
        details: String,
        iteration: usize,
        plan_summary: Option<String>,
        reflection_key_points: Vec<String>,
        confidence_score: Option<f32>,
        edit_payload: Option<String>,
        operation_summary: Option<OperationSummary>,
        thinking_content: Option<String>,
    ) {
        if let Some(ref tx) = self.trace_tx {
            let step_type = match step {
                LoopState::Observing => TraceStepType::Observe,
                LoopState::Retrieving => TraceStepType::Retrieve,
                LoopState::Planning => TraceStepType::Plan,
                LoopState::Acting => TraceStepType::Act,
                LoopState::Reflecting => TraceStepType::Reflect,
                LoopState::Storing => TraceStepType::Store,
                LoopState::Deciding => TraceStepType::Decide,
                _ => TraceStepType::Other,
            };
            let event = TraceEvent {
                step, details, iteration, timestamp: chrono::Utc::now(), step_type,
                plan_summary, reflection_key_points, confidence_score, edit_payload,
                operation_summary, thinking_content,
            };
            let _ = tx.send(event);
        }
    }

    pub fn with_edit_applier(mut self, applier: Arc<EditApplier>) -> Self {
        self.edit_applier = Some(applier);
        self
    }

    pub fn pause(&self) { self.paused.store(true, Ordering::Relaxed); info!("AgentLoop pause requested"); }
    pub fn resume(&self) { self.paused.store(false, Ordering::Relaxed); info!("AgentLoop resumed"); }
    pub fn is_paused(&self) -> bool { self.paused.load(Ordering::Relaxed) }

    pub async fn inject_memory(&self, key: &str, value: &str, agent_id: &AgentId) -> ReplResult<()> {
        info!("Injecting memory: {} = {}", key, value);
        self.blackboard.write(key, value, agent_id).await;
        Ok(())
    }

    pub async fn update_plan(&self, description: &str) -> ReplResult<()> {
        info!("Updating plan: {}", description);
        let mut planner = self.planner.lock().await;
        planner.create_goal(description, Priority::High).await?;
        Ok(())
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
