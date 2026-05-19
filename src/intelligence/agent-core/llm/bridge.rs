//! LLM adapter bridge: engine-llm-core → planner::LlmClient / reflector::ReflectionLlmClient.

use async_trait::async_trait;
use chimera_repl::traits::{ReplError, ReplResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Bridge that wraps an engine-llm-core LlmClient to implement planner::LlmClient.
pub struct PlannerLlmBridge {
    inner: Arc<dyn engine_llm_core::LlmClient>,
    blackboard: Option<Arc<crate::blackboard::Blackboard>>,
    clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>,
    tool_registry: Option<Arc<engine_tool_system::ToolRegistry>>,
}

impl PlannerLlmBridge {
    pub fn new(inner: Arc<dyn engine_llm_core::LlmClient>) -> Self {
        Self {
            inner,
            blackboard: None,
            clients: std::collections::HashMap::new(),
            tool_registry: None,
        }
    }
    pub fn with_blackboard(mut self, bb: Arc<crate::blackboard::Blackboard>) -> Self {
        self.blackboard = Some(bb);
        self
    }
    pub fn with_clients(
        mut self,
        clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>,
    ) -> Self {
        self.clients = clients;
        self
    }
    /// Builder: attach a tool registry via `with_tool_registry` (AGENT-PROMPT-CORE-001 Day 5).
    pub fn with_tool_registry(mut self, reg: Arc<engine_tool_system::ToolRegistry>) -> Self {
        self.tool_registry = Some(reg);
        self
    }
}

#[async_trait]
impl crate::planner::LlmClient for PlannerLlmBridge {
    async fn decompose_goal(
        &self,
        goal: &crate::planner::Goal,
    ) -> ReplResult<Vec<crate::planner::SubGoal>> {
        if crate::prompts::is_planner_v1_enabled() {
            self.decompose_goal_v1(goal).await
        } else {
            self.decompose_goal_legacy(goal).await
        }
    }

    async fn generate_tasks(
        &self,
        subgoal: &crate::planner::SubGoal,
    ) -> ReplResult<Vec<crate::planner::Task>> {
        let prompt = format!("Generate tasks for sub-goal. Return ONLY JSON array of strings.\nSubGoal: {}\n\nFormat: [\"task 1\",\"task 2\"]", subgoal.description);
        let text = self.chat_and_collect(prompt).await?;
        let descriptions: Vec<String> = serde_json::from_str(&text).map_err(ReplError::Protocol)?;
        Ok(descriptions
            .into_iter()
            .enumerate()
            .map(|(i, d)| mk_task(&subgoal.id, &d, i))
            .collect())
    }
}

impl PlannerLlmBridge {
    /// Legacy decompose without tool manifest (fallback when feature-gate is off).
    async fn decompose_goal_legacy(
        &self,
        goal: &crate::planner::Goal,
    ) -> ReplResult<Vec<crate::planner::SubGoal>> {
        let prompt = format!("Decompose the goal into sub-goals. Return ONLY JSON array.\nGoal: {}\nPriority: {:?}\n\nFormat: [{{\"description\":\"...\",\"priority\":\"High\"}}]", goal.description, goal.priority);
        let text = self.chat_and_collect(prompt).await?;
        let dtos: Vec<SubGoalDto> = serde_json::from_str(&text).map_err(ReplError::Protocol)?;
        Ok(dtos
            .into_iter()
            .enumerate()
            .map(|(i, d)| mk_subgoal(&goal.id, &d.description, d.priority, i))
            .collect())
    }

    /// V1 decompose with ToolManifest injection and PlannerSubgoalPlanV1Dto parsing.
    async fn decompose_goal_v1(
        &self,
        goal: &crate::planner::Goal,
    ) -> ReplResult<Vec<crate::planner::SubGoal>> {
        let catalog = crate::tool_manifest::ToolCatalog::new();
        let generator = crate::tool_manifest::ToolManifestGenerator::new(catalog);
        let request = crate::tool_manifest::ToolManifestRequest {
            goal_description: goal.description.clone(),
            step_type: crate::tool_manifest::StepType::Plan,
            current_task: None,
            recently_failed_tools: vec![],
            available_budget_tokens: 1600,
            max_tools: 15,
        };
        let manifest = generator.generate(&request);
        let prompt = format!("Given the following available tools:\n{}\n\nDecompose the goal into sub-goals that can be executed using these tools. Return ONLY valid JSON matching PlannerSubgoalPlanV1.\nGoal: {}\nPriority: {:?}",
            serde_json::to_string(&manifest).unwrap_or_else(|_| "[]".to_string()), goal.description, goal.priority);
        let text = self.chat_and_collect(prompt).await?;
        let dto: crate::planner_dto::PlannerSubgoalPlanV1Dto =
            serde_json::from_str(&text).map_err(ReplError::Protocol)?;
        let mut subgoals = Vec::new();
        for sg in dto.subgoals {
            let mut suggested = sg.suggested_tools.clone();
            if let Some(ref registry) = self.tool_registry {
                suggested.retain(|name| {
                    if registry.get(name).is_some() {
                        true
                    } else {
                        tracing::warn!("Planner suggested unknown tool '{}', removing", name);
                        false
                    }
                });
            }
            let mut metadata = HashMap::new();
            metadata.insert("__hajimi_planner_v1".to_string(), "true".to_string());
            metadata.insert(
                "validation_intent".to_string(),
                format!("{:?}", sg.validation_intent),
            );
            metadata.insert("risk_level".to_string(), format!("{:?}", sg.risk_level));
            // Store filtered suggested_tools, expected_evidence, and stop_conditions
            // in metadata for downstream reflect/stop-loss logic (B-05-FIX-001).
            metadata.insert(
                "suggested_tools".to_string(),
                serde_json::to_string(&suggested).unwrap_or_default(),
            );
            metadata.insert(
                "expected_evidence".to_string(),
                serde_json::to_string(&sg.expected_evidence).unwrap_or_default(),
            );
            metadata.insert(
                "stop_conditions".to_string(),
                serde_json::to_string(&sg.stop_conditions).unwrap_or_default(),
            );
            subgoals.push(crate::planner::SubGoal {
                id: sg.id_hint,
                parent_goal: goal.id.clone(),
                description: sg.description,
                priority: sg.priority,
                status: crate::planner::PlanStatus::Pending,
                tasks: Vec::new(),
                dependencies: sg.depends_on,
                metadata,
            });
        }
        Ok(subgoals)
    }
}

impl PlannerLlmBridge {
    async fn chat_and_collect(&self, prompt: String) -> ReplResult<String> {
        let client = if let Some(ref bb) = self.blackboard {
            if let Some(entry) = bb.read("__hajimi_provider_id").await {
                tracing::info!(
                    "PlannerLlmBridge selecting client for provider_id: {}",
                    entry.value
                );
                self.clients.get(&entry.value).unwrap_or(&self.inner)
            } else {
                &self.inner
            }
        } else {
            &self.inner
        };
        let prompt = format!(
            "{}\n\n{}",
            crate::planner::THINKING_FORMAT_INSTRUCTION,
            prompt
        );

        // Phase 4 Day 12: ContextWindowManager integration with feature-gate
        let mut stream = if crate::prompts::is_context_window_enabled() {
            let mut resolve_input = crate::context_budget::BudgetResolveInput::default();
            if let Some(ref bb) = self.blackboard {
                if let Some(entry) = bb.read("__hajimi_provider_id").await {
                    resolve_input.provider_id = Some(entry.value.clone());
                }
                if let Some(entry) = bb.read("__hajimi_model").await {
                    resolve_input.model = Some(entry.value.clone());
                }
            }
            let budget = crate::context_budget::resolve_context_budget(resolve_input);

            let sys_content = if crate::prompts::is_persona_enabled() {
                crate::prompts::load_agent_persona().to_string()
            } else {
                "System: Please assist.".to_string()
            };

            match assemble_messages_for_bridge(&prompt, &sys_content, &budget) {
                Ok((messages, estimated_input, omitted_count)) => {
                    tracing::info!(
                        "Planner budget stats: provider={}, model={}, max_context={}, input_budget={}, estimated_input={}, omitted_count={}",
                        budget.provider_id,
                        budget.model,
                        budget.max_context_tokens,
                        budget.input_budget,
                        estimated_input,
                        omitted_count
                    );
                    client
                        .stream_chat_with_context(messages, None)
                        .await
                        .map_err(|e| ReplError::Session(e.to_string()))?
                }
                Err(e) => {
                    tracing::warn!(
                        "ContextWindow assembly failed: {}. P0 overflow occurred with input_budget: {}!",
                        e,
                        budget.input_budget
                    );
                    return Err(ReplError::Session(format!(
                        "Context window assembly failed (P0 overflow) with input_budget: {}: {}",
                        budget.input_budget, e
                    )));
                }
            }
        } else {
            // legacy path: simple 2-message path
            if crate::prompts::is_persona_enabled() {
                let messages = vec![
                    engine_llm_core::ChatMessage {
                        role: "system".into(),
                        content: crate::prompts::load_agent_persona().into(),
                        timestamp: None,
                    },
                    engine_llm_core::ChatMessage {
                        role: "user".into(),
                        content: prompt.clone(),
                        timestamp: None,
                    },
                ];
                client
                    .stream_chat_with_context(messages, None)
                    .await
                    .map_err(|e| ReplError::Session(e.to_string()))?
            } else {
                client
                    .stream_chat(prompt.clone())
                    .await
                    .map_err(|e| ReplError::Session(e.to_string()))?
            }
        };

        let text = collect_stream(&mut stream)
            .await
            .map_err(|e| ReplError::Session(e.to_string()))?;
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
    pub fn new(inner: Arc<dyn engine_llm_core::LlmClient>) -> Self {
        Self {
            inner,
            blackboard: None,
            clients: std::collections::HashMap::new(),
        }
    }
    pub fn with_blackboard(mut self, bb: Arc<crate::blackboard::Blackboard>) -> Self {
        self.blackboard = Some(bb);
        self
    }
    pub fn with_clients(
        mut self,
        clients: std::collections::HashMap<String, Arc<dyn engine_llm_core::LlmClient>>,
    ) -> Self {
        self.clients = clients;
        self
    }
}

#[async_trait]
impl crate::reflector::ReflectionLlmClient for ReflectorLlmBridge {
    async fn llm_critique(
        &self,
        goal: &crate::planner::Goal,
        result: &crate::planner::TaskResult,
    ) -> ReplResult<crate::reflector::Critique> {
        let prompt = format!(
            "Critique execution result. Return ONLY valid JSON matching ReflectorCritiqueV1 schema.\n\
             Goal: {}\nOutput: {}\nSuccess: {}\n\n\
             Required JSON schema:\n\
             {{\n  \"schema_version\": \"ReflectorCritiqueV1\",\n  \"success\": true,\n  \"severity\": \"Low\",\n  \"confidence\": 0.9,\n  \"evidence\": [\"...\"],\n  \"root_cause\": {{\"category\":\"None\",\"description\":\"...\",\"confidence\":0.8}},\n  \"issues\": [],\n  \"new_risks\": [],\n  \"suggestions\": [],\n  \"plan_adjustment\": {{\"action\":\"Continue\",\"reason\":\"...\",\"revised_subgoals\":[]}},\n  \"stop_loss\": {{\"triggered\":false,\"reason\":\"...\",\"escalation_target\":\"user\"}}\n}}",
            goal.description, result.output, result.success
        );
        let text = self.chat_and_collect(prompt).await?;
        // B-09/14: Only attempt V1 DTO parsing when the reflector V1 feature-gate is enabled.
        // When disabled, skip directly to legacy Critique JSON parsing (fallback path).
        if crate::prompts::is_reflector_v1_enabled() {
            if let Ok(dto) =
                serde_json::from_str::<crate::reflector_dto::ReflectorCritiqueV1Dto>(&text)
            {
                return Ok(dto.to_critique());
            }
            tracing::warn!(
                "ReflectorCritiqueV1 parse failed, falling back to legacy Critique JSON"
            );
        }
        // legacy Critique fallback — always available regardless of feature-gate
        serde_json::from_str(&text).map_err(ReplError::Protocol)
    }

    async fn llm_optimize(
        &self,
        goal: &crate::planner::Goal,
        critique: &crate::reflector::Critique,
    ) -> ReplResult<String> {
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
                tracing::info!(
                    "ReflectorLlmBridge selecting client for provider_id: {}",
                    entry.value
                );
                self.clients.get(&entry.value).unwrap_or(&self.inner)
            } else {
                &self.inner
            }
        } else {
            &self.inner
        };
        let prompt = format!(
            "{}\n\n{}",
            crate::reflector::THINKING_FORMAT_INSTRUCTION,
            prompt
        );

        // Phase 4 Day 12: ContextWindowManager integration with feature-gate
        let mut stream = if crate::prompts::is_context_window_enabled() {
            let mut resolve_input = crate::context_budget::BudgetResolveInput::default();
            if let Some(ref bb) = self.blackboard {
                if let Some(entry) = bb.read("__hajimi_provider_id").await {
                    resolve_input.provider_id = Some(entry.value.clone());
                }
                if let Some(entry) = bb.read("__hajimi_model").await {
                    resolve_input.model = Some(entry.value.clone());
                }
            }
            let budget = crate::context_budget::resolve_context_budget(resolve_input);

            let sys_content = if crate::prompts::is_persona_enabled() {
                crate::prompts::load_agent_persona().to_string()
            } else {
                "System: Please assist.".to_string()
            };

            match assemble_messages_for_bridge(&prompt, &sys_content, &budget) {
                Ok((messages, estimated_input, omitted_count)) => {
                    tracing::info!(
                        "Reflector budget stats: provider={}, model={}, max_context={}, input_budget={}, estimated_input={}, omitted_count={}",
                        budget.provider_id,
                        budget.model,
                        budget.max_context_tokens,
                        budget.input_budget,
                        estimated_input,
                        omitted_count
                    );
                    client
                        .stream_chat_with_context(messages, None)
                        .await
                        .map_err(|e| ReplError::Session(e.to_string()))?
                }
                Err(e) => {
                    tracing::warn!(
                        "ContextWindow assembly failed: {}. P0 overflow occurred with input_budget: {}!",
                        e,
                        budget.input_budget
                    );
                    return Err(ReplError::Session(format!(
                        "Context window assembly failed (P0 overflow) with input_budget: {}: {}",
                        budget.input_budget, e
                    )));
                }
            }
        } else {
            // legacy path: simple 2-message path
            if crate::prompts::is_persona_enabled() {
                let messages = vec![
                    engine_llm_core::ChatMessage {
                        role: "system".into(),
                        content: crate::prompts::load_agent_persona().into(),
                        timestamp: None,
                    },
                    engine_llm_core::ChatMessage {
                        role: "user".into(),
                        content: prompt.clone(),
                        timestamp: None,
                    },
                ];
                client
                    .stream_chat_with_context(messages, None)
                    .await
                    .map_err(|e| ReplError::Session(e.to_string()))?
            } else {
                client
                    .stream_chat(prompt.clone())
                    .await
                    .map_err(|e| ReplError::Session(e.to_string()))?
            }
        };

        let text = collect_stream(&mut stream)
            .await
            .map_err(|e| ReplError::Session(e.to_string()))?;
        if let Some(thinking) = crate::reflector::extract_thinking(&text) {
            if let Some(ref bb) = self.blackboard {
                bb.write("__hajimi_thinking", &thinking, "reflector").await;
            }
        }
        Ok(crate::planner::remove_thinking_tags(&text))
    }
}

