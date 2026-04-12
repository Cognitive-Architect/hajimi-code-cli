//! Backpressure controller tests
//! DEBT-W02-B03: Bounded channel + Semaphore双重backpressure测试

use hajimi_core::streaming::backpressure::BackpressureController;
use hajimi_core::streaming::types::{StreamChunk, StreamConfig};

#[tokio::test]
async fn test_backpressure_controller_creation() {
    let config = StreamConfig::default();
    let (controller, _rx) = BackpressureController::new(config);
    assert_eq!(controller.capacity(), 100);
}

#[tokio::test]
async fn test_try_send_success() {
    let config = StreamConfig::default();
    let (controller, mut rx) = BackpressureController::new(config);
    
    let result = controller.try_send(StreamChunk::Output("test".to_string())).await;
    assert!(result.is_ok());
    
    let chunk = rx.recv().await;
    assert!(matches!(chunk, Some(StreamChunk::Output(_))));
}

#[tokio::test]
async fn test_try_send_backpressure() {
    let config = StreamConfig { buffer_size: 1, timeout_ms: 1000, heartbeat_interval_ms: 100 };
    let (controller, _rx) = BackpressureController::new(config);
    
    // Fill the buffer
    let _ = controller.try_send(StreamChunk::Output("first".to_string())).await;
    
    // Second send should fail due to backpressure
    let result = controller.try_send(StreamChunk::Output("second".to_string())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_send_with_timeout_success() {
    let config = StreamConfig::default();
    let (controller, mut rx) = BackpressureController::new(config);
    
    let result = controller.send_with_timeout(StreamChunk::Output("test".to_string()), 1000).await;
    assert!(result.is_ok());
    
    let chunk = rx.recv().await;
    assert!(matches!(chunk, Some(StreamChunk::Output(_))));
}

#[tokio::test]
async fn test_send_with_timeout_timeout() {
    let config = StreamConfig { buffer_size: 1, timeout_ms: 5000, heartbeat_interval_ms: 100 };
    let (controller, _rx) = BackpressureController::new(config);
    
    // Fill the buffer
    let _ = controller.try_send(StreamChunk::Output("first".to_string())).await;
    
    // This should timeout
    let result = controller.send_with_timeout(StreamChunk::Output("second".to_string()), 50).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_backpressure_stress() {
    let config = StreamConfig { buffer_size: 100, timeout_ms: 5000, heartbeat_interval_ms: 100 };
    let (controller, mut rx) = BackpressureController::new(config);
    
    // Spawn consumer task
    let consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Some(_) = rx.recv().await {
            count += 1;
            if count >= 1000 {
                break;
            }
        }
        count
    });
    
    // Send 1000 messages with backpressure
    for i in 0..1000 {
        let result = controller.send_with_timeout(
            StreamChunk::Output(format!("msg-{}", i)), 
            5000
        ).await;
        assert!(result.is_ok(), "Failed to send message {}: {:?}", i, result);
    }
    
    // Wait for consumer to finish
    let count = consumer.await.unwrap();
    assert_eq!(count, 1000, "Expected 1000 messages, got {}", count);
}

#[tokio::test]
async fn test_bounded_channel_capacity() {
    let config = StreamConfig { buffer_size: 50, timeout_ms: 1000, heartbeat_interval_ms: 100 };
    let (controller, _rx) = BackpressureController::new(config);
    
    assert_eq!(controller.capacity(), 50);
}

#[tokio::test]
async fn test_different_chunk_types() {
    let config = StreamConfig::default();
    let (controller, mut rx) = BackpressureController::new(config);
    
    controller.try_send(StreamChunk::Output("data".to_string())).await.ok();
    controller.try_send(StreamChunk::Error("error".to_string())).await.ok();
    controller.try_send(StreamChunk::Done).await.ok();
    controller.try_send(StreamChunk::Heartbeat).await.ok();
    
    assert!(matches!(rx.recv().await, Some(StreamChunk::Output(_))));
    assert!(matches!(rx.recv().await, Some(StreamChunk::Error(_))));
    assert!(matches!(rx.recv().await, Some(StreamChunk::Done)));
    assert!(matches!(rx.recv().await, Some(StreamChunk::Heartbeat)));
}
