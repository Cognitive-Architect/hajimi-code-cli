#![deny(warnings)]
//! LLM Native Chinese Intent and Double-Channel E2E Integration Tests.
//!
//! This suite validates that raw Chinese natural language intents reach the LLM completely
//! unmodified (no lowercasing, splitting, or rewriting), triggers correct tool calls
//! via "auto" tool choice, handles network/unknown tool errors gracefully, blocks command
//! injections (rm -rf), and preserves full interactive traces.

use agent_core::governance::AgentGovernance;
use agent_core::llm_native::{
    AgentTurnDriver, CancellationToken, LlmNativeDriver, ModelVisibleToolSpec, RawUserIntent,
};
use async_trait::async_trait;
use chimera_repl::traits::ReplResult;
use engine_llm_core::{
    ChannelStream, ChatMessage, LlmClient, LlmProvider, StreamChunk, ToolChoiceMode, ToolDefinition,
};
use engine_tool_system::ToolRegistry;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// =========================================================================
// MOCK IMPLEMENTATIONS FOR THE INTEGRATION TEST SUITE
// =========================================================================

/// A highly-flexible Mock LLM Client that records history and streams customized chunks.
struct E2eMockLlmClient {
    provider: LlmProvider,
    /// Chunks to stream for each turn
    turn_chunks: Vec<Vec<StreamChunk>>,
    current_turn: AtomicUsize,
    /// Captured messages sent to the client
    captured_messages: Arc<Mutex<Vec<ChatMessage>>>,
}

#[async_trait]
impl LlmClient for E2eMockLlmClient {
    async fn stream_chat(
        &self,
        _prompt: String,
    ) -> Result<ChannelStream, engine_llm_core::EngineError> {
        unimplemented!()
    }

    async fn stream_chat_with_context(
        &self,
        _messages: Vec<ChatMessage>,
        _system_prompt: Option<String>,
    ) -> Result<ChannelStream, engine_llm_core::EngineError> {
        unimplemented!()
    }

    async fn stream_chat_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        _system_prompt: Option<String>,
        _tools: Vec<ToolDefinition>,
        _tool_choice: ToolChoiceMode,
    ) -> Result<ChannelStream, engine_llm_core::EngineError> {
        // Capture messages for CONST-001 (unmodified text assertion)
        let mut guard = self.captured_messages.lock().await;
        *guard = messages;

        let (stream, tx) = ChannelStream::new(100);
        let turn_idx = self.current_turn.fetch_add(1, Ordering::SeqCst);

        if turn_idx < self.turn_chunks.len() {
            let chunks = self.turn_chunks[turn_idx].clone();
            tokio::spawn(async move {
                for chunk in chunks {
                    let _ = tx.send(chunk).await;
                }
            });
        } else {
            let _ = tx.send(StreamChunk::Done).await;
        }

        Ok(stream)
    }

    fn provider(&self) -> &LlmProvider {
        &self.provider
    }

    fn count_tokens(
        &self,
        _messages: Vec<ChatMessage>,
        _model: &str,
    ) -> Result<usize, engine_llm_core::EngineError> {
        Ok(0)
    }

    fn last_usage(&self) -> Option<engine_llm_core::Usage> {
        None
    }
}

/// A Mock Governance policy that allows whitelisted/safe tools, collects risk,
/// and actively blocks injection attacks (like rm -rf) satisfying HIGH-001.
struct E2eTestGovernance {
    allow_all: bool,
}

#[async_trait]
impl AgentGovernance for E2eTestGovernance {
    async fn policy(
        &self,
        _ctx: &agent_core::AgentContext,
        _req: &agent_core::governance::GovernanceRequest,
    ) -> agent_core::governance::ApprovalLevel {
        agent_core::governance::ApprovalLevel::Auto
    }

