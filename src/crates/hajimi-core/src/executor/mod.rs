use crate::error::EngineError;
use crate::query::{Query, QueryResult};

pub mod parallel;
pub mod serial;

pub use parallel::ParallelExecutor;
pub use serial::SerialExecutor;

#[allow(async_fn_in_trait)]
pub trait Executor: Send + Sync {
    async fn execute(&self, query: Query) -> Result<QueryResult, EngineError>;
    async fn execute_batch(&self, queries: Vec<Query>) -> Vec<Result<QueryResult, EngineError>>;
}
