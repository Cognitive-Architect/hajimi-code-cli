//! DEBT-LINES-B04B: Extracted worker lifecycle trace methods from events.rs
//! Provides structured trace logging for Swarm worker lifecycle events.

use tracing::{error, info, warn};

/// Structured trace emitter for worker lifecycle events.
pub struct EventTracing;

impl EventTracing {
    /// Emit a trace event when a worker is spawned.
    pub async fn trace_worker_spawn(agent_id: &str, worker_id: &str) {
        info!(agent_id = %agent_id, worker_id = %worker_id, "Worker spawned");
    }

    /// Emit a trace event when a worker starts processing a task.
    pub async fn trace_worker_start(agent_id: &str, task_id: &str, worker_id: &str) {
        info!(agent_id = %agent_id, task_id = %task_id, worker_id = %worker_id, "Worker started task");
    }

    /// Emit a trace event when a worker completes a task.
    pub async fn trace_worker_complete(
        agent_id: &str,
        task_id: &str,
        worker_id: &str,
        success: bool,
        duration_ms: u64,
    ) {
        info!(agent_id = %agent_id, task_id = %task_id, worker_id = %worker_id, success = success, duration_ms = duration_ms, "Worker completed task");
    }

    /// Emit a trace event when a worker fails a task.
    pub async fn trace_worker_fail(agent_id: &str, task_id: &str, worker_id: &str, error: &str) {
        error!(agent_id = %agent_id, task_id = %task_id, worker_id = %worker_id, error = %error, "Worker task failed");
    }

    /// Emit a trace event when a worker crashes.
    pub async fn trace_worker_crash(agent_id: &str, worker_id: &str, error: &str) {
        error!(agent_id = %agent_id, worker_id = %worker_id, error = %error, "Worker crashed");
    }

    /// Emit a trace event when a worker is restarted after crash.
    pub async fn trace_worker_restart(
        agent_id: &str,
        old_worker_id: &str,
        new_worker_id: &str,
        attempt: u8,
    ) {
        warn!(agent_id = %agent_id, old_worker_id = %old_worker_id, new_worker_id = %new_worker_id, attempt = attempt, "Worker restarted after crash");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trace_worker_spawn() {
        EventTracing::trace_worker_spawn("agent-1", "worker-1").await;
    }

    #[tokio::test]
    async fn test_trace_worker_complete() {
        EventTracing::trace_worker_complete("agent-1", "task-1", "worker-1", true, 42).await;
    }
}
