use crate::agent_loop::AgentLoop;
use crate::agent_loop_builder::AgentLoopBuilder;
use crate::blackboard::Blackboard;
use crate::checkpoint::{Checkpoint, CheckpointManager};
use crate::{AgentContext, AgentError};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

/// Result of loading project memory, carrying the shared gateway and restored checkpoint state.
pub struct BootstrapResult {
    pub gateway: Arc<Mutex<memory::memory_gateway::MemoryGateway>>,
    pub checkpoint_mgr: CheckpointManager,
    pub summary: String,
}

/// # Safety: MemoryBootstrapper coordinates initialization order to prevent race conditions
/// between Checkpoint restore and MemoryGateway tier enablement. All async operations
/// are serialized within each bootstrap call.
pub struct MemoryBootstrapper {
    project_id: String,
    device_id: String,
    agent_id: String,
    llm_client: Option<Arc<dyn engine_llm_core::LlmClient>>,
}

impl MemoryBootstrapper {
    pub fn new(project_id: &str, device_id: &str, agent_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            device_id: device_id.to_string(),
            agent_id: agent_id.to_string(),
            llm_client: None,
        }
    }

    pub fn with_llm_client(mut self, client: Arc<dyn engine_llm_core::LlmClient>) -> Self {
        self.llm_client = Some(client);
        self
    }

    /// Initialize MemoryGateway, enable Auto/Graph/Dream tiers, restore latest Checkpoint,
    /// and generate a human-readable project memory summary.
    pub async fn load_project_memory(&self) -> Result<BootstrapResult, AgentError> {
        let mut gateway =
            memory::memory_gateway::MemoryGateway::new_with_project(&self.device_id, Some(&self.project_id));
        let _ = gateway.enable_auto(&self.project_id);
        gateway.enable_graph(&self.project_id);
        let _ = gateway.enable_dream(&self.project_id);
        let gateway_arc = Arc::new(Mutex::new(gateway));
        let checkpoint_mgr = CheckpointManager::new().with_memory(gateway_arc.clone());
        let checkpoint = checkpoint_mgr
            .restore_latest_from_disk(&self.project_id, &self.agent_id)
            .await
            .ok();

        let summary = match checkpoint {
            Some(ref cp) => {
                if let Some(cached) = self.load_summary_from_disk().await {
                    cached
                } else {
                    let ctx = Self::collect_context(cp);
                    let prompt = Self::build_summary_prompt(&ctx);
                    match self.generate_natural_language_summary(&prompt).await {
                        Ok(s) => {
                            let _ = self.save_summary_to_disk(&s).await;
                            s
                        }
                        Err(e) => {
                            warn!("LLM summary failed, fallback to raw: {}", e);
                            Self::format_raw_summary(cp)
                        }
                    }
                }
            }
            None => "No checkpoint available".to_string(),
        };

        Ok(BootstrapResult {
            gateway: gateway_arc,
            checkpoint_mgr,
            summary,
        })
    }

    /// Build a fully configured AgentLoop using the shared MemoryGateway from load_project_memory.
    /// Injects project_memory_summary into the Blackboard before returning.
    pub async fn build_agent_loop_with_memory(&self) -> Result<AgentLoop, AgentError> {
        let result = self.load_project_memory().await?;
        let blackboard = Arc::new(Blackboard::new());
        blackboard
            .write("project_memory_summary", &result.summary, &self.agent_id)
            .await;
        let context = AgentContext::new();
        let planner = Arc::new(Mutex::new(crate::planner::HierarchicalPlanner::new(
            result.gateway.clone(),
            context.clone(),
        )));
        let reflector = Arc::new(Mutex::new(crate::reflector::AutonomousReflector::new(
            result.gateway.clone(),
            context.clone(),
        )));
        AgentLoopBuilder::production_ready(&self.device_id)
            .with_context(context)
            .with_planner(planner)
            .with_reflector(reflector)
            .with_memory(Some(result.gateway.clone()))
            .with_blackboard(blackboard)
            .with_checkpoint_mgr(Arc::new(result.checkpoint_mgr))
            .build()
    }

    /// Legacy key-value summary format (retained for backward compatibility).
    #[allow(dead_code)]
    fn generate_summary(checkpoint: &Option<Checkpoint>) -> String {
        match checkpoint {
            Some(cp) => {
                let plan_summary = cp.plan.as_ref().map(|_| "active").unwrap_or("none");
                let reflection_count = cp.reflections.len();
                let goal_progress = cp
                    .goal_progress
                    .map(|p| format!("{:.0}%", p * 100.0))
                    .unwrap_or_else(|| "N/A".to_string());
                format!(
                    "plan_summary={}; reflections={}; goal_progress={}",
                    plan_summary, reflection_count, goal_progress
                )
            }
            None => "No checkpoint available".to_string(),
        }
    }

    // --- Natural Language Summary (B-02/17 + B-03/17) ---

    /// Aggregate checkpoint data into context segments for prompt construction.
    /// Retains at most the 5 most recent reflections to stay within token budget.
    fn collect_context(checkpoint: &Checkpoint) -> SummaryContext {
        let plan_desc = checkpoint
            .plan
            .as_ref()
            .map(|p| p.goal.description.clone())
            .unwrap_or_else(|| "无活跃计划".to_string());

        let mut reflections_text = String::new();
        for (i, r) in checkpoint.reflections.iter().rev().take(5).enumerate() {
            let issues = r.critique.issues.join("、");
            let suggestions = r.critique.suggestions.join("、");
            reflections_text.push_str(&format!(
                "反思{}: 问题[{}] 建议[{}]\n",
                i + 1,
                if issues.is_empty() { "无" } else { &issues },
                if suggestions.is_empty() { "无" } else { &suggestions }
            ));
        }

        let goal_prog = checkpoint
            .goal_progress
            .map(|p| format!("{:.0}%", p * 100.0))
            .unwrap_or_else(|| "未知".to_string());

        let mut blackboard_keys = Vec::new();
        for (k, v) in &checkpoint.blackboard {
            if k.starts_with("project_") || k.starts_with("dialogue_") {
                let preview = &v.value[..v.value.len().min(40)];
                blackboard_keys.push(format!("{}={}", k, preview));
            }
        }

        SummaryContext {
            plan_description: plan_desc,
            reflections: reflections_text,
            goal_progress: goal_prog,
            blackboard_hints: blackboard_keys.join("; "),
        }
    }

    fn build_summary_prompt(ctx: &SummaryContext) -> String {
        format!(
            "你是一个代码项目的记忆助手。以下是项目的上下文信息：\n\n\
             【最近计划】\n{}\n\n\
             【反思记录】\n{}\n\n\
             【目标进度】\n{}\n\n\
             【关键状态】\n{}\n\n\
             请用自然语言、第一人称总结：\n\
             1. 上次工作到哪里了？\n\
             2. 当前正在解决什么问题？\n\
             3. 下一步计划是什么？\n\n\
             要求：简洁（<200字）、口语化、突出关键决策。",
            ctx.plan_description,
            if ctx.reflections.is_empty() { "暂无反思记录" } else { &ctx.reflections },
            ctx.goal_progress,
            if ctx.blackboard_hints.is_empty() { "无" } else { &ctx.blackboard_hints }
        )
    }

    async fn generate_natural_language_summary(
        &self,
        prompt: &str,
    ) -> Result<String, AgentError> {
        let client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| AgentError::Session("LLM client not configured".to_string()))?;
        let mut stream = client
            .stream_chat(prompt.to_string())
            .await
            .map_err(|e| AgentError::Session(format!("LLM stream error: {}", e)))?;
        crate::llm::bridge::collect_stream(&mut stream)
            .await
            .map_err(|e| AgentError::Session(format!("LLM collect error: {}", e)))
    }

    fn format_raw_summary(checkpoint: &Checkpoint) -> String {
        let plan_desc = checkpoint
            .plan
            .as_ref()
            .map(|p| &*p.goal.description)
            .unwrap_or("无计划");
        let reflection_count = checkpoint.reflections.len();
        let goal_progress = checkpoint
            .goal_progress
            .map(|p| format!("{:.0}%", p * 100.0))
            .unwrap_or_else(|| "N/A".to_string());
        format!(
            "⭐ 项目状态摘要\n===\n📋 计划: {}\n📝 反思数: {}\n🎯 进度: {}\n===",
            plan_desc, reflection_count, goal_progress
        )
    }

    /// Save natural-language summary to ~/.hajimi/memory/{project_id}/summary.md
    pub(crate) async fn save_summary_to_disk(&self, summary: &str) -> std::io::Result<()> {
        let base = dirs::config_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "config dir not found"))?;
        let dir = base.join(".hajimi").join("memory").join(&self.project_id);
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::write(dir.join("summary.md"), summary).await
    }

    /// Load natural-language summary from disk if it exists.
    pub(crate) async fn load_summary_from_disk(&self) -> Option<String> {
        let base = dirs::config_dir()?;
        let path = base.join(".hajimi").join("memory").join(&self.project_id).join("summary.md");
        if !path.exists() {
            return None;
        }
        match tokio::fs::read_to_string(&path).await {
            Ok(s) if !s.trim().is_empty() => Some(s),
            _ => None,
        }
    }
}

/// Context segments aggregated from a Checkpoint for prompt construction.
struct SummaryContext {
    plan_description: String,
    reflections: String,
    goal_progress: String,
    blackboard_hints: String,
}
