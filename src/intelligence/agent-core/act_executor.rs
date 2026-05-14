use crate::act_dto::{ActDecision, ToolCallV1};
use crate::blackboard::Blackboard;
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::tool_manifest::RiskLevel;
use crate::{AgentContext, AgentId};
use engine_llm_core::LlmClient;
use engine_tool_system::{ToolError, ToolOutput, ToolRegistry};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::Mutex;

/// BB_NEXT_TOOL: Stores the serialized next ToolCallV1, or a next-step hint.
pub(crate) const BB_NEXT_TOOL: &str = "__hajimi_act_next_tool";
/// BB_LAST_TOOL: Stores the last tool name attempted by ActExecutor.
pub(crate) const BB_LAST_TOOL: &str = "__hajimi_act_last_tool";
/// BB_LAST_TOOL_RESULT: Stores the last successful tool output summary.
pub(crate) const BB_LAST_TOOL_RESULT: &str = "__hajimi_act_last_tool_result";
/// BB_LAST_ERROR: Stores the latest current-tool error and micro-reflect summary.
pub(crate) const BB_LAST_ERROR: &str = "__hajimi_act_last_error";
/// BB_FAILED_TOOL_FINGERPRINT: Stores the latest failed tool + parameters fingerprint.
pub(crate) const BB_FAILED_TOOL_FINGERPRINT: &str = "__hajimi_act_failed_tool_fingerprint";
/// BB_ATTEMPT_COUNT: Stores the consecutive corrected-argument failure count.
pub(crate) const BB_ATTEMPT_COUNT: &str = "__hajimi_act_attempt_count";

/// The component responsible for executing actions decided by the agent.
pub struct ActExecutor {
    tool_registry: Arc<Mutex<ToolRegistry>>,
    governance: Arc<dyn AgentGovernance>,
}

#[derive(Debug, Clone)]
pub struct ActChainResult {
    pub success: bool,
    pub output: String,
    pub decision: ActDecision,
}

struct RetryPolicy {
    failed_fingerprint: Option<String>,
    attempt_count: usize,
}

enum RetryDecision {
    Execute,
    StopAndHandoff(String),
}

impl RetryPolicy {
    async fn from_blackboard(blackboard: &Blackboard) -> Self {
        let failed_fingerprint = blackboard
            .read(BB_FAILED_TOOL_FINGERPRINT)
            .await
            .map(|entry| entry.value);
        let attempt_count = blackboard
            .read(BB_ATTEMPT_COUNT)
            .await
            .and_then(|entry| entry.value.parse::<usize>().ok())
            .unwrap_or(0);
        Self {
            failed_fingerprint,
            attempt_count,
        }
    }

    fn should_retry(&self, fingerprint: &str) -> RetryDecision {
        // retry/fingerprint guard: the exact same tool + parameters are already tried.
        let fingerprint_match = self
            .failed_fingerprint
            .as_deref()
            .map(|failed| failed == fingerprint)
            .unwrap_or(false);
        if fingerprint_match {
            return RetryDecision::StopAndHandoff(format!(
                "Tool call already tried with identical fingerprint: {}",
                fingerprint
            ));
        }
        // retry/attempt guard: corrected parameters get one retry, then handoff.
        if self.attempt_count >= 2 {
            return RetryDecision::StopAndHandoff(format!(
                "StopAndHandoff after {} failed corrected attempts",
                self.attempt_count
            ));
        }
        RetryDecision::Execute
    }
}

impl ActExecutor {
    /// Creates a new ActExecutor.
    pub fn new(
        tool_registry: Arc<Mutex<ToolRegistry>>,
        governance: Arc<dyn AgentGovernance>,
    ) -> Self {
        Self {
            tool_registry,
            governance,
        }
    }

