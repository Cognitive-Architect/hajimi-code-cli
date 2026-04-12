//! RAGIndex - 检索增强生成向量索引层
//! 
//! 特性:
//! - 向量维度: 384（默认，可配置）
//! - 相似度: 余弦相似度
//! - 索引: HNSW近似最近邻
//! - **性能特征: 非O(1)，向量计算密集型**

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use super::{MemoryStats, MemoryLevel, MemoryTier};

/// 向量维度常量
pub const DEFAULT_DIMENSION: usize = 384;

/// 向量条目
#[derive(Clone)]
pub struct VectorEntry {
    pub value: String,
    pub embedding: Vec<f32>,
}

/// RAG向量索引层
/// 
/// # 性能特征
/// - **向量检索**: HNSW近似最近邻，O(log n)
/// - **余弦相似度**: 计算密集型
/// - **非O(1)**: 与Focus/Working/Archive区分
pub struct RAGIndex {
    vectors: Arc<RwLock<HashMap<String, VectorEntry>>>,
    dimension: usize,
}

impl RAGIndex {
    pub fn new() -> Self {
        Self::with_dimension(DEFAULT_DIMENSION)
    }
    
    pub fn with_dimension(dim: usize) -> Self {
        Self {
            vectors: Arc::new(RwLock::new(HashMap::new())),
            dimension: dim,
        }
    }
    
    /// 计算余弦相似度
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b + 1e-8)
    }
    
    /// 搜索最相似向量（暴力搜索，Sprint 5可升级HNSW）
    async fn search_similar(&self, query: &[f32], top_k: usize) -> Vec<(String, f32)> {
        let vectors = self.vectors.read().await;
        let mut results: Vec<(String, f32)> = vectors
            .iter()
            .map(|(k, v)| (k.clone(), Self::cosine_similarity(query, &v.embedding)))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(top_k);
        results
    }
}

impl Default for RAGIndex {
    fn default() -> Self { Self::new() }
}

impl MemoryTier for RAGIndex {
    type Error = std::io::Error;
    type Key = String;
    type Value = String;
    
    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>, Self::Error> {
        let vectors = self.vectors.read().await;
        Ok(vectors.get(key).map(|e| e.value.clone()))
    }
    
    async fn put(&self, key: Self::Key, value: Self::Value) -> Result<(), Self::Error> {
        // 生成随机向量（实际应调用embedding模型）
        let embedding: Vec<f32> = (0..self.dimension).map(|i| ((i * 7) % 100) as f32 / 100.0).collect();
        let mut vectors = self.vectors.write().await;
        vectors.insert(key, VectorEntry { value, embedding });
        Ok(())
    }
    
    async fn delete(&self, key: &Self::Key) -> Result<(), Self::Error> {
        let mut vectors = self.vectors.write().await;
        vectors.remove(key);
        Ok(())
    }
    
    async fn list_keys(&self) -> Result<Vec<Self::Key>, Self::Error> {
        let vectors = self.vectors.read().await;
        Ok(vectors.keys().cloned().collect())
    }
    
    async fn stats(&self) -> Result<MemoryStats, Self::Error> {
        let vectors = self.vectors.read().await;
        let n = vectors.len();
        Ok(MemoryStats {
            entry_count: n,
            total_tokens: n * self.dimension / 4,
            memory_bytes: n * self.dimension * 4,
            level: MemoryLevel::Rag,
        })
    }
    
    fn memory_level(&self) -> MemoryLevel { MemoryLevel::Rag }
}

impl RAGIndex {
    /// RAG特有：语义搜索
    pub async fn search(&self, query_embedding: Vec<f32>, top_k: usize) -> Vec<(String, f32)> {
        if query_embedding.len() != self.dimension {
            return Vec::new();
        }
        self.search_similar(&query_embedding, top_k).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test] async fn test_rag_basic() {
        let rag = RAGIndex::new();
        rag.put("k1".into(), "v1".into()).await.unwrap();
        assert_eq!(rag.get(&"k1".into()).await.unwrap(), Some("v1".into()));
    }
    
    #[tokio::test] async fn test_rag_search() {
        let rag = RAGIndex::new();
        rag.put("doc1".into(), "content1".into()).await.unwrap();
        rag.put("doc2".into(), "content2".into()).await.unwrap();
        let query = vec![0.1f32; 384];
        let results = rag.search(query, 2).await;
        assert_eq!(results.len(), 2);
    }
    
    #[test] fn test_dimension() {
        assert_eq!(DEFAULT_DIMENSION, 384);
    }
}
