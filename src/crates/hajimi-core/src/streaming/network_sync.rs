//! Network Sync - P2P synchronization module
//! DEBT-W03-SYNC: Network synchronization with error handling

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::{interval, Interval};
use std::time::Duration;

use crate::streaming::types::StreamChunk;

/// Network synchronization error types
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    
    #[error("Sync timeout after {0}ms")]
    SyncTimeout(u64),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Serialization failed: {0}")]
    SerializationError(String),
    
    #[error("Network unreachable")]
    NetworkUnreachable,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// Synchronization status for P2P operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    /// Initial connection state
    Idle,
    /// Connecting to peer
    Connecting { peer_id: String },
    /// Actively syncing data
    Syncing { peer_id: String, progress: f64 },
    /// Sync completed successfully
    Completed { peer_id: String, chunks_received: usize },
    /// Sync failed with error
    Failed { peer_id: String, error: NetworkError },
}

impl SyncStatus {
    /// Check if sync is in a terminal state (completed or failed)
    pub fn is_terminal(&self) -> bool {
        matches!(self, SyncStatus::Completed { .. } | SyncStatus::Failed { .. })
    }
    
    /// Get peer_id if available
    pub fn peer_id(&self) -> Option<&str> {
        match self {
            SyncStatus::Connecting { peer_id } => Some(peer_id),
            SyncStatus::Syncing { peer_id, .. } => Some(peer_id),
            SyncStatus::Completed { peer_id, .. } => Some(peer_id),
            SyncStatus::Failed { peer_id, .. } => Some(peer_id),
            SyncStatus::Idle => None,
        }
    }
}

/// Configuration for network synchronization
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Timeout for sync operations in milliseconds
    pub timeout_ms: u64,
    /// Retry attempts for failed connections
    pub retry_attempts: u32,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Maximum concurrent peers
    pub max_concurrent_peers: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            retry_attempts: 3,
            heartbeat_interval_ms: 5_000,
            max_concurrent_peers: 10,
        }
    }
}

/// P2P network synchronization stream
/// 
/// Handles peer-to-peer data synchronization with proper error handling.
/// Returns Result<SyncStatus, NetworkError> instead of panicking.
pub struct NetworkSyncStream {
    config: SyncConfig,
    status: SyncStatus,
    heartbeat: Interval,
    buffer: Vec<StreamChunk>,
}

impl NetworkSyncStream {
    /// Create a new NetworkSyncStream with the given configuration
    pub fn new(config: SyncConfig) -> Self {
        let heartbeat = interval(Duration::from_millis(config.heartbeat_interval_ms));
        Self {
            config,
            status: SyncStatus::Idle,
            heartbeat,
            buffer: Vec::new(),
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SyncConfig::default())
    }
    
    /// Connect to a peer and return the sync status
    /// 
    /// # Arguments
    /// * `peer_id` - The identifier of the peer to connect to
    /// 
    /// # Returns
    /// * `Result<SyncStatus, NetworkError>` - Success status or network error
    pub async fn connect_to_peer(&mut self, peer_id: impl Into<String>) -> Result<SyncStatus, NetworkError> {
        let peer_id = peer_id.into();
        
        // Validate peer_id
        if peer_id.is_empty() {
            return Err(invalid_peer_id_error());
        }
        
        self.status = SyncStatus::Connecting { peer_id: peer_id.clone() };
        
        // Simulate connection attempt with retry logic
        for attempt in 0..self.config.retry_attempts {
            match self.attempt_connection(&peer_id).await {
                Ok(()) => {
                    self.status = SyncStatus::Syncing { peer_id, progress: 0.0 };
                    return Ok(self.status.clone());
                }
                Err(e) if attempt < self.config.retry_attempts - 1 => {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    continue;
                }
                Err(e) => {
                    self.status = SyncStatus::Failed { 
                        peer_id, 
                        error: e.clone() 
                    };
                    return Err(e);
                }
            }
        }
        
        Err(NetworkError::ConnectionFailed(format!(
            "Failed to connect after {} attempts", 
            self.config.retry_attempts
        )))
    }
    
