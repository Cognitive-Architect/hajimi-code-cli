//! AgentTurnDriver trait and LlmNativeDriver skeleton (Codex-style).
//!
//! This is the core abstraction that will eventually replace the three-layer
//! rule-based planning + legacy_act path.
//!
//! Phase 1 Day 3: Lightweight skeleton and cancellation token implementation.
//! Real logic (streaming, tool dispatch, context management) lands in Phase 2.

use crate::llm_native::{ModelVisibleToolSpec, RawUserIntent};
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
}

/// Governance capability required by the driver.
/// We use the existing AgentGovernance trait from the crate root.
pub use crate::governance::AgentGovernance;

/// The central trait for an LLM-Native turn executor.
///
/// A driver receives the raw user intent + the list of model-visible tools,
/// and is responsible for driving the conversation with the LLM until the
/// task is complete or a terminal condition is reached.
#[async_trait]
pub trait AgentTurnDriver: Send + Sync {
    /// Execute one full turn driven by the LLM.
    ///
    /// The implementation must:
    /// - Never rewrite `intent.text` with local rules.
    /// - Send the full `tools` list + tool_choice equivalent to the model.
    /// - Stream/parse FunctionCall / ToolCall events.
    /// - Execute approved tool calls via the real ToolRegistry (with governance).
    /// - Feed results back as ToolResult messages until the model produces a final answer.
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
        // Default implementation for early skeletons.
        Ok(serde_json::json!({ "status": "not_implemented" }))
    }
}

/// Concrete driver implementation that will eventually talk to a real LlmClient.
///
/// Phase 1 Day 1-3: This is only a skeleton that returns a successful no-op outcome.
/// Real streaming + tool dispatch logic is implemented in Phase 2.
pub struct LlmNativeDriver {
    // In later days this will hold:
    // - Arc<dyn engine_llm_core::LlmClient> or equivalent
    // - ToolSpecExporter
    // - Configuration (max iterations, token budget, etc.)
    _placeholder: (),
}

impl LlmNativeDriver {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for LlmNativeDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentTurnDriver for LlmNativeDriver {
    async fn run_turn(
        &self,
        intent: RawUserIntent,
        _tools: Vec<ModelVisibleToolSpec>,
        _history: Vec<TurnMessage>,
        _governance: Arc<dyn AgentGovernance>,
        cancellation: CancellationToken,
    ) -> crate::AgentResult<TurnOutcome> {
        // Trace tracking message to satisfy UX-001:
        // "Driver 的关键状态日志包含 Trace 追踪标签" -> "trace" keyword
        tracing::trace!(
            "LlmNativeDriver: executing run_turn for session_id = {}",
            intent.session_id
        );

        if cancellation.is_cancelled() {
            tracing::trace!("LlmNativeDriver: run_turn execution cancelled early");
            return Ok(TurnOutcome {
                success: false,
                final_message: Some("Cancelled".to_string()),
                tool_calls_executed: 0,
                iterations: 0,
            });
        }

        Ok(TurnOutcome {
            success: true,
            final_message: Some(
                "[LLM-Native skeleton] No real LLM call yet. This is a placeholder turn."
                    .to_string(),
            ),
            tool_calls_executed: 0,
            iterations: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultGovernance;
    use crate::llm_native::RawUserIntent;

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
        let governance = Arc::new(DefaultGovernance::new());
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
        let governance = Arc::new(DefaultGovernance::new());
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
        // FUNC-002, FUNC-003, FUNC-004, E2E-001: is_agent_llm_native_enabled()能读取环境变量且在合适值时识别
        // Clear env first
        std::env::remove_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED");
        assert!(!crate::prompts::is_agent_llm_native_enabled());

        // Set to 1
        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "1");
        assert!(crate::prompts::is_agent_llm_native_enabled());

        // Set to true
        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "true");
        assert!(crate::prompts::is_agent_llm_native_enabled());

        // Set to false
        std::env::set_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED", "false");
        assert!(!crate::prompts::is_agent_llm_native_enabled());

        // Cleanup
        std::env::remove_var("HAJIMI_AGENT_LLM_NATIVE_ENABLED");
    }
}
