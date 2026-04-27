//! DEBT-LINES-B0301B: Extracted from reflector.rs.
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::reflector::{CritiqueSeverity, Reflection};
use crate::AgentContext;
use chimera_repl::traits::ReplResult;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ReflectionPersistence {
    memory: Arc<Mutex<memory::memory_gateway::MemoryGateway>>,
}

impl ReflectionPersistence {
    pub fn new(memory: Arc<Mutex<memory::memory_gateway::MemoryGateway>>) -> Self {
        Self { memory }
    }

    pub async fn persist(&self, reflection: &Reflection) -> ReplResult<()> {
        let key = format!("refl_{}", reflection.reflection_id);
        let content = serde_json::to_string(reflection).map_err(chimera_repl::traits::ReplError::Protocol)?;
        let mut guard = self.memory.lock().await;
        if guard.dream.is_some() { let _ = guard.push_vector(&key, &content); }
        let entry = memory::types::MemoryEntry {
            id: reflection.reflection_id.clone(),
            content: format!("Reflection for goal {}: confidence={}", reflection.original_goal_id, reflection.confidence),
            tokens: 100, timestamp: chrono::Utc::now(), layer: memory::types::MemoryLayerId::Graph,
        };
        if let Some(graph) = guard.graph.as_mut() { let _ = graph.store(entry); }
        Ok(())
    }

    pub async fn approve(&self, reflection: &Reflection, governance: &dyn AgentGovernance, context: &AgentContext) -> ReplResult<bool> {
        let severity_score = match reflection.critique.severity {
            CritiqueSeverity::Low => 0.2, CritiqueSeverity::Medium => 0.5,
            CritiqueSeverity::High => 0.8, CritiqueSeverity::Critical => 1.0,
        };
        let req = GovernanceRequest {
            requester: "reflector".to_string(), action_type: "approve_reflection".to_string(), risk_score: severity_score,
            description: format!("Reflection for goal {} with confidence {}", reflection.original_goal_id, reflection.confidence),
            level: if severity_score > 0.8 { ApprovalLevel::Critical } else { ApprovalLevel::Advisory },
        };
        let decision = governance.approve(context, &req).await?;
        Ok(matches!(decision, Decision::Approved | Decision::Escalated(_)))
    }

    pub async fn load(&self, reflection_id: &str) -> ReplResult<Option<Reflection>> {
        let guard = self.memory.lock().await;
        if let Some(session) = guard.session.get(&format!("refl_{}", reflection_id)) {
            if let Ok(r) = serde_json::from_str(&format!("{:?}", session)) { return Ok(Some(r)); }
        }
        Ok(None)
    }
}
