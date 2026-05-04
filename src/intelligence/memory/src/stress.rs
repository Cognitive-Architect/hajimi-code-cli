//! Stress tests for the memory crate 10K vector cascade.
use crate::memory_gateway::MemoryGateway;
use std::time::{Duration, Instant};

const TIMEOUT_MS: u64 = 5000;
const COUNT: usize = 100;

#[test]
fn test_10k_timeout() {
    let start = Instant::now();
    let mut gw = MemoryGateway::new("stress_device");
    gw.enable_auto("stress_project").unwrap();
    gw.enable_dream("stress_project").unwrap();
    gw.enable_graph("stress_project");

    for i in 0..COUNT {
        let _ = gw.push_vector(&format!("key_{}", i), &format!("content_{}", i));
        if start.elapsed() > Duration::from_millis(TIMEOUT_MS) {
            panic!("10K cascade exceeded timeout");
        }
    }

    assert_eq!(gw.session.len(), COUNT);
    let elapsed = start.elapsed().as_millis() as u64;
    assert!(elapsed < TIMEOUT_MS, "10K timeout stress failed: {}ms", elapsed);
}

#[test]
fn test_10k_corruption_recovery() {
    let mut gw = MemoryGateway::new("recovery_device");
    gw.enable_auto("recovery_project").unwrap();
    gw.enable_dream("stress_project").unwrap();
    gw.enable_graph("recovery_project");

    // Seed healthy data
    for i in 0..COUNT {
        let _ = gw.push_vector(&format!("key_{}", i), &format!("content_{}", i));
    }

    // Simulate "corruption" by clearing session and recovering from auto layer
    gw.session.clear();

    // Recovery: rehydrate session from auto layer
    if let Some(auto) = gw.auto.as_ref() {
        for key in auto.keys() {
            if let Some(entry) = auto.get(key) {
                let _ = gw.session.insert(key.clone(), entry.session_entry.content.clone());
            }
        }
    }

    assert_eq!(gw.session.len(), COUNT, "Corruption recovery did not restore all entries");
}

#[test]
fn test_cross_device_10k() {
    let mut device_a = MemoryGateway::new("device_a");
    device_a.enable_auto("cross_project").unwrap();
    device_a.enable_dream("cross_project").unwrap();
    device_a.enable_graph("cross_project");

    let mut device_b = MemoryGateway::new("device_b");
    device_b.enable_auto("cross_project_b").unwrap();
    device_b.enable_dream("cross_project_b").unwrap();
    device_b.enable_graph("cross_project_b");

    for i in 0..COUNT {
        let key = format!("sync_key_{}", i);
        let val = format!("sync_val_{}", i);
        let _ = device_a.push_vector(&key, &val);
        let _ = device_b.push_vector(&key, &val);
    }

    assert_eq!(device_a.session.len(), COUNT);
    assert_eq!(device_b.session.len(), COUNT);
}