/// Collect all Output chunks from a ChannelStream. Returns error on StreamChunk::Error.
pub async fn collect_stream(
    stream: &mut engine_llm_core::ChannelStream,
) -> Result<String, engine_llm_core::EngineError> {
    let mut text = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            engine_llm_core::StreamChunk::Output(s) => text.push_str(&s),
            engine_llm_core::StreamChunk::Error(e) => {
                return Err(engine_llm_core::EngineError::InvalidParameters(e))
            }
            engine_llm_core::StreamChunk::Done => break,
        }
    }
    Ok(text)
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct SubGoalDto {
    description: String,
    priority: crate::planner::Priority,
}

fn mk_subgoal(
    parent: &str,
    desc: &str,
    priority: crate::planner::Priority,
    idx: usize,
) -> crate::planner::SubGoal {
    crate::planner::SubGoal {
        id: format!("{}-sg{}", parent, idx),
        parent_goal: parent.to_string(),
        description: desc.to_string(),
        priority,
        status: crate::planner::PlanStatus::Pending,
        tasks: Vec::new(),
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    }
}

fn mk_task(parent: &str, desc: &str, idx: usize) -> crate::planner::Task {
    crate::planner::Task {
        id: format!("{}-t{}", parent, idx),
        parent_subgoal: parent.to_string(),
        description: desc.to_string(),
        tool_calls: Vec::new(),
        status: crate::planner::PlanStatus::Pending,
        result: None,
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflector::ReflectionLlmClient;
    use std::sync::Mutex;

    /// Serialisation lock for tests that mutate process-wide environment variables.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Mock LlmClient that returns a pre-configured response for testing decompose paths.
    struct MockLlmClient {
        response: String,
    }
    #[async_trait]
    impl engine_llm_core::LlmClient for MockLlmClient {
        async fn stream_chat(
            &self,
            _prompt: String,
        ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
            let (stream, tx) = engine_llm_core::ChannelStream::new(10);
            tx.send(engine_llm_core::StreamChunk::Output(self.response.clone()))
                .await
                .unwrap();
            tx.send(engine_llm_core::StreamChunk::Done).await.unwrap();
            Ok(stream)
        }
        async fn stream_chat_with_context(
            &self,
            _messages: Vec<engine_llm_core::ChatMessage>,
            _system: Option<String>,
        ) -> Result<engine_llm_core::ChannelStream, engine_llm_core::EngineError> {
            self.stream_chat(String::new()).await
        }
        fn provider(&self) -> &engine_llm_core::LlmProvider {
            Box::leak(Box::new(engine_llm_core::LlmProvider::Ollama {
                base_url: "http://localhost:11434".to_string(),
                model: "test".to_string(),
            }))
        }
        fn count_tokens(
            &self,
            _messages: Vec<engine_llm_core::ChatMessage>,
            _model: &str,
        ) -> Result<usize, engine_llm_core::EngineError> {
            Ok(0)
        }
        fn last_usage(&self) -> Option<engine_llm_core::Usage> {
            None
        }
    }

    fn mk_test_goal() -> crate::planner::Goal {
        crate::planner::Goal {
            id: "g1".to_string(),
            description: "test goal".to_string(),
            priority: crate::planner::Priority::High,
            status: crate::planner::PlanStatus::Pending,
            subgoals: vec![],
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            approved: true,
        }
    }

    #[tokio::test]
    async fn test_collect_stream_success() {
        let (mut stream, tx) = engine_llm_core::ChannelStream::new(10);
        tx.send(engine_llm_core::StreamChunk::Output("hello ".into()))
            .await
            .unwrap();
        tx.send(engine_llm_core::StreamChunk::Output("world".into()))
            .await
            .unwrap();
        tx.send(engine_llm_core::StreamChunk::Done).await.unwrap();
        drop(tx);
        assert_eq!(collect_stream(&mut stream).await.unwrap(), "hello world");
    }

    #[tokio::test]
    async fn test_collect_stream_error() {
        let (mut stream, tx) = engine_llm_core::ChannelStream::new(10);
        tx.send(engine_llm_core::StreamChunk::Output("partial".into()))
            .await
            .unwrap();
        tx.send(engine_llm_core::StreamChunk::Error("fail".into()))
            .await
            .unwrap();
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
        let dtos: Vec<SubGoalDto> =
            serde_json::from_str(r#"[{"description":"A","priority":"High"}]"#).unwrap();
        assert_eq!(dtos[0].description, "A");
    }

    #[test]
    fn test_mk_subgoal_and_task() {
        let sg = mk_subgoal("g1", "Desc", crate::planner::Priority::Medium, 0);
        assert_eq!(sg.parent_goal, "g1");
        let t = mk_task("sg1", "Task", 1);
        assert_eq!(t.parent_subgoal, "sg1");
    }

    /// B-06/14: v1 path with valid PlannerSubgoalPlanV1 JSON produces SubGoals.
    /// B-06/14: v1 path with valid PlannerSubgoalPlanV1 JSON produces SubGoals.
    #[tokio::test]
    async fn test_decompose_goal_v1_valid_json() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("HAJIMI_PLANNER_V1_ENABLED", "true");
        let response = r#"{"schema_version":"PlannerSubgoalPlanV1","goal_id":"g1","summary":"test","subgoals":[{"id_hint":"sg1","description":"Analyze","priority":"High","depends_on":[],"suggested_tools":[],"expected_evidence":[],"validation_intent":"None","risk_level":"Low","requires_user_approval":false,"stop_conditions":[]}],"global_risks":[],"notes":[]}"#;
        let bridge = PlannerLlmBridge::new(Arc::new(MockLlmClient {
            response: response.to_string(),
        }));
        let goal = mk_test_goal();
        let sgs = bridge.decompose_goal_v1(&goal).await;
        assert!(
            sgs.is_ok(),
            "v1 valid JSON should produce subgoals: {:?}",
            sgs.err()
        );
        let sgs = sgs.unwrap();
        assert_eq!(sgs.len(), 1);
        assert_eq!(sgs[0].description, "Analyze");
        std::env::remove_var("HAJIMI_PLANNER_V1_ENABLED");
    }

    /// B-06/14: v1 path with invalid JSON returns Err (no panic).
    #[tokio::test]
    async fn test_decompose_goal_v1_invalid_json() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("HAJIMI_PLANNER_V1_ENABLED", "true");
        let bridge = PlannerLlmBridge::new(Arc::new(MockLlmClient {
            response: "not json".to_string(),
        }));
        let goal = mk_test_goal();
        let sgs = bridge.decompose_goal_v1(&goal).await;
        assert!(sgs.is_err(), "v1 invalid JSON should return Err, not panic");
        std::env::remove_var("HAJIMI_PLANNER_V1_ENABLED");
    }

    /// B-06/14: legacy path produces SubGoals from simple JSON array.
    #[tokio::test]
    async fn test_decompose_goal_legacy_path() {
        let response = r#"[{"description":"Legacy subgoal","priority":"Medium"}]"#;
        let bridge = PlannerLlmBridge::new(Arc::new(MockLlmClient {
            response: response.to_string(),
        }));
        let goal = mk_test_goal();
        let sgs = bridge.decompose_goal_legacy(&goal).await;
        assert!(
            sgs.is_ok(),
            "legacy path should produce subgoals: {:?}",
            sgs.err()
        );
        let sgs = sgs.unwrap();
        assert_eq!(sgs.len(), 1);
        assert_eq!(sgs[0].description, "Legacy subgoal");
    }

    /// B-07/14: Reflector v1 path with valid ReflectorCritiqueV1 JSON produces Critique.
    #[tokio::test]
    async fn test_llm_critique_v1_valid_json() {
        let response = r#"{"schema_version":"ReflectorCritiqueV1","success":false,"severity":"High","confidence":0.7,"evidence":["test"],"root_cause":{"category":"ToolFailure","description":"cmd not found","confidence":0.9},"issues":["shell error"],"new_risks":[],"suggestions":["check PATH"],"plan_adjustment":{"action":"RetryWithNewArgs","reason":"fix args","revised_subgoals":[]},"stop_loss":{"triggered":false,"reason":"none","escalation_target":"user"}}"#;
        let bridge = ReflectorLlmBridge::new(Arc::new(MockLlmClient {
            response: response.to_string(),
        }));
        let goal = mk_test_goal();
        let result = crate::planner::TaskResult {
            success: false,
            output: "error".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let critique = bridge.llm_critique(&goal, &result).await;
        assert!(
            critique.is_ok(),
            "v1 valid JSON should produce critique: {:?}",
            critique.err()
        );
        let critique = critique.unwrap();
        assert!(!critique.success);
        assert_eq!(critique.severity, crate::reflector::CritiqueSeverity::High);
        assert!(critique.issues.iter().any(|i| i.contains("shell error")));
    }

    /// B-07/14: Invalid V1 JSON falls back to legacy Critique parsing.
    #[tokio::test]
    async fn test_llm_critique_v1_invalid_json_fallback() {
        let response = r#"{"success":true,"issues":[],"suggestions":["ok"],"severity":"Low"}"#;
        let bridge = ReflectorLlmBridge::new(Arc::new(MockLlmClient {
            response: response.to_string(),
        }));
        let goal = mk_test_goal();
        let result = crate::planner::TaskResult {
            success: true,
            output: "ok".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let critique = bridge.llm_critique(&goal, &result).await;
        assert!(
            critique.is_ok(),
            "legacy JSON fallback should produce critique: {:?}",
            critique.err()
        );
        let critique = critique.unwrap();
        assert!(critique.success);
        assert_eq!(critique.severity, crate::reflector::CritiqueSeverity::Low);
    }

    /// B-07/14: V1 JSON with null optional fields parses successfully.
    #[tokio::test]
    async fn test_llm_critique_v1_partial_fields() {
        let response = r#"{"schema_version":"ReflectorCritiqueV1","success":true,"severity":"Medium","confidence":0.5,"evidence":[],"root_cause":{"category":"None","description":"ok","confidence":1.0},"issues":[],"new_risks":[],"suggestions":[],"plan_adjustment":null,"stop_loss":null}"#;
        let bridge = ReflectorLlmBridge::new(Arc::new(MockLlmClient {
            response: response.to_string(),
        }));
        let goal = mk_test_goal();
        let result = crate::planner::TaskResult {
            success: true,
            output: "done".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let critique = bridge.llm_critique(&goal, &result).await;
        assert!(
            critique.is_ok(),
            "partial V1 JSON should parse: {:?}",
            critique.err()
        );
        let critique = critique.unwrap();
        assert!(critique.success);
        assert_eq!(
            critique.severity,
            crate::reflector::CritiqueSeverity::Medium
        );
    }

    #[test]
    fn test_assemble_messages_for_bridge_success() {
        let budget = crate::context_budget::fast_128k();
        let prompt = "Analyze repository files";
        let sys_content = "Persona instructions";
        let res = assemble_messages_for_bridge(prompt, sys_content, &budget);
        assert!(res.is_ok());
        let (messages, estimated_input, omitted_count) = res.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, sys_content);
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, prompt);
        assert!(estimated_input > 0);
        assert_eq!(omitted_count, 0);
    }

    #[test]
    fn test_assemble_messages_for_bridge_overflow() {
        let mut budget = crate::context_budget::legacy_8k();
        budget.input_budget = 2; // tiny budget
        let prompt = "Analyze repository files";
        let sys_content = "Persona instructions";
        let res = assemble_messages_for_bridge(prompt, sys_content, &budget);
        assert!(res.is_err());
    }
}

/// Helper to assemble messages for bridge.
pub(crate) fn assemble_messages_for_bridge(
    prompt: &str,
    sys_content: &str,
    budget: &crate::context_budget::ContextBudget,
) -> Result<
    (Vec<engine_llm_core::ChatMessage>, usize, usize),
    crate::context_window_manager::ContextError,
> {
    let mgr = crate::context_window_manager::ContextWindowManager::new(budget.input_budget);
    let sys_estimate = crate::context_window_manager::estimate_tokens(sys_content);
    let user_estimate = crate::context_window_manager::estimate_tokens(prompt);

    let blocks = vec![
        crate::context_window_manager::ContextBlock {
            name: "system_prompt".to_string(),
            priority: crate::context_window_manager::ContextPriority::P0,
            content_type: crate::context_window_manager::ContentType::SystemPrompt,
            content: sys_content.to_string(),
            token_estimate: sys_estimate,
            truncatable: false,
        },
        crate::context_window_manager::ContextBlock {
            name: "user_prompt".to_string(),
            priority: crate::context_window_manager::ContextPriority::P0,
            content_type: crate::context_window_manager::ContentType::Text,
            content: prompt.to_string(),
            token_estimate: user_estimate,
            truncatable: false,
        },
    ];

    let assembled = mgr.assemble(blocks)?;
    let mut messages = Vec::new();
    for b in assembled.blocks {
        let role = match b.content_type {
            crate::context_window_manager::ContentType::SystemPrompt => "system",
            _ => "user",
        };
        messages.push(engine_llm_core::ChatMessage {
            role: role.into(),
            content: b.content,
            timestamp: None,
        });
    }
    Ok((messages, assembled.total_tokens, assembled.omitted.len()))
}
