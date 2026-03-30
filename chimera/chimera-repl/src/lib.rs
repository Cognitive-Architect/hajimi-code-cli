//! Chimera REPL Engine - TUI-free command interaction layer
//!
//! ZeroTUI architecture: pure business logic with I/O injection.

use std::sync::Arc;

use codex_protocol::ThreadId;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

pub mod clock;
pub mod engine;
pub mod event;
pub mod io;
pub mod repl;
pub mod session;
pub mod state;
pub mod traits;

pub use clock::{Clock, MockClock, SystemTimeClock};
pub use engine::{EngineController, EngineState};
pub use event::{ReplEvent, ReplEventSender};
pub use io::{InputSource, MockInput, StdinInput};
pub use repl::ChimeraRepl;
pub use session::SessionState;
pub use state::{ReplState, Role, SessionMeta, TurnItem};
pub use traits::{ReplConfig, ReplEngineCore, ReplError, ReplResult};

/// Default ReplState type alias (SystemTimeClock).
pub type DefaultReplState = ReplState<SystemTimeClock>;

/// Core REPL engine state and event handling.
pub struct ReplEngine {
    pub thread_id: ThreadId,
    pub event_tx: ReplEventSender,
    pub session: Arc<RwLock<SessionState>>,
    pub config: ReplConfig,
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
