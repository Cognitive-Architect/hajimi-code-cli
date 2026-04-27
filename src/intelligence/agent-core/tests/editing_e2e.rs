//! Phase 4 Day 6: Comprehensive E2E tests for edge cases, memory optimization,
//! and stress testing.
//!
//! Covers: undo file restoration, size guards, hunk limits, checkpoint pruning,
//! ResourceMonitor integration, undo stack eviction, re-apply cycles, and stress.

use agent_core::{
    edit_applier::{EditApplier, EditHunk, ProposedEdit},
    governance::DefaultGovernance,
    checkpoint::CheckpointManager,
    resource_monitor::ResourceMonitor,
    AgentContext, AgentId,
};
use std::sync::Arc;

fn test_context() -> AgentContext { AgentContext::new() }
fn test_governance() -> Arc<DefaultGovernance> { Arc::new(DefaultGovernance::new()) }
fn test_checkpoint_mgr() -> Arc<CheckpointManager> { Arc::new(CheckpointManager::new()) }

async fn write_temp_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(format!("hajimi_e2e_{}_{}", name, uuid::Uuid::new_v4().simple()));
    let path_str = path.to_str().unwrap().to_string();
    tokio::fs::write(&path_str, content).await.expect("write temp file");
    path_str
}

// ------------------------------------------------------------------
// 1. Undo actually restores original file content
// ------------------------------------------------------------------
#[tokio::test]
async fn test_undo_actually_restores_file_content() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-undo".to_string();

    let original = "fn old() {}\nfn main() {}\n";
    let path = write_temp_file("undo_restore", original).await;

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["fn old() {}".to_string()],
        new_lines: vec!["fn new() { println!(\"ok\"); }".to_string()],
        start_line: 1,
        confidence: 0.95,
    };
    let proposed = ProposedEdit {
        id: "undo-1".to_string(),
        hunks: vec![hunk],
        summary: "Rename old to new".to_string(),
        confidence_score: 0.95,
        rationale: "Better naming".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    applier.review(true, &agent_id).await.expect("review");
    let _applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    let after_apply = tokio::fs::read_to_string(&path).await.expect("read after apply");
    assert!(after_apply.contains("fn new()"), "File should be modified after apply");
    assert!(!after_apply.contains("fn old()"), "Old content should be gone");

    let undone = applier.undo_last(&agent_id).await.expect("undo");
    assert!(undone.is_some(), "Undo should return the applied edit");

    let after_undo = tokio::fs::read_to_string(&path).await.expect("read after undo");
    assert_eq!(after_undo, original, "File should be restored to original content after undo");

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 2. Large file (>10MB) is rejected
// ------------------------------------------------------------------
#[tokio::test]
async fn test_large_file_apply_rejected() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-large".to_string();

    let big_content = "x".repeat(11 * 1024 * 1024); // 11 MB
    let path = write_temp_file("large", &big_content).await;

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["x".to_string()],
        new_lines: vec!["y".to_string()],
        start_line: 1,
        confidence: 0.9,
    };
    let proposed = ProposedEdit {
        id: "large-1".to_string(),
        hunks: vec![hunk],
        summary: "Modify large file".to_string(),
        confidence_score: 0.9,
        rationale: "test".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    applier.review(true, &agent_id).await.expect("review");
    let result = applier.apply(&proposed, &agent_id).await;

    assert!(result.is_err(), "Apply should fail for oversized file");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("exceeding maximum"), "Error should mention size limit: {}", err);

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 3. Too many hunks (>50) is rejected
// ------------------------------------------------------------------
#[tokio::test]
async fn test_too_many_hunks_rejected() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-hunks".to_string();

    let path = write_temp_file("hunks", "line1\n").await;
    let mut hunks = Vec::new();
    for i in 0..51 {
        hunks.push(EditHunk {
            file_path: path.clone(),
            old_lines: vec!["line1".to_string()],
            new_lines: vec![format!("modified_{}", i)],
            start_line: 1,
            confidence: 0.9,
        });
    }
    let proposed = ProposedEdit {
        id: "hunks-1".to_string(),
        hunks,
        summary: "Too many hunks".to_string(),
        confidence_score: 0.9,
        rationale: "test".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    applier.review(true, &agent_id).await.expect("review");
    let result = applier.apply(&proposed, &agent_id).await;

    assert!(result.is_err(), "Apply should fail for too many hunks");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("exceeding maximum"), "Error should mention hunk limit: {}", err);

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 4. Apply to a new file then undo removes the file
// ------------------------------------------------------------------
#[tokio::test]
async fn test_apply_new_file_then_undo_removes_it() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-newfile".to_string();

    // Path that does NOT exist yet
    let path = std::env::temp_dir()
        .join(format!("hajimi_newfile_{}.txt", uuid::Uuid::new_v4().simple()));
    let path_str = path.to_str().unwrap().to_string();

    // Hunk with empty old_lines means pure insertion (new file)
    let hunk = EditHunk {
        file_path: path_str.clone(),
        old_lines: vec![],
        new_lines: vec!["new content".to_string()],
        start_line: 1,
        confidence: 0.9,
    };
    let proposed = ProposedEdit {
        id: "newfile-1".to_string(),
        hunks: vec![hunk],
        summary: "Create new file".to_string(),
        confidence_score: 0.9,
        rationale: "test".to_string(),
    };

    let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
    applier.review(true, &agent_id).await.expect("review");
    let _applied = applier.apply(&proposed, &agent_id).await.expect("apply");

    assert!(tokio::fs::metadata(&path_str).await.is_ok(), "File should exist after apply");

    let undone = applier.undo_last(&agent_id).await.expect("undo");
    assert!(undone.is_some());

    assert!(tokio::fs::metadata(&path_str).await.is_err(), "File should be removed after undo");
}

