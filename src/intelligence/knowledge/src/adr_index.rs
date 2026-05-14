//! ADR pattern extraction and knowledge graph indexing module.
//!
//! Implements SimHash-64 shard routing for distributing ADR patterns
//! across 16 shards. Part of the 5-tier memory cascade:
//! Session → Auto → Dream → Graph → Knowledge

use foundation_hash::{get_shard_id, NUM_SHARDS};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents an extracted ADR pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrPattern {
    pub id: String,
    pub title: String,
    pub keywords: Vec<String>,
    pub shard_id: usize,
    pub path: String,
}

/// Errors that can occur during ADR indexing.
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid ADR format")]
    InvalidFormat,
}

/// In-memory knowledge graph index for ADR patterns.
/// Uses HashMap-backed structures for O(1) lookups by ID or keyword.
#[derive(Debug, Default)]
pub struct KnowledgeGraphIndex {
    shards: Vec<Vec<AdrPattern>>,
    id_map: HashMap<String, String>,
    keyword_index: HashMap<String, Vec<String>>,
}

impl KnowledgeGraphIndex {
    /// Creates a new index with 16 empty shards.
    pub fn new() -> Self {
        let mut shards = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            shards.push(Vec::new());
        }
        Self {
            shards,
            id_map: HashMap::new(),
            keyword_index: HashMap::new(),
        }
    }

    /// Inserts a pattern into the index, updating all lookup tables.
    pub fn insert(&mut self, pattern: AdrPattern) {
        let shard_id = pattern.shard_id % NUM_SHARDS;
        self.shards[shard_id].push(pattern.clone());
        self.id_map.insert(pattern.id.clone(), pattern.path.clone());
        for kw in &pattern.keywords {
            self.keyword_index
                .entry(kw.clone())
                .or_default()
                .push(pattern.path.clone());
        }
    }

    /// Searches for an ADR by its ID. Returns the file path if found.
    pub fn search_by_id(&self, id: &str) -> Option<&str> {
        self.id_map.get(id).map(|s| s.as_str())
    }

    /// Searches for ADRs by keyword. Returns matching file paths.
    pub fn search_by_keyword(&self, keyword: &str) -> Vec<&str> {
        self.keyword_index
            .get(&keyword.to_lowercase())
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Returns the configured shard count.
    pub fn shard_count(&self) -> usize {
        NUM_SHARDS
    }
}

/// Extracts ADR patterns from raw markdown content.
pub fn extract_adr_patterns(content: &str) -> Vec<AdrPattern> {
    let title = content
        .lines()
        .find(|l| l.starts_with("title:"))
        .map(|l| l.trim_start_matches("title:").trim().to_string())
        .unwrap_or_default();
    let mut keywords = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            keywords.push(trimmed.trim_start_matches("# ").trim().to_lowercase());
        } else if trimmed.starts_with("## ") {
            keywords.push(trimmed.trim_start_matches("## ").trim().to_lowercase());
        }
        for word in trimmed.split_whitespace() {
            let clean = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if clean.starts_with("ADR-") {
                keywords.push(clean.to_string());
            }
        }
    }
    keywords.sort();
    keywords.dedup();
    let id = content
        .lines()
        .find(|l| l.starts_with("id:"))
        .map(|l| l.trim_start_matches("id:").trim().to_string())
        .unwrap_or_default();
    let shard_id = if id.is_empty() { 0 } else { get_shard_id(&id) };
    vec![AdrPattern {
        id,
        title,
        keywords,
        shard_id,
        path: String::new(),
    }]
}

/// Builds a `KnowledgeGraphIndex` by scanning a directory for `.md` files.
pub fn build_from_adr_dir(dir: &str) -> Result<KnowledgeGraphIndex, IndexError> {
    let mut index = KnowledgeGraphIndex::new();
    let path = Path::new(dir);
    if !path.exists() {
        return Err(IndexError::InvalidFormat);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&file_path)?;
        let path_str = file_path.to_string_lossy().to_string();
        let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let fallback_id = if stem.starts_with("ADR-") {
            stem.split('-').take(2).collect::<Vec<_>>().join("-")
        } else {
            String::new()
        };
        let mut patterns = extract_adr_patterns(&content);
        if let Some(pattern) = patterns.first_mut() {
            if pattern.id.is_empty() {
                pattern.id = fallback_id;
                pattern.shard_id = get_shard_id(&pattern.id);
            }
            pattern.path = path_str;
            index.insert(pattern.clone());
        }
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tempfile::TempDir;

    #[test]
    fn knowledge_graph_init() {
        let start = Instant::now();
        let index = KnowledgeGraphIndex::new();
        assert_eq!(index.shard_count(), NUM_SHARDS);
        assert!(start.elapsed().as_millis() < 1);
    }

    #[test]
    fn invalid_adr_format() {
        assert!(build_from_adr_dir("/nonexistent/path/12345").is_err());
    }

    #[test]
    fn search_adr_by_id() -> Result<(), IndexError> {
        let tmp = TempDir::new()?;
        let adr_path = tmp.path().join("ADR-001-test.md");
        fs::write(
            &adr_path,
            "---\ntitle: Test ADR\nid: ADR-001\n---\n\n# Introduction",
        )?;
        let index = build_from_adr_dir(tmp.path().to_str().unwrap())?;
        let found = index.search_by_id("ADR-001");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), adr_path.to_string_lossy());
        Ok(())
    }
}
