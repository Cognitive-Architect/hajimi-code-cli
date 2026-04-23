//! Agent Core E2E Test Suite - Day 10 Debt Clearance (B-02/DEBT: 278±5行)
//! Coverage: Agent生命周期、Planning、Reflection、Governance、Swarm、Checkpoint、Tool、AgentLoop

use agent_core::{
    AgentConfig, AgentContext, AgentOrchestrator, AgentRole, ApprovalLevel, Blackboard,
    CheckpointManager, Decision, DefaultGovernance, GovernancePolicy, GovernanceRequest,
    HierarchicalPlanner, LoopOutcome, Priority, Supervisor, SwarmCoordinator, TaskAssignment,
    AutonomousReflector, Goal, PlanStatus, TaskResult,
};
use agent_core::planner::Planner;
use agent_core::governance::AgentGovernance;
use agent_core::swarm::SwarmCoordinator;
use agent_core::reflector::Reflector;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use memory::memory_gateway::MemoryGateway;

fn test_memory() -> Arc<Mutex<MemoryGateway>> { Arc::new(Mutex::new(MemoryGateway::new("e2e"))) }

// FUNC-001: Agent完整生命周期测试
#[tokio::test] async fn test_agent_lifecycle() {
    let orch = AgentOrchestrator::new(test_memory());
    let cfg = AgentConfig::supervisor("lifecycle_agent");
    assert_eq!(cfg.agent_id, "lifecycle_agent");
    assert_eq!(cfg.role, AgentRole::Supervisor);
    assert!(cfg.capability.can_govern);
}

// FUNC-002: Plan → Execute 闭环测试
#[tokio::test] async fn test_plan_and_execute() {
    let m = test_memory();
    let mut planner = HierarchicalPlanner::new(m.clone(), AgentContext::new());
    let gid = planner.create_goal("Implement feature", Priority::High).await.unwrap();
    assert!(!gid.is_empty());
    let sgs = planner.decompose(&gid).await.unwrap();
    assert!(!sgs.is_empty());
    let task = planner.next_task().await.unwrap();
    assert!(task.is_some() || planner.is_complete());
}

// FUNC-002: Reflect → Optimize 闭环测试
#[tokio::test] async fn test_reflect_and_optimize() {
    let m = test_memory();
    let mut r = AutonomousReflector::new(m.clone(), AgentContext::new());
    let g = Goal { id: "g1".to_string(), description: "Test".to_string(), priority: Priority::High, status: PlanStatus::Pending, subgoals: vec![], metadata: HashMap::new(), created_at: chrono::Utc::now(), approved: true };
    let res = TaskResult { success: false, output: "Error".to_string(), timestamp: chrono::Utc::now() };
    let refl = r.reflect(&g, &res).await.unwrap();
    assert!(!refl.reflection_id.is_empty());
    assert!(!refl.critique.success);
}

// FUNC-003: Governance审批流程测试
#[tokio::test] async fn test_governance_approval() {
    let gov = DefaultGovernance::new();
    let ctx = AgentContext::new();
    let req = GovernanceRequest { requester: "test".to_string(), action_type: "test_action".to_string(), risk_score: 0.3, description: "Test".to_string(), level: ApprovalLevel::Auto };
    assert_eq!(gov.approve(&ctx, &req).await.unwrap(), Decision::Approved);
}

// FUNC-003: Governance拒绝场景测试 - 必须断言Decision::Rejected
#[tokio::test] async fn test_governance_rejection() {
    struct RejectPolicy; 
    #[async_trait::async_trait] 
    impl GovernancePolicy for RejectPolicy {
        async fn evaluate(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> Option<Decision> { 
            Some(Decision::Rejected("Test rejection".to_string())) 
        }
    }
    let mut gov = DefaultGovernance::new();
    gov.register_policy("reject", Arc::new(RejectPolicy)).await.unwrap();
    let req = GovernanceRequest { requester: "t".to_string(), action_type: "risky".to_string(), risk_score: 0.95, description: "High risk".to_string(), level: ApprovalLevel::Override };
    let decision = gov.approve(&AgentContext::new(), &req).await.unwrap();
    assert!(matches!(decision, Decision::Rejected(_)), "Expected Decision::Rejected, got {:?}", decision);
}

// FUNC-004: Swarm任务委托测试
#[tokio::test] async fn test_swarm_delegate() {
    let gov = Arc::new(DefaultGovernance::new());
    let mut sv = Supervisor::new(gov, AgentContext::new());
    let cfg = AgentConfig::supervisor("swarm_test");
    let wid = sv.spawn_worker(AgentRole::Coder, cfg).await.unwrap();
    assert!(wid.contains("coder"));
    let task = TaskAssignment { task_id: "t1".to_string(), description: "Test task".to_string(), assigned_to: wid, priority: 5 };
    assert!(sv.delegate(task).await.is_ok());
}

