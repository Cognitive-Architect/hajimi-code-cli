//! Hajimi - Vector Search and Memory System
#![deny(unsafe_code)]

pub mod db;
pub mod index;
pub mod knowledge;
pub mod intelligence;

pub use intelligence::memory::graph_query::{self, KnowledgeGraph, Path, GraphError, bfs_traverse, dfs_traverse, find_paths, MemoryGateway};
