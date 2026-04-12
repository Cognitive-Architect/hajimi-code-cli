pub mod auto;
pub use auto::{AutoMemory, AutoEntry, AutoError};

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