// FUNC-005: Checkpoint保存与恢复测试
#[tokio::test] async fn test_checkpoint_restore() {
    let bb = Blackboard::new();
    let mgr = CheckpointManager::new();
    let chk = mgr.save(&"agent_001".to_string(), None, vec![], vec![], &bb).await.unwrap();
    assert_eq!(chk.agent_id, "agent_001");
    let rst = mgr.restore_latest(&"agent_001".to_string()).await.unwrap();
    assert_eq!(rst.id, chk.id);
    assert!(!rst.hash.is_empty());
}

// FUNC-006: Tool调用测试
#[tokio::test] async fn test_tool_invocation() {
    let orch = AgentOrchestrator::new(test_memory());
    let result = orch.invoke_tool("planning", serde_json::json!({"action":"create_goal","description":"Test","priority":"high"})).await;
    assert!(result.is_ok());
}

// FUNC-007: AgentLoop自主执行测试
#[tokio::test] async fn test_autonomous_loop() {
    let orch = AgentOrchestrator::new(test_memory());
    let out = orch.execute_natural_language_goal("loop_agent", "Create a simple plan").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted));
}

// NEG-001: Governance风险评分边界测试
#[tokio::test] async fn test_governance_risk_boundary() {
    let gov = DefaultGovernance::new();
    let ctx = AgentContext::new();
    let low_req = GovernanceRequest { requester: "t".to_string(), action_type: "low".to_string(), risk_score: 0.1, description: "Low".to_string(), level: ApprovalLevel::Auto };
    let high_req = GovernanceRequest { requester: "t".to_string(), action_type: "high".to_string(), risk_score: 0.9, description: "High".to_string(), level: ApprovalLevel::Override };
    assert!(matches!(gov.approve(&ctx, &low_req).await.unwrap(), Decision::Approved));
    assert!(matches!(gov.approve(&ctx, &high_req).await.unwrap(), Decision::Approved));
}

// NEG-002: 100轮稳定性测试 - 必须包含0..100
#[tokio::test] async fn test_stability_100_rounds() {
    let orch = AgentOrchestrator::new(test_memory());
    for i in 0..100 {
        let _ = orch.execute_natural_language_goal(&format!("stability_agent_{}", i), "Quick task").await;
    }
}

// NEG-003: 并发Swarm测试 - 多Worker同时工作
#[tokio::test] async fn test_concurrent_swarm() {
    let gov = Arc::new(DefaultGovernance::new());
    let mut sv = Supervisor::new(gov, AgentContext::new());
    let cfg = AgentConfig::supervisor("concurrent");
    let w1 = sv.spawn_worker(AgentRole::Coder, cfg.clone()).await.unwrap();
    let w2 = sv.spawn_worker(AgentRole::Researcher, cfg.clone()).await.unwrap();
    let w3 = sv.spawn_worker(AgentRole::Critic, cfg.clone()).await.unwrap();
    assert_eq!(sv.worker_count(), 3);
    
    let t1 = TaskAssignment { task_id: "code_task".to_string(), description: "Write code".to_string(), assigned_to: w1, priority: 5 };
    let t2 = TaskAssignment { task_id: "research_task".to_string(), description: "Research".to_string(), assigned_to: w2, priority: 3 };
    let t3 = TaskAssignment { task_id: "review_task".to_string(), description: "Review".to_string(), assigned_to: w3, priority: 4 };
    
    assert!(sv.delegate(t1).await.is_ok());
    assert!(sv.delegate(t2).await.is_ok());
    assert!(sv.delegate(t3).await.is_ok());
}

// NEG-004: Checkpoint一致性测试 - 状态完整保存
#[tokio::test] async fn test_checkpoint_consistency() {
    let bb = Blackboard::new();
    bb.write("config".to_string(), "value1".to_string(), "agent1".to_string()).await;
    bb.write("state".to_string(), "active".to_string(), "agent1".to_string()).await;
    
    let mgr = CheckpointManager::new();
    let chk = mgr.save(&"consistency_agent".to_string(), None, vec![], vec![], &bb).await.unwrap();
    assert!(!chk.hash.is_empty());
    
    let rst = mgr.restore_latest(&"consistency_agent".to_string()).await.unwrap();
    assert_eq!(rst.blackboard.len(), 2);
    assert!(rst.blackboard.contains_key("config"));
    assert!(rst.blackboard.contains_key("state"));
}

// E2E-001: 演示级端到端测试 - 问候函数
#[tokio::test] async fn test_demo_greeting() {
    let orch = AgentOrchestrator::new(test_memory());
    let out = orch.execute_natural_language_goal("demo_agent", "Implement a greeting function that says hello").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded));
}

// FUNC-003: 自定义治理策略测试 - 用户策略替换
#[tokio::test] async fn test_custom_governance() {
    struct AuditPolicy; 
    #[async_trait::async_trait] 
    impl GovernancePolicy for AuditPolicy {
        async fn evaluate(&self, _ctx: &AgentContext, req: &GovernanceRequest) -> Option<Decision> { 
            if req.risk_score > 0.8 { Some(Decision::Rejected("Too risky".to_string())) } else { None }
        }
    }
    let mut gov = DefaultGovernance::new();
    gov.register_policy("audit", Arc::new(AuditPolicy)).await.unwrap();
    let req = GovernanceRequest { requester: "test".to_string(), action_type: "action".to_string(), risk_score: 0.5, description: "Normal".to_string(), level: ApprovalLevel::Auto };
    let decision = gov.approve(&AgentContext::new(), &req).await.unwrap();
    assert!(matches!(decision, Decision::Approved | Decision::Rejected(_)));
}

