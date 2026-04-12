//! GNN图神经网络接口

use crate::knowledge::graph::{GraphDb, Result};
use crate::knowledge::graph::attention::{attention_pooling, EMBEDDING_DIM};

/// GNN聚合（注意力加权）
pub fn gnn_aggregate(db: &GraphDb, node_ids: &[String]) -> Result<Vec<f32>> {
    let embeddings: Vec<Vec<f32>> = node_ids.iter()
        .filter_map(|id| db.get_node_embedding(id).ok().flatten())
        .collect();
    if embeddings.is_empty() {
        return Ok(vec![0.0; EMBEDDING_DIM]);
    }
    attention_pooling(&embeddings, &embeddings[0])
}
