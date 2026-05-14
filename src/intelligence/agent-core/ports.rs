//! Agent Core local error types — decouples from chimera-repl interface layer.
//! Phase 2 Day 1: Added WorkerCallback trait for Swarm result propagation.

use chimera_repl::traits::ReplError;
use std::fmt;

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

// SyncMemoryGateway re-exports from memory crate (port-adapter boundary).
pub use memory::sync_gateway::{
    BlackboardSnapshot, GatewayEvent, MemoryTier, SyncGatewayError, SyncMemoryGateway, TierHealth,
};

// ============================================================================
// Phase 2 Day 1: Worker Callback Interface
// ============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Status of a worker execution result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerResultStatus {
    Success,
    Failed,
    Timeout,
    Crashed,
}

/// Lightweight metrics carried with a worker result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetrics {
    /// Execution duration in milliseconds.
    pub execution_time_ms: u64,
    /// Number of retry attempts before this result.
    pub retry_count: u32,
    /// Timestamp when the result was produced.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WorkerMetrics {
    /// Create default metrics with zero retry count.
    pub fn new(execution_time_ms: u64) -> Self {
        Self {
            execution_time_ms,
            retry_count: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create metrics with a specific retry count.
    pub fn with_retries(execution_time_ms: u64, retry_count: u32) -> Self {
        Self {
            execution_time_ms,
            retry_count,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Callback trait for worker result notification.
/// Implementations are invoked asynchronously when a worker finishes,
/// fails, or times out. The trait is object-safe and requires Send+Sync
/// for use across task boundaries.
#[async_trait]
pub trait WorkerCallback: Send + Sync {
    /// Called when a worker task completes successfully.
    async fn on_success(
        &self,
        task_id: &str,
        worker_id: &str,
        output: &str,
        metrics: &WorkerMetrics,
    );
    /// Called when a worker task fails.
    async fn on_failure(
        &self,
        task_id: &str,
        worker_id: &str,
        error: &str,
        metrics: &WorkerMetrics,
    );
    /// Called when a worker task times out.
    async fn on_timeout(&self, task_id: &str, worker_id: &str, elapsed_ms: u64);
}
