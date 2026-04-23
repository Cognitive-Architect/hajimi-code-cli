//! Agent Core local error types — decouples from chimera-repl interface layer.

use std::fmt;
use chimera_repl::traits::ReplError;

/// AgentCore-local error type, decoupled from ReplError.
#[derive(Debug, Clone)]
pub enum AgentError {
    PlanNotInitialized,
    SubgoalNotFound(String),
    GovernanceRejected(String),
    Session(String),
    Protocol(String),
    Internal(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::PlanNotInitialized => write!(f, "Plan not initialized"),
            AgentError::SubgoalNotFound(id) => write!(f, "SubGoal {} not found in plan", id),
            AgentError::GovernanceRejected(r) => write!(f, "Governance rejected: {}", r),
            AgentError::Session(s) => write!(f, "Session error: {}", s),
            AgentError::Protocol(s) => write!(f, "Protocol error: {}", s),
            AgentError::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl std::error::Error for AgentError {}

/// Convert from chimera_repl::ReplError to AgentError.
/// Maps Session→Session, Protocol→Protocol, Channel→Internal.
/// Unknown variants fall back to Internal to avoid panics.
impl From<ReplError> for AgentError {
    fn from(e: ReplError) -> Self {
        match e {
            ReplError::Session(s) => AgentError::Session(s),
            ReplError::Protocol(p) => AgentError::Protocol(p.to_string()),
            ReplError::Channel(c) => AgentError::Internal(format!("Channel: {}", c)),
        }
    }
}

/// Convert from AgentError to chimera_repl::ReplError.
impl From<AgentError> for ReplError {
    fn from(e: AgentError) -> Self {
        ReplError::Session(e.to_string())
    }
}

/// Convenience type alias for AgentCore results.
pub type AgentResult<T> = Result<T, AgentError>;
