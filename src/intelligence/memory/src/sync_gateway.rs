//! SyncMemoryGateway - Cross-tier memory retrieval and persistence abstraction.
//!
//! Provides unified interface for AgentLoop to retrieve from and push to
//! the 5-tier memory architecture (Session → Auto → Dream → Graph → Cloud).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::types::{MemoryEntry, MemoryLayerId};

/// Memory tier enumeration for gateway operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTier {
    Session,
    Auto,
    Dream,
    Graph,
    Cloud,
}

impl MemoryTier {
    /// Returns the fallback tier order for retrieval (hot → cold).
    pub fn fallback_order() -> &'static [MemoryTier] {
        &[MemoryTier::Session, MemoryTier::Auto, MemoryTier::Dream, MemoryTier::Graph, MemoryTier::Cloud]
    }
    /// Returns relevance score for sorting multi-tier results.
    pub fn score(&self) -> u8 {
        match self { MemoryTier::Session => 100, MemoryTier::Auto => 80, MemoryTier::Dream => 60, MemoryTier::Graph => 40, MemoryTier::Cloud => 20 }
    }
}

impl From<MemoryLayerId> for MemoryTier {
    fn from(l: MemoryLayerId) -> Self {
        match l { MemoryLayerId::Session => MemoryTier::Session, MemoryLayerId::Auto => MemoryTier::Auto, MemoryLayerId::Dream => MemoryTier::Dream, MemoryLayerId::Graph => MemoryTier::Graph, MemoryLayerId::Cloud => MemoryTier::Cloud }
    }
}

impl From<MemoryTier> for MemoryLayerId {
    fn from(t: MemoryTier) -> Self {
        match t { MemoryTier::Session => MemoryLayerId::Session, MemoryTier::Auto => MemoryLayerId::Auto, MemoryTier::Dream => MemoryLayerId::Dream, MemoryTier::Graph => MemoryLayerId::Graph, MemoryTier::Cloud => MemoryLayerId::Cloud }
    }
}

/// Generic event for gateway persistence.
/// Decoupled from upstream event types to prevent circular dependencies.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GatewayEvent {
    pub event_type: String,
    pub payload: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
}

impl GatewayEvent {
    /// Create a new gateway event with the current timestamp.
    pub fn new(et: impl Into<String>, p: impl Into<String>, s: impl Into<String>) -> Self {
        Self { event_type: et.into(), payload: p.into(), source: s.into(), timestamp: Utc::now() }
    }
}

/// Snapshot of blackboard state for synchronization.
#[derive(Debug, Clone, Default)]
pub struct BlackboardSnapshot {
    pub entries: HashMap<String, String>,
}

impl BlackboardSnapshot {
    /// Create a new empty snapshot.
    pub fn new() -> Self { Self::default() }
}

/// Health status for a single memory tier.
#[derive(Debug, Clone)]
pub struct TierHealth {
    pub tier: MemoryTier,
    pub available: bool,
    pub entry_count: usize,
    pub last_sync: Option<DateTime<Utc>>,
}

impl TierHealth {
    /// Create a new tier health report.
    pub fn new(tier: MemoryTier, available: bool, entry_count: usize) -> Self {
        Self { tier, available, entry_count, last_sync: None }
    }
}

/// Errors from SyncMemoryGateway operations.
#[derive(Debug, thiserror::Error)]
pub enum SyncGatewayError {
    #[error("Tier not available: {0:?}")]
    TierNotAvailable(MemoryTier),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid tier value: {0}")]
    InvalidTier(String),
}

