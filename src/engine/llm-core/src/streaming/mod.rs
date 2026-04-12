//! Streaming module for LLM responses

use tokio::sync::mpsc;

/// Stream chunk types for LLM responses
#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// Output text chunk
    Output(String),
    /// Error message
    Error(String),
    /// Stream completed
    Done,
}

/// Channel-based stream for LLM responses
pub struct ChannelStream {
    receiver: mpsc::Receiver<StreamChunk>,
}

impl ChannelStream {
    /// Create a new channel stream with given capacity
    pub fn new(capacity: usize) -> (Self, mpsc::Sender<StreamChunk>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { receiver: rx }, tx)
    }
    
    /// Receive next chunk
    pub async fn next(&mut self) -> Option<StreamChunk> {
        self.receiver.recv().await
    }
}

pub mod channel_stream {
    pub use super::{ChannelStream, StreamChunk};
}
