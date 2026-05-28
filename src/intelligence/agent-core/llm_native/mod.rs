//! LLM-Native Agent primitives (Codex-style).
//!
//! This module introduces the core abstractions for the LLM-Native execution path:
//! - RawUserIntent: carries the user's original input without any local rule rewriting.
//! - ModelVisibleToolSpec: tool specifications exposed to the model (tool_choice=auto equivalent).
//! - AgentTurnDriver / LlmNativeDriver: the driver that lets the LLM directly control tool calls.
//!
//! All code in this module and its consumers on the new path MUST preserve the invariant:
//! **original user intent reaches the LLM unchanged**.

pub mod driver;
pub mod intent;
pub mod tool_spec;

pub use driver::{
    AgentTurnDriver, CancellationToken, LlmNativeDriver, ToolChoiceMode, TurnMessage, TurnOutcome,
};
pub use intent::{IntentAttachment, RawUserIntent};
pub use tool_spec::{ModelVisibleToolSpec, RiskLevel, ToolSpecExporter};
