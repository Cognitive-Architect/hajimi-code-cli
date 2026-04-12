pub mod auto;
pub mod session;
pub mod dream;
pub mod scheduler;
// DEBT-HNSW-W34: HNSW模块临时禁用，Week 34重构
// pub mod hnsw;
pub mod batch_compute;

#[cfg(test)]
pub mod test_utils;

pub use auto::{AutoMemory, AutoEntry, AutoError};
pub use session::{SessionMemory, SessionEntry, SessionError};
pub use dream::{DreamMemory, DreamEntry, DreamError};
pub use scheduler::{MemoryScheduler, SchedulerError};
// pub use hnsw::{HnswIndex, Node, Neighbor, HnswError, EMBEDDING_DIM, M, EF_CONSTRUCTION};
