//! Codex-Twist MemoryGateway bridge for Chimera REPL
use std::path::PathBuf;

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

impl<C: Clock> CodexBridge<C> {
    pub fn new(state: ReplState<C>) -> Result<Self, ReplError> {
        Ok(Self { gateway: MemoryGateway::new(), thread: None, state })
    }

    pub fn init_thread(&mut self, path: PathBuf) -> Result<&Thread, ReplError> {
        let thread = codex_twist::create_thread(path)
            .map_err(|e| ReplError::Session(format!("Thread init: {}", e)))?;
        self.thread = Some(thread);
        Ok(self.thread.as_ref().unwrap())
    }

    fn role_to_codex(role: Role) -> &'static str {
        match role { Role::User => "user", Role::Turn => "assistant", Role::Error => "system" }
    }

    pub fn map_turn(&self, item: &TurnItem) -> Result<Turn, ReplError> {
        if !item.validate() { return Err(ReplError::Session("Invalid turn".to_string())); }
        Ok(Turn {
            id: item.id.clone(),
            role: Self::role_to_codex(item.role).to_string(),
            content: item.content.clone(),
            timestamp_ms: item.timestamp,
            status: if item.processed { TurnStatus::Complete } else { TurnStatus::Pending },
        })
    }

    pub async fn sync_turn(&self, idx: usize) -> Result<(), ReplError> {
        let item = self.state.turn_items.get(idx)
            .ok_or_else(|| ReplError::Session("Invalid index".to_string()))?;
        let turn = self.map_turn(item)?;
        let (key, value) = (format!("turn:{}", turn.id), serde_json::to_string(&turn).map_err(|e| ReplError::Protocol(e))?);
        self.gateway.put(key, value, codex_twist::memory::MemoryLevel::Working).await;
        Ok(())
    }

    pub async fn get_turn(&self, turn_id: &str) -> Option<Turn> {
        self.gateway.get(&format!("turn:{}", turn_id)).await
            .and_then(|v| serde_json::from_str(&v).ok())
    }

    pub async fn memory_stats(&self) -> GatewayStats {
        let s = self.gateway.stats().await;
        GatewayStats { focus_entries: s.focus_entries, focus_tokens: s.focus_tokens, working_entries: s.working_entries, working_tokens: s.working_tokens, archive_entries: s.archive_entries, archive_tokens: s.archive_tokens }
    }

    pub async fn clear_memory(&self) {
        self.gateway.clear_focus().await;
        self.gateway.clear_working().await;
        self.gateway.clear_archive().await;
    }
}

#[derive(Debug, Clone, Default)]
pub struct GatewayStats {
    pub focus_entries: usize, pub focus_tokens: usize, pub working_entries: usize,
    pub working_tokens: usize, pub archive_entries: usize, pub archive_tokens: usize,
}

pub struct BridgeFactory;
impl BridgeFactory {
    pub fn default_bridge<C: Clock>(state: ReplState<C>) -> Result<CodexBridge<C>, ReplError> {
        CodexBridge::new(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::MockClock;

    #[test]
    fn test_role_mapping() {
        assert_eq!(CodexBridge::<MockClock>::role_to_codex(Role::User), "user");
        assert_eq!(CodexBridge::<MockClock>::role_to_codex(Role::Turn), "assistant");
    }

    #[test]
    fn test_turn_mapping() {
        let state: ReplState<MockClock> = ReplState::default();
        let bridge = CodexBridge::new(state).unwrap();
        let item = TurnItem::new("t1".to_string(), Role::User, "hello".to_string(), 1000);
        let turn = bridge.map_turn(&item).unwrap();
        assert_eq!(turn.id, "t1");
        assert_eq!(turn.role, "user");
    }
}
