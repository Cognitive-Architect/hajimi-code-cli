//! LLM adapter bridge: engine-llm-core → planner::LlmClient / reflector::ReflectionLlmClient.

use async_trait::async_trait;
use chimera_repl::traits::{ReplError, ReplResult};
use std::sync::Arc;

/// Bridge that wraps an engine-llm-core LlmClient to implement planner::LlmClient.
pub struct PlannerLlmBridge {
    inner: Arc<dyn engine_llm_core::LlmClient>,
    blackboard: Option<Arc<crate::blackboard::Blackboard>>,
    clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>,
}

impl PlannerLlmBridge {
    pub fn new(inner: Arc<dyn engine_llm_core::LlmClient>) -> Self { Self { inner, blackboard: None, clients: std::collections::HashMap::new() } }
    pub fn with_blackboard(mut self, bb: Arc<crate::blackboard::Blackboard>) -> Self { self.blackboard = Some(bb); self }
    pub fn with_clients(mut self, clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>) -> Self { self.clients = clients; self }
}

#[async_trait]
impl crate::planner::LlmClient for PlannerLlmBridge {
    async fn decompose_goal(&self, goal: &crate::planner::Goal) -> ReplResult<Vec<crate::planner::SubGoal>> {
        let prompt = format!(
            "Decompose the goal into sub-goals. Return ONLY JSON array.\nGoal: {}\nPriority: {:?}\n\nFormat: [{{\"description\":\"...\",\"priority\":\"High\"}}]",
            goal.description, goal.priority
        );
        let text = self.chat_and_collect(prompt).await?;
        let dtos: Vec<SubGoalDto> = serde_json::from_str(&text).map_err(ReplError::Protocol)?;
        Ok(dtos.into_iter().enumerate().map(|(i, d)| mk_subgoal(&goal.id, &d.description, d.priority, i)).collect())
    }

    async fn generate_tasks(&self, subgoal: &crate::planner::SubGoal) -> ReplResult<Vec<crate::planner::Task>> {
        let prompt = format!(
            "Generate tasks for sub-goal. Return ONLY JSON array of strings.\nSubGoal: {}\n\nFormat: [\"task 1\",\"task 2\"]",
            subgoal.description
        );
        let text = self.chat_and_collect(prompt).await?;
        let descriptions: Vec<String> = serde_json::from_str(&text).map_err(ReplError::Protocol)?;
        Ok(descriptions.into_iter().enumerate().map(|(i, d)| mk_task(&subgoal.id, &d, i)).collect())
    }
}

impl PlannerLlmBridge {
    async fn chat_and_collect(&self, prompt: String) -> ReplResult<String> {
        let client = if let Some(ref bb) = self.blackboard {
            if let Some(entry) = bb.read("__hajimi_provider_id").await {
                tracing::info!("PlannerLlmBridge selecting client for provider_id: {}", entry.value);
                self.clients.get(&entry.value).unwrap_or(&self.inner)
            } else { &self.inner }
        } else { &self.inner };
        let prompt = format!("{}\n\n{}", crate::planner::THINKING_FORMAT_INSTRUCTION, prompt);

        // Phase 1 (AGENT-PROMPT-CORE-001): Inject Agent Persona as system prompt
        let mut stream = if crate::prompts::is_persona_enabled() {
            let messages = vec![
                engine_llm_core::ChatMessage {
                    role: "system".into(),
                    content: crate::prompts::load_agent_persona().into(),
                    timestamp: None,
                },
                engine_llm_core::ChatMessage {
                    role: "user".into(),
                    content: prompt,
                    timestamp: None,
                },
            ];
            client.stream_chat_with_context(messages, None).await
        } else {
            client.stream_chat(prompt).await
        }.map_err(|e| ReplError::Session(e.to_string()))?;

        let text = collect_stream(&mut stream).await.map_err(|e| ReplError::Session(e.to_string()))?;
        if let Some(thinking) = crate::planner::extract_thinking(&text) {
            if let Some(ref bb) = self.blackboard {
                bb.write("__hajimi_thinking", &thinking, "planner").await;
            }
        }
        Ok(crate::planner::remove_thinking_tags(&text))
    }
}

/// Bridge that wraps an engine-llm-core LlmClient to implement reflector::ReflectionLlmClient.
pub struct ReflectorLlmBridge {
    inner: Arc<dyn engine_llm_core::LlmClient>,
    blackboard: Option<Arc<crate::blackboard::Blackboard>>,
    clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>,
}

impl ReflectorLlmBridge {
    pub fn new(inner: Arc<dyn engine_llm_core::LlmClient>) -> Self { Self { inner, blackboard: None, clients: std::collections::HashMap::new() } }
    pub fn with_blackboard(mut self, bb: Arc<crate::blackboard::Blackboard>) -> Self { self.blackboard = Some(bb); self }
    pub fn with_clients(mut self, clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>) -> Self { self.clients = clients; self }
}

#[async_trait]
impl crate::reflector::ReflectionLlmClient for ReflectorLlmBridge {
    async fn llm_critique(&self, goal: &crate::planner::Goal, result: &crate::planner::TaskResult) -> ReplResult<crate::reflector::Critique> {
        let prompt = format!(
            "Critique execution result. Return ONLY JSON.\nGoal: {}\nOutput: {}\nSuccess: {}\n\nFormat: {{\"success\":true,\"issues\":[],\"suggestions\":[],\"severity\":\"Low\"}}",
            goal.description, result.output, result.success
        );
        let text = self.chat_and_collect(prompt).await?;
        serde_json::from_str(&text).map_err(ReplError::Protocol)
    }

