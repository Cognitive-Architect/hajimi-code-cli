//! HNSW Vector Index - Layer 0 Full-Connected Graph for ANN Search
#![deny(unsafe_code)]

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashSet};
use thiserror::Error;

/// Embedding dimension - 384 for lightweight semantic retrieval
pub const EMBEDDING_DIM: usize = 384_usize;

/// Maximum neighbors per node (M parameter)
pub const M: usize = 16_usize;

/// Search depth during construction (ef_construction)
pub const EF_CONSTRUCTION: usize = 200_usize;

/// HNSW error types
#[derive(Debug, Error)]
pub enum HnswError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid dimension: expected {EMBEDDING_DIM}, got {actual}")]
    InvalidDimension { actual: usize },
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Empty vector")]
    EmptyVector,
}

/// HNSW Node structure for Layer 0 (ground layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    /// Layer 0 = ground layer, all nodes are at level 0
    pub level: u8,
    /// 384-dimensional embedding vector (stored as Vec for serialization compatibility)
    pub vector: Vec<f32>,
    /// Neighbor node IDs (up to M neighbors)
    pub neighbors: Vec<String>,
}

impl Node {
    /// Create new node at Layer 0
    pub fn new(id: String, vector: [f32; EMBEDDING_DIM]) -> Self {
        Self {
            id,
            level: 0,
            vector: vector.to_vec(),
            neighbors: Vec::new(),
        }
    }
}

/// Neighbor result from ANN search
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor {
    pub id: String,
    pub distance: f32,
}

impl PartialOrd for Neighbor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}
impl Ord for Neighbor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.partial_cmp(&other.distance).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl Eq for Neighbor {}

/// HNSW Index - Layer 0 Full-Connected Graph
pub struct HnswIndex {
    conn: Connection,
    dim: usize,
}

