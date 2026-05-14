//! Planner DTO module for AGENT-PROMPT-CORE-001 Phase 2.
//!
//! Defines the machine-readable schema that the LLM Planner returns
//! when decomposing a goal. These DTOs are deserialized from JSON and
//! then mapped to existing runtime types (`SubGoal`, `Task`) via
//! adapter code in `planner.rs` (Day 5).
//!
//! # Mapping Notes
//! - `id_hint` → runtime `SubGoalId`: `format!("{goal_id}-{id_hint}")`
//! - `depends_on[]` references `id_hint`, not runtime IDs.
//! - Fields that do not fit the existing `SubGoal` struct are stored
//!   in `SubGoal.metadata` as a temporary compatibility measure.
//!
//! # Schema Stability
//! `schema_version` must equal `"PlannerSubgoalPlanV1"`. Future
//! revisions will bump this discriminator.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Re-use the existing Priority enum from the planner module so that
// downstream mapping does not require translation between two
// semantically identical types.
use crate::planner::Priority;

// Re-use RiskLevel from tool_manifest to keep the risk taxonomy
// consistent across Planner and ToolManifest specifications.
use crate::tool_manifest::RiskLevel;

// -----------------------------------------------------------------------------
// PlannerSubgoalPlanV1 DTOs
// -----------------------------------------------------------------------------

/// Top-level plan object returned by the LLM Planner.
///
/// Wraps a decomposed goal into an ordered list of subgoals with
/// dependency links, suggested tools, validation intent, and risk
/// metadata. This is the root DTO deserialized from the Planner LLM
/// response.
///
/// # Validation Rules
/// - `schema_version` must equal `"PlannerSubgoalPlanV1"`.
/// - `goal_id` must match the input goal ID.
/// - `subgoals` must be non-empty unless the goal is blocked.
#[derive(Debug, Clone, Deserialize)]
pub struct PlannerSubgoalPlanV1Dto {
    /// Schema version discriminator.
    /// Must equal `"PlannerSubgoalPlanV1"`.
    pub schema_version: String,
    /// Identifier of the goal being planned.
    /// Must match the input goal passed to the Planner.
    pub goal_id: String,
    /// One-sentence summary of the overall plan.
    pub summary: String,
    /// Ordered list of subgoals to execute.
    /// Dependencies are expressed via `id_hint` references.
    pub subgoals: Vec<PlannerSubgoalDto>,
    /// Risks that affect the plan as a whole.
    /// These are elevated above individual subgoal risks.
    pub global_risks: Vec<String>,
    /// Free-form notes from the planner.
    /// May include assumptions, caveats, or scope clarifications.
    pub notes: Vec<String>,
}

impl PlannerSubgoalPlanV1Dto {
    /// Validate that every `depends_on` reference points to an existing
    /// `id_hint` within the plan.
    ///
    /// TODO: Phase 2 Day 5 — expand to also verify `schema_version`
    /// and check that every `suggested_tools` entry exists in the
    /// current `ToolManifest`.
    pub fn validate_dependencies(&self) -> Result<(), String> {
        let ids: HashSet<&str> = self.subgoals.iter().map(|s| s.id_hint.as_str()).collect();
        for sg in &self.subgoals {
            for dep in &sg.depends_on {
                if !ids.contains(dep.as_str()) {
                    return Err(format!("unknown dependency: {}", dep));
                }
            }
        }
        Ok(())
    }
}

/// A single subgoal inside a `PlannerSubgoalPlanV1Dto`.
///
/// Represents one unit of work within the larger plan. Each subgoal
/// carries enough metadata for the Act step to select tools, the
/// Reflect step to verify evidence, and the Decide step to know when
/// to stop or ask the user.
///
/// # Runtime Mapping
/// Designed to map into the existing `SubGoal` struct via an adapter
/// layer. Fields that do not yet fit `SubGoal` (e.g.
/// `suggested_tools`, `expected_evidence`) are stored in its
/// `metadata: HashMap<String, String>` field as a temporary
/// compatibility measure until the Owner approves a struct extension.
#[derive(Debug, Clone, Deserialize)]
pub struct PlannerSubgoalDto {
    /// Stable local identifier unique within this plan.
    ///
    /// Mapped at runtime to a globally unique `SubGoalId` via:
    /// `format!("{goal_id}-{id_hint}")`.
    pub id_hint: String,
    /// Human-readable description of what this subgoal achieves.
    pub description: String,
    /// Priority relative to sibling subgoals.
    ///
    /// Uses the same `Priority` enum as the runtime `SubGoal` struct
    /// to avoid translation errors during mapping.
    pub priority: Priority,
    /// `id_hint` values of subgoals that must complete before this one.
    ///
    /// These references are resolved to runtime `SubGoalId` values
    /// after all subgoals have been created.
    pub depends_on: Vec<String>,
    /// Names of tools recommended for executing this subgoal.
    ///
    /// Every name in this list must exist in the current
    /// `ToolManifest`. Unknown tools are removed with a warning.
    pub suggested_tools: Vec<String>,
    /// Observable proof required to confirm this subgoal is complete.
    ///
    /// Examples: file path, command output, trace ID, diff summary.
    pub expected_evidence: Vec<String>,
    /// What kind of validation is expected after execution.
    ///
    /// Informs the Act and Reflect steps about what evidence to
    /// collect and how to judge success.
    pub validation_intent: ValidationIntent,
    /// Estimated risk of executing this subgoal.
    ///
    /// Higher when edits, shell commands, Git operations, or
    /// destructive changes are involved.
    pub risk_level: RiskLevel,
    /// Whether user approval is required before execution.
    ///
    /// Always `true` for high-risk or scope-changing operations.
    pub requires_user_approval: bool,
    /// Conditions that should trigger a handoff or user ask.
    ///
    /// Concrete sentinel conditions such as "required file is
    /// unavailable" or "test environment cannot be reached".
    pub stop_conditions: Vec<String>,
}

