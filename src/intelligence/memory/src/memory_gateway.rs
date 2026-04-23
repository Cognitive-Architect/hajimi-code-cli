//! 5-tier Memory Gateway: Session -> Auto -> Dream -> Graph -> Cloud
use crate::auto::AutoMemory;
use crate::cloud::CloudMemory;
use crate::dream::DreamMemory;
use crate::episodic::EpisodicMemory;
use crate::graph::GraphMemory;
use crate::session::SessionMemory;
use crate::sync::{MemorySyncEngine, SyncResult};
use crate::types::{MemoryEntry, MemoryLayerId};

/// Lightweight 6-tier memory gateway (Session/Auto/Dream/Graph/Cloud/Episodic).
pub struct MemoryGateway {
    pub session: SessionMemory,
    pub auto: Option<AutoMemory>,
    pub dream: Option<DreamMemory>,
    pub graph: Option<GraphMemory>,
    pub cloud: Option<CloudMemory>,
    pub episodic: Option<EpisodicMemory>,
    pub sync_engine: MemorySyncEngine,
}

impl MemoryGateway {
    pub fn new(device_id: &str) -> Self {
        Self {
            session: SessionMemory::new(),
            auto: None,
            dream: None,
            graph: None,
            cloud: Some(CloudMemory::new(device_id)),
            episodic: None,
            sync_engine: MemorySyncEngine::new(),
        }
    }

    pub fn enable_episodic(&mut self) { self.episodic = Some(EpisodicMemory::new()); }

    pub fn enable_auto(&mut self, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.auto = Some(AutoMemory::new(project_id)?);
        Ok(())
    }

    pub fn enable_dream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.dream = Some(DreamMemory::new("dream_project")?);
        Ok(())
    }

    pub fn enable_graph(&mut self) {
        self.graph = Some(GraphMemory::new());
    }

    pub fn enable_cloud(&mut self, device_id: &str) {
        self.cloud = Some(CloudMemory::new(device_id));
    }

    /// Push a vector through Session -> Auto -> Dream -> Graph -> Cloud.
    pub fn push_vector(&mut self, key: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.session.insert(key.to_string(), content.to_string())?;

        if let Some(auto) = self.auto.as_mut() {
            if let Some(entry) = self.session.get(key) {
                auto.insert(key.to_string(), entry.clone())?;
            }
        }

        if let Some(dream) = self.dream.as_mut() {
            if let Some(entry) = self.session.get(key) {
                let embedding = vec![0.0f32; crate::dream::EMBEDDING_DIM];
                let _ = dream.insert(key, content, entry.tokens, &embedding);
            }
        }

        if let Some(graph) = self.graph.as_mut() {
            let entry = MemoryEntry {
                id: key.to_string(),
                content: content.to_string(),
                tokens: content.len(),
                timestamp: chrono::Utc::now(),
                layer: MemoryLayerId::Graph,
            };
            let _ = graph.store(entry);
        }

        Ok(())
    }

    pub fn layer_count(&self) -> usize {
        let mut count = 1; // session always present
        if self.auto.is_some() { count += 1; }
        if self.dream.is_some() { count += 1; }
        if self.graph.is_some() { count += 1; }
        if self.cloud.is_some() { count += 1; }
        if self.episodic.is_some() { count += 1; }
        count
    }

    /// Record episode to EpisodicMemory layer.
    pub fn record_episode(&self, action_type: &str, content: &str, outcome: &str, confidence: f32) -> Option<String> {
        self.episodic.as_ref().map(|e| e.record(action_type, content, outcome, confidence))
    }

    /// Synchronize entries upward through the tier stack (hot → cloud).
    pub fn sync_up(&mut self, entries: &[MemoryEntry]) -> Vec<SyncResult> {
        self.sync_engine.sync_up(entries)
    }

    /// Synchronize entries downward from cloud → hot (restore / hydration).
    pub fn sync_down(&mut self, entries: &[MemoryEntry]) -> Vec<SyncResult> {
        self.sync_engine.sync_down(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_6_tier_init() {
        let mut gw = MemoryGateway::new("test_device");
        assert!(gw.auto.is_none());
        gw.enable_auto("test_project").unwrap();
        gw.enable_dream().unwrap();
        gw.enable_graph();
        gw.enable_episodic();
        assert_eq!(gw.layer_count(), 6);
    }

    #[test]
    fn test_record_episode() {
        let mut gw = MemoryGateway::new("test_device");
        gw.enable_episodic();
        let id = gw.record_episode("action", "content", "success", 0.9);
        assert!(id.is_some());
    }

    #[test]
    fn test_push_vector_cascade() {
        let mut gw = MemoryGateway::new("test_device");
        gw.enable_auto("test_project").unwrap();
        gw.enable_dream().unwrap();
        gw.enable_graph();
        assert!(gw.push_vector("k1", "hello cascade").is_ok());
        assert!(gw.session.get("k1").is_some());
    }
}
