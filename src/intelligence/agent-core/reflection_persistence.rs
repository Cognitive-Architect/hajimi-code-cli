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
        let mut key = String::from("refl_");
        key.push_str(&reflection.reflection_id);
        let content = serde_json::to_string(reflection).map_err(chimera_repl::traits::ReplError::Protocol)?;
        let mut guard = self.memory.lock().await;
        // Always write to session so load() can retrieve even when Dream/Graph are unavailable.
        let _ = guard.session.insert(key.clone(), content.clone());
        if guard.dream.is_some() { let _ = guard.push_vector(&key, &content); }
        let mut entry_content = String::from("Reflection for goal ");
        entry_content.push_str(&reflection.original_goal_id);
        entry_content.push_str(": confidence=");
        entry_content.push_str(&reflection.confidence.to_string());
        let entry = memory::types::MemoryEntry {
            id: reflection.reflection_id.clone(),
            content: entry_content,
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
        let mut desc = String::from("Reflection for goal ");
        desc.push_str(&reflection.original_goal_id);
        desc.push_str(" with confidence ");
        desc.push_str(&reflection.confidence.to_string());
        let req = GovernanceRequest {
            requester: "reflector".to_string(), action_type: "approve_reflection".to_string(), risk_score: severity_score,
            description: desc,
            level: if severity_score > 0.8 { ApprovalLevel::Critical } else { ApprovalLevel::Advisory },
        };
        let decision = governance.approve(context, &req).await?;
        Ok(matches!(decision, Decision::Approved | Decision::Escalated(_)))
    }

    pub async fn load(&self, reflection_id: &str) -> ReplResult<Option<Reflection>> {
        let guard = self.memory.lock().await;
        let mut key = String::from("refl_");
        key.push_str(reflection_id);
        if let Some(session) = guard.session.get(&key) {
            // SAFETY: Reflection session.content is the JSON string persisted by serde_json::to_string.
            match serde_json::from_str::<Reflection>(&session.content) {
                Ok(r) => return Ok(Some(r)),
                Err(e) => { tracing::warn!("Reflection deserialization failed: {}", e); }
            }
        }
        Ok(None)
    }
}
