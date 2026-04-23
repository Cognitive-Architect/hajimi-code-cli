//! 注意力聚合模块（GNN消息传递）
//! B-36/01: attention_pooling + attention_weights

use crate::graph::Result;

pub const EMBEDDING_DIM: usize = 384;

/// 注意力权重计算（点积相似度）
pub fn attention_weights(query: &[f32], keys: &[Vec<f32>]) -> Vec<f32> {
    keys.iter()
        .map(|k| cosine_similarity(query, k))
        .collect()
}

/// 注意力池化（加权平均）
pub fn attention_pooling(embeddings: &[Vec<f32>], query: &[f32]) -> Result<Vec<f32>> {
    if embeddings.is_empty() {
        return Ok(query.to_vec());
    }
    let weights = attention_weights(query, embeddings);
    let weight_sum: f32 = weights.iter().sum::<f32>().max(1e-6);
    let mut result = vec![0.0f32; EMBEDDING_DIM];
    for (emb, w) in embeddings.iter().zip(weights.iter()) {
        for (i, &v) in emb.iter().enumerate() {
            if i < EMBEDDING_DIM {
                result[i] += v * w / weight_sum;
            }
        }
    }
    Ok(result)
}

/// 余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b).max(1e-6)
}

/// GNN聚合（简化版：邻居平均）
pub fn gnn_aggregate(neighbor_embs: &[Vec<f32>], self_emb: &[f32], alpha: f32) -> Result<Vec<f32>> {
    if neighbor_embs.is_empty() {
        return Ok(self_emb.to_vec());
    }
    let neighbor_avg = neighbor_embs.iter()
        .fold(vec![0.0f32; EMBEDDING_DIM], |mut acc, emb| {
            for (i, &v) in emb.iter().enumerate() {
                if i < EMBEDDING_DIM { acc[i] += v; }
            }
            acc
        })
        .into_iter()
        .map(|v| v / neighbor_embs.len() as f32)
        .collect::<Vec<_>>();
    let mut result = vec![0.0f32; EMBEDDING_DIM];
    for i in 0..EMBEDDING_DIM {
        result[i] = alpha * self_emb.get(i).copied().unwrap_or(0.0)
            + (1.0 - alpha) * neighbor_avg.get(i).copied().unwrap_or(0.0);
    }
    Ok(result)
}
