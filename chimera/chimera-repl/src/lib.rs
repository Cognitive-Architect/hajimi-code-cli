//! Chimera REPL Engine - TUI-free command interaction layer
//!
//! Provides a headless REPL interface for Codex protocol interactions,
//! decoupled from terminal UI concerns for programmatic use and testing.

use std::sync::Arc;

use codex_protocol::ThreadId;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

pub mod engine;
pub mod event;
pub mod session;
pub mod traits;

pub use engine::{EngineController, EngineState};
pub use event::{ReplEvent, ReplEventSender};
pub use session::SessionState;
pub use traits::{ReplConfig, ReplEngineCore, ReplError, ReplResult};

/// Core REPL engine state and event handling.
pub struct ReplEngine {
    /// Unique thread identifier for this REPL session.
    pub thread_id: ThreadId,
    /// Event transmission channel.
    pub event_tx: ReplEventSender,
    /// Session state management.
    pub session: Arc<RwLock<SessionState>>,
    /// Engine configuration.
    pub config: ReplConfig,
    /// Running state flag.
    pub running: Arc<RwLock<bool>>,
}

#[async_trait::async_trait]
impl ReplEngineCore for ReplEngine {
    async fn new(config: ReplConfig) -> ReplResult<Self> {
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

    async fn run(&self) -> ReplResult<()> {
        *self.running.write().await = true;
        info!("REPL engine started");
        while *self.running.read().await {
            debug!("Processing REPL events");
        }
        Ok(())
    }

    async fn shutdown(&self) -> ReplResult<()> {
        *self.running.write().await = false;
        info!("REPL engine shutdown complete");
        Ok(())
    }
}
