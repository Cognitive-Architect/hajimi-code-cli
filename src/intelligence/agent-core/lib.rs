//! Agent Core: Autonomous multi-agent orchestration system.
//! Extends ReplEngineCore with proactive reasoning capabilities.

pub mod act_dto;
pub mod act_executor;
pub mod agent_loop;
pub mod agent_loop_builder;
pub mod blackboard;
pub mod checkpoint;
pub mod context_window_manager;
pub mod degrade;
pub mod edit_applier;
pub mod event_tracing;
pub mod events;
pub mod governance;
pub mod llm;
pub mod loop_state_machine;
pub mod memory_bootstrapper;
pub mod memory_retriever;
pub mod multi_worker_aggregator;
pub mod orchestrator;
pub mod plan_optimizer;
pub mod planner;
pub mod planner_dto;
pub mod ports;
pub mod prompts;
pub mod reflection_persistence;
pub mod reflector;
pub mod reflector_dto;
pub mod resource_monitor;
pub mod swarm;
pub mod swarm_delegate;
pub mod tool_manifest;
pub mod tools;
pub mod worker_lifecycle_manager;
pub mod workflow_orchestrator;
pub use edit_applier::{edit_summary, AppliedEdit, EditApplier, EditHunk, EditState, ProposedEdit};
pub use workflow_orchestrator::{WorkflowOrchestrator, WorkflowOutcome};

// Phase 4 Day 2: AST context re-exports from engine layer (strict layering)
pub use agent_loop::{
    AgentLoop, LoopOutcome, LoopState, Observation, OperationSummary, TraceEvent,
};
pub use agent_loop_builder::AgentLoopBuilder;
pub use blackboard::Blackboard;
pub use checkpoint::{Checkpoint, CheckpointManager, WorkerState};
pub use engine_tool_system::ast_provider::{AstSymbolIndex, CodeSymbol};
pub use engine_tool_system::lsp_integration::{
    ASTContextProvider, LspContextProvider, SymbolContext,
};
pub use governance::{
    ApprovalLevel, Decision, DefaultGovernance, GovernancePolicy, GovernanceRequest,
    PermissionLevel, Vote,
};
pub use memory_bootstrapper::{BootstrapResult, MemoryBootstrapper};
pub use orchestrator::AgentOrchestrator;
pub use planner::{
    Goal, HierarchicalPlanner, Plan, PlanStatus, Priority, SubGoal, Task, TaskResult,
};
pub use reflector::{AutonomousReflector, Critique, CritiqueSeverity, Reflection};
pub use swarm::{
    Supervisor, SupervisorMetrics, SwarmCoordinator, SwarmMessage, TaskAssignment, Worker,
    WorkerResult, WorkerStatus,
};

#[cfg(test)]
mod agent_loop_tests;
#[cfg(test)]
mod prompt_golden_tests;
pub use degrade::degrade_warn;
pub use ports::{AgentError, AgentResult};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// AgentResult/AgentError exported from ports.rs; per-module imports for Repl types
pub use memory::memory_gateway::MemoryGateway;

/// Unique identifier for an agent instance.
pub type AgentId = String;

/// Core trait for autonomous agent implementations.
/// Extends ReplEngineCore with goal-driven proactive capabilities.
/// Requires Debug for logging and state inspection.
#[async_trait]
pub trait Agent: Send + Sync + std::fmt::Debug {
    /// Return the agent's unique identifier.
    fn id(&self) -> &AgentId;
    /// Return the agent's assigned role.
    fn role(&self) -> AgentRole;
    /// Return the agent's capability set.
    fn capabilities(&self) -> &AgentCapability;
    /// Execute one autonomous reasoning cycle.
    async fn tick(&mut self) -> AgentResult<AgentOutcome>;
    /// Check if agent has completed its current goal.
    fn is_goal_achieved(&self) -> bool;
}

/// Outcome of a single agent execution cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentOutcome {
    /// Action completed successfully, continue to next cycle.
    Continue,
    /// Goal achieved, agent can be deactivated.
    Completed,
    /// Blocked and requires governance escalation.
    Escalated(String),
    /// Critical error occurred (error message only, ReplError not Clone).
    Failed(String),
}

/// Role classification for agent specialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// High-level coordinator that delegates to specialists.
    Supervisor,
    /// Code generation and modification specialist.
    Coder,
    /// Research and information gathering specialist.
    Researcher,
    /// Critical evaluation and review specialist.
    Critic,
    /// Tool execution and environment interaction specialist.
    Executor,
    /// General purpose agent with balanced capabilities.
    Generalist,
}

/// Capability specification for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Human-readable description of what this agent can do.
    pub description: String,
    /// Tools this agent is authorized to use (tool names).
    pub authorized_tools: Vec<String>,
    /// Memory layers this agent can access.
    pub memory_access: Vec<MemoryLayerAccess>,
    /// Maximum number of sub-agents this agent can spawn.
    pub max_subagents: usize,
    /// Whether this agent can approve actions from other agents.
    pub can_govern: bool,
    /// Preferred LLM provider ID for this agent (e.g. "openai", "anthropic").
    /// None falls back to system default.
    pub preferred_provider: Option<String>,
}

impl AgentCapability {
    /// Create minimal capabilities for a new agent.
    pub fn minimal() -> Self {
        Self {
            description: String::new(),
            authorized_tools: Vec::new(),
            memory_access: vec![MemoryLayerAccess::Session],
            max_subagents: 0,
            can_govern: false,
            preferred_provider: None,
        }
    }
}

/// Memory layer access levels for capability-based security.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLayerAccess {
    /// Volatile session memory only.
    Session,
    /// Auto-extracted local file memory.
    Auto,
    /// Dream layer with embeddings.
    Dream,
    /// Knowledge graph for structured relationships.
    Graph,
    /// Encrypted cloud sync layer.
    Cloud,
}

/// Configuration for agent initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier.
    pub agent_id: AgentId,
    /// Assigned role.
    pub role: AgentRole,
    /// Capability specification.
    pub capability: AgentCapability,
    /// Initial goal or objective.
    pub initial_goal: Option<String>,
    /// Custom metadata for extensibility.
    pub metadata: HashMap<String, String>,
}

impl AgentConfig {
    /// Create config for a supervisor agent.
    pub fn supervisor(id: &str) -> Self {
        Self {
            agent_id: id.to_string(),
            role: AgentRole::Supervisor,
            capability: AgentCapability {
                description: "Coordinates specialist agents".to_string(),
                authorized_tools: vec!["delegate".to_string(), "approve".to_string()],
                memory_access: vec![MemoryLayerAccess::Session, MemoryLayerAccess::Graph],
                max_subagents: 4,
                can_govern: true,
                preferred_provider: None,
            },
            initial_goal: None,
            metadata: HashMap::new(),
        }
    }
}

/// Shared state container for agent coordination.
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Current execution cycle count.
    pub cycle_count: u64,
    /// Shared blackboard for inter-agent communication (now integrated with Blackboard struct).
    pub blackboard: Arc<Blackboard>,
}

impl AgentContext {
    /// Create new context.
    pub fn new() -> Self {
        Self {
            cycle_count: 0,
            blackboard: Arc::new(Blackboard::new()),
        }
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new()
    }
}
