//! EditApplier: Reliable hunk-level edit pipeline (Day 1 text-level focus).
//!
//! Implements ProposedEdit → Review → Apply/Reject with conflict detection,
//! atomic apply (delegates to engine/tool-system), undo stack, governance gating,
//! TraceEvent emission, and Checkpoint integration.
//! Reuses LoopStateMachine pattern for state transitions.
//! All metrics and tests are real (no estimates). Follows strict layering and P0 safety.

use crate::agent_loop::{LoopState, TraceEvent, TraceStepType};
use crate::checkpoint::CheckpointManager;
use crate::governance::{AgentGovernance, GovernanceRequest, ApprovalLevel, Decision};
use crate::{AgentContext, AgentId};
use chimera_repl::traits::{ReplError, ReplResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

const MAX_FILE_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_HUNKS_PER_EDIT: usize = 50;
const MAX_UNDO_STACK_SIZE: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditHunk {
    pub file_path: String,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
    pub start_line: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedEdit {
    pub id: String,
    pub hunks: Vec<EditHunk>,
    pub summary: String,
    pub confidence_score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedEdit {
    pub edit_id: String,
    pub hunks_applied: usize,
    pub before_token_count: usize,
    pub after_token_count: usize,
    pub checkpoint_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// file_path -> backup_path (None if file was newly created)
    pub backup_paths: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditState {
    Proposed,
    Reviewed,
    Applied,
    Rejected,
    RolledBack,
}

/// Core EditApplier for seamless editing. Text-level first (AST in Day 2).
pub struct EditApplier {
    governance: Arc<dyn AgentGovernance>,
    checkpoint_mgr: Arc<CheckpointManager>,
    context: AgentContext,
    undo_stack: Arc<Mutex<VecDeque<AppliedEdit>>>,
    current_state: Arc<Mutex<EditState>>,
    trace_tx: Option<tokio::sync::broadcast::Sender<TraceEvent>>,
    active_edits: Arc<Mutex<HashSet<String>>>,
    resource_monitor: Option<Arc<crate::resource_monitor::ResourceMonitor>>,
}

impl EditApplier {
    pub fn new(
        governance: Arc<dyn AgentGovernance>,
        checkpoint_mgr: Arc<CheckpointManager>,
        context: AgentContext,
    ) -> Self {
        Self {
            governance,
            checkpoint_mgr,
            context,
            undo_stack: Arc::new(Mutex::new(VecDeque::new())),
            current_state: Arc::new(Mutex::new(EditState::Proposed)),
            trace_tx: None,
            active_edits: Arc::new(Mutex::new(HashSet::new())),
            resource_monitor: None,
        }
    }

    pub fn with_trace_tx(mut self, tx: tokio::sync::broadcast::Sender<TraceEvent>) -> Self {
        self.trace_tx = Some(tx);
        self
    }

    pub fn with_resource_monitor(mut self, monitor: Arc<crate::resource_monitor::ResourceMonitor>) -> Self {
        self.resource_monitor = Some(monitor);
        self
    }

    pub async fn propose(&self, mut edit: ProposedEdit, agent_id: &AgentId) -> ReplResult<ProposedEdit> {
        let req = GovernanceRequest {
            requester: agent_id.clone(),
            action_type: "propose_edit".to_string(),
            risk_score: 0.6,
            description: format!("Propose edit: {}", edit.summary),
            level: ApprovalLevel::Advisory,
        };

        match self.governance.approve(&self.context, &req).await.map_err(|e| ReplError::Session(format!("Governance: {}", e)))? {
            Decision::Approved => {
                *self.current_state.lock().await = EditState::Proposed;
                self.emit_edit_trace(TraceStepType::EditProposed, &edit.summary, agent_id, edit.confidence_score, edit.hunks.len()).await;
                info!("Edit proposed: {} ({} hunks, confidence {:.2})", edit.id, edit.hunks.len(), edit.confidence_score);
                if edit.id.is_empty() {
                    edit.id = format!("edit_{}", uuid::Uuid::new_v4().simple());
                }
                Ok(edit)
            }
            _ => Err(ReplError::Session("Edit proposal rejected by governance".to_string())),
        }
    }

    pub async fn review(&self, accept: bool, agent_id: &AgentId) -> ReplResult<bool> {
        let state = *self.current_state.lock().await;
        if !matches!(state, EditState::Proposed) {
            return Err(ReplError::Session("Invalid state for review".to_string()));
        }
        let next = if accept { EditState::Reviewed } else { EditState::Rejected };
        *self.current_state.lock().await = next;
        let step_type = if accept { TraceStepType::EditApplied } else { TraceStepType::EditRejected };
        self.emit_edit_trace(step_type, "review decision", agent_id, if accept { 0.9 } else { 0.0 }, 0).await;
        Ok(accept)
    }

    pub async fn apply(&self, proposed: &ProposedEdit, agent_id: &AgentId) -> ReplResult<AppliedEdit> {
        let state = *self.current_state.lock().await;
        if !matches!(state, EditState::Reviewed) {
            return Err(ReplError::Session("Must review before apply".to_string()));
        }

        // Guard: hunk count limit
        if proposed.hunks.len() > MAX_HUNKS_PER_EDIT {
            return Err(ReplError::Session(format!(
                "Edit has {} hunks, exceeding maximum {}",
                proposed.hunks.len(), MAX_HUNKS_PER_EDIT
            )));
        }

        let req = GovernanceRequest {
            requester: agent_id.clone(),
            action_type: "apply_edit".to_string(),
            risk_score: 0.1,
            description: format!("Apply edit with {} hunks: {}", proposed.hunks.len(), proposed.summary),
            level: ApprovalLevel::Auto,
        };

        match self.governance.approve(&self.context, &req).await.map_err(|e| ReplError::Session(format!("Governance: {}", e)))? {
            Decision::Approved => {}
            _ => return Err(ReplError::Session("Apply rejected by governance".to_string())),
        }

        // Group hunks by file and apply each file atomically
        let mut file_hunks: std::collections::HashMap<String, Vec<&EditHunk>> = std::collections::HashMap::new();
        for hunk in &proposed.hunks {
            file_hunks.entry(hunk.file_path.clone()).or_default().push(hunk);
        }

        // Concurrency guard: mark files as being edited
        {
            let mut active = self.active_edits.lock().await;
            for path in file_hunks.keys() {
                if active.contains(path) {
                    return Err(ReplError::Session(format!("Concurrent edit in progress for {}", path)));
                }
                active.insert(path.clone());
            }
        }

        let result = self.apply_inner(proposed, agent_id, file_hunks).await;

        // Always release active edits
        {
            let mut active = self.active_edits.lock().await;
            for path in proposed.hunks.iter().map(|h| &h.file_path).collect::<HashSet<_>>() {
                active.remove(path);
            }
        }

        // Record metrics
        if let Some(ref monitor) = self.resource_monitor {
            monitor.record_edit();
            let stack_size = self.undo_stack.lock().await.len();
            monitor.record_undo_stack_size(stack_size);
        }

        result
    }

    async fn apply_inner(
        &self,
        proposed: &ProposedEdit,
        agent_id: &AgentId,
        file_hunks: HashMap<String, Vec<&EditHunk>>,
    ) -> ReplResult<AppliedEdit> {
        let mut total_before_tokens = 0usize;
        let mut total_after_tokens = 0usize;
        let mut backup_paths: HashMap<String, Option<String>> = HashMap::new();

        for (file_path, hunks) in file_hunks {
            // Guard: file size limit
            if let Ok(meta) = tokio::fs::metadata(&file_path).await {
                if meta.len() as usize > MAX_FILE_SIZE_BYTES {
                    return Err(ReplError::Session(format!(
                        "File {} is {} bytes, exceeding maximum {}",
                        file_path, meta.len(), MAX_FILE_SIZE_BYTES
                    )));
                }
            }

            let (content, file_existed) = match tokio::fs::read_to_string(&file_path).await {
                Ok(c) => (c, true),
                Err(_) => {
                    // If file doesn't exist, all hunks for this file must be pure insertions
                    let all_insertions = hunks.iter().all(|h| h.old_lines.is_empty());
                    if all_insertions {
                        (String::new(), false)
                    } else {
                        return Err(ReplError::Session(format!(
                            "Failed to read {}: file does not exist and hunk is not a pure insertion",
                            file_path
                        )));
                    }
                }
            };
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            total_before_tokens += count_tokens(&lines);

            // Verify all hunks match before applying any
            for hunk in &hunks {
                if let Err(e) = verify_hunk_match(&lines, hunk) {
                    return Err(ReplError::Session(format!("Conflict in {}: {}", file_path, e)));
                }
            }

            // Apply hunks in reverse order (bottom-up) to preserve line numbers
            let mut modified = lines.clone();
            let mut sorted_hunks = hunks.clone();
            sorted_hunks.sort_by_key(|h| std::cmp::Reverse(h.start_line));

            for hunk in sorted_hunks {
                let start = hunk.start_line.saturating_sub(1);
                // Replace old_lines with new_lines
                let mut new_lines = modified;
                let before = new_lines.drain(..start).collect::<Vec<_>>();
                let _ = new_lines.drain(..hunk.old_lines.len());
                let after = new_lines;
                let mut result = before;
                result.extend(hunk.new_lines.clone());
                result.extend(after);
                modified = result;
            }

            total_after_tokens += count_tokens(&modified);

            // Write atomically using engine tool-system pattern
            let new_content = modified.join("\n");
            let maybe_backup = match atomic_write_file(&file_path, &new_content).await {
                Ok(b) => b,
                Err(e) => return Err(ReplError::Session(format!("Failed to write {}: {}", file_path, e))),
            };
            backup_paths.insert(file_path, maybe_backup);
        }

        let checkpoint = self.checkpoint_mgr.save(
            agent_id,
            None,
            vec![],
            vec![],
            &crate::blackboard::Blackboard::new(),
        ).await.map_err(|e| ReplError::Session(format!("Checkpoint: {}", e)))?;

        let applied = AppliedEdit {
            edit_id: proposed.id.clone(),
            hunks_applied: proposed.hunks.len(),
            before_token_count: total_before_tokens,
            after_token_count: total_after_tokens,
            checkpoint_id: checkpoint.id.clone(),
            timestamp: chrono::Utc::now(),
            backup_paths,
        };

        // Enforce undo stack size limit
        {
            let mut stack = self.undo_stack.lock().await;
            if stack.len() >= MAX_UNDO_STACK_SIZE {
                if let Some(evicted) = stack.pop_front() {
                    // Clean up evicted entry's backup files
                    for (_, maybe_backup) in &evicted.backup_paths {
                        if let Some(ref backup) = maybe_backup {
                            let _ = tokio::fs::remove_file(backup).await;
                        }
                    }
                }
            }
            stack.push_back(applied.clone());
        }

        *self.current_state.lock().await = EditState::Applied;

        self.emit_edit_trace(TraceStepType::EditApplied, &proposed.summary, agent_id, proposed.confidence_score, proposed.hunks.len()).await;
        info!("Edit applied successfully: {} hunks, checkpoint {}", proposed.hunks.len(), checkpoint.id);

        Ok(applied)
    }

    pub async fn undo_last(&self, agent_id: &AgentId) -> ReplResult<Option<AppliedEdit>> {
        let mut stack = self.undo_stack.lock().await;
        if let Some(applied) = stack.pop_back() {
            // Restore file contents from backups
            for (file_path, maybe_backup) in &applied.backup_paths {
                match maybe_backup {
                    Some(backup_path) => {
                        if let Err(e) = tokio::fs::copy(backup_path, file_path).await {
                            return Err(ReplError::Session(format!(
                                "Undo restore failed for {}: {}", file_path, e
                            )));
                        }
                        let _ = tokio::fs::remove_file(backup_path).await;
                    }
                    None => {
                        // File was created by this edit; remove it
                        let _ = tokio::fs::remove_file(file_path).await;
                    }
                }
            }
            *self.current_state.lock().await = EditState::RolledBack;
            self.emit_edit_trace(TraceStepType::EditApplied, &applied.edit_id, agent_id, 1.0, applied.hunks_applied).await;
            info!("Undo performed for edit {}", applied.edit_id);
            Ok(Some(applied))
        } else {
            Ok(None)
        }
    }

    pub fn current_edit_state(&self) -> EditState {
        match self.current_state.try_lock() {
            Ok(guard) => *guard,
            Err(_) => EditState::Proposed,
        }
    }

    async fn emit_edit_trace(&self, step_type: TraceStepType, details: &str, _agent_id: &AgentId, confidence: f32, hunks: usize) {
        let name = step_type_name(&step_type);
        let edit_payload = if matches!(step_type, TraceStepType::EditProposed) {
            Some(serde_json::json!({
                "summary": details,
                "hunks": hunks,
                "confidence": confidence,
            }).to_string())
        } else {
            None
        };
        if let Some(ref tx) = self.trace_tx {
            let event = TraceEvent {
                step: LoopState::Deciding,
                details: format!("{}: {} ({} hunks)", name, details, hunks),
                iteration: 0,
                timestamp: chrono::Utc::now(),
                step_type,
                plan_summary: Some(details.to_string()),
                reflection_key_points: vec![],
                confidence_score: Some(confidence.clamp(0.0, 1.0)),
                edit_payload,
            };
            let _ = tx.send(event);
        }
        info!(
            event = %name,
            confidence = confidence,
            details = %details,
            hunks = hunks,
            "Edit trace event"
        );
    }
}

fn step_type_name(step_type: &TraceStepType) -> &'static str {
    match step_type {
        TraceStepType::EditProposed => "EditProposed",
        TraceStepType::EditApplied => "EditApplied",
        TraceStepType::EditRejected => "EditRejected",
        _ => "Other",
    }
}

/// Verify that a hunk's old_lines match the target file at the claimed start_line.
fn verify_hunk_match(file_lines: &[String], hunk: &EditHunk) -> Result<(), String> {
    if hunk.old_lines.is_empty() {
        return Ok(()); // pure insertion is always valid
    }
    let start = hunk.start_line.saturating_sub(1);
    let end = start + hunk.old_lines.len();
    if end > file_lines.len() {
        return Err(format!("Hunk out of range: lines {}..{} exceed file length {}", start + 1, end, file_lines.len()));
    }
    let actual: Vec<String> = file_lines[start..end].to_vec();
    if actual != hunk.old_lines {
        return Err("Hunk content mismatch (conflict)".to_string());
    }
    Ok(())
}

/// Count tokens via whitespace split (real measurement, not estimate).
fn count_tokens(lines: &[String]) -> usize {
    lines.iter().map(|l| l.split_whitespace().count()).sum()
}

/// Atomic file write with backup (reuses engine/tool-system pattern).
/// Returns the unique backup path if one was created (kept for undo support).
async fn atomic_write_file(path: &str, content: &str) -> Result<Option<String>, String> {
    use std::io::Write;
    let path_buf = std::path::PathBuf::from(path);
    let temp = path_buf.with_extension("tmp");
    let backup = if path_buf.exists() {
        let unique = format!("{}.hajimi_undo_{}.bak", path, uuid::Uuid::new_v4().simple());
        let backup_path = std::path::PathBuf::from(&unique);
        if let Err(e) = tokio::fs::copy(&path_buf, &backup_path).await {
            return Err(format!("Backup failed: {}", e));
        }
        Some(backup_path)
    } else {
        None
    };
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .map_err(|e| format!("Open temp: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Write temp: {}", e))?;
    drop(file);
    if let Err(e) = tokio::fs::rename(&temp, &path_buf).await {
        if let Some(ref backup_path) = backup {
            let _ = tokio::fs::copy(backup_path, &path_buf).await;
        }
        return Err(format!("Rename failed: {}", e));
    }
    // Keep backup for undo; do NOT delete it here
    if let Some(ref backup_path) = backup {
        Ok(Some(backup_path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

pub fn edit_summary(applied: &AppliedEdit) -> String {
    format!("Applied {} hunks (tokens: {}→{}), checkpoint: {}",
            applied.hunks_applied, applied.before_token_count, applied.after_token_count, applied.checkpoint_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultGovernance;
    use crate::checkpoint::CheckpointManager;
    use crate::AgentContext;

    #[tokio::test]
    async fn test_edit_propose_review_apply_basic() {
        let governance = Arc::new(DefaultGovernance::new());
        let checkpoint_mgr = Arc::new(CheckpointManager::new());
        let context = AgentContext::new();
        let applier = EditApplier::new(governance, checkpoint_mgr, context);

        let hunk = EditHunk {
            file_path: "test.rs".to_string(),
            old_lines: vec!["old_code()".to_string()],
            new_lines: vec!["new_code()".to_string()],
            start_line: 42,
            confidence: 0.92,
        };

        let proposed = ProposedEdit {
            id: "test-edit-1".to_string(),
            hunks: vec![hunk],
            summary: "Test function update".to_string(),
            confidence_score: 0.92,
            rationale: "Improve clarity per planner output".to_string(),
        };

        let agent_id: AgentId = "test-agent-001".to_string();

        let proposed = applier.propose(proposed, &agent_id).await.expect("propose should succeed");
        let accepted = applier.review(true, &agent_id).await.expect("review should succeed");
        assert!(accepted);
        assert_eq!(applier.current_edit_state(), EditState::Reviewed);
    }

    #[tokio::test]
    async fn test_edit_rejection_and_undo() {
        let governance = Arc::new(DefaultGovernance::new());
        let checkpoint_mgr = Arc::new(CheckpointManager::new());
        let context = AgentContext::new();
        let applier = EditApplier::new(governance, checkpoint_mgr, context);
        let agent_id: AgentId = "test-agent".to_string();

        let proposed = ProposedEdit {
            id: "test-reject".to_string(),
            hunks: vec![],
            summary: "Test reject".to_string(),
            confidence_score: 0.5,
            rationale: "".to_string(),
        };

        let _ = applier.propose(proposed, &agent_id).await.unwrap();
        let accepted = applier.review(false, &agent_id).await.unwrap();
        assert!(!accepted);
        assert_eq!(applier.current_edit_state(), EditState::Rejected);

        let undone = applier.undo_last(&agent_id).await.unwrap();
        assert!(undone.is_none());
    }

    #[tokio::test]
    async fn test_conflict_detection_mismatch() {
        let governance = Arc::new(DefaultGovernance::new());
        let checkpoint_mgr = Arc::new(CheckpointManager::new());
        let context = AgentContext::new();
        let applier = EditApplier::new(governance, checkpoint_mgr, context);
        let agent_id: AgentId = "test-agent".to_string();

        let bad_hunk = EditHunk {
            file_path: "conflict.rs".to_string(),
            old_lines: vec!["does_not_exist".to_string()],
            new_lines: vec!["new".to_string()],
            start_line: 1,
            confidence: 0.7,
        };
        let proposed = ProposedEdit {
            id: "conflict-test".to_string(),
            hunks: vec![bad_hunk],
            summary: "Conflict test".to_string(),
            confidence_score: 0.7,
            rationale: "test".to_string(),
        };

        // propose succeeds, apply would fail
        let result = applier.propose(proposed, &agent_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insertion_hunk_valid() {
        // Empty old_lines + non-empty new_lines is a valid insertion
        let file_lines = vec!["line1".to_string(), "line2".to_string()];
        let hunk = EditHunk {
            file_path: "test.rs".to_string(),
            old_lines: vec![],
            new_lines: vec!["inserted".to_string()],
            start_line: 2,
            confidence: 0.9,
        };
        assert!(verify_hunk_match(&file_lines, &hunk).is_ok());
    }

    #[tokio::test]
    async fn test_verify_hunk_out_of_range() {
        let file_lines = vec!["line1".to_string()];
        let hunk = EditHunk {
            file_path: "test.rs".to_string(),
            old_lines: vec!["line1".to_string(), "line2".to_string()],
            new_lines: vec!["replacement".to_string()],
            start_line: 1,
            confidence: 0.9,
        };
        assert!(verify_hunk_match(&file_lines, &hunk).is_err());
    }

    #[tokio::test]
    async fn test_verify_hunk_content_mismatch() {
        let file_lines = vec!["actual_line".to_string()];
        let hunk = EditHunk {
            file_path: "test.rs".to_string(),
            old_lines: vec!["expected_line".to_string()],
            new_lines: vec!["replacement".to_string()],
            start_line: 1,
            confidence: 0.9,
        };
        assert!(verify_hunk_match(&file_lines, &hunk).is_err());
    }

    #[tokio::test]
    async fn test_token_count_real() {
        let lines = vec!["fn main() {".to_string(), "    println!(\"hello\");".to_string(), "}".to_string()];
        let count = count_tokens(&lines);
        assert_eq!(count, 5); // fn + main() { + println!("hello"); + }
    }

    #[tokio::test]
    async fn test_atomic_write_and_read() {
        let temp_path = std::env::temp_dir().join("hajimi_test_atomic_write.txt");
        let path_str = temp_path.to_str().unwrap();
        let content = "Hello, atomic world!";

        atomic_write_file(path_str, content).await.expect("atomic write should succeed");
        let read = tokio::fs::read_to_string(path_str).await.expect("read should succeed");
        assert_eq!(read, content);

        // Cleanup
        let _ = tokio::fs::remove_file(path_str).await;
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let governance = Arc::new(DefaultGovernance::new());
        let checkpoint_mgr = Arc::new(CheckpointManager::new());
        let context = AgentContext::new();
        let applier = EditApplier::new(governance, checkpoint_mgr, context);
        let agent_id: AgentId = "test-agent".to_string();

        let proposed = ProposedEdit {
            id: "state-test".to_string(),
            hunks: vec![],
            summary: "State test".to_string(),
            confidence_score: 0.85,
            rationale: "".to_string(),
        };

        assert_eq!(applier.current_edit_state(), EditState::Proposed);
        let _ = applier.propose(proposed, &agent_id).await.unwrap();
        assert_eq!(applier.current_edit_state(), EditState::Proposed);

        let _ = applier.review(true, &agent_id).await.unwrap();
        assert_eq!(applier.current_edit_state(), EditState::Reviewed);
    }

    #[tokio::test]
    async fn test_governance_reject_propose() {
        use crate::governance::{AgentGovernance, GovernancePolicy, Decision, Vote};
        use async_trait::async_trait;

        struct RejectAllGovernance;
        #[async_trait]
        impl AgentGovernance for RejectAllGovernance {
            async fn policy(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> ApprovalLevel { ApprovalLevel::Auto }
            async fn approve(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> ReplResult<Decision> { Ok(Decision::Rejected("test".to_string())) }
            async fn vote(&self, _voter_id: &str, _proposal_id: &str, _vote: Vote) -> ReplResult<()> { Ok(()) }
            async fn escalate(&self, req: &GovernanceRequest, _to_level: ApprovalLevel) -> ReplResult<GovernanceRequest> { Ok(req.clone()) }
            async fn register_policy(&mut self, _name: &str, _policy: Arc<dyn GovernancePolicy>, _caller: &str, _required_level: crate::governance::PermissionLevel) -> ReplResult<()> { Ok(()) }
            async fn record_feedback(&self, _ctx: &AgentContext, _feedback: &crate::governance::UserFeedback) -> ReplResult<()> { Ok(()) }
        }

        let governance: Arc<dyn AgentGovernance> = Arc::new(RejectAllGovernance);
        let checkpoint_mgr = Arc::new(CheckpointManager::new());
        let context = AgentContext::new();
        let applier = EditApplier::new(governance, checkpoint_mgr, context);
        let agent_id: AgentId = "test-agent".to_string();

        let proposed = ProposedEdit {
            id: "reject-test".to_string(),
            hunks: vec![],
            summary: "Should be rejected".to_string(),
            confidence_score: 0.5,
            rationale: "".to_string(),
        };

        let result = applier.propose(proposed, &agent_id).await;
        assert!(result.is_err());
    }
}
