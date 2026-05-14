//! REPL Engine core implementation.
//!
//! Provides the main event loop and command processing logic
//! without TUI dependencies.

use crate::eventloop_adapter::{ArcRwLock, rwlock, write};
use tracing::{debug, info};

use crate::{ReplResult, SessionState};

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
    pub state: ArcRwLock<EngineState>,
    /// Reference to session state.
    pub session: ArcRwLock<SessionState>,
}

impl EngineController {
    /// Create new controller with shared state.
    pub fn new(session: ArcRwLock<SessionState>) -> Self {
        Self {
            state: rwlock(EngineState::Idle),
            session,
        }
    }

    /// Transition to running state.
    pub async fn start(&self) -> ReplResult<()> {
        let mut state = write(&self.state).await;
        *state = EngineState::Running;
        info!("Engine transitioned to Running state");
        Ok(())
    }

    /// Transition to shutdown state.
    pub async fn stop(&self) -> ReplResult<()> {
        let mut state = write(&self.state).await;
        *state = EngineState::ShuttingDown;
        debug!("Engine shutdown initiated");
        Ok(())
    }
}
