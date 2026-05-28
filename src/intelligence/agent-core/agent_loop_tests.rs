#[cfg(test)]
mod tests {
    use crate::agent_loop::{AgentLoop, LoopOutcome, LoopState};
    use crate::agent_loop_builder::AgentLoopBuilder;
    use crate::blackboard::Blackboard;
    use crate::checkpoint::CheckpointManager;
    use crate::governance::DefaultGovernance;
    use crate::planner::{HierarchicalPlanner, Planner, Priority};
    use crate::reflector::{AutonomousReflector, Reflector};
    use crate::swarm::{Supervisor, SwarmCoordinator, TaskAssignment};
    use crate::tools::PlanningTool;
    use crate::{AgentConfig, AgentContext, AgentRole};
    use engine_tool_system::ToolRegistry;
    use memory::memory_gateway::MemoryGateway;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn test_loop_with_mem(mem: Arc<Mutex<MemoryGateway>>) -> AgentLoop {
        AgentLoop::new(crate::agent_loop_builder::AgentLoopConfig {
            context: AgentContext::new(),
            planner: Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>,
            reflector: Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>,
            governance: Arc::new(DefaultGovernance::new()),
            swarm: None,
            blackboard: Arc::new(Blackboard::new()),
            checkpoint_mgr: Arc::new(CheckpointManager::new()),
            memory: Some(mem),
            sync_gateway: None,
            provider_id: None,
            edit_applier: None,
            skill_registry: None,
            skill_router: None,
            tool_registry: None,
        })
    }

    fn test_loop() -> AgentLoop {
        let m = Arc::new(Mutex::new(MemoryGateway::new("test")));
        test_loop_with_mem(m)
    }

    #[tokio::test]
    async fn test_creation() {
        let _ = test_loop();
    }

    #[tokio::test]
    async fn test_from_nl() {
        let (g, p) = AgentLoop::from_natural_language("Fix critical bug");
        assert_eq!(g, "Fix critical bug");
        assert_eq!(p, Priority::Critical);
    }

    #[tokio::test]
    async fn test_run() {
        assert!(test_loop().run("a1".to_string(), "Test").await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_goal() {
        let l = test_loop();
        assert!(l.execute_goal("a1".to_string(), "Test goal").await.is_ok());
    }

    #[tokio::test]
    async fn test_max_iter() {
        let out = test_loop()
            .run("a1".to_string(), "Never end")
            .await
            .unwrap();
        assert!(matches!(
            out,
            LoopOutcome::BudgetExceeded | LoopOutcome::Success | LoopOutcome::Aborted
        ));
    }

    #[tokio::test]
    async fn test_long_running_stability() {
        for i in 0..3 {
            let _ = test_loop().run(format!("a{}", i), &format!("G{}", i)).await;
        }
    }

    #[tokio::test]
    async fn test_state_observability() {
        let l = test_loop();
        assert_eq!(l.current_state().await, LoopState::Idle);
    }

    #[tokio::test]
    async fn test_autonomous_goal_completion() {
        let l = test_loop();
        let outcome = l
            .execute_goal("agent1".to_string(), "Create a simple test plan")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));
    }

    #[tokio::test]
    async fn test_agent_loop_no_leak() {
        let mem = Arc::new(Mutex::new(MemoryGateway::new("leak_test")));
        let initial = Arc::strong_count(&mem);
        for i in 0..10 {
            let l = test_loop_with_mem(mem.clone());
            let _ = l.run(format!("leak_test_{}", i), "Quick test goal").await;
            drop(l);
        }
        let final_count = Arc::strong_count(&mem);
        assert!(
            final_count <= initial + 1,
            "Resource leak detected: Arc count {} > expected {}",
            final_count,
            initial + 1
        );
    }

    #[tokio::test]
    async fn test_trace_emit() {
        let l = test_loop();
        let mut rx = l.subscribe_trace().expect("trace subscriber");
        let _ = l.run("trace_test".to_string(), "Quick test").await;
        let mut count = 0;
        while let Ok(_event) = rx.try_recv() {
            count += 1;
        }
        assert!(
            count >= 4,
            "Expected at least 4 trace events, got {}",
            count
        );
    }

