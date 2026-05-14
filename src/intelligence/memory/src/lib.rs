pub mod auto;
pub mod session;
pub mod types;
// NOTE: DreamMemory MVP uses hash-based embedding with cosine similarity.
pub mod dream;
pub mod scheduler;
pub mod sync;
pub mod sync_gateway;
pub mod sync_wrapper;
// DEBT-HNSW-W34: HNSW模块临时禁用，Week 34重构
// pub mod hnsw;
pub mod batch_compute;
pub mod cloud;
pub mod episodic;
pub mod graph;
pub mod graph_query;
pub mod memory_gateway;

#[cfg(test)]
pub mod stress;

#[cfg(test)]
pub mod test_utils;

pub use auto::{AutoEntry, AutoError, AutoMemory};
pub use dream::{DreamEntry, DreamError, DreamMemory};
pub use scheduler::{MemoryScheduler, SchedulerError};
pub use session::{SessionEntry, SessionError, SessionMemory};
pub use sync_gateway::{
    BlackboardSnapshot, GatewayEvent, MemoryTier, SyncGatewayError, SyncMemoryGateway, TierHealth,
};
pub use types::{MemoryEntry, MemoryLayer, MemoryLayerId, MemoryStorage, TokenCount};
// pub use hnsw::{HnswIndex, Node, Neighbor, HnswError, EMBEDDING_DIM, M, EF_CONSTRUCTION};
pub use cloud::{
    constant_time_verify, decrypt_chunk, derive_key, encrypt_chunk, generate_identity,
    verify_argon2_params, CloudError, CloudIdentity, CloudMemory, CloudSyncMeta, EncryptedChunk,
};
pub use episodic::{Episode, EpisodicError, EpisodicMemory};
pub use graph::{
    extract_entities, Edge, Entity, GraphError, GraphMemory, KnowledgeGraph, NerError, Node,
};
pub use graph_query::{
    bfs_traverse, dfs_traverse, find_paths, GraphError as QueryGraphError,
    KnowledgeGraph as QueryGraph, MemoryGateway, Path,
};
