    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_event_processor_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let sender = ReplEventSender::new(tx);
        let memory = Arc::new(Mutex::new(MemoryGateway::new("test")));
        let context = AgentContext::new();

        let processor = AgentEventProcessor::new(sender, memory, context);
        assert_eq!(processor.context.cycle_count, 0);
    }

    #[tokio::test]
    async fn test_process_worker_result_event() {
        let (tx, mut rx) = mpsc::channel(10);
        let sender = ReplEventSender::new(tx);
        let memory = Arc::new(Mutex::new(MemoryGateway::new("test")));
        let context = AgentContext::new();
        let processor = AgentEventProcessor::new(sender, memory, context);

        let wr = WorkerResult::success("t1", "w1".to_string(), "output", WorkerMetrics::new(42));
        processor.process_worker_result("agent-1", &wr).await.unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            ReplEvent::SwarmTaskCompleted { task_id, success, .. } => {
                assert_eq!(task_id, "t1");
                assert!(success);
            }
            _ => panic!("Expected SwarmTaskCompleted"),
        }
    }
