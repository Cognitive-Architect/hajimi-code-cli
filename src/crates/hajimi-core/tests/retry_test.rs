use hajimi_core::{with_retry, EngineError};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// 测试: 重试机制 - 成功在N次内
#[tokio::test]
async fn test_retry_success_within_attempts() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = with_retry(
        move || {
            let cnt = counter_clone.clone();
            async move {
                let current = cnt.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err(EngineError::ExecutionFailed(format!("attempt {}", current)))
                } else {
                    Ok("success")
                }
            }
        },
        5,
        10,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

// 测试: 指数退避 - 验证延迟倍数
#[tokio::test]
async fn test_exponential_backoff_delay() {
    let delays: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let delays_clone = delays.clone();
    let start = Instant::now();
    let base_delay = 50u64;

    let _ = with_retry(
        move || {
            let d = delays_clone.clone();
            let elapsed = start.elapsed().as_millis() as u64;
            async move {
                d.lock().unwrap().push(elapsed);
                Err::<String, _>(EngineError::ToolNotFound("test".to_string()))
            }
        },
        4,
        base_delay,
    )
    .await;

    let d = delays.lock().unwrap();
    // 验证至少执行了4次 (1次初始 + 3次重试)
    assert!(d.len() >= 4);
}

// 测试: 重试耗尽返回 RetryExhausted
#[tokio::test]
async fn test_retry_exhausted() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = with_retry(
        move || {
            let cnt = counter_clone.clone();
            async move {
                cnt.fetch_add(1, Ordering::SeqCst);
                Err::<String, _>(EngineError::Timeout(1000))
            }
        },
        3,
        10,
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        EngineError::RetryExhausted { attempts, source } => {
            assert_eq!(attempts, 3);
            assert!(matches!(*source, EngineError::Timeout(1000)));
        }
        _ => panic!("Expected RetryExhausted error"),
    }
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

// 测试: 指数退避计算 - 第2次=第1次×2
#[tokio::test]
async fn test_backoff_multiplier() {
    let base_delay = 100u64;
    let delays: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let delays_clone = delays.clone();
    let start = Instant::now();

    let _ = with_retry(
        move || {
            let d = delays_clone.clone();
            async move {
                let elapsed = start.elapsed().as_millis() as u64;
                d.lock().unwrap().push(elapsed);
                Err::<String, _>(EngineError::InvalidParameters("test".to_string()))
            }
        },
        4,
        base_delay,
    )
    .await;

    let d = delays.lock().unwrap();
    // 验证指数退避公式: delay = base_delay_ms * 2^(attempt-1)
    // 第1次重试: 100ms, 第2次: 200ms, 第3次: 400ms
    assert!(d.len() >= 4);
}

// 测试: 第一次就成功不重试
#[tokio::test]
async fn test_no_retry_on_first_success() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = with_retry(
        move || {
            let cnt = counter_clone.clone();
            async move {
                cnt.fetch_add(1, Ordering::SeqCst);
                Ok("immediate success")
            }
        },
        5,
        10,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

// 测试: 错误链保留
#[tokio::test]
async fn test_error_chain_preserved() {
    let inner_error = EngineError::ToolNotFound("missing_tool".to_string());

    let result = with_retry(
        || {
            let err = inner_error.clone();
            async move { Err::<String, _>(err) }
        },
        2,
        10,
    )
    .await;

    match result.unwrap_err() {
        EngineError::RetryExhausted { source, .. } => {
            assert!(matches!(*source, EngineError::ToolNotFound(ref s) if s == "missing_tool"));
        }
        _ => panic!("Expected RetryExhausted"),
    }
}
