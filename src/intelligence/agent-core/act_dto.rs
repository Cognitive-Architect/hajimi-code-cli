use crate::tool_manifest::RiskLevel;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the type of action an agent can take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Agent decides to call a tool.
    CallTool,
    /// Agent cannot act given the current context.
    CannotAct,
    /// Agent needs to ask the user a question.
    AskUser,
    /// Agent stops the current reasoning loop and hands off control.
    StopAndHandoff,
}

/// DTO representing a request to call a tool, version 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallV1 {
    /// Schema version for the tool call.
    pub schema_version: String,
    /// Type of action, typically CallTool.
    pub action_type: ActionType,
    /// The name of the tool to invoke.
    pub tool_name: String,
    /// JSON parameters for the tool.
    pub parameters: Value,
    /// The reasoning behind choosing this tool.
    pub reason: String,
    /// Expected output from the tool execution.
    pub expected_output: String,
    /// Expected evidence to look for in the output.
    pub expected_evidence: String,
    /// An optional fallback tool if this one fails.
    pub fallback_tool: Option<String>,
    /// Indicates whether governance approval is required.
    pub governance_required: bool,
    /// The risk level of this tool call.
    pub risk_level: RiskLevel,
    /// Idempotency key to prevent duplicate executions.
    pub idempotency_key: String,
    /// A hint for what the next step should be after this tool call.
    pub next_step_hint: Option<String>,
}

/// The decision made by the agent's act phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActDecision {
    /// The agent decided to call a tool.
    ToolCall(Box<ToolCallV1>),
    /// The agent cannot act.
    CannotAct {
        /// The reason why the agent cannot act.
        reason: String,
    },
    /// The agent needs to ask the user.
    AskUser {
        /// The reason for asking the user.
        reason: String,
    },
    /// The agent decides to stop and hand off control.
    StopAndHandoff {
        /// The reason for handing off.
        reason: String,
    },
}
