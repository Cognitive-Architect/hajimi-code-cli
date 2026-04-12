use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub tool_name: String,
    pub parameters: Value,
    pub timeout_ms: u64,
}

impl Query {
    pub fn new(tool_name: impl Into<String>, parameters: Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            parameters,
            timeout_ms: 30_000,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl QueryResult {
    pub fn success(output: impl Into<String>, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            output: Some(output.into()),
            error: None,
            execution_time_ms,
        }
    }

    pub fn error(error: impl Into<String>, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error.into()),
            execution_time_ms,
        }
    }
}
