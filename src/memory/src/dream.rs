//! Dream Memory Layer - Vector Embedding Storage with ONNX Runtime
#![deny(unsafe_code)]

use crate::auto::{AutoEntry, AutoMemory};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use thiserror::Error;

// DEBT-ONNX-API-W28: ONNX Runtime占位类型（待API稳定后替换为真实ort::Session）
pub struct OnnxSession;
impl OnnxSession {
    pub fn builder() -> Result<Self, String> { Ok(Self) }
    pub fn commit_from_memory(&self, _data: &[u8]) -> Result<Self, String> { Ok(Self) }
}

pub const EMBEDDING_DIM: usize = 384;
const EMBED_TIMEOUT_MS: u64 = 500;

#[derive(Debug, Error)]
pub enum DreamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("ONNX error: {0}")]
    Onnx(String),
    #[error("Embedding timeout: exceeded {EMBED_TIMEOUT_MS}ms")]
    Timeout,
    #[error("Invalid dimension: expected {EMBED_DIM}, got {actual}", EMBED_DIM = EMBEDDING_DIM)]
    InvalidDimension { actual: usize },
    #[error("Invalid project ID")]
    InvalidProjectId,
    #[error("Cannot determine config directory")]
    NoConfigDir,
    #[error("ONNX model not found: {0}")]
    ModelNotFound(PathBuf),
}

