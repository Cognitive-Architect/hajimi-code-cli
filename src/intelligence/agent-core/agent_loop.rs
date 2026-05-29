//! Proactive Agent Loop: Observe → Retrieve → Plan → Act → Reflect → Store → Decide
use crate::act_dto::ToolCallV1;
use crate::act_executor::{ActExecutor, BB_NEXT_TOOL};
use crate::agent_loop_builder::AgentLoopConfig;
use crate::blackboard::Blackboard;
use crate::checkpoint::CheckpointManager;
use crate::edit_applier::EditApplier;
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::memory_retriever::{MemoryRetriever, RetrieveOutcome};
use crate::planner::{Goal, PlanStatus, Planner, Priority, TaskResult};
use crate::reflector::Reflector;
use crate::reflector_dto::RecommendedAction;
use crate::resource_monitor::ResourceMonitor;
use crate::swarm::{Supervisor, SwarmCoordinator};
use crate::swarm_delegate::SwarmDelegate;
use crate::{AgentContext, AgentId};
use chimera_repl::traits::{ReplError, ReplResult};
use engine_tool_system::ToolRegistry;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn};

// Blackboard keys for plan adjustment routing and stop-loss tracking (B-08/14, B-09/14).

/// BB_REFLECTOR_CRITIQUE: Stores the serialized ReflectorCritiqueV1Dto JSON
/// written by the Reflector bridge after LLM critique parsing.
#[allow(dead_code)]
const BB_REFLECTOR_CRITIQUE: &str = "__hajimi_reflector_critique";
/// BB_PLAN_ADJUSTMENT: Stores the plan adjustment proposal (action + reason + suggested_tools)
/// extracted from the Reflector's critique for downstream routing.
const BB_PLAN_ADJUSTMENT: &str = "__hajimi_plan_adjustment";
/// BB_STOP_LOSS: Stores the serialized StopLossDto JSON when a stop-loss condition
/// is detected, triggering escalation to the user or handoff.
#[allow(dead_code)]
const BB_STOP_LOSS: &str = "__hajimi_stop_loss";
const BB_FAILURE_COUNT: &str = "__hajimi_failure_count";
const BB_LAST_EVIDENCE: &str = "__hajimi_last_evidence";
const BB_RETRY_FLAG: &str = "__hajimi_retry_with_new_args";
const BB_ALT_TOOL_FLAG: &str = "__hajimi_use_alternative_tool";
const BB_ASK_USER_REASON: &str = "__hajimi_ask_user_reason";
const BB_HANDOFF_SUMMARY: &str = "__hajimi_handoff_summary";
const BB_ALT_TOOL_NAME: &str = "__hajimi_alt_tool_name";

