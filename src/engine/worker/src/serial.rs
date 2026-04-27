use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, timeout, Duration};
use tracing::{info, warn};
use engine_tool_system::{ToolRegistry, ToolArgs};
use crate::error::EngineError;
use crate::Executor;
use crate::query::{Query, QueryResult};

pub struct SerialExecutor {
    tool_registry: Option<Arc<tokio::sync::Mutex<ToolRegistry>>>,
    callback: Option<Arc<dyn crate::ExecutionCallback>>,
}

impl SerialExecutor {
    pub fn new() -> Self {
        Self { tool_registry: None, callback: None }
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

        if query.content.is_empty() {
            return Err(EngineError::InvalidParameters("Query content is empty".to_string()));
        }

        let result = timeout(
            Duration::from_millis(query.timeout_ms),
            async {
                if let Some(reg_arc) = registry {
                    let reg = reg_arc.lock().await;
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

impl Default for SerialExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for SerialExecutor {
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
        let mut results = Vec::with_capacity(queries.len());
        for query in queries {
            results.push(Self::run_query(query, self.tool_registry.clone()).await);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serial_executor_fallback() {
        let exec = SerialExecutor::new();
        let query = Query::new("test", "hello");
        let result = exec.execute(query).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_serial_executor_empty_query_fails() {
        let exec = SerialExecutor::new();
        let query = Query::new("test", "");
        let result = exec.execute(query).await;
        assert!(matches!(result, Err(EngineError::InvalidParameters(_))));
    }
}
