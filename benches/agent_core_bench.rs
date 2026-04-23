//! Agent Core Performance Benchmarks (B-03/10)
//! cargo bench --bench agent_core_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;
use memory::memory_gateway::MemoryGateway;

fn create_test_memory() -> Arc<Mutex<MemoryGateway>> {
    Arc::new(Mutex::new(MemoryGateway::new("bench")))
}

fn bench_agent_loop_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("agent_loop_creation", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::AgentOrchestrator;
            let _orch = AgentOrchestrator::new(create_test_memory());
            black_box(_orch);
        });
    });
}

fn bench_goal_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("goal_execution");
    group.sample_size(10);
    
    group.bench_function("simple_goal", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::AgentOrchestrator;
            let orch = AgentOrchestrator::new(create_test_memory());
            let _ = orch.execute_natural_language_goal(black_box("bench_agent"), black_box("Simple task")).await;
        });
    });
    
    group.bench_function("complex_goal", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::AgentOrchestrator;
            let orch = AgentOrchestrator::new(create_test_memory());
            let _ = orch.execute_natural_language_goal(
                black_box("bench_agent"), 
                black_box("Implement a feature with design and tests")
            ).await;
        });
    });
    
    group.finish();
}

fn bench_governance_approval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("governance_approval", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::{DefaultGovernance, AgentContext, GovernanceRequest, ApprovalLevel};
            let gov = DefaultGovernance::new();
            let ctx = AgentContext::new();
            let req = GovernanceRequest {
                requester: "bench".to_string(),
                action_type: "test".to_string(),
                risk_score: 0.5,
                description: "Benchmark".to_string(),
                level: ApprovalLevel::Auto,
            };
            let _ = gov.approve(&ctx, &req).await;
        });
    });
}

fn bench_checkpoint_save_restore(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("checkpoint");
    
    group.bench_function("save", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::{CheckpointManager, Blackboard};
            let bb = Blackboard::new();
            let mgr = CheckpointManager::new();
            let _ = mgr.save(&"bench".to_string(), None, vec![], vec![], &bb).await;
        });
    });
    
    group.bench_function("restore", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::{CheckpointManager, Blackboard};
            let bb = Blackboard::new();
            let mgr = CheckpointManager::new();
            let _ = mgr.save(&"bench".to_string(), None, vec![], vec![], &bb).await;
            let _ = mgr.restore_latest(&"bench".to_string()).await;
        });
    });
    
    group.finish();
}

fn bench_blackboard_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("blackboard");
    
    group.bench_function("write", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::Blackboard;
            let bb = Blackboard::new();
            bb.write(black_box("key"), black_box("value"), black_box("agent")).await;
        });
    });
    
    group.bench_function("read", |b| {
        b.to_async(&rt).iter(|| async {
            use agent_core::Blackboard;
            let bb = Blackboard::new();
            bb.write("key", "value", "agent").await;
            let _ = bb.read(black_box("key")).await;
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_agent_loop_creation,
    bench_goal_execution,
    bench_governance_approval,
    bench_checkpoint_save_restore,
    bench_blackboard_operations
);
criterion_main!(benches);
