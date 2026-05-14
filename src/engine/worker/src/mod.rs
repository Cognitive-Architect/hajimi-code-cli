mod error;
mod query;
use error::EngineError;
use query::{Query, QueryResult};

pub mod parallel;
pub mod serial;

pub use parallel::ParallelExecutor;
pub use serial::SerialExecutor;

use async_trait::async_trait;
use engine_tool_system::ToolRegistry;

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
pub trait Executor: Send + Sync {
    fn execute(
        &self,
        query: Query,
    ) -> impl std::future::Future<Output = Result<QueryResult, EngineError>> + Send;
    fn execute_batch(
        &self,
        queries: Vec<Query>,
    ) -> impl std::future::Future<Output = Vec<Result<QueryResult, EngineError>>> + Send;
}

/// Extension trait for executing queries with a ToolRegistry.
pub trait ToolExecutor {
    /// Execute a query by looking up and running a real tool from the registry.
    fn execute_with_registry(
        &self,
        query: &Query,
        registry: &ToolRegistry,
    ) -> impl std::future::Future<Output = Result<QueryResult, EngineError>> + Send;
}
