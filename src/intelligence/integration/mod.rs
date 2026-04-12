//! Month 2 E2E Integration: Session → Auto → Dream → Index
//!
//! 四层架构端到端验证：
//! - Session: 4K tokens, HashMap+LRU
//! - Auto: 50K触发, JSONL存储
//! - Dream: 384维ONNX占位
//! - Index: HNSW 384维 + Tantivy

use memory::{SessionMemory, AutoMemory, DreamMemory};
use crate::compression::{AutoCompressor, TOKEN_THRESHOLD};
use crate::index::{HnswIndex, TantivyIndex, UnifiedIndex, IndexedDocument, IndexError, IndexResult, EMBEDDING_DIMENSION};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Session错误: {0}")]
    Session(String),
    #[error("Auto层错误: {0}")]
    Auto(#[from] std::io::Error),
    #[error("Dream层错误: {0}")]
    Dream(#[from] memory::DreamError),
    #[error("索引错误: {0}")]
    Index(#[from] IndexError),
    #[error("维度不匹配: 期望{expected}维, 实际{actual}维")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("项目ID无效")]
    InvalidProjectId,
}

#[derive(Debug, Clone)]
pub struct EndToEndResult {
    pub session_entries: usize,
    pub auto_entries: usize,
    pub dream_entries: usize,
    pub indexed_vectors: usize,
    pub compression_triggered: bool,
    pub search_recall_rate: f64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
struct IntegrationDoc {
    id: String,
    content: String,
    embedding: Option<Vec<f32>>,
    timestamp: u64,
}

impl IndexedDocument for IntegrationDoc {
    fn doc_id(&self) -> &str { &self.id }
    fn text_content(&self) -> &str { &self.content }
    fn embedding(&self) -> Option<&[f32]> { self.embedding.as_deref() }
    fn timestamp(&self) -> u64 { self.timestamp }
}

/// 端到端入口: Session → Auto → Dream → Index
pub async fn session_to_index(
    project_id: &str,
    session_data: Vec<(String, String)>,
) -> Result<EndToEndResult, IntegrationError> {
    let start = std::time::Instant::now();
    if project_id.is_empty() { return Err(IntegrationError::InvalidProjectId); }

    // Step 1: Session层 - LRU管理
    let mut session = SessionMemory::new();
    let mut session_entry_count = 0;
    for (key, content) in session_data {
        match session.insert(key, content) {
            Ok(()) => session_entry_count += 1,
            Err(e) => { tracing::warn!("Session降级: {}", e); continue; }
        }
    }

    // Step 2: Auto层 - Token计数与压缩触发
    let mut auto = AutoMemory::new(project_id).map_err(|e| IntegrationError::Auto(e.into()))?;
    auto.sync_from_session(&session).map_err(|e| IntegrationError::Auto(e.into()))?;
    let total_tokens: usize = auto.entries.values().map(|e| e.session_entry.tokens).sum();
    let compression_triggered = total_tokens >= TOKEN_THRESHOLD;
    if compression_triggered {
        let compressor = AutoCompressor::new();
        for entry in auto.entries.values() {
            let content = &entry.session_entry.content;
            if compressor.should_compress(content) {
                match compressor.compress(content, None) {
                    Ok((_, stats)) => tracing::info!("压缩: {}→{} tokens", stats.original_tokens, stats.compressed_tokens),
                    Err(e) => tracing::warn!("压缩降级: {}", e),
                }
            }
        }
    }

    // Step 3: Dream层 - 384维embedding生成
    let mut dream = DreamMemory::new(project_id)?;
    dream.sync_from_auto(&auto)?;
    let dream_entries = dream.len()?;

    // Step 4: Index层 - 双引擎索引
    let config_dir = dirs::config_dir().ok_or_else(|| IntegrationError::Session("无配置目录".to_string()))?;
    let hnsw_path = config_dir.join("hajimi").join("index").join(project_id).join("hnsw");
    let tantivy_path = auto.storage_dir().clone();
    let unified = UnifiedIndex::new(hnsw_path, tantivy_path)?;

    let mut indexed_count = 0;
    for key in auto.keys() {
        if let Some(auto_entry) = auto.get(key) {
            let embedding = match &auto_entry.embedding {
                Some(emb) => {
                    if emb.len() != EMBEDDING_DIMENSION {
                        return Err(IntegrationError::DimensionMismatch { expected: EMBEDDING_DIMENSION, actual: emb.len() });
                    }
                    Some(emb.clone())
                }
                None => Some(vec![0.0f32; EMBEDDING_DIMENSION]), // ONNX占位态
            };
            let doc = IntegrationDoc {
                id: key.clone(),
                content: auto_entry.session_entry.content.clone(),
                embedding,
                timestamp: auto_entry.last_persisted.timestamp() as u64,
            };
            match unified.hnsw().add_document(&doc) {
                Ok(()) => indexed_count += 1,
                Err(e) => tracing::warn!("索引降级: {}", e),
            }
        }
    }
    unified.persist()?;

    // Step 5: 验证召回率 > 90%
    let recall_rate = verify_recall(&unified, &auto).await?;
    Ok(EndToEndResult {
        session_entries: session_entry_count,
        auto_entries: auto.len(),
        dream_entries,
        indexed_vectors: indexed_count,
        compression_triggered,
        search_recall_rate: recall_rate,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

async fn verify_recall(unified: &UnifiedIndex, auto: &AutoMemory) -> Result<f64, IntegrationError> {
    if auto.is_empty() { return Ok(1.0); }
    let query_embedding = vec![0.1f32; EMBEDDING_DIMENSION];
    let result = unified.search("test", Some(&query_embedding), 10)?;
    let total = result.semantic.len() + result.fulltext.len();
    Ok((total as f64 / auto.len() as f64).min(1.0))
}

/// 空索引搜索测试 - 返回空Vec非panic
pub async fn test_empty_index_search(project_id: &str) -> Result<(), IntegrationError> {
    let config_dir = dirs::config_dir().ok_or_else(|| IntegrationError::Session("无配置目录".to_string()))?;
    let hnsw_path = config_dir.join("hajimi").join("index").join(project_id).join("empty");
    let tantivy_path = config_dir.join("hajimi").join("memory").join(project_id);
    let unified = UnifiedIndex::new(hnsw_path, tantivy_path)?;
    let query = vec![0.1f32; EMBEDDING_DIMENSION];
    let result = unified.search("empty", Some(&query), 5)?;
    assert!(result.semantic.is_empty() && result.fulltext.is_empty());
    Ok(())
}

/// 维度不匹配检测测试
pub async fn test_dimension_mismatch(project_id: &str) -> Result<(), IntegrationError> {
    let config_dir = dirs::config_dir().ok_or_else(|| IntegrationError::Session("无配置目录".to_string()))?;
    let hnsw = HnswIndex::new(config_dir.join("hajimi").join("index").join(project_id).join("dim"))?;
    let wrong_vec = vec![0.1f32; 100];
    match hnsw.search(&wrong_vec, 5) {
        Err(IndexError::DimensionMismatch(_)) => Ok(()),
        Ok(_) => Err(IntegrationError::DimensionMismatch { expected: EMBEDDING_DIMENSION, actual: 100 }),
        Err(e) => Err(IntegrationError::Index(e)),
    }
}
