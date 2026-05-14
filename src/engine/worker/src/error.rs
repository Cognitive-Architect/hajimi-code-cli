use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EngineError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Execution timeout after {0}ms")]
    Timeout(u64),

    #[error("Retry exhausted after {attempts} attempts: {source}")]
    RetryExhausted {
        attempts: u32,
        source: Box<EngineError>,
    },

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Query error: {0}")]
    QueryError(String),
}
