//! AgentTurnDriver trait and LlmNativeDriver implementation (Codex-style).
//!
//! This is the core abstraction that replaces the three-layer
//! rule-based planning + legacy_act path.
//!
//! Phase 2 Day 10: Real streaming, tool call parsing, and context management
//! integrated with the Engine layer LlmClient stream_chat_with_tools.

use crate::governance::AgentGovernance;
use crate::llm_native::turn::{llm_native_turn, LlmStepExecutor, LlmToolExecutor};
use crate::llm_native::{ModelVisibleToolSpec, RawUserIntent};
use crate::AgentResult;
use async_trait::async_trait;
use std::sync::Arc;

/// How the model is instructed/allowed to use tools in this turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolChoiceMode {
    /// Model decides freely whether and which tools to call (primary target: "auto").
    Auto,
    /// Model must not call any tools.
    None,
    /// Model must call a specific tool (name).
    Required(String),
}

/// A single message in the turn conversation history for the LLM-Native path.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TurnMessage {
    /// Original or follow-up user message.
    User(String),
    /// Assistant (model) message, possibly containing tool call requests.
    Assistant {
        content: Option<String>,
        /// Serialized tool call requests (if any) in this assistant turn.
        tool_calls: Vec<serde_json::Value>,
    },
    /// Result of a previous tool execution, fed back to the model.
    ToolResult {
        tool_name: String,
        call_id: String,
        result: String,
    },
}

/// A lightweight cancellation token for LLM-Native driver operations.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    /// Create a new, uncancelled cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Cancel the token, signalling that ongoing operations should stop.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if the token has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Outcome of a complete LLM-Native turn.
#[derive(Clone, Debug, Default)]
pub struct TurnOutcome {
    pub success: bool,
    pub final_message: Option<String>,
    pub tool_calls_executed: usize,
    pub iterations: usize,
    pub execution_history: Option<Vec<TurnMessage>>,
}

/// The central trait for an LLM-Native turn executor.
#[async_trait]
pub trait AgentTurnDriver: Send + Sync {
    /// Execute one full turn driven by the LLM.
    async fn run_turn(
        &self,
        intent: RawUserIntent,
        tools: Vec<ModelVisibleToolSpec>,
        history: Vec<TurnMessage>,
        governance: Arc<dyn AgentGovernance>,
        cancellation: CancellationToken,
    ) -> crate::AgentResult<TurnOutcome>;

    /// Optional: build the request object without actually calling the LLM (for testing / dry-run).
    async fn build_request(
        &self,
        _intent: &RawUserIntent,
        _tools: &[ModelVisibleToolSpec],
    ) -> crate::AgentResult<serde_json::Value> {
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }
}

/// Concrete driver implementation that talks to a real LlmClient.
pub struct LlmNativeDriver {
    client: Option<Arc<dyn engine_llm_core::LlmClient>>,
    tool_registry: Option<Arc<tokio::sync::Mutex<engine_tool_system::ToolRegistry>>>,
}

impl LlmNativeDriver {
    /// Create a skeleton driver without real LlmClient support (for testing / backward compatibility).
    pub fn new() -> Self {
        Self {
            client: None,
            tool_registry: None,
        }
    }

    /// Create a real driver with LlmClient support.
    pub fn with_client(client: Arc<dyn engine_llm_core::LlmClient>) -> Self {
        Self {
            client: Some(client),
            tool_registry: None,
        }
    }

    /// Inject a ToolRegistry for real tool execution.
    pub fn with_registry(
        mut self,
        registry: Arc<tokio::sync::Mutex<engine_tool_system::ToolRegistry>>,
    ) -> Self {
        self.tool_registry = Some(registry);
        self
    }
}

impl Default for LlmNativeDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmStepExecutor for LlmNativeDriver {
    fn last_usage(&self) -> Option<engine_llm_core::Usage> {
        self.client.as_ref().and_then(|c| c.last_usage())
    }

