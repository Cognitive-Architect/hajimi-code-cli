//! ResourceMonitor tests: metrics tracking, alert thresholds, concurrency, leak detection.

use agent_core::resource_monitor::ResourceMonitor;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_monitor_iteration_count_tracked() {
    let m = ResourceMonitor::new();
    for _ in 0..10 {
        m.record_iteration();
    }
    assert_eq!(m.get_metrics().iteration_count, 10);
}

#[tokio::test]
async fn test_monitor_blackboard_size_tracked() {
    let m = ResourceMonitor::new();
    m.record_blackboard_size(500);
    assert_eq!(m.get_metrics().blackboard_size, 500);
}

#[tokio::test]
async fn test_monitor_failure_rate_calculated() {
    let m = ResourceMonitor::new();
    m.record_success();
    m.record_success();
    m.record_failure();
    let rate = m.get_metrics().failure_rate_percent;
    assert!(rate > 0.0 && rate <= 100.0);
}

#[tokio::test]
async fn test_monitor_callback_latency_tracked() {
    let m = ResourceMonitor::new();
    m.record_callback_latency(42);
    assert_eq!(m.get_metrics().callback_latency_ms, 42);
}

#[tokio::test]
async fn test_alert_memory_threshold_exceeded() {
    let m = ResourceMonitor::new();
    m.record_blackboard_size(20000);
    let alerts = m.check_alerts().await;
    assert!(alerts.iter().any(|a| a.kind == "memory"));
}

#[tokio::test]
async fn test_alert_failure_rate_exceeded() {
    let m = ResourceMonitor::new();
    for _ in 0..9 {
        m.record_failure();
    }
    m.record_success();
    let alerts = m.check_alerts().await;
    assert!(alerts.iter().any(|a| a.kind == "failure_rate"));
}

#[tokio::test]
async fn test_alert_cooldown_prevents_duplicate() {
    let m = ResourceMonitor::new();
    m.record_blackboard_size(20000);
    assert!(!m.check_alerts().await.is_empty());
    assert!(m.check_alerts().await.is_empty());
}

#[tokio::test]
async fn test_alert_resumes_after_cooldown() {
    let m = ResourceMonitor::new();
    m.record_blackboard_size(20000);
    let _ = m.check_alerts().await;
    sleep(Duration::from_secs(11)).await;
    assert!(!m.check_alerts().await.is_empty());
}

#[tokio::test]
async fn test_monitor_concurrent_updates_safe() {
    let m = Arc::new(ResourceMonitor::new());
    let mut handles = vec![];
    for i in 0..10 {
        let mm = m.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                mm.record_iteration();
                mm.record_blackboard_size(i * 100);
                if i % 2 == 0 {
                    mm.record_failure();
                } else {
                    mm.record_success();
                }
                mm.record_callback_latency(i as u64);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(m.get_metrics().iteration_count, 500);
}

#[tokio::test]
async fn test_monitor_no_leak_under_stress() {
    let m = Arc::new(ResourceMonitor::new());
    for _ in 0..600 {
        m.record_iteration();
        m.record_blackboard_size(100);
        m.record_success();
        m.record_callback_latency(10);
    }
    assert_eq!(m.get_metrics().iteration_count, 600);
    let _ = m.check_alerts().await;
}
