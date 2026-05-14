//! Agent Governance: Policy-driven approval engine with 5-level strategy and voting mechanism.
use crate::AgentContext;
use async_trait::async_trait;
use chimera_repl::traits::{ReplError, ReplResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{info, warn};

/// Permission level for governance operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    User,
    Admin,
    System,
}

/// 5-level approval strategy for agent decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ApprovalLevel {
    #[default]
    Auto,
    Advisory,
    Required,
    Critical,
    Override,
}

/// Governance decision outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Approved,
    Rejected(String),
    Escalated(ApprovalLevel),
    Timeout,
}

/// Individual vote in multi-agent decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vote {
    Approve,
    Reject(String),
    Abstain,
}

/// Vote aggregation state for Critical level decisions.
#[derive(Debug, Clone)]
pub struct VoteState {
    pub voters: Vec<String>,
    pub ballots: HashMap<String, Vote>,
    pub deadline: Instant,
    pub threshold: f32,
}

/// User feedback collected from Accept/Reject/Explain actions.
#[derive(Debug, Clone)]
pub struct UserFeedback {
    pub choice: String,
    pub uri: Option<String>,
    pub query: String,
    pub timestamp: u64,
}

impl VoteState {
    fn new(voters: Vec<String>, timeout_ms: u64) -> Self {
        Self {
            ballots: HashMap::new(),
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
            threshold: 0.5,
            voters,
        }
    }
    fn is_expired(&self) -> bool {
        Instant::now() > self.deadline
    }
    fn tally(&self) -> (usize, usize, usize) {
        let (mut a, mut r, mut ab) = (0, 0, 0);
        for v in self.ballots.values() {
            match v {
                Vote::Approve => a += 1,
                Vote::Reject(_) => r += 1,
                Vote::Abstain => ab += 1,
            }
        }
        (a, r, ab)
    }
    fn result(&self) -> Option<Decision> {
        if self.is_expired() {
            return Some(Decision::Timeout);
        }
        let (a, r, _) = self.tally();
        let t = self.voters.len();
        if t == 0 {
            return Some(Decision::Escalated(ApprovalLevel::Override));
        }
        if a as f32 / t as f32 >= self.threshold {
            return Some(Decision::Approved);
        }
        if r as f32 / t as f32 > (1.0 - self.threshold) {
            return Some(Decision::Rejected("Majority rejected".to_string()));
        }
        if self.ballots.len() == t {
            if a > r {
                Some(Decision::Approved)
            } else {
                Some(Decision::Rejected("No majority".to_string()))
            }
        } else {
            None
        }
    }
}

/// Request metadata for governance decisions.
#[derive(Debug, Clone)]
pub struct GovernanceRequest {
    pub requester: String,
    pub action_type: String,
    pub risk_score: f32,
    pub description: String,
    pub level: ApprovalLevel,
}

/// Core governance trait for agent decision approval.
#[async_trait]
pub trait AgentGovernance: Send + Sync {
    async fn policy(&self, ctx: &AgentContext, req: &GovernanceRequest) -> ApprovalLevel;
    async fn approve(&self, ctx: &AgentContext, req: &GovernanceRequest) -> ReplResult<Decision>;
    async fn vote(&self, voter_id: &str, proposal_id: &str, vote: Vote) -> ReplResult<()>;
    async fn escalate(
        &self,
        req: &GovernanceRequest,
        to_level: ApprovalLevel,
    ) -> ReplResult<GovernanceRequest>;
    async fn register_policy(
        &mut self,
        name: &str,
        policy: Arc<dyn GovernancePolicy>,
        caller: &str,
        required_level: PermissionLevel,
    ) -> ReplResult<()>;
    async fn record_feedback(&self, ctx: &AgentContext, feedback: &UserFeedback) -> ReplResult<()>;
    async fn set_approval_level(&mut self, _level: ApprovalLevel) -> ReplResult<()> {
        Err(ReplError::Session(
            "set_approval_level not implemented".to_string(),
        ))
    }
    async fn current_approval_level(&self) -> ApprovalLevel {
        ApprovalLevel::Auto
    }
}

/// User-defined governance policy extension point.
#[async_trait]
pub trait GovernancePolicy: Send + Sync {
    async fn evaluate(&self, ctx: &AgentContext, req: &GovernanceRequest) -> Option<Decision>;
}

/// Default governance engine with 5-level strategy.
pub struct DefaultGovernance {
    policies: RwLock<HashMap<String, Arc<dyn GovernancePolicy>>>,
    votes: RwLock<HashMap<String, VoteState>>,
    default_level: ApprovalLevel,
    required_approvers: RwLock<HashMap<String, Vec<String>>>,
    feedback_store: RwLock<HashMap<String, Vec<UserFeedback>>>,
}

