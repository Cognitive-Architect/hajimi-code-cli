pub mod types;
pub use types::{MemoryEntry, MemoryLayerId, MemoryStorage, TokenCount, MemoryLayer};

pub mod cloud;
pub use cloud::{CloudMemory, CloudIdentity, CloudError, CloudSyncMeta, EncryptedChunk};
pub use cloud::{generate_identity, encrypt_chunk, decrypt_chunk, derive_key, verify_argon2_params};

pub mod session;
pub use session::{SessionMemory, SessionEntry, SessionError};

pub mod auto;
pub mod dream;
pub mod graph;
pub mod hnsw;
pub use hnsw::HnswIndex;

pub use auto::AutoMemory;
pub use dream::DreamMemory;
pub use graph::{
    GraphMemory, KnowledgeGraph, Entity, Node, Edge,
    extract_entities, NerError, GraphError, EntityType,
    EntityNode, RelationEdge, RelationType, EntityIndex,
    EntityId, EdgeId
};

// B-05/A: Graph NER + 知识图谱模块导出
// - KnowledgeGraph: 图存储核心结构
// - extract_entities: Regex+Heuristic混合NER
// - Entity: 实体节点类型 (id, label, span, confidence)
// - NerError: NER错误类型
pub type GraphResult<T> = Result<T, GraphError>;
pub type EntityResult = Result<Vec<Entity>, NerError>;

/// Graph模块常量定义
pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.3;
pub const MAX_ENTITIES_PER_QUERY: usize = 1000;

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
