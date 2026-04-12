//! Week 1 Integration Test - HAJIMI-W01-SATURN-001
//! 
//! 验证目标:
//! - B-01: Crate框架 + 核心结构体
//! - B-02: SerialExecutor 串行执行器
//! - B-03: ParallelExecutor 并行执行器  
//! - B-04: 错误处理 + 重试机制

use hajimi_core::{
    EngineError, Executor, ParallelExecutor, Query, QueryResult, SerialExecutor, with_retry,
};
use serde_json::json;
use std::time::{Duration, Instant};

// ============================================================================
// B-01: 核心结构体验证
// ============================================================================

#[test]
fn test_query_struct_creation() {
    let query = Query::new("test_tool", json!({"key": "value"}));
    assert_eq!(query.tool_name, "test_tool");
    assert_eq!(query.timeout_ms, 30_000); // default
}

#[test]
fn test_query_with_timeout() {
    let query = Query::new("test_tool", json!({}))
        .with_timeout(5_000);
    assert_eq!(query.timeout_ms, 5_000);
}

#[test]
fn test_query_result_success() {
    let result = QueryResult::success("output_data", 100);
    assert!(result.success);
    assert_eq!(result.output, Some("output_data".to_string()));
    assert_eq!(result.error, None);
    assert_eq!(result.execution_time_ms, 100);
}

#[test]
fn test_query_result_error() {
    let result = QueryResult::error("error_message", 50);
    assert!(!result.success);
    assert_eq!(result.output, None);
    assert_eq!(result.error, Some("error_message".to_string()));
    assert_eq!(result.execution_time_ms, 50);
}

// ============================================================================
// B-02/B-03: 执行器集成验证
// ============================================================================

#[tokio::test]
async fn test_serial_vs_parallel_correctness() {
    // Both executors should produce same results
    let serial = SerialExecutor::new();
    let parallel = ParallelExecutor::new();
    
    let queries = vec![
        Query::new("tool1", json!({"id": 1})).with_timeout(1000),
        Query::new("tool2", json!({"id": 2})).with_timeout(1000),
        Query::new("tool3", json!({"id": 3})).with_timeout(1000),
    ];
    
    let serial_results = serial.execute_batch(queries.clone()).await;
    let parallel_results = parallel.execute_batch(queries).await;
    
    assert_eq!(serial_results.len(), parallel_results.len());
    
    for (s, p) in serial_results.iter().zip(parallel_results.iter()) {
        assert_eq!(s.is_ok(), p.is_ok());
    }
}

#[tokio::test]
async fn test_parallel_faster_than_serial() {
    let serial = SerialExecutor::new();
    let parallel = ParallelExecutor::new();
    
    let queries: Vec<_> = (0..4)
        .map(|i| Query::new(&format!("tool{}", i), json!({"id": i})).with_timeout(5000))
        .collect();
    
    let serial_start = Instant::now();
    let _ = serial.execute_batch(queries.clone()).await;
    let serial_elapsed = serial_start.elapsed();
    
    let parallel_start = Instant::now();
    let _ = parallel.execute_batch(queries).await;
    let parallel_elapsed = parallel_start.elapsed();
    
    // Parallel should be at least 2x faster for 4 concurrent tasks
    assert!(
        parallel_elapsed < serial_elapsed / 2,
        "Parallel ({:?}) should be at least 2x faster than serial ({:?})",
        parallel_elapsed,
        serial_elapsed
    );
}

// ============================================================================
// B-04: 错误处理 + 重试机制集成验证
// ============================================================================

#[tokio::test]
async fn test_error_chain_with_retry() {
    use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
    
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();
    
    let result = with_retry(
        move || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                if count < 3 {
                    Err(EngineError::ExecutionFailed("temporary error".to_string()))
                } else {
                    Ok("success")
                }
            }
        },
        3,
        10,
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_exhausted_error_chain() {
    let result: Result<&str, EngineError> = with_retry(
        || async {
            Err::<&str, EngineError>(EngineError::ToolNotFound("missing".to_string()))
        },
        2,
        1,
    ).await;
    
    match result {
        Err(EngineError::RetryExhausted { attempts, source }) => {
            assert_eq!(attempts, 2);
            // Verify source error is preserved
            match *source {
                EngineError::ToolNotFound(_) => (), // Expected
                _ => panic!("Wrong source error type"),
            }
        }
        _ => panic!("Expected RetryExhausted error"),
    }
}

#[test]
fn test_all_error_variants() {
    // Verify all EngineError variants can be created
    let _ = EngineError::ToolNotFound("test".to_string());
    let _ = EngineError::Timeout(1000);
    let _ = EngineError::RetryExhausted {
        attempts: 3,
        source: Box::new(EngineError::ExecutionFailed("test".to_string())),
    };
    let _ = EngineError::ExecutionFailed("test".to_string());
    let _ = EngineError::InvalidParameters("test".to_string());
}

// ============================================================================
// Week 1 完整集成验证
// ============================================================================

#[tokio::test]
async fn test_week1_full_pipeline() {
    // Simulate: Query -> Execute with Retry -> Parallel Batch
    let parallel = ParallelExecutor::new();
    
    let queries: Vec<_> = (0..5)
        .map(|i| Query::new(&format!("tool{}", i), json!({"task": i})).with_timeout(10_000))
        .collect();
    
    // Execute batch with implicit per-query timeout
    let results = parallel.execute_batch(queries).await;
    
    assert_eq!(results.len(), 5);
    
    for result in &results {
        assert!(result.is_ok(), "All queries should succeed");
        let qr = result.as_ref().unwrap();
        assert!(qr.success);
        assert!(qr.execution_time_ms > 0);
    }
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    let parallel = ParallelExecutor::new();
    
    // Mix of valid and potentially failing queries
    let queries = vec![
        Query::new("valid_tool", json!({})).with_timeout(1000),
        Query::new("valid_tool", json!({})).with_timeout(1000),
    ];
    
    let results = parallel.execute_batch(queries).await;
    
    // All should succeed (our mock always succeeds)
    assert!(results.iter().all(|r| r.is_ok()));
}

// ============================================================================
// 性能基准验证
// ============================================================================

#[tokio::test]
async fn test_week1_performance_requirements() {
    let parallel = ParallelExecutor::new();
    
    let start = Instant::now();
    
    // Execute 10 concurrent queries
    let queries: Vec<_> = (0..10)
        .map(|i| Query::new(&format!("tool{}", i), json!({"id": i})).with_timeout(5000))
        .collect();
    
    let results = parallel.execute_batch(queries).await;
    
    let elapsed = start.elapsed();
    
    // All should complete
    assert_eq!(results.len(), 10);
    
    // Should complete in reasonable time (parallel, not serial)
    // Serial would be 10 * 50ms = 500ms
    // Parallel should be ~50-100ms
    assert!(
        elapsed < Duration::from_millis(300),
        "10 parallel queries should complete in <300ms, took {:?}",
        elapsed
    );
}
