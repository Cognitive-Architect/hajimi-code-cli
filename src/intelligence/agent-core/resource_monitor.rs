//! DEBT-LINES-B0305A: Resource monitoring with atomic counters and alert thresholds.
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const DEFAULT_MEMORY_THRESHOLD_ENTRIES: usize = 10000;
const DEFAULT_FAILURE_RATE_THRESHOLD: u64 = 5000; // 50.00%
const ALERT_COOLDOWN_SECS: u64 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub iteration_count: u64,
    pub blackboard_size: usize,
    pub failure_rate_percent: f32,
    pub callback_latency_ms: u64,
    pub edit_count: u64,
    pub undo_stack_size: u64,
    pub checkpoint_count: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub kind: String,
    pub message: String,
    pub current: u64,
    pub threshold: u64,
}

pub struct ResourceMonitor {
    iteration_count: AtomicU64,
    blackboard_size: AtomicU64,
    failure_count: AtomicU64,
    total_actions: AtomicU64,
    callback_latency_ms: AtomicU64,
    edit_count: AtomicU64,
    undo_stack_size: AtomicU64,
    checkpoint_count: AtomicU64,
    last_alert: Arc<Mutex<Instant>>,
    memory_threshold: usize,
    failure_threshold: u64,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            iteration_count: AtomicU64::new(0), blackboard_size: AtomicU64::new(0),
            failure_count: AtomicU64::new(0), total_actions: AtomicU64::new(0),
            callback_latency_ms: AtomicU64::new(0),
            edit_count: AtomicU64::new(0), undo_stack_size: AtomicU64::new(0),
            checkpoint_count: AtomicU64::new(0),
            last_alert: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(ALERT_COOLDOWN_SECS + 1))),
            memory_threshold: DEFAULT_MEMORY_THRESHOLD_ENTRIES,
            failure_threshold: DEFAULT_FAILURE_RATE_THRESHOLD,
        }
    }

    pub fn record_iteration(&self) { self.iteration_count.fetch_add(1, Ordering::Relaxed); }
    pub fn record_blackboard_size(&self, size: usize) { self.blackboard_size.store(size as u64, Ordering::Relaxed); }
    pub fn record_failure(&self) { self.failure_count.fetch_add(1, Ordering::Relaxed); self.total_actions.fetch_add(1, Ordering::Relaxed); }
    pub fn record_success(&self) { self.total_actions.fetch_add(1, Ordering::Relaxed); }
    pub fn record_callback_latency(&self, ms: u64) { self.callback_latency_ms.store(ms, Ordering::Relaxed); }
    pub fn record_edit(&self) { self.edit_count.fetch_add(1, Ordering::Relaxed); }
    pub fn record_undo_stack_size(&self, size: usize) { self.undo_stack_size.store(size as u64, Ordering::Relaxed); }
    pub fn record_checkpoint_count(&self, count: usize) { self.checkpoint_count.store(count as u64, Ordering::Relaxed); }

    pub fn get_metrics(&self) -> ResourceMetrics {
        let total = self.total_actions.load(Ordering::Relaxed).max(1);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let rate = (failures as f32 / total as f32) * 100.0;
        ResourceMetrics {
            iteration_count: self.iteration_count.load(Ordering::Relaxed),
            blackboard_size: self.blackboard_size.load(Ordering::Relaxed) as usize,
            failure_rate_percent: rate,
            callback_latency_ms: self.callback_latency_ms.load(Ordering::Relaxed),
            edit_count: self.edit_count.load(Ordering::Relaxed),
            undo_stack_size: self.undo_stack_size.load(Ordering::Relaxed),
            checkpoint_count: self.checkpoint_count.load(Ordering::Relaxed),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub async fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let mut last = self.last_alert.lock().await;
        if last.elapsed() < Duration::from_secs(ALERT_COOLDOWN_SECS) { return alerts; }
        let bb_size = self.blackboard_size.load(Ordering::Relaxed) as usize;
        if bb_size > self.memory_threshold {
            alerts.push(Alert { kind: "memory".to_string(), message: format!("Blackboard size {} exceeds threshold {}", bb_size, self.memory_threshold), current: bb_size as u64, threshold: self.memory_threshold as u64 });
        }
        let total = self.total_actions.load(Ordering::Relaxed).max(1);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let rate_x100 = (failures * 10000) / total;
        if rate_x100 > self.failure_threshold {
            alerts.push(Alert { kind: "failure_rate".to_string(), message: format!("Failure rate {:.2}% exceeds threshold {:.2}%", rate_x100 as f32 / 100.0, self.failure_threshold as f32 / 100.0), current: rate_x100, threshold: self.failure_threshold });
        }
        let undo_size = self.undo_stack_size.load(Ordering::Relaxed);
        if undo_size > 100 {
            alerts.push(Alert { kind: "undo_stack".to_string(), message: format!("Undo stack size {} exceeds threshold 100", undo_size), current: undo_size, threshold: 100 });
        }
        let chk_count = self.checkpoint_count.load(Ordering::Relaxed);
        if chk_count > 150 {
            alerts.push(Alert { kind: "checkpoint_count".to_string(), message: format!("Checkpoint count {} exceeds threshold 150", chk_count), current: chk_count, threshold: 150 });
        }
        if !alerts.is_empty() { *last = Instant::now(); }
        alerts
    }
}

impl Default for ResourceMonitor { fn default() -> Self { Self::new() } }
