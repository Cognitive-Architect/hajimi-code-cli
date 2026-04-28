//! Codex-Twist MemoryGateway bridge for Chimera REPL
use std::collections::HashMap;

use crate::clock::Clock;
use crate::state::{ReplState, Role, TurnItem};
use crate::ReplError;

pub use codex_twist::memory::MemoryGateway;
pub use codex_twist::thread::{Thread, ThreadConfig, ThreadId};
pub use codex_twist::turn::{Turn, TurnStatus};

/// Bridge between ReplState and Codex MemoryGateway
pub struct CodexBridge<C: Clock> {
    gateway: MemoryGateway,
    thread: Option<Thread>,
    state: ReplState<C>,
}

/// Turn with metadata for serialization
#[derive(Clone, Debug, Default)]
pub struct TurnWithMeta {
    pub turn: Turn,
    pub metadata: HashMap<String, String>,
}

impl<C: Clock> CodexBridge<C> {
    pub fn new(state: ReplState<C>) -> Result<Self, ReplError> {
        Ok(Self { gateway: MemoryGateway::new(), thread: None, state })
    }

    fn role_to_codex(role: Role) -> &'static str {
        match role { Role::User => "user", Role::Turn => "assistant", Role::Error => "system" }
    }

    fn extract_metadata(item: &TurnItem) -> HashMap<String, String> {
        item.metadata.as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn map_turn(&self, item: &TurnItem) -> Result<TurnWithMeta, ReplError> {
        if !item.validate() { return Err(ReplError::Session("Invalid turn".to_string())); }
        let turn = Turn {
            id: item.id.clone(),
            role: Self::role_to_codex(item.role).to_string(),
            content: item.content.clone(),
            timestamp_ms: item.timestamp,
            status: if item.processed { TurnStatus::Complete } else { TurnStatus::Pending },
        };
        let metadata = Self::extract_metadata(item);
        Ok(TurnWithMeta { turn, metadata })
    }

    pub async fn sync_turn(&self, idx: usize) -> Result<(), ReplError> {
        let item = self.state.turn_items.get(idx)
            .ok_or_else(|| ReplError::Session("Invalid index".to_string()))?;
        let turn_meta = self.map_turn(item)?;
        let key = format!("turn:{}", turn_meta.turn.id);
        let value = serde_json::to_string(&turn_meta).map_err(ReplError::Protocol)?;
        self.gateway.put(key, value, codex_twist::memory::MemoryLevel::Working).await;
        Ok(())
    }

    pub fn get_metadata(&self, item: &TurnItem) -> HashMap<String, String> {
        Self::extract_metadata(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::TestClock;
    use serde_json::json;

    #[test]
    fn test_role_mapping() {
        assert_eq!(CodexBridge::<TestClock>::role_to_codex(Role::User), "user");
        assert_eq!(CodexBridge::<TestClock>::role_to_codex(Role::Turn), "assistant");
    }

    #[test]
    fn test_turn_mapping() {
        let state: ReplState<TestClock> = ReplState::default();
        let bridge = CodexBridge::new(state).unwrap();
        let item = TurnItem::new("t1".to_string(), Role::User, "hello".to_string(), 1000);
        let turn_meta = bridge.map_turn(&item).unwrap();
        assert_eq!(turn_meta.turn.id, "t1");
        assert_eq!(turn_meta.turn.role, "user");
    }

    #[test]
    fn test_metadata_extraction() {
        let state: ReplState<TestClock> = ReplState::default();
        let bridge = CodexBridge::new(state).unwrap();
        let mut item = TurnItem::new("t2".to_string(), Role::User, "test".to_string(), 1000);
        let mut meta = HashMap::new();
        meta.insert("key1".to_string(), "value1".to_string());
        item.metadata = Some(json!(meta));
        
        let turn_meta = bridge.map_turn(&item).unwrap();
        assert_eq!(turn_meta.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_empty_metadata() {
        let state: ReplState<TestClock> = ReplState::default();
        let bridge = CodexBridge::new(state).unwrap();
        let item = TurnItem::new("t3".to_string(), Role::User, "test".to_string(), 1000);
        let turn_meta = bridge.map_turn(&item).unwrap();
        assert!(turn_meta.metadata.is_empty());
    }
}
