//! Chimera REPL - Pure business event loop, ZeroTUI architecture.
use std::pin::Pin;

use tokio::io::AsyncWrite;
use crate::eventloop_adapter::EventReceiver;
use tracing::{debug, error, info};

use crate::clock::Clock;
use crate::event::{EventHandler, ReplEvent};
use crate::io::InputSource;
use crate::state::ReplState;
use crate::{ReplConfig, ReplResult};

/// ZeroTUI REPL with I/O injection (Clock + InputSource + AsyncWrite).
pub struct ChimeraRepl<C: Clock, I: InputSource, R: AsyncWrite + Unpin> {
    state: ReplState<C>,
    input: I,
    #[allow(dead_code)]
    output: Pin<Box<R>>,
    event_rx: EventReceiver<ReplEvent>,
    #[allow(dead_code)]
    config: ReplConfig,
    running: bool,
}

impl<C: Clock, I: InputSource, R: AsyncWrite + Unpin> ChimeraRepl<C, I, R> {
    /// Create new REPL with injected I/O.
    pub fn new(
        state: ReplState<C>,
        input: I,
        output: R,
        event_rx: EventReceiver<ReplEvent>,
        config: ReplConfig,
    ) -> Self {
        Self { state, input, output: Box::pin(output), event_rx, config, running: false }
    }

    /// Main event loop - pure business logic.
    pub async fn run<H: EventHandler>(&mut self, handler: &mut H) -> ReplResult<()> {
        self.running = true;
        info!("ChimeraRepl ZeroTUI loop started");
        while self.running {
            tokio::select! {
                event = self.event_rx.recv() => match event {
                    Some(e) => { if let Err(err) = handler.handle(e).await { error!(?err, "Handler error"); } }
                    None => { info!("Channel closed"); self.running = false; }
                },
                line = self.input.read_line() => match line {
                    Some(input) => { debug!(len = input.len(), "Input received"); }
                    None => { info!("Input EOF"); self.running = false; }
                },
            }
        }
        info!("ChimeraRepl shutdown");
        Ok(())
    }

    /// Graceful shutdown.
    pub fn shutdown(&mut self) { self.running = false; info!("Shutdown signal"); }

    /// Get mutable state.
    pub fn state_mut(&mut self) -> &mut ReplState<C> { &mut self.state }
}
