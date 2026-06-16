use crate::state::{await_ui_approval_response, AppState};
use agent_core::TraceEvent;
use async_trait::async_trait;
use engine_tool_system::ToolRegistry;
use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;

const APPROVAL_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

fn sanitize_description(s: &str) -> String {
    let re_assignment = regex::Regex::new(r#"(?i)(key|secret|password|token|auth|credential)\s*[:=]\s*['"]?[a-zA-Z0-9_\-\.]{8,}['"]?"#).unwrap();
    let redacted = re_assignment.replace_all(s, "$1=[REDACTED]");

    let re_sk = regex::Regex::new(r#"(?i)sk-[a-zA-Z0-9_\-\.]{12,}"#).unwrap();
    let redacted_sk = re_sk.replace_all(&redacted, "[REDACTED_KEY]");

    redacted_sk.to_string()
}

/// Desktop LLM-Native Agent Turn Driver with dynamic late-binding.
pub struct DesktopAgentTurnDriver {
    pub agent_llm_client: Arc<tokio::sync::RwLock<Option<Arc<dyn engine_llm_core::LlmClient>>>>,
    pub tool_registry: Arc<tokio::sync::Mutex<ToolRegistry>>,
}

#[async_trait]
impl agent_core::llm_native::AgentTurnDriver for DesktopAgentTurnDriver {
    async fn run_turn(
        &self,
        intent: agent_core::llm_native::RawUserIntent,
        tools: Vec<agent_core::llm_native::ModelVisibleToolSpec>,
        history: Vec<agent_core::llm_native::TurnMessage>,
        governance: Arc<dyn agent_core::governance::AgentGovernance>,
        cancellation: agent_core::llm_native::CancellationToken,
    ) -> agent_core::AgentResult<agent_core::llm_native::TurnOutcome> {
        let client = self.agent_llm_client.read().await.clone().ok_or_else(|| {
            agent_core::AgentError::Session(
                "LLM provider not configured. Please select a provider and configure API key before using /agent.".to_string()
            )
        })?;

        let driver = agent_core::llm_native::LlmNativeDriver::with_client(client)
            .with_registry(self.tool_registry.clone());

        driver
            .run_turn(intent, tools, history, governance, cancellation)
            .await
    }

    async fn run_turn_with_trace(
        &self,
        intent: agent_core::llm_native::RawUserIntent,
        tools: Vec<agent_core::llm_native::ModelVisibleToolSpec>,
        history: Vec<agent_core::llm_native::TurnMessage>,
        governance: Arc<dyn agent_core::governance::AgentGovernance>,
        cancellation: agent_core::llm_native::CancellationToken,
        trace_tx: Option<tokio::sync::broadcast::Sender<TraceEvent>>,
    ) -> agent_core::AgentResult<agent_core::llm_native::TurnOutcome> {
        let client = self.agent_llm_client.read().await.clone().ok_or_else(|| {
            agent_core::AgentError::Session(
                "LLM provider not configured. Please select a provider and configure API key before using /agent.".to_string()
            )
        })?;

        let driver = agent_core::llm_native::LlmNativeDriver::with_client(client)
            .with_registry(self.tool_registry.clone());

        driver
            .run_turn_with_trace(intent, tools, history, governance, cancellation, trace_tx)
            .await
    }
}

pub struct UiBridgeGovernance {
    pub inner: Arc<agent_core::governance::DefaultGovernance>,
    pub app_handle: tauri::AppHandle,
}

#[async_trait]
impl agent_core::governance::AgentGovernance for UiBridgeGovernance {
    async fn policy(
        &self,
        ctx: &agent_core::AgentContext,
        req: &agent_core::governance::GovernanceRequest,
    ) -> agent_core::governance::ApprovalLevel {
        if let Some(state) = self.app_handle.try_state::<AppState>() {
            let app_level_str = state.approval_level.lock().unwrap().clone();
            let app_level = match app_level_str.as_str() {
                "Auto" => agent_core::governance::ApprovalLevel::Auto,
                "Advisory" => agent_core::governance::ApprovalLevel::Advisory,
                "Required" => agent_core::governance::ApprovalLevel::Required,
                "Critical" => agent_core::governance::ApprovalLevel::Critical,
                "Override" => agent_core::governance::ApprovalLevel::Override,
                _ => agent_core::governance::ApprovalLevel::Auto,
            };

            fn level_val(l: agent_core::governance::ApprovalLevel) -> u32 {
                match l {
                    agent_core::governance::ApprovalLevel::Auto => 0,
                    agent_core::governance::ApprovalLevel::Advisory => 1,
                    agent_core::governance::ApprovalLevel::Required => 2,
                    agent_core::governance::ApprovalLevel::Critical => 3,
                    agent_core::governance::ApprovalLevel::Override => 4,
                }
            }
            if level_val(app_level) > level_val(req.level) {
                return app_level;
            }
        }
        self.inner.policy(ctx, req).await
    }

    async fn approve(
        &self,
        ctx: &agent_core::AgentContext,
        req: &agent_core::governance::GovernanceRequest,
    ) -> chimera_repl::traits::ReplResult<agent_core::governance::Decision> {
        let level = self.policy(ctx, req).await;
        if level == agent_core::governance::ApprovalLevel::Required
            || level == agent_core::governance::ApprovalLevel::Critical
        {
            if let Some(state) = self.app_handle.try_state::<AppState>() {
                let request_id = uuid::Uuid::new_v4().to_string();
                let (tx, rx) = tokio::sync::oneshot::channel();

                {
                    let mut map = state.pending_approvals.lock().await;
                    map.insert(request_id.clone(), tx);
                }

                let redacted_description = sanitize_description(&req.description);
                let redacted_action = sanitize_description(&req.action_type);

                #[derive(serde::Serialize, Clone)]
                struct ApprovalRequestPayload {
                    request_id: String,
                    action_type: String,
                    risk_score: f32,
                    description: String,
                    timeout_ms: u64,
                }

                let payload = ApprovalRequestPayload {
                    request_id: request_id.clone(),
                    action_type: redacted_action,
                    risk_score: req.risk_score,
                    description: redacted_description,
                    timeout_ms: APPROVAL_REQUEST_TIMEOUT.as_millis() as u64,
                };

                let _ = self.app_handle.emit("approval_request", payload);

                Ok(await_ui_approval_response(
                    state.pending_approvals.clone(),
                    request_id,
                    req.action_type.clone(),
                    rx,
                    APPROVAL_REQUEST_TIMEOUT,
                )
                .await)
            } else {
                self.inner.approve(ctx, req).await
            }
        } else {
            self.inner.approve(ctx, req).await
        }
    }

    async fn vote(
        &self,
        voter_id: &str,
        proposal_id: &str,
        vote: agent_core::governance::Vote,
    ) -> chimera_repl::traits::ReplResult<()> {
        self.inner.vote(voter_id, proposal_id, vote).await
    }

    async fn escalate(
        &self,
        req: &agent_core::governance::GovernanceRequest,
        to_level: agent_core::governance::ApprovalLevel,
    ) -> chimera_repl::traits::ReplResult<agent_core::governance::GovernanceRequest> {
        self.inner.escalate(req, to_level).await
    }

    async fn register_policy(
        &mut self,
        _name: &str,
        _policy: Arc<dyn agent_core::governance::GovernancePolicy>,
        _caller: &str,
        _required_level: agent_core::governance::PermissionLevel,
    ) -> chimera_repl::traits::ReplResult<()> {
        Ok(())
    }

    async fn record_feedback(
        &self,
        ctx: &agent_core::AgentContext,
        feedback: &agent_core::governance::UserFeedback,
    ) -> chimera_repl::traits::ReplResult<()> {
        self.inner.record_feedback(ctx, feedback).await
    }

    async fn set_approval_level(
        &mut self,
        level: agent_core::governance::ApprovalLevel,
    ) -> chimera_repl::traits::ReplResult<()> {
        if let Some(state) = self.app_handle.try_state::<AppState>() {
            let level_str = match level {
                agent_core::governance::ApprovalLevel::Auto => "Auto",
                agent_core::governance::ApprovalLevel::Advisory => "Advisory",
                agent_core::governance::ApprovalLevel::Required => "Required",
                agent_core::governance::ApprovalLevel::Critical => "Critical",
                agent_core::governance::ApprovalLevel::Override => "Override",
            };
            *state.approval_level.lock().unwrap() = level_str.to_string();
        }
        Ok(())
    }

    async fn current_approval_level(&self) -> agent_core::governance::ApprovalLevel {
        if let Some(state) = self.app_handle.try_state::<AppState>() {
            let level_str = state.approval_level.lock().unwrap().clone();
            match level_str.as_str() {
                "Auto" => agent_core::governance::ApprovalLevel::Auto,
                "Advisory" => agent_core::governance::ApprovalLevel::Advisory,
                "Required" => agent_core::governance::ApprovalLevel::Required,
                "Critical" => agent_core::governance::ApprovalLevel::Critical,
                "Override" => agent_core::governance::ApprovalLevel::Override,
                _ => agent_core::governance::ApprovalLevel::Auto,
            }
        } else {
            agent_core::governance::ApprovalLevel::Auto
        }
    }
}
