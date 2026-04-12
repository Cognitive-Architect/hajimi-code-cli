use std::time::Duration;
use tokio::time::sleep;
use crate::error::EngineError;

pub async fn with_retry<F, Fut, T>(
    mut operation: F,
    max_attempts: u32,
    base_delay_ms: u64,
) -> Result<T, EngineError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, EngineError>>,
{
    let mut last_error = None;
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_attempts {
                    let backoff = 1 << (attempt - 1);
                    sleep(Duration::from_millis(base_delay_ms * backoff)).await;
                }
            }
        }
    }
    Err(EngineError::RetryExhausted {
        attempts: max_attempts,
        source: Box::new(last_error.expect("BUG: last_error should be Some after at least one failed attempt")),
    })
}
