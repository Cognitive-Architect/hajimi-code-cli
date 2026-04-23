//! Event system for REPL communication.
//!
//! Replaces TUI event handling with async channel-based messaging.
//! Day 2 Extension: Added autonomous agent loop events (AgentTick, ObservationReceived, etc.)

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

/// Events emitted by the REPL engine and agent system.
/// Supports serde for persistence and cross-process communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ReplEvent {
    /// User input received.
    Input(String),
    /// Command execution request.
    Command { name: String, args: Vec<String> },
    /// Protocol event placeholder (Codex protocol integration).
    ProtocolEvent(String),
    /// Session state change.
    SessionUpdate(crate::SessionState),
    /// Shutdown request.
    Shutdown,

    // === Day 2: Autonomous Agent Loop Events ===
    /// Agent execution tick - signals one reasoning cycle.
    /// Contains agent ID and current cycle count.
    AgentTick { agent_id: String, cycle: u64 },

    /// Observation received from environment or tool execution.
    /// Carries structured observation data for agent processing.
    ObservationReceived { agent_id: String, observation: String, source: String },

    /// Tool execution result available.
    /// Contains tool name, result payload, and success status.
    ToolResult { agent_id: String, tool_name: String, result: String, success: bool },

    /// Reflection cycle completed.
    /// Signals agent has finished self-critique and learning.
    ReflectionComplete { agent_id: String, insights: Vec<String>, confidence: f32 },

    /// Plan update event - plan created or modified.
    /// Contains serialized plan structure and version.
    PlanUpdate { agent_id: String, plan_version: u32, description: String, subtasks: Vec<String> },
}

impl ReplEvent {
    /// Returns true if this event should be persisted to memory.
    pub fn should_persist(&self) -> bool {
        matches!(
            self,
            ReplEvent::AgentTick { .. }
                | ReplEvent::ObservationReceived { .. }
                | ReplEvent::ToolResult { .. }
                | ReplEvent::ReflectionComplete { .. }
                | ReplEvent::PlanUpdate { .. }
                | ReplEvent::ProtocolEvent(_)
        )
    }

    /// Returns the agent ID associated with this event, if any.
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            ReplEvent::AgentTick { agent_id, .. } => Some(agent_id),
            ReplEvent::ObservationReceived { agent_id, .. } => Some(agent_id),
            ReplEvent::ToolResult { agent_id, .. } => Some(agent_id),
            ReplEvent::ReflectionComplete { agent_id, .. } => Some(agent_id),
            ReplEvent::PlanUpdate { agent_id, .. } => Some(agent_id),
            _ => None,
        }
    }
}

/// Event sender wrapper for REPL communication.
#[derive(Debug, Clone)]
pub struct ReplEventSender {
    inner: Sender<ReplEvent>,
}

impl ReplEventSender {
    /// Create new event sender.
    pub fn new(sender: Sender<ReplEvent>) -> Self {
        Self { inner: sender }
    }

    /// Send event to engine (non-blocking).
    pub async fn send(&self, event: ReplEvent) -> Result<(), crate::ReplError> {
        self.inner
            .send(event)
            .await
            .map_err(|_| crate::ReplError::Channel("Event channel closed".to_string()))
    }
}

/// Event handler trait for custom processing.
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Process a REPL event.
    async fn handle(&self, event: ReplEvent) -> Result<(), crate::ReplError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_events_serialize() {
        let event = ReplEvent::AgentTick {
            agent_id: "agent-1".to_string(),
            cycle: 42,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("AgentTick"));
        assert!(json.contains("agent-1"));
    }

    #[test]
    fn test_event_should_persist() {
        assert!(ReplEvent::AgentTick { agent_id: "a".to_string(), cycle: 1 }.should_persist());
        assert!(!ReplEvent::Input("test".to_string()).should_persist());
    }
}
