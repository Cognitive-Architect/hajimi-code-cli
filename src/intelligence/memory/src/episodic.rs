//! Episodic Memory: Time-series memory fragments for agent experience.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

const MAX_EPISODES: usize = 1000;

/// Episodic层错误类型
#[derive(Debug, thiserror::Error)]
pub enum EpisodicError {
    #[error("IO错误: {0}")]
    Io(#[from] io::Error),
    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("无法获取用户主目录")]
    NoHomeDir,
    #[error("无效的项目ID")]
    InvalidProjectId,
}

/// A single episode in agent's experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub content: String,
    pub outcome: String,
    pub confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Time-series episodic memory with JSONL persistence.
pub struct EpisodicMemory {
    episodes: Arc<Mutex<VecDeque<Episode>>>,
    storage_dir: Option<PathBuf>,
    jsonl_path: Option<PathBuf>,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        Self { episodes: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_EPISODES))), storage_dir: None, jsonl_path: None }
    }
    /// Create with JSONL persistence at ~/.hajimi/memory/{project_id}/episodes.jsonl
    pub fn new_with_persist(project_id: &str) -> Result<Self, EpisodicError> {
        if project_id.is_empty() || project_id.contains('/') || project_id.contains('\\') {
            return Err(EpisodicError::InvalidProjectId);
        }
        let home = dirs::home_dir().ok_or(EpisodicError::NoHomeDir)?;
        let storage_dir = home.join(".hajimi").join("memory").join(project_id);
        fs::create_dir_all(&storage_dir)?;
        let jsonl_path = storage_dir.join("episodes.jsonl");
        let mut mem = Self { episodes: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_EPISODES))), storage_dir: Some(storage_dir), jsonl_path: Some(jsonl_path.clone()) };
        mem.load_from_disk()?;
        Ok(mem)
    }
    /// Load episodes from JSONL on disk. Graceful if file does not exist.
    pub fn load_from_disk(&mut self) -> Result<(), EpisodicError> {
        let path = match &self.jsonl_path { Some(p) => p, None => return Ok(()) };
        if !path.exists() { return Ok(()); }
        let content = fs::read_to_string(path)?;
        let mut eps = self.episodes.lock().unwrap_or_else(|e| e.into_inner());
        eps.clear();
        for line in content.lines() {
            if line.trim().is_empty() { continue; }
            match serde_json::from_str::<Episode>(line) {
                Ok(ep) => { eps.push_back(ep); if eps.len() > MAX_EPISODES { eps.pop_front(); } }
                Err(_) => continue, // skip bad lines
            }
        }
        Ok(())
    }
    /// Atomically append an episode to JSONL (NamedTempFile + rename).
    pub fn append_to_jsonl(&self, episode: &Episode) -> Result<(), EpisodicError> {
        let (storage_dir, jsonl_path) = match (&self.storage_dir, &self.jsonl_path) { (Some(d), Some(p)) => (d, p), _ => return Ok(()) };
        let mut temp = NamedTempFile::new_in(storage_dir)?;
        if jsonl_path.exists() {
            let mut src = fs::File::open(jsonl_path)?;
            std::io::copy(&mut src, &mut temp)?;
        }
        writeln!(temp, "{}", serde_json::to_string(episode)?)?;
        temp.flush()?;
        fs::rename(temp.path(), jsonl_path)?;
        Ok(())
    }
    /// Record a new episode. Persists automatically if jsonl_path is set.
    pub fn record(&self, action_type: &str, content: &str, outcome: &str, confidence: f32) -> String {
        let episode = Episode { id: format!("ep_{}", uuid::Uuid::new_v4()), timestamp: chrono::Utc::now(), action: action_type.to_string(), content: content.to_string(), outcome: outcome.to_string(), confidence, metadata: None };
        let id = episode.id.clone();
        // SAFETY: Mutex is never poisoned in this context
        let mut eps = self.episodes.lock().unwrap_or_else(|e| e.into_inner());
        if eps.len() >= MAX_EPISODES { eps.pop_front(); }
        eps.push_back(episode.clone());
        drop(eps);
        let _ = self.append_to_jsonl(&episode);
        id
    }
    pub fn query_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Vec<Episode> {
        self.episodes.lock().unwrap_or_else(|e| e.into_inner()).iter().filter(|e| e.timestamp >= start && e.timestamp <= end).cloned().collect()
    }
    pub fn query_recent(&self, n: usize) -> Vec<Episode> {
        self.episodes.lock().unwrap_or_else(|e| e.into_inner()).iter().rev().take(n).cloned().collect()
    }
    pub fn query_by_keyword(&self, keyword: &str) -> Vec<Episode> {
        let eps = self.episodes.lock().unwrap_or_else(|e| e.into_inner());
        if keyword.is_empty() { return eps.iter().cloned().collect(); }
        let kw = keyword.to_lowercase();
        eps.iter().filter(|e| e.action.to_lowercase().contains(&kw) || e.content.to_lowercase().contains(&kw) || e.outcome.to_lowercase().contains(&kw)).cloned().collect()
    }
    pub fn len(&self) -> usize { self.episodes.lock().unwrap_or_else(|e| e.into_inner()).len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn export_all(&self) -> Vec<Episode> {
        self.episodes.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect()
    }
    pub fn import(&self, episodes: Vec<Episode>) {
        let mut eps = self.episodes.lock().unwrap_or_else(|e| e.into_inner());
        eps.clear();
        for ep in episodes { eps.push_back(ep); }
    }
}
impl Default for EpisodicMemory { fn default() -> Self { Self::new() } }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_record() { assert!(!EpisodicMemory::new().record("a","c","o",0.9).is_empty()); }
    #[test] fn test_query_range() { let m = EpisodicMemory::new(); let s = chrono::Utc::now(); m.record("a","c","o",0.9); assert_eq!(m.query_range(s, chrono::Utc::now()).len(), 1); }
    #[test] fn test_export_import() { let m1 = EpisodicMemory::new(); m1.record("a","c","o",0.9); let m2 = EpisodicMemory::new(); m2.import(m1.export_all()); assert_eq!(m2.len(), 1); }

    #[test]
    fn test_new_with_persist() -> Result<(), EpisodicError> {
        let m = EpisodicMemory::new_with_persist(&format!("test_ep_{}", uuid::Uuid::new_v4()))?;
        assert!(m.storage_dir.as_ref().unwrap().exists());
        assert!(EpisodicMemory::new_with_persist("").is_err());
        assert!(EpisodicMemory::new_with_persist("a/b").is_err());
        Ok(())
    }

    #[test]
    fn test_load_empty_and_persist() -> Result<(), EpisodicError> {
        let pid = format!("test_persist_{}", uuid::Uuid::new_v4());
        let mut m = EpisodicMemory::new_with_persist(&pid)?;
        m.load_from_disk()?;
        assert_eq!(m.len(), 0);
        let id = m.record("a1","c1","s",0.9);
        drop(m);
        let m2 = EpisodicMemory::new_with_persist(&pid)?;
        assert_eq!(m2.len(), 1);
        assert_eq!(m2.query_recent(1)[0].id, id);
        Ok(())
    }

    #[test]
    fn test_skip_bad_lines() -> Result<(), EpisodicError> {
        let pid = format!("test_bad_{}", uuid::Uuid::new_v4());
        let dir = dirs::home_dir().unwrap().join(".hajimi/memory").join(&pid);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("episodes.jsonl"), "{\"id\":\"g\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"action\":\"a\",\"content\":\"c\",\"outcome\":\"o\",\"confidence\":0.5}\nbad\n")?;
        let mut m = EpisodicMemory::new_with_persist(&pid)?;
        m.load_from_disk()?;
        assert_eq!(m.len(), 1);
        Ok(())
    }

    #[test]
    fn test_append_order() -> Result<(), EpisodicError> {
        let pid = format!("test_ord_{}", uuid::Uuid::new_v4());
        let m = EpisodicMemory::new_with_persist(&pid)?;
        m.record("f","c","o",1.0); m.record("s","c","o",1.0); m.record("t","c","o",1.0);
        drop(m);
        let m2 = EpisodicMemory::new_with_persist(&pid)?;
        let a = m2.export_all();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].action, "f"); assert_eq!(a[1].action, "s"); assert_eq!(a[2].action, "t");
        Ok(())
    }
    #[test] fn test_query_by_keyword() { let m = EpisodicMemory::new(); m.record("search","hello","ok",0.9); assert_eq!(m.query_by_keyword("search").len(),1); assert_eq!(m.query_by_keyword("").len(),1); assert!(m.query_by_keyword("xyz").is_empty()); }
    #[test] fn test_query_recent_zero() { assert!(EpisodicMemory::new().query_recent(0).is_empty()); }
    #[test] fn test_capacity_eviction() { let m = EpisodicMemory::new(); for i in 0..1001 { m.record(&format!("a{}",i),"c","o",0.5); } assert_eq!(m.len(),1000); assert_eq!(m.export_all()[0].action,"a1"); }
    #[test]
    fn test_episodic_roundtrip() -> Result<(),EpisodicError> {
        let pid = format!("rt_{}",uuid::Uuid::new_v4());
        let m1 = EpisodicMemory::new_with_persist(&pid)?; let id = m1.record("a","c","o",0.9); drop(m1);
        let m2 = EpisodicMemory::new_with_persist(&pid)?; assert_eq!(m2.len(),1); assert_eq!(m2.query_recent(1)[0].id,id); Ok(())
    }
}
