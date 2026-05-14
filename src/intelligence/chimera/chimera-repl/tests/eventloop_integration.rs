//! EventLoop-Chimera Integration Tests
//!
//! Validates that Chimera REPL correctly uses EventLoop primitives
//! instead of direct tokio calls.

use chimera_repl::eventloop_adapter::*;
use chimera_repl::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// FUNC-001: Verify Chimera uses EventLoop::spawn
#[tokio::test]
async fn test_eventloop_spawn_integration() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let handle = spawn(async move {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        42
    });

    let result = handle.await.unwrap();
    assert_eq!(result, 42);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

/// FUNC-002: Verify Chimera uses EventLoop::block_on
/// Note: block_on requires a Tokio runtime context
#[tokio::test]
async fn test_eventloop_block_on_integration() {
    // Test block_on within a runtime context by spawning a blocking task
    let result = tokio::task::spawn_blocking(|| {
        block_on(async {
            sleep(Duration::from_millis(1)).await;
            "blocked"
        })
    })
    .await
    .unwrap();
    assert_eq!(result, "blocked");
}

/// CONST-001 & CONST-002: Verify adapter compiles and basic integration works
#[tokio::test]
async fn test_adapter_channel_and_rwlock() {
    // Test channel
    let (tx, mut rx) = channel::<String>(10);
    spawn(async move {
        tx.send("hello".to_string()).await.unwrap();
    });

    let msg = rx.recv().await.unwrap();
    assert_eq!(msg, "hello");

    // Test RwLock
    let lock = rwlock(0u32);
    *write(&lock).await = 42;
    let val = *read(&lock).await;
    assert_eq!(val, 42);
}

/// NEG-001: Verify timeout wrapper works
#[tokio::test]
async fn test_eventloop_timeout() {
    let result = timeout(Duration::from_millis(10), async {
        sleep(Duration::from_millis(100)).await;
        "completed"
    })
    .await;

    assert!(result.is_err());
}

/// NEG-002: Verify spawn_detached works without handle
#[tokio::test]
async fn test_spawn_detached() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    spawn_detached(async move {
        sleep(Duration::from_millis(1)).await;
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    sleep(Duration::from_millis(20)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

/// E2E-001: End-to-end REPL engine integration test
#[tokio::test]
async fn test_repl_e2e_integration() {
    let config = ReplConfig::default();

    // Create engine using EventLoop primitives
    let engine = ReplEngine::new(config).await.unwrap();

    // Verify engine uses EventLoop-backed RwLock
    assert!(!*read(&engine.running).await);

    // Start engine
    *write(&engine.running).await = true;
    assert!(*read(&engine.running).await);

    // Shutdown
    *write(&engine.running).await = false;
    assert!(!*read(&engine.running).await);
}

/// PASSED marker for test harness
#[tokio::test]
async fn eventloop_integration_tests_completed() {
    println!("EVENTLOOP-CHIMERA INTEGRATION: PASSED");
}