/// Cross-tier memory gateway trait.
/// Abstracts retrieval and persistence across the 5-tier memory stack.
#[async_trait]
pub trait SyncMemoryGateway: Send {
    /// Retrieve memories from a specific tier by query string.
    async fn retrieve_from_tier(&mut self, tier: MemoryTier, query: &str) -> Result<Vec<MemoryEntry>, SyncGatewayError>;
    /// Persist a gateway event to the appropriate tier(s).
    async fn push_event(&mut self, event: GatewayEvent) -> Result<(), SyncGatewayError>;
    /// Synchronize blackboard snapshot with memory tiers.
    async fn sync_with_blackboard(&mut self, snapshot: &BlackboardSnapshot) -> Result<(), SyncGatewayError>;
    /// Retrieve from multiple tiers in priority order.
    async fn retrieve_multi(&mut self, tiers: &[MemoryTier], query: &str) -> Result<Vec<(MemoryTier, Vec<MemoryEntry>)>, SyncGatewayError>;
    /// Check health status of a specific tier.
    async fn tier_health(&mut self, tier: MemoryTier) -> Result<TierHealth, SyncGatewayError>;
}

/// Thread-safe handle type used by AgentLoop for concurrent access.
pub type SyncGatewayHandle = std::sync::Arc<tokio::sync::Mutex<dyn SyncMemoryGateway>>;

