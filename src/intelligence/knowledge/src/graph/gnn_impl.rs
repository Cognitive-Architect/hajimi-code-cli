//! GNN推理引擎实现
use crate::graph::{GraphDb, GraphError, Result};
use crate::graph::attention::attention_pooling;

pub struct GnnEngine { max_hops: usize }

impl GnnEngine {
    pub fn new(max_hops: usize) -> Self { Self { max_hops } }

    pub fn inference(&self, db: &GraphDb, node_id: &str) -> Result<Vec<f32>> {
        let target = db.get_node(node_id)?;
        let mut all_ids = vec![node_id.to_string()];
        let mut current = vec![node_id.to_string()];
        for _ in 0..self.max_hops {
            let mut next = Vec::new();
            for id in &current {
                if let Ok(nbs) = db.get_neighbors(id, None) {
                    for n in nbs { if !all_ids.contains(&n.id) { all_ids.push(n.id.clone()); next.push(n.id); } }
                }
            }
            current = next;
        }
        let embs: Vec<_> = all_ids.iter().filter(|i| *i != node_id)
            .filter_map(|i| db.get_node_embedding(i).ok().flatten()).collect();
        let target_emb = target.embedding.ok_or_else(|| GraphError::Inference("No emb".into()))?;
        attention_pooling(&embs, &target_emb)
    }

    pub fn find_similar(&self, db: &GraphDb, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        let mut s: Vec<_> = db.get_all_nodes()?.into_iter()
            .filter_map(|n| n.embedding.map(|e| (n.id, cosine(query, &e)))).collect();
        s.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(s.into_iter().take(k).collect())
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (na * nb).max(1e-6)
}
