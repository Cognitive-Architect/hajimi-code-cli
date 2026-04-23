pub mod auto;
pub use auto::{AutoMemory, AutoEntry, AutoError};

pub mod cloud;
pub use cloud::{CloudMemory, CloudMemoryEntry, CloudSyncMeta, CloudError};

pub mod graph_query;
pub use graph_query::{KnowledgeGraph, Path, GraphError, bfs_traverse, dfs_traverse, find_paths, MemoryGateway};

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
