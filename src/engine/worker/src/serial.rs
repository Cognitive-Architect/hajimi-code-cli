use std::time::Instant;
use tokio::time::{sleep, timeout, Duration};
use crate::error::EngineError;
use crate::Executor;
use crate::query::{Query, QueryResult};

pub struct SerialExecutor;

impl SerialExecutor {
    pub fn new() -> Self {
        Self
    }

    async fn run_query(query: Query) -> Result<QueryResult, EngineError> {
        let start = Instant::now();
        
        // 模拟工具执行时间（50ms）
        let result = timeout(
            Duration::from_millis(query.timeout_ms),
            async {
                sleep(Duration::from_millis(50)).await;
                let elapsed = start.elapsed().as_millis() as u64;
                Ok(QueryResult::success("executed", elapsed.max(1)))
            }
        ).await;

        match result {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(EngineError::Timeout(query.timeout_ms)),
        }
    }
}

impl Default for SerialExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for SerialExecutor {
    async fn execute(&self, query: Query) -> Result<QueryResult, EngineError> {
        Self::run_query(query).await
    }

    async fn execute_batch(&self, queries: Vec<Query>) -> Vec<Result<QueryResult, EngineError>> {
        let mut results = Vec::with_capacity(queries.len());
        for query in queries {
            results.push(Self::run_query(query).await);
        }
        results
    }
}
