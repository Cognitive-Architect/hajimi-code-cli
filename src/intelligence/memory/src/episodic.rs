//! Episodic Memory: Time-series memory fragments for agent experience.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const MAX_EPISODES: usize = 1000;

/// A single episode in agent's experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action_type: String,
    pub content: String,
    pub outcome: String,
    pub confidence: f32,
}

/// Time-series episodic memory with pruning.
pub struct EpisodicMemory {
    episodes: Arc<Mutex<VecDeque<Episode>>>,
}

impl EpisodicMemory {
    pub fn new() -> Self { Self { episodes: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_EPISODES))) } }

    /// Record a new episode.
    pub fn record(&self, action_type: &str, content: &str, outcome: &str, confidence: f32) -> String {
        let episode = Episode { id: format!("ep_{}", uuid::Uuid::new_v4()), timestamp: chrono::Utc::now(), action_type: action_type.to_string(), content: content.to_string(), outcome: outcome.to_string(), confidence };
        let id = episode.id.clone();
        let mut eps = self.episodes.lock().unwrap();
        if eps.len() >= MAX_EPISODES { eps.pop_front(); }
        eps.push_back(episode); id
    }

    /// Query episodes in time range.
    pub fn query_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Vec<Episode> {
        self.episodes.lock().unwrap().iter().filter(|e| e.timestamp >= start && e.timestamp <= end).cloned().collect()
    }

    /// Query recent N episodes.
    pub fn query_recent(&self, n: usize) -> Vec<Episode> { self.episodes.lock().unwrap().iter().rev().take(n).cloned().collect() }

    /// Get episode count.
    pub fn len(&self) -> usize { self.episodes.lock().unwrap().len() }

    /// Export all for checkpoint.
    pub fn export_all(&self) -> Vec<Episode> { self.episodes.lock().unwrap().iter().cloned().collect() }

    /// Import from checkpoint.
    pub fn import(&self, episodes: Vec<Episode>) { let mut eps = self.episodes.lock().unwrap(); eps.clear(); for ep in episodes { eps.push_back(ep); } }
}

impl Default for EpisodicMemory { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_record() { let mem = EpisodicMemory::new(); assert!(!mem.record("act", "c", "ok", 0.9).is_empty()); }
    #[test] fn test_query_range() { let mem = EpisodicMemory::new(); let s = chrono::Utc::now(); mem.record("a", "c", "o", 0.9); assert_eq!(mem.query_range(s, chrono::Utc::now()).len(), 1); }
    #[test] fn test_export_import() { let m1 = EpisodicMemory::new(); m1.record("a", "c", "o", 0.9); let m2 = EpisodicMemory::new(); m2.import(m1.export_all()); assert_eq!(m2.len(), 1); }
}