    /// Validates a tool call, routes through governance if critical, and executes it.
    ///
    /// # Errors
    /// Returns a ToolError if the tool is not found in the registry, if the parameters
    /// are not valid JSON, or if the tool execution itself fails. It does not panic.
    pub async fn execute_tool_call(
        &self,
        ctx: &AgentContext,
        call: &ToolCallV1,
    ) -> Result<ToolOutput, ToolError> {
        // Validation: Verify parameters are valid JSON object
        let args = if let serde_json::Value::Object(map) = &call.parameters {
            map.clone()
        } else {
            return Err(ToolError::new(
                "Tool parameters must be a valid JSON object",
            ));
        };

        // Governance routing: check if required or risk level is critical
        if call.governance_required || matches!(call.risk_level, RiskLevel::Critical) {
            let req = GovernanceRequest {
                requester: "act_executor".to_string(),
                action_type: format!("invoke_tool:{}", call.tool_name),
                risk_score: 1.0, // Critical implies high risk score
                description: format!("Agent requests to invoke {}", call.tool_name),
                level: ApprovalLevel::Critical,
            };
            let decision = self.governance.approve(ctx, &req).await;
            match decision {
                Ok(Decision::Approved) => {}
                Ok(Decision::Rejected(reason)) => {
                    return Err(ToolError::new(format!(
                        "Tool invocation rejected by governance: {}",
                        reason
                    )));
                }
                Ok(Decision::Escalated(level)) => {
                    return Err(ToolError::new(format!(
                        "Tool invocation escalated by governance: {:?}",
                        level
                    )));
                }
                Ok(Decision::Timeout) => {
                    return Err(ToolError::new("Tool invocation governance timed out"));
                }
                Err(e) => {
                    return Err(ToolError::new(format!("Governance error: {}", e)));
                }
            }
        }

        // Tool execution
        let registry = self.tool_registry.lock().await;
        if let Some(tool) = registry.get(&call.tool_name) {
            tool.execute(serde_json::Value::Object(args)).await
        } else {
            Err(ToolError::new(format!(
                "Tool not found: {}",
                call.tool_name
            )))
        }
    }

    /// Executes a ToolCallV1 using the blackboard chain protocol and retry policy.
    pub async fn execute_chain(
        &self,
        ctx: &AgentContext,
        blackboard: &Blackboard,
        agent_id: &AgentId,
        call: &ToolCallV1,
    ) -> ActChainResult {
        let fingerprint = fingerprint_tool_call(call);
        let policy = RetryPolicy::from_blackboard(blackboard).await;
        if let RetryDecision::StopAndHandoff(reason) = policy.should_retry(&fingerprint) {
            blackboard.write(BB_LAST_ERROR, &reason, agent_id).await;
            return ActChainResult {
                success: false,
                output: reason.clone(),
                decision: ActDecision::StopAndHandoff { reason },
            };
        }

        blackboard
            .write(BB_LAST_TOOL, &call.tool_name, agent_id)
            .await;
        match self.execute_tool_call(ctx, call).await {
            Ok(output) => {
                self.record_success(blackboard, agent_id, call, &output)
                    .await;
                ActChainResult {
                    success: true,
                    output: output_summary(&output),
                    decision: ActDecision::ToolCall(Box::new(call.clone())),
                }
            }
            Err(error) => {
                let attempt_count = policy.attempt_count + 1;
                self.record_failure(
                    blackboard,
                    agent_id,
                    call,
                    &fingerprint,
                    attempt_count,
                    &error,
                )
                .await;
                let reflected = micro_reflect_tool_error(call, &error);
                if attempt_count >= 2 {
                    ActChainResult {
                        success: false,
                        output: reflected.clone(),
                        decision: ActDecision::StopAndHandoff { reason: reflected },
                    }
                } else {
                    ActChainResult {
                        success: false,
                        output: reflected.clone(),
                        decision: ActDecision::CannotAct { reason: reflected },
                    }
                }
            }
        }
    }

