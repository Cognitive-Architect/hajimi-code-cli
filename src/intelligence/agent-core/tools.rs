//! Agent Core Tools: PlanningTool and ReflectionTool for ToolRegistry integration.
//! Day 8: Tool System enhancement with Planning and Reflection capabilities.

use crate::blackboard::Blackboard;
use crate::governance::{AgentGovernance, ApprovalLevel, Decision, GovernanceRequest};
use crate::planner::{Goal, PlanStatus};
use crate::planner::{HierarchicalPlanner, Planner, Priority, TaskResult};
use crate::reflector::{AutonomousReflector, Critique, CritiqueSeverity, Reflector};
use crate::AgentContext;
use async_trait::async_trait;
use engine_tool_system::{PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// PlanningTool: Creates goals, decomposes subgoals, and generates tasks.
pub struct PlanningTool {
    planner: Arc<Mutex<HierarchicalPlanner>>,
    governance: Arc<dyn AgentGovernance>,
    blackboard: Arc<Blackboard>,
}

impl PlanningTool {
    pub fn new(
        planner: Arc<Mutex<HierarchicalPlanner>>,
        governance: Arc<dyn AgentGovernance>,
        blackboard: Arc<Blackboard>,
    ) -> Self {
        Self {
            planner,
            governance,
            blackboard,
        }
    }
    fn validate_args(&self, args: &ToolArgs) -> Result<(String, String), ToolError> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("create_goal");
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if description.len() > 1000 {
            return Err(ToolError::new("Description too long"));
        }
        if description.contains("<script>") || description.contains("${") {
            return Err(ToolError::new("Invalid characters"));
        }
        Ok((action.to_string(), description.to_string()))
    }
    async fn request_approval(&self, action: &str, description: &str) -> Result<bool, ToolError> {
        let req = GovernanceRequest {
            requester: "planning_tool".to_string(),
            action_type: action.to_string(),
            risk_score: 0.5,
            description: description.to_string(),
            level: ApprovalLevel::Required,
        };
        match self.governance.approve(&AgentContext::new(), &req).await {
            Ok(Decision::Approved) => Ok(true),
            Ok(Decision::Rejected(r)) => {
                warn!("PlanningTool rejected: {}", r);
                Ok(false)
            }
            Ok(_) => Ok(false),
            Err(e) => Err(ToolError::new(format!("Governance error: {}", e))),
        }
    }
    async fn write_blackboard(&self, key: &str, value: &str) {
        let _ = self.blackboard.write(key, value, "planning_tool").await;
    }
}

#[async_trait]
impl Tool for PlanningTool {
    fn name(&self) -> &str {
        "planning"
    }
    fn description(&self) -> &str {
        "Creates goals, decomposes subgoals, generates tasks. Actions: create_goal, decompose, next_task."
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        info!("PlanningTool executing: {:?}", args);
        let (action, description) = self.validate_args(&args)?;
        if !self.request_approval(&action, &description).await? {
            return Ok(ToolOutput::error("Planning rejected by governance", 1));
        }
        let mut planner = self.planner.lock().await;
        match action.as_str() {
            "create_goal" => {
                let priority = args
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .map(|s| match s {
                        "critical" => Priority::Critical,
                        "high" => Priority::High,
                        "low" => Priority::Low,
                        _ => Priority::Medium,
                    })
                    .unwrap_or(Priority::Medium);
                match planner.create_goal(&description, priority).await {
                    Ok(goal_id) => {
                        self.write_blackboard("last_goal_id", &goal_id).await;
                        Ok(ToolOutput::success(json!({"goal_id": goal_id}).to_string()))
                    }
                    Err(e) => Ok(ToolOutput::error(format!("Failed: {}", e), 1)),
                }
            }
            "decompose" => {
                let goal_id = args
                    .get("goal_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::new("goal_id required"))?;
                match planner.decompose(&goal_id.to_string()).await {
                    Ok(subgoal_ids) => {
                        self.write_blackboard(
                            "last_subgoals",
                            &json!({"subgoals": &subgoal_ids}).to_string(),
                        )
                        .await;
                        Ok(ToolOutput::success(
                            json!({"subgoals": subgoal_ids, "count": subgoal_ids.len()})
                                .to_string(),
                        ))
                    }
                    Err(e) => Ok(ToolOutput::error(format!("Failed: {}", e), 1)),
                }
            }
            "next_task" => match planner.next_task().await {
                Ok(Some(task)) => {
                    let r = json!({"task_id": task.id, "description": task.description});
                    self.write_blackboard("current_task", &r.to_string()).await;
                    Ok(ToolOutput::success(r.to_string()))
                }
                Ok(None) => Ok(ToolOutput::success("No pending tasks".to_string())),
                Err(e) => Ok(ToolOutput::error(format!("Failed: {}", e), 1)),
            },
            _ => Ok(ToolOutput::error(format!("Unknown action: {}", action), 1)),
        }
    }
}

/// ReflectionTool: Executes reflection cycles and optimizes plans.
pub struct ReflectionTool {
    reflector: Arc<Mutex<AutonomousReflector>>,
    governance: Arc<dyn AgentGovernance>,
    blackboard: Arc<Blackboard>,
}

impl ReflectionTool {
    pub fn new(
        reflector: Arc<Mutex<AutonomousReflector>>,
        governance: Arc<dyn AgentGovernance>,
        blackboard: Arc<Blackboard>,
    ) -> Self {
        Self {
            reflector,
            governance,
            blackboard,
        }
    }
    fn validate_args(&self, args: &ToolArgs) -> Result<(String, String, bool), ToolError> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("reflect");
        let goal_id = args.get("goal_id").and_then(|v| v.as_str()).unwrap_or("");
        let success = args
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if goal_id.len() > 100 {
            return Err(ToolError::new("Goal ID too long"));
        }
        Ok((action.to_string(), goal_id.to_string(), success))
    }
    async fn request_approval(&self, action: &str, goal_id: &str) -> Result<bool, ToolError> {
        let req = GovernanceRequest {
            requester: "reflection_tool".to_string(),
            action_type: action.to_string(),
            risk_score: 0.4,
            description: format!("Reflection for goal {}", goal_id),
            level: ApprovalLevel::Required,
        };
        match self.governance.approve(&AgentContext::new(), &req).await {
            Ok(Decision::Approved) => Ok(true),
            Ok(Decision::Rejected(r)) => {
                warn!("ReflectionTool rejected: {}", r);
                Ok(false)
            }
            Ok(_) => Ok(false),
            Err(e) => Err(ToolError::new(format!("Governance error: {}", e))),
        }
    }
    async fn write_blackboard(&self, key: &str, value: &str) {
        let _ = self.blackboard.write(key, value, "planning_tool").await;
    }
    fn create_goal(&self, id: &str) -> Goal {
        Goal {
            id: id.to_string(),
            description: "Reflected goal".to_string(),
            priority: Priority::Medium,
            status: PlanStatus::InProgress,
            subgoals: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            approved: true,
        }
    }
}