const MAX_ITERATIONS: usize = 100;
const ITERATION_BUDGET: usize = 50;
const CHECKPOINT_INTERVAL: usize = 10;
const ACT_FAILURE_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoopState {
    Idle,
    Observing,
    Retrieving,
    Planning,
    Acting,
    Reflecting,
    Storing,
    Deciding,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TraceStepType {
    Observe,
    Retrieve,
    Plan,
    Act,
    Reflect,
    Store,
    Decide,
    Other,
    EditProposed,
    EditApplied,
    EditRejected,
}

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
    pub skill_registry: Option<Arc<crate::skills::SkillRegistry>>,
    pub skill_router: Option<Arc<crate::skills::SkillRouter>>,
    /// The tool registry holding all active tools.
    /// SAFETY: Wrapped in Arc<Mutex<>> for thread safety and Option to support backward compatibility.
    pub tool_registry: Option<Arc<Mutex<ToolRegistry>>>,
    /// Native driver for LLM-native double-track support.
    pub native_driver: Option<Arc<dyn crate::llm_native::AgentTurnDriver>>,
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
        let tool_registry = config.tool_registry.clone();
        let tool_count = if let Some(ref reg) = tool_registry {
            if let Ok(guard) = reg.try_lock() {
                guard.list().len()
            } else {
                0
            }
        } else {
            0
        };
        info!("AgentLoop initialized with {} tools", tool_count);
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
            skill_registry: config.skill_registry,
            skill_router: config.skill_router,
            tool_registry,
            native_driver: config.native_driver,
        }
    }

    pub async fn run(&self, agent_id: AgentId, initial_goal: &str) -> ReplResult<LoopOutcome> {
        info!(
            "AgentLoop starting for {} with goal: {}",
            agent_id, initial_goal
        );

        // NEG-002: 传递空 goal 时，优雅报错返回 EmptyGoal 错误实体
        if initial_goal.trim().is_empty() {
            info!("UX-002: goal is empty, returning EmptyGoal error");
            return Err(chimera_repl::traits::ReplError::Session(
                "EmptyGoal".to_string(),
            ));
        }

        // FUNC-001: is_agent_llm_native_enabled() 为 true 时能成功跳转 native_turn 路径
        if crate::prompts::is_agent_llm_native_enabled() {
            // FUNC-003: 新路径启动时系统正常打印 'LLM-Native path enabled'
            info!("✨✨ [Hajimi IDE] LLM-Native Agent Loop Core Activated. Routing execution to autonomous LLM-Native path! ✨✨");
            if let Some(ref driver) = self.native_driver {
                info!("LLM-Native path enabled for goal: {}", initial_goal);

                // 将 Goal 原始自然语言文字封装成 RawUserIntent (HIGH-001 安全红线: 禁止降低/lowering 处理)
                let intent = crate::llm_native::RawUserIntent::from_text(
                    initial_goal.to_string(),
                    agent_id.to_string(),
                );

                // 转换 tools (Ouroboros 闭环从 tool_registry 导出)
                let tools_spec = if let Some(ref reg) = self.tool_registry {
                    let guard = reg.lock().await;
                    crate::llm_native::ToolSpecExporter::from_registry(&guard)
                } else {
                    vec![]
                };

                // 独立的 context (FUNC-004)
                let cancellation = crate::llm_native::CancellationToken::new();

                info!("LlmNativeDriver starts executing run_turn on LLM-Native path");
                match driver
                    .run_turn(
                        intent,
                        tools_spec,
                        vec![], // 初始历史为空
                        self.governance.clone(),
                        cancellation,
                    )
                    .await
                {
                    Ok(outcome) => {
                        info!(
                            "LlmNativeDriver execution outcomes: success = {}, iterations = {}",
                            outcome.success, outcome.iterations
                        );
                        if outcome.success {
                            return Ok(LoopOutcome::Success);
                        } else {
                            return Ok(LoopOutcome::ActFailed(
                                outcome
                                    .final_message
                                    .unwrap_or_else(|| "LLM-Native execution failed".to_string()),
                            ));
                        }
                    }
                    Err(e) => {
                        // NEG-001: 自动安全 fallback 到 legacy 分支而不闪退
                        warn!(
                            "LlmNativeDriver run_turn failed: {:?}. Falling back to legacy path.",
                            e
                        );
                    }
                }
            } else {
                // NEG-001: 中途遇到 Driver 初始化为空的情况，自动安全 fallback 到 legacy 分支而不闪退
                warn!("is_agent_llm_native_enabled is true but native_driver is None! Safe fallback to legacy branch.");
            }
        }

        if let Some(ref pid) = self.provider_id {
            self.blackboard
                .write("__hajimi_provider_id", pid, &agent_id)
                .await;
        }
        let thinking_content = self
            .blackboard
            .read("__hajimi_thinking")
            .await
            .map(|e| e.value);
        *self.current_state.lock().await = LoopState::Planning;
        self.emit_trace_with_meta(
            LoopState::Planning,
            format!("Planning initial goal: {}", initial_goal),
            0,
            None,
            vec![],
            None,
            None,
            None,
            thinking_content,
        );
        if crate::prompts::is_agent_skills_v0_enabled() {
            if self.skill_router.is_none() || self.skill_registry.is_none() {
                warn!("Agent skills V0 is enabled but skill_router or skill_registry is None; continuing without skills");
            } else if let Err(e) = self.route_and_load_skills(&agent_id, initial_goal).await {
                warn!(
                    "route_and_load_skills failed: {}. continuing without skills",
                    e
                );
            }
        }
        let goal_id = self.plan_initial_goal(initial_goal).await?;
        info!("Initial goal created: {}", goal_id);

        // Planner decompose 激活 (Day 6)
        {
            let mut planner = self.planner.lock().await;
            info!("Hajimi RealAgent Plan: decomposing goal {}", goal_id);
            match planner.decompose(&goal_id).await {
                Ok(sg_ids) => {
                    if !sg_ids.is_empty() {
                        let first_sg_id = &sg_ids[0];
                        info!(
                            "Decomposed goal into {} subgoals. Expanding first subgoal: {}",
                            sg_ids.len(),
                            first_sg_id
                        );
                        if let Err(e) = planner.expand(first_sg_id).await {
                            warn!("Planner expand failed for subgoal {}: {}", first_sg_id, e);
                        }
                    } else {
                        warn!("Decompose returned empty subgoals for goal {}", goal_id);
                    }
                }
                Err(e) => {
                    warn!(
                        "Planner decompose failed during run(): {}. Rule-based fallback active.",
                        e
                    );
                }
            }
        }

        let mut outcome = LoopOutcome::InProgress;
        for i in 0..MAX_ITERATIONS {
            while self.paused.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            self.resource_monitor.record_iteration();
            *self.iteration_count.lock().await = i;
            if i >= ITERATION_BUDGET {
                warn!("Iteration budget exhausted at step {}", i);
                outcome = LoopOutcome::BudgetExceeded;
                break;
            }
            *self.current_state.lock().await = LoopState::Observing;
            self.emit_trace(
                LoopState::Observing,
                format!("Observing environment for iteration {}", i),
                i,
            );
            let observation = self.observe(&agent_id).await;
            self.emit_trace(
                LoopState::Observing,
                format!(
                    "Observed {} blackboard keys",
                    observation.blackboard_keys.len()
                ),
                i,
            );
            *self.current_state.lock().await = LoopState::Retrieving;
            self.emit_trace(
                LoopState::Retrieving,
                format!("Retrieving memories for {}", agent_id),
                i,
            );
            self.retrieve(&agent_id).await;
            *self.current_state.lock().await = LoopState::Acting;
            self.emit_trace(LoopState::Acting, format!("Acting on goal {}", goal_id), i);
            let task_result = match self.act(&agent_id, &goal_id).await {
                Ok(tr) => {
                    self.resource_monitor.record_success();
                    tr
                }
                Err(e) => {
                    warn!("Act failed (continuing): {}", e);
                    self.emit_trace(LoopState::Acting, format!("Act failed: {}", e), i);
                    self.resource_monitor.record_failure();
                    TaskResult {
                        success: false,
                        output: e.to_string(),
                        timestamp: chrono::Utc::now(),
                    }
                }
            };
            *self.current_state.lock().await = LoopState::Reflecting;
            self.emit_trace(
                LoopState::Reflecting,
                format!("Reflecting on task result: success={}", task_result.success),
                i,
            );
            match self.reflect(&goal_id, &task_result).await {
                Ok(reflection) => {
                    match self
                        .handle_plan_adjustment(&agent_id, &goal_id, &reflection)
                        .await
                    {
                        Ok(Some(abort_outcome)) => {
                            outcome = abort_outcome;
                            break;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!("Plan adjustment routing failed (continuing): {}", e);
                        }
                    }
                    if !task_result.success && i > ACT_FAILURE_THRESHOLD {
                        outcome = LoopOutcome::ActFailed(task_result.output);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Reflection failed (continuing): {}", e);
                }
            }
            self.resource_monitor
                .record_blackboard_size(self.blackboard.snapshot().await.len());
            *self.current_state.lock().await = LoopState::Storing;
            self.emit_trace(
                LoopState::Storing,
                format!("Storing checkpoint for iteration {}", i),
                i,
            );
            if i % CHECKPOINT_INTERVAL == 0 {
                if let Err(e) = self.store(&agent_id).await {
                    warn!("Store failed (continuing): {}", e);
                }
            }
            *self.current_state.lock().await = LoopState::Deciding;
            self.emit_trace(
                LoopState::Deciding,
                format!("Deciding next action for goal {}", goal_id),
                i,
            );
            match self.decide(&agent_id, &goal_id).await? {
                DecisionOutcome::Continue => {}
                DecisionOutcome::Complete => {
                    outcome = LoopOutcome::Success;
                    break;
                }
                DecisionOutcome::Abort => {
                    outcome = LoopOutcome::Aborted;
                    break;
                }
            }
        }
        *self.current_state.lock().await = LoopState::Completed;
        self.emit_trace(
            LoopState::Completed,
            format!("Loop completed with outcome: {:?}", outcome),
            0,
        );
        info!("AgentLoop completed with outcome: {:?}", outcome);
        Ok(outcome)
    }

    pub fn blackboard(&self) -> &std::sync::Arc<Blackboard> {
        &self.blackboard
    }

    pub async fn execute_goal(
        &self,
        agent_id: AgentId,
        description: &str,
    ) -> ReplResult<LoopOutcome> {
        let (parsed_goal, priority) = Self::from_natural_language(description);
        info!(
            "Executing goal '{}' with priority {:?}",
            parsed_goal, priority
        );
        self.run(agent_id, &parsed_goal).await
    }

    pub(crate) async fn plan_initial_goal(&self, description: &str) -> ReplResult<String> {
        if !self.gov_check("create_goal", description, 0.1).await? {
            return Err(ReplError::Session(
                "Goal creation rejected by governance".to_string(),
            ));
        }
        self.planner
            .lock()
            .await
            .create_goal(description, Priority::High)
            .await
    }

    async fn observe(&self, agent_id: &AgentId) -> Observation {
        info!("Observing for {}", agent_id);
        let bb_state = self.blackboard.snapshot().await;
        Observation {
            agent_id: agent_id.clone(),
            blackboard_keys: bb_state.keys().cloned().collect(),
            timestamp: chrono::Utc::now(),
        }
    }

    async fn retrieve(&self, agent_id: &AgentId) {
        let iter = *self.iteration_count.lock().await;
        match self.memory_retriever.retrieve(agent_id).await {
            RetrieveOutcome::CacheHit(summary) => {
                self.emit_trace(
                    LoopState::Retrieving,
                    format!("Cache hit: {}", summary),
                    iter,
                );
            }
            RetrieveOutcome::Retrieved { summary } => {
                self.emit_trace(
                    LoopState::Retrieving,
                    format!("Retrieved {}", summary),
                    iter,
                );
            }
            RetrieveOutcome::Error(e) => {
                self.emit_trace(
                    LoopState::Retrieving,
                    format!("Retrieval error: {}", e),
                    iter,
                );
            }
        }
    }

    pub(crate) async fn act(&self, agent_id: &AgentId, goal_id: &str) -> ReplResult<TaskResult> {
        info!("Acting for {} on goal {}", agent_id, goal_id);
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(
            LoopState::Acting,
            format!("Acting start for goal {}", goal_id),
            iter,
        );
        if !self
            .gov_check("act", &format!("Execute task for {}", goal_id), 0.1)
            .await?
        {
            return Ok(TaskResult {
                success: false,
                output: "Act rejected by governance".to_string(),
                timestamp: chrono::Utc::now(),
            });
        }
        if crate::prompts::is_agent_llm_bootstrap_enabled() {
            if let Err(e) = self.bootstrap_first_tool_call(goal_id, agent_id).await {
                warn!("LLM bootstrap failed during act: {}", e);
            }
        }
        if crate::prompts::is_act_toolcall_v1_enabled() {
            if let Some(result) = self.try_act_executor_chain(agent_id).await? {
                return Ok(result);
            }
        } else {
            self.emit_trace(
                LoopState::Acting,
                "Act ToolCall V1 disabled, using legacy act path".to_string(),
                iter,
            );
        }
        self.legacy_act(agent_id, goal_id).await
    }

    async fn try_act_executor_chain(&self, agent_id: &AgentId) -> ReplResult<Option<TaskResult>> {
        let next_tool_entry = self.blackboard.read(BB_NEXT_TOOL).await;
        let Some(entry) = next_tool_entry else {
            return Ok(None);
        };
        let call = match serde_json::from_str::<ToolCallV1>(&entry.value) {
            Ok(call) => call,
            Err(_) => return Ok(None),
        };
        // SAFETY: If tool_registry is not injected, we fall back to an empty one.
        let registry = self.tool_registry.clone().unwrap_or_else(|| {
            warn!("No ToolRegistry injected, falling back to an empty one");
            Arc::new(Mutex::new(ToolRegistry::default()))
        });
        let act_executor = ActExecutor::new(registry, self.governance.clone());
        let result = act_executor
            .execute_chain(&self.context, &self.blackboard, agent_id, &call)
            .await;
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(
            LoopState::Acting,
            format!(
                "ActExecutor chain completed: success={}, decision={:?}",
                result.success, result.decision
            ),
            iter,
        );
        Ok(Some(TaskResult {
            success: result.success,
            output: result.output,
            timestamp: chrono::Utc::now(),
        }))
    }

    /// Extract a file path hint from a natural-language description.
    /// Supports patterns like "named X", "名为 X", "file X", "path X".
    fn extract_file_path(desc: &str) -> Option<String> {
        let desc_lower = desc.to_lowercase();
        // Chinese: 名为 xxx 的文件 / 文件 xxx
        if let Some(start) = desc_lower.find("名为 ") {
            let rest = &desc[start + 6..]; // "名为 " is 6 bytes in UTF-8
            let end = rest.find(' ').unwrap_or(rest.len());
            let path = rest[..end]
                .trim()
                .trim_matches(|c| c == '，' || c == ',' || c == '"' || c == '\'');
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
        // English: named xxx / file xxx
        if let Some(start) = desc_lower.find("named ") {
            let rest = &desc[start + 6..];
            let end = rest.find(' ').unwrap_or(rest.len());
            let path = rest[..end]
                .trim()
                .trim_matches(|c| c == ',' || c == '"' || c == '\'');
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
        None
    }

    /// Extract (path, content) for write_file from a natural-language description.
    /// Supports patterns like "content is Y" / "内容是 Y".
    fn extract_write_file_params(desc: &str) -> (String, String) {
        let path = Self::extract_file_path(desc).unwrap_or_else(|| "output.txt".to_string());
        let desc_lower = desc.to_lowercase();
        // Chinese: 内容是 xxx
        if let Some(start) = desc_lower.find("内容是 ") {
            let rest = &desc[start + 9..]; // "内容是 " is 9 bytes in UTF-8
            let content = rest.trim().trim_matches(|c| c == '"' || c == '\'');
            return (path, content.to_string());
        }
        // English: content is xxx
        if let Some(start) = desc_lower.find("content is ") {
            let rest = &desc[start + 11..];
            let content = rest.trim().trim_matches(|c| c == '"' || c == '\'');
            return (path, content.to_string());
        }
        // Fallback: use the whole description as content
        (path, desc.to_string())
    }

    /// LEGACY / OFFLINE FALLBACK
    ///
    /// LLM-NATIVE-TODO (Phase 3+): This is the third (and most destructive) layer of local rule-based intent mapping.
    /// The massive if-else chain at ~628-692 (and the duplicate in bootstrap_first_tool_call at ~848-910)
    /// performs keyword matching on Task.description — which has already been mangled by upstream Planner rules.
    ///
    /// In the LLM-Native path this entire method must be unreachable for normal operation.
    /// It is retained only as an emergency offline fallback.
    /// See LLM-NATIVE-AGENT-MIGRATION-001-EXECUTION-PLAN.md Day 10-14 for the systematic demotion plan.
    pub(crate) async fn legacy_act(
        &self,
        agent_id: &AgentId,
        goal_id: &str,
    ) -> ReplResult<TaskResult> {
        // legacy act path: preserve the pre-ActExecutor swarm/local fallback behavior.
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(
            LoopState::Acting,
            format!("legacy act path for goal {}", goal_id),
            iter,
        );
        let task_opt = { self.planner.lock().await.next_task().await? };
        if let Some(task) = task_opt {
            let mut delegate_success = false;
            let mut swarm_result = None;

            if let Some(ref swarm) = self.swarm {
                match SwarmDelegate::try_delegate(swarm, &self.blackboard, agent_id, &task).await {
                    Some(Ok(result)) => {
                        self.emit_trace(
                            LoopState::Acting,
                            format!(
                                "Task {} completed via Swarm execution: success={}",
                                task.id, result.success
                            ),
                            iter,
                        );
                        delegate_success = true;
                        swarm_result = Some(Ok(result));
                    }
                    Some(Err(e)) => {
                        warn!("Swarm delegation failed (falling back to local): {}", e);
                        self.emit_trace(
                            LoopState::Acting,
                            format!("Delegation failed for task {}: {}", task.id, e),
                            iter,
                        );
                        // Fall through to local execution
                    }
                    None => {
                        self.emit_trace(
                            LoopState::Acting,
                            "No idle worker available, falling back to local execution".to_string(),
                            iter,
                        );
                        // Fall through to local execution
                    }
                }
            }

            if delegate_success {
                if let Some(res) = swarm_result {
                    return res;
                }
            }

            // Local execution path (Day 6)
            self.emit_trace(
                LoopState::Acting,
                format!("Local execution path activated for task {}", task.id),
                iter,
            );

            // Perform local execution using registry
            let registry = self.tool_registry.clone().unwrap_or_else(|| {
                warn!("No ToolRegistry injected, falling back to an empty one");
                Arc::new(Mutex::new(ToolRegistry::default()))
            });

            // Determine the tool call
            let (tool_name, parameters) = if !task.tool_calls.is_empty() {
                let tc = &task.tool_calls[0];
                (
                    tc.tool_name.clone(),
                    serde_json::Value::Object(tc.parameters.clone().into_iter().collect()),
                )
            } else {
                // Task has no tool calls, perform rule-based mapping (fallback)
                // FIX-B08-005: Support Chinese keywords and parse parameters from description.
                let desc_lower = task.description.to_lowercase();
                let tool_name = if desc_lower.contains("read")
                    || desc_lower.contains("analyze")
                    || desc_lower.contains("读")
                    || desc_lower.contains("查看")
                    || desc_lower.contains("分析")
                {
                    "read_file".to_string()
                } else if desc_lower.contains("write")
                    || desc_lower.contains("edit")
                    || desc_lower.contains("create")
                    || desc_lower.contains("implement")
                    || desc_lower.contains("写")
                    || desc_lower.contains("编辑")
                    || desc_lower.contains("创建")
                    || desc_lower.contains("生成")
                    || desc_lower.contains("新建")
                {
                    "write_file".to_string()
                } else if desc_lower.contains("test")
                    || desc_lower.contains("run")
                    || desc_lower.contains("compile")
                    || desc_lower.contains("build")
                    || desc_lower.contains("测试")
                    || desc_lower.contains("运行")
                    || desc_lower.contains("编译")
                    || desc_lower.contains("构建")
                {
                    "powershell".to_string()
                } else {
                    "read_file".to_string() // default safe fallback
                };

                // Verify if the mapped tool exists in registry, else fallback to any available safe tool or default
                let mut resolved_tool_name = tool_name;
                {
                    let guard = registry.lock().await;
                    if guard.get(&resolved_tool_name).is_none() {
                        for alternative in &["analyze", "ls", "read_file"] {
                            if guard.get(alternative).is_some() {
                                resolved_tool_name = alternative.to_string();
                                break;
                            }
                        }
                    }
                }

                let parameters = if resolved_tool_name == "powershell" {
                    serde_json::json!({ "command": "echo 'local check'" })
                } else if resolved_tool_name == "read_file" {
                    // Try to extract file path from description, fallback to task description as hint
                    let path = Self::extract_file_path(&task.description)
                        .unwrap_or_else(|| "Cargo.toml".to_string());
                    serde_json::json!({ "path": path })
                } else if resolved_tool_name == "write_file" {
                    // Try to extract file path and content from description
                    let (path, content) = Self::extract_write_file_params(&task.description);
                    serde_json::json!({ "path": path, "content": content })
                } else {
                    serde_json::json!({})
                };

                (resolved_tool_name, parameters)
            };

            info!(
                "Executing tool '{}' via Local execution for task: {}",
                tool_name, task.description
            );
            let guard = registry.lock().await;
            if let Some(tool) = guard.get(&tool_name) {
                // Execute tool
                match tool.execute(parameters).await {
                    Ok(output) => {
                        let is_success = output.exit_code == Some(0);
                        let content = if is_success {
                            &output.stdout
                        } else {
                            &output.stderr
                        };
                        let summary = if is_success {
                            format!("Local execution of {} succeeded: {}", tool_name, content)
                        } else {
                            format!("Local execution of {} failed: {}", tool_name, content)
                        };
                        self.blackboard
                            .write(crate::act_executor::BB_LAST_TOOL_RESULT, &summary, agent_id)
                            .await;
                        self.emit_trace(
                            LoopState::Acting,
                            format!(
                                "Local execution of {} completed. Success: {}",
                                tool_name, is_success
                            ),
                            iter,
                        );
                        Ok(TaskResult {
                            success: is_success,
                            output: summary,
                            timestamp: chrono::Utc::now(),
                        })
                    }
                    Err(e) => {
                        let summary =
                            format!("Local execution of {} returned error: {}", tool_name, e);
                        self.blackboard
                            .write(crate::act_executor::BB_LAST_TOOL_RESULT, &summary, agent_id)
                            .await;
                        warn!("{}", summary);
                        Ok(TaskResult {
                            success: false,
                            output: summary,
                            timestamp: chrono::Utc::now(),
                        })
                    }
                }
            } else {
                let err_msg = format!(
                    "Local execution failed: Tool '{}' not found in registry",
                    tool_name
                );
                self.blackboard
                    .write(crate::act_executor::BB_LAST_TOOL_RESULT, &err_msg, agent_id)
                    .await;
                warn!("{}", err_msg);
                Ok(TaskResult {
                    success: false,
                    output: err_msg,
                    timestamp: chrono::Utc::now(),
                })
            }
        } else {
            Ok(TaskResult {
                success: true,
                output: "No pending tasks".to_string(),
                timestamp: chrono::Utc::now(),
            })
        }
    }

    // LLM-NATIVE-TODO (Phase 3+): bootstrap_first_tool_call still goes through Planner (decompose/expand)
    // and contains its own duplicate rule-mapping logic (~848-910) that is almost identical to legacy_act.
    // In LLM-Native mode this path should be bypassed entirely except for pure offline fallback scenarios.
    /// Bootstrap the first tool call when blackboard has no pending tools.
    ///
    /// This method is part of the LLM Bootstrap mechanism that automatically populates the planner
    /// tasks and generates the first tool call if the planner is empty.
    pub async fn bootstrap_first_tool_call(
        &self,
        goal_id: &str,
        agent_id: &str,
    ) -> ReplResult<Option<ToolCallV1>> {
        // Trigger condition: BB_NEXT_TOOL is empty and next_task() is None.
        let next_tool_entry = self.blackboard.read(BB_NEXT_TOOL).await;
        if next_tool_entry.is_some() {
            return Ok(None);
        }

        // Lock the planner to check next_task and call decompose/expand
        let mut planner = self.planner.lock().await;
        let next_task = planner.next_task().await?;
        if next_task.is_some() {
            return Ok(None);
        }

        info!(
            "AgentLoop bootstrapping first tool call for goal: {}",
            goal_id
        );

        // 1. Decompose the goal into subgoals
        let sg_ids = match planner.decompose(&goal_id.to_string()).await {
            Ok(ids) => ids,
            Err(e) => {
                warn!("Planner decompose failed during bootstrap, fallback rule-based already active: {}", e);
                return Err(e);
            }
        };

        if sg_ids.is_empty() {
            warn!(
                "No subgoals generated during bootstrap for goal {}",
                goal_id
            );
            return Ok(None);
        }

        // 2. Expand the first subgoal to generate tasks
        let first_sg_id = &sg_ids[0];
        let task_ids = planner.expand(first_sg_id).await?;
        if task_ids.is_empty() {
            warn!(
                "No tasks expanded for subgoal {} during bootstrap",
                first_sg_id
            );
            return Ok(None);
        }

        // 3. Resolve the next task
        let task_opt = planner.next_task().await?;
        let Some(task) = task_opt else {
            warn!("No next task resolved after expand in bootstrap");
            return Ok(None);
        };

        // 4. Convert Task to ToolCallV1
        let tool_call = if !task.tool_calls.is_empty() {
            let tc = &task.tool_calls[0];
            ToolCallV1 {
                schema_version: "1".to_string(),
                action_type: crate::act_dto::ActionType::CallTool,
                tool_name: tc.tool_name.clone(),
                parameters: serde_json::Value::Object(tc.parameters.clone().into_iter().collect()),
                reason: format!("Executing task: {}", task.description),
                expected_output: "Success".to_string(),
                expected_evidence: "Success".to_string(),
                fallback_tool: None,
                governance_required: false,
                risk_level: crate::tool_manifest::RiskLevel::Low,
                idempotency_key: format!("{}-bootstrap-{}", agent_id, uuid::Uuid::new_v4()),
                next_step_hint: None,
            }
        } else {
            // Task has no tool calls, perform rule-based mapping (fallback)
            let desc = task.description.to_lowercase();
            let tool_name = if desc.contains("read") || desc.contains("analyze") {
                "read_file".to_string()
            } else if desc.contains("write")
                || desc.contains("edit")
                || desc.contains("create")
                || desc.contains("implement")
            {
                "write_file".to_string()
            } else if desc.contains("test")
                || desc.contains("run")
                || desc.contains("compile")
                || desc.contains("build")
            {
                "powershell".to_string()
            } else {
                "read_file".to_string() // default safe fallback
            };

            // Verify if the mapped tool exists in self.tool_registry, else fallback to any available safe tool or default
            let mut resolved_tool_name = tool_name;
            if let Some(ref reg) = self.tool_registry {
                let guard = reg.lock().await;
                if guard.get(&resolved_tool_name).is_none() {
                    // Try some safe alternatives
                    for alternative in &["analyze", "ls", "read_file"] {
                        if guard.get(alternative).is_some() {
                            resolved_tool_name = alternative.to_string();
                            break;
                        }
                    }
                }
            }

            // Create default parameters matching the chosen tool
            let parameters = if resolved_tool_name == "powershell" {
                serde_json::json!({ "command": "echo 'bootstrap check'" })
            } else if resolved_tool_name == "read_file" {
                serde_json::json!({ "path": "Cargo.toml" })
            } else {
                serde_json::json!({})
            };

            ToolCallV1 {
                schema_version: "1".to_string(),
                action_type: crate::act_dto::ActionType::CallTool,
                tool_name: resolved_tool_name,
                parameters,
                reason: format!(
                    "Bootstrap rule-based fallback for task: {}",
                    task.description
                ),
                expected_output: "Success".to_string(),
                expected_evidence: "Success".to_string(),
                fallback_tool: None,
                governance_required: false,
                risk_level: crate::tool_manifest::RiskLevel::Low,
                idempotency_key: format!("{}-bootstrap-{}", agent_id, uuid::Uuid::new_v4()),
                next_step_hint: None,
            }
        };

        // Validate tool_name using registry before writing
        let valid_tool = if let Some(ref reg) = self.tool_registry {
            let guard = reg.lock().await;
            guard.get(&tool_call.tool_name).is_some()
        } else {
            false
        };

        if !valid_tool {
            warn!(
                "Bootstrapped tool '{}' not found in registry",
                tool_call.tool_name
            );
            return Ok(None);
        }

        // Write ToolCallV1 to BB_NEXT_TOOL as a serialized JSON string
        let json = serde_json::to_string(&tool_call).map_err(ReplError::Protocol)?;
        self.blackboard.write(BB_NEXT_TOOL, &json, agent_id).await;

        // Trace event with tool name
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(
            LoopState::Acting,
            format!(
                "LLM bootstrap generated ToolCallV1: {}",
                tool_call.tool_name
            ),
            iter,
        );

        Ok(Some(tool_call))
    }

    pub(crate) async fn reflect(
        &self,
        goal_id: &str,
        result: &TaskResult,
    ) -> ReplResult<crate::reflector::Reflection> {
        info!(
            "Reflecting on goal {} with success={}",
            goal_id, result.success
        );
        let agent_id = "agent_loop".to_string();
        if let Err(e) = self
            .evaluate_and_record_skills_receipts(&agent_id, &result.output)
            .await
        {
            warn!("Failed to evaluate and record skill receipts: {}", e);
        }
        let iter = *self.iteration_count.lock().await;
        self.emit_trace(
            LoopState::Reflecting,
            format!("Reflecting on goal {}", goal_id),
            iter,
        );
        let req = GovernanceRequest {
            requester: "agent_loop".to_string(),
            action_type: "reflect".to_string(),
            risk_score: 0.1,
            description: format!("Reflect on goal {}", goal_id),
            level: ApprovalLevel::Auto,
        };
        let gov_approved = match self.governance.approve(&self.context, &req).await? {
            Decision::Approved => true,
            Decision::Rejected(reason) => {
                warn!("Reflection rejected by governance. Reason: {}", reason);
                false
            }
            other => {
                warn!("Reflection not approved by governance. Status: {:?}", other);
                false
            }
        };

        if !gov_approved {
            warn!("gov_check reflect rejected");
            // Return a synthetic reflection so the caller can continue routing.
            return Ok(crate::reflector::Reflection {
                reflection_id: uuid::Uuid::new_v4().to_string(),
                original_goal_id: goal_id.to_string(),
                execution_result: result.clone(),
                critique: crate::reflector::Critique {
                    success: result.success,
                    issues: vec![],
                    suggestions: vec![],
                    severity: crate::reflector::CritiqueSeverity::Low,
                },
                optimized_plan: None,
                confidence: 0.5,
                timestamp: chrono::Utc::now(),
            });
        }
        // B-09/14: Skip V1 structured reflection when feature-gate is disabled;
        // the Reflector bridge will already fall back to legacy Critique parsing,
        // but we also skip BB routing for plan_adjustment / stop_loss.
        if !crate::prompts::is_reflector_v1_enabled() {
            info!("Reflector V1 feature-gate disabled; using legacy reflection path");
            self.emit_trace(
                LoopState::Reflecting,
                "Reflector V1 disabled, legacy path".to_string(),
                iter,
            );
        }
        let goal = Goal {
            id: goal_id.to_string(),
            description: "Reflected goal".to_string(),
            priority: Priority::Medium,
            status: PlanStatus::InProgress,
            subgoals: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            approved: true,
        };
        let reflection = if let Some(ref swarm) = self.swarm {
            let worker_results = swarm.lock().await.aggregate().await;
            if !worker_results.is_empty() {
                let r = self
                    .reflector
                    .lock()
                    .await
                    .reflect_multi(&goal, &worker_results)
                    .await?;
                self.emit_trace(
                    LoopState::Reflecting,
                    format!("Multi-worker reflection: confidence={:.2}", r.confidence),
                    iter,
                );
                r
            } else {
                self.reflector.lock().await.reflect(&goal, result).await?
            }
        } else {
            self.reflector.lock().await.reflect(&goal, result).await?
        };
        Ok(reflection)
    }

    async fn route_and_load_skills(
        &self,
        agent_id: &AgentId,
        initial_goal: &str,
    ) -> Result<(), String> {
        if !crate::prompts::is_agent_skills_v0_enabled() {
            return Ok(());
        }

        let router = self
            .skill_router
            .as_ref()
            .ok_or_else(|| "skill_router is None".to_string())?;
        let registry = self
            .skill_registry
            .as_ref()
            .ok_or_else(|| "skill_registry is None".to_string())?;

        let route_result = router.route(initial_goal);

        // Keep at most 3 selected skills
        let selected_matches: Vec<_> = route_result.selected.iter().take(3).cloned().collect();

        info!(
            "Skill routing selected {} active skills. Route receipt details: {:?}",
            selected_matches.len(),
            route_result.receipt
        );

        let loader = crate::skills::SkillLoader::new(registry.as_ref().clone());
        let mut loaded_skills = Vec::new();
        let mut eval_criteria = Vec::new();
        let mut instructions_combined = String::new();

        for m in &selected_matches {
            match loader.load(&m.name) {
                Ok(loaded) => {
                    match loader.load_eval_criteria(&m.name) {
                        Ok(Some(criteria)) => eval_criteria.push(criteria),
                        Ok(None) => {}
                        Err(e) => warn!(
                            "Failed to load skill eval criteria for '{}': {}. Continuing.",
                            m.name, e
                        ),
                    }
                    instructions_combined.push_str(&loaded.instructions);
                    instructions_combined.push('\n');
                    loaded_skills.push(loaded);
                }
                Err(e) => {
                    warn!("Failed to load skill '{}': {}. Continuing.", m.name, e);
                }
            }
        }

        let active_skills_json = serde_json::to_string(&loaded_skills)
            .map_err(|e| format!("Failed to serialize active skills: {}", e))?;
        self.blackboard
            .write(
                crate::skills::BB_ACTIVE_SKILLS,
                &active_skills_json,
                agent_id,
            )
            .await;

        let receipt_json = serde_json::to_string(&route_result.receipt)
            .map_err(|e| format!("Failed to serialize route receipt: {}", e))?;
        self.blackboard
            .write(
                crate::skills::BB_SKILL_ROUTE_RECEIPT,
                &receipt_json,
                agent_id,
            )
            .await;

        self.blackboard
            .write(
                crate::skills::BB_SKILL_INSTRUCTIONS,
                &instructions_combined,
                agent_id,
            )
            .await;

        if !eval_criteria.is_empty() {
            let eval_criteria_json = serde_json::to_string(&eval_criteria)
                .map_err(|e| format!("Failed to serialize skill eval criteria: {}", e))?;
            self.blackboard
                .write(
                    crate::skills::BB_SKILL_EVAL_CRITERIA,
                    &eval_criteria_json,
                    agent_id,
                )
                .await;
        }

        if crate::skills::is_agent_skill_runtime_enabled() && !loaded_skills.is_empty() {
            let runtime = crate::skills::SkillRuntime::new(
                crate::skills::default_runtime_tool_names().iter().copied(),
            );
            let tool_constraints: Vec<_> = loaded_skills
                .iter()
                .map(|skill| runtime.build_tool_constraints(&skill.manifest))
                .collect();
            let constraints_json = serde_json::to_string(&tool_constraints)
                .map_err(|e| format!("Failed to serialize skill tool constraints: {}", e))?;
            self.blackboard
                .write(
                    crate::skills::BB_SKILL_TOOL_CONSTRAINTS,
                    &constraints_json,
                    agent_id,
                )
                .await;
        }

        Ok(())
    }

    async fn evaluate_and_record_skills_receipts(
        &self,
        agent_id: &AgentId,
        output: &str,
    ) -> Result<(), String> {
        if !crate::prompts::is_agent_skills_v0_enabled() {
            return Ok(());
        }

        let registry = match &self.skill_registry {
            Some(r) => r,
            None => return Ok(()),
        };

        // Read BB_SKILL_ROUTE_RECEIPT from blackboard
        let route_receipt_entry = self
            .blackboard
            .read(crate::skills::BB_SKILL_ROUTE_RECEIPT)
            .await;
        let Some(entry) = route_receipt_entry else {
            return Ok(());
        };

        let route_receipt: crate::skills::SkillRouteReceipt = serde_json::from_str(&entry.value)
            .map_err(|e| format!("Failed to deserialize route receipt: {}", e))?;

        if route_receipt.selected.is_empty() {
            return Ok(());
        }

        // Read BB_ACTIVE_SKILLS from blackboard
        let active_skills_entry = self.blackboard.read(crate::skills::BB_ACTIVE_SKILLS).await;
        let active_skills: Vec<crate::skills::LoadedSkill> = if let Some(e) = active_skills_entry {
            serde_json::from_str(&e.value).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Evaluate output using evaluate_output
        let report =
            crate::skills::eval::evaluate_output(&route_receipt.selected, output, registry);

        // Generate receipts
        let receipts = crate::skills::eval::generate_execution_receipts(
            &report,
            &route_receipt,
            &active_skills,
        );

        // Serialize and write to blackboard under BB_SKILL_EXECUTION_RECEIPTS
        let receipts_json = serde_json::to_string(&receipts)
            .map_err(|e| format!("Failed to serialize execution receipts: {}", e))?;

        self.blackboard
            .write(
                crate::skills::BB_SKILL_EXECUTION_RECEIPTS,
                &receipts_json,
                agent_id,
            )
            .await;

        Ok(())
    }

    async fn store(&self, agent_id: &AgentId) -> ReplResult<()> {
        self.memory_retriever
            .store(agent_id, &self.checkpoint_mgr)
            .await
    }

    async fn decide(&self, agent_id: &AgentId, goal_id: &str) -> ReplResult<DecisionOutcome> {
        info!("Deciding for {} on goal {}", agent_id, goal_id);
        let req = GovernanceRequest {
            requester: agent_id.clone(),
            action_type: "decide_next".to_string(),
            risk_score: 0.1,
            description: format!("Decide next step for goal {}", goal_id),
            level: ApprovalLevel::Auto,
        };
        match self.governance.approve(&self.context, &req).await? {
            Decision::Approved => {
                if self.planner.lock().await.is_complete() {
                    Ok(DecisionOutcome::Complete)
                } else {
                    Ok(DecisionOutcome::Continue)
                }
            }
            Decision::Rejected(_) => Ok(DecisionOutcome::Abort),
            _ => Ok(DecisionOutcome::Continue),
        }
    }

    async fn gov_check(&self, action: &str, desc: &str, risk: f32) -> ReplResult<bool> {
        let req = GovernanceRequest {
            requester: "agent_loop".to_string(),
            action_type: action.to_string(),
            risk_score: risk,
            description: desc.to_string(),
            level: ApprovalLevel::Auto,
        };
        match self.governance.approve(&self.context, &req).await? {
            Decision::Approved => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn current_state(&self) -> LoopState {
        *self.current_state.lock().await
    }
    pub async fn iteration_count(&self) -> usize {
        *self.iteration_count.lock().await
    }

    pub fn subscribe_trace(&self) -> Option<tokio::sync::broadcast::Receiver<TraceEvent>> {
        self.trace_tx.as_ref().map(|tx| tx.subscribe())
    }

    /// Returns a clone of the broadcast sender for trace events.
    /// Callers can subscribe to this sender to receive real-time AgentLoop traces.
    pub fn trace_tx(&self) -> Option<tokio::sync::broadcast::Sender<TraceEvent>> {
        self.trace_tx.clone()
    }

    fn emit_trace(&self, step: LoopState, details: String, iteration: usize) {
        self.emit_trace_with_meta(
            step,
            details,
            iteration,
            None,
            vec![],
            None,
            None,
            None,
            None,
        );
    }

    #[allow(clippy::too_many_arguments)]
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
                step,
                details,
                iteration,
                timestamp: chrono::Utc::now(),
                step_type,
                plan_summary,
                reflection_key_points,
                confidence_score,
                edit_payload,
                operation_summary,
                thinking_content,
            };
            let _ = tx.send(event);
        }
    }

    /// Handle the recommended action from reflection and route to the appropriate branch.
    /// Returns `Some(LoopOutcome::Aborted)` when StopAndHandoff is triggered. (B-08/14)
    async fn handle_plan_adjustment(
        &self,
        agent_id: &AgentId,
        goal_id: &str,
        reflection: &crate::reflector::Reflection,
    ) -> ReplResult<Option<LoopOutcome>> {
        let action = self.infer_action(reflection).await;
        let category = {
            let text = reflection.critique.issues.join(" ").to_lowercase();
            if text.contains("tool") || text.contains("command not found") {
                "ToolFailure".to_string()
            } else if text.contains("plan") || text.contains("bad plan") {
                "BadPlan".to_string()
            } else if text.contains("permission") || text.contains("access denied") {
                "Permission".to_string()
            } else if text.contains("parse") || text.contains("deserialize") {
                "ParseFailure".to_string()
            } else if text.contains("validation") {
                "ValidationFailure".to_string()
            } else if text.contains("context") || text.contains("missing") {
                "MissingContext".to_string()
            } else if text.contains("user") {
                "UserInputNeeded".to_string()
            } else {
                "Unknown".to_string()
            }
        };
        let evidence: Vec<String> = reflection
            .critique
            .issues
            .iter()
            .chain(reflection.critique.suggestions.iter())
            .cloned()
            .collect();

        // Stop-Loss: same category failed >=2 times with no new evidence -> force StopAndHandoff.
        if self.check_stop_loss(&category, &evidence, agent_id).await {
            warn!(
                "Stop-Loss triggered for category {} (>=2 failures, no new evidence)",
                category
            );
            let summary = self.build_handoff_summary(reflection);
            self.blackboard
                .write(BB_HANDOFF_SUMMARY, &summary, agent_id)
                .await;
            self.emit_trace(
                LoopState::Reflecting,
                format!("Stop-Loss forced handoff: {}", summary),
                *self.iteration_count.lock().await,
            );
            return Ok(Some(LoopOutcome::Aborted));
        }

        // Update failure counter when critique indicates failure.
        if !reflection.critique.success {
            self.update_failure_count(&category, agent_id).await;
        }

        match action {
            RecommendedAction::Continue => {
                info!("Plan adjustment: Continue");
            }
            RecommendedAction::RetryWithNewArgs => {
                info!("Plan adjustment: RetryWithNewArgs");
                self.blackboard.write(BB_RETRY_FLAG, "true", agent_id).await;
                self.emit_trace(
                    LoopState::Reflecting,
                    "RetryWithNewArgs flagged".to_string(),
                    *self.iteration_count.lock().await,
                );
            }
            RecommendedAction::UseAlternativeTool => {
                info!("Plan adjustment: UseAlternativeTool");
                // DEBT-B08-002: Select an alternative tool from suggested_tools instead of just flagging.
                let mut alt_tool: Option<String> = None;
                // Priority 1: parse suggested_tools from BB_PLAN_ADJUSTMENT JSON.
                if let Some(entry) = self.blackboard.read(BB_PLAN_ADJUSTMENT).await {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&entry.value) {
                        if let Some(tools) = json.get("suggested_tools").and_then(|v| v.as_array())
                        {
                            alt_tool = tools
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .find(|s| !s.is_empty());
                        }
                    }
                }
                // Priority 2: fallback to blackboard __hajimi_suggested_tools.
                if alt_tool.is_none() {
                    if let Some(entry) = self.blackboard.read("__hajimi_suggested_tools").await {
                        if let Ok(tools) = serde_json::from_str::<Vec<String>>(&entry.value) {
                            alt_tool = tools.into_iter().find(|s| !s.is_empty());
                        }
                    }
                }
                if let Some(tool) = alt_tool {
                    self.blackboard
                        .write(BB_ALT_TOOL_NAME, &tool, agent_id)
                        .await;
                    self.emit_trace(
                        LoopState::Reflecting,
                        format!("UseAlternativeTool selected: {}", tool),
                        *self.iteration_count.lock().await,
                    );
                } else {
                    warn!("No suggested_tools available for UseAlternativeTool; falling back to flag only");
                    self.blackboard
                        .write(BB_ALT_TOOL_FLAG, "true", agent_id)
                        .await;
                    self.emit_trace(
                        LoopState::Reflecting,
                        "UseAlternativeTool flagged (no tools available)".to_string(),
                        *self.iteration_count.lock().await,
                    );
                }
            }
            RecommendedAction::RevisePlan => {
                info!("Plan adjustment: RevisePlan");
                let goal = Goal {
                    id: goal_id.to_string(),
                    description: "Revised goal".to_string(),
                    priority: Priority::Medium,
                    status: PlanStatus::InProgress,
                    subgoals: vec![],
                    metadata: HashMap::new(),
                    created_at: chrono::Utc::now(),
                    approved: true,
                };
                match self
                    .reflector
                    .lock()
                    .await
                    .optimize_plan(&goal, &reflection.critique)
                    .await
                {
                    Ok(Some(plan)) => {
                        self.emit_trace(
                            LoopState::Reflecting,
                            format!("Plan revised: {} subgoals", plan.subgoals.len()),
                            *self.iteration_count.lock().await,
                        );
                    }
                    Ok(None) => {
                        info!("Plan optimizer returned no changes");
                    }
                    Err(e) => {
                        warn!("Plan optimizer failed (continuing): {}", e);
                    }
                }
            }
            RecommendedAction::AskUser => {
                info!("Plan adjustment: AskUser");
                let reason = reflection
                    .critique
                    .suggestions
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "User input needed".to_string());
                self.blackboard
                    .write(BB_ASK_USER_REASON, &reason, agent_id)
                    .await;
                self.pause();
                self.emit_trace(
                    LoopState::Reflecting,
                    format!("AskUser paused: {}", reason),
                    *self.iteration_count.lock().await,
                );
            }
            RecommendedAction::StopAndHandoff => {
                info!("Plan adjustment: StopAndHandoff");
                let summary = self.build_handoff_summary(reflection);
                self.blackboard
                    .write(BB_HANDOFF_SUMMARY, &summary, agent_id)
                    .await;
                self.emit_trace(
                    LoopState::Reflecting,
                    format!("StopAndHandoff: {}", summary),
                    *self.iteration_count.lock().await,
                );
                return Ok(Some(LoopOutcome::Aborted));
            }
        }
        Ok(None)
    }

    /// Infer RecommendedAction from reflection; prefer blackboard-stored plan adjustment if present.
    async fn infer_action(&self, reflection: &crate::reflector::Reflection) -> RecommendedAction {
        // If the reflector (or bridge) wrote an explicit plan adjustment to the blackboard, use it.
        if let Some(entry) = self.blackboard.read(BB_PLAN_ADJUSTMENT).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&entry.value) {
                if let Some(action_str) = json.get("action").and_then(|v| v.as_str()) {
                    match action_str {
                        "Continue" => return RecommendedAction::Continue,
                        "RetryWithNewArgs" => return RecommendedAction::RetryWithNewArgs,
                        "UseAlternativeTool" => return RecommendedAction::UseAlternativeTool,
                        "RevisePlan" => return RecommendedAction::RevisePlan,
                        "AskUser" => return RecommendedAction::AskUser,
                        "StopAndHandoff" => return RecommendedAction::StopAndHandoff,
                        _ => {}
                    }
                }
            }
        }
        // Fallback: heuristic based on critique content.
        let critique = &reflection.critique;
        if critique.success {
            return RecommendedAction::Continue;
        }
        let text = critique.suggestions.join(" ").to_lowercase();
        if text.contains("ask user") {
            return RecommendedAction::AskUser;
        }
        // FIX-B08-001: "handoff" maps to StopAndHandoff, not AskUser.
        if text.contains("handoff") {
            return RecommendedAction::StopAndHandoff;
        }
        if text.contains("alternative tool") || text.contains("try another tool") {
            return RecommendedAction::UseAlternativeTool;
        }
        match critique.severity {
            crate::reflector::CritiqueSeverity::Critical => RecommendedAction::StopAndHandoff,
            crate::reflector::CritiqueSeverity::High => RecommendedAction::RevisePlan,
            crate::reflector::CritiqueSeverity::Medium => RecommendedAction::RetryWithNewArgs,
            crate::reflector::CritiqueSeverity::Low => RecommendedAction::Continue,
        }
    }

    /// Update per-category failure counter on blackboard.
    async fn update_failure_count(&self, category: &str, agent_id: &AgentId) {
        let mut counts: HashMap<String, usize> = self
            .blackboard
            .read(BB_FAILURE_COUNT)
            .await
            .and_then(|e| serde_json::from_str(&e.value).ok())
            .unwrap_or_default();
        *counts.entry(category.to_string()).or_insert(0) += 1;
        if let Ok(json) = serde_json::to_string(&counts) {
            self.blackboard
                .write(BB_FAILURE_COUNT, &json, agent_id)
                .await;
        }
    }

    /// Check stop-loss: same category >=2 failures with no new evidence.
    async fn check_stop_loss(
        &self,
        category: &str,
        evidence: &[String],
        agent_id: &AgentId,
    ) -> bool {
        let counts: HashMap<String, usize> = self
            .blackboard
            .read(BB_FAILURE_COUNT)
            .await
            .and_then(|e| serde_json::from_str(&e.value).ok())
            .unwrap_or_default();
        if counts.get(category).copied().unwrap_or(0) < 2 {
            return false;
        }
        // DEBT-B08-003: Use HashSet for robust evidence set comparison.
        let last: HashSet<String> = self
            .blackboard
            .read(BB_LAST_EVIDENCE)
            .await
            .and_then(|e| serde_json::from_str(&e.value).ok())
            .unwrap_or_default();
        let current: HashSet<String> = evidence.iter().cloned().collect();
        let no_new = current.is_subset(&last) && last.is_subset(&current);
        if let Ok(json) = serde_json::to_string(evidence) {
            self.blackboard
                .write(BB_LAST_EVIDENCE, &json, agent_id)
                .await;
        }
        no_new
    }

    /// Build handoff summary for StopAndHandoff.
    fn build_handoff_summary(&self, reflection: &crate::reflector::Reflection) -> String {
        format!(
            "Handoff: success={}, severity={:?}, issues={}, suggestions={}",
            reflection.critique.success,
            reflection.critique.severity,
            reflection.critique.issues.join("; "),
            reflection.critique.suggestions.join("; ")
        )
    }

    pub fn with_edit_applier(mut self, applier: Arc<EditApplier>) -> Self {
        self.edit_applier = Some(applier);
        self
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
        info!("AgentLoop pause requested");
    }
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
        info!("AgentLoop resumed");
    }
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub async fn inject_memory(
        &self,
        key: &str,
        value: &str,
        agent_id: &AgentId,
    ) -> ReplResult<()> {
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
        let priority = if goal.contains("urgent") || goal.contains("critical") {
            Priority::Critical
        } else if goal.contains("important") || goal.contains("high") {
            Priority::High
        } else if goal.contains("low") || goal.contains("minor") {
            Priority::Low
        } else {
            Priority::Medium
        };
        (goal.to_string(), priority)
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub agent_id: AgentId,
    pub blackboard_keys: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopOutcome {
    InProgress,
    Success,
    Aborted,
    BudgetExceeded,
    ActFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecisionOutcome {
    Continue,
    Complete,
    Abort,
}

/// Re-export thinking extraction for AgentLoop consumers (B-08/12).
pub use crate::planner::extract_thinking as extract_thinking_content;

#[cfg(test)]
#[allow(clippy::useless_vec)]
mod tests {
    use super::*;
    use crate::agent_loop_builder::AgentLoopConfig;
    use crate::checkpoint::CheckpointManager;
    use crate::governance::DefaultGovernance;
    use crate::planner::HierarchicalPlanner;
    use crate::reflector::AutonomousReflector;
    use memory::memory_gateway::MemoryGateway;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn mk_test_loop() -> AgentLoop {
        let mem = Arc::new(Mutex::new(MemoryGateway::new("test")));
        AgentLoop::new(AgentLoopConfig {
            context: AgentContext::new(),
            planner: Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>,
            reflector: Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>,
            governance: Arc::new(DefaultGovernance::new()),
            swarm: None,
            blackboard: Arc::new(Blackboard::new()),
            checkpoint_mgr: Arc::new(CheckpointManager::new()),
            memory: Some(mem),
            sync_gateway: None,
            provider_id: None,
            edit_applier: None,
            skill_registry: None,
            skill_router: None,
            tool_registry: None,
            native_driver: None,
        })
    }

    fn mk_reflection(
        success: bool,
        severity: crate::reflector::CritiqueSeverity,
        issues: Vec<&str>,
        suggestions: Vec<&str>,
    ) -> crate::reflector::Reflection {
        crate::reflector::Reflection {
            reflection_id: "r1".to_string(),
            original_goal_id: "g1".to_string(),
            execution_result: TaskResult {
                success,
                output: "test".to_string(),
                timestamp: chrono::Utc::now(),
            },
            critique: crate::reflector::Critique {
                success,
                issues: issues.into_iter().map(|s| s.to_string()).collect(),
                suggestions: suggestions.into_iter().map(|s| s.to_string()).collect(),
                severity,
            },
            optimized_plan: None,
            confidence: 0.5,
            timestamp: chrono::Utc::now(),
        }
    }

    /// B-08/14: Continue routing does not pause or write handoff.
    #[tokio::test]
    async fn test_handle_plan_adjustment_continue() {
        let loop_ = mk_test_loop();
        let r = mk_reflection(
            true,
            crate::reflector::CritiqueSeverity::Low,
            vec![],
            vec!["ok"],
        );
        let result = loop_
            .handle_plan_adjustment(&"agent1".to_string(), "g1", &r)
            .await;
        assert!(result.is_ok() && result.unwrap().is_none() && !loop_.is_paused());
    }

    /// B-08/14: AskUser routing sets paused and writes reason to blackboard.
    #[tokio::test]
    async fn test_handle_plan_adjustment_ask_user() {
        let loop_ = mk_test_loop();
        let r = mk_reflection(
            false,
            crate::reflector::CritiqueSeverity::Medium,
            vec!["issue"],
            vec!["ask user for clarification"],
        );
        let result = loop_
            .handle_plan_adjustment(&"agent1".to_string(), "g1", &r)
            .await;
        assert!(result.is_ok() && result.unwrap().is_none() && loop_.is_paused());
        assert!(loop_.blackboard.read(BB_ASK_USER_REASON).await.is_some());
    }

    /// B-08/14: Failure counter increments correctly on repeated failures.
    #[tokio::test]
    async fn test_failure_count_incremented() {
        let loop_ = mk_test_loop();
        let agent_id = "agent1".to_string();
        loop_.update_failure_count("ToolFailure", &agent_id).await;
        loop_.update_failure_count("ToolFailure", &agent_id).await;
        let entry = loop_
            .blackboard
            .read(BB_FAILURE_COUNT)
            .await
            .expect("failure count should exist");
        let counts: HashMap<String, usize> =
            serde_json::from_str(&entry.value).expect("valid JSON");
        assert_eq!(counts.get("ToolFailure"), Some(&2));
    }

    /// B-08/14: Stop-Loss triggers when same category fails >=2 times with identical evidence.
    #[tokio::test]
    async fn test_stop_loss_triggered() {
        let loop_ = mk_test_loop();
        let agent_id = "agent1".to_string();
        let evidence = vec!["tool failed".to_string()];
        let counts: HashMap<String, usize> = [("ToolFailure".to_string(), 2)].into_iter().collect();
        loop_
            .blackboard
            .write(
                BB_FAILURE_COUNT,
                &serde_json::to_string(&counts).unwrap(),
                &agent_id,
            )
            .await;
        loop_
            .blackboard
            .write(
                BB_LAST_EVIDENCE,
                &serde_json::to_string(&evidence).unwrap(),
                &agent_id,
            )
            .await;
        assert!(
            loop_
                .check_stop_loss("ToolFailure", &evidence, &agent_id)
                .await
        );
    }

    /// B-08/14: Stop-Loss does not trigger when evidence is new.
    #[tokio::test]
    async fn test_stop_loss_not_triggered_with_new_evidence() {
        let loop_ = mk_test_loop();
        let agent_id = "agent1".to_string();
        let counts: HashMap<String, usize> = [("ToolFailure".to_string(), 2)].into_iter().collect();
        loop_
            .blackboard
            .write(
                BB_FAILURE_COUNT,
                &serde_json::to_string(&counts).unwrap(),
                &agent_id,
            )
            .await;
        loop_
            .blackboard
            .write(
                BB_LAST_EVIDENCE,
                &serde_json::to_string(&vec!["old evidence"]).unwrap(),
                &agent_id,
            )
            .await;
        assert!(
            !loop_
                .check_stop_loss("ToolFailure", &vec!["new evidence".to_string()], &agent_id)
                .await
        );
    }

    /// FIX-B08-001: "handoff" in suggestions maps to StopAndHandoff, not AskUser.
    #[tokio::test]
    async fn test_infer_action_handoff_maps_to_stop_and_handoff() {
        let loop_ = mk_test_loop();
        let r = mk_reflection(
            false,
            crate::reflector::CritiqueSeverity::Medium,
            vec![],
            vec!["handoff to user"],
        );
        let action = loop_.infer_action(&r).await;
        assert_eq!(
            action,
            RecommendedAction::StopAndHandoff,
            "handoff should map to StopAndHandoff"
        );
    }

    /// DEBT-B08-002: UseAlternativeTool selects tool from BB_PLAN_ADJUSTMENT suggested_tools.
    #[tokio::test]
    async fn test_use_alternative_tool_selects_from_blackboard() {
        let loop_ = mk_test_loop();
        let agent_id = "agent1".to_string();
        let plan_adj = serde_json::json!({"action":"UseAlternativeTool","reason":"tool failed","suggested_tools":["alt_tool_1","alt_tool_2"]});
        loop_
            .blackboard
            .write(BB_PLAN_ADJUSTMENT, &plan_adj.to_string(), &agent_id)
            .await;
        let r = mk_reflection(
            false,
            crate::reflector::CritiqueSeverity::Medium,
            vec!["tool failed"],
            vec!["use alternative tool"],
        );
        let result = loop_.handle_plan_adjustment(&agent_id, "g1", &r).await;
        assert!(result.is_ok() && result.unwrap().is_none());
        let entry = loop_.blackboard.read(BB_ALT_TOOL_NAME).await;
        assert!(
            entry.is_some(),
            "alt tool name should be written to blackboard"
        );
        assert_eq!(entry.unwrap().value, "alt_tool_1");
    }

    /// DEBT-B08-003: check_stop_loss uses HashSet for robust evidence comparison.
    #[tokio::test]
    async fn test_check_stop_loss_hashset_comparison() {
        let loop_ = mk_test_loop();
        let agent_id = "agent1".to_string();
        // Same evidence in different order should still trigger stop-loss.
        let counts: HashMap<String, usize> = [("ToolFailure".to_string(), 2)].into_iter().collect();
        loop_
            .blackboard
            .write(
                BB_FAILURE_COUNT,
                &serde_json::to_string(&counts).unwrap(),
                &agent_id,
            )
            .await;
        loop_
            .blackboard
            .write(
                BB_LAST_EVIDENCE,
                &serde_json::to_string(&vec!["b".to_string(), "a".to_string()]).unwrap(),
                &agent_id,
            )
            .await;
        let triggered = loop_
            .check_stop_loss(
                "ToolFailure",
                &vec!["a".to_string(), "b".to_string()],
                &agent_id,
            )
            .await;
        assert!(
            triggered,
            "HashSet comparison should treat different order as identical evidence"
        );
    }

    /// B-09/14: Reflector V1 feature-gate disabled skips V1 routing without error.
    #[tokio::test]
    async fn test_reflect_with_feature_gate_disabled() {
        let _guard = std::sync::Mutex::new(());
        std::env::set_var("HAJIMI_REFLECTOR_V1_ENABLED", "false");
        let loop_ = mk_test_loop();
        let result = TaskResult {
            success: true,
            output: "ok".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let reflection = loop_.reflect("g1", &result).await;
        assert!(
            reflection.is_ok(),
            "reflect should succeed even with V1 disabled"
        );
        let r = reflection.unwrap();
        assert!(
            r.critique.success,
            "legacy path should return success critique"
        );
        std::env::remove_var("HAJIMI_REFLECTOR_V1_ENABLED");
    }

    /// B-09/14: Stop-Loss triggers handoff outcome via handle_plan_adjustment.
    #[tokio::test]
    async fn test_stop_loss_triggers_handoff_outcome() {
        let loop_ = mk_test_loop();
        let agent_id = "agent1".to_string();
        // Seed 2 failures for "ToolFailure" + identical evidence to trigger stop-loss.
        let counts: HashMap<String, usize> = [("ToolFailure".to_string(), 2)].into_iter().collect();
        // Evidence in handle_plan_adjustment is issues + suggestions combined.
        let evidence = vec!["tool failed".to_string(), "check PATH".to_string()];
        loop_
            .blackboard
            .write(
                BB_FAILURE_COUNT,
                &serde_json::to_string(&counts).unwrap(),
                &agent_id,
            )
            .await;
        loop_
            .blackboard
            .write(
                BB_LAST_EVIDENCE,
                &serde_json::to_string(&evidence).unwrap(),
                &agent_id,
            )
            .await;
        let r = mk_reflection(
            false,
            crate::reflector::CritiqueSeverity::High,
            vec!["tool failed"],
            vec!["check PATH"],
        );
        let result = loop_.handle_plan_adjustment(&agent_id, "g1", &r).await;
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert_eq!(
            outcome,
            Some(LoopOutcome::Aborted),
            "stop-loss should produce Aborted outcome"
        );
        assert!(
            loop_.blackboard.read(BB_HANDOFF_SUMMARY).await.is_some(),
            "handoff summary should be written"
        );
    }
}
