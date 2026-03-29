//! Chimera REPL Engine - TUI-free command interaction layer
//!
//! Provides a headless REPL interface for Codex protocol interactions,
//! decoupled from terminal UI concerns for programmatic use and testing.

use std::collections::HashMap;
use std::sync::Arc;

use codex_protocol::ThreadId;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

pub mod engine;
pub mod event;
pub mod session;

pub use engine::ReplEngine;
pub use event::{ReplEvent, ReplEventSender};
pub use session::SessionState;

/// Configuration for REPL engine behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplConfig {
    /// Thread ID for this REPL session.
    pub thread_id: Option<ThreadId>,
    /// Enable verbose logging.
    pub verbose: bool,
    /// Custom metadata storage.
    pub metadata: HashMap<String, String>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            thread_id: None,
            verbose: false,
            metadata: HashMap::new(),
        }
    }
}

/// Core REPL engine state and event handling.
/// 
/// Replaces TUI-specific `App` structure with a headless,
/// channel-driven architecture suitable for programmatic use.
pub struct ReplEngine {
    /// Unique thread identifier for this REPL session.
    pub thread_id: ThreadId,
    /// Event transmission channel (replaces TUI event loop).
    pub event_tx: ReplEventSender,
    /// Session state management.
    pub session: Arc<RwLock<SessionState>>,
    /// Engine configuration.
    pub config: ReplConfig,
    /// Running state flag.
    pub running: Arc<RwLock<bool>>,
}

impl ReplEngine {
    /// Create a new REPL engine instance.
    pub async fn new(config: ReplConfig) -> Result<Self, ReplError> {
        let thread_id = config.thread_id.unwrap_or_else(ThreadId::new);
        let (event_tx, _event_rx) = mpsc::channel(1024);
        
        info!(?thread_id, "Initializing Chimera REPL engine");
        
        Ok(Self {
            thread_id,
            event_tx: ReplEventSender::new(event_tx),
            session: Arc::new(RwLock::new(SessionState::default())),
            config,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the REPL event processing loop.
    pub async fn run(&self) -> Result<(), ReplError> {
        *self.running.write().await = true;
        info!("REPL engine started");
        
        while *self.running.read().await {
            // Event processing loop (TUI-free)
            debug!("Processing REPL events");
        }
        
        Ok(())
    }

    /// Gracefully shutdown the engine.
    pub async fn shutdown(&self) {
        *self.running.write().await = false;
        info!("REPL engine shutdown complete");
    }
}

/// REPL-specific error types.
#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    #[error("Session error: {0}")]
    Session(String),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] serde_json::Error),
    
    #[error("Channel error: {0}")]
    Channel(String),
}

/// Result type alias for REPL operations.
pub type ReplResult<T> = Result<T, ReplError>;