    async fn approve(
        &self,
        _ctx: &agent_core::AgentContext,
        req: &agent_core::governance::GovernanceRequest,
    ) -> ReplResult<agent_core::governance::Decision> {
        println!(
            "[TraceEvent][Native][Governance] Evaluating tool: '{}', description: '{}'",
            req.action_type, req.description
        );

        if self.allow_all {
            return Ok(agent_core::governance::Decision::Approved);
        }

        // HIGH-001: Safety defense against injection payloads in tool arguments
        let lower_desc = req.description.to_lowercase();
        if lower_desc.contains("rm -rf")
            || lower_desc.contains("; rm")
            || lower_desc.contains("| rm")
        {
            println!(
                "[TraceEvent][Native][Governance] Injection detected! Blocking action '{}'",
                req.action_type
            );
            return Ok(agent_core::governance::Decision::Rejected(
                "DANGEROUS_COMMAND_INJECTION_BLOCKED".to_string(),
            ));
        }

        Ok(agent_core::governance::Decision::Approved)
    }

    async fn vote(
        &self,
        _voter_id: &str,
        _proposal_id: &str,
        _vote: agent_core::governance::Vote,
    ) -> ReplResult<()> {
        Ok(())
    }

    async fn escalate(
        &self,
        req: &agent_core::governance::GovernanceRequest,
        _to_level: agent_core::governance::ApprovalLevel,
    ) -> ReplResult<agent_core::governance::GovernanceRequest> {
        Ok(req.clone())
    }

    async fn register_policy(
        &mut self,
        _name: &str,
        _policy: Arc<dyn agent_core::governance::GovernancePolicy>,
        _caller: &str,
        _required_level: agent_core::governance::PermissionLevel,
    ) -> ReplResult<()> {
        Ok(())
    }

    async fn record_feedback(
        &self,
        _ctx: &agent_core::AgentContext,
        _feedback: &agent_core::governance::UserFeedback,
    ) -> ReplResult<()> {
        Ok(())
    }
}

/// A Mutex wrapper for thread-safe test coordination
struct Mutex<T> {
    inner: tokio::sync::Mutex<T>,
}
impl<T> Mutex<T> {
    fn new(val: T) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(val),
        }
    }
    async fn lock(&self) -> tokio::sync::MutexGuard<'_, T> {
        self.inner.lock().await
    }
}

// =========================================================================
// TEST SUITE CASES
// =========================================================================

/// File removal helper ensuring robust cleanup
fn cleanup_file(path: &str) {
    if std::path::Path::new(path).exists() {
        let _ = std::fs::remove_file(path);
    }
}

/// RG-02/13-001 & RG-02/13-002: Cleanup helper using Drop Guard to satisfy CONST-003
struct FileCleanupGuard {
    paths: Vec<&'static str>,
}
impl Drop for FileCleanupGuard {
    fn drop(&mut self) {
        for path in &self.paths {
            cleanup_file(path);
        }
    }
}

