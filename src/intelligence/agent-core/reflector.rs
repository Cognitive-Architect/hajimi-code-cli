//! Reflector: Autonomous reflection loop with LLM-driven critique and memory integration.
//! Day 4: Reflection cycle (analysis → critique → optimization → Dream/Graph persistence)
//! DEBT: Swarm coordination deferred to Phase 5.

use crate::governance::{AgentGovernance, DefaultGovernance};
use crate::multi_worker_aggregator::MultiWorkerAggregator;
use crate::plan_optimizer::PlanOptimizer;
use crate::planner::{Goal, Plan, TaskResult};
use crate::reflection_persistence::ReflectionPersistence;
use crate::swarm::WorkerResult;
use crate::AgentContext;
use async_trait::async_trait;
use chimera_repl::traits::{ReplError, ReplResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Extract thinking content from LLM response enclosed in <thinking> tags.
/// Returns None if no thinking block is found.
pub fn extract_thinking(text: &str) -> Option<String> {
    let start = text.find("<thinking>")?;
    let end = text.find("</thinking>")?;
    if end > start {
        Some(text[start + 10..end].trim().to_string())
    } else {
        None
    }
}

/// Format instruction appended to LLM prompts to elicit structured thinking output.
pub const THINKING_FORMAT_INSTRUCTION: &str = r#"Before responding, wrap your reasoning in <thinking>...</thinking> tags, then provide your final answer in <response>...</response> tags."#;

/// Maximum reflection iterations to prevent infinite loops.
const MAX_REFLECTION_STEPS: usize = 10;
/// Token budget for reflection to prevent resource exhaustion.
const REFLECTION_TOKEN_BUDGET: usize = 4096;

/// Reflection outcome with actionable insights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub reflection_id: String,
    pub original_goal_id: String,
    pub execution_result: TaskResult,
    pub critique: Critique,
    pub optimized_plan: Option<Plan>,
    pub confidence: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Structured critique from LLM or rule-based analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    pub success: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub severity: CritiqueSeverity,
}

/// Severity levels for critique issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CritiqueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Core Reflector trait for autonomous reflection.
#[async_trait]
pub trait Reflector: Send + Sync {
    /// Execute full reflection cycle on execution result.
    async fn reflect(&mut self, goal: &Goal, result: &TaskResult) -> ReplResult<Reflection>;
    /// Execute reflection cycle on multiple worker results.
    async fn reflect_multi(
        &mut self,
        goal: &Goal,
        results: &[WorkerResult],
    ) -> ReplResult<Reflection>;
    /// Critique execution result (LLM-driven or rule-based).
    async fn critique(&self, goal: &Goal, result: &TaskResult) -> ReplResult<Critique>;
    /// Generate optimized plan based on critique.
    async fn optimize_plan(&self, goal: &Goal, critique: &Critique) -> ReplResult<Option<Plan>>;
    /// Persist reflection to Dream and Graph layers.
    async fn persist_reflection(&self, reflection: &Reflection) -> ReplResult<()>;
    /// Governance hook for reflection approval.
    async fn approve_reflection(&self, reflection: &Reflection) -> ReplResult<bool>;
}

/// LLM client extension for reflection-specific operations.
#[async_trait]
pub trait ReflectionLlmClient: Send + Sync {
    /// Critique execution result using LLM reasoning.
    async fn llm_critique(&self, goal: &Goal, result: &TaskResult) -> ReplResult<Critique>;
    /// Generate plan optimization suggestions.
    async fn llm_optimize(&self, goal: &Goal, critique: &Critique) -> ReplResult<String>;
}

/// Reflector implementation with MemoryGateway integration.
pub struct AutonomousReflector {
    #[allow(dead_code)]
    context: AgentContext,
    llm: Option<Arc<dyn ReflectionLlmClient>>,
    reflection_count: usize,
    token_used: usize,
    governance: Arc<dyn AgentGovernance>,
    persistence: ReflectionPersistence,
    plan_optimizer: PlanOptimizer,
    /// Last extracted thinking content from LLM response (B-06/12).
    pub thinking_content: Option<String>,
}

impl AutonomousReflector {
    /// Create new reflector with memory gateway.
    pub fn new(
        memory: Arc<Mutex<memory::memory_gateway::MemoryGateway>>,
        context: AgentContext,
    ) -> Self {
        let persistence = ReflectionPersistence::new(memory);
        let plan_optimizer = PlanOptimizer::new(None);
        Self {
            context,
            llm: None,
            reflection_count: 0,
            token_used: 0,
            governance: Arc::new(DefaultGovernance::new()),
            persistence,
            plan_optimizer,
            thinking_content: None,
        }
    }
    /// Attach governance engine for reflection approval.
    pub fn with_governance(mut self, gov: Arc<dyn AgentGovernance>) -> Self {
        self.governance = gov;
        self
    }

    /// Attach LLM client for dynamic critique.
    pub fn with_llm(mut self, llm: Arc<dyn ReflectionLlmClient>) -> Self {
        self.llm = Some(llm.clone());
        self.plan_optimizer = PlanOptimizer::new(Some(llm));
        self
    }

