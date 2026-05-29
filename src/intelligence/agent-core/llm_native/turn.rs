//! Turn execution loop and multi-turn scheduling (Codex-style).
//!
//! This module coordinates the conversational loop with the LLM and the execution of tools.
//! It implements the `llm_native_turn` which takes a `RawUserIntent` and executes
//! iterations of Assistant -> ToolCall -> ToolResult -> Assistant until either a terminal
//! message is received, cancellation is triggered, or maximum iterations budget is exceeded.

use crate::governance::AgentGovernance;
use crate::llm_native::{
    CancellationToken, ModelVisibleToolSpec, RawUserIntent, TurnMessage, TurnOutcome,
};
use crate::AgentResult;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait LlmStepExecutor: Send + Sync {
    /// Request the next message from the LLM based on conversation history.
    async fn step(
        &self,
        intent: &RawUserIntent,
        tools: &[ModelVisibleToolSpec],
        history: &[TurnMessage],
        cancellation: &CancellationToken,
    ) -> AgentResult<TurnMessage>;

    /// Get token usage of the last LLM step.
    fn last_usage(&self) -> Option<engine_llm_core::Usage> {
        None
    }
}

/// Trait representing the execution environment for tools.
#[async_trait]
pub trait LlmToolExecutor: Send + Sync {
    /// Execute a specific tool requested by the LLM.
    async fn execute_tool(
        &self,
        name: &str,
        arguments: &serde_json::Value,
        call_id: &str,
        governance: Arc<dyn AgentGovernance>,
        cancellation: &CancellationToken,
    ) -> AgentResult<String>;
}

/// Emit a structured trace event for auditing to the logging system.
///
/// **NEG-003**: Uses standard non-blocking tracing logs to ensure it never blocks the main execution flow.
fn emit_trace(step: &str, details: &str) {
    // UX-001: Includes "TraceEvent" and "Native" to ensure audit recognizability
    tracing::info!(
        "[TraceEvent][Native] Step: '{}', Details: '{}'",
        step,
        details
    );
}

/// Thread-safe, non-blocking O(1) Token usage accumulator.
#[derive(Debug, Default)]
pub struct TokenTracker {
    accumulated_prompt_tokens: std::sync::atomic::AtomicU64,
    accumulated_completion_tokens: std::sync::atomic::AtomicU64,
}

