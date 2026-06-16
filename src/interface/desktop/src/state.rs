use agent_core::TraceEvent;
use codex_twist::memory::{MemoryGateway, TokenUsageTracker};
use engine_tool_system::ToolRegistry;
use std::collections::HashMap;
use std::sync::Arc;

/// Phase 4 Day 5: Edit history entry for timeline visualization.
#[derive(Clone, serde::Serialize)]
pub struct EditHistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub step_type: String,
    pub summary: String,
    pub confidence: Option<f32>,
    pub token_before: Option<usize>,
    pub token_after: Option<usize>,
    pub checkpoint_id: Option<String>,
}

/// Day 08 checkpoint file reference for export/compare contracts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointFileRef {
    pub path: String,
    pub status: String,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub content: Option<String>,
    pub after_content: Option<String>,
}

/// Day 08 checkpoint diff summary. Detailed hunks are deferred to Day 09.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointDiffSummary {
    pub files_changed: usize,
    pub hunks: Option<usize>,
    pub additions: Option<usize>,
    pub deletions: Option<usize>,
    pub summary: String,
}

/// Day 08 checkpoint metadata for trace linkage and schema evolution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointMetadata {
    pub source: String,
    pub agent_id: Option<String>,
    pub iteration: usize,
    pub step_type: String,
    pub confidence: Option<f32>,
    pub schema_version: u32,
}

/// Minimal desktop-local checkpoint DTO for Day 09 export/compare.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointRecord {
    pub id: String,
    pub timestamp: String,
    pub label: String,
    pub files: Vec<CheckpointFileRef>,
    pub diff_summary: CheckpointDiffSummary,
    pub trace_event_ids: Vec<String>,
    pub metadata: CheckpointMetadata,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheckpointExportBundle {
    pub schema_version: u32,
    pub exported_at: String,
    pub workspace: String,
    pub checkpoints: Vec<CheckpointRecord>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CheckpointFileChange {
    pub path: String,
    pub before_status: Option<String>,
    pub after_status: Option<String>,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CheckpointCompareResult {
    pub id_a: String,
    pub id_b: String,
    pub same: bool,
    pub files_added: Vec<CheckpointFileChange>,
    pub files_removed: Vec<CheckpointFileChange>,
    pub files_modified: Vec<CheckpointFileChange>,
    pub summary: String,
    pub data_source: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RestoreFilePlan {
    pub path: String,
    pub action: String,
    pub target_exists: bool,
    pub backup_path: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RestoreResult {
    pub checkpoint_id: String,
    pub restored_at: String,
    pub dry_run: bool,
    pub backup_dir: String,
    pub files: Vec<RestoreFilePlan>,
    pub warnings: Vec<String>,
}

pub struct AppState {
    pub registry: Arc<tokio::sync::Mutex<ToolRegistry>>,
    pub active_profile: std::sync::Mutex<Option<String>>,
    pub agent_providers: std::sync::Mutex<HashMap<String, String>>,
    pub trace_tx: std::sync::Mutex<Option<tokio::sync::broadcast::Sender<TraceEvent>>>,
    pub paused: std::sync::Mutex<bool>,
    pub approval_level: std::sync::Mutex<String>,
    pub edit_history: Arc<tokio::sync::Mutex<Vec<EditHistoryEntry>>>,
    pub memory_gateway: Arc<MemoryGateway>,
    pub token_tracker: Arc<TokenUsageTracker>,
    pub pending_approvals:
        Arc<tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<bool>>>>,
    /// Shared LLM client slot for DesktopAgentTurnDriver.
    /// Updated by run_agent_task before each agent execution with the user's current provider.
    /// SAFETY: Arc<RwLock<>> ensures thread-safe concurrent access across Tauri commands.
    pub agent_llm_client: Arc<tokio::sync::RwLock<Option<Arc<dyn engine_llm_core::LlmClient>>>>,
}

pub type PendingApprovalMap =
    Arc<tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<bool>>>>;

pub async fn await_ui_approval_response(
    pending_approvals: PendingApprovalMap,
    request_id: String,
    action_type: String,
    rx: tokio::sync::oneshot::Receiver<bool>,
    timeout: std::time::Duration,
) -> agent_core::governance::Decision {
    let decision = match tokio::time::timeout(timeout, rx).await {
        Ok(Ok(true)) => agent_core::governance::Decision::Approved,
        Ok(Ok(false)) => {
            agent_core::governance::Decision::Rejected("User denied approval".to_string())
        }
        Ok(Err(_)) => {
            agent_core::governance::Decision::Rejected("Approval channel closed".to_string())
        }
        Err(_) => agent_core::governance::Decision::Rejected(format!(
            "Approval timed out while waiting for user response for tool '{}' after {}s",
            action_type,
            timeout.as_secs()
        )),
    };

    let mut map = pending_approvals.lock().await;
    map.remove(&request_id);
    decision
}

impl AppState {
    /// Inject the AgentLoop broadcast sender to enable trace event streaming.
    /// Call this after `AgentLoop::from_components()` creates the broadcast channel.
    pub fn set_trace_tx(&self, tx: tokio::sync::broadcast::Sender<TraceEvent>) {
        // SAFETY: trace_tx is thread-safe via Mutex; poison recovery via into_inner()
        *self.trace_tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    }
}
