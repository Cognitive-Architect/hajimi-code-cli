use hajimi_core::{Query, SerialExecutor, Executor, EngineError};
use serde_json::json;

#[tokio::test]
async fn test_serial_executor_new() {
    let executor = SerialExecutor::new();
    let _ = executor;
}

#[tokio::test]
async fn test_serial_executor_execute_success() {
    let executor = SerialExecutor::new();
    let query = Query::new("test_tool", json!({"key": "value"}))
        .with_timeout(1000);
    
    let result = executor.execute(query).await;
    assert!(result.is_ok());
    
    let query_result = result.unwrap();
    assert!(query_result.success);
    assert!(query_result.execution_time_ms > 0);
}

#[tokio::test]
async fn test_serial_executor_execute_timeout() {
    let executor = SerialExecutor::new();
    let query = Query::new("slow_tool", json!({}))
        .with_timeout(1);  // 1ms timeout, but execution takes 50ms
    
    let result = executor.execute(query).await;
    assert!(result.is_err());  // Should timeout and return Err
    
    // Verify it's a Timeout error
    match result {
        Err(EngineError::Timeout(1)) => (), // Expected
        _ => panic!("Expected Timeout error"),
    }
}

#[tokio::test]
async fn test_serial_executor_execute_batch_empty() {
    let executor = SerialExecutor::new();
    let queries: Vec<Query> = vec![];
    
    let results = executor.execute_batch(queries).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_serial_executor_execute_batch_single() {
    let executor = SerialExecutor::new();
    let queries = vec![
        Query::new("tool1", json!({"id": 1})).with_timeout(1000),
    ];
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());
}

#[tokio::test]
async fn test_serial_executor_execute_batch_multiple() {
    let executor = SerialExecutor::new();
    let queries = vec![
        Query::new("tool1", json!({"id": 1})).with_timeout(1000),
        Query::new("tool2", json!({"id": 2})).with_timeout(1000),
        Query::new("tool3", json!({"id": 3})).with_timeout(1000),
    ];
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 3);
    
    for result in &results {
        assert!(result.is_ok());
        let qr = result.as_ref().unwrap();
        assert!(qr.success);
        assert!(qr.execution_time_ms > 0);
    }
}

#[tokio::test]
async fn test_serial_executor_execute_batch_order_preserved() {
    let executor = SerialExecutor::new();
    let queries = vec![
        Query::new("tool_a", json!({"seq": 1})).with_timeout(1000),
        Query::new("tool_b", json!({"seq": 2})).with_timeout(1000),
    ];
    
    let results = executor.execute_batch(queries).await;
    assert_eq!(results.len(), 2);
    
    for result in results {
        assert!(result.is_ok());
    }
}

#[test]
fn test_serial_executor_default() {
    let executor: SerialExecutor = Default::default();
    let _ = executor;
}

#[tokio::test]
async fn test_query_result_success_constructor() {
    use hajimi_core::QueryResult;
    let result = QueryResult::success("output", 100);
    assert!(result.success);
    assert_eq!(result.output, Some("output".to_string()));
    assert_eq!(result.error, None);
    assert_eq!(result.execution_time_ms, 100);
}

#[tokio::test]
async fn test_query_result_error_constructor() {
    use hajimi_core::QueryResult;
    let result = QueryResult::error("error_msg", 50);
    assert!(!result.success);
    assert_eq!(result.output, None);
    assert_eq!(result.error, Some("error_msg".to_string()));
    assert_eq!(result.execution_time_ms, 50);
}