impl TokenTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_usage(&self, prompt: u64, completion: u64) {
        // We use relaxed ordering for O(1) lock-free thread safety without performance degradation.
        self.accumulated_prompt_tokens
            .fetch_add(prompt, std::sync::atomic::Ordering::Relaxed);
        self.accumulated_completion_tokens
            .fetch_add(completion, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn total_tokens(&self) -> u64 {
        let p = self
            .accumulated_prompt_tokens
            .load(std::sync::atomic::Ordering::Relaxed);
        let c = self
            .accumulated_completion_tokens
            .load(std::sync::atomic::Ordering::Relaxed);
        p.saturating_add(c)
    }

    pub fn prompt_tokens(&self) -> u64 {
        self.accumulated_prompt_tokens
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn completion_tokens(&self) -> u64 {
        self.accumulated_completion_tokens
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Coordinates a multi-step conversation with the LLM and tool executions.
///
/// **Invariant**: The original user intent (`intent.text`) is never modified or rewritten locally.
#[allow(clippy::too_many_arguments)]
pub async fn llm_native_turn(
    llm_executor: &dyn LlmStepExecutor,
    tool_executor: &dyn LlmToolExecutor,
    intent: RawUserIntent,
    tools: Vec<ModelVisibleToolSpec>,
    mut history: Vec<TurnMessage>,
    governance: Arc<dyn AgentGovernance>,
    cancellation: CancellationToken,
    max_iterations: usize,
) -> AgentResult<TurnOutcome> {
    let session_id = intent.session_id.clone();
    tracing::trace!(
        "llm_native_turn: Starting multi-turn loop. session_id = '{}', intent = '{}'",
        session_id,
        intent.text
    );

    let token_tracker = TokenTracker::new();
    let mut file_modifications: Vec<String> = Vec::new();

    let mut iterations = 0;
    let mut tool_calls_executed = 0;
    let mut final_message = None;
    let mut success = true;

    while iterations < max_iterations {
        if cancellation.is_cancelled() {
            tracing::trace!(
                "llm_native_turn: Cancellation detected before iteration {}.",
                iterations + 1
            );
            return Ok(TurnOutcome {
                success: false,
                final_message: Some("Cancelled".to_string()),
                tool_calls_executed,
                iterations,
                execution_history: Some(history.clone()),
            });
        }

        iterations += 1;
        tracing::trace!(
            "llm_native_turn: Iteration {}/{}. History length = {}",
            iterations,
            max_iterations,
            history.len()
        );

        let next_msg = llm_executor
            .step(&intent, &tools, &history, &cancellation)
            .await?;

        history.push(next_msg.clone());

        if let Some(usage) = llm_executor.last_usage() {
            let prompt = usage.prompt_tokens;
            let completion = usage.completion_tokens;
            token_tracker.add_usage(prompt, completion);

            tracing::trace!(
                "llm_native_turn: Token budget usage rate: {:.2}%. prompt_tokens = {}, completion_tokens = {}, total = {}",
                (token_tracker.total_tokens() as f64 / 8192.0) * 100.0,
                token_tracker.prompt_tokens(),
                token_tracker.completion_tokens(),
                token_tracker.total_tokens()
            );

            if token_tracker.total_tokens() >= 8192 {
                eprintln!("=============================================================");
                eprintln!(
                    "⚠️⚠️⚠️ [MELTDOWN WARNING] BUDGET EXCEEDED: TOKEN MELTDOWN THRESHOLD REACHED!"
                );
                eprintln!(
                    "⚠️⚠️⚠️ Total Tokens: {} >= 8192",
                    token_tracker.total_tokens()
                );
                eprintln!("⚠️⚠️⚠️ Preserving intermediate execution state and handing off...");
                eprintln!("=============================================================");
                return Ok(TurnOutcome {
                    success: false,
                    final_message: Some(format!(
                        "Handoff: Budget Exceeded. Token/Iteration meltdown threshold reached. Preserving execution state. Total tokens: {}",
                        token_tracker.total_tokens()
                    )),
                    tool_calls_executed,
                    iterations,
                    execution_history: Some(history.clone()),
                });
            }
        }

        match next_msg {
            TurnMessage::Assistant {
                content,
                tool_calls,
            } => {
                if tool_calls.is_empty() {
                    tracing::trace!(
                        "llm_native_turn: Assistant provided final content. Ending loop."
                    );
                    final_message = content;
                    break;
                } else {
                    tracing::trace!(
                        "llm_native_turn: Assistant requested {} tool call(s). Starting execution.",
                        tool_calls.len()
                    );

                    for tool_call in tool_calls {
                        if cancellation.is_cancelled() {
                            tracing::trace!(
                                "llm_native_turn: Cancellation detected during tool calls."
                            );
                            return Ok(TurnOutcome {
                                success: false,
                                final_message: Some("Cancelled".to_string()),
                                tool_calls_executed,
                                iterations,
                                execution_history: Some(history.clone()),
                            });
                        }

                        let tool_name = tool_call
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown_tool")
                            .to_string();

                        let call_id = tool_call
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown_call_id")
                            .to_string();

                        let arguments = tool_call
                            .get("arguments")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);

                        // HIGH-001: Safety check to prevent infinite self-lock/disk space exhaustion by repeated writes
                        if tool_name == "write_file" || tool_name == "append_file" {
                            if let Some(path_val) = arguments.get("path").and_then(|p| p.as_str()) {
                                file_modifications.push(path_val.to_string());
                                let count =
                                    file_modifications.iter().filter(|&p| p == path_val).count();
                                if count > 2 {
                                    let err_msg = format!(
                                        "Security Gate Alert: Potential infinite file modification self-lock detected for path '{}'. Blocking execution to prevent disk space exhaustion.",
                                        path_val
                                    );
                                    eprintln!("=============================================================");
                                    eprintln!("⚠️⚠️⚠️ [SECURITY MELTDOWN] POTENTIAL INFINITE WRITE LOCK INJECTED!");
                                    eprintln!(
                                        "⚠️⚠️⚠️ File '{}' was modified {} times in a single turn.",
                                        path_val, count
                                    );
                                    eprintln!("=============================================================");
                                    return Ok(TurnOutcome {
                                        success: false,
                                        final_message: Some(err_msg),
                                        tool_calls_executed,
                                        iterations,
                                        execution_history: Some(history.clone()),
                                    });
                                }
                            }
                        }

                        // 构造前置审查请求
                        let is_whitelisted = matches!(
                            tool_name.as_str(),
                            "git"
                                | "cargo"
                                | "npm"
                                | "node"
                                | "python3"
                                | "ls"
                                | "cat"
                                | "echo"
                                | "pwd"
                                | "rustc"
                                | "bash"
                                | "sh"
                                | "pwsh"
                                | "powershell"
                                | "curl"
                                | "wget"
                                | "tar"
                                | "unzip"
                                | "make"
                        );

                        let risk_score = if is_whitelisted { 0.1 } else { 0.95 };
                        let approval_level = if is_whitelisted {
                            crate::governance::ApprovalLevel::Auto
                        } else {
                            crate::governance::ApprovalLevel::Critical
                        };

                        let req = crate::governance::GovernanceRequest {
                            requester: intent.session_id.clone(),
                            action_type: tool_name.clone(),
                            risk_score,
                            description: format!(
                                "LLM-Native turn execution of tool '{}' with arguments: {:?}",
                                tool_name, arguments
                            ),
                            level: approval_level,
                        };

                        // FUNC-001: 在工具执行动作前，显式触发 governance.approve 审查拦截
                        let ctx = crate::AgentContext::new();
                        emit_trace(
                            "ToolCallInitiated",
                            &format!(
                                "Tool '{}' requested. Initiating governance approval gate.",
                                tool_name
                            ),
                        );

                        let tool_result_str = match governance.approve(&ctx, &req).await {
                            Ok(crate::governance::Decision::Approved) => {
                                emit_trace(
                                    "GovernanceApproved",
                                    &format!("Tool '{}' approval granted.", tool_name),
                                );

                                // 执行工具
                                match tool_executor
                                    .execute_tool(
                                        &tool_name,
                                        &arguments,
                                        &call_id,
                                        governance.clone(),
                                        &cancellation,
                                    )
                                    .await
                                {
                                    Ok(res) => {
                                        emit_trace(
                                            "ToolExecutionSuccess",
                                            &format!("Tool '{}' executed successfully.", tool_name),
                                        );
                                        res
                                    }
                                    Err(e) => {
                                        emit_trace(
                                            "ToolExecutionFailed",
                                            &format!("Tool '{}' failed: {:?}", tool_name, e),
                                        );
                                        format!("Error: {:?}", e)
                                    }
                                }
                            }
                            Ok(crate::governance::Decision::Rejected(reason)) => {
                                // FUNC-002: 当 governance 拒绝通过时，流程立刻安全退避阻断
                                // FUNC-004: 支持新 native_turn 鉴权被阻断时的专用告警码上报
                                // UX-002: 权限被拒时的报错文案高度人性化
                                let err_msg = format!(
                                    "Security Gate Alert: Permission Denied! Action [execute_tool: {}] was rejected by the Governance policy. Reason: {}. Risk score evaluated: {}",
                                    tool_name, reason, risk_score
                                );
                                emit_trace(
                                    "GovernanceRejected",
                                    &format!(
                                        "Tool '{}' execution blocked. Details: {}",
                                        tool_name, err_msg
                                    ),
                                );

                                // NEG-001: 循环能接住并转化为安全 outcome 返回
                                return Ok(TurnOutcome {
                                    success: false,
                                    final_message: Some(err_msg),
                                    tool_calls_executed,
                                    iterations,
                                    execution_history: Some(history.clone()),
                                });
                            }
                            Ok(other_decision) => {
                                let err_msg = format!(
                                    "Security Blocked: Action [execute_tool: {}] did not receive Auto or Admin approval. Current decision status: {:?}",
                                    tool_name, other_decision
                                );
                                emit_trace(
                                    "GovernanceBlocked",
                                    &format!(
                                        "Tool '{}' execution was blocked: {:?}",
                                        tool_name, other_decision
                                    ),
                                );

                                return Ok(TurnOutcome {
                                    success: false,
                                    final_message: Some(err_msg),
                                    tool_calls_executed,
                                    iterations,
                                    execution_history: Some(history.clone()),
                                });
                            }
                            Err(e) => {
                                let err_msg =
                                    format!("Governance Internal Error during approval: {:?}", e);
                                emit_trace("GovernanceError", &err_msg);
                                return Ok(TurnOutcome {
                                    success: false,
                                    final_message: Some(err_msg),
                                    tool_calls_executed,
                                    iterations,
                                    execution_history: Some(history.clone()),
                                });
                            }
                        };

                        tool_calls_executed += 1;

                        let tool_msg = TurnMessage::ToolResult {
                            tool_name: tool_name.clone(),
                            call_id: call_id.clone(),
                            result: tool_result_str,
                        };
                        history.push(tool_msg);
                    }
                }
            }
            _ => {
                tracing::trace!(
                    "llm_native_turn: Error. Expected Assistant message but got other."
                );
                return Err(crate::ports::AgentError::Internal(
                    "LLM step returned non-Assistant message".to_string(),
                ));
            }
        }
    }

    if iterations >= max_iterations && final_message.is_none() {
        tracing::trace!(
            "llm_native_turn: Maximum iteration budget of {} exceeded.",
            max_iterations
        );
        eprintln!("=============================================================");
        eprintln!(
            "⚠️⚠️⚠️ [MELTDOWN WARNING] BUDGET EXCEEDED: ITERATION MELTDOWN THRESHOLD REACHED!"
        );
        eprintln!("⚠️⚠️⚠️ Max iterations: {}", max_iterations);
        eprintln!("⚠️⚠️⚠️ Preserving intermediate execution state and handing off...");
        eprintln!("=============================================================");
        success = false;
        final_message = Some(format!(
            "Handoff: Budget Exceeded. Token/Iteration meltdown threshold reached. Preserving execution state. Max iterations exceeded: {}",
            max_iterations
        ));
    }

    Ok(TurnOutcome {
        success,
        final_message,
        tool_calls_executed,
        iterations,
        execution_history: Some(history),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_native::RawUserIntent;
    use chimera_repl::traits::ReplResult;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockApprovedGovernance;

    #[async_trait]
    impl crate::governance::AgentGovernance for MockApprovedGovernance {
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

    // A mock LlmStepExecutor that returns simulated assistant responses.
    pub struct MockLlmStepExecutor {
        steps: Vec<TurnMessage>,
        current_step: AtomicUsize,
        simulated_usage: Option<engine_llm_core::Usage>,
    }

    impl MockLlmStepExecutor {
        pub fn new(steps: Vec<TurnMessage>) -> Self {
            Self {
                steps,
                current_step: AtomicUsize::new(0),
                simulated_usage: None,
            }
        }

        pub fn with_simulated_usage(
            steps: Vec<TurnMessage>,
            usage: engine_llm_core::Usage,
        ) -> Self {
            Self {
                steps,
                current_step: AtomicUsize::new(0),
                simulated_usage: Some(usage),
            }
        }
    }

    #[async_trait]
    impl LlmStepExecutor for MockLlmStepExecutor {
        async fn step(
            &self,
            _intent: &RawUserIntent,
            _tools: &[ModelVisibleToolSpec],
            _history: &[TurnMessage],
            _cancellation: &CancellationToken,
        ) -> AgentResult<TurnMessage> {
            let idx = self.current_step.fetch_add(1, Ordering::SeqCst);
            if idx < self.steps.len() {
                Ok(self.steps[idx].clone())
            } else {
                Err(crate::ports::AgentError::Internal(
                    "Mock LLM ran out of steps".to_string(),
                ))
            }
        }

        fn last_usage(&self) -> Option<engine_llm_core::Usage> {
            self.simulated_usage
        }
    }

    // A mock LlmToolExecutor that simply logs and returns predefined results.
    pub struct MockLlmToolExecutor {
        result_map: std::collections::HashMap<String, String>,
    }

    impl MockLlmToolExecutor {
        pub fn new(results: Vec<(&str, &str)>) -> Self {
            let mut result_map = std::collections::HashMap::new();
            for (k, v) in results {
                result_map.insert(k.to_string(), v.to_string());
            }
            Self { result_map }
        }
    }

    #[async_trait]
    impl LlmToolExecutor for MockLlmToolExecutor {
        async fn execute_tool(
            &self,
            name: &str,
            _arguments: &serde_json::Value,
            _call_id: &str,
            _governance: Arc<dyn AgentGovernance>,
            _cancellation: &CancellationToken,
        ) -> AgentResult<String> {
            if let Some(res) = self.result_map.get(name) {
                Ok(res.clone())
            } else {
                Ok(format!("Executed {} successfully", name))
            }
        }
    }

    #[tokio::test]
    async fn test_llm_native_turn_e2e_closed_loop() {
        // E2E-001 & FUNC-003: 验证三轮 (User -> ToolCall -> ToolResult -> Assistant) 的完整循环顺利闭环
        // Step 1: Assistant requests a tool call to write a file
        let step1 = TurnMessage::Assistant {
            content: Some("I need to write a file first.".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "write_file",
                "id": "call_1",
                "arguments": {
                    "path": "test.txt",
                    "content": "hello"
                }
            })],
        };

        // Step 2: Assistant receives the ToolResult and then gives the final answer
        let step2 = TurnMessage::Assistant {
            content: Some("File written. Task completed successfully!".to_string()),
            tool_calls: vec![],
        };

        let llm = MockLlmStepExecutor::new(vec![step1, step2]);
        let tools = MockLlmToolExecutor::new(vec![("write_file", "File written successfully")]);

        let intent = RawUserIntent::from_text("创建一个文件并写入内容", "session_123");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            10,
        )
        .await
        .expect("llm_native_turn failed");

        assert!(outcome.success);
        assert_eq!(outcome.iterations, 2);
        assert_eq!(outcome.tool_calls_executed, 1);
        assert_eq!(
            outcome.final_message.unwrap(),
            "File written. Task completed successfully!"
        );
    }

    #[tokio::test]
    async fn test_llm_native_turn_max_iterations_exceeded() {
        // FUNC-002 & NEG-001: 最大迭代上限 max_iterations 生效，防止死循环并且安全退出
        let step = TurnMessage::Assistant {
            content: Some("Let's keep looping...".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "loop_tool",
                "id": "loop_call",
                "arguments": {}
            })],
        };

        // Generate an infinite stream of step calls
        let llm =
            MockLlmStepExecutor::new(vec![step.clone(), step.clone(), step.clone(), step.clone()]);
        let tools = MockLlmToolExecutor::new(vec![]);

        let intent = RawUserIntent::from_text("Loop forever", "session_loop");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            3, // Strict limit of 3 iterations
        )
        .await
        .expect("llm_native_turn failed");

        assert!(!outcome.success);
        assert_eq!(outcome.iterations, 3);
        assert_eq!(outcome.tool_calls_executed, 3);
        assert!(outcome
            .final_message
            .unwrap()
            .contains("Max iterations exceeded"));
    }

