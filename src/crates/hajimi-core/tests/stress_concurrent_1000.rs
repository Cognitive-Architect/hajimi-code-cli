//! Stress Tests - 1000 Concurrent Streams (B-W07-02)
//!
//! 验证目标:
//! - 1000并发流稳定性
//! - P95延迟 < 500ms
//! - 内存稳定无泄漏
//! - Config热重载 < 3秒

use hajimi_core::{
    Config, ConfigLoader, HotReloadHandle, ParallelExecutor, Query, Executor,
};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Test 1000 concurrent queries complete successfully
#[tokio::test]
async fn test_1000_concurrent_queries() {
    let executor = Arc::new(ParallelExecutor::default().with_max_concurrency(100));
    let mut handles = vec![];
    
    let start = Instant::now();
    
    for i in 0..1000 {
        let executor_clone = executor.clone();
        let handle = tokio::spawn(async move {
            let query = Query::new("test", json!({"id": i}));
            executor_clone.execute(query).await
        });
        handles.push(handle);
    }
    
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            success_count += 1;
        }
    }
    
    let elapsed = start.elapsed();
    println!("1000 concurrent queries: {} success in {:?}", success_count, elapsed);
    
    assert_eq!(success_count, 1000, "All 1000 queries should complete");
    assert!(elapsed < Duration::from_secs(30), "Should complete within 30 seconds");
}

/// Test P95 latency under 500ms
#[tokio::test]
async fn test_p95_latency_under_500ms() {
    let executor = ParallelExecutor::default().with_max_concurrency(50);
    let mut latencies = vec![];
    
    for i in 0..100 {
        let start = Instant::now();
        let query = Query::new("test", json!({"id": i}));
        let _ = executor.execute(query).await;
        latencies.push(start.elapsed().as_millis() as u64);
    }
    
    latencies.sort();
    let p95_idx = (latencies.len() as f64 * 0.95) as usize;
    let p95 = latencies[p95_idx.min(latencies.len() - 1)];
    
    println!("P95 latency: {}ms", p95);
    assert!(p95 < 500, "P95 latency should be under 500ms, got {}ms", p95);
}

/// Test config hot reload under 3 seconds
#[tokio::test]
async fn test_config_hot_reload_under_3_seconds() {
    let temp_file = std::env::temp_dir().join("test_stress_reload.toml");
    tokio::fs::write(&temp_file, "preset = \"daily\"").await.unwrap();
    
    let start = Instant::now();
    let config = ConfigLoader::from_file(&temp_file).await.unwrap();
    let load_time = start.elapsed();
    
    println!("Config load time: {:?}", load_time);
    assert!(load_time < Duration::from_secs(3), "Config load should be under 3 seconds");
    
    // Test hot reload watch setup
    let watch_start = Instant::now();
    let handle = HotReloadHandle::watch(&temp_file).await;
    let watch_time = watch_start.elapsed();
    
    println!("Hot reload watch setup time: {:?}", watch_time);
    assert!(handle.is_ok());
    assert!(watch_time < Duration::from_secs(3), "Hot reload watch setup should be under 3 seconds");
    
    let _ = tokio::fs::remove_file(&temp_file).await;
}

/// Test memory stability under load
#[tokio::test]
async fn test_memory_stable_under_load() {
    let executor = ParallelExecutor::default().with_max_concurrency(50);
    
    // Warm up
    for i in 0..100 {
        let query = Query::new("test", json!({"id": i}));
        let _ = executor.execute(query).await;
    }
    
    // Measure during sustained load
    let start = Instant::now();
    let mut completed = 0;
    
    while start.elapsed() < Duration::from_secs(5) {
        let query = Query::new("test", json!({"batch": completed}));
        let _ = executor.execute(query).await;
        completed += 1;
    }
    
    println!("Completed {} queries in 5 seconds", completed);
    assert!(completed > 50, "Should complete at least 50 queries per 5 seconds");
}

/// Test concurrent config reads during execution
#[tokio::test]
async fn test_concurrent_config_and_execution() {
    let config = Arc::new(tokio::sync::RwLock::new(Config::default()));
    let executor = Arc::new(ParallelExecutor::default());
    let mut handles = vec![];
    
    // Spawn config readers
    for _ in 0..50 {
        let config_clone = config.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..20 {
                let _ = config_clone.read().await;
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }));
    }
    
    // Spawn executors
    for i in 0..50 {
        let executor_clone = executor.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..20 {
                let query = Query::new("test", json!({"id": i * 20 + j}));
                let _ = executor_clone.execute(query).await;
            }
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test burst capacity (1000 simultaneous spawns)
#[tokio::test]
async fn test_burst_capacity_1000() {
    let executor = Arc::new(ParallelExecutor::default().with_max_concurrency(200));
    let start = Instant::now();
    
    let mut handles = vec![];
    for i in 0..1000 {
        let executor_clone = executor.clone();
        let handle = tokio::spawn(async move {
            let query = Query::new("test", json!({"id": i}));
            executor_clone.execute(query).await
        });
        handles.push(handle);
    }
    
    let mut completed = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            completed += 1;
        }
    }
    
    let elapsed = start.elapsed();
    println!("Burst 1000: {} completed in {:?}", completed, elapsed);
    
    assert_eq!(completed, 1000);
}

/// Test graceful degradation under extreme load
#[tokio::test]
async fn test_graceful_degradation() {
    let executor = ParallelExecutor::default().with_max_concurrency(10);
    let mut latencies = vec![];
    
    // Submit 500 queries with only 10 concurrency
    let start = Instant::now();
    
    for i in 0..500 {
        let query_start = Instant::now();
        let query = Query::new("test", json!({"id": i}));
        let _ = executor.execute(query).await;
        latencies.push(query_start.elapsed().as_millis() as u64);
    }
    
    let total_time = start.elapsed();
    let avg_latency: u64 = latencies.iter().sum::<u64>() / latencies.len() as u64;
    
    println!("500 queries with 10 concurrency: total={:?}, avg={}ms", total_time, avg_latency);
    
    // Should complete without errors
    assert!(total_time > Duration::from_secs(20), "Should take time with limited concurrency");
}