/// The kind of validation expected after a subgoal is executed.
///
/// This enum informs the Act and Reflect steps about what evidence
/// to collect. It does not dictate the exact validation command;
/// that is determined at runtime based on project type and context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationIntent {
    /// No explicit validation required.
    /// Used for read-only or information-gathering subgoals.
    None,
    /// Static analysis (lint, typecheck, format check).
    StaticCheck,
    /// Unit test execution.
    UnitTest,
    /// Integration or end-to-end test execution.
    IntegrationTest,
    /// Human review of the changes.
    ManualReview,
    /// Command output (build, diff, git status, etc.).
    CommandOutput,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B-06/14: Full PlannerSubgoalPlanV1Dto JSON deserialization.
    #[test]
    fn test_planner_subgoal_plan_v1_dto_deserialize() {
        let json = r#"{
            "schema_version": "PlannerSubgoalPlanV1",
            "goal_id": "goal-1",
            "summary": "Test plan",
            "subgoals": [
                {
                    "id_hint": "sg1",
                    "description": "First subgoal",
                    "priority": "High",
                    "depends_on": [],
                    "suggested_tools": ["read_file"],
                    "expected_evidence": ["file exists"],
                    "validation_intent": "StaticCheck",
                    "risk_level": "Medium",
                    "requires_user_approval": false,
                    "stop_conditions": ["file missing"]
                }
            ],
            "global_risks": ["scope creep"],
            "notes": ["assumption: git repo exists"]
        }"#;
        let dto: PlannerSubgoalPlanV1Dto =
            serde_json::from_str(json).expect("valid JSON should deserialize");
        assert_eq!(dto.schema_version, "PlannerSubgoalPlanV1");
        assert_eq!(dto.goal_id, "goal-1");
        assert_eq!(dto.subgoals.len(), 1);
        assert_eq!(dto.subgoals[0].id_hint, "sg1");
        assert_eq!(dto.subgoals[0].description, "First subgoal");
        assert_eq!(dto.subgoals[0].priority, Priority::High);
        assert_eq!(
            dto.subgoals[0].validation_intent,
            ValidationIntent::StaticCheck
        );
        assert_eq!(dto.subgoals[0].risk_level, RiskLevel::Medium);
    }

    /// B-06/14: PlannerSubgoalDto JSON deserialization with dependencies.
    #[test]
    fn test_planner_subgoal_dto_deserialize() {
        let json = r#"{
            "id_hint": "sg2",
            "description": "Second subgoal",
            "priority": "Low",
            "depends_on": ["sg1"],
            "suggested_tools": [],
            "expected_evidence": [],
            "validation_intent": "None",
            "risk_level": "Low",
            "requires_user_approval": false,
            "stop_conditions": []
        }"#;
        let dto: PlannerSubgoalDto =
            serde_json::from_str(json).expect("valid JSON should deserialize");
        assert_eq!(dto.id_hint, "sg2");
        assert_eq!(dto.description, "Second subgoal");
        assert_eq!(dto.priority, Priority::Low);
        assert_eq!(dto.depends_on, vec!["sg1"]);
    }

    /// B-06-FIX-001: validate_dependencies ok and fail cases combined.
    #[test]
    fn test_validate_dependencies() {
        let make = |deps: Vec<(&str, Vec<&str>)>| PlannerSubgoalPlanV1Dto {
            schema_version: "PlannerSubgoalPlanV1".to_string(),
            goal_id: "g1".to_string(),
            summary: "test".to_string(),
            subgoals: deps
                .into_iter()
                .map(|(id, ds)| PlannerSubgoalDto {
                    id_hint: id.to_string(),
                    description: id.to_string(),
                    priority: Priority::High,
                    depends_on: ds.into_iter().map(|d| d.to_string()).collect(),
                    suggested_tools: vec![],
                    expected_evidence: vec![],
                    validation_intent: ValidationIntent::None,
                    risk_level: RiskLevel::Low,
                    requires_user_approval: false,
                    stop_conditions: vec![],
                })
                .collect(),
            global_risks: vec![],
            notes: vec![],
        };
        assert!(make(vec![("sg1", vec![]), ("sg2", vec!["sg1"])])
            .validate_dependencies()
            .is_ok());
        let result = make(vec![("sg1", vec!["bad"])]).validate_dependencies();
        assert!(result.is_err() && result.unwrap_err().contains("bad"));
    }

    /// B-06/14: ValidationIntent serialize/deserialize roundtrip for all variants.
    #[test]
    fn test_validation_intent_roundtrip() {
        for intent in [
            ValidationIntent::None,
            ValidationIntent::StaticCheck,
            ValidationIntent::UnitTest,
            ValidationIntent::IntegrationTest,
            ValidationIntent::ManualReview,
            ValidationIntent::CommandOutput,
        ] {
            let json = serde_json::to_string(&intent).expect("should serialize");
            let deserialized: ValidationIntent =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(intent, deserialized);
        }
    }
}
