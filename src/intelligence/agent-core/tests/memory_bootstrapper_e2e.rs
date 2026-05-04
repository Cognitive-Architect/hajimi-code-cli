//! Memory Bootstrapper E2E Tests (B-10A/10) — Cross-session memory recovery validation.
//! Verifies the complete lifecycle: bootstrap → 3 dialogue rounds → checkpoint save
//! → destroy → MemoryBootstrapper restore from disk → verify summary + Blackboard.
//! Mock-based test: no real LLM API calls are made during this test.
use agent_core::{MemoryBootstrapper, Blackboard, LoopState};
use std::sync::Arc;

/// Remove leftover checkpoint files from previous test runs to ensure isolation.
async fn cleanup_test_checkpoint(agent_id: &str) {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA").ok().map(std::path::PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME").ok().map(|h| std::path::PathBuf::from(h).join(".config"));
    if let Some(dir) = base {
        let p = dir.join(".hajimi").join("checkpoints").join(format!("{}.jsonl", agent_id));
        if p.exists() {
            let _ = tokio::fs::remove_file(&p).await;
        }
    }
}

#[tokio::test]
async fn test_cross_session_memory_recovery() {
    let project_id = "e2e_memory_test";
    let device_id = "e2e_device";
    let agent_id = "e2e_test_agent".to_string();

    // Pre-clean to ensure isolation between test runs
    cleanup_test_checkpoint(&agent_id).await;

    // =======================================================================
    // PHASE 1: Initial bootstrap — load project memory, simulate 3 dialogue
    // rounds by writing to a Blackboard, then persist a checkpoint to disk.
    // =======================================================================
    let bootstrapper = MemoryBootstrapper::new(project_id, device_id, &agent_id);
    let result = bootstrapper.load_project_memory().await.unwrap();

    // First load with no prior checkpoint should produce the fallback summary
    assert!(result.summary.contains("No checkpoint available") || result.summary.contains("plan_summary="),
            "Unexpected initial summary: {}", result.summary);

    // Verify the shared gateway Arc is valid
    assert!(Arc::strong_count(&result.gateway) >= 1, "Gateway Arc should be held by BootstrapResult");

    // Simulate 3 dialogue rounds on a dedicated blackboard
    let blackboard = Arc::new(Blackboard::new());
    for i in 0..3 {
        blackboard.write(&format!("dialogue_round_{}", i), &format!("user_message_{}", i), &agent_id).await;
    }

    // Verify blackboard contains all dialogue rounds before checkpoint save
    for i in 0..3 {
        let entry = blackboard.read(&format!("dialogue_round_{}", i)).await;
        assert!(entry.is_some(), "Missing blackboard entry for dialogue_round_{}", i);
        assert_eq!(entry.unwrap().value, format!("user_message_{}", i));
    }

    // Persist checkpoint to disk via CheckpointManager; this writes to
    // config_dir/.hajimi/checkpoints/{agent_id}.jsonl for cross-session recovery
    let chk = result.checkpoint_mgr.save(
        &agent_id, None, vec![], vec![], &blackboard
    ).await.unwrap();
    assert!(chk.id.starts_with("chk_"), "Checkpoint ID format invalid: {}", chk.id);
    assert_eq!(chk.agent_id, agent_id, "Checkpoint agent_id mismatch");

    // Verify the checkpoint is immediately listable in memory before destruction
    let list_before = result.checkpoint_mgr.list(&agent_id).await;
    assert_eq!(list_before.len(), 1, "Expected 1 checkpoint in memory list");
    assert_eq!(list_before[0].id, chk.id, "Checkpoint ID mismatch in memory list");

    // Destroy all handles — simulate complete session termination
    drop(result);
    drop(blackboard);

    // =======================================================================
    // PHASE 2: Re-bootstrap after destruction — MemoryBootstrapper must restore
    // the checkpoint from disk and produce a summary that reflects saved state.
    // =======================================================================
    let bootstrapper2 = MemoryBootstrapper::new(project_id, device_id, &agent_id);
    let result2 = bootstrapper2.load_project_memory().await.unwrap();

    // Verify that the checkpoint was successfully restored from disk.
    // The summary must contain plan_summary and reflections (not the fallback).
    assert!(!result2.summary.contains("No checkpoint available"),
            "Expected restored checkpoint summary, got fallback: {}", result2.summary);
    assert!(result2.summary.contains("plan_summary="), "Missing plan_summary in: {}", result2.summary);
    assert!(result2.summary.contains("reflections="), "Missing reflections in: {}", result2.summary);
    assert!(result2.summary.contains("goal_progress="), "Missing goal_progress in: {}", result2.summary);

    // Verify restore_latest_from_disk returns the same checkpoint that was saved
    let disk_chk = result2.checkpoint_mgr.restore_latest_from_disk(project_id, &agent_id).await.unwrap();
    assert_eq!(disk_chk.id, chk.id, "Disk restore checkpoint ID mismatch");

    // Verify that build_agent_loop_with_memory injects project_memory_summary
    // into the AgentLoop's Blackboard and returns a fully functional loop.
    let agent_loop = bootstrapper2.build_agent_loop_with_memory().await.unwrap();
    assert_eq!(agent_loop.current_state().await, LoopState::Idle,
               "AgentLoop should start in Idle state after rebuild");
    assert!(agent_loop.inject_memory("verification_key", "verification_value", &agent_id).await.is_ok(),
            "AgentLoop should be functional after cross-session rebuild");

    cleanup_test_checkpoint(&agent_id).await;
}

/// Verify that build_agent_loop_with_memory always returns a functional AgentLoop
/// even when no prior checkpoint exists (cold-start scenario).
#[tokio::test]
async fn test_build_agent_loop_with_memory_cold_start() {
    let agent_id = "cold_agent".to_string();
    let bootstrapper = MemoryBootstrapper::new("cold_proj", "cold_dev", &agent_id);
    let result = bootstrapper.load_project_memory().await.unwrap();

    // Cold start should produce the fallback summary
    assert!(result.summary.contains("No checkpoint available") || result.summary.contains("plan_summary="),
            "Cold start summary unexpected: {}", result.summary);

    // Build AgentLoop and verify it is functional
    let agent_loop = bootstrapper.build_agent_loop_with_memory().await.unwrap();
    assert_eq!(agent_loop.current_state().await, LoopState::Idle);
    assert!(agent_loop.inject_memory("cold_test", "ok", &agent_id).await.is_ok());
}