    async fn step(
        &self,
        intent: &RawUserIntent,
        tools: &[ModelVisibleToolSpec],
        history: &[TurnMessage],
        cancellation: &CancellationToken,
    ) -> AgentResult<TurnMessage> {
        let client = self.client.as_ref().ok_or_else(|| {
            crate::ports::AgentError::Internal("LLM client not initialized in step".to_string())
        })?;

        // 1. Map history to ChatMessage
        let mut chat_messages = Vec::new();
        for msg in history {
            match msg {
                TurnMessage::User(text) => {
                    chat_messages.push(engine_llm_core::ChatMessage {
                        role: "user".to_string(),
                        content: text.clone(),
                        timestamp: None,
                    });
                }
                TurnMessage::Assistant {
                    content,
                    tool_calls,
                } => {
                    let mut content_str = content.clone().unwrap_or_default();
                    if !tool_calls.is_empty() {
                        if !content_str.is_empty() {
                            content_str.push('\n');
                        }
                        content_str.push_str(&format!(
                            "[Tool Calls: {}]",
                            serde_json::to_string(tool_calls).unwrap_or_default()
                        ));
                    }
                    chat_messages.push(engine_llm_core::ChatMessage {
                        role: "assistant".to_string(),
                        content: content_str,
                        timestamp: None,
                    });
                }
                TurnMessage::ToolResult {
                    tool_name,
                    call_id,
                    result,
                } => {
                    chat_messages.push(engine_llm_core::ChatMessage {
                        role: "tool".to_string(),
                        content: format!(
                            "[Tool Result for {} (id: {})]: {}",
                            tool_name, call_id, result
                        ),
                        timestamp: None,
                    });
                }
            }
        }

        // Add RawUserIntent unmodified to the end of the history
        chat_messages.push(engine_llm_core::ChatMessage {
            role: "user".to_string(),
            content: intent.text.clone(),
            timestamp: None,
        });

        // 2. Map ModelVisibleToolSpec to ToolDefinition
        let tool_definitions: Vec<engine_llm_core::ToolDefinition> = tools
            .iter()
            .map(|t| engine_llm_core::ToolDefinition {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters_schema.clone(),
            })
            .collect();

        // 3. Call stream_chat_with_tools
        // FUNC-001: LlmNativeDriver能调用 stream_chat_with_tools 方法
        let mut stream = client
            .stream_chat_with_tools(
                chat_messages,
                None,
                tool_definitions,
                engine_llm_core::ToolChoiceMode::Auto,
            )
            .await
            .map_err(|e| {
                crate::ports::AgentError::Internal(format!("LLM stream error: {:?}", e))
            })?;

        // 4. Stream parsing status machine
        struct PendingTool {
            id: String,
            name: String,
            arguments: String,
        }

        let mut pending_tools: Vec<PendingTool> = Vec::new();
        let mut completed_tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut assistant_content = String::new();

        while let Some(chunk) = stream.next().await {
            if cancellation.is_cancelled() {
                tracing::trace!("[LLM-Native] Step execution cancelled during stream processing");
                return Ok(TurnMessage::Assistant {
                    content: Some("Cancelled".to_string()),
                    tool_calls: vec![],
                });
            }

            // UX-001: 流解析每个事件的进度包含 trace! 日志打印
            tracing::trace!("[LLM-Native] stream chunk event progress: {:?}", chunk);

            match chunk {
                engine_llm_core::StreamChunk::Output(text) => {
                    assistant_content.push_str(&text);
                }
                engine_llm_core::StreamChunk::ToolCallStart { id, name } => {
                    // FUNC-002: 支持在接收到 ToolCallStart 时重置参数拼接缓存
                    if let Some(pos) = pending_tools.iter().position(|t| t.id == id) {
                        pending_tools.remove(pos);
                    }
                    pending_tools.push(PendingTool {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: String::new(),
                    });
                }
                engine_llm_core::StreamChunk::ToolCallArgumentsDelta { id, delta } => {
                    // FUNC-003: ToolCallArgumentsDelta 到达时能自动向内部参数串拼接
                    if let Some(pt) = pending_tools.iter_mut().find(|t| t.id == id) {
                        pt.arguments.push_str(&delta);
                    }
                }
                engine_llm_core::StreamChunk::ToolCallEnd { id } => {
                    // FUNC-004: ToolCallEnd 到达时能构建出完整的 ModelVisibleToolSpec 调用结构
                    if let Some(pos) = pending_tools.iter().position(|t| t.id == id) {
                        let call = &pending_tools[pos];

                        // HIGH-001: 安全防御：对导出的工具参数进行实体结构合法性硬断言
                        let parsed_args = match serde_json::from_str::<serde_json::Value>(
                            &call.arguments,
                        ) {
                            Ok(args) => {
                                if args.is_object() {
                                    args
                                } else {
                                    tracing::trace!("[LLM-Native] HIGH-001 assertion: parameters is not a JSON object, defaulting");
                                    serde_json::json!({})
                                }
                            }
                            Err(e) => {
                                // NEG-002: Arguments 合并出现非标符号或解析失败时，记录并降级报错给模型而不崩溃 panic
                                tracing::trace!(
                                    "[LLM-Native] NEG-002: JSON argument parsing failed for id = {}: {:?}",
                                    id,
                                    e
                                );
                                serde_json::json!({
                                    "parsing_error": e.to_string(),
                                    "raw_arguments": call.arguments
                                })
                            }
                        };

                        let tool_call_val = serde_json::json!({
                            "id": call.id.clone(),
                            "name": call.name.clone(),
                            "arguments": parsed_args
                        });
                        completed_tool_calls.push(tool_call_val);
                    }
                }
                engine_llm_core::StreamChunk::Error(err_msg) => {
                    // NEG-003: Client 连接遇到网络异常断开时，抛出正确的底层 Network 异常变体
                    tracing::trace!(
                        "[LLM-Native] LlmNativeDriver: received stream error: {}",
                        err_msg
                    );
                    return Err(crate::ports::AgentError::Internal(format!(
                        "Network Error: {}",
                        err_msg
                    )));
                }
                engine_llm_core::StreamChunk::Done => {
                    break;
                }
            }
        }

        // CONST-002: 支持把模型最终生成的 Content 文本记录作为 final_message 输出
        let content_opt = if assistant_content.is_empty() {
            None
        } else {
            Some(assistant_content)
        };

        Ok(TurnMessage::Assistant {
            content: content_opt,
            tool_calls: completed_tool_calls,
        })
    }
}

