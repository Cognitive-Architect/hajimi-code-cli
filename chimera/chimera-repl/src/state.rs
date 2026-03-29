//! REPL state machine - Pure data structures, zero TUI dependencies
use serde::{Deserialize, Serialize};

/// Core conversation state (replaces Codex ThreadEventStore).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplState {
    /// Historical turn items.
    pub turn_items: Vec<TurnItem>,
    /// Current active turn ID.
    pub current_turn_id: Option<String>,
    /// Loading indicator (pure data).
    pub is_loading: bool,
    /// Session metadata.
    pub session_meta: SessionMeta,
}

/// Single conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnItem {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
}

/// Message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Role {
    #[default]
    User,
    Assistant,
    System,
}

/// Session metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMeta {
    pub created_at: u64,
    pub updated_at: u64,
    pub turn_count: usize,
}

impl ReplState {
    /// Append a new turn.
    pub fn add_turn(&mut self, role: Role, content: String) {
        let timestamp = now_ms();
        let id = format!("turn_{}", self.session_meta.turn_count);
        self.turn_items.push(TurnItem { id, role, content, timestamp });
        self.session_meta.turn_count += 1;
        self.session_meta.updated_at = timestamp;
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}
