pub mod types;
pub use types::{MemoryEntry, MemoryLayerId, MemoryStorage, TokenCount};

pub mod session;
pub use session::{SessionMemory, SessionEntry, SessionError};

pub mod auto;
pub mod dream;
pub mod graph;
pub mod hnsw;
pub use hnsw::HnswIndex;

pub use auto::AutoMemory;
pub use dream::DreamMemory;
pub use graph::GraphMemory;

use thiserror::Error;

pub const MAX_SESSION_TOKENS: usize = 4_000;

pub trait MemoryLayer {
    fn persist(&self) -> Result<(), MemoryError>;
    fn load(&mut self) -> Result<(), MemoryError>;
    fn search(&self, query: &str) -> Vec<MemoryEntry>;
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    #[error("Layer not initialized")]
    NotInitialized,
}

impl MemoryLayer for SessionMemory {
    fn persist(&self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn load(&mut self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let _ = query;
        Vec::new()
    }
}

impl MemoryLayer for AutoMemory {
    fn persist(&self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn load(&mut self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let _ = query;
        Vec::new()
    }
}

impl MemoryLayer for DreamMemory {
    fn persist(&self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn load(&mut self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let _ = query;
        Vec::new()
    }
}

impl MemoryLayer for GraphMemory {
    fn persist(&self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn load(&mut self) -> Result<(), MemoryError> {
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let _ = query;
        Vec::new()
    }
}
