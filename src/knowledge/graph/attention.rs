//! 图注意力机制（缩放点积）

use crate::knowledge::graph::Result;

pub const EMBEDDING_DIM: usize = 384;

/// 注意力权重（缩放点积）
pub fn attention_weights(query: &[f32], key: &[f32]) -> Result<f32> {
    let score: f32 = query.iter().zip(key.iter()).map(|(a,b)| a*b).sum();
    Ok(score / (EMBEDDING_DIM as f32).sqrt())
}

/// 注意力池化
pub fn attention_pooling(embeddings: &[Vec<f32>], query: &[f32]) -> Result<Vec<f32>> {
    if embeddings.is_empty() {
        return Ok(vec![0.0; EMBEDDING_DIM]);
    }
    let weights: Vec<f32> = embeddings.iter()
        .map(|emb| attention_weights(query, emb).unwrap_or(0.0))
        .collect();
    let total: f32 = weights.iter().sum::<f32>().max(1e-6);
    let mut out = vec![0.0; EMBEDDING_DIM];
    for (i, emb) in embeddings.iter().enumerate() {
        for j in 0..EMBEDDING_DIM {
            out[j] += weights[i] * emb[j] / total;
        }
    }
    Ok(out)
}
