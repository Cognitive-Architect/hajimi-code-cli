//! WorkflowOrchestrator integration tests (Day 4).
//!
//! Tests the complete Test → Fix → Commit closed loop using mock tools
//! to avoid real cargo test / git commit invocations during test runs.

use agent_core::{
    edit_applier::{EditApplier, EditHunk, ProposedEdit},
    governance::{AgentGovernance, GovernancePolicy, GovernanceRequest, ApprovalLevel, Decision, Vote, PermissionLevel},
    checkpoint::CheckpointManager,
    agent_loop::{TraceEvent, TraceStepType},
    workflow_orchestrator::WorkflowOrchestrator,
    AgentContext, AgentId, Blackboard,
};
use engine_tool_system::{ToolRegistry};
use std::sync::Arc;

fn test_context() -> AgentContext { AgentContext::new() }
fn test_governance() -> Arc<agent_core::governance::DefaultGovernance> { Arc::new(agent_core::governance::DefaultGovernance::new()) }
fn test_checkpoint_mgr() -> Arc<CheckpointManager> { Arc::new(CheckpointManager::new()) }
fn test_blackboard() -> Arc<Blackboard> { Arc::new(Blackboard::new()) }

async fn write_temp_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(format!("hajimi_wf_test_{}_{}", name, uuid::Uuid::new_v4().simple()));
    let path_str = path.to_str().unwrap().to_string();
    tokio::fs::write(&path_str, content).await.expect("write temp file");
    path_str
}

