use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout, Duration};
use tracing::{info, warn};
use engine_tool_system::{ToolRegistry, ToolArgs};
use crate::error::EngineError;
use crate::Executor;
use crate::query::{Query, QueryResult};

pub struct ParallelExecutor {
    max_concurrency: usize,
    tool_registry: Option<Arc<tokio::sync::Mutex<ToolRegistry>>>,
    callback: Option<Arc<dyn crate::ExecutionCallback>>,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self { max_concurrency: 10, tool_registry: None, callback: None }
    }

    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }

    /// Attach a tool registry for real tool execution.
    pub fn with_tool_registry(mut self, registry: Arc<tokio::sync::Mutex<ToolRegistry>>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Attach an execution callback for result notification.
    pub fn with_callback(mut self, callback: Arc<dyn crate::ExecutionCallback>) -> Self {
        self.callback = Some(callback);
        self
    }

    async fn run_query(query: Query, registry: Option<Arc<tokio::sync::Mutex<ToolRegistry>>>) -> Result<QueryResult, EngineError> {
        let start = Instant::now();

        // Validate query content.
        if query.content.is_empty() {
            return Err(EngineError::InvalidParameters("Query content is empty".to_string()));
        }

        let result = timeout(
            Duration::from_millis(query.timeout_ms),
            async {
                let elapsed = start.elapsed().as_millis() as u64;

                // If a tool registry is attached, attempt real tool execution.
                if let Some(reg_arc) = registry {
                    let reg = reg_arc.lock().await;
                    // Use the query id as the tool name to look up.
                    if let Some(tool) = reg.get(&query.id) {
                        let args = ToolArgs::from(serde_json::json!({
                            "content": query.content,
                        }));
                        match tool.execute(args).await {
                            Ok(out) => {
                                let total_elapsed = start.elapsed().as_millis() as u64;
                                info!(query_id = %query.id, tool = %tool.name(), "Tool executed successfully");
                                Ok(QueryResult {
                                    query_id: query.id.clone(),
                                    content: out.stdout,
                                    metadata: None,
                                    execution_time_ms: total_elapsed.max(1),
                                    success: out.exit_code.unwrap_or(-1) == 0,
                                })
                            }
                            Err(e) => {
                                warn!(query_id = %query.id, error = %e.message, "Tool execution failed");
                                Err(EngineError::ExecutionFailed(e.message))
                            }
                        }
                    } else {
                        warn!(query_id = %query.id, "Tool not found in registry");
                        Err(EngineError::ToolNotFound(query.id.clone()))
                    }
                } else {
                    // Fallback simulated execution.
                    sleep(Duration::from_millis(50)).await;
                    let elapsed = start.elapsed().as_millis() as u64;
                    Ok(QueryResult::ok("executed", elapsed.max(1)).with_query_id(&query.id))
                }
            }
        ).await;

        match result {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(EngineError::Timeout(query.timeout_ms)),
        }
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for ParallelExecutor {
    async fn execute(&self, query: Query) -> Result<QueryResult, EngineError> {
        let query_id = query.id.clone();
        let result = Self::run_query(query, self.tool_registry.clone()).await;
        // Notify callback if registered.
        if let Some(ref cb) = self.callback {
            match &result {
                Ok(ref r) if r.success => cb.on_success(&r.query_id, &r.content, r.execution_time_ms).await,
                Ok(ref r) => cb.on_failure(&r.query_id, &r.content).await,
                Err(ref e) => cb.on_failure(&query_id, &e.to_string()).await,
            }
        }
        result
    }

    async fn execute_batch(&self, queries: Vec<Query>) -> Vec<Result<QueryResult, EngineError>> {
        let sem = Arc::new(Semaphore::new(self.max_concurrency));
        let mut join_set = JoinSet::new();
        let registry = self.tool_registry.clone();

        for (idx, query) in queries.into_iter().enumerate() {
            let permit = match sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    join_set.spawn(async move {
                        (idx, Err(EngineError::ExecutionFailed("Semaphore closed".to_string())))
                    });
                    continue;
                }
            };
            let reg = registry.clone();
            join_set.spawn(async move {
                let _p = permit;
                let res = Self::run_query(query, reg).await;
                (idx, res)
            });
        }

        let mut results: Vec<(usize, _)> = Vec::new();
        while let Some(res) = join_set.join_next().await {
            if let Ok((idx, r)) = res {
                results.push((idx, r));
            }
        }

        results.sort_by_key(|(i, _)| *i);
        results.into_iter().map(|(_, r)| r).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_executor_fallback() {
        let exec = ParallelExecutor::new();
        let query = Query::new("test", "hello");
        let result = exec.execute(query).await.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "executed");
    }

    #[tokio::test]
    async fn test_parallel_executor_empty_query_fails() {
        let exec = ParallelExecutor::new();
        let query = Query::new("test", "");
        let result = exec.execute(query).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::InvalidParameters(_) => {}
            other => panic!("Expected InvalidParameters, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_parallel_executor_timeout() {
        let exec = ParallelExecutor::new();
        let query = Query::new("test", "hello").with_timeout(1);
        let result = exec.execute(query).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::Timeout(_) => {}
            other => panic!("Expected Timeout, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_parallel_executor_batch() {
        let exec = ParallelExecutor::new().with_max_concurrency(2);
        let queries = vec![
            Query::new("q1", "hello"),
            Query::new("q2", "world"),
            Query::new("q3", "test"),
        ];
        let results = exec.execute_batch(queries).await;
        assert_eq!(results.len(), 3);
        for r in results {
            assert!(r.is_ok());
            assert!(r.unwrap().success);
        }
    }
}
