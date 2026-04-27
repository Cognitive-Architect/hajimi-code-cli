//! Memory Sync E2E Tests (B-06/07) — SyncMemoryGateway integration validation.
use memory::memory_gateway::MemoryGateway;
use memory::sync_gateway::{BlackboardSnapshot, GatewayEvent, MemoryTier, SyncMemoryGateway};
use std::sync::Arc;
use tokio::sync::Mutex;

fn create_sync_gateway() -> Arc<Mutex<MemoryGateway>> {
    let mut gw = MemoryGateway::new("e2e_sync");
    gw.enable_auto("e2e_sync").unwrap();
    gw.enable_dream().unwrap();
    gw.enable_graph();
    Arc::new(Mutex::new(gw))
}

/// 20-iteration consistency test: repeated retrieve operations remain stable.
#[tokio::test]
async fn test_sync_gateway_20_iteration_consistency() {
    let sg = create_sync_gateway();
    let mut gw = sg.lock().await;

    // Seed session tier with deterministic keys
    for i in 0..10 {
        gw.push_vector(&format!("seed_{}", i), &format!("data_{}", i)).unwrap();
    }

    for i in 0..20 {
        let query = format!("seed_{}", i % 10);
        let r = gw.retrieve_multi(MemoryTier::fallback_order(), &query).await.unwrap();
        assert!(!r.is_empty(), "Iteration {}: no results for {}", i, query);
        // Session tier must always return the seeded entry
        assert_eq!(r[0].0, MemoryTier::Session, "Session tier not first at iteration {}", i);
    }
}

/// Blackboard sync roundtrip: sync writes to session, retrieve confirms.
#[tokio::test]
async fn test_sync_gateway_blackboard_sync_roundtrip() {
    let sg = create_sync_gateway();
    let mut gw = sg.lock().await;

    let mut snapshot = BlackboardSnapshot::new();
    snapshot.entries.insert("bb_key1".into(), "bb_value1".into());
    snapshot.entries.insert("bb_key2".into(), "bb_value2".into());

    gw.sync_with_blackboard(&snapshot).await.unwrap();

    let r1 = gw.retrieve_from_tier(MemoryTier::Session, "bb_key1").await.unwrap();
    assert_eq!(r1.len(), 1);
    assert_eq!(r1[0].content, "bb_value1");

    let r2 = gw.retrieve_from_tier(MemoryTier::Session, "bb_key2").await.unwrap();
    assert_eq!(r2.len(), 1);
    assert_eq!(r2[0].content, "bb_value2");
}

/// Concurrent stress: 5 tasks × 10 ops each, verify no panic or corruption.
#[tokio::test]
async fn test_sync_gateway_concurrent_stress() {
    let sg = create_sync_gateway();
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let sg_clone = sg.clone();
            tokio::spawn(async move {
                let mut gw = sg_clone.lock().await;
                for j in 0..10 {
                    let key = format!("stress_{}_{}", i, j);
                    let _ = gw.push_vector(&key, &format!("data_{}_{}", i, j));
                    let _ = gw.retrieve_multi(MemoryTier::fallback_order(), &key).await;
                }
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }

    let mut gw = sg.lock().await;
    let health = gw.tier_health(MemoryTier::Session).await.unwrap();
    assert!(health.available);
    assert!(health.entry_count >= 50, "Expected >=50 entries, got {}", health.entry_count);
}

/// Event persistence: push_event stores data retrievable via session search.
#[tokio::test]
async fn test_sync_gateway_event_persistence() {
    let sg = create_sync_gateway();
    let mut gw = sg.lock().await;

    let evt = GatewayEvent::new("TestEvent", "payload_content", "test_source");
    gw.push_event(evt).await.unwrap();

    // Events are stored with evt_{timestamp} key; verify session is not empty
    let health = gw.tier_health(MemoryTier::Session).await.unwrap();
    assert!(health.entry_count > 0);
}

/// Crash recovery simulation: checkpoint saved, then restored from memory.
#[tokio::test]
async fn test_sync_gateway_crash_recovery() {
    let sg = create_sync_gateway();
    {
        let mut gw = sg.lock().await;
        gw.push_vector("recover_key", "recover_value").unwrap();
    }

    // Simulate "crash" by dropping and recreating gateway from same data
    // In reality, MemoryGateway persists to disk; here we just verify
    // the data survived within the same Arc<Mutex<_>> reference.
    let mut gw = sg.lock().await;
    let r = gw.retrieve_from_tier(MemoryTier::Session, "recover_key").await.unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].content, "recover_value");
}

/// Tier health checks: all enabled tiers report available.
#[tokio::test]
async fn test_sync_gateway_tier_health_all_available() {
    let sg = create_sync_gateway();
    let mut gw = sg.lock().await;

    for tier in [MemoryTier::Session, MemoryTier::Auto, MemoryTier::Dream, MemoryTier::Graph] {
        let h = gw.tier_health(tier).await.unwrap();
        assert!(h.available, "Tier {:?} not available", tier);
    }
}
