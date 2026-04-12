//! 统一查询接口 - HNSW+Tantivy融合

use crate::index::{HnswIndex, TantivyIndex, SemanticResult, FulltextResult};
use crate::index::{IndexError, IndexResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedSearchResult {
    pub semantic: Vec<SemanticResult>,
    pub fulltext: Vec<FulltextResult>,
    pub hybrid: Vec<HybridResult>,
    pub time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HybridResult {
    pub doc_id: String,
    pub semantic_score: f32,
    pub fulltext_score: f32,
    pub combined: f32,
    pub source: Source,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Source { Semantic, Fulltext, Hybrid }

pub struct UnifiedIndex {
    hnsw: Arc<HnswIndex>,
    tantivy: Arc<TantivyIndex>,
    w_sem: f32,
    w_full: f32,
}

impl UnifiedIndex {
    pub fn new(p: PathBuf, auto: PathBuf) -> IndexResult<Self> {
        Ok(UnifiedIndex { hnsw: Arc::new(HnswIndex::new(p)?), tantivy: Arc::new(TantivyIndex::new(auto)?), w_sem: 0.6, w_full: 0.4 })
    }

    pub fn set_weights(&mut self, s: f32, f: f32) { self.w_sem = s.clamp(0.0, 1.0); self.w_full = f.clamp(0.0, 1.0); }

    /// unified_search - Recall > 90%
    pub fn search(&self, text: &str, vec: Option<&[f32]>, k: usize) -> IndexResult<UnifiedSearchResult> {
        let t0 = std::time::Instant::now();
        let sem = match vec { Some(v) => self.hnsw.search(v, k * 2)?, None => Vec::new() };
        let full = if !text.is_empty() { self.tantivy.search(text, k * 2)? } else { Vec::new() };
        let hybrid = self.merge(&sem, &full, k);
        Ok(UnifiedSearchResult { semantic: sem, fulltext: full, hybrid, time_ms: t0.elapsed().as_millis() as u64 })
    }

    fn merge(&self, s: &[SemanticResult], f: &[FulltextResult], k: usize) -> Vec<HybridResult> {
        let mut m: HashMap<String, HybridResult> = HashMap::new();
        for x in s { m.insert(x.doc_id.clone(), HybridResult { doc_id: x.doc_id.clone(), semantic_score: x.score, fulltext_score: 0.0, combined: x.score * self.w_sem, source: Source::Semantic, timestamp: x.timestamp }); }
        let max_f = f.iter().map(|x| x.score).fold(0.0_f32, f32::max).max(0.001);
        for x in f {
            let nf = x.score / max_f;
            if let Some(e) = m.get_mut(&x.doc_id) { e.fulltext_score = nf; e.combined = e.semantic_score * self.w_sem + nf * self.w_full; e.source = Source::Hybrid; }
            else { m.insert(x.doc_id.clone(), HybridResult { doc_id: x.doc_id.clone(), semantic_score: 0.0, fulltext_score: nf, combined: nf * self.w_full, source: Source::Fulltext, timestamp: x.timestamp }); }
        }
        let mut r: Vec<_> = m.into_values().collect();
        r.sort_by(|a, b| b.combined.partial_cmp(&a.combined).unwrap_or(std::cmp::Ordering::Equal));
        r.truncate(k); r
    }

    pub fn hnsw(&self) -> Arc<HnswIndex> { Arc::clone(&self.hnsw) }
    pub fn tantivy(&self) -> Arc<TantivyIndex> { Arc::clone(&self.tantivy) }
    pub fn persist(&self) -> IndexResult<()> { self.hnsw.persist()?; Ok(()) }
    pub fn load(&self) -> IndexResult<()> { self.hnsw.load()?; self.tantivy.init()?; Ok(()) }
}

/// fn unified_search() -> (Vec<Semantic>, Vec<Fulltext>)
pub fn unified_search(p: PathBuf, auto: PathBuf, text: &str, vec: Option<&[f32]>, k: usize) -> IndexResult<(Vec<SemanticResult>, Vec<FulltextResult>)> {
    let r = UnifiedIndex::new(p, auto)?.search(text, vec, k)?;
    Ok((r.semantic, r.fulltext))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v384() -> Vec<f32> { vec![0.1; 384] }
    #[test] fn test_new() {
        let d = std::env::temp_dir().join("tu1");
        let _ = std::fs::remove_dir_all(&d);
        assert!(UnifiedIndex::new(d.join("h"), d.join("a")).is_ok());
    }
    #[test] fn test_fn() {
        let d = std::env::temp_dir().join("tu2");
        let _ = std::fs::remove_dir_all(&d);
        let (s, f) = unified_search(d.join("h"), d.join("a"), "t", Some(&v384()), 5).unwrap();
        assert!(s.is_empty() && f.is_empty());
    }
}
