//! Session state management for REPL.
//!
//! Maintains conversation context and execution state
//! independently of TUI presentation layer.

use std::collections::VecDeque;

use codex_twist::thread::ThreadId;
use serde::{Deserialize, Serialize};

/// Conversation turn record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    /// Turn sequence number.
    pub sequence: u64,
    /// User input for this turn.
    pub input: String,
    /// Timestamp in milliseconds.
    pub timestamp: u64,
}

/// REPL session state (TUI-free).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Associated thread ID.
    pub thread_id: Option<ThreadId>,
    /// Conversation history (limited buffer).
    pub history: VecDeque<TurnRecord>,
    /// Current working directory.
    pub cwd: String,
    /// Environment variables snapshot.
    pub env_vars: std::collections::HashMap<String, String>,
    /// Maximum history buffer size.
    pub max_history: usize,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            thread_id: None,
            history: VecDeque::with_capacity(1000),
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            env_vars: std::env::vars().collect(),
            max_history: 1000,
        }
    }
}

impl SessionState {
    /// Add a new turn to history.
    pub fn record_turn(&mut self, input: String) {
        let record = TurnRecord {
            sequence: self.history.len() as u64 + 1,
            input,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(record);
    }

    /// Clear conversation history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}