// ------------------------------------------------------------------
// 5. Checkpoint auto-prune keeps only MAX_CHECKPOINTS_PER_AGENT
// ------------------------------------------------------------------
#[tokio::test]
async fn test_checkpoint_auto_prune() {
    let monitor = Arc::new(ResourceMonitor::new());
    let checkpoint_mgr = Arc::new(CheckpointManager::new().with_resource_monitor(monitor.clone()));
    let agent_id: AgentId = "agent-prune".to_string();

    for i in 0..105 {
        let bb = agent_core::blackboard::Blackboard::new();
        let _ = checkpoint_mgr.save(&agent_id, None, vec![], vec![], &bb).await.expect("save");
        if i == 104 {
            // After the 105th save, pruning should have occurred
            let list = checkpoint_mgr.list(&agent_id).await;
            assert_eq!(list.len(), 100, "Should prune to MAX_CHECKPOINTS_PER_AGENT (100)");
        }
    }

    let final_list = checkpoint_mgr.list(&agent_id).await;
    assert_eq!(final_list.len(), 100, "Final count should be exactly 100");
}

// ------------------------------------------------------------------
// 6. ResourceMonitor tracks edit and undo-stack metrics
// ------------------------------------------------------------------
#[tokio::test]
async fn test_resource_monitor_edit_metrics() {
    let monitor = Arc::new(ResourceMonitor::new());
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context())
        .with_resource_monitor(monitor.clone());
    let agent_id: AgentId = "agent-metrics".to_string();

    let path = write_temp_file("metrics", "line1\nline2\n").await;

    let mut current = "line1".to_string();
    for i in 0..3 {
        let hunk = EditHunk {
            file_path: path.clone(),
            old_lines: vec![current.clone()],
            new_lines: vec![format!("edit{}", i)],
            start_line: 1,
            confidence: 0.9,
        };
        let proposed = ProposedEdit {
            id: format!("metrics-{}", i),
            hunks: vec![hunk],
            summary: format!("Edit {}", i),
            confidence_score: 0.9,
            rationale: "test".to_string(),
        };
        let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
        applier.review(true, &agent_id).await.expect("review");
        applier.apply(&proposed, &agent_id).await.expect("apply");
        current = format!("edit{}", i);
    }

    let metrics = monitor.get_metrics();
    assert_eq!(metrics.edit_count, 3, "Should have recorded 3 edits");
    assert_eq!(metrics.undo_stack_size, 3, "Undo stack should have 3 entries");

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 7. Undo stack max size eviction cleans oldest backups
// ------------------------------------------------------------------
#[tokio::test]
async fn test_undo_stack_max_size_eviction() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-evict".to_string();

    let base_path = std::env::temp_dir().join(format!("hajimi_evict_{}", uuid::Uuid::new_v4().simple()));
    let path = base_path.to_str().unwrap().to_string();
    tokio::fs::write(&path, "base\n").await.expect("write");

    // Apply 101 small edits to trigger eviction
    let mut current = "base".to_string();
    for i in 0..101 {
        let hunk = EditHunk {
            file_path: path.clone(),
            old_lines: vec![current.clone()],
            new_lines: vec![format!("v{}", i)],
            start_line: 1,
            confidence: 0.9,
        };
        let proposed = ProposedEdit {
            id: format!("evict-{}", i),
            hunks: vec![hunk],
            summary: format!("Evict edit {}", i),
            confidence_score: 0.9,
            rationale: "test".to_string(),
        };
        let proposed = applier.propose(proposed, &agent_id).await.expect("propose");
        applier.review(true, &agent_id).await.expect("review");
        applier.apply(&proposed, &agent_id).await.expect("apply");
        current = format!("v{}", i);
    }

    // The first edit's backup should have been evicted and removed
    // We verify by attempting to undo all 101 — the first one is gone from stack,
    // so only 100 undos should be possible.
    let mut undo_count = 0;
    while applier.undo_last(&agent_id).await.expect("undo check").is_some() {
        undo_count += 1;
    }
    assert_eq!(undo_count, 100, "Only 100 undos possible after evicting the oldest");

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 8. Apply → undo → re-apply cycle works end-to-end
// ------------------------------------------------------------------
#[tokio::test]
async fn test_apply_then_undo_then_reapply() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-cycle".to_string();

    let original = "fn alpha() {}\n";
    let path = write_temp_file("cycle", original).await;

    let hunk = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["fn alpha() {}".to_string()],
        new_lines: vec!["fn beta() {}".to_string()],
        start_line: 1,
        confidence: 0.9,
    };
    let proposed = ProposedEdit {
        id: "cycle-1".to_string(),
        hunks: vec![hunk],
        summary: "Rename alpha to beta".to_string(),
        confidence_score: 0.9,
        rationale: "test".to_string(),
    };

    // First apply
    let p1 = applier.propose(proposed.clone(), &agent_id).await.expect("propose 1");
    applier.review(true, &agent_id).await.expect("review 1");
    applier.apply(&p1, &agent_id).await.expect("apply 1");
    let content_after_1 = tokio::fs::read_to_string(&path).await.expect("read 1");
    assert!(content_after_1.contains("fn beta()"));

    // Undo
    applier.undo_last(&agent_id).await.expect("undo");
    let content_after_undo = tokio::fs::read_to_string(&path).await.expect("read undo");
    assert_eq!(content_after_undo, original, "Should restore original");

    // Re-apply (new proposal with same content)
    let p2 = applier.propose(proposed.clone(), &agent_id).await.expect("propose 2");
    applier.review(true, &agent_id).await.expect("review 2");
    applier.apply(&p2, &agent_id).await.expect("apply 2");
    let content_after_2 = tokio::fs::read_to_string(&path).await.expect("read 2");
    assert!(content_after_2.contains("fn beta()"), "Should be modified again after re-apply");

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 9. Hunk line shift after previous edit causes conflict on stale hunk
// ------------------------------------------------------------------
#[tokio::test]
async fn test_hunk_line_shift_after_previous_edit() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-shift".to_string();

    let original = "line1\nline2\nline3\n";
    let path = write_temp_file("shift", original).await;

    // First edit: insert before line 2, shifting line 3 down
    let hunk1 = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["line2".to_string()],
        new_lines: vec!["line2a".to_string(), "line2".to_string()],
        start_line: 2,
        confidence: 0.9,
    };
    let proposed1 = ProposedEdit {
        id: "shift-1".to_string(),
        hunks: vec![hunk1],
        summary: "Insert line2a".to_string(),
        confidence_score: 0.9,
        rationale: "test".to_string(),
    };
    let p1 = applier.propose(proposed1, &agent_id).await.expect("propose 1");
    applier.review(true, &agent_id).await.expect("review 1");
    applier.apply(&p1, &agent_id).await.expect("apply 1");

    // Second edit: targets original line 3, which is now at line 4
    // This should fail because the hunk references stale line numbers/content
    let hunk2 = EditHunk {
        file_path: path.clone(),
        old_lines: vec!["line3".to_string()],
        new_lines: vec!["line3modified".to_string()],
        start_line: 3,
        confidence: 0.9,
    };
    let proposed2 = ProposedEdit {
        id: "shift-2".to_string(),
        hunks: vec![hunk2],
        summary: "Modify line3".to_string(),
        confidence_score: 0.9,
        rationale: "test".to_string(),
    };
    let p2 = applier.propose(proposed2, &agent_id).await.expect("propose 2");
    applier.review(true, &agent_id).await.expect("review 2");
    let result = applier.apply(&p2, &agent_id).await;

    assert!(result.is_err(), "Should fail because line 3 is now at line 4 after previous insertion");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Conflict") || err.contains("mismatch") || err.contains("out of range"),
            "Error should indicate conflict: {}", err);

    let _ = tokio::fs::remove_file(&path).await;
}