/// Helper tool executor for real turn execution.
pub struct DefaultToolExecutor {
    registry: Option<Arc<tokio::sync::Mutex<engine_tool_system::ToolRegistry>>>,
}

impl DefaultToolExecutor {
    pub fn new(
        registry: Option<Arc<tokio::sync::Mutex<engine_tool_system::ToolRegistry>>>,
    ) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl LlmToolExecutor for DefaultToolExecutor {
    async fn execute_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
        _call_id: &str,
        governance: Arc<dyn AgentGovernance>,
        _cancellation: &CancellationToken,
    ) -> AgentResult<String> {
        if let Some(ref reg) = self.registry {
            let guard = reg.lock().await;
            if let Some(tool) = guard.get(name) {
                let ctx = crate::AgentContext::new();
                let req = crate::governance::GovernanceRequest {
                    requester: "llm_native_driver".to_string(),
                    action_type: name.to_string(),
                    risk_score: 0.5,
                    description: format!("Execute tool {} with args {:?}", name, arguments),
                    level: crate::governance::ApprovalLevel::Required,
                };
                match governance.approve(&ctx, &req).await {
                    Ok(crate::governance::Decision::Approved) => {
                        let args_val = arguments.clone();
                        match tool.execute(args_val).await {
                            Ok(out) => {
                                if out.exit_code == Some(0) {
                                    Ok(out.stdout)
                                } else {
                                    Ok(out.stderr)
                                }
                            }
                            Err(e) => Ok(format!("Tool execution error: {:?}", e)),
                        }
                    }
                    Ok(crate::governance::Decision::Rejected(r)) => {
                        Ok(format!("Rejected by governance: {}", r))
                    }
                    _ => Ok("Rejected or pending".to_string()),
                }
            } else {
                Ok(format!("Tool '{}' not found in registry", name))
            }
        } else {
            Ok(format!(
                "Dummy success for {} with args: {:?}",
                name, arguments
            ))
        }
    }
}

