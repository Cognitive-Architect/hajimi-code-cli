//! DEBT-LINES-B0301B: Extracted from reflector.rs.
use crate::planner::{Goal, Plan, PlanStatus, Priority, SubGoal};
use crate::reflector::{Critique, CritiqueSeverity, ReflectionLlmClient};
use chimera_repl::traits::ReplResult;
use std::collections::HashMap;
use std::sync::Arc;

pub struct PlanOptimizer {
    llm: Option<Arc<dyn ReflectionLlmClient>>,
}

impl PlanOptimizer {
    pub fn new(llm: Option<Arc<dyn ReflectionLlmClient>>) -> Self {
        Self { llm }
    }

    pub async fn optimize(&self, goal: &Goal, critique: &Critique) -> ReplResult<Option<Plan>> {
        if critique.success || critique.severity == CritiqueSeverity::Low {
            return Ok(None);
        }
        if let Some(ref llm) = self.llm {
            if llm.llm_optimize(goal, critique).await.is_ok() {
                return Ok(Some(self.build_fix_plan(goal, critique)));
            }
        }
        Ok(None)
    }

    pub fn apply_critique(&self, goal: &Goal, critique: &Critique) -> Option<Plan> {
        if critique.success || critique.severity == CritiqueSeverity::Low {
            return None;
        }
        Some(self.build_fix_plan(goal, critique))
    }

    fn build_fix_plan(&self, goal: &Goal, critique: &Critique) -> Plan {
        let mut plan = Plan {
            goal: goal.clone(),
            subgoals: HashMap::new(),
            tasks: HashMap::new(),
            version: 2,
        };
        if critique.severity == CritiqueSeverity::High
            || critique.severity == CritiqueSeverity::Critical
        {
            plan.subgoals.insert(
                format!("{}-fix", goal.id),
                SubGoal {
                    id: format!("{}-fix", goal.id),
                    parent_goal: goal.id.clone(),
                    description: format!("Fix: {}", critique.issues.join(", ")),
                    priority: Priority::Critical,
                    status: PlanStatus::Pending,
                    tasks: vec![],
                    dependencies: vec![],
                    metadata: HashMap::new(),
                },
            );
        }
        plan
    }
}