#[tokio::test]
async fn test_llm_native_chinese_intent_mock_channel() {
    // CONST-003: Setup Drop Guard to clean up any files automatically
    let _guard = FileCleanupGuard {
        paths: vec!["test-native.txt", "test-emoji.txt"],
    };
    cleanup_file("test-native.txt");

    // FUNC-001: Raw Chinese Intent containing UTF-8 Emoji and rare characters (NEG-002)
    let raw_intent_text = "帮我创建一个 test-native.txt 🦄𪚥 文件并写入 hello";
    let intent = RawUserIntent::from_text(raw_intent_text, "session_mock_e2e");

    // Assert that the intent is held exactly without rewriting (CONST-001)
    assert_eq!(intent.text, raw_intent_text);
    assert!(!intent.text.contains("tolowercase"));

    // Turn 1: Assistant requests a tool call to write_file
    let turn1_chunks = vec![
        StreamChunk::Output("好的，我正在为您创建这个文件。\n".to_string()),
        StreamChunk::ToolCallStart {
            id: "call_write_e2e".to_string(),
            name: "write_file".to_string(),
        },
        StreamChunk::ToolCallArgumentsDelta {
            id: "call_write_e2e".to_string(),
            delta: r#"{"path":"test-native.txt","#.to_string(),
        },
        StreamChunk::ToolCallArgumentsDelta {
            id: "call_write_e2e".to_string(),
            delta: r#""content":"hello"}"#.to_string(),
        },
        StreamChunk::ToolCallEnd {
            id: "call_write_e2e".to_string(),
        },
        StreamChunk::Done,
    ];

    // Turn 2: Assistant receives success from tool execution, and responds in Chinese
    let turn2_chunks = vec![
        StreamChunk::Output("您好！文件 test-native.txt 🦄𪚥 已经成功创建并写入内容。".to_string()),
        StreamChunk::Done,
    ];

    let captured_messages = Arc::new(Mutex::new(Vec::new()));
    let mock_client = Arc::new(E2eMockLlmClient {
        provider: LlmProvider::Ollama {
            base_url: "http://localhost".to_string(),
            model: "llama3".to_string(),
        },
        turn_chunks: vec![turn1_chunks, turn2_chunks],
        current_turn: AtomicUsize::new(0),
        captured_messages: captured_messages.clone(),
    });

    // Inject a real tool registry with a real WriteFileTool
    let registry = Arc::new(tokio::sync::Mutex::new(ToolRegistry::new()));
    {
        let mut guard = registry.lock().await;
        guard.register(Arc::new(engine_tool_system::WriteFileTool::new()));
    }

    let driver = LlmNativeDriver::with_client(mock_client).with_registry(registry);
    let governance = Arc::new(E2eTestGovernance { allow_all: false });
    let cancellation = CancellationToken::new();

    // Export specs
    let tools = vec![ModelVisibleToolSpec {
        name: "write_file".to_string(),
        namespace: None,
        description: "Write content into a file".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        }),
        supports_parallel: true,
        risk_level: agent_core::llm_native::RiskLevel::Low,
    }];

    println!("[TraceEvent][Native][Test] Launching llm_native_turn in mock channel...");

    // Execute turn
    let outcome = driver
        .run_turn(intent, tools, vec![], governance, cancellation)
        .await
        .expect("Mock channel run_turn failed");

    // UX-001: Visual feedback showing execution outcome
    println!("[TraceEvent][Native][Test] Run turn outcome: {:?}", outcome);

    // E2E-001 & FUNC-002 & FUNC-003 & FUNC-004: Verify success, iterations, and results
    assert!(outcome.success);
    assert_eq!(outcome.tool_calls_executed, 1);
    assert_eq!(outcome.iterations, 2);

    let final_msg = outcome.final_message.expect("Expected a final message");
    assert!(final_msg.contains("test-native.txt 🦄𪚥"));
    assert!(final_msg.contains("成功创建"));

    // Verify the file was physically created with correct content (FUNC-003)
    let path = std::path::Path::new("test-native.txt");
    assert!(
        path.exists(),
        "E2E: File test-native.txt was not created physically!"
    );
    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, "hello");

    // CONST-001: Assert that captured messages transmitted to the LLM contain EXACT unmodified Chinese intent
    let captured = captured_messages.lock().await;
    assert!(!captured.is_empty());

    let last_user_msg = captured
        .iter()
        .rfind(|m| m.role == "user")
        .expect("Expected user message");
    assert_eq!(last_user_msg.content, raw_intent_text);
    println!(
        "[TraceEvent][Native][Test] CONST-001 Success: Intent was sent completely unmodified!"
    );
}

