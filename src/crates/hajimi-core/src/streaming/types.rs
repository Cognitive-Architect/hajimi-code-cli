//! Stream types for HAJIMI streaming executor
//! DEBT-W01-003: Stream interface types

use std::pin::Pin;
use futures::Stream;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::query::Query;

/// Stream chunk variants for SSE/Stream output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamChunk {
    /// Incremental output chunk
    Output(String),
    /// Error message
    Error(String),
    /// Completion marker
    Done,
    /// Keep-alive heartbeat for SSE
    Heartbeat,
}

/// Stream configuration for backpressure control
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Buffer size for backpressure (default: 100 per audit)
    pub buffer_size: usize,
    /// Timeout in milliseconds (default: 30000)
    pub timeout_ms: u64,
    /// SSE heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            timeout_ms: 30_000,
            heartbeat_interval_ms: 5_000,
        }
    }
}

/// Metadata for streaming query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub started_at: u64,
    pub tool_name: String,
}

/// Result of a streaming query execution
pub struct StreamingQueryResult {
    pub result_id: Uuid,
    pub output_stream: Pin<Box<dyn Stream<Item = StreamChunk> + Send>>,
    pub metadata: QueryMetadata,
}

impl StreamingQueryResult {
    pub fn new(
        result_id: Uuid,
        output_stream: Pin<Box<dyn Stream<Item = StreamChunk> + Send>>,
        metadata: QueryMetadata,
    ) -> Self {
        Self {
            result_id,
            output_stream,
            metadata,
        }
    }
}

/// Streaming executor trait for async stream output
/// 
/// Provides backpressure-controlled streaming execution with configurable
/// buffer_size, timeout_ms, and heartbeat intervals for SSE compatibility.
#[allow(async_fn_in_trait)]
pub trait StreamingExecutor: Send + Sync {
    /// Execute query and return streaming result
    async fn execute_stream(
        &self,
        query: Query,
    ) -> Result<StreamingQueryResult, EngineError>;

    /// Get stream configuration for backpressure control
    fn stream_config(&self) -> StreamConfig;
}