// ------------------------------------------------------------------
// 10. Stress: 50 consecutive applies on the same file
// ------------------------------------------------------------------
#[tokio::test]
async fn test_stress_50_consecutive_applies() {
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let applier = EditApplier::new(governance, checkpoint_mgr, test_context());
    let agent_id: AgentId = "agent-stress".to_string();

    let path = write_temp_file("stress", "value: 0\n").await;

    for i in 0..50 {
        let hunk = EditHunk {
            file_path: path.clone(),
            old_lines: vec![format!("value: {}", i)],
            new_lines: vec![format!("value: {}", i + 1)],
            start_line: 1,
            confidence: 0.9,
        };
        let proposed = ProposedEdit {
            id: format!("stress-{}", i),
            hunks: vec![hunk],
            summary: format!("Increment to {}", i + 1),
            confidence_score: 0.9,
            rationale: "stress test".to_string(),
        };
        let proposed = applier.propose(proposed, &agent_id).await.expect(&format!("propose {}", i));
        applier.review(true, &agent_id).await.expect(&format!("review {}", i));
        applier.apply(&proposed, &agent_id).await.expect(&format!("apply {}", i));
    }

    let final_content = tokio::fs::read_to_string(&path).await.expect("read final");
    assert!(final_content.contains("value: 50"), "Final content should show value: 50");

    let _ = tokio::fs::remove_file(&path).await;
}