    async fn llm_optimize(&self, goal: &crate::planner::Goal, critique: &crate::reflector::Critique) -> ReplResult<String> {
        let prompt = format!(
            "Optimize plan based on critique. Return plain text.\nGoal: {}\nIssues: {:?}\nSuggestions: {:?}\nSeverity: {:?}",
            goal.description, critique.issues, critique.suggestions, critique.severity
        );
        self.chat_and_collect(prompt).await
    }
}

impl ReflectorLlmBridge {
    async fn chat_and_collect(&self, prompt: String) -> ReplResult<String> {
        let client = if let Some(ref bb) = self.blackboard {
            if let Some(entry) = bb.read("__hajimi_provider_id").await {
                tracing::info!("ReflectorLlmBridge selecting client for provider_id: {}", entry.value);
                self.clients.get(&entry.value).unwrap_or(&self.inner)
            } else { &self.inner }
        } else { &self.inner };
        let prompt = format!("{}\n\n{}", crate::reflector::THINKING_FORMAT_INSTRUCTION, prompt);

        // Phase 1 (AGENT-PROMPT-CORE-001): Inject Agent Persona as system prompt
        let mut stream = if crate::prompts::is_persona_enabled() {
            let messages = vec![
                engine_llm_core::ChatMessage {
                    role: "system".into(),
                    content: crate::prompts::load_agent_persona().into(),
                    timestamp: None,
                },
                engine_llm_core::ChatMessage {
                    role: "user".into(),
                    content: prompt,
                    timestamp: None,
                },
            ];
            client.stream_chat_with_context(messages, None).await
        } else {
            client.stream_chat(prompt).await
        }.map_err(|e| ReplError::Session(e.to_string()))?;

        let text = collect_stream(&mut stream).await.map_err(|e| ReplError::Session(e.to_string()))?;
        if let Some(thinking) = crate::reflector::extract_thinking(&text) {
            if let Some(ref bb) = self.blackboard {
                bb.write("__hajimi_thinking", &thinking, "reflector").await;
            }
        }
        Ok(crate::planner::remove_thinking_tags(&text))
    }
}

/// Collect all Output chunks from a ChannelStream. Returns error on StreamChunk::Error.
pub async fn collect_stream(stream: &mut engine_llm_core::ChannelStream) -> Result<String, engine_llm_core::EngineError> {
    let mut text = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            engine_llm_core::StreamChunk::Output(s) => text.push_str(&s),
            engine_llm_core::StreamChunk::Error(e) => return Err(engine_llm_core::EngineError::InvalidParameters(e)),
            engine_llm_core::StreamChunk::Done => break,
        }
    }
    Ok(text)
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct SubGoalDto { description: String, priority: crate::planner::Priority }

fn mk_subgoal(parent: &str, desc: &str, priority: crate::planner::Priority, idx: usize) -> crate::planner::SubGoal {
    crate::planner::SubGoal {
        id: format!("{}-sg{}", parent, idx), parent_goal: parent.to_string(),
        description: desc.to_string(), priority, status: crate::planner::PlanStatus::Pending,
        tasks: Vec::new(), dependencies: Vec::new(),
    }
}

fn mk_task(parent: &str, desc: &str, idx: usize) -> crate::planner::Task {
    crate::planner::Task {
        id: format!("{}-t{}", parent, idx), parent_subgoal: parent.to_string(),
        description: desc.to_string(), tool_calls: Vec::new(),
        status: crate::planner::PlanStatus::Pending, result: None,
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_stream_success() {
        let (mut stream, tx) = engine_llm_core::ChannelStream::new(10);
        tx.send(engine_llm_core::StreamChunk::Output("hello ".into())).await.unwrap();
        tx.send(engine_llm_core::StreamChunk::Output("world".into())).await.unwrap();
        tx.send(engine_llm_core::StreamChunk::Done).await.unwrap();
        drop(tx);
        assert_eq!(collect_stream(&mut stream).await.unwrap(), "hello world");
    }

    #[tokio::test]
    async fn test_collect_stream_error() {
        let (mut stream, tx) = engine_llm_core::ChannelStream::new(10);
        tx.send(engine_llm_core::StreamChunk::Output("partial".into())).await.unwrap();
        tx.send(engine_llm_core::StreamChunk::Error("fail".into())).await.unwrap();
        drop(tx);
        assert!(collect_stream(&mut stream).await.is_err());
    }

    #[test]
    fn test_bridge_types_exist() {
        assert!(std::mem::size_of::<PlannerLlmBridge>() > 0);
        assert!(std::mem::size_of::<ReflectorLlmBridge>() > 0);
    }

    #[test]
    fn test_subgoal_dto_parse() {
        let dtos: Vec<SubGoalDto> = serde_json::from_str(r#"[{"description":"A","priority":"High"}]"#).unwrap();
        assert_eq!(dtos[0].description, "A");
    }

    #[test]
    fn test_mk_subgoal_and_task() {
        let sg = mk_subgoal("g1", "Desc", crate::planner::Priority::Medium, 0);
        assert_eq!(sg.parent_goal, "g1");
        let t = mk_task("sg1", "Task", 1);
        assert_eq!(t.parent_subgoal, "sg1");
    }
}