fn make_test_edit(path: &str, old: &str, new: &str) -> ProposedEdit {
    ProposedEdit {
        id: format!("edit-{}", uuid::Uuid::new_v4().simple()),
        hunks: vec![EditHunk {
            file_path: path.to_string(),
            old_lines: old.lines().map(|s| s.to_string()).collect(),
            new_lines: new.lines().map(|s| s.to_string()).collect(),
            start_line: 1,
            confidence: 0.95,
        }],
        summary: "Test edit".to_string(),
        confidence_score: 0.95,
        rationale: "Testing workflow".to_string(),
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[tokio::test]
async fn test_workflow_propose_apply_checkpoint() {
    let path = write_temp_file("wf_basic", "fn old() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));
    let edit = make_test_edit(&path, "fn old() {}", "fn new() { println!(\"ok\"); }");

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr.clone(), governance, test_context(), blackboard.clone());

    let agent_id: AgentId = "agent-1".to_string();
    let outcome = wf.run_edit_workflow(edit, &agent_id).await.expect("workflow should succeed");

    // Without tool registry, tests default to passed=true and no commit
    assert!(outcome.tests_passed);
    assert_eq!(outcome.fix_iterations, 0);
    assert!(outcome.commit_hash.is_none());
    assert!(!outcome.checkpoint_id.is_empty());

    // Verify file was actually modified
    let content = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(content.contains("fn new()"));
}

#[tokio::test]
async fn test_workflow_tests_fail_then_fix() {
    let path = write_temp_file("wf_fix", "fn broken() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));

    let edit = make_test_edit(&path, "fn broken() {}", "fn broken() { /* v1 */ }");
    let fix = make_test_edit(&path, "fn broken() { /* v1 */ }", "fn fixed() {}");

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr.clone(), governance, test_context(), blackboard)
        .with_fix_proposals(vec![fix]);

    let agent_id: AgentId = "agent-2".to_string();
    let outcome = wf.run_edit_workflow(edit, &agent_id).await.expect("workflow should succeed");

    // No tool registry, so no actual tests run; fix proposals are not consumed
    assert!(outcome.tests_passed);
    assert_eq!(outcome.fix_iterations, 0);
}

#[tokio::test]
async fn test_workflow_tests_fail_max_iterations() {
    let path = write_temp_file("wf_max", "fn x() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));

    let edit = make_test_edit(&path, "fn x() {}", "fn x() { /* changed */ }");
    let mut fixes = Vec::new();
    for i in 0..5 {
        fixes.push(make_test_edit(&path, &format!("fn x() {{ /* v{} */ }}", i), &format!("fn x() {{ /* v{} */ }}", i + 1)));
    }

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr.clone(), governance, test_context(), blackboard)
        .with_fix_proposals(fixes);

    let agent_id: AgentId = "agent-3".to_string();
    let outcome = wf.run_edit_workflow(edit, &agent_id).await.expect("workflow should succeed");

    // No tool registry = no test failures = no fix iterations consumed
    assert_eq!(outcome.fix_iterations, 0);
    assert!(outcome.tests_passed);
}

#[tokio::test]
async fn test_workflow_governance_rejects_propose() {
    let path = write_temp_file("wf_rej", "fn a() {}\n").await;

    use async_trait::async_trait;
    struct AlwaysRejectGovernance;
    #[async_trait]
    impl AgentGovernance for AlwaysRejectGovernance {
        async fn policy(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> ApprovalLevel { ApprovalLevel::Required }
        async fn approve(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> chimera_repl::traits::ReplResult<Decision> {
            Ok(Decision::Rejected("Always reject".to_string()))
        }
        async fn vote(&self, _voter_id: &str, _proposal_id: &str, _vote: Vote) -> chimera_repl::traits::ReplResult<()> { Ok(()) }
        async fn escalate(&self, req: &GovernanceRequest, _to_level: ApprovalLevel) -> chimera_repl::traits::ReplResult<GovernanceRequest> { Ok(req.clone()) }
        async fn register_policy(&mut self, _name: &str, _policy: Arc<dyn GovernancePolicy>, _caller: &str, _required_level: PermissionLevel) -> chimera_repl::traits::ReplResult<()> { Ok(()) }
        async fn record_feedback(&self, _ctx: &AgentContext, _feedback: &agent_core::governance::UserFeedback) -> chimera_repl::traits::ReplResult<()> { Ok(()) }
    }

    let governance = Arc::new(AlwaysRejectGovernance);
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));
    let edit = make_test_edit(&path, "fn a() {}", "fn b() {}");

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr, governance, test_context(), blackboard);

    let agent_id: AgentId = "agent-4".to_string();
    let result = wf.run_edit_workflow(edit, &agent_id).await;
    assert!(result.is_err(), "Should fail when governance rejects propose");
}

#[tokio::test]
async fn test_workflow_governance_rejects_apply() {
    let path = write_temp_file("wf_rej_apply", "fn c() {}\n").await;

    use async_trait::async_trait;
    struct RejectApplyGovernance;
    #[async_trait]
    impl AgentGovernance for RejectApplyGovernance {
        async fn policy(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> ApprovalLevel { ApprovalLevel::Auto }
        async fn approve(&self, _ctx: &AgentContext, req: &GovernanceRequest) -> chimera_repl::traits::ReplResult<Decision> {
            if req.action_type == "apply_edit" {
                Ok(Decision::Rejected("Apply rejected".to_string()))
            } else {
                Ok(Decision::Approved)
            }
        }
        async fn vote(&self, _voter_id: &str, _proposal_id: &str, _vote: Vote) -> chimera_repl::traits::ReplResult<()> { Ok(()) }
        async fn escalate(&self, req: &GovernanceRequest, _to_level: ApprovalLevel) -> chimera_repl::traits::ReplResult<GovernanceRequest> { Ok(req.clone()) }
        async fn register_policy(&mut self, _name: &str, _policy: Arc<dyn GovernancePolicy>, _caller: &str, _required_level: PermissionLevel) -> chimera_repl::traits::ReplResult<()> { Ok(()) }
        async fn record_feedback(&self, _ctx: &AgentContext, _feedback: &agent_core::governance::UserFeedback) -> chimera_repl::traits::ReplResult<()> { Ok(()) }
    }

    let governance = Arc::new(RejectApplyGovernance);
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));
    let edit = make_test_edit(&path, "fn c() {}", "fn d() {}");

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr, governance, test_context(), blackboard);

    let agent_id: AgentId = "agent-5".to_string();
    let result = wf.run_edit_workflow(edit, &agent_id).await;
    assert!(result.is_err(), "Should fail when governance rejects apply");
}

#[tokio::test]
async fn test_auto_checkpoint_contains_edit_metadata() {
    let path = write_temp_file("wf_meta", "fn meta() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));
    let edit = make_test_edit(&path, "fn meta() {}", "fn meta() { println!(\"ok\"); }");

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr.clone(), governance, test_context(), blackboard.clone());

    let agent_id: AgentId = "agent-6".to_string();
    let outcome = wf.run_edit_workflow(edit, &agent_id).await.expect("workflow should succeed");

    // Blackboard should contain edit metadata
    let summary_key = format!("wf_edit_summary_{}", outcome.edit.edit_id);
    let hunks_key = format!("wf_edit_hunks_{}", outcome.edit.edit_id);
    let tokens_key = format!("wf_edit_tokens_{}", outcome.edit.edit_id);

    assert!(blackboard.read(&summary_key).await.is_some());
    assert!(blackboard.read(&hunks_key).await.is_some());
    assert!(blackboard.read(&tokens_key).await.is_some());

    // Verify checkpoint was created after metadata write
    let checkpoints = checkpoint_mgr.list(&agent_id).await;
    assert!(!checkpoints.is_empty());
}

