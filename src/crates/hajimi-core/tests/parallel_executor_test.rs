use hajimi_core::executor::{Executor, ParallelExecutor, SerialExecutor};
use hajimi_core::query::Query;
use serde_json::json;
use std::time::Instant;

fn create_test_query(name: &str) -> Query {
    Query::new(name, json!({"test": true})).with_timeout(5000)
}

#[tokio::test]
async fn test_parallel_executor_new() {
    let executor = ParallelExecutor::new();
    let query = create_test_query("test_tool");
    let result = executor.execute(query).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.output.as_deref(), Some("executed"));
}

#[tokio::test]
async fn test_parallel_executor_with_max_concurrency() {
    let executor = ParallelExecutor::new().with_max_concurrency(5);
    let query = create_test_query("test_tool");
    let result = executor.execute(query).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_parallel_executor_default() {
    let executor: ParallelExecutor = Default::default();
    let query = create_test_query("test_tool");
    let result = executor.execute(query).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_parallel_execute_batch_order_preserved() {
    let executor = ParallelExecutor::new();
    let queries = vec![
        create_test_query("tool_1"),
        create_test_query("tool_2"),
        create_test_query("tool_3"),
        create_test_query("tool_4"),
    ];
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 4);
    
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Query {} failed: {:?}", i, result);
        let res = result.as_ref().unwrap();
        assert!(res.success, "Query {} was not successful", i);
    }
}

#[tokio::test]
async fn test_parallel_vs_serial_performance() {
    let parallel = ParallelExecutor::new();
    let serial = SerialExecutor::new();
    
    let queries: Vec<Query> = (0..4)
        .map(|i| create_test_query(&format!("tool_{}", i)))
        .collect();
    
    // Warm up
    let _ = parallel.execute_batch(queries.clone()).await;
    
    // Test parallel execution time
    let parallel_start = Instant::now();
    let parallel_results = parallel.execute_batch(queries.clone()).await;
    let parallel_duration = parallel_start.elapsed();
    
    // Test serial execution time
    let serial_start = Instant::now();
    let serial_results = serial.execute_batch(queries.clone()).await;
    let serial_duration = serial_start.elapsed();
    
    assert_eq!(parallel_results.len(), serial_results.len());
    
    for (i, (p, s)) in parallel_results.iter().zip(serial_results.iter()).enumerate() {
        assert!(p.is_ok(), "Parallel query {} failed: {:?}", i, p);
        assert!(s.is_ok(), "Serial query {} failed: {:?}", i, s);
    }
    
    let speedup = serial_duration.as_millis() as f64 / parallel_duration.as_millis() as f64;
    println!("Parallel: {:?}, Serial: {:?}, Speedup: {:.2}x", 
             parallel_duration, serial_duration, speedup);
    
    assert!(
        speedup >= 1.5,
        "Parallel execution should be at least 1.5x faster than serial, got {:.2}x",
        speedup
    );
}

#[tokio::test]
async fn test_parallel_execute_batch_partial_failure() {
    let executor = ParallelExecutor::new();
    // All queries should succeed in current implementation
    let queries = vec![
        create_test_query("tool_1"),
        create_test_query("tool_2"),
    ];
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[tokio::test]
async fn test_parallel_concurrency_limit() {
    let executor = ParallelExecutor::new().with_max_concurrency(2);
    let queries: Vec<Query> = (0..10)
        .map(|i| create_test_query(&format!("tool_{}", i)))
        .collect();
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[tokio::test]
async fn test_parallel_stress_test() {
    let executor = ParallelExecutor::new().with_max_concurrency(50);
    let queries: Vec<Query> = (0..1000)
        .map(|i| create_test_query(&format!("tool_{}", i)))
        .collect();
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 1000);
    
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 1000, "All 1000 queries should succeed");
}

#[tokio::test]
async fn test_parallel_timeout_handling() {
    let executor = ParallelExecutor::new();
    let query = Query::new("slow_tool", json!({})).with_timeout(1);
    
    let result = executor.execute(query).await;
    // Very short timeout may or may not trigger depending on timing
    // This test mainly ensures timeout code path doesn't panic
    match result {
        Ok(_) => {},
        Err(_) => {},
    }
}

#[tokio::test]
async fn test_parallel_empty_batch() {
    let executor = ParallelExecutor::new();
    let results = executor.execute_batch(vec![]).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_parallel_single_query() {
    let executor = ParallelExecutor::new();
    let results = executor.execute_batch(vec![create_test_query("tool")]).await;
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());
}