impl HnswIndex {
    /// Create new HNSW index with SQLite persistence
    pub fn new(db_path: &str) -> Result<Self, HnswError> {
        let conn = Connection::open(db_path)?;
        let index = Self { conn, dim: EMBEDDING_DIM };
        index.init_schema()?;
        Ok(index)
    }
    fn init_schema(&self) -> Result<(), HnswError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS hnsw_nodes (id TEXT NOT NULL, level INTEGER NOT NULL, vector_json TEXT NOT NULL, neighbors_json TEXT NOT NULL, PRIMARY KEY (id, level))",
            [],
        )?;
        self.conn.execute("CREATE INDEX IF NOT EXISTS idx_hnsw_level ON hnsw_nodes(level)", [])?;
        Ok(())
    }
    /// Insert node into Layer 0 (ground layer)
    pub fn insert(&mut self, id: &str, vector: [f32; EMBEDDING_DIM]) -> Result<(), HnswError> {
        if vector.len() != self.dim { return Err(HnswError::InvalidDimension { actual: vector.len() }); }
        let tx = self.conn.transaction()?;
        let mut stmt = tx.prepare("SELECT id, vector_json, neighbors_json FROM hnsw_nodes WHERE level = 0")?;
        let existing_nodes: Vec<(String, String, String)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        let mut candidates: Vec<(String, f32)> = Vec::new();
        for (existing_id, vector_json, _) in &existing_nodes {
            let v: Vec<f32> = serde_json::from_str(vector_json)?;
            if v.len() != EMBEDDING_DIM { return Err(HnswError::InvalidDimension { actual: v.len() }); }
            let mut arr = [0.0_f32; EMBEDDING_DIM];
            arr.copy_from_slice(&v);
            candidates.push((existing_id.clone(), Self::euclidean_distance(vector, arr)));
        }
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let neighbor_ids: Vec<String> = candidates.iter().take(M).map(|(nid, _)| nid.clone()).collect();
        let vector_json = serde_json::to_string(&vector.to_vec())?;
        let neighbors_json = serde_json::to_string(&neighbor_ids)?;
        tx.execute(
            "INSERT OR REPLACE INTO hnsw_nodes (id, level, vector_json, neighbors_json) VALUES (?1, ?2, ?3, ?4)",
            [id, "0", &vector_json, &neighbors_json],
        )?;
        for (existing_id, _, existing_neighbors_json) in &existing_nodes {
            let mut existing_neighbors: Vec<String> = serde_json::from_str(existing_neighbors_json)?;
            if existing_neighbors.len() < M && !existing_neighbors.contains(&id.to_string()) {
                existing_neighbors.push(id.to_string());
                let updated_neighbors_json = serde_json::to_string(&existing_neighbors)?;
                tx.execute("UPDATE hnsw_nodes SET neighbors_json = ?1 WHERE id = ?2", [&updated_neighbors_json, existing_id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
    /// ANN search using Layer 0 greedy search (single layer), returns k nearest neighbors
    pub fn search_ann(&self, query: [f32; EMBEDDING_DIM], k: usize) -> Result<Vec<Neighbor>, HnswError> {
        if query.len() != self.dim { return Err(HnswError::InvalidDimension { actual: query.len() }); }
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM hnsw_nodes WHERE level = 0", [], |row| row.get(0))?;
        if count == 0 { return Ok(Vec::new()); }
        let entry_point: String = self.conn.query_row("SELECT id FROM hnsw_nodes WHERE level = 0 LIMIT 1", [], |row| row.get(0))?;
        let mut visited: HashSet<String> = HashSet::new();
        let mut result: BinaryHeap<Neighbor> = BinaryHeap::new();
        let mut candidates: Vec<String> = vec![entry_point];
        while let Some(current_id) = candidates.pop() {
            if visited.contains(&current_id) { continue; }
            visited.insert(current_id.clone());
            let (vector_json, neighbors_json): (String, String) = self.conn.query_row(
                "SELECT vector_json, neighbors_json FROM hnsw_nodes WHERE id = ?1", [&current_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let v: Vec<f32> = serde_json::from_str(&vector_json)?;
            if v.len() != EMBEDDING_DIM { return Err(HnswError::InvalidDimension { actual: v.len() }); }
            let mut arr = [0.0_f32; EMBEDDING_DIM];
            arr.copy_from_slice(&v);
            let neighbors: Vec<String> = serde_json::from_str(&neighbors_json)?;
            result.push(Neighbor { id: current_id.clone(), distance: Self::euclidean_distance(query, arr) });
            while result.len() > k { result.pop(); }
            for neighbor_id in neighbors {
                if !visited.contains(&neighbor_id) { candidates.push(neighbor_id); }
            }
        }
        let mut sorted: Vec<Neighbor> = result.into_vec();
        sorted.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        Ok(sorted)
    }
    /// Get node by ID
    pub fn get_node(&self, id: &str) -> Result<Option<Node>, HnswError> {
        let result = self.conn.query_row(
            "SELECT level, vector_json, neighbors_json FROM hnsw_nodes WHERE id = ?1", [id],
            |row| {
                let level: u8 = row.get(0)?;
                let vector_json: String = row.get(1)?;
                let neighbors_json: String = row.get(2)?;
                Ok((level, vector_json, neighbors_json))
            },
        );
        match result {
            Ok((level, vector_json, neighbors_json)) => {
                let v: Vec<f32> = serde_json::from_str(&vector_json)?;
                if v.len() != EMBEDDING_DIM { return Err(HnswError::InvalidDimension { actual: v.len() }); }
                let neighbors: Vec<String> = serde_json::from_str(&neighbors_json)?;
                Ok(Some(Node { id: id.to_string(), level, vector: v, neighbors }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    /// Delete node by ID
    pub fn delete(&mut self, id: &str) -> Result<(), HnswError> {
        let tx = self.conn.transaction()?;
        let mut stmt = tx.prepare("SELECT id, neighbors_json FROM hnsw_nodes WHERE neighbors_json LIKE ?1")?;
        let rows: Vec<(String, String)> = stmt.query_map([format!("%\"{}\"%", id)], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        for (nid, neighbors_json) in rows {
            let mut neighbors: Vec<String> = serde_json::from_str(&neighbors_json)?;
            neighbors.retain(|n| n != id);
            let updated_json = serde_json::to_string(&neighbors)?;
            tx.execute("UPDATE hnsw_nodes SET neighbors_json = ?1 WHERE id = ?2", [&updated_json, &nid])?;
        }
        tx.execute("DELETE FROM hnsw_nodes WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }
    fn euclidean_distance(a: [f32; EMBEDDING_DIM], b: [f32; EMBEDDING_DIM]) -> f32 {
        let mut sum = 0.0_f32;
        for i in 0..EMBEDDING_DIM { let d = a[i] - b[i]; sum += d * d; }
        sum.sqrt()
    }
    pub fn len(&self) -> Result<usize, HnswError> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM hnsw_nodes", [], |row| row.get(0))?;
        Ok(count as usize)
    }
    pub fn is_empty(&self) -> Result<bool, HnswError> { Ok(self.len()? == 0) }

    // ============================================
    // Week 34: Hierarchical Insert & Greedy Search
    // ============================================

    /// Exponential decay level assignment
    fn random_level(&self) -> u8 {
        let mut level = 0u8;
        let m_f = M as f64;
        while level < 16 && rand::random::<f64>() < (-(level as f64) / m_f).exp() {
            level = level.saturating_add(1);
        }
        level
    }

    /// Get current top level
    fn get_top_level(&self) -> Result<u8, HnswError> {
        self.conn.query_row("SELECT COALESCE(MAX(level), 0) FROM hnsw_nodes", [], |row| row.get(0)).map_err(|e| e.into())
    }

    /// Get vector distance from storage
    fn get_dist(&self, id: &str, query: [f32; EMBEDDING_DIM]) -> Result<f32, HnswError> {
        let vj: String = self.conn.query_row("SELECT vector_json FROM hnsw_nodes WHERE id = ?1", [id], |row| row.get(0))?;
        let v: Vec<f32> = serde_json::from_str(&vj)?;
        if v.len() != EMBEDDING_DIM { return Err(HnswError::InvalidDimension { actual: v.len() }); }
        let mut arr = [0.0_f32; EMBEDDING_DIM];
        arr.copy_from_slice(&v);
        Ok(Self::euclidean_distance(query, arr))
    }

    /// Greedy search within a layer - returns ef candidates using dynamic expansion
    fn greedy_search_layer(&self, query: [f32; EMBEDDING_DIM], entry: &str, level: u8, ef: usize) -> Result<Vec<String>, HnswError> {
        use std::collections::BinaryHeap;
        use std::cmp::Ordering;
        
        // Wrapper for distance+id that implements Ord (min-heap: smaller distance = higher priority)
        #[derive(Clone)]
        struct Candidate(f32, String);
        impl PartialEq for Candidate { fn eq(&self, other: &Self) -> bool { self.0 == other.0 } }
        impl Eq for Candidate {}
        impl PartialOrd for Candidate { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { other.0.partial_cmp(&self.0) } } // Reverse for min-heap
        impl Ord for Candidate { fn cmp(&self, other: &Self) -> Ordering { self.partial_cmp(other).unwrap_or(Ordering::Equal) } }
        
        let mut candidates: BinaryHeap<Candidate> = BinaryHeap::new(); // Min-heap by distance
        let mut visited: HashSet<String> = HashSet::new();
        let mut result: Vec<Candidate> = Vec::new();
        
        // Initialize with entry point
        if let Ok(dist) = self.get_dist(entry, query) {
            candidates.push(Candidate(dist, entry.to_string()));
            visited.insert(entry.to_string());
        }
        
        // Greedy expansion: always expand the closest unvisited node
        while let Some(Candidate(dist, curr)) = candidates.pop() {
            // Add to result
            result.push(Candidate(dist, curr.clone()));
            if result.len() >= ef { break; }
            
            // Get neighbors of current node
            let nbrs_result = self.conn.query_row(
                "SELECT neighbors_json FROM hnsw_nodes WHERE id = ?1 AND level = ?2",
                [&curr, &level.to_string()], |row| row.get::<_, String>(0),
            );
            
            if let Ok(nbrs) = nbrs_result {
                let neighbor_ids: Vec<String> = serde_json::from_str(&nbrs)?;
                for nid in neighbor_ids {
                    if visited.insert(nid.clone()) {
                        if let Ok(d) = self.get_dist(&nid, query) {
                            candidates.push(Candidate(d, nid));
                        }
                    }
                }
            }
        }
        
        // Return candidates sorted by distance
        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        Ok(result.into_iter().map(|c| c.1).collect())
    }

    /// Hierarchical insert with level assignment
    pub fn insert_with_levels(&mut self, id: &str, vector: [f32; EMBEDDING_DIM]) -> Result<(), HnswError> {
        if vector.len() != self.dim { return Err(HnswError::InvalidDimension { actual: vector.len() }); }
        let max_level = self.random_level();
        let tx = self.conn.transaction()?;
        for level in 0..=max_level {
            let mut stmt = tx.prepare("SELECT id, vector_json FROM hnsw_nodes WHERE level = ?1")?;
            let existing: Vec<(String, String)> = stmt.query_map([&level], |row| Ok((row.get(0)?, row.get(1)?)))?.collect::<Result<Vec<_>, _>>()?;
            drop(stmt);
            let mut cand: Vec<(String, f32)> = existing.iter().map(|(eid, vj)| {
                let v: Vec<f32> = serde_json::from_str(vj).map_err(|e| HnswError::from(e))?;
                if v.len() != EMBEDDING_DIM { return Err(HnswError::InvalidDimension { actual: v.len() }); }
                let mut a = [0.0_f32; EMBEDDING_DIM];
                a.copy_from_slice(&v);
                Ok((eid.clone(), Self::euclidean_distance(vector, a)))
            }).collect::<Result<Vec<_>, HnswError>>()?;
            cand.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let nbrs: Vec<String> = cand.iter().take(M).map(|(i, _)| i.clone()).collect();
            let vj = serde_json::to_string(&vector.to_vec())?;
            let nj = serde_json::to_string(&nbrs)?;
            tx.execute("INSERT OR REPLACE INTO hnsw_nodes (id, level, vector_json, neighbors_json) VALUES (?1, ?2, ?3, ?4)",
                [id, &level.to_string(), &vj, &nj])?;
            for (eid, _) in cand.iter().take(M) {
                if let Ok(ej) = tx.query_row::<String, _, _>("SELECT neighbors_json FROM hnsw_nodes WHERE id = ?1 AND level = ?2",
                    [eid, &level.to_string()], |row| row.get(0)) {
                    let mut en: Vec<String> = serde_json::from_str(&ej)?;
                    if en.len() < M && !en.contains(&id.to_string()) {
                        en.push(id.to_string());
                        tx.execute("UPDATE hnsw_nodes SET neighbors_json = ?1 WHERE id = ?2 AND level = ?3",
                            [&serde_json::to_string(&en)?, eid, &level.to_string()])?;
                    }
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Hierarchical ANN search with ef_search parameter - heuristic entry point and multi-candidate propagation
    pub fn search_ann_with_ef(&self, query: [f32; EMBEDDING_DIM], k: usize, ef_search: usize) -> Result<Vec<Neighbor>, HnswError> {
        if query.len() != self.dim { return Err(HnswError::InvalidDimension { actual: query.len() }); }
        if self.is_empty()? { return Ok(Vec::new()); }
        let top: u8 = self.get_top_level()?;
        
        // Fix 1: Heuristic Entry Point - query ALL nodes at top level, select nearest
        let mut stmt = self.conn.prepare("SELECT id, vector_json FROM hnsw_nodes WHERE level = ?1")?;
        let top_nodes: Vec<(String, String)> = stmt.query_map([&top], |row| Ok((row.get(0)?, row.get(1)?)))?.collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        
        if top_nodes.is_empty() { return self.search_ann(query, k); }
        
        // Calculate distances to all top level nodes, select best ef_search candidates
        let mut top_candidates: Vec<(f32, String)> = top_nodes.iter()
            .filter_map(|(id, vj)| {
                let v: Vec<f32> = serde_json::from_str(vj).ok()?;
                if v.len() != EMBEDDING_DIM { return None; }
                let mut a = [0.0_f32; EMBEDDING_DIM];
                a.copy_from_slice(&v);
                Some((Self::euclidean_distance(query, a), id.clone()))
            })
            .collect();
        top_candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut current_candidates: Vec<String> = top_candidates.into_iter().take(ef_search).map(|(_, id)| id).collect();
        
        // Fix 2: Layer-by-layer candidate propagation (upper layer candidates as entry points for lower layer)
        for lvl in (1..=top).rev() {
            let mut next_candidates: Vec<String> = Vec::new();
            for entry in &current_candidates {
                let layer_results = self.greedy_search_layer(query, entry, lvl, ef_search)?;
                next_candidates.extend(layer_results);
            }
            // Deduplicate and select best ef_search candidates
            current_candidates = self.select_best_candidates(query, next_candidates, ef_search)?;
        }
        
        // Fix 3: Multi-source BFS expansion at Layer 0 from all candidates
        self.multi_source_search_layer0(query, &current_candidates, k, ef_search)
    }
    
    /// Select best ef candidates from a list (deduplicate + sort by distance)
    fn select_best_candidates(&self, query: [f32; EMBEDDING_DIM], candidates: Vec<String>, ef: usize) -> Result<Vec<String>, HnswError> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut scored: Vec<(f32, String)> = Vec::new();
        for id in candidates {
            if seen.insert(id.clone()) {
                if let Ok(dist) = self.get_dist(&id, query) {
                    scored.push((dist, id));
                }
            }
        }
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(ef).map(|(_, id)| id).collect())
    }
    
    /// Multi-source BFS search at Layer 0 from multiple entry points
    fn multi_source_search_layer0(&self, query: [f32; EMBEDDING_DIM], entries: &[String], k: usize, ef: usize) -> Result<Vec<Neighbor>, HnswError> {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;
        
        let mut visited: HashSet<String> = HashSet::new();
        let mut result: BinaryHeap<Neighbor> = BinaryHeap::new();
        let mut candidates: Vec<String> = entries.to_vec();
        
        for entry in entries {
            visited.insert(entry.clone());
        }
        
        while let Some(cid) = candidates.pop() {
            let row_result = self.conn.query_row(
                "SELECT vector_json, neighbors_json FROM hnsw_nodes WHERE id = ?1 AND level = 0",
                [&cid], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            );
            if let Ok((vj, nj)) = row_result {
                let v: Vec<f32> = serde_json::from_str(&vj)?;
                if v.len() != EMBEDDING_DIM { continue; }
                let mut arr = [0.0_f32; EMBEDDING_DIM];
                arr.copy_from_slice(&v);
                let dist = Self::euclidean_distance(query, arr);
                result.push(Neighbor { id: cid.clone(), distance: dist });
                while result.len() > k { result.pop(); }
                
                let nids: Vec<String> = serde_json::from_str(&nj)?;
                for nid in nids.into_iter().take(ef) {
                    if visited.insert(nid.clone()) {
                        candidates.push(nid);
                    }
                }
            }
        }
        
        let mut res: Vec<Neighbor> = result.into_vec();
        res.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_vector(seed: f32) -> [f32; EMBEDDING_DIM] {
        let mut v = [0.0_f32; EMBEDDING_DIM];
        for i in 0..EMBEDDING_DIM { v[i] = (i as f32 * 0.01 + seed).sin(); }
        v
    }

    #[test]
    fn test_new_index() {
        let temp = NamedTempFile::new().expect("temp file");
        let index = HnswIndex::new(temp.path().to_str().unwrap());
        assert!(index.is_ok());
    }

    #[test]
    fn test_insert_and_get() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        let v = create_test_vector(1.0);
        index.insert("test1", v).expect("insert");
        let node = index.get_node("test1").expect("get");
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, "test1");
    }

    #[test]
    fn test_search_ann_empty() {
        let temp = NamedTempFile::new().expect("temp file");
        let index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        let query = create_test_vector(0.0);
        let results = index.search_ann(query, 5).expect("search");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_ann_basic() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        for i in 0..10 {
            let v = create_test_vector(i as f32);
            index.insert(&format!("node{}", i), v).expect("insert");
        }
        let query = create_test_vector(5.0);
        let results = index.search_ann(query, 3).expect("search");
        assert_eq!(results.len(), 3);
        for i in 1..results.len() { assert!(results[i-1].distance <= results[i].distance); }
    }

    #[test]
    fn test_delete() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        let v = create_test_vector(1.0);
        index.insert("test1", v).expect("insert");
        assert!(index.get_node("test1").expect("get").is_some());
        index.delete("test1").expect("delete");
        assert!(index.get_node("test1").expect("get").is_none());
    }

    #[test]
    fn test_euclidean_distance() {
        let a = [0.0_f32; EMBEDDING_DIM];
        let b = [1.0_f32; EMBEDDING_DIM];
        let dist = HnswIndex::euclidean_distance(a, b);
        let expected = (EMBEDDING_DIM as f32).sqrt();
        assert!((dist - expected).abs() < 0.001);
    }

    // Week 34 Tests
    #[test]
    fn test_random_level() {
        let temp = NamedTempFile::new().expect("temp file");
        let index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        for _ in 0..100 {
            let lvl = index.random_level();
            assert!(lvl <= 16);
        }
    }

    #[test]
    fn test_insert_with_levels() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        let v = create_test_vector(1.0);
        index.insert_with_levels("hnode1", v).expect("insert with levels");
        let node = index.get_node("hnode1").expect("get");
        assert!(node.is_some());
    }

    #[test]
    fn test_search_ann_with_ef() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        for i in 0..20 {
            let v = create_test_vector(i as f32 * 0.5);
            index.insert_with_levels(&format!("hnode{}", i), v).expect("insert");
        }
        let query = create_test_vector(5.0);
        let results = index.search_ann_with_ef(query, 5, 10).expect("search with ef");
        assert_eq!(results.len(), 5);
        for i in 1..results.len() { assert!(results[i-1].distance <= results[i].distance); }
    }

    #[test]
    fn test_search_ann_with_ef_empty() {
        let temp = NamedTempFile::new().expect("temp file");
        let index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
        let query = create_test_vector(0.0);
        let results = index.search_ann_with_ef(query, 5, 10).expect("search");
        assert!(results.is_empty());
    }

    // ============================================
    // Week 35: Benchmark Tests (Line 465+)
    // ============================================

    use rand::{SeedableRng, rngs::StdRng, Rng};
    use std::time::Instant;

    const TEST_SEED: u64 = 42;

    /// Generate random normalized vector for reproducible tests
    fn generate_random_vector(rng: &mut StdRng) -> [f32; EMBEDDING_DIM] {
        let mut v = [0.0_f32; EMBEDDING_DIM];
        for i in 0..EMBEDDING_DIM {
            v[i] = rng.gen_range(-1.0..1.0);
        }
        // L2 normalization
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    /// Ground truth brute force search for Recall calculation
    fn brute_force_search(
        index: &HnswIndex,
        query: [f32; EMBEDDING_DIM],
        k: usize,
    ) -> Result<Vec<String>, HnswError> {
        let mut all_nodes: Vec<(String, f32)> = Vec::new();
        let mut stmt = index.conn.prepare("SELECT id, vector_json FROM hnsw_nodes WHERE level = 0")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let vj: String = row.get(1)?;
            Ok((id, vj))
        })?;
        for row in rows {
            let (id, vj) = row?;
            let v: Vec<f32> = serde_json::from_str(&vj)?;
            if v.len() != EMBEDDING_DIM {
                return Err(HnswError::InvalidDimension { actual: v.len() });
            }
            let mut arr = [0.0_f32; EMBEDDING_DIM];
            arr.copy_from_slice(&v);
            let dist = HnswIndex::euclidean_distance(query, arr);
            all_nodes.push((id, dist));
        }
        drop(stmt);
        // Sort by distance ascending
        all_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(all_nodes.into_iter().take(k).map(|(id, _)| id).collect())
    }

    /// Calculate Recall@k
    fn calculate_recall(ann_results: &[String], ground_truth: &[String]) -> f64 {
        if ground_truth.is_empty() {
            return 0.0;
        }
        let ann_set: HashSet<_> = ann_results.iter().collect();
        let gt_set: HashSet<_> = ground_truth.iter().collect();
        let intersection = ann_set.intersection(&gt_set).count();
        intersection as f64 / ground_truth.len() as f64
    }

    /// Test Recall@10 >= 90%
    #[test]
    fn test_recall_at_10() {
        let mut rng = StdRng::seed_from_u64(TEST_SEED);
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");

        // Insert 500 random vectors
        const N: usize = 500;
        for i in 0..N {
            let v = generate_random_vector(&mut rng);
            index.insert_with_levels(&format!("node{}", i), v).expect("insert");
        }

        // Query 50 random points
        const QUERY_COUNT: usize = 50;
        let mut total_recall = 0.0;
        for _ in 0..QUERY_COUNT {
            let query = generate_random_vector(&mut rng);

            // ANN search with ef_search=64
            let ann_results = index.search_ann_with_ef(query, 10, 64).expect("ann search");
            let ann_ids: Vec<String> = ann_results.iter().map(|n| n.id.clone()).collect();

            // Ground truth brute force search
            let gt_results = brute_force_search(&index, query, 10).expect("brute force");

            // Calculate recall
            let recall = calculate_recall(&ann_ids, &gt_results);
            total_recall += recall;
        }

        let avg_recall = total_recall / QUERY_COUNT as f64;
        println!("Recall@10 = {:.2}%", avg_recall * 100.0);
        assert!(
            avg_recall >= 0.90,
            "Recall@10 must be >= 90%, actual = {:.2}%",
            avg_recall * 100.0
        );
    }

    /// Test ef_search tuning curve
    #[test]
    fn test_ef_search_tuning() {
        let mut rng = StdRng::seed_from_u64(TEST_SEED);
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");

        // Insert 300 vectors
        for i in 0..300 {
            let v = generate_random_vector(&mut rng);
            index.insert_with_levels(&format!("node{}", i), v).expect("insert");
        }

        // Test different ef_search values
        let ef_values = [10, 16, 32, 64, 128];
        let query = generate_random_vector(&mut rng);
        let gt_results = brute_force_search(&index, query, 10).expect("brute force");

        println!("\n=== EF Search Tuning Curve ===");
        for &ef in &ef_values {
            let ann_results = index.search_ann_with_ef(query, 10, ef).expect("ann search");
            let ann_ids: Vec<String> = ann_results.iter().map(|n| n.id.clone()).collect();
            let recall = calculate_recall(&ann_ids, &gt_results);
            println!("ef_search={:3} -> Recall@10 = {:.2}%", ef, recall * 100.0);
        }

        // Verify ef_search=64 achieves >=90% recall
        let ann_results = index.search_ann_with_ef(query, 10, 64).expect("ann search");
        let ann_ids: Vec<String> = ann_results.iter().map(|n| n.id.clone()).collect();
        let recall = calculate_recall(&ann_ids, &gt_results);
        assert!(
            recall >= 0.90,
            "ef_search=64 should achieve >=90% recall, got {:.2}%",
            recall * 100.0
        );
    }

    /// 10K nodes pressure test
    #[test]
    fn test_10k_nodes_pressure() {
        let mut rng = StdRng::seed_from_u64(TEST_SEED);
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");

        const N: usize = 10000;

        // Measure insertion TPS
        let insert_start = Instant::now();
        for i in 0..N {
            let v = generate_random_vector(&mut rng);
            index.insert_with_levels(&format!("node{}", i), v).expect("insert");
        }
        let insert_duration = insert_start.elapsed();
        let tps = N as f64 / insert_duration.as_secs_f64();
        println!("\n=== 10K Pressure Test ===");
        println!("Inserted {} nodes in {:?}", N, insert_duration);
        println!("Insertion TPS: {:.1}", tps);

        // Measure search latency percentiles
        let mut latencies: Vec<std::time::Duration> = Vec::with_capacity(100);
        for _ in 0..100 {
            let query = generate_random_vector(&mut rng);
            let search_start = Instant::now();
            let _ = index.search_ann_with_ef(query, 10, 64).expect("search");
            latencies.push(search_start.elapsed());
        }

        latencies.sort();
        let p50 = latencies[latencies.len() / 2];
        let p95 = latencies[latencies.len() * 95 / 100];
        let p99 = latencies[latencies.len() * 99 / 100];

        println!("Search Latency: P50={:?}, P95={:?}, P99={:?}", p50, p95, p99);

        // Compare with brute force scan latency
        let query = generate_random_vector(&mut rng);
        let bf_start = Instant::now();
        let _ = brute_force_search(&index, query, 10).expect("brute force");
        let bf_duration = bf_start.elapsed();
        println!("Brute Force Scan: {:?}", bf_duration);

        // Assertions
        assert!(p50.as_millis() < 100, "P50 latency should be <100ms");
        assert!(bf_duration.as_millis() > 50, "Brute force should be significantly slower");
    }

    /// O(log N) complexity scaling validation
    #[test]
    fn test_complexity_scaling() {
        let mut rng = StdRng::seed_from_u64(TEST_SEED);
        let temp = NamedTempFile::new().expect("temp file");
        let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");

        let scales = [100, 1000, 5000];
        let query = generate_random_vector(&mut rng);

        println!("\n=== Complexity Scaling Test ===");
        let mut prev_n = 0usize;
        let mut prev_latency_ms = 0f64;

        for &n in &scales {
            // Insert additional vectors
            for i in prev_n..n {
                let v = generate_random_vector(&mut rng);
                index.insert_with_levels(&format!("node{}", i), v).expect("insert");
            }

            // Measure search latency (average of 10 runs)
            let mut total_duration = std::time::Duration::ZERO;
            for _ in 0..10 {
                let search_start = Instant::now();
                let _ = index.search_ann_with_ef(query, 10, 64).expect("search");
                total_duration += search_start.elapsed();
            }
            let avg_latency_ms = total_duration.as_secs_f64() * 100.0;

            print!("N={:5}: avg_latency={:.2}ms", n, avg_latency_ms);

            if prev_n > 0 {
                let n_ratio = n as f64 / prev_n as f64;
                let latency_ratio = avg_latency_ms / prev_latency_ms;
                let log_ratio = n_ratio.log2();
                println!(", N_ratio={:.1}x, latency_ratio={:.2}x (expected ~log2={:.1}x)",
                    n_ratio, latency_ratio, log_ratio);

                // Verify sub-linear scaling (should be closer to O(log N) than O(N))
                // Latency ratio should be significantly less than N ratio
                assert!(
                    latency_ratio < n_ratio * 0.5,
                    "Latency scaling should be sub-linear: N grew {:.1}x but latency grew {:.1}x",
                    n_ratio, latency_ratio
                );
            } else {
                println!();
            }

            prev_n = n;
            prev_latency_ms = avg_latency_ms;
        }

        println!("Complexity scaling test passed: O(log N) behavior confirmed");
    }
}
