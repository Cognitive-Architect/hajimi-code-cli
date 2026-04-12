//! ChannelStream integration tests
//! DEBT-W02-B02: Stream backpressure and lifecycle tests

use futures::StreamExt;
use hajimi_core::streaming::{ChannelStream, StreamChunk};

/// Basic streaming: send 3 chunks, receive 3 chunks
#[tokio::test]
async fn test_basic_streaming_three_chunks() {
    let (stream, tx) = ChannelStream::new(10);

    // Send 3 chunks
    tx.send(StreamChunk::Output("chunk-1".to_string()))
        .await
        .expect("send chunk-1");
    tx.send(StreamChunk::Output("chunk-2".to_string()))
        .await
        .expect("send chunk-2");
    tx.send(StreamChunk::Done).await.expect("send Done");

    // Drop sender to close channel
    drop(tx);

    // Collect all chunks
    let chunks: Vec<StreamChunk> = stream.collect().await;

    // Verify 3 chunks received in order
    assert_eq!(chunks.len(), 3);
    match &chunks[0] {
        StreamChunk::Output(s) => assert_eq!(s, "chunk-1"),
        _ => panic!("expected Output chunk-1"),
    }
    match &chunks[1] {
        StreamChunk::Output(s) => assert_eq!(s, "chunk-2"),
        _ => panic!("expected Output chunk-2"),
    }
    match &chunks[2] {
        StreamChunk::Done => {}
        _ => panic!("expected Done"),
    }
}

/// Channel close test: sender drop causes stream to end with None
#[tokio::test]
async fn test_channel_close_graceful() {
    let (mut stream, tx) = ChannelStream::new(5);

    tx.send(StreamChunk::Output("before-close".to_string()))
        .await
        .expect("send before close");

    // Drop sender to close channel
    drop(tx);

    // First poll returns the sent chunk
    let first = stream.next().await;
    assert!(first.is_some(), "should receive chunk before close");

    // Subsequent poll returns None (channel closed)
    let second = stream.next().await;
    assert!(second.is_none(), "should return None after sender dropped");
}

/// Backpressure test: 100 messages without overflow
#[tokio::test]
async fn test_backpressure_100_messages() {
    const MESSAGE_COUNT: usize = 100;
    let (stream, tx) = ChannelStream::new(MESSAGE_COUNT);

    // Spawn producer task to send 100 messages
    let producer = tokio::spawn(async move {
        for i in 0..MESSAGE_COUNT {
            let chunk = StreamChunk::Output(format!("message-{}", i));
            tx.send(chunk).await.expect("send should succeed");
        }
        // Explicitly drop sender to signal end
        drop(tx);
    });

    // Collect all messages
    let chunks: Vec<StreamChunk> = stream.collect().await;

    // Wait for producer to complete
    producer.await.expect("producer task completed");

    // Verify all 100 messages received in order
    assert_eq!(chunks.len(), MESSAGE_COUNT, "should receive exactly 100 chunks");

    // Verify order is preserved
    for (i, chunk) in chunks.iter().enumerate() {
        match chunk {
            StreamChunk::Output(s) => {
                assert_eq!(s, &format!("message-{}", i), "message order should be preserved");
            }
            _ => panic!("expected Output variant at index {}", i),
        }
    }
}

/// Error variant propagation test
#[tokio::test]
async fn test_error_chunk_propagation() {
    let (stream, tx) = ChannelStream::new(5);

    tx.send(StreamChunk::Output("ok".to_string())).await.ok();
    tx.send(StreamChunk::Error("test error".to_string())).await.ok();
    tx.send(StreamChunk::Done).await.ok();
    drop(tx);

    let chunks: Vec<StreamChunk> = stream.collect().await;
    assert_eq!(chunks.len(), 3);

    match &chunks[1] {
        StreamChunk::Error(e) => assert_eq!(e, "test error"),
        _ => panic!("expected Error variant"),
    }
}

/// Heartbeat variant test
#[tokio::test]
async fn test_heartbeat_chunk() {
    let (stream, tx) = ChannelStream::new(5);

    tx.send(StreamChunk::Heartbeat).await.ok();
    tx.send(StreamChunk::Output("data".to_string())).await.ok();
    drop(tx);

    let chunks: Vec<StreamChunk> = stream.collect().await;
    assert_eq!(chunks.len(), 2);
    assert!(matches!(chunks[0], StreamChunk::Heartbeat));
}
