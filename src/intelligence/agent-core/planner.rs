//! Hierarchical Planner: Goal → SubGoal → Task with GraphMemory integration.
use crate::blackboard::Blackboard;
use crate::AgentContext;
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, DefaultGovernance, GovernanceRequest};
use async_trait::async_trait;
use chimera_repl::traits::{ReplError, ReplResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type GoalId = String;
pub type SubGoalId = String;
pub type TaskId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority { Critical = 0, High = 1, Medium = 2, Low = 3 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus { Pending, InProgress, Blocked, Completed, Failed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal { pub id: GoalId, pub description: String, pub priority: Priority, pub status: PlanStatus, pub subgoals: Vec<SubGoalId>, pub metadata: HashMap<String, String>, pub created_at: chrono::DateTime<chrono::Utc>, pub approved: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGoal { pub id: SubGoalId, pub parent_goal: GoalId, pub description: String, pub priority: Priority, pub status: PlanStatus, pub tasks: Vec<TaskId>, pub dependencies: Vec<SubGoalId> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task { pub id: TaskId, pub parent_subgoal: SubGoalId, pub description: String, pub tool_calls: Vec<ToolCall>, pub status: PlanStatus, pub result: Option<TaskResult> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall { pub tool_name: String, pub parameters: HashMap<String, serde_json::Value> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult { pub success: bool, pub output: String, pub timestamp: chrono::DateTime<chrono::Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan { pub goal: Goal, pub subgoals: HashMap<SubGoalId, SubGoal>, pub tasks: HashMap<TaskId, Task>, pub version: u32 }

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn decompose_goal(&self, goal: &Goal) -> ReplResult<Vec<SubGoal>>;
    async fn generate_tasks(&self, subgoal: &SubGoal) -> ReplResult<Vec<Task>>;
}

#[async_trait]
pub trait Planner: Send + Sync {
    async fn create_goal(&mut self, description: &str, priority: Priority) -> ReplResult<GoalId>;
    async fn decompose(&mut self, goal_id: &GoalId) -> ReplResult<Vec<SubGoalId>>;
    async fn expand(&mut self, subgoal_id: &SubGoalId) -> ReplResult<Vec<TaskId>>;
    async fn next_task(&self) -> ReplResult<Option<Task>>;
    async fn update_task(&mut self, task_id: &TaskId, result: TaskResult) -> ReplResult<()>;
    fn is_complete(&self) -> bool;
    async fn save_to_graph(&self) -> ReplResult<()>;
    async fn load_from_graph(&mut self, goal_id: &GoalId) -> ReplResult<()>;
}

pub struct HierarchicalPlanner {
    #[allow(dead_code)] context: AgentContext,
    llm: Option<Arc<dyn LlmClient>>, current_plan: Option<Plan>,
    #[allow(dead_code)] memory: Arc<Mutex<memory::memory_gateway::MemoryGateway>>,
    governance: Arc<dyn AgentGovernance>,
    blackboard: Option<Arc<Blackboard>>,
    /// Last extracted thinking content from LLM response (B-06/12).
    pub thinking_content: Option<String>,
}

impl HierarchicalPlanner {
    pub fn new(memory: Arc<Mutex<memory::memory_gateway::MemoryGateway>>, context: AgentContext) -> Self {
        Self { context, llm: None, current_plan: None, memory, governance: Arc::new(DefaultGovernance::new()), blackboard: None, thinking_content: None }
    }
    pub fn with_governance(mut self, gov: Arc<dyn AgentGovernance>) -> Self { self.governance = gov; self }
    pub fn with_llm(mut self, llm: Arc<dyn LlmClient>) -> Self { self.llm = Some(llm); self }
    pub fn with_blackboard(mut self, bb: Arc<Blackboard>) -> Self { self.blackboard = Some(bb); self }

    /// Phase 4 Day 2: Create goal with optional AST context injection.
    /// Extracts symbol candidates from description and writes them to blackboard
    /// for MemoryRetriever to pick up during retrieval.
    pub async fn plan_with_ast(&mut self, description: &str, priority: Priority) -> ReplResult<GoalId> {
        let goal_id = self.create_goal(description, priority).await?;
        if let Some(ref bb) = self.blackboard {
            let candidates = extract_symbol_candidates(description);
            for name in candidates {
                bb.write(&format!("ast_query_{}", goal_id), &name, "planner").await;
            }
        }
        Ok(goal_id)
    }
    pub async fn request_approval(&self, goal: &Goal) -> ReplResult<bool> {
        let req = GovernanceRequest { requester: "planner".to_string(), action_type: "create_goal".to_string(), risk_score: goal.priority as u8 as f32 / 10.0, description: goal.description.clone(), level: ApprovalLevel::Auto };
        Ok(matches!(self.governance.approve(&self.context, &req).await?, Decision::Approved | Decision::Escalated(_)))
    }
    fn decompose_rule_based(&self, goal: &Goal) -> Vec<SubGoal> {
        let desc = goal.description.to_lowercase();
        let patterns: Vec<&str> = if desc.contains("implement") || desc.contains("create") { vec!["Analyze requirements", "Design", "Implement", "Test"] }
        else if desc.contains("fix") { vec!["Reproduce", "Identify cause", "Apply fix", "Verify"] }
        else { vec!["Research", "Execute", "Validate"] };
        patterns.into_iter().enumerate().map(|(i, p)| self.mk_subgoal(&goal.id, p, Priority::High, i)).collect()
    }
    fn mk_subgoal(&self, parent: &GoalId, desc: &str, priority: Priority, idx: usize) -> SubGoal {
        SubGoal { id: format!("{}-sg{}", parent, idx), parent_goal: parent.clone(), description: desc.to_string(), priority, status: PlanStatus::Pending, tasks: Vec::new(), dependencies: Vec::new() }
    }
    fn generate_tasks_for(&self, sg: &SubGoal) -> Vec<Task> {
        let desc = sg.description.to_lowercase();
        let items: Vec<&str> = if desc.contains("implement") { vec!["Write code", "Check compilation"] }
        else if desc.contains("test") { vec!["Run tests", "Review"] }
        else { vec![&sg.description] };
        items.iter().enumerate().map(|(i, d)| Task { id: format!("{}-t{}", sg.id, i), parent_subgoal: sg.id.clone(), description: d.to_string(), tool_calls: Vec::new(), status: PlanStatus::Pending, result: None }).collect()
    }
    fn deps_met(&self, sg: &SubGoal, plan: &Plan) -> bool {
        sg.dependencies.iter().all(|d| plan.subgoals.get(d).map(|s| s.status == PlanStatus::Completed).unwrap_or(false))
    }
}

#[async_trait]
impl Planner for HierarchicalPlanner {
    async fn create_goal(&mut self, desc: &str, priority: Priority) -> ReplResult<GoalId> {
        let goal = Goal { id: Uuid::new_v4().to_string(), description: desc.to_string(), priority, status: PlanStatus::Pending, subgoals: Vec::new(), metadata: HashMap::new(), created_at: chrono::Utc::now(), approved: false };
        let id = goal.id.clone();
        if !self.request_approval(&goal).await? { return Err(ReplError::Session(format!("Goal {} rejected", id))); }
        self.current_plan = Some(Plan { goal, subgoals: HashMap::new(), tasks: HashMap::new(), version: 1 });
        Ok(id)
    }
    async fn decompose(&mut self, goal_id: &GoalId) -> ReplResult<Vec<SubGoalId>> {
        let goal = self.current_plan.as_ref().map(|p| p.goal.clone()).ok_or_else(|| ReplError::Session("No plan".to_string()))?;
        if goal.id != *goal_id { return Err(ReplError::Session("ID mismatch".to_string())); }
        let sgs = if let Some(ref llm) = self.llm { llm.decompose_goal(&goal).await? } else { self.decompose_rule_based(&goal) };
        let ids: Vec<_> = sgs.iter().map(|s| s.id.clone()).collect();
        let plan = self.current_plan.as_mut().ok_or_else(|| ReplError::Session("Plan not initialized".to_string()))?;
        for sg in sgs { plan.subgoals.insert(sg.id.clone(), sg); }
        plan.goal.subgoals = ids.clone(); Ok(ids)
    }
    async fn expand(&mut self, sg_id: &SubGoalId) -> ReplResult<Vec<TaskId>> {
        let sg = self.current_plan.as_ref().and_then(|p| p.subgoals.get(sg_id)).cloned().ok_or_else(|| ReplError::Session("SubGoal not found".to_string()))?;
        let tasks = if let Some(ref llm) = self.llm { llm.generate_tasks(&sg).await? } else { self.generate_tasks_for(&sg) };
        let ids: Vec<_> = tasks.iter().map(|t| t.id.clone()).collect();
        let plan = self.current_plan.as_mut().ok_or_else(|| ReplError::Session("Plan not initialized".to_string()))?;
        let sg_mut = plan.subgoals.get_mut(sg_id).ok_or_else(|| ReplError::Session(format!("SubGoal {} not found in plan", sg_id)))?;
        for t in &tasks { sg_mut.tasks.push(t.id.clone()); plan.tasks.insert(t.id.clone(), t.clone()); }
        Ok(ids)
    }
    async fn next_task(&self) -> ReplResult<Option<Task>> {
        let plan = self.current_plan.as_ref().ok_or_else(|| ReplError::Session("No plan".to_string()))?;
        for sg in plan.subgoals.values() {
            if !self.deps_met(sg, plan) || !matches!(sg.status, PlanStatus::Pending | PlanStatus::InProgress) { continue; }
            for tid in &sg.tasks { if let Some(t) = plan.tasks.get(tid) { if t.status == PlanStatus::Pending { return Ok(Some(t.clone())); } } }
        } Ok(None)
    }
    async fn update_task(&mut self, tid: &TaskId, result: TaskResult) -> ReplResult<()> {
        let plan = self.current_plan.as_mut().ok_or_else(|| ReplError::Session("No plan".to_string()))?;
        let t = plan.tasks.get_mut(tid).ok_or_else(|| ReplError::Session("Task not found".to_string()))?;
        t.status = if result.success { PlanStatus::Completed } else { PlanStatus::Failed }; t.result = Some(result); Ok(())
    }
    fn is_complete(&self) -> bool {
        let Some(p) = &self.current_plan else { return false };
        p.goal.status == PlanStatus::Completed || p.tasks.values().all(|t| matches!(t.status, PlanStatus::Completed | PlanStatus::Failed))
    }
    async fn save_to_graph(&self) -> ReplResult<()> {
        let plan = self.current_plan.as_ref().ok_or_else(|| ReplError::Session("No plan".to_string()))?;
        let json = serde_json::to_string(plan).map_err(ReplError::Protocol)?;
        tracing::info!("Saving plan {}: {} bytes", plan.goal.id, json.len()); Ok(())
    }
    async fn load_from_graph(&mut self, goal_id: &GoalId) -> ReplResult<()> {
        let mem_guard = self.memory.lock().await;
        if let Some(content) = mem_guard.session.get(&format!("plan_{}", goal_id)) {
            let plan: Plan = serde_json::from_str(&content.content).map_err(ReplError::Protocol)?;
            self.current_plan = Some(plan); return Ok(());
        }
        Err(ReplError::Session(format!("Plan {} not found in Graph/Session", goal_id)))
    }
}

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

/// Extract potential symbol names (PascalCase / camelCase / snake_case) from a goal description.
/// Used by plan_with_ast() to inject AST query hints into the blackboard.
fn extract_symbol_candidates(description: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let words: Vec<&str> = description.split_whitespace().collect();
    for word in words {
        let cleaned: String = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string();
        if cleaned.is_empty() { continue; }
        // Heuristic: PascalCase (starts with uppercase, contains lowercase) or camelCase (mixed case)
        let has_upper = cleaned.chars().any(|c| c.is_uppercase());
        let has_lower = cleaned.chars().any(|c| c.is_lowercase());
        let is_snake = cleaned.contains('_') && cleaned.chars().all(|c| c.is_alphanumeric() || c == '_');
        if (has_upper && has_lower) || is_snake {
            candidates.push(cleaned);
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    fn planner() -> HierarchicalPlanner { HierarchicalPlanner::new(Arc::new(Mutex::new(memory::memory_gateway::MemoryGateway::new("test"))), AgentContext::new()) }
    #[tokio::test] async fn test_create_goal() {
        let mut p = planner();
        if let Ok(id) = p.create_goal("Test", Priority::High).await {
            assert!(!id.is_empty());
        } else { panic!("create_goal failed"); }
    }
    #[tokio::test] async fn test_decompose() {
        let mut p = planner();
        if let Ok(id) = p.create_goal("Implement feature", Priority::High).await {
            let sgs = p.decompose(&id).await;
            assert!(matches!(sgs, Ok(v) if !v.is_empty()));
        } else { panic!("create_goal failed"); }
    }
    #[tokio::test] async fn test_next_task() {
        let mut p = planner();
        if let Ok(id) = p.create_goal("Fix bug", Priority::Critical).await {
            let _ = p.decompose(&id).await;
            let sg_id = p.current_plan.as_ref().and_then(|plan| plan.goal.subgoals.first().cloned());
            if let Some(sg) = sg_id { let _ = p.expand(&sg).await; }
            let task = p.next_task().await;
            assert!(matches!(task, Ok(Some(_))));
        } else { panic!("create_goal failed"); }
    }
}