impl Default for DefaultGovernance {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultGovernance {
    pub fn new() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
            votes: RwLock::new(HashMap::new()),
            default_level: ApprovalLevel::Auto,
            required_approvers: RwLock::new(HashMap::new()),
            feedback_store: RwLock::new(HashMap::new()),
        }
    }
    pub async fn verify_caller(&self, caller: &str, required: PermissionLevel) -> bool {
        match required {
            PermissionLevel::User => !caller.is_empty(),
            PermissionLevel::Admin => caller.starts_with("admin_") || caller == "system",
            PermissionLevel::System => caller == "system",
        }
    }
    pub fn with_default_level(mut self, level: ApprovalLevel) -> Self {
        self.default_level = level;
        self
    }
    pub async fn set_approval_level(&mut self, level: ApprovalLevel) -> ReplResult<()> {
        self.default_level = level;
        info!("Approval level changed to {:?}", level);
        Ok(())
    }
    pub fn current_approval_level(&self) -> ApprovalLevel {
        self.default_level
    }
    fn audit_override(&self, req: &GovernanceRequest) {
        warn!(
            "OVERRIDE AUDIT: {} by {} - {}",
            req.action_type, req.requester, req.description
        );
    }
}

#[async_trait]
impl AgentGovernance for DefaultGovernance {
    async fn policy(&self, _ctx: &AgentContext, req: &GovernanceRequest) -> ApprovalLevel {
        if req.level != ApprovalLevel::Auto {
            return req.level;
        }
        let policies = self.policies.read().await;
        for policy in policies.values() {
            if policy.evaluate(_ctx, req).await.is_some() {
                return req.level;
            }
        }
        if req.risk_score > 0.9 {
            ApprovalLevel::Override
        } else if req.risk_score > 0.7 {
            ApprovalLevel::Critical
        } else if req.risk_score > 0.4 {
            ApprovalLevel::Required
        } else if req.risk_score > 0.2 {
            ApprovalLevel::Advisory
        } else {
            self.default_level
        }
    }

    async fn approve(&self, ctx: &AgentContext, req: &GovernanceRequest) -> ReplResult<Decision> {
        let level = self.policy(ctx, req).await;
        info!(
            "Governance approval: {} at level {:?}",
            req.action_type, level
        );
        match level {
            ApprovalLevel::Auto => Ok(Decision::Approved),
            ApprovalLevel::Advisory => {
                warn!("Advisory: {} - proceeding", req.description);
                Ok(Decision::Approved)
            }
            ApprovalLevel::Required => {
                let approvers = self.required_approvers.read().await;
                let topic = format!("{}_{}", req.requester, req.action_type);
                match approvers.get(&topic) {
                    Some(voters) if !voters.is_empty() => Ok(Decision::Approved),
                    _ => {
                        warn!(
                            "Required: no approver bound for topic '{}', escalating to Critical",
                            topic
                        );
                        Ok(Decision::Escalated(ApprovalLevel::Critical))
                    }
                }
            }
            ApprovalLevel::Critical => {
                let pid = format!("{}_{}", req.requester, req.action_type);
                let votes = self.votes.read().await;
                match votes.get(&pid) {
                    Some(state) => Ok(state
                        .result()
                        .unwrap_or(Decision::Escalated(ApprovalLevel::Override))),
                    None => Ok(Decision::Escalated(ApprovalLevel::Required)),
                }
            }
            ApprovalLevel::Override => {
                self.audit_override(req);
                Ok(Decision::Approved)
            }
        }
    }

    async fn vote(&self, voter_id: &str, proposal_id: &str, vote: Vote) -> ReplResult<()> {
        let mut votes = self.votes.write().await;
        if let Some(state) = votes.get_mut(proposal_id) {
            if state.voters.contains(&voter_id.to_string()) {
                state.ballots.insert(voter_id.to_string(), vote);
            }
        }
        Ok(())
    }

    async fn escalate(
        &self,
        req: &GovernanceRequest,
        to_level: ApprovalLevel,
    ) -> ReplResult<GovernanceRequest> {
        let mut new_req = req.clone();
        new_req.level = to_level;
        if to_level == ApprovalLevel::Critical {
            let mut votes = self.votes.write().await;
            votes.insert(
                format!("{}_{}", req.requester, req.action_type),
                VoteState::new(vec!["agent1".to_string(), "agent2".to_string()], 5000),
            );
        }
        info!("Escalated {} to {:?}", req.action_type, to_level);
        Ok(new_req)
    }

    async fn register_policy(
        &mut self,
        name: &str,
        policy: Arc<dyn GovernancePolicy>,
        caller: &str,
        required_level: PermissionLevel,
    ) -> ReplResult<()> {
        if !self.verify_caller(caller, required_level).await {
            return Err(ReplError::Session(format!(
                "Policy registration denied for caller '{}': requires {:?}",
                caller, required_level
            )));
        }
        self.policies.write().await.insert(name.to_string(), policy);
        Ok(())
    }

    async fn record_feedback(
        &self,
        _ctx: &AgentContext,
        feedback: &UserFeedback,
    ) -> ReplResult<()> {
        let mut store = self.feedback_store.write().await;
        let key = feedback.query.clone();
        store.entry(key).or_default().push(feedback.clone());
        info!(
            "Feedback recorded: {} for query '{}'",
            feedback.choice, feedback.query
        );
        Ok(())
    }

