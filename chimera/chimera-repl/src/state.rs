//! REPL state machine - Pure data structures, zero TUI dependencies
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::clock::Clock;

/// Core conversation state.
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
        Self { turn_items: Vec::new(), current_turn_id: None, is_loading: false, session_meta: SessionMeta::default(), _clock: PhantomData }
    }
}

impl<C: Clock> ReplState<C> {
    pub fn add_turn(&mut self, clock: &C, role: Role, content: String) {
        let ts = clock.now_ms();
        self.turn_items.push(TurnItem::new(format!("t{}", self.session_meta.turn_count), role, content, ts));
        self.session_meta.turn_count += 1;
        self.session_meta.updated_at = ts;
    }

    pub fn process_user(&mut self, clock: &C, content: String) -> &TurnItem {
        self.add_turn(clock, Role::User, content);
        self.turn_items.last().unwrap()
    }

    pub fn process_turn(&mut self, clock: &C, content: String) -> Result<&TurnItem, ()> {
        if content.is_empty() { return Err(()); }
        self.add_turn(clock, Role::Turn, content);
        Ok(self.turn_items.last().unwrap())
    }

    pub fn handle_error(&mut self, clock: &C, code: u32, msg: String) -> &TurnItem {
        let mut item = TurnItem::new(format!("e{}", code), Role::Error, msg, clock.now_ms());
        item.error_code = Some(code);
        self.turn_items.push(item);
        self.session_meta.turn_count += 1;
        self.turn_items.last().unwrap()
    }
}

/// Single conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnItem {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
    pub metadata: Option<serde_json::Value>,
    pub processed: bool,
    pub error_code: Option<u32>,
}

impl TurnItem {
    pub fn new(id: String, role: Role, content: String, timestamp: u64) -> Self {
        Self { id, role, content, timestamp, metadata: None, processed: false, error_code: None }
    }
    pub fn validate(&self) -> bool { !self.content.is_empty() && self.content.len() <= 100_000 }
    pub fn is_user_input(&self) -> bool { matches!(self.role, Role::User) }
    pub fn is_ai_response(&self) -> bool { matches!(self.role, Role::Turn) }
    pub fn is_error(&self) -> bool { matches!(self.role, Role::Error) }
}

/// Message role with full variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Role {
    #[default]
    User,
    Turn,
    Error,
}

/// Session metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMeta {
    pub created_at: u64,
    pub updated_at: u64,
    pub turn_count: usize,
}
