use serde::{Deserialize, Serialize};

/// Query representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub id: String,
    pub content: String,
    pub context: Option<QueryContext>,
    pub timeout_ms: u64,
}

/// Query execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query_id: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    /// Whether the query execution succeeded.
    pub success: bool,
}

/// Query context for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl Query {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            context: None,
            timeout_ms: 30000, // default 30s
        }
    }

    pub fn with_context(mut self, context: QueryContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

impl QueryResult {
    pub fn new(query_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            query_id: query_id.into(),
            content: content.into(),
            metadata: None,
            execution_time_ms: 0,
            success: true,
        }
    }

    /// Create a successful result (defaults success=true).
    pub fn ok(content: impl Into<String>, elapsed_ms: u64) -> Self {
        Self {
            query_id: String::new(),
            content: content.into(),
            metadata: None,
            execution_time_ms: elapsed_ms,
            success: true,
        }
    }

    pub fn with_query_id(mut self, query_id: impl Into<String>) -> Self {
        self.query_id = query_id.into();
        self
    }

    /// Mark the result as failed.
    pub fn with_failure(mut self) -> Self {
        self.success = false;
        self
    }
}
