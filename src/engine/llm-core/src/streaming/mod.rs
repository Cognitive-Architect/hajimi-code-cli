//! Streaming module for LLM responses

use tokio::sync::mpsc;

/// Stream chunk types for LLM responses / LLM 响应的流式片段类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamChunk {
    /// Output text chunk / 文本输出片段
    Output(String),
    /// Error message / 错误信息
    Error(String),
    /// Stream completed / 流传输完成
    Done,
    /// Start of a tool call event / 工具调用开始事件
    ToolCallStart {
        /// Unique identifier for the tool call / 工具调用唯一标识符
        id: String,
        /// Name of the tool to be called / 待调用的工具名称
        name: String,
    },
    /// Incremental argument chunk for a tool call / 工具调用参数的增量片段
    ToolCallArgumentsDelta {
        /// Unique identifier for the tool call / 工具调用唯一标识符
        id: String,
        /// Argument delta text chunk / 参数增量片段文本
        delta: String,
    },
    /// End of a tool call event / 工具调用结束事件
    ToolCallEnd {
        /// Unique identifier for the tool call / 工具调用唯一标识符
        id: String,
    },
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
    pub use super::ChannelStream;
}
