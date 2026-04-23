//! EventLoop - Lightweight Tokio runtime utilities
//! Provides unified spawn interface with WASM32 compatibility.
//! Uses global Tokio runtime - no custom Runtime::new() or Builder.

/// Spawn a new async task.
/// - Native: Uses `tokio::spawn` multi-threaded
/// - WASM32: Uses `wasm_bindgen_futures::spawn_local`
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

/// Block on future (non-WASM only).
#[cfg(not(target_arch = "wasm32"))]
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    tokio::runtime::Handle::current().block_on(future)
}

/// Yield control to runtime scheduler.
pub async fn yield_now() {
    tokio::task::yield_now().await;
}

/// Sleep for specified duration.
pub async fn sleep(duration: std::time::Duration) {
    tokio::time::sleep(duration).await;
}

/// Timeout wrapper (non-WASM only).
#[cfg(not(target_arch = "wasm32"))]
pub async fn timeout<F>(duration: std::time::Duration, future: F) -> Result<F::Output, ()>
where
    F: std::future::Future,
{
    tokio::time::timeout(duration, future).await.map_err(|_| ())
}

/// Abort handle for task management.
#[cfg(not(target_arch = "wasm32"))]
pub struct AbortHandle {
    inner: tokio::task::AbortHandle,
}

#[cfg(not(target_arch = "wasm32"))]
impl AbortHandle {
    pub fn is_finished(&self) -> bool { self.inner.is_finished() }
    pub fn abort(&self) { self.inner.abort(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_spawn() {
        let handle = spawn(async { 42 });
        assert_eq!(handle.await.unwrap(), 42);
    }
    #[tokio::test]
    async fn test_sleep() {
        sleep(std::time::Duration::from_millis(1)).await;
    }
    #[tokio::test]
    async fn test_yield() {
        yield_now().await;
    }
}
