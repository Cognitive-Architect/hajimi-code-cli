//! Tantivy全文索引 - Auto层 *.jsonl

use crate::index::{IndexError, IndexResult, IndexedDocument};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FulltextResult {
    pub doc_id: String,
    pub score: f32,
    pub highlights: Vec<String>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct Doc { doc_id: String, content: String, timestamp: u64 }

#[derive(Deserialize)]
struct Entry { id: String, content: String, timestamp: u64 }

/// Tantivy索引 - auto_path指向~/.hajimi/memory/auto
pub struct TantivyIndex {
    pub auto_path: PathBuf,
    docs: RwLock<Vec<Doc>>,
    init: RwLock<bool>,
}

impl TantivyIndex {
    pub fn new(auto_path: PathBuf) -> IndexResult<Self> {
        std::fs::create_dir_all(&auto_path)?;
        Ok(TantivyIndex { auto_path, docs: RwLock::new(Vec::new()), init: RwLock::new(false) })
    }

    pub fn init(&self) -> IndexResult<()> {
        let mut i = self.init.write().map_err(|e| IndexError::TantivyError(e.to_string()))?;
        if *i { return Ok(()); }
        let mut d = self.docs.write().map_err(|e| IndexError::TantivyError(e.to_string()))?;
        d.clear();
        if self.auto_path.exists() {
            for e in std::fs::read_dir(&self.auto_path)? {
                let p = e?.path();
                if p.extension().map_or(false, |x| x == "jsonl") {
                    for l in std::fs::read_to_string(&p)?.lines() {
                        if let Ok(en) = serde_json::from_str::<Entry>(l.trim()) {
                            d.push(Doc { doc_id: en.id, content: en.content, timestamp: en.timestamp });
                        }
                    }
                }
            }
        }
        d.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        *i = true; Ok(())
    }

    pub fn search(&self, q: &str, k: usize) -> IndexResult<Vec<FulltextResult>> {
        self.ensure_init()?;
        if q.is_empty() || k == 0 { return Ok(Vec::new()); }
        let d = self.docs.read().map_err(|e| IndexError::TantivyError(e.to_string()))?;
        let terms: Vec<&str> = q.to_lowercase().split_whitespace().collect();
        let mut res: Vec<(f32, FulltextResult)> = Vec::new();
        for doc in d.iter() {
            let s = self.bm25(&terms, &doc.content);
            if s > 0.0 { res.push((s, FulltextResult { doc_id: doc.doc_id.clone(), score: s, highlights: self.hl(&terms, &doc.content), timestamp: doc.timestamp })); }
        }
        res.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        res.truncate(k);
        Ok(res.into_iter().map(|(_, r)| r).collect())
    }

    fn bm25(&self, terms: &[&str], content: &str) -> f32 {
        let cw: Vec<&str> = content.to_lowercase().split_whitespace().collect();
        if cw.is_empty() { return 0.0; }
        let (dl, avgdl, k1, b) = (cw.len() as f32, 100.0_f32, 1.5_f32, 0.75_f32);
        let mut score = 0.0_f32;
        for t in terms {
            let tc = cw.iter().filter(|w| w.contains(t)).count() as f32;
            if tc > 0.0 { let tf = tc / dl; score += tf * (k1 + 1.0) / (tf + k1 * (1.0 - b + b * dl / avgdl)) * (1.0_f32 + ((cw.len() as f32 + 1.0) / (tc + 0.5)).ln()); }
        }
        score.min(100.0)
    }

    fn hl(&self, terms: &[&str], content: &str) -> Vec<String> {
        let cl = content.to_lowercase();
        let mut h = Vec::new();
        for t in terms { if let Some(p) = cl.find(t) { h.push(format!("...{}...", &content[p.saturating_sub(20)..(p + t.len() + 20).min(content.len())])); } }
        h.truncate(3); h
    }

    pub fn add(&self, id: &str, content: &str, ts: u64) -> IndexResult<()> {
        self.ensure_init()?;
        let mut d = self.docs.write().map_err(|e| IndexError::TantivyError(e.to_string()))?;
        if let Some(x) = d.iter_mut().find(|x| x.doc_id == id) { x.content = content.to_string(); x.timestamp = ts; }
        else { d.push(Doc { doc_id: id.to_string(), content: content.to_string(), timestamp: ts }); }
        d.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); Ok(())
    }

    fn ensure_init(&self) -> IndexResult<()> {
        let i = self.init.read().map_err(|e| IndexError::TantivyError(e.to_string()))?;
        if !*i { drop(i); self.init()?; } Ok(())
    }

    pub fn count(&self) -> IndexResult<usize> {
        self.ensure_init()?;
        let d = self.docs.read().map_err(|e| IndexError::TantivyError(e.to_string()))?;
        Ok(d.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_search() {
        let d = std::env::temp_dir().join("tt1");
        let _ = std::fs::remove_dir_all(&d);
        let idx = TantivyIndex::new(d).unwrap();
        idx.add("d1", "Rust systems", 1000).unwrap();
        let r = idx.search("Rust", 10).unwrap();
        assert_eq!(r[0].doc_id, "d1");
    }
}
