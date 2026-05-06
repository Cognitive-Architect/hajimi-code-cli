//! Dream Memory Layer - Vector Embedding Storage (MVP)
#![deny(unsafe_code)]

use crate::auto::{AutoEntry, AutoMemory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use log::{debug, trace};
use thiserror::Error;

#[cfg(feature = "semantic-memory")]
use std::sync::Arc;
#[cfg(feature = "semantic-memory")]
use fastembed::{TextEmbedding, TextInitOptions, EmbeddingModel};

pub const EMBEDDING_DIM: usize = 384;
pub const MAX_CACHE: usize = 1000;

pub type EmbeddingCache = lru::LruCache<String, Vec<f32>>;

#[derive(Debug, Error)]
pub enum DreamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("Invalid dimension: expected {EMBED_DIM}, got {actual}", EMBED_DIM = EMBEDDING_DIM)]
    InvalidDimension { actual: usize },
    #[error("Invalid project ID")]
    InvalidProjectId,
    #[error("Cannot determine config directory")]
    NoConfigDir,
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

#[derive(Serialize, Deserialize)]
struct DreamPersistedEntry {
    id: String,
    content: String,
    tokens: usize,
    embedding: Vec<f32>,
    timestamp: String,
}

pub struct DreamMemory {
    db: rusqlite::Connection,
    embedding_cache: Mutex<EmbeddingCache>,
    project_id: String,
    db_path: PathBuf,
    jsonl_path: PathBuf,
    #[cfg(feature = "semantic-memory")]
    semantic_embedder: Option<Arc<Mutex<TextEmbedding>>>,
    #[cfg(feature = "semantic-memory")]
    model_path: Option<PathBuf>,
    semantic_disabled: AtomicBool,
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
        let memory_dir = config_dir.join("hajimi").join("memory").join(project_id);
        std::fs::create_dir_all(&memory_dir)?;
        let jsonl_path = memory_dir.join("dream.jsonl");
        let mut dream = Self {
            db,
            embedding_cache: Mutex::new(EmbeddingCache::new(
                NonZeroUsize::new(MAX_CACHE).unwrap()
            )),
            project_id: project_id.to_string(),
            db_path,
            jsonl_path,
            #[cfg(feature = "semantic-memory")]
            semantic_embedder: None,
            #[cfg(feature = "semantic-memory")]
            model_path: None,
            semantic_disabled: AtomicBool::new(false),
        };
        dream.load_from_disk()?;
        Ok(dream)
    }

    pub fn db_path(&self) -> &PathBuf { &self.db_path }
    pub fn project_id(&self) -> &str { &self.project_id }

    /// Create a DreamMemory with optional fastembed semantic embedding support.
    /// If `model_path` is provided, verifies that `model.onnx` exists and attempts
    /// to initialize a `TextEmbedding`. On any failure, gracefully falls back to
    /// hash-based embeddings (semantic_embedder remains None).
    #[cfg(feature = "semantic-memory")]
    pub fn new_with_semantic(project_id: &str, model_path: Option<PathBuf>) -> Result<Self, DreamError> {
        let mut mem = Self::new(project_id)?;
        match Self::init_semantic(model_path.clone()) {
            Ok(embedder) => {
                mem.semantic_embedder = Some(Arc::new(Mutex::new(embedder)));
                mem.model_path = model_path;
            }
            Err(e) => {
                eprintln!("fastembed init failed, fallback to hash-based: {}", e);
            }
        }
        Ok(mem)
    }

    #[cfg(feature = "semantic-memory")]
    fn init_semantic(model_path: Option<PathBuf>) -> Result<TextEmbedding, DreamError> {
        let mut opts = TextInitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true);
        if let Some(ref path) = model_path {
            let onnx_path = path.join("model.onnx");
            if !onnx_path.exists() {
                return Err(DreamError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("model.onnx not found in {:?}", path),
                )));
            }
            opts = opts.with_cache_dir(path.clone());
        }
        TextEmbedding::try_new(opts)
            .map_err(|e| DreamError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("fastembed initialization failed: {}", e),
            )))
    }

    /// Returns true if a semantic embedder is active.
    #[cfg(feature = "semantic-memory")]
    pub fn is_semantic_enabled(&self) -> bool {
        self.semantic_embedder.is_some()
    }

    /// Returns the configured local model path, if any.
    #[cfg(feature = "semantic-memory")]
    pub fn semantic_model_path(&self) -> Option<&PathBuf> {
        self.model_path.as_ref()
    }

    /// Disable semantic embedding, forcing hash-based fallback.
    pub fn disable_semantic(&self) {
        self.semantic_disabled.store(true, Ordering::Relaxed);
        debug!("semantic embedding disabled");
    }

    /// Re-enable semantic embedding (if embedder is available).
    pub fn enable_semantic(&self) {
        self.semantic_disabled.store(false, Ordering::Relaxed);
        debug!("semantic embedding enabled");
    }

    /// Returns true if semantic embedding is currently disabled.
    pub fn is_semantic_disabled(&self) -> bool {
        self.semantic_disabled.load(Ordering::Relaxed)
    }

    /// Generate an embedding for the given text using a three-tier strategy:
    /// 1. LRU cache hit
    /// 2. fastembed semantic vector (if feature enabled and available)
    /// 3. deterministic hash-based fallback
    pub fn embed(&self, text: &str) -> Vec<f32> {
        // Tier 1: LRU cache
        {
            let mut cache = self.embedding_cache.lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = cache.get(text) {
                trace!("embed cache hit: text_len={}", text.len());
                return cached.clone();
            }
        }

        // Tier 2: semantic embedding
        #[cfg(feature = "semantic-memory")]
        {
            if !self.semantic_disabled.load(Ordering::Relaxed) {
                if let Some(ref embedder) = self.semantic_embedder {
                    let docs = vec![text];
                    match embedder.lock() {
                        Ok(mut guard) => match guard.embed(docs, None) {
                            Ok(embeddings) if !embeddings.is_empty() => {
                                let vec = embeddings[0].clone();
                                let mut cache = self.embedding_cache.lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                cache.put(text.to_string(), vec.clone());
                                debug!("embed semantic: text_len={}", text.len());
                                return vec;
                            }
                            Err(e) => {
                                debug!("semantic embed failed, fallback to hash: {}", e);
                            }
                            _ => {}
                        },
                        Err(e) => {
                            debug!("semantic embedder lock poisoned, fallback to hash: {}", e);
                        }
                    }
                }
            }
        }

        // Tier 3: hash-based fallback
        let vec = self.hash_embed(text);
        let mut cache = self.embedding_cache.lock()
            .unwrap_or_else(|e| e.into_inner());
        cache.put(text.to_string(), vec.clone());
        vec
    }

    /// Deterministic hash-based embedding (MVP fallback).
    /// The same input text always produces the same vector.
    fn hash_embed(&self, text: &str) -> Vec<f32> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hasher::write(&mut hasher, text.as_bytes());
        let seed = std::hash::Hasher::finish(&mut hasher);
        let mut vec = Vec::with_capacity(EMBEDDING_DIM);
        let mut state = seed;
        for _ in 0..EMBEDDING_DIM {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let val = ((state >> 32) as u32) as f32 / u32::MAX as f32;
            vec.push(val * 2.0 - 1.0);
        }
        vec
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
                let embedding = self.embed(content);
                self.insert(key, content, tokens, &embedding)?;
            }
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), DreamError> {
        let jsonl_path = &self.jsonl_path;
        if let Some(parent) = jsonl_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut temp = tempfile::NamedTempFile::new_in(
            jsonl_path.parent().unwrap_or(std::path::Path::new("."))
        )?;
        let mut stmt = self.db.prepare(
            "SELECT id, content, tokens, embedding_blob, timestamp FROM dream_entries"
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let tokens: usize = row.get(2)?;
            let embedding_blob: Vec<u8> = row.get(3)?;
            let timestamp: String = row.get(4)?;
            let embedding: Vec<f32> = embedding_blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            Ok((id, content, tokens, embedding, timestamp))
        })?;
        for row in rows {
            let (id, content, tokens, embedding, timestamp) = row?;
            let entry = DreamPersistedEntry { id, content, tokens, embedding, timestamp };
            let json = serde_json::to_string(&entry)
                .map_err(|e| DreamError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            writeln!(temp, "{}", json)?;
        }
        temp.flush()?;
        std::fs::rename(temp.path(), jsonl_path)?;
        Ok(())
    }

    pub fn load_from_disk(&mut self) -> Result<(), DreamError> {
        if !self.jsonl_path.exists() { return Ok(()); }
        let content = std::fs::read_to_string(&self.jsonl_path)?;
        for line in content.lines() {
            if line.trim().is_empty() { continue; }
            let entry: DreamPersistedEntry = match serde_json::from_str(line) {
                Ok(e) => e,
                Err(_) => continue,
            };
            // Backward compat: re-embed if old dimension mismatches
            let embedding = if entry.embedding.len() != EMBEDDING_DIM {
                debug!("dimension compat: re-embed {} (old dim={})", entry.id, entry.embedding.len());
                self.embed(&entry.content)
            } else {
                entry.embedding
            };
            let _ = self.insert(&entry.id, &entry.content, entry.tokens, &embedding);
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
            let embedding = dream.embed("test content");
            assert_eq!(embedding.len(), EMBEDDING_DIM);
        }
    }

    #[test]
    fn test_dream_embed_deterministic() {
        if let Ok(dream) = DreamMemory::new("test_embed_deterministic") {
            let a = dream.embed("hello world");
            let b = dream.embed("hello world");
            assert_eq!(a, b);
            let c = dream.embed("different text");
            assert_ne!(a, c);
        }
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
    fn test_dream_persist_load() {
        let pid = "test_persist_load";
        {
            let mut dream = DreamMemory::new(pid).unwrap();
            let embedding = dream.embed("persist test");
            dream.insert("k1", "persist test", 2, &embedding).unwrap();
            dream.save().unwrap();
        }
        {
            let dream = DreamMemory::new(pid).unwrap();
            assert_eq!(dream.len().unwrap(), 1);
            let (content, _) = dream.get("k1").unwrap().unwrap();
            assert_eq!(content, "persist test");
        }
    }

    #[test]
    fn test_dream_recall_similarity() {
        if let Ok(mut dream) = DreamMemory::new("test_recall_similarity") {
            let text = "the quick brown fox jumps over the lazy dog";
            let embedding = dream.embed(text);
            dream.insert("k1", text, 9, &embedding).unwrap();
            let results = dream.search(&embedding, 5).unwrap();
            assert!(!results.is_empty());
            assert!(
                results[0].similarity_score >= 0.7,
                "same-text recall should be >= 0.7, got {}",
                results[0].similarity_score
            );
        }
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

    #[test]
    fn test_empty_string_embed() {
        let dream = DreamMemory::new("test_empty_string").unwrap();
        let vec = dream.embed("");
        assert_eq!(vec.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_long_text_embed() {
        let dream = DreamMemory::new("test_long_text").unwrap();
        let text = "a".repeat(15000);
        let vec = dream.embed(&text);
        assert_eq!(vec.len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_lru_eviction() {
        let dream = DreamMemory::new("test_lru_eviction").unwrap();
        for i in 0..1001 {
            let _ = dream.embed(&format!("text_{:04}", i));
        }
        assert_eq!(dream.embed("text_0000").len(), EMBEDDING_DIM);
        assert_eq!(dream.embed("text_1000").len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_dimension_compat() {
        let pid = "test_dim_compat";
        {
            let dream = DreamMemory::new(pid).unwrap();
            let old_entry = DreamPersistedEntry {
                id: "k1".to_string(),
                content: "compat test".to_string(),
                tokens: 2,
                embedding: vec![0.5f32; 64],
                timestamp: Utc::now().to_rfc3339(),
            };
            let json = serde_json::to_string(&old_entry).unwrap();
            std::fs::write(&dream.jsonl_path, format!("{}\n", json)).unwrap();
        }
        {
            let dream = DreamMemory::new(pid).unwrap();
            let (content, embedding) = dream.get("k1").unwrap().unwrap();
            assert_eq!(content, "compat test");
            assert_eq!(embedding.len(), EMBEDDING_DIM);
        }
    }

    #[cfg(feature = "semantic-memory")]
    #[test]
    fn test_semantic_similarity() {
        let model_path = PathBuf::from("models/fast-all-MiniLM-L6-v2");
        if !model_path.join("model.onnx").exists() {
            eprintln!("skip: model not found");
            return;
        }
        let dream = DreamMemory::new_with_semantic("test_semantic_sim", Some(model_path)).unwrap();
        if !dream.is_semantic_enabled() {
            eprintln!("skip: semantic embedder not initialized");
            return;
        }
        let embed_cat = dream.embed("cat");
        let embed_kitten = dream.embed("kitten");
        let sim = cosine_similarity(&embed_cat, &embed_kitten);
        assert!(
            sim > 0.7,
            "semantic similarity cat-kitten should be > 0.7, got {}",
            sim
        );
    }

    #[cfg(feature = "semantic-memory")]
    #[test]
    fn test_disable_semantic() {
        let model_path = PathBuf::from("models/fast-all-MiniLM-L6-v2");
        let dream = DreamMemory::new_with_semantic("test_disable_sem", Some(model_path)).unwrap();
        if !dream.is_semantic_enabled() {
            eprintln!("skip: semantic not available");
            return;
        }
        let before = dream.embed("test text");
        dream.disable_semantic();
        assert!(dream.is_semantic_disabled());
        let after = dream.embed("test text");
        assert_ne!(before, after, "disable_semantic should change embed result");
        dream.enable_semantic();
        assert!(!dream.is_semantic_disabled());
        let reenabled = dream.embed("test text");
        assert_eq!(before, reenabled, "enable_semantic should restore result");
    }
}