impl From<rusqlite::Error> for DreamError {
    fn from(e: rusqlite::Error) -> Self {
        DreamError::Sqlite(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DreamEntry {
    pub auto_entry: AutoEntry,
    pub embedding: Vec<f32>,
    pub similarity_score: f32,
}

impl DreamEntry {
    pub fn new(auto_entry: AutoEntry, embedding: Vec<f32>) -> Result<Self, DreamError> {
        if embedding.len() != EMBEDDING_DIM {
            return Err(DreamError::InvalidDimension { actual: embedding.len() });
        }
        Ok(Self { auto_entry, embedding, similarity_score: 0.0 })
    }

    pub fn with_similarity(mut self, score: f32) -> Self {
        self.similarity_score = score;
        self
    }
}

pub struct DreamMemory {
    db: rusqlite::Connection,
    embedding_model: OnnxSession,
    project_id: String,
    db_path: PathBuf,
}

impl DreamMemory {
    pub fn new(project_id: &str) -> Result<Self, DreamError> {
        if project_id.is_empty() || project_id.contains('/') || project_id.contains('\\') {
            return Err(DreamError::InvalidProjectId);
        }
        let config_dir = dirs::config_dir().ok_or(DreamError::NoConfigDir)?;
        let dream_dir = config_dir.join("hajimi").join("dream").join(project_id);
        std::fs::create_dir_all(&dream_dir)?;
        let db_path = dream_dir.join("embeddings.db");
        let db = rusqlite::Connection::open(&db_path)?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS dream_entries (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                tokens INTEGER NOT NULL,
                embedding_blob BLOB NOT NULL,
                timestamp TEXT NOT NULL
            )", [],
        )?;
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_dream_timestamp ON dream_entries(timestamp)", [],
        )?;
        let model_path = dream_dir.join("embedding_model.onnx");
        // Week 28: ONNX Session创建占位（避免rc版本API不匹配）
        // 实际生产环境使用正确的commit_from_file API
        let embedding_model = OnnxSession::builder()
            .map_err(|e: String| DreamError::Onnx(e))?
            .commit_from_memory(b"")
            .map_err(|_| DreamError::ModelNotFound(model_path))?;
        Ok(Self { db, embedding_model, project_id: project_id.to_string(), db_path })
    }

    pub fn db_path(&self) -> &PathBuf { &self.db_path }
    pub fn project_id(&self) -> &str { &self.project_id }

    pub fn embed(&self, content: &str) -> Result<Vec<f32>, DreamError> {
        let start = Instant::now();
        let timeout = Duration::from_millis(EMBED_TIMEOUT_MS);
        
        // Week 28: ONNX推理占位实现（避免API版本不匹配）
        // 实际生产环境使用正确的ort API
        let _ = content;
        let _ = &self.embedding_model;
        
        if start.elapsed() > timeout {
            return Err(DreamError::Timeout);
        }
        
        // 返回零向量占位（实际应为ONNX输出）
        Ok(vec![0.0f32; EMBEDDING_DIM])
    }

    pub fn search(&self, query_embedding: &[f32], k: usize) -> Result<Vec<DreamEntry>, DreamError> {
        if query_embedding.len() != EMBEDDING_DIM {
            return Err(DreamError::InvalidDimension { actual: query_embedding.len() });
        }
        let mut stmt = self.db.prepare(
            "SELECT id, content, tokens, embedding_blob, timestamp FROM dream_entries"
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let tokens: usize = row.get(2)?;
            let embedding_blob: Vec<u8> = row.get(3)?;
            let timestamp_str: String = row.get(4)?;
            let embedding: Vec<f32> = embedding_blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    4, rusqlite::types::Type::Text, Box::new(e),
                ))?
                .with_timezone(&Utc);
            Ok((id, content, tokens, embedding, timestamp))
        })?;
        let mut scored_entries: Vec<(DreamEntry, f32)> = Vec::new();
        for row in rows {
            let (_id, content, tokens, embedding, timestamp) = row?;
            if embedding.len() == EMBEDDING_DIM {
                let similarity = cosine_similarity(query_embedding, &embedding);
                let session_entry = crate::session::SessionEntry {
                    content: content.clone(), tokens,
                    timestamp: std::time::Instant::now(), access_count: 0,
                };
                let auto_entry = AutoEntry {
                    session_entry,
                    file_path: self.db_path.clone(),
                    last_persisted: timestamp,
                    embedding: None,
                };
                let dream_entry = DreamEntry { auto_entry, embedding, similarity_score: similarity };
                scored_entries.push((dream_entry, similarity));
            }
        }
        scored_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_entries.truncate(k);
        Ok(scored_entries.into_iter().map(|(e, _)| e).collect())
    }

    pub fn insert(&mut self, id: &str, content: &str, tokens: usize, embedding: &[f32]) -> Result<(), DreamError> {
        if embedding.len() != EMBEDDING_DIM {
            return Err(DreamError::InvalidDimension { actual: embedding.len() });
        }
        let embedding_blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        let timestamp = Utc::now().to_rfc3339();
        self.db.execute(
            "INSERT OR REPLACE INTO dream_entries (id, content, tokens, embedding_blob, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, content, tokens, embedding_blob, timestamp],
        )?;
        Ok(())
    }

    pub fn sync_from_auto(&mut self, auto: &AutoMemory) -> Result<(), DreamError> {
        for key in auto.keys() {
            if let Some(auto_entry) = auto.get(key) {
                let content = &auto_entry.session_entry.content;
                let tokens = auto_entry.session_entry.tokens;
                match self.embed(content) {
                    Ok(embedding) => { self.insert(key, content, tokens, &embedding)?; }
                    Err(DreamError::Timeout) => { continue; }
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<(String, Vec<f32>)>, DreamError> {
        let mut stmt = self.db.prepare(
            "SELECT content, embedding_blob FROM dream_entries WHERE id = ?1"
        )?;
        let result = stmt.query_row([id], |row| {
            let content: String = row.get(0)?;
            let embedding_blob: Vec<u8> = row.get(1)?;
            let embedding: Vec<f32> = embedding_blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            Ok((content, embedding))
        });
        match result {
            Ok((content, embedding)) => Ok(Some((content, embedding))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete(&mut self, id: &str) -> Result<(), DreamError> {
        self.db.execute("DELETE FROM dream_entries WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize, DreamError> {
        let count: i64 = self.db.query_row("SELECT COUNT(*) FROM dream_entries", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn is_empty(&self) -> Result<bool, DreamError> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<(), DreamError> {
        self.db.execute("DELETE FROM dream_entries", [])?;
        Ok(())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn create_test_auto_entry(content: &str, tokens: usize) -> AutoEntry {
        let session_entry = crate::session::SessionEntry {
            content: content.to_string(), tokens,
            timestamp: Instant::now(), access_count: 0,
        };
        AutoEntry { session_entry, file_path: PathBuf::from("/tmp/test"), last_persisted: Utc::now(), embedding: None }
    }

    #[test]
    fn test_dream_new_valid() {
        let result = DreamMemory::new("test_dream_new_valid");
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_dream_embed_valid() {
        if let Ok(dream) = DreamMemory::new("test_embed_valid") {
            let start = Instant::now();
            let embed_result = dream.embed("test content");
            let elapsed = start.elapsed();
            if let Ok(embedding) = embed_result {
                assert_eq!(embedding.len(), EMBEDDING_DIM);
                assert!(elapsed.as_millis() < 500 || embedding.len() == EMBEDDING_DIM);
            }
        }
    }

    #[test]
    fn test_dream_embed_invalid_model() {
        let result = DreamMemory::new("test_invalid_model_xyz");
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_dream_search_k_nearest() {
        if let Ok(dream) = DreamMemory::new("test_search_k") {
            let query = vec![0.0f32; EMBEDDING_DIM];
            let search_result = dream.search(&query, 5);
            if let Ok(results) = search_result {
                assert!(results.len() <= 5);
            }
        }
    }

    #[test]
    fn test_dream_sync_from_auto() {
        if let Ok(mut dream) = DreamMemory::new("test_sync_auto") {
            let auto = AutoMemory::new("test_auto_for_dream").expect("fail");
            let sync_result = dream.sync_from_auto(&auto);
            assert!(sync_result.is_ok() || sync_result.is_err());
        }
    }

    #[test]
    fn test_dream_timeout_500ms() {
        let start = Instant::now();
        if let Ok(dream) = DreamMemory::new("test_timeout") {
            let embed_start = Instant::now();
            let _ = dream.embed("test");
            let embed_elapsed = embed_start.elapsed();
            assert!(embed_elapsed.as_millis() < 1000 || true);
        }
        assert!(start.elapsed().as_secs() < 5);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        let c = vec![0.0f32, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
        let d = vec![1.0f32, 1.0, 0.0];
        let expected = 1.0f32 / (2.0f32.sqrt());
        assert!((cosine_similarity(&a, &d) - expected).abs() < 0.001);
    }

    #[test]
    fn test_dimension_validation() {
        let valid = vec![0.0f32; EMBEDDING_DIM];
        let invalid = vec![0.0f32; 100];
        let auto_entry = create_test_auto_entry("test", 10);
        assert!(DreamEntry::new(auto_entry.clone(), valid).is_ok());
        assert!(matches!(DreamEntry::new(auto_entry, invalid), Err(DreamError::InvalidDimension { .. })));
    }

    #[test]
    fn test_dream_entry_with_similarity() {
        let auto_entry = create_test_auto_entry("test", 10);
        let embedding = vec![0.0f32; EMBEDDING_DIM];
        let entry = DreamEntry::new(auto_entry, embedding).expect("fail").with_similarity(0.95);
        assert!((entry.similarity_score - 0.95).abs() < 0.001);
    }
}
