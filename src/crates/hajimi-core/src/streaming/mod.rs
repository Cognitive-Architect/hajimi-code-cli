//! StreamingExecutor trait for HAJIMI Core
//! DEBT-W01-003: StreamingExecutor implementation

pub mod backpressure;
pub mod batched;
pub mod channel_stream;
pub mod network_sync;
pub mod sse;
pub mod types;

pub use batched::{BatchConfig, BatchedStream};
pub use channel_stream::ChannelStream;
pub use network_sync::{NetworkSyncStream, SyncConfig, SyncStatus, NetworkError};
pub use types::{
    QueryMetadata, StreamChunk, StreamConfig, StreamingExecutor, StreamingQueryResult,
};