    #[tokio::test]
    async fn test_llm_native_turn_cancellation() {
        // FUNC-004: 支持通过 CancellationToken 进行优雅的运行中强行退出
        let step = TurnMessage::Assistant {
            content: Some("Working...".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "long_tool",
                "id": "call_long",
                "arguments": {}
            })],
        };

        let llm = MockLlmStepExecutor::new(vec![step]);
        let tools = MockLlmToolExecutor::new(vec![]);

        let intent = RawUserIntent::from_text("Long task", "session_cancel");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        // Cancel it beforehand
        cancellation.cancel();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            5,
        )
        .await
        .expect("llm_native_turn failed");

        assert!(!outcome.success);
        assert_eq!(outcome.iterations, 0);
        assert_eq!(outcome.tool_calls_executed, 0);
        assert_eq!(outcome.final_message.unwrap(), "Cancelled");
    }

    #[tokio::test]
    async fn test_llm_native_turn_non_ascii_intent() {
        // NEG-002: 传入非 ASCII 中文特殊自然语言意图，整个循环依然完整运转
        let step = TurnMessage::Assistant {
            content: Some("你好，有什么我可以帮您的吗？".to_string()),
            tool_calls: vec![],
        };

        let llm = MockLlmStepExecutor::new(vec![step]);
        let tools = MockLlmToolExecutor::new(vec![]);

        let intent = RawUserIntent::from_text("你好，测试一下中文！", "session_zh");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            5,
        )
        .await
        .expect("llm_native_turn failed");

        assert!(outcome.success);
        assert_eq!(outcome.iterations, 1);
        assert_eq!(outcome.tool_calls_executed, 0);
        assert_eq!(
            outcome.final_message.unwrap(),
            "你好，有什么我可以帮您的吗？"
        );
    }

    struct MockRejectedGovernance;

    #[async_trait]
    impl crate::governance::AgentGovernance for MockRejectedGovernance {
        async fn policy(
            &self,
            _ctx: &crate::AgentContext,
            _req: &crate::governance::GovernanceRequest,
        ) -> crate::governance::ApprovalLevel {
            crate::governance::ApprovalLevel::Critical
        }
        async fn approve(
            &self,
            _ctx: &crate::AgentContext,
            _req: &crate::governance::GovernanceRequest,
        ) -> ReplResult<crate::governance::Decision> {
            Ok(crate::governance::Decision::Rejected(
                "highly restricted command".to_string(),
            ))
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

    #[tokio::test]
    async fn test_native_governance_block() {
        // E2E-001 & FUNC-002 & NEG-001: 验证治理审批拦截拒绝通过时，流程立刻退避并返回拒绝结果
        let step = TurnMessage::Assistant {
            content: Some("I need to write a file.".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "write_file",
                "id": "call_1",
                "arguments": {
                    "path": "test.txt",
                    "content": "hello"
                }
            })],
        };

        let llm = MockLlmStepExecutor::new(vec![step]);
        let tools = MockLlmToolExecutor::new(vec![]);

        let intent = RawUserIntent::from_text("Write file", "session_governance_test");
        let governance = Arc::new(MockRejectedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            5,
        )
        .await
        .expect("llm_native_turn failed");

        assert!(!outcome.success);
        assert_eq!(outcome.tool_calls_executed, 0); // 应该前置拦截，没有投递工具执行
        let err_msg = outcome.final_message.unwrap();
        assert!(err_msg.contains("Security Gate Alert: Permission Denied!"));
        assert!(err_msg.contains("highly restricted command"));
    }

    #[tokio::test]
    async fn test_llm_native_turn_token_limit_meltdown() {
        // E2E-001 & FUNC-002: 集成用例模拟多轮超支，证明系统在指定上限帧处强行退避成功，且中间态执行历史被安全保留
        let step1 = TurnMessage::Assistant {
            content: Some("Thinking...".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "ls",
                "id": "call_1",
                "arguments": {}
            })],
        };
        let step2 = TurnMessage::Assistant {
            content: Some("Done".to_string()),
            tool_calls: vec![],
        };

        // Simulated usage is 8500 prompt tokens (over 8192)
        let simulated_usage = engine_llm_core::Usage {
            prompt_tokens: 8500,
            completion_tokens: 100,
        };
        let llm = MockLlmStepExecutor::with_simulated_usage(vec![step1, step2], simulated_usage);
        let tools = MockLlmToolExecutor::new(vec![("ls", "file1.txt\nfile2.txt")]);

        let intent = RawUserIntent::from_text("List files", "session_token_meltdown");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            5,
        )
        .await
        .expect("llm_native_turn failed");

        assert!(!outcome.success);
        let final_msg = outcome.final_message.unwrap();
        assert!(
            final_msg.contains("Budget Meltdown Alert") || final_msg.contains("Budget Exceeded")
        );

        // Assert that intermediate execution history was preserved
        let history = outcome
            .execution_history
            .expect("Expected intermediate execution history");
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_llm_native_turn_preventative_file_modification_lock() {
        // HIGH-001: 校验防范由于多轮循环中反复修改文件引发的无限自锁与磁盘占满长尾问题
        let step1 = TurnMessage::Assistant {
            content: Some("First write".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "write_file",
                "id": "call_1",
                "arguments": {
                    "path": "inf.txt",
                    "content": "a"
                }
            })],
        };
        let step2 = TurnMessage::Assistant {
            content: Some("Second write".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "write_file",
                "id": "call_2",
                "arguments": {
                    "path": "inf.txt",
                    "content": "b"
                }
            })],
        };
        let step3 = TurnMessage::Assistant {
            content: Some("Third write".to_string()),
            tool_calls: vec![serde_json::json!({
                "name": "write_file",
                "id": "call_3",
                "arguments": {
                    "path": "inf.txt",
                    "content": "c"
                }
            })],
        };

        let llm = MockLlmStepExecutor::new(vec![step1, step2, step3]);
        let tools = MockLlmToolExecutor::new(vec![
            ("write_file", "success"),
            ("write_file", "success"),
            ("write_file", "success"),
        ]);

        let intent = RawUserIntent::from_text("Write repeatedly", "session_write_lock");
        let governance = Arc::new(MockApprovedGovernance);
        let cancellation = CancellationToken::new();

        let outcome = llm_native_turn(
            &llm,
            &tools,
            intent,
            vec![],
            vec![],
            governance,
            cancellation,
            5,
        )
        .await
        .expect("llm_native_turn failed");

        assert!(!outcome.success);
        let final_msg = outcome.final_message.unwrap();
        assert!(final_msg.contains("Potential infinite file modification self-lock detected"));
        assert_eq!(outcome.tool_calls_executed, 2); // Third one is blocked before execution!
    }
}
