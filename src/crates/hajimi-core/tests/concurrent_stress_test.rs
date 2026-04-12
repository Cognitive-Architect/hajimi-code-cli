//! 1000 Concurrent Stream Stress Test - HAJIMI-W03-B05
//! QueryEngine v1.0 - Cluster Load Testing
//! 
//! This test validates the streaming infrastructure under extreme load:
//! - 1000 concurrent streams with tokio::spawn
//! - 10 messages per stream  
//! - Memory stability verification (no OOM)
//! - Error rate < 1% requirement

use hajimi_core::streaming::{ChannelStream, StreamChunk};
use futures::StreamExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CONCURRENT_STREAMS: usize = 1000;
const MESSAGES_PER_STREAM: usize = 10;
const MAX_ERROR_RATE: f64 = 0.01;
const CHANNEL_BUFFER_SIZE: usize = 20;
const MEMORY_TEST_MESSAGES: usize = 10000;
const ISOLATION_TEST_STREAMS: usize = 100;

/// Main stress test: 1000 concurrent streams with JoinSet coordination
/// Validates cluster-wide streaming capacity without OOM
#[tokio::test]
async fn test_1000_concurrent_streams() {
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    let total_messages = Arc::new(AtomicUsize::new(0));
    let mut handles = tokio::task::JoinSet::new();

    // Spawn 1000 concurrent streams
    for stream_id in 0..CONCURRENT_STREAMS {
        let success = success_count.clone();
        let errors = error_count.clone();
        let msgs = total_messages.clone();
        
        handles.spawn(async move {
            let (mut stream, tx) = ChannelStream::new(CHANNEL_BUFFER_SIZE);
            
            // Send 10 messages per stream
            for msg_id in 0..MESSAGES_PER_STREAM {
                let chunk = StreamChunk::Output(format!("s{}-m{}", stream_id, msg_id));
                if tx.send(chunk).await.is_err() {
                    errors.fetch_add(1, Ordering::SeqCst);
                    return;
                }
            }
            tx.send(StreamChunk::Done).await.ok();
            drop(tx);
            
            // Collect all messages from stream
            let mut received = 0;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    StreamChunk::Output(_) => received += 1,
                    StreamChunk::Done => break,
                    StreamChunk::Error(_) => {
                        errors.fetch_add(1, Ordering::SeqCst);
                        return;
                    }
                    _ => {}
                }
            }
            
            // Verify complete message delivery
            if received == MESSAGES_PER_STREAM {
                success.fetch_add(1, Ordering::SeqCst);
                msgs.fetch_add(received, Ordering::SeqCst);
            }
        });
    }
    
    // Wait for all spawned tasks to complete
    while handles.join_next().await.is_some() {}
    
    // Validate test results
    let success = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);
    let error_rate = errors as f64 / CONCURRENT_STREAMS as f64;
    
    assert_eq!(success, CONCURRENT_STREAMS, "All streams should complete successfully");
    assert!(error_rate < MAX_ERROR_RATE, "Error rate {:.2}% exceeds 1% limit", error_rate * 100.0);
    assert_eq!(total_messages.load(Ordering::SeqCst), CONCURRENT_STREAMS * MESSAGES_PER_STREAM);
}

/// Memory stability test: High volume message throughput without OOM
#[tokio::test]
async fn test_memory_stable_under_load() {
    let (stream, tx) = ChannelStream::new(100);
    
    // Spawn producer task
    tokio::spawn(async move {
        for i in 0..MEMORY_TEST_MESSAGES {
            let _ = tx.send(StreamChunk::Output(format!("msg-{}", i))).await;
        }
        tx.send(StreamChunk::Done).await.ok();
    });
    
    // Consume all messages
    let collected: Vec<_> = stream.collect().await;
    assert_eq!(collected.len(), MEMORY_TEST_MESSAGES + 1);
}

/// tokio::spawn isolation test for concurrent stream safety validation
#[tokio::test]
async fn test_tokio_spawn_stream_isolation() {
    let mut handles = vec![];
    
    // Create ISOLATION_TEST_STREAMS concurrent streams
    for i in 0..ISOLATION_TEST_STREAMS {
        handles.push(tokio::spawn(async move {
            let (mut s, t) = ChannelStream::new(5);
            t.send(StreamChunk::Output(format!("data{}", i))).await.ok();
            drop(t);
            assert!(matches!(s.next().await, Some(StreamChunk::Output(_))));
        }));
    }
    
    // Await all spawned tasks
    for h in handles {
        h.await.unwrap();
    }
}
