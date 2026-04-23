//! Graph Search Memory - Zero-copy HNSW integration
use crate::graph::{KnowledgeGraph, Entity};
use crate::hnsw_optimized::HnswIndex;
use uuid::Uuid;

/// Graph vector search bridge
pub struct GraphSearchMemory {
    graph: KnowledgeGraph,
    index: HnswIndex,
}

impl GraphSearchMemory {
    pub fn new(graph: KnowledgeGraph) -> Self {
        Self { graph, index: HnswIndex::new() }
    }
    
    /// Index entity embeddings into HNSW
    pub fn index_entity(&mut self, entity: &Entity, embedding: Vec<f32>) {
        self.index.add_vector(embedding);
    }
    
    /// Search similar entities by vector
    pub fn search_similar(&self, query: &[f32], k: usize) -> Vec<usize> {
        self.index.search(query, k)
    }
    
    /// Zero-copy interface: get vector from index
    pub fn get_embedding(&self, idx: usize) -> Option<&[f32]> {
        self.index.get_vector(idx)
    }
}