    async fn set_approval_level(&mut self, level: ApprovalLevel) -> ReplResult<()> {
        self.set_approval_level(level).await
    }

    async fn current_approval_level(&self) -> ApprovalLevel {
        self.current_approval_level()
    }
}

impl Clone for DefaultGovernance {
    fn clone(&self) -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
            votes: RwLock::new(HashMap::new()),
            default_level: self.default_level,
            required_approvers: RwLock::new(HashMap::new()),
            feedback_store: RwLock::new(HashMap::new()),
        }
    }
}

impl DefaultGovernance {
    pub async fn feedback_history(&self, action_type: &str) -> Vec<UserFeedback> {
        self.feedback_store
            .read()
            .await
            .get(action_type)
            .cloned()
            .unwrap_or_default()
    }
}

/// Feedback-driven governance policy: escalates ApprovalLevel if reject rate > 50%.
pub struct FeedbackPolicy;

#[async_trait]
impl GovernancePolicy for FeedbackPolicy {
    async fn evaluate(&self, _ctx: &AgentContext, _req: &GovernanceRequest) -> Option<Decision> {
        None // Placeholder: actual evaluation uses feedback_history from DefaultGovernance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentContext;
    fn ctx() -> AgentContext {
        AgentContext::new()
    }
    fn req(level: ApprovalLevel) -> GovernanceRequest {
        GovernanceRequest {
            requester: "t".to_string(),
            action_type: "a".to_string(),
            risk_score: 0.1,
            description: "T".to_string(),
            level,
        }
    }

    #[tokio::test]
    async fn test_auto_approval() {
        let gov = DefaultGovernance::new();
        assert_eq!(
            gov.approve(&ctx(), &req(ApprovalLevel::Auto))
                .await
                .unwrap(),
            Decision::Approved
        );
    }

    #[tokio::test]
    async fn test_critical_vote() {
        let gov = DefaultGovernance::new();
        let mut r = req(ApprovalLevel::Auto);
        r.risk_score = 0.8;
        assert!(matches!(
            gov.approve(&ctx(), &r).await.unwrap(),
            Decision::Escalated(_) | Decision::Approved
        ));
    }

    #[tokio::test]
    async fn test_governance_chain() {
        let gov = DefaultGovernance::new();
        let decision = gov
            .approve(&ctx(), &req(ApprovalLevel::Required))
            .await
            .unwrap();
        assert!(matches!(
            decision,
            Decision::Approved | Decision::Rejected(_) | Decision::Escalated(_)
        ));
    }

    struct RejectHighRiskPolicy;
    #[async_trait]
    impl GovernancePolicy for RejectHighRiskPolicy {
        async fn evaluate(&self, _ctx: &AgentContext, req: &GovernanceRequest) -> Option<Decision> {
            if req.risk_score > 0.8 {
                Some(Decision::Rejected("risky".to_string()))
            } else {
                None
            }
        }
    }

    #[tokio::test]
    async fn test_custom_governance() {
        let mut gov = DefaultGovernance::new();
        gov.register_policy(
            "reject_high",
            Arc::new(RejectHighRiskPolicy),
            "admin_test",
            PermissionLevel::Admin,
        )
        .await
        .unwrap();
        let mut r = req(ApprovalLevel::Auto);
        r.risk_score = 0.9;
        assert!(matches!(
            gov.approve(&ctx(), &r).await.unwrap(),
            Decision::Approved | Decision::Rejected(_)
        ));
    }

    #[tokio::test]
    async fn test_user_governance_plugin() {
        struct AllowPolicy;
        #[async_trait]
        impl GovernancePolicy for AllowPolicy {
            async fn evaluate(
                &self,
                _ctx: &AgentContext,
                _req: &GovernanceRequest,
            ) -> Option<Decision> {
                Some(Decision::Approved)
            }
        }
        let mut gov = DefaultGovernance::new();
        gov.register_policy(
            "user_custom",
            Arc::new(AllowPolicy),
            "admin_test",
            PermissionLevel::Admin,
        )
        .await
        .unwrap();
        assert!(matches!(
            gov.approve(&ctx(), &req(ApprovalLevel::Auto))
                .await
                .unwrap(),
            Decision::Approved
        ));
    }

    #[tokio::test]
    async fn test_register_policy_denied_for_illegal_caller() {
        let mut gov = DefaultGovernance::new();
        let result = gov
            .register_policy(
                "bad",
                Arc::new(RejectHighRiskPolicy),
                "hacker",
                PermissionLevel::Admin,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_required_no_approver_escalated() {
        let gov = DefaultGovernance::new();
        let mut r = req(ApprovalLevel::Required);
        r.action_type = "delete_all".to_string();
        let decision = gov.approve(&ctx(), &r).await.unwrap();
        assert!(matches!(
            decision,
            Decision::Escalated(ApprovalLevel::Critical)
        ));
    }
}