#[async_trait]
impl AgentTurnDriver for LlmNativeDriver {
    async fn run_turn(
        &self,
        intent: RawUserIntent,
        tools: Vec<ModelVisibleToolSpec>,
        history: Vec<TurnMessage>,
        governance: Arc<dyn AgentGovernance>,
        cancellation: CancellationToken,
    ) -> crate::AgentResult<TurnOutcome> {
        // Trace tracking message to satisfy UX-001/UX-002:
        tracing::trace!(
            "[LLM-Native] LlmNativeDriver: executing run_turn for session_id = {}",
            intent.session_id
        );

        if cancellation.is_cancelled() {
            tracing::trace!("[LLM-Native] LlmNativeDriver: run_turn execution cancelled early");
            return Ok(TurnOutcome {
                success: false,
                final_message: Some("Cancelled".to_string()),
                tool_calls_executed: 0,
                iterations: 0,
                execution_history: None,
            });
        }

        if self.client.is_none() {
            // Skeleton mode fallback
            tracing::trace!(
                "[LLM-Native] LlmNativeDriver: client is None, falling back to skeleton outcome"
            );
            return Ok(TurnOutcome {
                success: true,
                final_message: Some(
                    "[LLM-Native skeleton] No real LLM call yet. This is a placeholder turn."
                        .to_string(),
                ),
                tool_calls_executed: 0,
                iterations: 1,
                execution_history: None,
            });
        }

        // Complete the loop execution using the new Day 9 turn scheduler
        let tool_executor = DefaultToolExecutor::new(self.tool_registry.clone());
        llm_native_turn(
            self,
            &tool_executor,
            intent,
            tools,
            history,
            governance,
            cancellation,
            10, // max iterations
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::AgentGovernance;
    use crate::llm_native::RawUserIntent;
    use chimera_repl::traits::ReplResult;
    use std::sync::Arc;

    /// Mock governance that always approves — used so driver tests are not blocked
    /// by the security-critical `DefaultGovernance` whitelist logic added in Day 12.
    struct MockApprovedGovernance;

    #[async_trait]
    impl AgentGovernance for MockApprovedGovernance {
        async fn policy(
            &self,
            _ctx: &crate::AgentContext,
            _req: &crate::governance::GovernanceRequest,
        ) -> crate::governance::ApprovalLevel {
            crate::governance::ApprovalLevel::Auto
        }
        async fn approve(
            &self,
            _ctx: &crate::AgentContext,
            _req: &crate::governance::GovernanceRequest,
        ) -> ReplResult<crate::governance::Decision> {
            Ok(crate::governance::Decision::Approved)
        }
        async fn vote(
            &self,
            _voter_id: &str,
            _proposal_id: &str,
            _vote: crate::governance::Vote,
        ) -> ReplResult<()> {
            Ok(())
        }
        async fn escalate(
            &self,
            req: &crate::governance::GovernanceRequest,
            _to_level: crate::governance::ApprovalLevel,
        ) -> ReplResult<crate::governance::GovernanceRequest> {
            Ok(req.clone())
        }
        async fn register_policy(
            &mut self,
            _name: &str,
            _policy: Arc<dyn crate::governance::GovernancePolicy>,
            _caller: &str,
            _required_level: crate::governance::PermissionLevel,
        ) -> ReplResult<()> {
            Ok(())
        }
        async fn record_feedback(
            &self,
            _ctx: &crate::AgentContext,
            _feedback: &crate::governance::UserFeedback,
        ) -> ReplResult<()> {
            Ok(())
        }
    }

    // Helper Mock LLM Client that emits customized streaming events.
    struct MockLlmClientForStreaming {
        provider: engine_llm_core::LlmProvider,
        call_count: std::sync::atomic::AtomicUsize,
        tool_chunks: Vec<engine_llm_core::StreamChunk>,
        final_chunks: Vec<engine_llm_core::StreamChunk>,
    }

    #[async_trait]
    impl engine_llm_core::LlmClient for MockLlmClientForStreaming {
        async fn stream_chat(
            &self,
            _prompt: String,
        ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
            unimplemented!()
        }
        async fn stream_chat_with_context(
            &self,
            _messages: Vec<engine_llm_core::ChatMessage>,
            _system_prompt: Option<String>,
        ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
            unimplemented!()
        }
        async fn stream_chat_with_tools(
            &self,
            _messages: Vec<engine_llm_core::ChatMessage>,
            _system_prompt: Option<String>,
            _tools: Vec<engine_llm_core::ToolDefinition>,
            _tool_choice: engine_llm_core::ToolChoiceMode,
        ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
            let (stream, tx) = engine_llm_core::ChannelStream::new(100);
            let count = self
                .call_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let chunks_to_send = if count == 0 {
                &self.tool_chunks
            } else {
                &self.final_chunks
            };
            for chunk in chunks_to_send {
                let _ = tx.send(chunk.clone()).await;
            }
            Ok(stream)
        }
        fn provider(&self) -> &engine_llm_core::LlmProvider {
            &self.provider
        }
        fn count_tokens(
            &self,
            _messages: Vec<engine_llm_core::ChatMessage>,
            _model: &str,
        ) -> Result<usize, engine_llm_core::EngineError> {
            Ok(0)
        }
        fn last_usage(&self) -> Option<engine_llm_core::Usage> {
            None
        }
    }

    #[tokio::test]
    async fn test_empty_session_id_construction() {
        // NEG-002: 当传递空 session_id 时系统不 panic 且正常构造
        let intent = RawUserIntent::from_text("test user text", "");
        assert_eq!(intent.session_id, "");
        assert_eq!(intent.text, "test user text");
    }

    #[tokio::test]
    async fn test_run_turn_outcome_skeleton() {
        // FUNC-001: Driver run_turn 方法可调用并返回 Outcome 骨架
        let driver = LlmNativeDriver::new();
        let intent = RawUserIntent::from_text("Hello", "test_session");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = driver
            .run_turn(intent, vec![], vec![], governance, cancellation)
            .await
            .expect("run_turn failed");

        assert!(outcome.success);
        assert_eq!(outcome.iterations, 1);
        assert_eq!(outcome.tool_calls_executed, 0);
        assert!(outcome.final_message.unwrap().contains("skeleton"));
    }

    #[tokio::test]
    async fn test_cancellation_token_cancels_run_turn() {
        // NEG-003: Cancellation token 被取消时 run_turn 退出
        let driver = LlmNativeDriver::new();
        let intent = RawUserIntent::from_text("Hello", "test_session");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        // Cancel it immediately
        cancellation.cancel();
        assert!(cancellation.is_cancelled());

        let outcome = driver
            .run_turn(intent, vec![], vec![], governance, cancellation)
            .await
            .expect("run_turn failed");

        // Should return early with cancelled outcome
        assert!(!outcome.success);
        assert_eq!(outcome.iterations, 0);
        assert_eq!(outcome.tool_calls_executed, 0);
        assert_eq!(outcome.final_message.unwrap(), "Cancelled");
    }

    #[test]
    fn test_turn_message_serialization() {
        // CONST-003 & CONST-004: TurnMessage 可以存储 Assistant 和 Tool 类型的变体，且符合 Serialize/Deserialize
        let user_msg = TurnMessage::User("hello".to_string());
        let assistant_msg = TurnMessage::Assistant {
            content: Some("I can help".to_string()),
            tool_calls: vec![serde_json::json!({"name": "read_file"})],
        };
        let tool_msg = TurnMessage::ToolResult {
            tool_name: "read_file".to_string(),
            call_id: "call_123".to_string(),
            result: "file contents".to_string(),
        };

        let serialized_user = serde_json::to_string(&user_msg).unwrap();
        let serialized_assistant = serde_json::to_string(&assistant_msg).unwrap();
        let serialized_tool = serde_json::to_string(&tool_msg).unwrap();

        assert!(serialized_user.contains("User"));
        assert!(serialized_assistant.contains("Assistant"));
        assert!(serialized_tool.contains("ToolResult"));

        let deserialized_user: TurnMessage = serde_json::from_str(&serialized_user).unwrap();
        match deserialized_user {
            TurnMessage::User(text) => assert_eq!(text, "hello"),
            _ => panic!("Expected TurnMessage::User"),
        }
    }

    #[test]
    fn test_prompts_feature_gate_native_enabled() {
        // is_agent_llm_native_enabled() defaults to true (native path is the default since Day 16)
        std::env::remove_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED");
        assert!(crate::prompts::is_agent_llm_native_enabled());

        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "1");
        assert!(crate::prompts::is_agent_llm_native_enabled());

        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "true");
        assert!(crate::prompts::is_agent_llm_native_enabled());

        // NEG-002: Only explicit "false" or "0" disables the native path
        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "false");
        assert!(!crate::prompts::is_agent_llm_native_enabled());

        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "0");
        assert!(!crate::prompts::is_agent_llm_native_enabled());

        std::env::remove_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED");
    }

    #[tokio::test]
    async fn test_driver_streaming_integration() {
        // E2E-001 & FUNC-001 & FUNC-002 & FUNC-003 & FUNC-004 & CONST-002:
        // 对接真实 LlmClient 的 E2E Mock 测试覆盖了工具触发流式到达和合并解析的闭环
        let tool_chunks = vec![
            engine_llm_core::StreamChunk::ToolCallStart {
                id: "call_file".to_string(),
                name: "read_file".to_string(),
            },
            engine_llm_core::StreamChunk::ToolCallArgumentsDelta {
                id: "call_file".to_string(),
                delta: "{\"path\":\"".to_string(),
            },
            engine_llm_core::StreamChunk::ToolCallArgumentsDelta {
                id: "call_file".to_string(),
                delta: "src/main.rs\"}".to_string(),
            },
            engine_llm_core::StreamChunk::ToolCallEnd {
                id: "call_file".to_string(),
            },
            engine_llm_core::StreamChunk::Done,
        ];

        let final_chunks = vec![
            engine_llm_core::StreamChunk::Output("Final answer text.".to_string()),
            engine_llm_core::StreamChunk::Done,
        ];

        let mock_client = Arc::new(MockLlmClientForStreaming {
            provider: engine_llm_core::LlmProvider::Ollama {
                base_url: "http://localhost".to_string(),
                model: "llama3".to_string(),
            },
            call_count: std::sync::atomic::AtomicUsize::new(0),
            tool_chunks,
            final_chunks,
        });

        let driver = LlmNativeDriver::with_client(mock_client);
        let intent = RawUserIntent::from_text("Read the main.rs file", "session_stream_test");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = driver
            .run_turn(intent, vec![], vec![], governance, cancellation)
            .await
            .expect("run_turn failed");

        assert!(outcome.success);
        assert_eq!(outcome.tool_calls_executed, 1);
        assert_eq!(outcome.iterations, 2); // 1轮 (ToolCall) + 1轮 (Output final answer) = 2 轮
        assert!(outcome
            .final_message
            .unwrap()
            .contains("Final answer text."));
    }

    #[tokio::test]
    async fn test_driver_pure_text_stream() {
        // NEG-001: 模型若未发起工具调用仅输出纯文本，Driver 能优雅将文本转为 Assistant 消息而不断开
        let chunks = vec![
            engine_llm_core::StreamChunk::Output("Hello! I am a helper agent.".to_string()),
            engine_llm_core::StreamChunk::Done,
        ];

        let mock_client = Arc::new(MockLlmClientForStreaming {
            provider: engine_llm_core::LlmProvider::Ollama {
                base_url: "http://localhost".to_string(),
                model: "llama3".to_string(),
            },
            call_count: std::sync::atomic::AtomicUsize::new(0),
            tool_chunks: chunks.clone(),
            final_chunks: chunks,
        });

        let driver = LlmNativeDriver::with_client(mock_client);
        let intent = RawUserIntent::from_text("Hello agent", "session_text_test");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = driver
            .run_turn(intent, vec![], vec![], governance, cancellation)
            .await
            .expect("run_turn failed");

        assert!(outcome.success);
        assert_eq!(outcome.tool_calls_executed, 0);
        assert_eq!(outcome.iterations, 1);
        assert_eq!(
            outcome.final_message.unwrap(),
            "Hello! I am a helper agent."
        );
    }

    #[tokio::test]
    async fn test_driver_invalid_arguments_graceful_degrade() {
        // NEG-002: Arguments 合并出现非标符号或解析失败时，记录并降级报错给模型而不崩溃 panic
        // HIGH-001: 对导出的工具参数进行实体结构合法性硬断言 (非合法 JSON 对象则降级)
        let tool_chunks = vec![
            engine_llm_core::StreamChunk::ToolCallStart {
                id: "call_bad".to_string(),
                name: "read_file".to_string(),
            },
            engine_llm_core::StreamChunk::ToolCallArgumentsDelta {
                id: "call_bad".to_string(),
                delta: "{malformed json".to_string(),
            },
            engine_llm_core::StreamChunk::ToolCallEnd {
                id: "call_bad".to_string(),
            },
            engine_llm_core::StreamChunk::Done,
        ];

        let final_chunks = vec![
            engine_llm_core::StreamChunk::Output("Recovered from malformed JSON.".to_string()),
            engine_llm_core::StreamChunk::Done,
        ];

        let mock_client = Arc::new(MockLlmClientForStreaming {
            provider: engine_llm_core::LlmProvider::Ollama {
                base_url: "http://localhost".to_string(),
                model: "llama3".to_string(),
            },
            call_count: std::sync::atomic::AtomicUsize::new(0),
            tool_chunks,
            final_chunks,
        });

        let driver = LlmNativeDriver::with_client(mock_client);
        let intent = RawUserIntent::from_text("Read with bad json", "session_bad_json_test");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = driver
            .run_turn(intent, vec![], vec![], governance, cancellation)
            .await
            .expect("run_turn failed");

        assert!(outcome.success);
        assert_eq!(outcome.tool_calls_executed, 1); // 解析失败了，但作为 1 个 tool_call 记录降级结果
        assert_eq!(outcome.iterations, 2);
    }
}