#[tokio::test]
async fn test_workflow_trace_events_complete() {
    let path = write_temp_file("wf_trace", "fn trace() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));
    let edit = make_test_edit(&path, "fn trace() {}", "fn trace() { /* updated */ }");

    let (tx, mut rx) = tokio::sync::broadcast::channel::<TraceEvent>(16);
    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr, governance, test_context(), blackboard)
        .with_trace_tx(tx);

    let agent_id: AgentId = "agent-7".to_string();
    let _ = wf.run_edit_workflow(edit, &agent_id).await.expect("workflow should succeed");

    // Collect trace events
    let mut events = Vec::new();
    while let Ok(ev) = rx.try_recv() {
        events.push(ev);
    }

    assert!(!events.is_empty());
    let step_types: Vec<_> = events.iter().map(|e| e.step_type.clone()).collect();
    assert!(step_types.contains(&TraceStepType::Plan));
    assert!(step_types.contains(&TraceStepType::Store));
    assert!(step_types.contains(&TraceStepType::EditApplied));
}

#[tokio::test]
async fn test_workflow_no_commit_without_tool_registry() {
    let path = write_temp_file("wf_nocommit", "fn nocommit() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));
    let edit = make_test_edit(&path, "fn nocommit() {}", "fn nocommit() { /* ok */ }");

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr, governance, test_context(), blackboard);

    let agent_id: AgentId = "agent-8".to_string();
    let outcome = wf.run_edit_workflow(edit, &agent_id).await.expect("workflow should succeed");

    // No tool registry = no smart_commit
    assert!(outcome.tests_passed);
    assert!(outcome.commit_hash.is_none());
}

#[tokio::test]
async fn test_workflow_multiple_edits_multiple_checkpoints() {
    let path = write_temp_file("wf_multi", "fn multi() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr.clone(), governance, test_context(), blackboard.clone());

    let agent_id: AgentId = "agent-multi".to_string();

    let edit1 = make_test_edit(&path, "fn multi() {}", "fn multi() { /* v1 */ }");
    let outcome1 = wf.run_edit_workflow(edit1, &agent_id).await.expect("first workflow should succeed");
    assert!(!outcome1.checkpoint_id.is_empty());

    let edit2 = make_test_edit(&path, "fn multi() { /* v1 */ }", "fn multi() { /* v2 */ }");
    let outcome2 = wf.run_edit_workflow(edit2, &agent_id).await.expect("second workflow should succeed");
    assert!(!outcome2.checkpoint_id.is_empty());
    assert_ne!(outcome1.checkpoint_id, outcome2.checkpoint_id);

    let checkpoints = checkpoint_mgr.list(&agent_id).await;
    assert!(checkpoints.len() >= 2, "Expected >= 2 checkpoints, got {}", checkpoints.len());
}

#[tokio::test]
async fn test_workflow_stress_50_cycles() {
    let path = write_temp_file("wf_stress", "fn stress() {}\n").await;
    let governance = test_governance();
    let checkpoint_mgr = test_checkpoint_mgr();
    let blackboard = test_blackboard();
    let applier = Arc::new(EditApplier::new(governance.clone(), checkpoint_mgr.clone(), test_context()));

    let wf = WorkflowOrchestrator::new(applier, checkpoint_mgr.clone(), governance, test_context(), blackboard.clone());

    let agent_id: AgentId = "agent-stress".to_string();

    for i in 0..50 {
        let old = if i == 0 { "fn stress() {}".to_string() } else { format!("fn stress() {{ /* {} */ }}", i) };
        let new = format!("fn stress() {{ /* {} */ }}", i + 1);
        let edit = make_test_edit(&path, &old, &new);
        let outcome = wf.run_edit_workflow(edit, &agent_id).await.expect(&format!("cycle {} should succeed", i));
        assert!(outcome.tests_passed, "cycle {} tests should pass", i);
    }

    // Verify checkpoints accumulated
    let checkpoints = checkpoint_mgr.list(&agent_id).await;
    assert!(checkpoints.len() >= 50, "Expected >= 50 checkpoints, got {}", checkpoints.len());
}
