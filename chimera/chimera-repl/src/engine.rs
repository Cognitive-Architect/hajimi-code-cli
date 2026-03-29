//! REPL Engine core implementation.
//!
//! Provides the main event loop and command processing logic
//! without TUI dependencies.

use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

use crate::{
    ReplConfig, ReplEngineCore, ReplError, ReplEvent, ReplEventSender, ReplResult,
    SessionState,
};

/// Engine state machine for REPL lifecycle management.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Initial state before startup.
    Idle,
    /// Actively processing events.
    Running,
    /// Graceful shutdown in progress.
    ShuttingDown,
    /// Terminal state.
    Stopped,
}

/// Extended engine operations beyond basic lifecycle.
pub struct EngineController {
    /// Current engine state.
    pub state: Arc<RwLock<EngineState>>,
    /// Reference to session state.
    pub session: Arc<RwLock<SessionState>>,
}

impl EngineController {
    /// Create new controller with shared state.
    pub fn new(session: Arc<RwLock<SessionState>>) -> Self {
        Self {
            state: Arc::new(RwLock::new(EngineState::Idle)),
            session,
        }
    }

    /// Transition to running state.
    pub async fn start(&self) -> ReplResult<()> {
        let mut state = self.state.write().await;
        *state = EngineState::Running;
        info!("Engine transitioned to Running state");
        Ok(())
    }

    /// Transition to shutdown state.
    pub async fn stop(&self) -> ReplResult<()> {
        let mut state = self.state.write().await;
        *state = EngineState::ShuttingDown;
        debug!("Engine shutdown initiated");
        Ok(())
    }
}
