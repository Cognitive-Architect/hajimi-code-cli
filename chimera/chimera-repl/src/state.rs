//! REPL state machine - Pure data structures, zero TUI dependencies
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::clock::Clock;

/// Core conversation state with clock abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplState<C: Clock> {
    pub turn_items: Vec<TurnItem>,
    pub current_turn_id: Option<String>,
    pub is_loading: bool,
    pub session_meta: SessionMeta,
    #[serde(skip)]
    _clock: PhantomData<C>,
}

impl<C: Clock> Default for ReplState<C> {
    fn default() -> Self {
        Self {
            turn_items: Vec::new(),
            current_turn_id: None,
            is_loading: false,
            session_meta: SessionMeta::default(),
            _clock: PhantomData,
        }
    }
}

impl<C: Clock> ReplState<C> {
    /// Append a new turn using injected clock.
    pub fn add_turn(&mut self, clock: &C, role: Role, content: String) {
        let timestamp = clock.now_ms();
        let id = format!("turn_{}", self.session_meta.turn_count);
        self.turn_items.push(TurnItem { id, role, content, timestamp });
        self.session_meta.turn_count += 1;
        self.session_meta.updated_at = timestamp;
    }
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
