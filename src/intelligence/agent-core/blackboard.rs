//! Blackboard: Shared state pattern for inter-agent communication.
//! Day 6: Central state storage with conflict resolution and subscription.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::warn;

/// Blackboard entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardEntry {
    pub key: String,
    pub value: String,
    pub timestamp: u64,
    pub agent_id: String,
    pub version: u64,
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    LastWriteWins,
    FirstWriteWins,
    Merge,
}

/// Blackboard events.
#[derive(Debug, Clone)]
pub enum BlackboardEvent {
    EntryAdded(String, BlackboardEntry),
    EntryUpdated(String, BlackboardEntry),
    EntryRemoved(String),
}

/// Subscription handle.
pub struct Subscription {
    pub id: String,
    pub pattern: String,
    pub tx: mpsc::Sender<BlackboardEvent>,
}

/// Central blackboard for agent state sharing.
pub struct Blackboard {
    state: Arc<RwLock<HashMap<String, BlackboardEntry>>>,
    subs: Arc<RwLock<Vec<Subscription>>>,
    strategy: ConflictStrategy,
    counter: Arc<RwLock<u64>>,
}

impl std::fmt::Debug for Blackboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blackboard")
            .field("strategy", &self.strategy)
            .finish_non_exhaustive()
    }
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            subs: Arc::new(RwLock::new(Vec::new())),
            strategy: ConflictStrategy::LastWriteWins,
            counter: Arc::new(RwLock::new(0)),
        }
    }
    pub fn with_strategy(s: ConflictStrategy) -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            subs: Arc::new(RwLock::new(Vec::new())),
            strategy: s,
            counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Read entry by key.
    pub async fn read(&self, key: &str) -> Option<BlackboardEntry> {
        self.state.read().await.get(key).cloned()
    }

    /// Write entry with conflict resolution.
    pub async fn write(&self, key: &str, value: &str, agent_id: &str) {
        let mut c = self.counter.write().await;
        *c += 1;
        let v = *c;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_millis() as u64;
        let new = BlackboardEntry {
            key: key.to_string(),
            value: value.to_string(),
            timestamp: ts,
            agent_id: agent_id.to_string(),
            version: v,
        };
        let mut s = self.state.write().await;
        let evt = if let Some(e) = s.get(key) {
            let r = self.resolve(e, &new);
            s.insert(key.to_string(), r.clone());
            BlackboardEvent::EntryUpdated(key.to_string(), r)
        } else {
            s.insert(key.to_string(), new.clone());
            BlackboardEvent::EntryAdded(key.to_string(), new)
        };
        drop(s);
        self.notify(key, evt).await;
    }

    /// Subscribe to key pattern.
    pub async fn subscribe(&self, pat: &str) -> mpsc::Receiver<BlackboardEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subs.write().await.push(Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            pattern: pat.to_string(),
            tx,
        });
        rx
    }

    /// Create snapshot.
    pub async fn snapshot(&self) -> HashMap<String, BlackboardEntry> {
        self.state.read().await.clone()
    }

    /// Remove entry.
    pub async fn remove(&self, key: &str) {
        self.state.write().await.remove(key);
        self.notify(key, BlackboardEvent::EntryRemoved(key.to_string()))
            .await;
    }

    /// Get keys matching pattern.
    pub async fn keys(&self, pat: &str) -> Vec<String> {
        let s = self.state.read().await;
        if pat == "*" {
            s.keys().cloned().collect()
        } else {
            s.keys().filter(|k| k.contains(pat)).cloned().collect()
        }
    }

    fn resolve(&self, e: &BlackboardEntry, n: &BlackboardEntry) -> BlackboardEntry {
        match self.strategy {
            ConflictStrategy::LastWriteWins => {
                if n.timestamp > e.timestamp
                    || (n.timestamp == e.timestamp && n.version > e.version)
                {
                    n.clone()
                } else {
                    e.clone()
                }
            }
            ConflictStrategy::FirstWriteWins => e.clone(),
            ConflictStrategy::Merge => {
                warn!("Merge not implemented, using last-write");
                n.clone()
            }
        }
    }

    async fn notify(&self, key: &str, evt: BlackboardEvent) {
        for s in self.subs.read().await.iter() {
            if key.contains(&s.pattern) || s.pattern == "*" {
                let _ = s.tx.send(evt.clone()).await;
            }
        }
    }

    /// Unsubscribe by id.
    pub async fn unsubscribe(&self, id: &str) {
        self.subs.write().await.retain(|s| s.id != id);
    }
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_write() {
        let bb = Blackboard::new();
        bb.write("k1", "v1", "a1").await;
        assert_eq!(bb.read("k1").await.unwrap().value, "v1");
    }

    #[tokio::test]
    async fn test_conflict() {
        let bb = Blackboard::with_strategy(ConflictStrategy::LastWriteWins);
        bb.write("k", "first", "a1").await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        bb.write("k", "second", "a2").await;
        assert_eq!(bb.read("k").await.unwrap().value, "second");
    }

    #[tokio::test]
    async fn test_subscribe() {
        let bb = Blackboard::new();
        let mut rx = bb.subscribe("k").await;
        bb.write("k1", "v1", "a1").await;
        if let Ok(BlackboardEvent::EntryAdded(k, _)) = rx.try_recv() {
            assert!(k.contains("k"));
        }
    }

    #[tokio::test]
    async fn test_snapshot() {
        let bb = Blackboard::new();
        bb.write("k1", "v1", "a1").await;
        bb.write("k2", "v2", "a2").await;
        assert_eq!(bb.snapshot().await.len(), 2);
    }

    #[tokio::test]
    async fn test_keys() {
        let bb = Blackboard::new();
        bb.write("p_k1", "v1", "a1").await;
        bb.write("p_k2", "v2", "a2").await;
        bb.write("o", "v3", "a3").await;
        assert_eq!(bb.keys("p").await.len(), 2);
    }
}