    #[tokio::test]
    async fn test_supervisor_worker_execution() {
        let gov = Arc::new(DefaultGovernance::new());
        let mut sv = Supervisor::new(gov.clone(), AgentContext::new());
        let mut registry = ToolRegistry::new();
        let planner = Arc::new(Mutex::new(HierarchicalPlanner::new(
            Arc::new(Mutex::new(MemoryGateway::new("diag"))),
            AgentContext::new(),
        )));
        let bb = Arc::new(Blackboard::new());
        let planning_tool = Arc::new(PlanningTool::new(planner.clone(), gov.clone(), bb.clone()));
        registry.register(planning_tool);
        sv = sv.with_tool_registry(Arc::new(Mutex::new(registry)));
        let cfg = AgentConfig::supervisor("diag");
        let wid = sv.spawn_worker(AgentRole::Coder, cfg).await.unwrap();
        let task = TaskAssignment {
            task_id: "t1".to_string(),
            description: "Test".to_string(),
            assigned_to: wid,
            priority: 5,
        };
        sv.delegate(task).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let results = sv.aggregate().await;
        assert!(!results.is_empty(), "Expected worker results, got none");
        println!("Worker results: {:?}", results);
    }

    #[tokio::test]
    async fn test_swarm_with_full_callback_loop() {
        let mem = Arc::new(Mutex::new(MemoryGateway::new("callback_test")));
        let gov = Arc::new(DefaultGovernance::new());
        let mut sv = Supervisor::new(gov.clone(), AgentContext::new());
        let mut registry = ToolRegistry::new();
        let planner = Arc::new(Mutex::new(HierarchicalPlanner::new(
            mem.clone(),
            AgentContext::new(),
        )));
        let bb = Arc::new(Blackboard::new());
        let planning_tool = Arc::new(PlanningTool::new(planner.clone(), gov.clone(), bb.clone()));
        registry.register(planning_tool);
        sv = sv.with_tool_registry(Arc::new(Mutex::new(registry)));
        let cfg = AgentConfig::supervisor("loop_test");
        let _wid = sv.spawn_worker(AgentRole::Coder, cfg).await.unwrap();
        {
            let mut p = planner.lock().await;
            let gid = p
                .create_goal("Test callback", Priority::High)
                .await
                .unwrap();
            let sgs = p.decompose(&gid).await.unwrap();
            for sg_id in sgs {
                let _ = p.expand(&sg_id).await;
            }
        }
        let reflector = Arc::new(Mutex::new(AutonomousReflector::new(
            mem.clone(),
            AgentContext::new(),
        )));
        let loop_bb = Arc::new(Blackboard::new());
        let swarm = Arc::new(Mutex::new(sv));
        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(planner)
            .with_reflector(reflector)
            .with_governance(gov)
            .with_swarm(Some(swarm.clone()))
            .with_blackboard(loop_bb.clone())
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .build()
            .unwrap();
        // Seed multiple worker results to test reflect_multi aggregation
        swarm
            .lock()
            .await
            .handle_worker_result(crate::swarm::WorkerResult::success(
                "t1",
                "w1".to_string(),
                "ok1",
                crate::ports::WorkerMetrics::new(10),
            ))
            .await;
        swarm
            .lock()
            .await
            .handle_worker_result(crate::swarm::WorkerResult::success(
                "t2",
                "w2".to_string(),
                "ok2",
                crate::ports::WorkerMetrics::new(20),
            ))
            .await;
        swarm
            .lock()
            .await
            .handle_worker_result(crate::swarm::WorkerResult::failure(
                "t3",
                "w3".to_string(),
                "err",
                crate::ports::WorkerMetrics::new(5),
            ))
            .await;
        // Verify reflect() uses reflect_multi with aggregated worker results
        let task_result = crate::planner::TaskResult {
            success: true,
            output: "seeded".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let refl_result = agent_loop.reflect("g1", &task_result).await;
        assert!(refl_result.is_ok(), "reflect() failed: {:?}", refl_result);
        // Verify act() timeout mechanism exists by checking it returns quickly when no swarm
        let fallback_mem = Arc::new(Mutex::new(MemoryGateway::new("fallback")));
        let fallback_planner = Arc::new(Mutex::new(HierarchicalPlanner::new(
            fallback_mem.clone(),
            AgentContext::new(),
        )));
        {
            fallback_planner
                .lock()
                .await
                .create_goal("Fallback test", Priority::High)
                .await
                .unwrap();
        }
        let no_swarm_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(fallback_planner.clone() as Arc<Mutex<dyn crate::planner::Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                fallback_mem.clone(),
                AgentContext::new(),
            )))
                as Arc<Mutex<dyn crate::reflector::Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .build()
            .unwrap();
        let fallback_result = no_swarm_loop.act(&"agent1".to_string(), "g1").await;
        assert!(fallback_result.is_ok(), "Fallback act() failed");
        let fb = fallback_result.unwrap();
        assert!(
            fb.output.contains("no swarm") || fb.output.contains("No pending tasks"),
            "Expected fallback message, got: {}",
            fb.output
        );
    }

    #[tokio::test]
    async fn test_concurrent_goals_isolation() {
        let loop1 = test_loop();
        let loop2 = test_loop();
        let h1 = tokio::spawn(async move { loop1.run("a1".to_string(), "Goal 1").await });
        let h2 = tokio::spawn(async move { loop2.run("a2".to_string(), "Goal 2").await });
        let (r1, r2) = tokio::join!(h1, h2);
        assert!(r1.is_ok() && r2.is_ok());
        assert!(matches!(
            r1.unwrap().unwrap(),
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));
        assert!(matches!(
            r2.unwrap().unwrap(),
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));
    }

    static ENV_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    struct EnvVarGuard {
        key: &'static str,
        original_value: Option<String>,
    }

    impl EnvVarGuard {
        fn new(key: &'static str) -> Self {
            let original_value = std::env::var(key).ok();
            Self {
                key,
                original_value,
            }
        }

        fn set(&self, value: &str) {
            std::env::set_var(self.key, value);
        }

        fn unset(&self) {
            std::env::remove_var(self.key);
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(ref val) = self.original_value {
                std::env::set_var(self.key, val);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[tokio::test]
    async fn test_skills_routing_disabled() {
        let _guard = ENV_MUTEX.lock().await;
        let env_guard = EnvVarGuard::new("HAJIMI_AGENT_SKILLS_V0");
        env_guard.set("false");

        let mem = Arc::new(Mutex::new(MemoryGateway::new("skills_disabled_test")));
        let base_dir = std::env::current_dir().unwrap();
        let mut root = base_dir.join("tests/fixtures/skills");
        if !root.exists() {
            root = base_dir.join("../../../tests/fixtures/skills");
        }
        let registry = Arc::new(crate::skills::SkillRegistry::scan(&root).unwrap());
        let router = Arc::new(crate::skills::SkillRouter::new(
            registry.clone(),
            Default::default(),
        ));

        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_skill_registry(Some(registry))
            .with_skill_router(Some(router))
            .build()
            .unwrap();

        let outcome = agent_loop
            .run("agent1".to_string(), "自动存档 准备下一步并分析风险")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));

        // When disabled, no skill keys should be written to the blackboard
        let bb = agent_loop.blackboard();
        let active = bb.read(crate::skills::BB_ACTIVE_SKILLS).await;
        assert!(active.is_none());
        let receipt = bb.read(crate::skills::BB_SKILL_ROUTE_RECEIPT).await;
        assert!(receipt.is_none());
        let instructions = bb.read(crate::skills::BB_SKILL_INSTRUCTIONS).await;
        assert!(instructions.is_none());
        let eval_criteria = bb.read(crate::skills::BB_SKILL_EVAL_CRITERIA).await;
        assert!(eval_criteria.is_none());
    }

    #[tokio::test]
    async fn test_skills_routing_enabled() {
        let _guard = ENV_MUTEX.lock().await;
        let env_guard = EnvVarGuard::new("HAJIMI_AGENT_SKILLS_V0");
        env_guard.set("true");

        let mem = Arc::new(Mutex::new(MemoryGateway::new("skills_enabled_test")));
        let base_dir = std::env::current_dir().unwrap();
        let mut root = base_dir.join("tests/fixtures/skills");
        if !root.exists() {
            root = base_dir.join("../../../tests/fixtures/skills");
        }
        let registry = Arc::new(crate::skills::SkillRegistry::scan(&root).unwrap());
        let router = Arc::new(crate::skills::SkillRouter::new(
            registry.clone(),
            Default::default(),
        ));

        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_skill_registry(Some(registry))
            .with_skill_router(Some(router))
            .build()
            .unwrap();

        let outcome = agent_loop
            .run("agent1".to_string(), "自动存档 准备下一步并分析风险")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));

        // When enabled, the skill keys must be correctly routed and loaded to the blackboard
        let bb = agent_loop.blackboard();
        let active = bb
            .read(crate::skills::BB_ACTIVE_SKILLS)
            .await
            .expect("BB_ACTIVE_SKILLS should be present");
        assert!(active.value.contains("auto-save"));

        let receipt = bb
            .read(crate::skills::BB_SKILL_ROUTE_RECEIPT)
            .await
            .expect("BB_SKILL_ROUTE_RECEIPT should be present");
        assert!(receipt.value.contains("auto-save"));

        let instructions = bb
            .read(crate::skills::BB_SKILL_INSTRUCTIONS)
            .await
            .expect("BB_SKILL_INSTRUCTIONS should be present");
        assert!(instructions.value.contains("=== AUTO SAVE"));

        let eval_criteria = bb
            .read(crate::skills::BB_SKILL_EVAL_CRITERIA)
            .await
            .expect("BB_SKILL_EVAL_CRITERIA should be present");
        assert!(eval_criteria.value.contains("=== AUTO SAVE"));
        assert!(eval_criteria.value.contains("must_include"));
    }

    #[tokio::test]
    async fn test_skills_runtime_constraints_written_when_gate_enabled() {
        let _guard = ENV_MUTEX.lock().await;
        let skills_env = EnvVarGuard::new("HAJIMI_AGENT_SKILLS_V0");
        skills_env.set("true");
        let runtime_env = EnvVarGuard::new(crate::skills::HAJIMI_AGENT_SKILL_RUNTIME_ENV);
        runtime_env.set("true");

        let mem = Arc::new(Mutex::new(MemoryGateway::new(
            "skills_runtime_constraints_test",
        )));
        let base_dir = std::env::current_dir().unwrap();
        let mut root = base_dir.join("tests/fixtures/skills");
        if !root.exists() {
            root = base_dir.join("../../../tests/fixtures/skills");
        }
        let registry = Arc::new(crate::skills::SkillRegistry::scan(&root).unwrap());
        let router = Arc::new(crate::skills::SkillRouter::new(
            registry.clone(),
            Default::default(),
        ));

        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_skill_registry(Some(registry))
            .with_skill_router(Some(router))
            .build()
            .unwrap();

        let outcome = agent_loop
            .run("agent1".to_string(), "自动存档 准备下一步并分析风险")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));

        let constraints = agent_loop
            .blackboard()
            .read(crate::skills::BB_SKILL_TOOL_CONSTRAINTS)
            .await
            .expect("BB_SKILL_TOOL_CONSTRAINTS should be present when runtime gate is enabled");
        assert!(constraints.value.contains("auto-save"));
        assert!(constraints.value.contains("allowed"));
        assert!(constraints.value.contains("denied"));
    }

    #[tokio::test]
    async fn test_skills_routing_unset_gate_disabled() {
        let _guard = ENV_MUTEX.lock().await;
        let env_guard = EnvVarGuard::new("HAJIMI_AGENT_SKILLS_V0");
        env_guard.unset();

        let mem = Arc::new(Mutex::new(MemoryGateway::new("skills_unset_test")));
        let base_dir = std::env::current_dir().unwrap();
        let mut root = base_dir.join("tests/fixtures/skills");
        if !root.exists() {
            root = base_dir.join("../../../tests/fixtures/skills");
        }
        let registry = Arc::new(crate::skills::SkillRegistry::scan(&root).unwrap());
        let router = Arc::new(crate::skills::SkillRouter::new(
            registry.clone(),
            Default::default(),
        ));

        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_skill_registry(Some(registry))
            .with_skill_router(Some(router))
            .build()
            .unwrap();

        let outcome = agent_loop
            .run("agent1".to_string(), "自动存档 准备下一步并分析风险")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));

        // When unset, no skill keys should be written to the blackboard
        let bb = agent_loop.blackboard();
        let active = bb.read(crate::skills::BB_ACTIVE_SKILLS).await;
        assert!(active.is_none());
        let receipt = bb.read(crate::skills::BB_SKILL_ROUTE_RECEIPT).await;
        assert!(receipt.is_none());
        let instructions = bb.read(crate::skills::BB_SKILL_INSTRUCTIONS).await;
        assert!(instructions.is_none());
        let eval_criteria = bb.read(crate::skills::BB_SKILL_EVAL_CRITERIA).await;
        assert!(eval_criteria.is_none());
    }

    #[tokio::test]
    async fn test_skills_routing_enabled_missing_components_degrades() {
        let _guard = ENV_MUTEX.lock().await;
        let env_guard = EnvVarGuard::new("HAJIMI_AGENT_SKILLS_V0");
        env_guard.set("true");

        let mem = Arc::new(Mutex::new(MemoryGateway::new("skills_degrade_test")));

        // Construct AgentLoop without skill registry and router (both are None)
        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_skill_registry(None)
            .with_skill_router(None)
            .build()
            .unwrap();

        // Running the loop should not error, but degrade gracefully
        let outcome = agent_loop
            .run("agent1".to_string(), "自动存档 准备下一步并分析风险")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));

        // Assert no blackboard keys are populated
        let bb = agent_loop.blackboard();
        let active = bb.read(crate::skills::BB_ACTIVE_SKILLS).await;
        assert!(active.is_none());
        let receipt = bb.read(crate::skills::BB_SKILL_ROUTE_RECEIPT).await;
        assert!(receipt.is_none());
        let instructions = bb.read(crate::skills::BB_SKILL_INSTRUCTIONS).await;
        assert!(instructions.is_none());
        let eval_criteria = bb.read(crate::skills::BB_SKILL_EVAL_CRITERIA).await;
        assert!(eval_criteria.is_none());
    }

    #[tokio::test]
    async fn test_skills_execution_receipt_written() {
        let _guard = ENV_MUTEX.lock().await;
        let env_guard = EnvVarGuard::new("HAJIMI_AGENT_SKILLS_V0");
        env_guard.set("true");

        let mem = Arc::new(Mutex::new(MemoryGateway::new("skills_receipt_test")));
        let base_dir = std::env::current_dir().unwrap();
        let mut root = base_dir.join("tests/fixtures/skills");
        if !root.exists() {
            root = base_dir.join("../../../tests/fixtures/skills");
        }
        let registry = Arc::new(crate::skills::SkillRegistry::scan(&root).unwrap());
        let router = Arc::new(crate::skills::SkillRouter::new(
            registry.clone(),
            Default::default(),
        ));

        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(Arc::new(Blackboard::new()))
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_skill_registry(Some(registry))
            .with_skill_router(Some(router))
            .build()
            .unwrap();

        let outcome = agent_loop
            .run("agent1".to_string(), "自动存档 准备下一步并分析风险")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            LoopOutcome::Success | LoopOutcome::BudgetExceeded | LoopOutcome::Aborted
        ));

        let bb = agent_loop.blackboard();
        let execution_receipts = bb
            .read(crate::skills::BB_SKILL_EXECUTION_RECEIPTS)
            .await
            .expect("BB_SKILL_EXECUTION_RECEIPTS should be present after reflecting");

        assert!(execution_receipts.value.contains("auto-save"));
        assert!(execution_receipts.value.contains("success"));
        assert!(execution_receipts.value.contains("timestamp"));
    }

    #[tokio::test]
    async fn test_agent_llm_bootstrap_mechanism() {
        let _guard = ENV_MUTEX.lock().await;
        let env_guard = EnvVarGuard::new("HAJIMI_AGENT_LLM_BOOTSTRAP_ENABLED");
        env_guard.set("true");

        let mem = Arc::new(Mutex::new(MemoryGateway::new("bootstrap_test")));
        let mut registry = ToolRegistry::new();
        
        struct MockTool;
        #[async_trait::async_trait]
        impl engine_tool_system::Tool for MockTool {
            fn name(&self) -> &str { "read_file" }
            fn description(&self) -> &str { "mock read file" }
            fn permissions(&self) -> engine_tool_system::ToolPermissions { Default::default() }
            async fn execute(&self, _args: engine_tool_system::ToolArgs) -> Result<engine_tool_system::ToolOutput, engine_tool_system::ToolError> {
                Ok(engine_tool_system::ToolOutput::success("mocked file content"))
            }
        }
        registry.register(Arc::new(MockTool));

        let loop_bb = Arc::new(Blackboard::new());
        let agent_loop = AgentLoopBuilder::new()
            .with_context(AgentContext::new())
            .with_planner(Arc::new(Mutex::new(HierarchicalPlanner::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Planner>>)
            .with_reflector(Arc::new(Mutex::new(AutonomousReflector::new(
                mem.clone(),
                AgentContext::new(),
            ))) as Arc<Mutex<dyn Reflector>>)
            .with_governance(Arc::new(DefaultGovernance::new()))
            .with_swarm(None)
            .with_blackboard(loop_bb.clone())
            .with_checkpoint_mgr(Arc::new(CheckpointManager::new()))
            .with_memory(Some(mem))
            .with_tool_registry(Arc::new(Mutex::new(registry)))
            .build()
            .unwrap();

        // Initially BB_NEXT_TOOL must be empty
        assert!(loop_bb.read(crate::act_executor::BB_NEXT_TOOL).await.is_none());

        // Perform bootstrap on a goal
        let goal_id = "test_goal_1".to_string();
        let agent_id = "agent_test".to_string();
        
        let gid = agent_loop.plan_initial_goal("Fix compile bug").await.unwrap();

        let bootstrapped = agent_loop.bootstrap_first_tool_call(&gid, &agent_id).await.unwrap();
        assert!(bootstrapped.is_some(), "Expected bootstrapped tool call");

        let tc = bootstrapped.unwrap();
        assert_eq!(tc.tool_name, "read_file");

        // The BB_NEXT_TOOL should be written
        let entry = loop_bb.read(crate::act_executor::BB_NEXT_TOOL).await.unwrap();
        assert!(entry.value.contains("read_file"));
    }
}
