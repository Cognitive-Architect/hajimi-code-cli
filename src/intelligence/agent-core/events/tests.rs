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
    processor
        .process_worker_result("agent-1", &wr)
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        ReplEvent::SwarmTaskCompleted {
            task_id, success, ..
        } => {
            assert_eq!(task_id, "t1");
            assert!(success);
        }
        _ => panic!("Expected SwarmTaskCompleted"),
    }
}

#[tokio::test]
async fn test_process_operation_summary() {
    let (tx, mut rx) = mpsc::channel(10);
    let sender = ReplEventSender::new(tx);
    let memory = Arc::new(Mutex::new(MemoryGateway::new("test")));
    let context = AgentContext::new();
    let processor = AgentEventProcessor::new(sender, memory, context);

    let summary = crate::OperationSummary {
        files_edited: 2,
        files_created: 1,
        files_deleted: 0,
        commands_run: 3,
        total_diff_lines: 45,
    };
    processor
        .process_operation_summary("agent-1", summary.clone())
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        ReplEvent::OperationSummary {
            agent_id,
            summary: s,
        } => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(s.files_edited, 2);
            assert_eq!(s.files_created, 1);
            assert_eq!(s.total_diff_lines, 45);
        }
        _ => panic!("Expected OperationSummary"),
    }
}

#[tokio::test]
async fn test_process_thinking_content() {
    let (tx, mut rx) = mpsc::channel(10);
    let sender = ReplEventSender::new(tx);
    let memory = Arc::new(Mutex::new(MemoryGateway::new("test")));
    let context = AgentContext::new();
    let processor = AgentEventProcessor::new(sender, memory, context);

    processor
        .process_thinking_content("agent-1", "Analyzing the codebase structure...")
        .await
        .unwrap();

    let event = rx.recv().await.unwrap();
    match event {
        ReplEvent::ThinkingContent { agent_id, content } => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(content, "Analyzing the codebase structure...");
        }
        _ => panic!("Expected ThinkingContent"),
    }
}