#[async_trait]
impl SyncMemoryGateway for crate::memory_gateway::MemoryGateway {
    async fn retrieve_from_tier(&mut self, tier: MemoryTier, query: &str) -> Result<Vec<MemoryEntry>, SyncGatewayError> {
        if query.is_empty() { return Ok(Vec::new()); }
        match tier {
            MemoryTier::Session => Ok(self.session.get_mut(query).map(|e| vec![MemoryEntry::new(query.into(), e.content.clone(), e.tokens, MemoryLayerId::Session)]).unwrap_or_default()),
            MemoryTier::Auto => Ok(self.auto.as_mut().and_then(|a| a.get(query)).map(|e| vec![MemoryEntry::new(query.into(), e.session_entry.content.clone(), e.session_entry.tokens, MemoryLayerId::Auto)]).unwrap_or_default()),
            MemoryTier::Dream => self.dream.as_ref().map_or(Err(SyncGatewayError::TierNotAvailable(MemoryTier::Dream)), |d| match d.search(&vec![0.0f32; crate::dream::EMBEDDING_DIM], 5) { Ok(r) => Ok(r.into_iter().map(|e| MemoryEntry::new(e.auto_entry.session_entry.content.clone(), e.auto_entry.session_entry.content.clone(), e.auto_entry.session_entry.tokens, MemoryLayerId::Dream)).collect()), Err(crate::dream::DreamError::InvalidDimension { .. }) => Err(SyncGatewayError::StorageError("InvalidDimension".to_string())), Err(e) => Err(SyncGatewayError::StorageError(e.to_string())) }),
            MemoryTier::Graph => self.graph.as_mut().map_or(Err(SyncGatewayError::TierNotAvailable(MemoryTier::Graph)), |graph| graph.recall(query).map(|r| r.into_iter().map(|n| MemoryEntry::new(n.id.clone(), n.name, 10, MemoryLayerId::Graph)).collect()).map_err(|e| SyncGatewayError::StorageError(e.to_string()))),
            MemoryTier::Cloud => Err(SyncGatewayError::TierNotAvailable(MemoryTier::Cloud)),
        }
    }
    async fn push_event(&mut self, event: GatewayEvent) -> Result<(), SyncGatewayError> {
        let json = serde_json::to_string(&event).map_err(|e| SyncGatewayError::StorageError(e.to_string()))?;
        self.push_vector(&format!("evt_{}", event.timestamp.timestamp()), &json).map_err(|e| SyncGatewayError::StorageError(e.to_string()))
    }
    async fn sync_with_blackboard(&mut self, snapshot: &BlackboardSnapshot) -> Result<(), SyncGatewayError> {
        for (k, v) in &snapshot.entries { self.push_vector(k, v).map_err(|e| SyncGatewayError::StorageError(e.to_string()))?; }
        Ok(())
    }
    async fn retrieve_multi(&mut self, tiers: &[MemoryTier], query: &str) -> Result<Vec<(MemoryTier, Vec<MemoryEntry>)>, SyncGatewayError> {
        let mut r = Vec::new();
        for t in tiers { match self.retrieve_from_tier(*t, query).await { Ok(e) if !e.is_empty() => r.push((*t, e)), Ok(_) | Err(SyncGatewayError::TierNotAvailable(_)) => continue, Err(e) => return Err(e), } }
        r.sort_by(|a, b| b.0.score().cmp(&a.0.score()));
        Ok(r)
    }
    async fn tier_health(&mut self, tier: MemoryTier) -> Result<TierHealth, SyncGatewayError> {
        let (a, c) = match tier {
            MemoryTier::Session => (true, self.session.len()),
            MemoryTier::Auto => (self.auto.is_some(), self.auto.as_ref().map(|a| a.len()).unwrap_or(0)),
            MemoryTier::Dream => (self.dream.is_some(), self.dream.as_ref().and_then(|d| d.len().ok()).unwrap_or(0)),
            MemoryTier::Graph => (self.graph.is_some(), self.graph.as_ref().map(|g| g.node_count()).unwrap_or(0)),
            MemoryTier::Cloud => (self.cloud.is_some(), 0),
        };
        Ok(TierHealth::new(tier, a, c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn gw() -> crate::memory_gateway::MemoryGateway {
        let mut g = crate::memory_gateway::MemoryGateway::new("t");
        g.enable_auto("t").unwrap(); g.enable_dream().unwrap(); g.enable_graph(); g
    }
    fn rt() -> tokio::runtime::Runtime { tokio::runtime::Runtime::new().unwrap() }
    #[test]
    fn test_session_hit() { let mut g = gw(); g.session.insert("k1".into(), "v1".into()).unwrap(); assert_eq!(rt().block_on(g.retrieve_from_tier(MemoryTier::Session, "k1")).unwrap().len(), 1); }
    #[test]
    fn test_session_miss() { assert!(rt().block_on(gw().retrieve_from_tier(MemoryTier::Session, "x")).unwrap().is_empty()); }
    #[test]
    fn test_session_empty() { assert!(rt().block_on(gw().retrieve_from_tier(MemoryTier::Session, "")).unwrap().is_empty()); }
    #[test]
    fn test_auto_miss() { assert!(rt().block_on(gw().retrieve_from_tier(MemoryTier::Auto, "x")).unwrap().is_empty()); }
    #[test]
    fn test_dream_available() { assert!(rt().block_on(gw().retrieve_from_tier(MemoryTier::Dream, "q")).is_ok()); }
    #[test]
    fn test_graph_empty() { assert!(rt().block_on(gw().retrieve_from_tier(MemoryTier::Graph, "q")).unwrap().is_empty()); }
    #[test]
    fn test_cloud_unavailable() { assert!(matches!(rt().block_on(gw().retrieve_from_tier(MemoryTier::Cloud, "q")), Err(SyncGatewayError::TierNotAvailable(_)))); }
    #[test]
    fn test_multi_fallback() { let mut g = gw(); g.session.insert("k1".into(), "v1".into()).unwrap(); let r = rt().block_on(g.retrieve_multi(MemoryTier::fallback_order(), "k1")).unwrap(); assert!(!r.is_empty()); assert_eq!(r[0].0, MemoryTier::Session); }
    #[test]
    fn test_push_event() { assert!(rt().block_on(gw().push_event(GatewayEvent::new("t", "p", "s"))).is_ok()); }
    #[test]
    fn test_sync_blackboard() { let mut s = BlackboardSnapshot::new(); s.entries.insert("k1".into(), "v1".into()); assert!(rt().block_on(gw().sync_with_blackboard(&s)).is_ok()); }
    #[test]
    fn test_tier_health() { assert!(rt().block_on(gw().tier_health(MemoryTier::Session)).unwrap().available); }
    #[test]
    fn test_access_count() {
        let mut g = gw(); g.session.insert("k1".into(), "v1".into()).unwrap();
        let _ = rt().block_on(g.retrieve_from_tier(MemoryTier::Session, "k1"));
        assert_eq!(g.session.get("k1").unwrap().access_count, 1);
        let _ = rt().block_on(g.retrieve_from_tier(MemoryTier::Session, "k1"));
        assert_eq!(g.session.get("k1").unwrap().access_count, 2);
    }
}