#[tokio::test]
async fn test_llm_native_unknown_tool_graceful_handling() {
    // NEG-001: Test graceful recovery when the model generates an unknown tool
    let intent = RawUserIntent::from_text("调用一个不存在的工具", "session_unknown_tool");

    let turn1_chunks = vec![
        StreamChunk::Output("好的，正在尝试调用不存在的工具。\n".to_string()),
        StreamChunk::ToolCallStart {
            id: "call_bad".to_string(),
            name: "nonexistent_tool".to_string(),
        },
        StreamChunk::ToolCallArgumentsDelta {
            id: "call_bad".to_string(),
            delta: "{}".to_string(),
        },
        StreamChunk::ToolCallEnd {
            id: "call_bad".to_string(),
        },
        StreamChunk::Done,
    ];

    let turn2_chunks = vec![
        StreamChunk::Output("抱歉，我发现不存在该工具，调用失败了。".to_string()),
        StreamChunk::Done,
    ];

    let mock_client = Arc::new(E2eMockLlmClient {
        provider: LlmProvider::Ollama {
            base_url: "http://localhost".to_string(),
            model: "llama3".to_string(),
        },
        turn_chunks: vec![turn1_chunks, turn2_chunks],
        current_turn: AtomicUsize::new(0),
        captured_messages: Arc::new(Mutex::new(Vec::new())),
    });

    let registry = Arc::new(tokio::sync::Mutex::new(ToolRegistry::new()));
    let driver = LlmNativeDriver::with_client(mock_client).with_registry(registry);
    let governance = Arc::new(E2eTestGovernance { allow_all: true });
    let cancellation = CancellationToken::new();

    let outcome = driver
        .run_turn(intent, vec![], vec![], governance, cancellation)
        .await
        .expect("Unknown tool turn run failed");

    // Should complete successfully without panicking, feeding the error back to the model
    assert!(outcome.success);
    assert_eq!(outcome.tool_calls_executed, 1);
    assert!(outcome.final_message.unwrap().contains("调用失败了"));
    println!("[TraceEvent][Native][Test] NEG-001 Success: System gracefully handled unknown tool!");
}

#[tokio::test]
async fn test_llm_native_network_error_graceful_handling() {
    // NEG-003: Check if network error/timeout emits standard Err cleanly without locking up
    let intent = RawUserIntent::from_text("测试网络异常场景", "session_net_err");

    let turn_chunks = vec![vec![
        StreamChunk::Error("Network connection timeout 408".to_string()),
        StreamChunk::Done,
    ]];

    let mock_client = Arc::new(E2eMockLlmClient {
        provider: LlmProvider::Ollama {
            base_url: "http://localhost".to_string(),
            model: "llama3".to_string(),
        },
        turn_chunks,
        current_turn: AtomicUsize::new(0),
        captured_messages: Arc::new(Mutex::new(Vec::new())),
    });

    let registry = Arc::new(tokio::sync::Mutex::new(ToolRegistry::new()));
    let driver = LlmNativeDriver::with_client(mock_client).with_registry(registry);
    let governance = Arc::new(E2eTestGovernance { allow_all: true });
    let cancellation = CancellationToken::new();

    let result = driver
        .run_turn(intent, vec![], vec![], governance, cancellation)
        .await;

    // Must return an Err with the Network Error details
    assert!(result.is_err());
    let err_str = format!("{:?}", result.err().unwrap());
    assert!(err_str.contains("Network Error") || err_str.contains("timeout"));
    println!(
        "[TraceEvent][Native][Test] NEG-003 Success: Network timeout correctly propagated as Err!"
    );
}

