//! Agent Core: Autonomous multi-agent orchestration system.
//! Extends ReplEngineCore with proactive reasoning capabilities.

pub mod orchestrator;
pub mod events;
pub mod planner;
pub mod reflector;
pub mod governance;
pub mod swarm;
pub mod blackboard;
pub mod checkpoint;
pub mod tools;
pub mod agent_loop;
pub mod agent_loop_builder;
pub mod ports;
pub mod degrade;
pub mod llm;

pub use blackboard::Blackboard;
pub use orchestrator::AgentOrchestrator;
pub use planner::{HierarchicalPlanner, Goal, SubGoal, Task, Plan, Priority, PlanStatus, TaskResult};
pub use reflector::{AutonomousReflector, Reflection, Critique, CritiqueSeverity};
pub use governance::{DefaultGovernance, GovernanceRequest, GovernancePolicy, ApprovalLevel, Decision, Vote, PermissionLevel};
pub use swarm::{Supervisor, Worker, TaskAssignment, WorkerResult, SwarmMessage, WorkerStatus, SwarmCoordinator};
pub use checkpoint::{CheckpointManager, Checkpoint, WorkerState};
pub use agent_loop::{AgentLoop, LoopState, LoopOutcome, Observation, TraceEvent};
pub use agent_loop_builder::AgentLoopBuilder;

#[cfg(test)]
mod agent_loop_tests;
pub use ports::{AgentError, AgentResult};
pub use degrade::degrade_warn;

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
