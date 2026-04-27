mod error;
mod query;
use error::EngineError;
use query::{Query, QueryResult};

pub mod parallel;
pub mod serial;

pub use parallel::ParallelExecutor;
pub use serial::SerialExecutor;

use async_trait::async_trait;
use engine_tool_system::{ToolRegistry, ToolArgs};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Callback trait for query execution results.
/// Decoupled from agent-core types to keep engine-worker independent.
#[async_trait]
pub trait ExecutionCallback: Send + Sync {
    /// Called when a query executes successfully.
    async fn on_success(&self, query_id: &str, output: &str, execution_time_ms: u64);
    /// Called when a query fails.
    async fn on_failure(&self, query_id: &str, error: &str);
}

/// Executor trait for query execution.
#[allow(async_fn_in_trait)]
pub trait Executor: Send + Sync {
    async fn execute(&self, query: Query) -> Result<QueryResult, EngineError>;
    async fn execute_batch(&self, queries: Vec<Query>) -> Vec<Result<QueryResult, EngineError>>;
}

/// Extension trait for executing queries with a ToolRegistry.
pub trait ToolExecutor {
    /// Execute a query by looking up and running a real tool from the registry.
    async fn execute_with_registry(
        &self,
        query: &Query,
        registry: &ToolRegistry,
    ) -> Result<QueryResult, EngineError>;
}
