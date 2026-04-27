//! EditApplier integration tests (Day 1).
//!
//! Tests real file system operations, trace broadcast, multi-hunk apply,
//! atomic rollback, and governance integration.

use agent_core::{
    edit_applier::{EditApplier, EditHunk, ProposedEdit, AppliedEdit, EditState, edit_summary},
    governance::{DefaultGovernance, AgentGovernance, GovernanceRequest, ApprovalLevel, Decision, Vote, GovernancePolicy},
    checkpoint::CheckpointManager,
    agent_loop::{TraceEvent, TraceStepType},
    AgentContext, AgentId,
};
use std::sync::Arc;

fn test_context() -> AgentContext { AgentContext::new() }
fn test_governance() -> Arc<DefaultGovernance> { Arc::new(DefaultGovernance::new()) }
fn test_checkpoint_mgr() -> Arc<CheckpointManager> { Arc::new(CheckpointManager::new()) }

async fn write_temp_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(format!("hajimi_edit_test_{}_{}", name, uuid::Uuid::new_v4().simple()));
    let path_str = path.to_str().unwrap().to_string();
    tokio::fs::write(&path_str, content).await.expect("write temp file");
    path_str
}

#[tokio::test]
async fn test_end_to_end_file_apply() {
    let path = write_temp_file("e2e", "fn old() {}\nfn main() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-1".to_string();

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["fn old() {}".to_string()],
        new_lines: vec!["fn new() { println!(\"ok\"); }".to_string()],
        start_line: 1,
        confidence: 0.95,
    };

    let proposed = ProposedEdit {
        id: "e2e-1".to_string(),
        hunks: vec![hunk],
        summary: "Rename old to new".to_string(),
        confidence_score: 0.95,
        rationale: "Better naming".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    assert_eq!(applied.hunks_applied, 1);
    assert!(applied.before_token_count > 0);
    assert!(applied.after_token_count > applied.before_token_count);

    let content = tokio::fs::read_to_string(&path).await.expect("read");
    assert!(content.contains("fn new()"));
    assert!(!content.contains("fn old()"));

    // Cleanup
    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_multi_hunk_single_file() {
    let path = write_temp_file("multi", "line1\nline2\nline3\nline4\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-2".to_string();

    let hunks = vec![
        EditHunk {
            file_path: path.clone(),
            old_lines: vec!["line1".to_string()],
            new_lines: vec!["modified1".to_string()],
            start_line: 1,
            confidence: 0.9,
        },
        EditHunk {
            file_path: path.clone(),
            old_lines: vec!["line3".to_string()],
            new_lines: vec!["modified3".to_string()],
            start_line: 3,
            confidence: 0.9,
        },
    ];

    let proposed = ProposedEdit {
        id: "multi-1".to_string(),
        hunks,
        summary: "Multi hunk test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    assert_eq!(applied.hunks_applied, 2);

    let content = tokio::fs::read_to_string(&path).await.expect("read");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "modified1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "modified3");
    assert_eq!(lines[3], "line4");

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_trace_broadcast_received() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let (tx, mut rx) = tokio::sync::broadcast::channel(16);
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context()).with_trace_tx(tx);
    let agent_id: AgentId = "agent-trace".to_string();

    let hunk = EditHunk {
        file_path: write_temp_file("trace", "old\n").await,
        old_lines: vec!["old".to_string()],
        new_lines: vec!["new".to_string()],
        start_line: 1,
        confidence: 0.9,
    };
    let path = hunk.file_path.clone();

    let proposed = ProposedEdit {
        id: "trace-1".to_string(),
        hunks: vec![hunk],
        summary: "Trace test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let _ = applier.apply(&proposed, &agent_id).await.expect("apply");

    // Verify trace events were broadcast
    let mut found_proposed = false;
    let mut found_applied = false;
    while let Ok(event) = rx.try_recv() {
        match event.step_type {
            TraceStepType::EditProposed => found_proposed = true,
            TraceStepType::EditApplied => found_applied = true,
            _ => {}
        }
    }
    assert!(found_proposed, "EditProposed trace event should be broadcast");
    assert!(found_applied, "EditApplied trace event should be broadcast");

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_apply_rejected_without_review() {
    let path = write_temp_file("noreview", "content\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-3".to_string();

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["content".to_string()],
        new_lines: vec!["changed".to_string()],
        start_line: 1,
        confidence: 0.9,
    };

    let proposed = ProposedEdit {
        id: "noreview-1".to_string(),
        hunks: vec![hunk],
        summary: "No review test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    // Skip review
    let result = applier.apply(&proposed, &agent_id).await;
    assert!(result.is_err(), "Apply without review should fail");

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_atomic_failure_no_partial_write() {
    let path = write_temp_file("atomic", "line1\nline2\nline3\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-4".to_string();

    // First hunk is valid, second hunk has mismatch → should fail atomically for that file
    let hunks = vec![
        EditHunk {
            file_path: path.clone(),
            old_lines: vec!["line1".to_string()],
            new_lines: vec!["modified1".to_string()],
            start_line: 1,
            confidence: 0.9,
        },
        EditHunk {
            file_path: path.clone(),
            old_lines: vec!["wrong_line".to_string()], // mismatch
            new_lines: vec!["modified3".to_string()],
            start_line: 3,
            confidence: 0.9,
        },
    ];

    let proposed = ProposedEdit {
        id: "atomic-1".to_string(),
        hunks,
        summary: "Atomic failure test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let result = applier.apply(&proposed, &agent_id).await;
    assert!(result.is_err(), "Should fail due to hunk mismatch");

    // Verify file was NOT modified
    let content = tokio::fs::read_to_string(&path).await.expect("read");
    assert!(content.contains("line1"), "File should remain unchanged after failed apply");
    assert!(content.contains("line2"));
    assert!(content.contains("line3"));

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_insertion_at_line_boundary() {
    let path = write_temp_file("insert", "line1\nline2\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-5".to_string();

    // Insert after line 2 (at end)
    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec![],
        new_lines: vec!["line3".to_string()],
        start_line: 3, // after line 2
        confidence: 0.9,
    };

    let proposed = ProposedEdit {
        id: "insert-1".to_string(),
        hunks: vec![hunk],
        summary: "Insertion test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let _ = applier.apply(&proposed, &agent_id).await.expect("apply");

    let content = tokio::fs::read_to_string(&path).await.expect("read");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[2], "line3");

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_undo_stack_ordering() {
    let path = write_temp_file("undo", "original\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-6".to_string();

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["original".to_string()],
        new_lines: vec!["changed".to_string()],
        start_line: 1,
        confidence: 0.9,
    };

    let proposed = ProposedEdit {
        id: "undo-1".to_string(),
        hunks: vec![hunk],
        summary: "Undo test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    let undone = applier.undo_last(&agent_id).await.expect("undo").expect("should have undo entry");
    assert_eq!(undone.edit_id, applied.edit_id);
    assert_eq!(undone.checkpoint_id, applied.checkpoint_id);

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_real_token_count_after_apply() {
    let path = write_temp_file("tokens", "fn main() {\n    println!(\"hello\");\n}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-7".to_string();

    // Replace short line with longer line → token count should increase
    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["    println!(\"hello\");".to_string()],
        new_lines: vec!["    println!(\"hello world this is longer\");".to_string()],
        start_line: 2,
        confidence: 0.9,
    };

    let proposed = ProposedEdit {
        id: "token-1".to_string(),
        hunks: vec![hunk],
        summary: "Token count test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    // Original: fn main() { (3) + println!("hello"); (2) + } (1) = 6 tokens (whitespace split)
    // New: fn main() { (3) + println!("hello world this is longer"); (5) + } (1) = 9 tokens
    assert!(applied.before_token_count > 0);
    assert!(applied.after_token_count > applied.before_token_count);

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_empty_edit_noop() {
    let path = write_temp_file("noop", "unchanged\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-8".to_string();

    let proposed = ProposedEdit {
        id: "noop-1".to_string(),
        hunks: vec![], // no hunks
        summary: "No-op edit".to_string(),
        confidence_score: 1.0,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    assert_eq!(applied.hunks_applied, 0);

    let content = tokio::fs::read_to_string(&path).await.expect("read");
    assert_eq!(content, "unchanged\n");

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_governance_reject_apply_level() {
    use agent_core::governance::{AgentGovernance, GovernancePolicy};
    use async_trait::async_trait;

    struct RejectApplyGovernance;
    #[async_trait]
    impl AgentGovernance for RejectApplyGovernance {
        async fn policy(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> ApprovalLevel { ApprovalLevel::Auto }
        async fn approve(&self, _ctx: &AgentContext, req: &GovernanceRequest) -> Result<Decision, chimera_repl::traits::ReplError> {
            if req.action_type == "apply_edit" {
                Ok(Decision::Rejected("apply rejected".to_string()))
            } else {
                Ok(Decision::Approved)
            }
        }
        async fn vote(&self, _voter_id: &str, _proposal_id: &str, _vote: Vote) -> Result<(), chimera_repl::traits::ReplError> { Ok(()) }
        async fn escalate(&self, req: &GovernanceRequest, _to_level: ApprovalLevel) -> Result<GovernanceRequest, chimera_repl::traits::ReplError> { Ok(req.clone()) }
        async fn register_policy(&mut self, _name: &str, _policy: Arc<dyn GovernancePolicy>, _caller: &str, _required_level: agent_core::governance::PermissionLevel) -> Result<(), chimera_repl::traits::ReplError> { Ok(()) }
        async fn record_feedback(&self, _ctx: &AgentContext, _feedback: &agent_core::governance::UserFeedback) -> Result<(), chimera_repl::traits::ReplError> { Ok(()) }
    }

    let path = write_temp_file("gov_apply", "content\n").await;
    let governance: Arc<dyn AgentGovernance> = Arc::new(RejectApplyGovernance);
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-9".to_string();

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["content".to_string()],
        new_lines: vec!["changed".to_string()],
        start_line: 1,
        confidence: 0.9,
    };

    let proposed = ProposedEdit {
        id: "gov-1".to_string(),
        hunks: vec![hunk],
        summary: "Governance apply rejection test".to_string(),
        confidence_score: 0.9,
        rationale: "".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose should succeed (Advisory level)");
    let _ = applier.review(true, &agent_id).await.expect("review");
    let result = applier.apply(&proposed, &agent_id).await;
    assert!(result.is_err(), "Apply should be rejected by governance");

    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_edit_summary_format() {
    let applied = AppliedEdit {
        edit_id: "test-1".to_string(),
        hunks_applied: 3,
        before_token_count: 42,
        after_token_count: 55,
        checkpoint_id: "chk_abc".to_string(),
        timestamp: chrono::Utc::now(),
        backup_paths: std::collections::HashMap::new(),
    };
    let summary = edit_summary(&applied);
    assert!(summary.contains("3 hunks"));
    assert!(summary.contains("42→55"));
    assert!(summary.contains("chk_abc"));
}
