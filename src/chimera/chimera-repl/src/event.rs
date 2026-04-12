//! Event system for REPL communication.
//!
//! Replaces TUI event handling with async channel-based messaging.

use tokio::sync::mpsc::Sender;

/// Events emitted by the REPL engine.
#[derive(Debug, Clone)]
pub enum ReplEvent {
    /// User input received.
    Input(String),
    /// Command execution request.
    Command { name: String, args: Vec<String> },
    /// Protocol event placeholder (Codex protocol integration).
    ProtocolEvent(String),
    /// Session state change.
    SessionUpdate(crate::SessionState),
    /// Shutdown request.
    Shutdown,
}

/// Event sender wrapper for REPL communication.
#[derive(Debug, Clone)]
pub struct ReplEventSender {
    inner: Sender<ReplEvent>,
}

impl ReplEventSender {
    /// Create new event sender.
    pub fn new(sender: Sender<ReplEvent>) -> Self {
        Self { inner: sender }
    }

    /// Send event to engine (non-blocking).
    pub async fn send(&self, event: ReplEvent) -> Result<(), crate::ReplError> {
        self.inner
            .send(event)
            .await
            .map_err(|_| crate::ReplError::Channel("Event channel closed".to_string()))
    }
}

/// Event handler trait for custom processing.
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Process a REPL event.
    async fn handle(&self, event: ReplEvent) -> Result<(), crate::ReplError>;
}