    async fn record_success(
        &self,
        blackboard: &Blackboard,
        agent_id: &AgentId,
        call: &ToolCallV1,
        output: &ToolOutput,
    ) {
        let summary = output_summary(output);
        blackboard
            .write(BB_LAST_TOOL_RESULT, &summary, agent_id)
            .await;
        blackboard.write(BB_ATTEMPT_COUNT, "0", agent_id).await;
        blackboard
            .write(BB_FAILED_TOOL_FINGERPRINT, "", agent_id)
            .await;
        if let Some(next_step_hint) = &call.next_step_hint {
            blackboard
                .write(BB_NEXT_TOOL, next_step_hint, agent_id)
                .await;
        }
    }

    async fn record_failure(
        &self,
        blackboard: &Blackboard,
        agent_id: &AgentId,
        call: &ToolCallV1,
        fingerprint: &str,
        attempt_count: usize,
        error: &ToolError,
    ) {
        let reflected = micro_reflect_tool_error(call, error);
        blackboard.write(BB_LAST_ERROR, &reflected, agent_id).await;
        blackboard
            .write(BB_FAILED_TOOL_FINGERPRINT, fingerprint, agent_id)
            .await;
        blackboard
            .write(BB_ATTEMPT_COUNT, &attempt_count.to_string(), agent_id)
            .await;
        if let Some(fallback_tool) = &call.fallback_tool {
            let mut retry_call = call.clone();
            retry_call.tool_name = fallback_tool.clone();
            retry_call.idempotency_key = format!("{}:fallback", call.idempotency_key);
            if let Ok(json) = serde_json::to_string(&retry_call) {
                blackboard.write(BB_NEXT_TOOL, &json, agent_id).await;
            }
        }
    }
}

