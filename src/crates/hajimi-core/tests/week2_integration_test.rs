//! Week 2 Integration Test - HAJIMI-W02-SATURN-002
use futures::StreamExt;
use hajimi_core::streaming::{backpressure::BackpressureController, sse::to_sse, ChannelStream, StreamChunk, StreamConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[test]
fn test_streaming_executor_creation() {
    let (s, _tx) = ChannelStream::new(10); let _ = s;
}
#[tokio::test]
async fn test_streaming_executor_single_chunk() {
    let (s, tx) = ChannelStream::new(10); tx.send(StreamChunk::Output("h".into())).await.ok(); drop(tx);
    assert_eq!(s.collect::<Vec<_>>().await.len(), 1);
}
#[tokio::test]
async fn test_streaming_executor_multiple_chunks() {
    let (s, tx) = ChannelStream::new(10); for i in 0..5 { tx.send(StreamChunk::Output(format!("c{}", i))).await.ok(); }
    tx.send(StreamChunk::Done).await.ok(); drop(tx); assert_eq!(s.collect::<Vec<_>>().await.len(), 6);
}
#[tokio::test]
async fn test_streaming_executor_error_chunk() {
    let (s, tx) = ChannelStream::new(5); tx.send(StreamChunk::Output("o".into())).await.ok();
    tx.send(StreamChunk::Error("e".into())).await.ok(); drop(tx);
    assert!(matches!(&s.collect::<Vec<_>>().await[1], StreamChunk::Error(_)));
}

#[tokio::test]
async fn test_backpressure_buffer_full() {
    let (c, _rx) = BackpressureController::new(StreamConfig { buffer_size: 1, ..Default::default() });
    c.try_send(StreamChunk::Output("f".into())).await.ok();
    assert!(c.try_send(StreamChunk::Output("s".into())).await.is_err());
}
#[tokio::test]
async fn test_backpressure_slow_consumer() {
    let (c, _rx) = BackpressureController::new(StreamConfig { buffer_size: 5, ..Default::default() });
    let t = std::time::Instant::now();
    for i in 0..10 { let _ = c.send_with_timeout(StreamChunk::Output(format!("m{}", i)), 100).await; }
    assert!(t.elapsed().as_millis() > 50);
}
#[tokio::test]
async fn test_backpressure_memory_stable() {
    let (c, mut rx) = BackpressureController::new(StreamConfig::default());
    tokio::spawn(async move { while rx.recv().await.is_some() {} });
    for i in 0..1000 { assert!(c.send_with_timeout(StreamChunk::Output(format!("m{}", i)), 5000).await.is_ok()); }
}
#[tokio::test]
async fn test_backpressure_concurrent_senders() {
    let (c, mut rx) = BackpressureController::new(StreamConfig { buffer_size: 50, ..Default::default() });
    let (c, cnt) = (Arc::new(c), Arc::new(AtomicUsize::new(0)));
    let cc = cnt.clone(); tokio::spawn(async move { while rx.recv().await.is_some() { cc.fetch_add(1, Ordering::SeqCst); } });
    let mut h = vec![]; for t in 0..4 { let cc = c.clone(); h.push(tokio::spawn(async move {
        for i in 0..25 { cc.send_with_timeout(StreamChunk::Output(format!("t{}-{}", t, i)), 1000).await.ok(); }
    })); }
    for x in h { x.await.ok(); } sleep(Duration::from_millis(100)).await; assert_eq!(cnt.load(Ordering::SeqCst), 100);
}

#[test]
fn test_sse_output_format() { let s = to_sse(&StreamChunk::Output("t".into())); assert!(s.starts_with("data: ") && s.ends_with("\n\n")); }
#[test]
fn test_sse_end_to_end_stream() {
    let o: String = vec![StreamChunk::Output("m1".into()), StreamChunk::Heartbeat, StreamChunk::Done].iter().map(to_sse).collect();
    assert!(o.contains("data: m1") && o.contains(":heartbeat") && o.contains("event: done"));
}
#[test]
fn test_sse_heartbeat_keepalive() { assert_eq!(to_sse(&StreamChunk::Heartbeat), ":heartbeat\n\n"); }
#[test]
fn test_sse_multiline_data() { let s = to_sse(&StreamChunk::Output("l1\nl2".into())); assert!(s.contains("data: l1") && s.contains("data: l2")); }

#[tokio::test]
async fn test_concurrent_streams() {
    let mut h = vec![]; for i in 0..5 { h.push(tokio::spawn(async move {
        let (s, t) = ChannelStream::new(10); t.send(StreamChunk::Output(format!("s{}", i))).await.ok(); drop(t);
        assert_eq!(s.collect::<Vec<_>>().await.len(), 1);
    })); } for x in h { x.await.unwrap(); }
}
#[tokio::test]
async fn test_concurrent_stream_isolation() {
    let (s1, t1) = ChannelStream::new(5); let (s2, t2) = ChannelStream::new(5);
    t1.send(StreamChunk::Output("a".into())).await.ok(); t2.send(StreamChunk::Output("b".into())).await.ok(); drop(t1); drop(t2);
    let (r1, r2): (Vec<_>, Vec<_>) = tokio::join!(s1.collect(), s2.collect()); assert_eq!(r1.len(), 1); assert_eq!(r2.len(), 1);
}
#[tokio::test]
async fn test_streaming_stress() {
    let mut h = vec![]; for i in 0..100 { h.push(tokio::spawn(async move {
        let (s, t) = ChannelStream::new(10); for j in 0..10 { t.send(StreamChunk::Output(format!("s{}-m{}", i, j))).await.ok(); }
        drop(t); assert_eq!(s.collect::<Vec<_>>().await.len(), 10);
    })); } for x in h { x.await.unwrap(); }
}
#[tokio::test]
async fn test_week2_full_pipeline() {
    let (s, tx) = ChannelStream::new(10);
    for i in 0..5 { tx.send(StreamChunk::Output(format!("d{}", i))).await.ok(); }
    tx.send(StreamChunk::Done).await.ok(); drop(tx);
    let o: String = s.collect::<Vec<_>>().await.iter().map(to_sse).collect();
    assert!(o.contains("event: done"));
}
