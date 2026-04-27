    use super::*;
    use crate::governance::DefaultGovernance;
    use crate::ports::WorkerCallback;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn sv() -> Supervisor { Supervisor::new(Arc::new(DefaultGovernance::new()), crate::AgentContext::new()) }

    struct Cb { sc: AtomicUsize, fc: AtomicUsize, tc: AtomicUsize }
    impl Cb { fn new() -> Self { Self { sc: AtomicUsize::new(0), fc: AtomicUsize::new(0), tc: AtomicUsize::new(0) } } fn s(&self) -> usize { self.sc.load(Ordering::SeqCst) } fn f(&self) -> usize { self.fc.load(Ordering::SeqCst) } fn t(&self) -> usize { self.tc.load(Ordering::SeqCst) } }
    #[async_trait] impl WorkerCallback for Cb {
        async fn on_success(&self, _t: &str, _w: &str, _o: &str, _m: &WorkerMetrics) { self.sc.fetch_add(1, Ordering::SeqCst); }
        async fn on_failure(&self, _t: &str, _w: &str, _e: &str, _m: &WorkerMetrics) { self.fc.fetch_add(1, Ordering::SeqCst); }
        async fn on_timeout(&self, _t: &str, _w: &str, _e: u64) { self.tc.fetch_add(1, Ordering::SeqCst); }
    }

    #[tokio::test] async fn t01_spawn() { let mut s = sv(); let id = s.spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t")).await.unwrap(); assert!(id.contains("coder")); assert_eq!(s.worker_count(), 1); }
    #[tokio::test] async fn t02_stop() { let mut s = sv(); let id = s.spawn_worker(AgentRole::Researcher, AgentConfig::supervisor("t")).await.unwrap(); s.stop_worker(&id).await.unwrap(); tokio::time::sleep(tokio::time::Duration::from_millis(50)).await; assert_eq!(s.worker_count(), 0); }
    #[tokio::test] async fn t03_restart() { let mut s = sv(); let id = s.spawn_worker(AgentRole::Critic, AgentConfig::supervisor("t")).await.unwrap(); let nid = s.restart_worker(&id).await.unwrap(); assert_ne!(id, nid); assert_eq!(s.worker_count(), 1); }
    #[tokio::test] async fn t04_delegate() { let mut s = sv(); let wid = s.spawn_worker(AgentRole::Executor, AgentConfig::supervisor("t")).await.unwrap(); assert!(s.delegate(TaskAssignment { task_id: "t1".to_string(), description: "T".to_string(), assigned_to: wid, priority: 5 }).await.is_ok()); }
    #[tokio::test] async fn t05_crash() { let mut s = sv(); let id = s.spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t")).await.unwrap(); s.handle_worker_crash(&id).await; assert_eq!(s.worker_count(), 1); }
    #[tokio::test] async fn t06_e2e() { let mut s = sv(); let c = s.spawn_worker(AgentRole::Coder, AgentConfig::supervisor("t")).await.unwrap(); let r = s.spawn_worker(AgentRole::Researcher, AgentConfig::supervisor("t")).await.unwrap(); assert_eq!(s.worker_count(), 2); assert!(s.delegate(TaskAssignment { task_id: "c".to_string(), description: "W".to_string(), assigned_to: c, priority: 5 }).await.is_ok()); assert!(s.delegate(TaskAssignment { task_id: "r".to_string(), description: "S".to_string(), assigned_to: r, priority: 3 }).await.is_ok()); }
    #[tokio::test] async fn t07_register_cb() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.register_callback(cb.clone()); assert_eq!(cb.s(), 0); }
    #[tokio::test] async fn t08_handle_success() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.register_callback(cb.clone()); s.handle_worker_result(WorkerResult::success("t1", "w1".to_string(), "ok", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(cb.s(), 1); assert_eq!(s.lifecycle.results().lock().await.len(), 1); }
    #[tokio::test] async fn t09_handle_failure() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.register_callback(cb.clone()); s.handle_worker_result(WorkerResult::failure("t1", "w1".to_string(), "err", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(cb.f(), 1); }
    #[tokio::test] async fn t10_no_cb_fallback() { let s = sv(); s.handle_worker_result(WorkerResult::success("t1", "w1".to_string(), "ok", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(s.lifecycle.results().lock().await.len(), 1); }
    #[tokio::test] async fn t11_retry() { let s = sv(); assert!(s.retry_task(TaskAssignment { task_id: "t1".to_string(), description: "T".to_string(), assigned_to: "w1".to_string(), priority: 5 }).await.is_err()); }
    #[tokio::test] async fn t12_concurrent_cb() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.register_callback(cb.clone()); let s = Arc::new(s); let mut h = vec![]; for i in 0..5 { let sc = s.clone(); let wr = WorkerResult::success(format!("t{}", i), format!("w{}", i), "ok", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() }); h.push(tokio::spawn(async move { sc.handle_worker_result(wr).await; })); } for x in h { let _ = x.await; } assert_eq!(cb.s(), 5); }
    #[tokio::test] async fn t13_crash_notify() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.register_callback(cb.clone()); s.handle_worker_result(WorkerResult::failure("t1", "w1".to_string(), "crashed", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(cb.f(), 1); }
    #[tokio::test] async fn t14_retry_count() { let s = sv(); s.handle_worker_result(WorkerResult::failure("t1", "w1".to_string(), "err", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(s.lifecycle.retry_counts().lock().await.get("t1"), Some(&1)); }
    #[tokio::test] async fn t15_timeout_notify() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.register_callback(cb.clone()); s.handle_worker_result(WorkerResult::failure("t1", "w1".to_string(), "timeout", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(cb.f(), 1); }
    #[tokio::test] async fn t16_set_cb() { let mut s = sv(); let cb = Arc::new(Cb::new()); s.set_callback(cb.clone()); s.handle_worker_result(WorkerResult::success("t1", "w1".to_string(), "ok", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await; assert_eq!(cb.s(), 1); }

    // Phase 2 Patch: truncate_output tests
    #[test] fn test_truncate_output_empty() { assert_eq!(Supervisor::truncate_output("", 10), ""); }
    #[test] fn test_truncate_output_small() { let s = "hello"; assert_eq!(Supervisor::truncate_output(s, 10), s); }
    #[test] fn test_truncate_output_utf8_boundary() {
        let s = "αβγδε"; // 10 bytes (2 bytes each)
        assert_eq!(Supervisor::truncate_output(s, 3), "α...[truncated]"); // cuts at 2 bytes (α), 3 is not a boundary
    }

    // Phase 2 Patch: metrics accumulation
    #[tokio::test] async fn test_metrics_accumulate() {
        let s = sv();
        s.handle_worker_result(WorkerResult::success("t1", "w1".to_string(), "ok", WorkerMetrics { execution_time_ms: 42, retry_count: 0, timestamp: chrono::Utc::now() })).await;
        s.handle_worker_result(WorkerResult::failure("t2", "w2".to_string(), "err", WorkerMetrics { execution_time_ms: 10, retry_count: 0, timestamp: chrono::Utc::now() })).await;
        let m = s.metrics();
        assert_eq!(m.total_tasks.load(Ordering::Relaxed), 2);
        assert_eq!(m.successful_tasks.load(Ordering::Relaxed), 1);
        assert_eq!(m.failed_tasks.load(Ordering::Relaxed), 1);
        assert_eq!(m.total_execution_time_ms.load(Ordering::Relaxed), 52);
    }

    // Phase 2 Patch: duplicate task_id rejection
    #[tokio::test] async fn test_duplicate_task_id_rejected() {
        let mut s = sv();
        let wid = s.spawn_worker(AgentRole::Executor, AgentConfig::supervisor("t")).await.unwrap();
        let t1 = TaskAssignment { task_id: "dup".to_string(), description: "D".to_string(), assigned_to: wid.clone(), priority: 5 };
        s.delegate(t1).await.unwrap();
        // Manually push a result so the duplicate check sees it as in-flight
        s.handle_worker_result(WorkerResult::success("dup", wid.clone(), "done", WorkerMetrics { execution_time_ms: 0, retry_count: 0, timestamp: chrono::Utc::now() })).await;
        let t2 = TaskAssignment { task_id: "dup".to_string(), description: "D2".to_string(), assigned_to: wid, priority: 5 };
        assert!(s.delegate(t2).await.is_err(), "Expected duplicate task_id to be rejected");
    }