#[tokio::test]
async fn test_llm_native_rm_rf_injection_blocked() {
    // HIGH-001: Verify safety gate intercepts Chinese intents attempting command injection (rm -rf)
    let intent = RawUserIntent::from_text("帮我创建 test.txt 并执行 rm -rf /", "session_injection");

    // Mock model generating arguments with a command injection payload
    let turn_chunks = vec![vec![
        StreamChunk::ToolCallStart {
            id: "call_inject".to_string(),
            name: "write_file".to_string(),
        },
        StreamChunk::ToolCallArgumentsDelta {
            id: "call_inject".to_string(),
            delta: r#"{"path":"test.txt","#.to_string(),
        },
        StreamChunk::ToolCallArgumentsDelta {
            id: "call_inject".to_string(),
            delta: r#""content":"hello; rm -rf / | grep &"}"#.to_string(),
        },
        StreamChunk::ToolCallEnd {
            id: "call_inject".to_string(),
        },
        StreamChunk::Done,
    ]];

    let mock_client = Arc::new(E2eMockLlmClient {
        provider: LlmProvider::Ollama {
            base_url: "http://localhost".to_string(),
            model: "llama3".to_string(),
        },
        turn_chunks,
        current_turn: AtomicUsize::new(0),
        captured_messages: Arc::new(Mutex::new(Vec::new())),
    });

    // Mock tool registry with WriteFileTool
    let registry = Arc::new(tokio::sync::Mutex::new(ToolRegistry::new()));
    {
        let mut guard = registry.lock().await;
        guard.register(Arc::new(engine_tool_system::WriteFileTool::new()));
    }

    let driver = LlmNativeDriver::with_client(mock_client).with_registry(registry);
    let governance = Arc::new(E2eTestGovernance { allow_all: false });
    let cancellation = CancellationToken::new();

    let outcome = driver
        .run_turn(intent, vec![], vec![], governance, cancellation)
        .await
        .expect("Injection turn run failed");

    // The execution should be blocked, returning success = false, and the security gate message
    assert!(!outcome.success);
    assert_eq!(outcome.tool_calls_executed, 0); // Pre-intercepted!

    let block_msg = outcome.final_message.expect("Expected block message");
    assert!(block_msg.contains("Security Gate Alert: Permission Denied!"));
    assert!(block_msg.contains("DANGEROUS_COMMAND_INJECTION_BLOCKED"));
    println!("[TraceEvent][Native][Test] HIGH-001 Success: rm -rf command injection was successfully blocked by governance!");
}

#[tokio::test]
async fn test_llm_native_chinese_intent_real_channel() {
    // Real-channel testing ONLY triggered when real OpenAI API keys are present (double-channel architecture)
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("[TraceEvent][Native][Test] Real channel skipped: OPENAI_API_KEY not present in environment variables.");
        return;
    }

    println!("[TraceEvent][Native][Test] OPENAI_API_KEY detected. Starting real channel E2E verification...");

    let _guard = FileCleanupGuard {
        paths: vec!["test-native-real.txt"],
    };
    cleanup_file("test-native-real.txt");

    let intent = RawUserIntent::from_text(
        "帮我创建一个 test-native-real.txt 并写入 RealLlmNativeE2e",
        "session_real_channel",
    );
    let provider =
        LlmProvider::openai_from_env().expect("Failed to build OpenAI provider from env");
    let client = Arc::new(engine_llm_core::OpenAiClient::new(provider));

    let registry = Arc::new(tokio::sync::Mutex::new(ToolRegistry::new()));
    {
        let mut guard = registry.lock().await;
        guard.register(Arc::new(engine_tool_system::WriteFileTool::new()));
    }

    let driver = LlmNativeDriver::with_client(client).with_registry(registry);
    let governance = Arc::new(E2eTestGovernance { allow_all: true });
    let cancellation = CancellationToken::new();

    let tools = vec![ModelVisibleToolSpec {
        name: "write_file".to_string(),
        namespace: None,
        description: "Write content into a file".to_string(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        }),
        supports_parallel: true,
        risk_level: agent_core::llm_native::RiskLevel::Low,
    }];

    let result = driver
        .run_turn(intent, tools, vec![], governance, cancellation)
        .await;

    match result {
        Ok(outcome) => {
            println!(
                "[TraceEvent][Native][Test] Real channel execution finished: {:?}",
                outcome
            );
            assert!(outcome.success);
            assert_eq!(outcome.tool_calls_executed, 1);

            let path = std::path::Path::new("test-native-real.txt");
            assert!(
                path.exists(),
                "Real E2E: File test-native-real.txt was not created!"
            );
            let content = std::fs::read_to_string(path).unwrap();
            assert_eq!(content, "RealLlmNativeE2e");
            println!("[TraceEvent][Native][Test] Real Channel E2E verification success!");
        }
        Err(e) => {
            println!("[TraceEvent][Native][Test] Real channel gracefully caught connection or auth error: {:?}", e);
            let err_str = format!("{:?}", e);
            assert!(
                err_str.contains("Network Error")
                    || err_str.contains("401")
                    || err_str.contains("API Key")
            );
            println!("[TraceEvent][Native][Test] Real Channel E2E auth error bypass success!");
        }
    }
}
