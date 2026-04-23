//! Chimera REPL Engine - TUI-free command interaction layer
//!
//! ZeroTUI architecture: pure business logic with I/O injection.

use codex_twist::thread::ThreadId;
use eventloop_adapter::{channel, rwlock, write, read, ArcRwLock};
use tracing::{debug, info};

pub mod eventloop_adapter;

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
    pub session: ArcRwLock<SessionState>,
    pub config: ReplConfig,
    pub running: ArcRwLock<bool>,
}

#[async_trait::async_trait]
impl ReplEngineCore for ReplEngine {
    async fn new(config: ReplConfig) -> ReplResult<Self> {
        let thread_id = config.thread_id.clone().unwrap_or_else(ThreadId::new);
        let (event_tx, _event_rx) = channel(1024);
        info!(?thread_id, "Initializing Chimera REPL engine");
        Ok(Self {
            thread_id,
            event_tx: ReplEventSender::new(event_tx),
            session: rwlock(SessionState::default()),
            config,
            running: rwlock(false),
        })
    }

    async fn run(&self) -> ReplResult<()> {
        *write(&self.running).await = true;
        info!("REPL engine started");
        while *read(&self.running).await {
            debug!("Processing REPL events");
        }
        Ok(())
    }

    async fn shutdown(&self) -> ReplResult<()> {
        *write(&self.running).await = false;
        info!("REPL engine shutdown complete");
        Ok(())
    }
}
