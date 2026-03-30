//! Chimera REPL - Pure business event loop, ZeroTUI architecture.
use std::pin::Pin;

use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::clock::Clock;
use crate::event::{EventHandler, ReplEvent};
use crate::state::{ReplState, Role};
use crate::{ReplConfig, ReplError, ReplResult};

/// ZeroTUI REPL implementation with I/O injection.
pub struct ChimeraRepl<C: Clock, R: AsyncWrite + Unpin> {
    state: ReplState<C>,
    config: ReplConfig,
    output: Pin<Box<R>>,
    event_rx: mpsc::Receiver<ReplEvent>,
    running: bool,
}

impl<C: Clock, R: AsyncWrite + Unpin> ChimeraRepl<C, R> {
    /// Create new REPL instance with injected I/O.
    pub fn new(
        state: ReplState<C>,
        config: ReplConfig,
        output: R,
        event_rx: mpsc::Receiver<ReplEvent>,
    ) -> Self {
        Self {
            state,
            config,
            output: Box::pin(output),
            event_rx,
            running: false,
        }
    }

    /// Main event loop - pure business logic, zero TUI.
    pub async fn run<H: EventHandler>(&mut self, handler: &mut H) -> ReplResult<()> {
        self.running = true;
        info!("ChimeraRepl ZeroTUI loop started");

        while self.running {
            match self.event_rx.recv().await {
                Some(event) => {
                    debug!(?event, "Processing REPL event");
                    if let Err(e) = handler.handle(event).await {
                        error!(?e, "Event handler error");
                    }
                }
                None => {
                    info!("Event channel closed, shutting down");
                    self.running = false;
                }
            }
        }

        self.output.flush().await.map_err(|e| ReplError::Channel(e.to_string()))?;
        info!("ChimeraRepl graceful shutdown complete");
        Ok(())
    }

    /// Graceful shutdown signal.
    pub fn shutdown(&mut self) {
        self.running = false;
        info!("Shutdown signal received");
    }

    /// Get mutable state reference.
    pub fn state_mut(&mut self) -> &mut ReplState<C> {
        &mut self.state
    }
}

/// Build REPL with stdin/stdout (convenience constructor).
pub fn build_stdio_repl<C: Clock>(
    state: ReplState<C>,
    config: ReplConfig,
) -> (ChimeraRepl<C, tokio::io::Stdout>, mpsc::Sender<ReplEvent>) {
    let (event_tx, event_rx) = mpsc::channel(1024);
    let repl = ChimeraRepl::new(state, config, tokio::io::stdout(), event_rx);
    (repl, event_tx)
}
