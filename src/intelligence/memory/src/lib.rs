pub mod types;
pub mod auto;
pub mod session;
// NOTE: DreamMemory MVP uses hash-based embedding with cosine similarity.
pub mod dream;
pub mod scheduler;
pub mod sync;
pub mod sync_wrapper;
pub mod sync_gateway;
// DEBT-HNSW-W34: HNSW模块临时禁用，Week 34重构
// pub mod hnsw;
pub mod cloud;
pub mod batch_compute;
pub mod graph;
pub mod graph_query;
pub mod memory_gateway;
pub mod episodic;

#[cfg(test)]
pub mod stress;

#[cfg(test)]
pub mod test_utils;

pub use types::{MemoryEntry, MemoryLayer, MemoryLayerId, MemoryStorage, TokenCount};
pub use auto::{AutoMemory, AutoEntry, AutoError};
pub use session::{SessionMemory, SessionEntry, SessionError};
pub use dream::{DreamMemory, DreamEntry, DreamError};
pub use scheduler::{MemoryScheduler, SchedulerError};
pub use sync_gateway::{
    BlackboardSnapshot, GatewayEvent, MemoryTier, SyncGatewayError,
    SyncMemoryGateway, TierHealth,
};
// pub use hnsw::{HnswIndex, Node, Neighbor, HnswError, EMBEDDING_DIM, M, EF_CONSTRUCTION};
pub use graph::{GraphMemory, KnowledgeGraph, Entity, Node, Edge, extract_entities, NerError, GraphError};
pub use graph_query::{KnowledgeGraph as QueryGraph, Path, GraphError as QueryGraphError, bfs_traverse, dfs_traverse, find_paths, MemoryGateway};
pub use cloud::{CloudMemory, CloudIdentity, CloudError, EncryptedChunk, CloudSyncMeta, generate_identity, derive_key, verify_argon2_params, encrypt_chunk, decrypt_chunk, constant_time_verify};
pub use episodic::{EpisodicMemory, Episode, EpisodicError};
