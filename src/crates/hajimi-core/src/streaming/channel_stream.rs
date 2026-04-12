//! Channel-based Stream implementation for HAJIMI Core
//! DEBT-W02-B02: mpsc channel Stream adapter

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::streaming::types::StreamChunk;

/// Stream adapter wrapping mpsc channel receiver
/// Provides backpressure-controlled streaming output
pub struct ChannelStream {
    receiver: Receiver<StreamChunk>,
}

impl ChannelStream {
    /// Create new ChannelStream with specified buffer size
    /// Returns (stream, sender) pair for producer/consumer pattern
    pub fn new(buffer_size: usize) -> (Self, Sender<StreamChunk>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (Self { receiver: rx }, tx)
    }
}

impl Stream for ChannelStream {
    type Item = StreamChunk;

    /// Poll next chunk from channel
    /// Returns None when sender is dropped (channel closed)
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_basic_streaming() {
        let (stream, tx) = ChannelStream::new(10);
        tx.send(StreamChunk::Output("hello".to_string())).await.ok();
        tx.send(StreamChunk::Output("world".to_string())).await.ok();
        tx.send(StreamChunk::Done).await.ok();
        drop(tx);

        let chunks: Vec<_> = stream.collect().await;
        assert_eq!(chunks.len(), 3);
    }

    #[tokio::test]
    async fn test_channel_close() {
        let (mut stream, tx) = ChannelStream::new(5);
        tx.send(StreamChunk::Output("test".to_string())).await.ok();
        drop(tx);

        assert!(stream.next().await.is_some());
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_backpressure_100() {
        let (stream, tx) = ChannelStream::new(100);
        for i in 0..100 {
            tx.send(StreamChunk::Output(format!("msg-{}", i))).await.ok();
        }
        drop(tx);

        let chunks: Vec<_> = stream.collect().await;
        assert_eq!(chunks.len(), 100);
    }
}
