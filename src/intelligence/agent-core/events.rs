//! Agent Core Event Handling
//!
//! Bridges ReplEvent system with AgentOrchestrator for autonomous loop integration.

use crate::AgentContext;
use chimera_repl::traits::ReplResult;
use chimera_repl::event::{ReplEvent, ReplEventSender};
use memory::memory_gateway::MemoryGateway;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Event processor for agent-specific events.
/// Integrates with MemoryGateway for automatic persistence (Phase 5).
pub struct AgentEventProcessor {
    event_sender: ReplEventSender,
    // DEBT-MEMORY-SYNC(Phase 5): 事件持久化待SyncMemoryGateway集成
    #[allow(dead_code)]
    memory: Arc<Mutex<MemoryGateway>>,

    #[allow(dead_code)]
    context: AgentContext,
}

impl AgentEventProcessor {
    /// Create new event processor with shared resources.
    pub fn new(
        event_sender: ReplEventSender,
        memory: Arc<Mutex<MemoryGateway>>,
        context: AgentContext,
    ) -> Self {
        Self {
            event_sender,
            memory,
            context,
        }
    }

    /// Process agent tick event and broadcast.
    pub async fn process_tick(&self, agent_id: &str, cycle: u64) -> ReplResult<()> {
        let event = ReplEvent::AgentTick {
            agent_id: agent_id.to_string(),
            cycle,
        };
        self.event_sender.send(event).await
    }

    /// Process observation and broadcast event.
    pub async fn process_observation(
        &self,
        agent_id: &str,
        observation: &str,
        source: &str,
    ) -> ReplResult<()> {
        let event = ReplEvent::ObservationReceived {
            agent_id: agent_id.to_string(),
            observation: observation.to_string(),
            source: source.to_string(),
        };
        self.event_sender.send(event).await
    }

    /// Process tool result and broadcast.
    pub async fn process_tool_result(
        &self,
        agent_id: &str,
        tool_name: &str,
        result: &str,
        success: bool,
    ) -> ReplResult<()> {
        let event = ReplEvent::ToolResult {
            agent_id: agent_id.to_string(),
            tool_name: tool_name.to_string(),
            result: result.to_string(),
            success,
        };
        self.event_sender.send(event).await
    }

    /// Process reflection completion and broadcast.
    pub async fn process_reflection(
        &self,
        agent_id: &str,
        insights: Vec<String>,
        confidence: f32,
    ) -> ReplResult<()> {
        let event = ReplEvent::ReflectionComplete {
            agent_id: agent_id.to_string(),
            insights,
            confidence,
        };
        self.event_sender.send(event).await
    }

    /// Process plan update and broadcast.
    pub async fn process_plan_update(
        &self,
        agent_id: &str,
        plan_version: u32,
        description: &str,
        subtasks: Vec<String>,
    ) -> ReplResult<()> {
        let event = ReplEvent::PlanUpdate {
            agent_id: agent_id.to_string(),
            plan_version,
            description: description.to_string(),
            subtasks,
        };
        self.event_sender.send(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_event_processor_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let sender = ReplEventSender::new(tx);
        let memory = Arc::new(Mutex::new(MemoryGateway::new("test")));
        let context = AgentContext::new();

        let processor = AgentEventProcessor::new(sender, memory, context);
        assert_eq!(processor.context.cycle_count, 0);
    }
}
