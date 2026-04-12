//! BatchedStream - DEBT-W03-B04: Batch flush optimization
//! Aggregates StreamChunk: batch_size=10, flush_interval=50ms

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use futures::Stream;
use tokio::time::Interval;
use crate::streaming::types::StreamChunk;
use crate::streaming::channel_stream::ChannelStream;

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub compression: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self { batch_size: 10, flush_interval_ms: 50, compression: false }
    }
}

pub struct BatchedStream {
    inner: ChannelStream,
    buffer: Vec<StreamChunk>,
    config: BatchConfig,
    flush_timer: Interval,
}

impl BatchedStream {
    pub fn new(inner: ChannelStream, config: BatchConfig) -> Self {
        let flush_timer = tokio::time::interval(Duration::from_millis(config.flush_interval_ms));
        Self { inner, buffer: Vec::with_capacity(config.batch_size), config, flush_timer }
    }
    pub fn with_defaults(inner: ChannelStream) -> Self { Self::new(inner, BatchConfig::default()) }

    fn flush_buffer(&mut self) -> Option<StreamChunk> {
        if self.buffer.is_empty() { return None; }
        let mut output = String::new();
        let (mut has_error, mut is_done) = (false, false);
        for chunk in self.buffer.drain(..) {
            match chunk {
                StreamChunk::Output(s) => output.push_str(&s),
                StreamChunk::Error(_) => has_error = true,
                StreamChunk::Done => is_done = true,
                StreamChunk::Heartbeat => {}
            }
        }
        if has_error { Some(StreamChunk::Error("Batch contained error".into())) }
        else if is_done { Some(StreamChunk::Done) }
        else if !output.is_empty() { Some(StreamChunk::Output(output)) }
        else { None }
    }
}

impl Stream for BatchedStream {
    type Item = StreamChunk;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.flush_timer.poll_tick(cx).is_ready() && !self.buffer.is_empty() {
            return Poll::Ready(self.flush_buffer());
        }
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(chunk)) => {
                self.buffer.push(chunk);
                if self.buffer.len() >= self.config.batch_size { return Poll::Ready(self.flush_buffer()); }
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(self.flush_buffer()),
            Poll::Pending => { if !self.buffer.is_empty() { cx.waker().wake_by_ref(); } Poll::Pending }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_batch_flush_on_size() -> Result<(), Box<dyn std::error::Error>> {
        let (inner, tx) = ChannelStream::new(20);
        let mut batched = BatchedStream::new(inner, BatchConfig { batch_size: 3, flush_interval_ms: 1000, compression: false });
        tx.send(StreamChunk::Output("a".into())).await.ok();
        tx.send(StreamChunk::Output("b".into())).await.ok();
        tx.send(StreamChunk::Output("c".into())).await.ok();
        drop(tx);
        let chunk = batched.next().await.ok_or("Stream ended unexpectedly")?;
        assert!(matches!(chunk, StreamChunk::Output(ref s) if s == "abc"));
        Ok(())
    }

    #[tokio::test]
    async fn test_batch_flush_on_stream_end() -> Result<(), Box<dyn std::error::Error>> {
        let (inner, tx) = ChannelStream::new(10);
        let mut batched = BatchedStream::with_defaults(inner);
        tx.send(StreamChunk::Output("hello".into())).await.ok();
        drop(tx);
        let chunk = batched.next().await.ok_or("Stream ended unexpectedly")?;
        assert!(matches!(chunk, StreamChunk::Output(ref s) if s == "hello"));
        Ok(())
    }
}
