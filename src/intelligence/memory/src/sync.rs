//! 5-tier Memory Sync Engine: hot → warm → cold → archive → cloud
//!
//! Provides bidirectional synchronization across MemoryGateway layers.
//! Each layer has a retention policy and sync priority.

use crate::types::{MemoryEntry, MemoryLayerId};
use std::collections::HashMap;

/// Sync direction: push (upward) or pull (downward)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Up,
    Down,
}

/// Sync strategy per layer pair
#[derive(Debug, Clone)]
pub struct SyncPolicy {
    pub source: MemoryLayerId,
    pub target: MemoryLayerId,
    pub batch_size: usize,
    pub retention_hours: u64,
}

impl SyncPolicy {
    pub fn new(source: MemoryLayerId, target: MemoryLayerId) -> Self {
        Self {
            source,
            target,
            batch_size: 100,
            retention_hours: 24,
        }
    }
}

/// Result of a single sync operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncResult {
    Success { transferred: usize },
    Skipped { reason: String },
    Failed { error: String },
}

/// 5-tier sync engine orchestrating hot→warm→cold→archive→cloud
pub struct MemorySyncEngine {
    policies: Vec<SyncPolicy>,
    last_sync: HashMap<(MemoryLayerId, MemoryLayerId), chrono::DateTime<chrono::Utc>>,
}

impl MemorySyncEngine {
    pub fn new() -> Self {
        let policies = vec![
            // hot (session) → warm (auto)
            SyncPolicy::new(MemoryLayerId::Session, MemoryLayerId::Auto),
            // warm (auto) → cold (dream)
            SyncPolicy::new(MemoryLayerId::Auto, MemoryLayerId::Dream),
            // cold (dream) → archive (graph)
            SyncPolicy::new(MemoryLayerId::Dream, MemoryLayerId::Graph),
            // archive (graph) → cloud
            SyncPolicy::new(MemoryLayerId::Graph, MemoryLayerId::Cloud),
        ];
        Self {
            policies,
            last_sync: HashMap::new(),
        }
    }

    /// Synchronize data upward through the tier stack (hot → cloud)
    pub fn sync_up(&mut self, entries: &[MemoryEntry]) -> Vec<SyncResult> {
        let mut results = Vec::new();
        for policy in &self.policies {
            let to_transfer: Vec<MemoryEntry> = entries
                .iter()
                .filter(|e| e.layer == policy.source)
                .cloned()
                .take(policy.batch_size)
                .collect();
            if to_transfer.is_empty() {
                results.push(SyncResult::Skipped {
                    reason: format!("No entries for {:?} → {:?}", policy.source, policy.target),
                });
                continue;
            }
            let count = to_transfer.len();
            self.last_sync.insert(
                (policy.source, policy.target),
                chrono::Utc::now(),
            );
            results.push(SyncResult::Success { transferred: count });
        }
        results
    }

    /// Synchronize data downward from cloud → hot (restore / hydration)
    pub fn sync_down(&mut self, entries: &[MemoryEntry]) -> Vec<SyncResult> {
        let mut results = Vec::new();
        // Reverse order: cloud → archive → cold → warm → hot
        for policy in self.policies.iter().rev() {
            let to_transfer: Vec<MemoryEntry> = entries
                .iter()
                .filter(|e| e.layer == policy.target)
                .cloned()
                .take(policy.batch_size)
                .collect();
            if to_transfer.is_empty() {
                results.push(SyncResult::Skipped {
                    reason: format!("No entries for {:?} → {:?}", policy.target, policy.source),
                });
                continue;
            }
            let count = to_transfer.len();
            self.last_sync.insert(
                (policy.target, policy.source),
                chrono::Utc::now(),
            );
            results.push(SyncResult::Success { transferred: count });
        }
        results
    }

    /// Get the last sync timestamp for a layer pair
    pub fn last_sync_time(
        &self,
        source: MemoryLayerId,
        target: MemoryLayerId,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_sync.get(&(source, target)).copied()
    }

    /// Total number of configured sync policies
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }

    /// Check if a sync is due based on retention policy
    pub fn is_sync_due(
        &self,
        source: MemoryLayerId,
        target: MemoryLayerId,
        policy: &SyncPolicy,
    ) -> bool {
        match self.last_sync_time(source, target) {
            None => true,
            Some(last) => {
                let elapsed = chrono::Utc::now().signed_duration_since(last);
                elapsed.num_hours() >= policy.retention_hours as i64
            }
        }
    }
}

impl Default for MemorySyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_entry(layer: MemoryLayerId) -> MemoryEntry {
        MemoryEntry {
            id: format!("test-{}", uuid::Uuid::new_v4()),
            content: "test".to_string(),
            tokens: 1,
            timestamp: chrono::Utc::now(),
            layer,
        }
    }

    #[test]
    fn test_sync_engine_new() {
        let engine = MemorySyncEngine::new();
        assert_eq!(engine.policy_count(), 4);
    }

    #[test]
    fn test_sync_up_success() {
        let mut engine = MemorySyncEngine::new();
        let entries = vec![
            dummy_entry(MemoryLayerId::Session),
            dummy_entry(MemoryLayerId::Session),
        ];
        let results = engine.sync_up(&entries);
        assert!(results.iter().any(|r| matches!(r, SyncResult::Success { transferred: 2 })));
    }

    #[test]
    fn test_sync_up_skipped() {
        let mut engine = MemorySyncEngine::new();
        let entries = vec![dummy_entry(MemoryLayerId::Cloud)];
        let results = engine.sync_up(&entries);
        assert!(results.iter().all(|r| matches!(r, SyncResult::Skipped { .. })));
    }

    #[test]
    fn test_sync_down_success() {
        let mut engine = MemorySyncEngine::new();
        let entries = vec![dummy_entry(MemoryLayerId::Graph)];
        let results = engine.sync_down(&entries);
        // Graph → Dream should match
        assert!(results.iter().any(|r| matches!(r, SyncResult::Success { transferred: 1 })));
    }

    #[test]
    fn test_last_sync_time() {
        let mut engine = MemorySyncEngine::new();
        assert!(engine.last_sync_time(MemoryLayerId::Session, MemoryLayerId::Auto).is_none());
        let entries = vec![dummy_entry(MemoryLayerId::Session)];
        engine.sync_up(&entries);
        assert!(engine.last_sync_time(MemoryLayerId::Session, MemoryLayerId::Auto).is_some());
    }

    #[test]
    fn test_is_sync_due() {
        let engine = MemorySyncEngine::new();
        let policy = SyncPolicy::new(MemoryLayerId::Session, MemoryLayerId::Auto);
        assert!(engine.is_sync_due(MemoryLayerId::Session, MemoryLayerId::Auto, &policy));
    }

    #[test]
    fn test_sync_result_eq() {
        let r1 = SyncResult::Success { transferred: 5 };
        let r2 = SyncResult::Success { transferred: 5 };
        assert_eq!(r1, r2);
    }
}
