content = """use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::{MemoryEntry, MemoryLayer, MemoryLayerId, TokenCount};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use regex::Regex;
use aho_corasick::AhoCorasick;
use lru::LruCache;
use std::num::NonZeroUsize;

pub type EntityId = String;
pub type EdgeId = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType { Person, Org, Location, Concept, Product }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityNode { pub id: EntityId, pub name: String, pub entity_type: EntityType, pub created_at: DateTime<Utc> }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType { RelatedTo, PartOf, CreatedBy, LocatedIn }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationEdge { pub id: EdgeId, pub source_id: EntityId, pub target_id: EntityId, pub relation_type: RelationType }

#[derive(Clone, Debug, Default)]
pub struct EntityIndex { pub name_to_id: HashMap<String, EntityId>, pub type_to_ids: HashMap<EntityType, Vec<EntityId>> }

#[derive(Clone, Debug)]
pub struct GraphMemory {
    pub nodes: HashMap<EntityId, EntityNode>,
    pub edges: HashMap<EdgeId, RelationEdge>,
    pub index: EntityIndex,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphError { NotFound(String), Duplicate(String) }

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        match self { Self::NotFound(i) => write!(f, "Not found: {}", i), Self::Duplicate(i) => write!(f, "Duplicate: {}", i) } 
    }
}

impl std::error::Error for GraphError {}

impl GraphMemory {
    pub fn new() -> Self { Self { nodes: HashMap::new(), edges: HashMap::new(), index: EntityIndex::default() } }
    pub fn recall(&self, _query: &str) -> Result<Vec<EntityNode>, GraphError> { Ok(Vec::new()) }
    pub fn store(&mut self, _entry: MemoryEntry) -> Result<(), GraphError> { Ok(()) }
    pub fn search(&self, name: &str) -> Result<Vec<EntityNode>, GraphError> {
        match self.index.name_to_id.get(name) { 
            Some(id) => self.nodes.get(id).map(|n| vec![n.clone()]).ok_or_else(|| GraphError::NotFound(id.clone())), 
            None => Ok(Vec::new()) 
        }
    }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
}

impl Default for GraphMemory { fn default() -> Self { Self::new() } }

impl MemoryLayer for GraphMemory { 
    fn layer_id(&self) -> MemoryLayerId { MemoryLayerId::Graph } 
    fn capacity(&self) -> TokenCount { self.node_count() * 100 + self.edge_count() * 50 } 
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entity { pub id: Uuid, pub label: String, pub span: (usize, usize), pub confidence: f32 }

#[derive(Clone, Debug)]
pub struct Node { pub entity: Entity, pub relations: Vec<Edge> }

#[derive(Clone, Debug)]
pub struct Edge { pub target_id: Uuid, pub relation_type: String, pub weight: f32 }

pub struct KnowledgeGraph { 
    pub nodes: HashMap<Uuid, Node>, 
    pub edges: HashMap<(Uuid, Uuid), Edge>, 
    db: Arc<Mutex<rusqlite::Connection>>,
    cache: Arc<Mutex<LruCache<String, Vec<Entity>>>>,
}
"""
with open('F:/hajimi-code-cli/src/intelligence/memory/src/graph.rs', 'w') as f:
    f.write(content)
print('Part 1 written')
