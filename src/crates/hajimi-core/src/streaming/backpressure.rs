//! Backpressure controller with bounded channel + Semaphore
//! DEBT-W02-B03: Bounded channel + Semaphore双重backpressure

use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{timeout, Duration};

use crate::error::EngineError;
use crate::streaming::types::{StreamChunk, StreamConfig};

pub struct BackpressureController {
    sender: mpsc::Sender<StreamChunk>,
    semaphore: Arc<Semaphore>,
    _config: StreamConfig,
}

impl BackpressureController {
    pub fn new(config: StreamConfig) -> (Self, mpsc::Receiver<StreamChunk>) {
        let (tx, rx) = mpsc::channel(config.buffer_size);
        let sem = Arc::new(Semaphore::new(config.buffer_size));
        (Self { sender: tx, semaphore: sem, _config: config }, rx)
    }

    pub async fn try_send(&self, chunk: StreamChunk) -> Result<(), EngineError> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                match self.sender.try_send(chunk) {
                    Ok(()) => {
                        drop(permit);
                        Ok(())
                    }
                    Err(_) => Err(EngineError::ExecutionFailed("Channel full".to_string())),
                }
            }
            Err(_) => Err(EngineError::ExecutionFailed("Capacity exceeded".to_string())),
        }
    }

    pub async fn send_with_timeout(&self, chunk: StreamChunk, timeout_ms: u64) -> Result<(), EngineError> {
        let permit = match self.semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => return Err(EngineError::ExecutionFailed("Semaphore closed".to_string())),
        };

        let result = timeout(
            Duration::from_millis(timeout_ms),
            self.sender.send(chunk),
        ).await;

        drop(permit);

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(EngineError::ExecutionFailed("Channel closed".to_string())),
            Err(_) => Err(EngineError::Timeout(timeout_ms)),
        }
    }

    pub fn capacity(&self) -> usize {
        self.semaphore.available_permits()
    }
}