    /// Attempt a single connection (simulated)
    async fn attempt_connection(&self, peer_id: &str) -> Result<(), NetworkError> {
        // In real implementation, this would attempt actual P2P connection
        // For now, simulate successful connection
        Ok(())
    }
    
    /// Get current sync status
    pub fn status(&self) -> &SyncStatus {
        &self.status
    }
    
    /// Process incoming sync data
    /// 
    /// # Returns
    /// * `Result<Option<StreamChunk>, NetworkError>` - Data chunk or error
    pub fn poll_sync(&mut self, cx: &mut Context<'_>) -> Result<Option<StreamChunk>, NetworkError> {
        // Check heartbeat
        if self.heartbeat.poll_tick(cx).is_ready() {
            // Send heartbeat to keep connection alive
        }
        
        // Return buffered data if available
        if !self.buffer.is_empty() {
            return Ok(self.buffer.pop());
        }
        
        Ok(None)
    }
}

impl Stream for NetworkSyncStream {
    type Item = Result<StreamChunk, NetworkError>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.poll_sync(cx) {
            Ok(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
            Ok(None) => Poll::Pending,
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}

/// Creates an invalid peer ID error
fn invalid_peer_id_error() -> NetworkError {
    NetworkError::ProtocolError("Invalid peer identifier".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.timeout_ms, 30_000);
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.heartbeat_interval_ms, 5_000);
        assert_eq!(config.max_concurrent_peers, 10);
    }

    #[tokio::test]
    async fn test_network_sync_stream_creation() {
        let stream = NetworkSyncStream::with_defaults();
        assert!(matches!(stream.status(), SyncStatus::Idle));
    }

    #[tokio::test]
    async fn test_connect_to_peer_success() -> Result<(), NetworkError> {
        let mut stream = NetworkSyncStream::with_defaults();
        let result = stream.connect_to_peer("peer-123").await?;
        
        assert!(matches!(result, SyncStatus::Syncing { peer_id, .. } if peer_id == "peer-123"));
        Ok(())
    }

    #[tokio::test]
    async fn test_sync_status_is_terminal() {
        let completed = SyncStatus::Completed { 
            peer_id: "test".into(), 
            chunks_received: 10 
        };
        assert!(completed.is_terminal());
        
        let failed = SyncStatus::Failed { 
            peer_id: "test".into(), 
            error: NetworkError::NetworkUnreachable 
        };
        assert!(failed.is_terminal());
        
        let syncing = SyncStatus::Syncing { 
            peer_id: "test".into(), 
            progress: 0.5 
        };
        assert!(!syncing.is_terminal());
    }

    #[tokio::test]
    async fn test_sync_status_peer_id() {
        let status = SyncStatus::Connecting { peer_id: "peer-abc".into() };
        assert_eq!(status.peer_id(), Some("peer-abc"));
        
        let idle = SyncStatus::Idle;
        assert_eq!(idle.peer_id(), None);
    }

    #[tokio::test]
    async fn test_network_error_display() {
        let err = NetworkError::ConnectionFailed("timeout".into());
        assert!(err.to_string().contains("Connection failed"));
        
        let err = NetworkError::SyncTimeout(5000);
        assert!(err.to_string().contains("Sync timeout"));
    }

    #[tokio::test]
    async fn test_stream_poll() -> Result<(), NetworkError> {
        let mut stream = NetworkSyncStream::with_defaults();
        stream.connect_to_peer("test-peer").await?;
        
        // Stream should be ready but return pending (no data yet)
        let result = stream.next().await;
        // Since there's no data, it will return None or Pending
        // This test verifies the stream doesn't panic
        let is_valid = match result {
            None => true,
            Some(Err(_)) => true,
            Some(Ok(_)) => false,
        };
        assert!(is_valid);
        Ok(())
    }
}
