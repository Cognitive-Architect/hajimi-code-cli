//! 双引擎索引模块 - HNSW语义 + Tantivy全文 + pgvector

mod hnsw;
mod pgvector;
mod tantivy;
mod unified;

pub use hnsw::{HnswIndex, SemanticResult};
pub use pgvector::PgVectorIndex;
pub use tantivy::{TantivyIndex, FulltextResult};
pub use unified::{UnifiedIndex, UnifiedSearchResult, unified_search};

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("HNSW错误: {0}")] HnswError(String),
    #[error("Tantivy错误: {0}")] TantivyError(String),
    #[error("维度错误: 期望384维, 实际{0}维")] DimensionMismatch(usize),
    #[error("IO错误: {0}")] IoError(#[from] std::io::Error),
    #[error("序列化错误: {0}")] SerializationError(#[from] serde_json::Error),
    #[error("路径错误: {0}")] PathError(String),
}

pub type IndexResult<T> = Result<T, IndexError>;

/// IDX-001: 强制384维
pub const EMBEDDING_DIMENSION: usize = 384;

/// IDX-015: dirs::config_dir路径安全
pub fn default_auto_path() -> IndexResult<PathBuf> {
    let c = dirs::config_dir().ok_or_else(|| IndexError::PathError("无法获取配置目录".to_string()))?;
    Ok(c.join("hajimi").join("memory").join("auto"))
}

pub trait IndexedDocument {
    fn doc_id(&self) -> &str;
    fn text_content(&self) -> &str;
    fn embedding(&self) -> Option<&[f32]>;
    fn timestamp(&self) -> u64;
}
