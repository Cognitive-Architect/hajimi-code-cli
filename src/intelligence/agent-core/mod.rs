//! Agent Core: Autonomous multi-agent orchestration for HAJIMI.
//!
//! Architecture: lib.rs(traits) → orchestrator.rs(coordination) → agent_loop.rs(7-step cycle)
//! Components: planner, reflector, governance(pluggable), swarm, blackboard, checkpoint
//!
//! # DEBT Summary (Day 10)
//! Total: 4 active (target ≤ 8) - DEBT-RETRIEVE-PHASE5, DEBT-WORKER-TOOL-EXECUTION, DEBT-MEMORY-SYNC, DEBT-LEAK-TEST-PHASE5

pub use lib::*;
pub use orchestrator::AgentOrchestrator;
pub mod events;
pub mod planner;
pub use events::AgentEventProcessor;
pub use planner::{Goal, SubGoal, Task, Plan, Planner, HierarchicalPlanner, Priority, PlanStatus};
pub mod reflector;
pub use reflector::{Reflector, AutonomousReflector, Reflection, Critique, CritiqueSeverity, ReflectionLlmClient};
pub mod governance;
pub use governance::{AgentGovernance, DefaultGovernance, ApprovalLevel, Decision, Vote, GovernanceRequest, GovernancePolicy, ConflictStrategy};
pub mod swarm;
pub use swarm::{SwarmCoordinator, Supervisor, Worker, TaskAssignment, WorkerResult, SwarmMessage, WorkerStatus};
pub mod blackboard;
pub use blackboard::{Blackboard, BlackboardEntry, BlackboardEvent, Subscription};
pub mod checkpoint;
pub use checkpoint::{Checkpoint, CheckpointManager, WorkerState};
pub mod agent_loop;
pub use agent_loop::{AgentLoop, LoopState, LoopOutcome, Observation};
pub mod tools;
pub use tools::{PlanningTool, ReflectionTool};
#[cfg(test)]
mod tests;

/// Plugin extension point for user-defined governance
/// Example: impl GovernancePolicy for CustomPolicy { ... }
pub mod plugin_examples {
    pub use super::governance::GovernancePolicy;
}
