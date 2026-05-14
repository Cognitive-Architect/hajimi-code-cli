use crate::auto::{AutoError, AutoMemory};
use chrono::TimeZone;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};

/// Scheduler错误类型
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("任务执行失败: {0}")]
    TaskFailed(String),
    #[error("Persist错误: {0}")]
    Persist(#[from] AutoError),
}

/// 内存调度器 - Dream层夜间Cron + Auto层自动persist
pub struct MemoryScheduler {
    _dream_cron: String,
    auto_persist_interval: Duration,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl MemoryScheduler {
    pub fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        Self {
            _dream_cron: "0 0 2 * * *".to_string(),
            auto_persist_interval: Duration::from_secs(300),
            shutdown_tx,
            handles: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn seconds_until_2am() -> u64 {
        let now = chrono::Local::now();
        let target = now
            .date_naive()
            .and_hms_opt(2, 0, 0)
            .unwrap_or(now.naive_local());
        let target = chrono::Local
            .from_local_datetime(&target)
            .single()
            .unwrap_or(now);
        if target > now {
            target.signed_duration_since(now).num_seconds() as u64
        } else {
            let next_target = target + chrono::Duration::days(1);
            next_target.signed_duration_since(now).num_seconds() as u64
        }
    }

    /// 启动Dream层夜间维护任务（每天02:00）
    pub fn spawn_dream_maintenance(&self) -> Result<(), SchedulerError> {
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let handles = self.handles.clone();
        let handle = tokio::spawn(async move {
            loop {
                let wait_secs = Self::seconds_until_2am();
                tokio::select! {
                    _ = sleep(Duration::from_secs(wait_secs)) => {
                        let _ = tokio::fs::create_dir_all(".hajimi/dream").await;
                        sleep(Duration::from_secs(3600)).await;
                    }
                    _ = shutdown_rx.changed() => { if *shutdown_rx.borrow() { break; } }
                }
            }
        });
        tokio::spawn(async move {
            handles.write().await.push(handle);
        });
        Ok(())
    }

    /// 启动Auto层定时persist任务（每5分钟检查dirty）
    pub fn spawn_auto_persist(&self, auto: Arc<Mutex<AutoMemory>>) -> Result<(), SchedulerError> {
        let interval_secs = self.auto_persist_interval;
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let handles = self.handles.clone();
        let handle = tokio::spawn(async move {
            let mut tick = interval(interval_secs);
            tick.tick().await;
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        let mut memory = auto.lock().await;
                        if memory.is_dirty() { let _ = memory.persist(); }
                    }
                    _ = shutdown_rx.changed() => { if *shutdown_rx.borrow() { break; } }
                }
            }
        });
        tokio::spawn(async move {
            handles.write().await.push(handle);
        });
        Ok(())
    }

    /// 优雅关闭所有任务
    pub async fn shutdown(&self) -> Result<(), SchedulerError> {
        let _ = self.shutdown_tx.send(true);
        let mut handles = self.handles.write().await;
        while let Some(h) = handles.pop() {
            let _ = h.await;
        }
        Ok(())
    }
}

impl Default for MemoryScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_new() {
        let scheduler = MemoryScheduler::new();
        assert_eq!(scheduler.auto_persist_interval, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_scheduler_spawn_auto_persist() {
        let scheduler = MemoryScheduler::new();
        let auto = Arc::new(Mutex::new(AutoMemory::new("test_sched").expect("fail")));
        assert!(scheduler.spawn_auto_persist(auto).is_ok());
        sleep(Duration::from_millis(50)).await;
        let _ = scheduler.shutdown().await;
    }

    #[tokio::test]
    async fn test_scheduler_shutdown() {
        let scheduler = MemoryScheduler::new();
        let auto = Arc::new(Mutex::new(AutoMemory::new("test_shutdown").expect("fail")));
        let _ = scheduler.spawn_auto_persist(auto);
        sleep(Duration::from_millis(50)).await;
        assert!(scheduler.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_scheduler_panic_recovery() {
        let scheduler = MemoryScheduler::new();
        let auto = Arc::new(Mutex::new(AutoMemory::new("test_panic").expect("fail")));
        let _ = scheduler.spawn_auto_persist(auto.clone());
        sleep(Duration::from_millis(30)).await;
        let _ = scheduler.shutdown().await;
        assert!(scheduler.spawn_auto_persist(auto).is_ok());
        let _ = scheduler.shutdown().await;
    }
}