    /// Check if reflection budget exhausted.
    fn budget_exhausted(&self) -> bool {
        self.reflection_count >= MAX_REFLECTION_STEPS || self.token_used >= REFLECTION_TOKEN_BUDGET
    }

    /// Rule-based critique when LLM unavailable.
    fn critique_rule_based(&self, _goal: &Goal, result: &TaskResult) -> Critique {
        if result.success {
            Critique {
                success: true,
                issues: vec![],
                suggestions: vec!["Continue with next task".to_string()],
                severity: CritiqueSeverity::Low,
            }
        } else {
            Critique {
                success: false,
                issues: vec![result.output.clone()],
                suggestions: vec!["Retry with modified parameters".to_string()],
                severity: CritiqueSeverity::High,
            }
        }
    }
}

#[async_trait]
impl Reflector for AutonomousReflector {
    /// Execute full reflection cycle: analyze → critique → optimize → persist.
    async fn reflect(&mut self, goal: &Goal, result: &TaskResult) -> ReplResult<Reflection> {
        if self.budget_exhausted() {
            return Err(ReplError::Session(
                "Reflection budget exhausted".to_string(),
            ));
        }
        self.reflection_count += 1;

        let critique = self.critique(goal, result).await?;
        let optimized_plan = self.optimize_plan(goal, &critique).await?;

        let reflection = Reflection {
            reflection_id: uuid::Uuid::new_v4().to_string(),
            original_goal_id: goal.id.clone(),
            execution_result: result.clone(),
            critique,
            optimized_plan,
            confidence: if result.success { 0.9 } else { 0.5 },
            timestamp: chrono::Utc::now(),
        };

        if !self.approve_reflection(&reflection).await? {
            return Err(ReplError::Session(
                "Reflection rejected by governance".to_string(),
            ));
        }

        self.persist_reflection(&reflection).await?;
        Ok(reflection)
    }

    /// Execute reflection cycle on multiple worker results: aggregate → critique → optimize → persist.
    async fn reflect_multi(
        &mut self,
        goal: &Goal,
        results: &[WorkerResult],
    ) -> ReplResult<Reflection> {
        if self.budget_exhausted() {
            return Err(ReplError::Session(
                "Reflection budget exhausted".to_string(),
            ));
        }
        self.reflection_count += 1;

        if results.is_empty() {
            return Ok(Reflection {
                reflection_id: uuid::Uuid::new_v4().to_string(),
                original_goal_id: goal.id.clone(),
                execution_result: TaskResult {
                    success: false,
                    output: "No worker results".to_string(),
                    timestamp: chrono::Utc::now(),
                },
                critique: Critique {
                    success: false,
                    issues: vec!["Empty worker result set".to_string()],
                    suggestions: vec!["Dispatch workers before reflecting".to_string()],
                    severity: CritiqueSeverity::Low,
                },
                optimized_plan: None,
                confidence: 0.0,
                timestamp: chrono::Utc::now(),
            });
        }

        let total_time: u64 = results
            .iter()
            .filter_map(|r| r.metrics.as_ref().map(|m| m.execution_time_ms))
            .sum();
        let avg_time = total_time as f32 / results.len().max(1) as f32;
        let success_count = results.iter().filter(|r| r.success).count();

        let (critique, confidence) = MultiWorkerAggregator::aggregate_results(results);
        let optimized_plan = self.optimize_plan(goal, &critique).await?;

        let reflection = Reflection {
            reflection_id: uuid::Uuid::new_v4().to_string(),
            original_goal_id: goal.id.clone(),
            execution_result: TaskResult {
                success: confidence > 0.5,
                output: format!(
                    "Multi-worker: {}/{} succeeded, avg_time={:.1}ms",
                    success_count,
                    results.len(),
                    avg_time
                ),
                timestamp: chrono::Utc::now(),
            },
            critique,
            optimized_plan,
            confidence,
            timestamp: chrono::Utc::now(),
        };

        if !self.approve_reflection(&reflection).await? {
            return Err(ReplError::Session(
                "Reflection rejected by governance".to_string(),
            ));
        }
        self.persist_reflection(&reflection).await?;
        Ok(reflection)
    }

    /// Critique execution result using LLM if available, otherwise rule-based.
    async fn critique(&self, goal: &Goal, result: &TaskResult) -> ReplResult<Critique> {
        if let Some(ref llm) = self.llm {
            if let Ok(critique) = llm.llm_critique(goal, result).await {
                return Ok(critique);
            }
        }
        Ok(self.critique_rule_based(goal, result))
    }

    /// Generate optimized plan based on critique.
    async fn optimize_plan(&self, goal: &Goal, critique: &Critique) -> ReplResult<Option<Plan>> {
        self.plan_optimizer.optimize(goal, critique).await
    }

    /// Persist reflection to Dream and Graph layers.
    async fn persist_reflection(&self, reflection: &Reflection) -> ReplResult<()> {
        self.persistence.persist(reflection).await
    }

    /// Governance hook for reflection approval - integrated with AgentGovernance trait.
    async fn approve_reflection(&self, reflection: &Reflection) -> ReplResult<bool> {
        self.persistence
            .approve(reflection, self.governance.as_ref(), &self.context)
            .await
    }
}

#[cfg(test)]
mod tests;