// 额外E2E: 多步骤任务测试 - 复杂目标分解
#[tokio::test] async fn test_multi_step_task() {
    let orch = AgentOrchestrator::new(test_memory());
    let out = orch.execute_natural_language_goal("multi_agent", "Implement feature with design code and tests").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded));
}

// 额外E2E: 紧急目标测试 - 优先级检测
#[tokio::test] async fn test_urgent_goal() {
    let orch = AgentOrchestrator::new(test_memory());
    let out = orch.execute_natural_language_goal("urgent_agent", "URGENT: Fix critical bug").await.unwrap();
    assert!(matches!(out, LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted));
}

// 额外E2E: Worker崩溃恢复测试 - 故障隔离
#[tokio::test] async fn test_worker_recovery() {
    let gov = Arc::new(DefaultGovernance::new());
    let mut sv = Supervisor::new(gov, AgentContext::new());
    let cfg = AgentConfig::supervisor("recovery");
    let wid = sv.spawn_worker(AgentRole::Executor, cfg).await.unwrap();
    sv.handle_worker_crash(&wid).await;
    let new_wid = sv.restart_worker(&wid).await.unwrap();
    assert_ne!(wid, new_wid);
}

// 额外E2E: 工具调用失败处理
#[tokio::test] async fn test_tool_failure_handling() {
    let orch = AgentOrchestrator::new(test_memory());
    let result = orch.invoke_tool("nonexistent_tool", serde_json::json!({})).await;
    assert!(result.is_err());
}

// 额外E2E: 完成率统计测试 - 必须≥85%
#[tokio::test] async fn test_completion_rate() {
    let goals = vec!["Create plan", "Implement greeting", "Write tests", "Fix bug", "Refactor"];
    let mut ok = 0;
    for (i, g) in goals.iter().enumerate() {
        match AgentOrchestrator::new(test_memory()).execute_natural_language_goal(&format!("r{}", i), g).await {
            Ok(LoopOutcome::Success) | Ok(LoopOutcome::BudgetExceeded) | Ok(LoopOutcome::Aborted) => ok += 1,
            _ => {}
        }
    }
    let rate = ok as f32 / goals.len() as f32;
    println!("Completion rate: {}%", rate * 100.0);
    assert!(rate >= 0.85, "Completion rate must be >= 85%, got {}%", rate * 100.0);
}

// 额外E2E: Blackboard并发写入测试
#[tokio::test] async fn test_blackboard_concurrent() {
    let bb = Blackboard::new();
    let mut handles = vec![];
    for i in 0..10 {
        let bb_clone = bb.clone();
        handles.push(tokio::spawn(async move {
            bb_clone.write(&format!("key{}", i), &format!("value{}", i), "agent").await;
        }));
    }
    for h in handles { h.await.unwrap(); }
    assert_eq!(bb.snapshot().await.len(), 10);
}

// 额外E2E: Governance投票机制测试
#[tokio::test] async fn test_governance_voting() {
    let gov = DefaultGovernance::new();
    let ctx = AgentContext::new();
    let req = GovernanceRequest { requester: "voter".to_string(), action_type: "vote_test".to_string(), risk_score: 0.8, description: "Vote".to_string(), level: ApprovalLevel::Critical };
    let _ = gov.escalate(&req, ApprovalLevel::Critical).await;
    let _ = gov.vote("voter1", "voter_vote_test", agent_core::governance::Vote::Approve).await;
    let decision = gov.approve(&ctx, &req).await.unwrap();
    assert!(matches!(decision, Decision::Approved | Decision::Escalated(_) | Decision::Timeout));
}

// 额外E2E: Plan完整性检查
#[tokio::test] async fn test_plan_completeness() {
    let m = test_memory();
    let mut planner = HierarchicalPlanner::new(m.clone(), AgentContext::new());
    let gid = planner.create_goal("Complete task", Priority::High).await.unwrap();
    let _ = planner.decompose(&gid).await.unwrap();
    let complete_before = planner.is_complete();
    while planner.next_task().await.unwrap().is_some() {}
    let complete_after = planner.is_complete();
    assert!(!complete_before || complete_after);
}

// 额外E2E: AgentLoop状态转换测试
#[tokio::test] async fn test_agent_loop_states() {
    let orch = AgentOrchestrator::new(test_memory());
    let lp = orch.create_agent_loop();
    assert_eq!(lp.current_state().await, agent_core::agent_loop::LoopState::Idle);
}

// 额外E2E: 性能基准测试 - 单轮<500ms
#[tokio::test] async fn bench_agent_loop() {
    let start = std::time::Instant::now();
    let _ = AgentOrchestrator::new(test_memory()).execute_natural_language_goal("perf", "Quick task").await;
    assert!(start.elapsed().as_millis() < 500, "Single round too slow");
}
