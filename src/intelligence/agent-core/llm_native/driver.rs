//! AgentTurnDriver trait and LlmNativeDriver skeleton (Codex-style).
//!
//! This is the core abstraction that will eventually replace the three-layer
//! rule-based planning + legacy_act path.
//!
//! Phase 1 Day 1: Only type definitions and a placeholder implementation.
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
        _intent: RawUserIntent,
        _tools: Vec<ModelVisibleToolSpec>,
        _history: Vec<TurnMessage>,
        _governance: Arc<dyn AgentGovernance>,
    ) -> crate::AgentResult<TurnOutcome> {
        // LLM-NATIVE-TODO (Phase 2): Implement the real Codex-style loop:
        // 1. Construct initial messages (system + user intent)
        // 2. Call LLM with full tools + tool_choice=auto (or equivalent)
        // 3. Parse streaming FunctionCall events
        // 4. For each call: governance.approve → execute via registry → append ToolResult
        // 5. Repeat until model returns pure text with no tool calls
        // 6. Respect cancellation, budgets, stop-loss, etc.

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