#[async_trait]
impl Tool for ReflectionTool {
    fn name(&self) -> &str {
        "reflection"
    }
    fn description(&self) -> &str {
        "Executes reflection cycles, optimizes plans. Actions: reflect, optimize, get_history."
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        info!("ReflectionTool executing: {:?}", args);
        let (action, goal_id, success) = self.validate_args(&args)?;
        if !self.request_approval(&action, &goal_id).await? {
            return Ok(ToolOutput::error("Reflection rejected by governance", 1));
        }
        let mut reflector = self.reflector.lock().await;
        match action.as_str() {
            "reflect" => {
                let output = args.get("output").and_then(|v| v.as_str()).unwrap_or("");
                let result = TaskResult {
                    success,
                    output: output.to_string(),
                    timestamp: chrono::Utc::now(),
                };
                match reflector
                    .reflect(&self.create_goal(&goal_id), &result)
                    .await
                {
                    Ok(r) => {
                        let res =
                            json!({"reflection_id": r.reflection_id, "confidence": r.confidence});
                        self.write_blackboard(&format!("reflection_{}", goal_id), &res.to_string())
                            .await;
                        Ok(ToolOutput::success(res.to_string()))
                    }
                    Err(e) => Ok(ToolOutput::error(format!("Failed: {}", e), 1)),
                }
            }
            "optimize" => {
                let critique = Critique {
                    success: false,
                    issues: vec!["Optimization requested".to_string()],
                    suggestions: vec!["Improve plan".to_string()],
                    severity: CritiqueSeverity::Medium,
                };
                match reflector
                    .optimize_plan(&self.create_goal(&goal_id), &critique)
                    .await
                {
                    Ok(Some(plan)) => {
                        let res = json!({"optimized": true, "version": plan.version});
                        self.write_blackboard(&format!("optimized_{}", goal_id), &res.to_string())
                            .await;
                        Ok(ToolOutput::success(res.to_string()))
                    }
                    Ok(None) => Ok(ToolOutput::success("No optimization needed".to_string())),
                    Err(e) => Ok(ToolOutput::error(format!("Failed: {}", e), 1)),
                }
            }
            "get_history" => Ok(ToolOutput::success(
                json!({"status": "History in blackboard", "goal_id": goal_id}).to_string(),
            )),
            _ => Ok(ToolOutput::error(format!("Unknown action: {}", action), 1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::DefaultGovernance;
    use memory::memory_gateway::MemoryGateway;
    fn planning_tool() -> PlanningTool {
        let m = Arc::new(Mutex::new(MemoryGateway::new("test")));
        PlanningTool::new(
            Arc::new(Mutex::new(HierarchicalPlanner::new(
                m.clone(),
                AgentContext::new(),
            ))),
            Arc::new(DefaultGovernance::new()),
            Arc::new(Blackboard::new()),
        )
    }
    fn reflection_tool() -> ReflectionTool {
        let m = Arc::new(Mutex::new(MemoryGateway::new("test")));
        ReflectionTool::new(
            Arc::new(Mutex::new(AutonomousReflector::new(
                m.clone(),
                AgentContext::new(),
            ))),
            Arc::new(DefaultGovernance::new()),
            Arc::new(Blackboard::new()),
        )
    }
    #[tokio::test]
    async fn test_planning_name() {
        assert_eq!(planning_tool().name(), "planning");
    }
    #[tokio::test]
    async fn test_reflection_name() {
        assert_eq!(reflection_tool().name(), "reflection");
    }
    #[tokio::test]
    async fn test_planning_perms() {
        assert!(planning_tool().permissions().requires_confirmation);
    }
    #[tokio::test]
    async fn test_reflection_perms() {
        assert!(reflection_tool().permissions().requires_confirmation);
    }
    #[tokio::test]
    async fn test_concurrent_invoke() {
        let t1 = Arc::new(planning_tool());
        let t2 = Arc::new(reflection_tool());
        let f1 = tokio::spawn({
            let t = t1.clone();
            async move {
                t.execute(json!({"action": "create_goal", "description": "Test"}))
                    .await
            }
        });
        let f2 = tokio::spawn({
            let t = t2.clone();
            async move {
                t.execute(json!({"action": "get_history", "goal_id": "g1"}))
                    .await
            }
        });
        assert!(f1.await.is_ok() && f2.await.is_ok());
    }
}
