//! Chimera REPL core traits.
//!
//! Defines the abstract interface for REPL engine operations,
//! enabling pluggable implementations and test doubles.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use codex_twist::thread::ThreadId;

/// Configuration for REPL engine behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplConfig {
    /// Thread ID for this REPL session.
    pub thread_id: Option<ThreadId>,
    /// Enable verbose logging.
    pub verbose: bool,
    /// Custom metadata storage.
    pub metadata: HashMap<String, String>,
}

/// Core trait for REPL engine implementations.
///
/// Defines the minimal interface for a headless REPL engine
/// without TUI dependencies. Implementors must be Send + Sync
/// for concurrent usage.
#[async_trait]
pub trait ReplEngineCore: Send + Sync {
    /// Create a new REPL engine instance with the given config.
    ///
    /// Returns the engine instance and an event receiver channel.
    async fn new(config: ReplConfig) -> Result<Self, ReplError>
    where
        Self: Sized;

    /// Start the REPL event processing loop.
    ///
    /// Blocks until shutdown is requested. Runs the main event
    /// loop without TUI interactions.
    async fn run(&self) -> Result<(), ReplError>;

    /// Gracefully shutdown the engine.
    ///
    /// Signals the event loop to terminate and awaits cleanup.
    async fn shutdown(&self) -> Result<(), ReplError>;
}

/// REPL-specific error types.
#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    /// Session state operation failed.
    #[error("Session error: {0}")]
    Session(String),

    /// Protocol serialization/deserialization error.
    #[error("Protocol error: {0}")]
    Protocol(#[from] serde_json::Error),

    /// Channel communication error.
    #[error("Channel error: {0}")]
    Channel(String),
}

/// Result type alias for REPL operations.
pub type ReplResult<T> = Result<T, ReplError>;
