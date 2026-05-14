//! EventLoop Adapter for Chimera REPL
//! Bridges Chimera REPL with foundation EventLoop abstraction.

pub use foundation_eventloop::{block_on, sleep, spawn, timeout, yield_now};
use std::sync::Arc;
pub use tokio::sync::mpsc;

/// Async RwLock type.
pub type RwLock<T> = tokio::sync::RwLock<T>;
/// Arc-wrapped RwLock type.
pub type ArcRwLock<T> = Arc<RwLock<T>>;
/// Read lock guard.
pub type ReadGuard<'a, T> = tokio::sync::RwLockReadGuard<'a, T>;
/// Write lock guard.
pub type WriteGuard<'a, T> = tokio::sync::RwLockWriteGuard<'a, T>;
/// Channel sender.
pub type EventSender<T> = mpsc::Sender<T>;
/// Channel receiver.
pub type EventReceiver<T> = mpsc::Receiver<T>;
/// Duration type.
pub type Duration = std::time::Duration;

/// Create new async channel.
pub fn channel<T>(buffer: usize) -> (EventSender<T>, EventReceiver<T>) {
    mpsc::channel(buffer)
}

/// Create new async RwLock in Arc.
pub fn rwlock<T>(value: T) -> ArcRwLock<T> {
    Arc::new(RwLock::new(value))
}

/// Async read lock helper.
pub async fn read<T>(lock: &ArcRwLock<T>) -> ReadGuard<'_, T> {
    lock.read().await
}

/// Async write lock helper.
pub async fn write<T>(lock: &ArcRwLock<T>) -> WriteGuard<'_, T> {
    lock.write().await
}

/// Spawn detached background task.
pub fn spawn_detached<F>(future: F)
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[allow(clippy::let_underscore_future)]
    let _ = spawn(future);
}

/// Block on future with timeout.
pub fn block_on_timeout<F>(future: F, timeout_ms: u64) -> Option<F::Output>
where
    F: std::future::Future,
{
    block_on(async move {
        timeout(Duration::from_millis(timeout_ms), future)
            .await
            .ok()
    })
}