fn fingerprint_tool_call(call: &ToolCallV1) -> String {
    let params =
        serde_json::to_string(&call.parameters).unwrap_or_else(|_| call.parameters.to_string());
    let mut hasher = DefaultHasher::new();
    call.tool_name.hash(&mut hasher);
    params.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn micro_reflect_tool_error(call: &ToolCallV1, error: &ToolError) -> String {
    format!(
        "micro_reflect: tool={} failed; error={}; next=fix current tool arguments or handoff",
        call.tool_name, error.message
    )
}

fn output_summary(output: &ToolOutput) -> String {
    let mut parts = Vec::new();
    if !output.stdout.is_empty() {
        parts.push(format!("stdout={}", output.stdout));
    }
    if !output.stderr.is_empty() {
        parts.push(format!("stderr={}", output.stderr));
    }
    if let Some(code) = output.exit_code {
        parts.push(format!("exit_code={}", code));
    }
    if parts.is_empty() {
        "tool completed with empty output".to_string()
    } else {
        parts.join("; ")
    }
}

/// A bridge to an LLM specifically for deciding the next action (Act decision).
pub struct ActLlmBridge {
    #[allow(dead_code)]
    client: Arc<dyn LlmClient>,
}

impl ActLlmBridge {
    /// Creates a new ActLlmBridge.
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    /// Asks the LLM to decide on the next action.
    /// The prompt instructs the LLM to return ONLY valid JSON matching ToolCallV1 or ActDecision.
    pub async fn llm_decide(&self, prompt_context: &str) -> Result<ActDecision, String> {
        // Placeholder for actual LLM call. The LLM must be instructed properly:
        // "You must return ONLY valid JSON matching the ToolCallV1 schema..."
        Err(format!(
            "LLM bridge not fully implemented. Context: {}",
            prompt_context
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::act_dto::ActionType;
    use crate::governance::DefaultGovernance;
    use async_trait::async_trait;
    use engine_tool_system::{Config, Tool, ToolArgs, ToolPermissions};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingFailTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for CountingFailTool {
        fn name(&self) -> &str {
            "fail_tool"
        }

        fn description(&self) -> &str {
            "fails for retry tests"
        }

        fn permissions(&self) -> ToolPermissions {
            ToolPermissions::default()
        }

        fn is_enabled(&self, config: &Config) -> bool {
            config.enabled_tools.is_empty()
                || config.enabled_tools.contains(&self.name().to_string())
        }

        async fn execute(&self, _args: ToolArgs) -> Result<ToolOutput, ToolError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(ToolError::new("synthetic failure"))
        }
    }

    struct SuccessTool;

    #[async_trait]
    impl Tool for SuccessTool {
        fn name(&self) -> &str {
            "success_tool"
        }

        fn description(&self) -> &str {
            "succeeds for chain tests"
        }

        fn permissions(&self) -> ToolPermissions {
            ToolPermissions::default()
        }

        async fn execute(&self, _args: ToolArgs) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::success("ok"))
        }
    }

    fn tool_call(tool_name: &str, parameters: serde_json::Value) -> ToolCallV1 {
        ToolCallV1 {
            schema_version: "1".to_string(),
            action_type: ActionType::CallTool,
            tool_name: tool_name.to_string(),
            parameters,
            reason: "test".to_string(),
            expected_output: "output".to_string(),
            expected_evidence: "evidence".to_string(),
            fallback_tool: None,
            governance_required: false,
            risk_level: RiskLevel::Low,
            idempotency_key: format!("{}-key", tool_name),
            next_step_hint: None,
        }
    }

    #[tokio::test]
    async fn test_same_fingerprint_is_not_repeated() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(CountingFailTool {
            calls: calls.clone(),
        }));
        let executor = ActExecutor::new(
            Arc::new(Mutex::new(registry)),
            Arc::new(DefaultGovernance::new()),
        );
        let blackboard = Blackboard::new();
        let agent_id = "agent1".to_string();
        let call = tool_call("fail_tool", json!({"path":"a"}));

        let first = executor
            .execute_chain(&AgentContext::new(), &blackboard, &agent_id, &call)
            .await;
        let second = executor
            .execute_chain(&AgentContext::new(), &blackboard, &agent_id, &call)
            .await;

        assert!(!first.success);
        assert!(matches!(
            second.decision,
            ActDecision::StopAndHandoff { .. }
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_corrected_args_retry_once_then_handoff() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(CountingFailTool {
            calls: calls.clone(),
        }));
        let executor = ActExecutor::new(
            Arc::new(Mutex::new(registry)),
            Arc::new(DefaultGovernance::new()),
        );
        let blackboard = Blackboard::new();
        let agent_id = "agent1".to_string();

        let first = executor
            .execute_chain(
                &AgentContext::new(),
                &blackboard,
                &agent_id,
                &tool_call("fail_tool", json!({"path":"a"})),
            )
            .await;
        let second = executor
            .execute_chain(
                &AgentContext::new(),
                &blackboard,
                &agent_id,
                &tool_call("fail_tool", json!({"path":"b"})),
            )
            .await;

        assert!(matches!(first.decision, ActDecision::CannotAct { .. }));
        assert!(matches!(
            second.decision,
            ActDecision::StopAndHandoff { .. }
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        assert_eq!(
            blackboard
                .read(BB_ATTEMPT_COUNT)
                .await
                .map(|entry| entry.value),
            Some("2".to_string())
        );
    }

    #[tokio::test]
    async fn test_success_writes_chain_blackboard_state() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(SuccessTool));
        let executor = ActExecutor::new(
            Arc::new(Mutex::new(registry)),
            Arc::new(DefaultGovernance::new()),
        );
        let blackboard = Blackboard::new();
        let agent_id = "agent1".to_string();

        let result = executor
            .execute_chain(
                &AgentContext::new(),
                &blackboard,
                &agent_id,
                &tool_call("success_tool", json!({"path":"a"})),
            )
            .await;

        assert!(result.success);
        assert_eq!(
            blackboard.read(BB_LAST_TOOL).await.map(|entry| entry.value),
            Some("success_tool".to_string())
        );
        assert!(
            blackboard.read(BB_LAST_TOOL_RESULT).await.is_some(),
            "successful tool result should be written"
        );
    }
}
