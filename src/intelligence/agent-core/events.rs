//! Agent Core Event Handling
//!
//! Bridges ReplEvent system with AgentOrchestrator for autonomous loop integration.
//! Phase 2 Day 1: Extended with Swarm worker result events.

use crate::event_tracing::EventTracing;
use crate::ports::{WorkerResultStatus, WorkerMetrics};
use crate::swarm::WorkerResult;
use crate::AgentContext;
use chimera_repl::traits::ReplResult;
use chimera_repl::event::{ReplEvent, ReplEventSender};
use memory::memory_gateway::MemoryGateway;
use memory::sync_gateway::GatewayEvent;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Structured event representing a worker execution result.
/// Decoupled from ReplEvent for internal processing before broadcast.
#[derive(Debug, Clone)]
pub struct WorkerResultEvent {
    pub task_id: String,
    pub worker_id: String,
    pub status: WorkerResultStatus,
    pub output: String,
    pub error: Option<String>,
    pub metrics: Option<WorkerMetrics>,
    pub timestamp: std::time::Instant,
}

impl WorkerResultEvent {
    /// Build a WorkerResultEvent from a WorkerResult.
    pub fn from_worker_result(result: &WorkerResult) -> Self {
        Self {
            task_id: result.task_id.clone(),
            worker_id: result.worker_id.clone(),
            status: result.status,
            output: result.output.clone(),
            error: result.error.clone(),
            metrics: result.metrics.clone(),
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Event processor for agent-specific events.
/// Integrates with MemoryGateway for automatic persistence (Phase 5).
pub struct AgentEventProcessor {
    event_sender: ReplEventSender,
    #[allow(dead_code)] // retained for API compatibility; sync_gateway now handles persistence
    memory: Arc<Mutex<MemoryGateway>>,
    sync_gateway: Option<memory::sync_gateway::SyncGatewayHandle>,
    #[allow(dead_code)]
    context: AgentContext,
    /// Accumulated operation stats across tool executions (B-06/12).
    operation_summary: std::sync::Mutex<crate::OperationSummary>,
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
            sync_gateway: None,
            context,
            operation_summary: std::sync::Mutex::new(crate::OperationSummary::default()),
        }
    }
    pub fn with_sync_gateway(mut self, sg: Option<memory::sync_gateway::SyncGatewayHandle>) -> Self {
        self.sync_gateway = sg; self
    }

    /// Helper: emit a GatewayEvent via sync_gateway if configured.
    async fn emit_gateway(&self, event_type: &str, detail: String, agent_id: &str) {
        if let Some(ref sg) = self.sync_gateway {
            let ge = GatewayEvent::new(event_type, detail, agent_id);
            let _ = sg.lock().await.push_event(ge).await;
        }
    }

    /// Process agent tick event and broadcast.
    pub async fn process_tick(&self, agent_id: &str, cycle: u64) -> ReplResult<()> {
        let event = ReplEvent::AgentTick { agent_id: agent_id.to_string(), cycle };
        self.event_sender.send(event).await?;
        self.emit_gateway("AgentTick", format!("cycle={}", cycle), agent_id).await;
        Ok(())
    }

    /// Process observation and broadcast event.
    pub async fn process_observation(&self, agent_id: &str, observation: &str, source: &str) -> ReplResult<()> {
        let event = ReplEvent::ObservationReceived { agent_id: agent_id.to_string(), observation: observation.to_string(), source: source.to_string() };
        self.event_sender.send(event).await?;
        self.emit_gateway("Observation", format!("{}: {}", source, observation), agent_id).await;
        Ok(())
    }

    /// Process tool result and broadcast.
    /// Accumulates operation stats (files_edited, commands_run) based on tool name (B-06/12).
    pub async fn process_tool_result(&self, agent_id: &str, tool_name: &str, result: &str, success: bool) -> ReplResult<()> {
        // Accumulate operation stats by tool name.
        {
            let mut summary = self.operation_summary.lock().unwrap();
            let lower = tool_name.to_lowercase();
            if lower.contains("edit") || lower.contains("write") || lower.contains("file") {
                summary.files_edited += 1;
                summary.total_diff_lines = summary.total_diff_lines.saturating_add(result.lines().count());
            }
            if lower.contains("create") || lower.contains("new") {
                summary.files_created += 1;
            }
            if lower.contains("delete") || lower.contains("remove") {
                summary.files_deleted += 1;
            }
            if lower.contains("bash") || lower.contains("shell") || lower.contains("powershell") || lower.contains("cargo") || lower.contains("npm") || lower.contains("git") {
                summary.commands_run += 1;
            }
        }

        let event = ReplEvent::ToolResult { agent_id: agent_id.to_string(), tool_name: tool_name.to_string(), result: result.to_string(), success };
        self.event_sender.send(event).await?;

        // Broadcast accumulated OperationSummary after each tool execution.
        let summary = self.operation_summary.lock().unwrap().clone();
        self.process_operation_summary(agent_id, summary).await?;

        self.emit_gateway("ToolResult", format!("{}: success={}", tool_name, success), agent_id).await;
        Ok(())
    }

    /// Process reflection completion and broadcast.
    pub async fn process_reflection(&self, agent_id: &str, insights: Vec<String>, confidence: f32) -> ReplResult<()> {
        let event = ReplEvent::ReflectionComplete { agent_id: agent_id.to_string(), insights: insights.clone(), confidence };
        self.event_sender.send(event).await?;
        self.emit_gateway("Reflection", format!("confidence={}: {}", confidence, insights.join("; ")), agent_id).await;
        Ok(())
    }

    /// Process plan update and broadcast.
    pub async fn process_plan_update(&self, agent_id: &str, plan_version: u32, description: &str, subtasks: Vec<String>) -> ReplResult<()> {
        let event = ReplEvent::PlanUpdate { agent_id: agent_id.to_string(), plan_version, description: description.to_string(), subtasks: subtasks.clone() };
        self.event_sender.send(event).await?;
        self.emit_gateway("PlanUpdate", format!("v{}: {} subtasks={:?}", plan_version, description, subtasks), agent_id).await;
        Ok(())
    }

    /// Process a worker result event and broadcast SwarmTaskCompleted.
    /// Gracefully degrades if sync_gateway is not initialized.
    pub async fn process_worker_result(&self, agent_id: &str, result: &WorkerResult) -> ReplResult<()> {
        let event = ReplEvent::SwarmTaskCompleted {
            agent_id: agent_id.to_string(),
            task_id: result.task_id.clone(),
            worker_id: result.worker_id.clone(),
            success: result.success,
            output: result.output.clone(),
        };
        self.event_sender.send(event).await?;

        self.emit_gateway("SwarmTaskCompleted", format!("task={} worker={} status={:?} success={}", result.task_id, result.worker_id, result.status, result.success), agent_id).await;

        // Emit structured trace for observability.
        info!(
            task_id = %result.task_id,
            worker_id = %result.worker_id,
            success = result.success,
            "Worker result processed"
        );

        Ok(())
    }

    /// Process operation summary and broadcast.
    pub async fn process_operation_summary(&self, agent_id: &str, summary: crate::OperationSummary) -> ReplResult<()> {
        let event = ReplEvent::OperationSummary {
            agent_id: agent_id.to_string(),
            summary: chimera_repl::OperationSummary {
                files_edited: summary.files_edited,
                files_created: summary.files_created,
                files_deleted: summary.files_deleted,
                commands_run: summary.commands_run,
                total_diff_lines: summary.total_diff_lines,
            },
        };
        self.event_sender.send(event).await?;
        self.emit_gateway("OperationSummary", format!("edited={} created={} deleted={} commands={} diff_lines={}", summary.files_edited, summary.files_created, summary.files_deleted, summary.commands_run, summary.total_diff_lines), agent_id).await;
        Ok(())
    }

    /// Process thinking content and broadcast.
    pub async fn process_thinking_content(&self, agent_id: &str, content: &str) -> ReplResult<()> {
        let event = ReplEvent::ThinkingContent { agent_id: agent_id.to_string(), content: content.to_string() };
        self.event_sender.send(event).await?;
        self.emit_gateway("ThinkingContent", format!("content_len={}", content.len()), agent_id).await;
        Ok(())
    }

    /// Emit a trace event when a worker is spawned.
    pub async fn trace_worker_spawn(&self, agent_id: &str, worker_id: &str) {
        EventTracing::trace_worker_spawn(agent_id, worker_id).await;
    }

    /// Emit a trace event when a worker starts processing a task.
    pub async fn trace_worker_start(&self, agent_id: &str, task_id: &str, worker_id: &str) {
        EventTracing::trace_worker_start(agent_id, task_id, worker_id).await;
    }

    /// Emit a trace event when a worker completes a task.
    pub async fn trace_worker_complete(&self, agent_id: &str, task_id: &str, worker_id: &str, success: bool, duration_ms: u64) {
        EventTracing::trace_worker_complete(agent_id, task_id, worker_id, success, duration_ms).await;
    }

    /// Emit a trace event when a worker fails a task.
    pub async fn trace_worker_fail(&self, agent_id: &str, task_id: &str, worker_id: &str, error: &str) {
        EventTracing::trace_worker_fail(agent_id, task_id, worker_id, error).await;
    }

    /// Emit a trace event when a worker crashes.
    pub async fn trace_worker_crash(&self, agent_id: &str, worker_id: &str, error: &str) {
        EventTracing::trace_worker_crash(agent_id, worker_id, error).await;
    }

    /// Emit a trace event when a worker is restarted after crash.
    pub async fn trace_worker_restart(&self, agent_id: &str, old_worker_id: &str, new_worker_id: &str, attempt: u8) {
        EventTracing::trace_worker_restart(agent_id, old_worker_id, new_worker_id, attempt).await;
    }

    /// Emit a trace event for tool execution completion within a worker.
    pub async fn process_tool_execution_completed(
        &self,
        agent_id: &str,
        tool_name: &str,
        task_id: &str,
        result: &str,
        success: bool,
    ) -> ReplResult<()> {
        let event = ReplEvent::ToolExecutionCompleted {
            agent_id: agent_id.to_string(),
            tool_name: tool_name.to_string(),
            task_id: task_id.to_string(),
            result: result.to_string(),
            success,
        };
        self.event_sender.send(event).await?;

        self.emit_gateway("ToolExecutionCompleted", format!("{}: task={} success={}", tool_name, task_id, success), agent_id).await;

        info!(
            agent_id = %agent_id,
            tool_name = %tool_name,
            task_id = %task_id,
            success = success,
            "Tool execution completed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests;
