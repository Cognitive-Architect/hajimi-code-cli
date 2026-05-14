//! Reflector DTO module for AGENT-PROMPT-CORE-001 Phase 3.
//!
//! Defines the machine-readable schema that the LLM Reflector returns
//! when critiquing execution results. These DTOs are deserialized from
//! JSON and then mapped to the existing runtime `Critique` type via
//! `ReflectorCritiqueV1Dto::to_critique()`.

use crate::reflector::CritiqueSeverity;
use serde::{Deserialize, Serialize};

/// Classification of why a subgoal or task failed or underperformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootCauseCategory {
    None,
    ToolFailure,
    BadPlan,
    MissingContext,
    Permission,
    ValidationFailure,
    ParseFailure,
    UserInputNeeded,
    Unknown,
}

/// Action the agent should take after reflection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedAction {
    Continue,
    RetryWithNewArgs,
    UseAlternativeTool,
    RevisePlan,
    AskUser,
    StopAndHandoff,
}

/// Detailed root cause diagnosed by the Reflector.
#[derive(Debug, Clone, Deserialize)]
pub struct RootCauseDto {
    pub category: RootCauseCategory,
    pub description: String,
    pub confidence: f32,
}

/// Proposed plan adjustment when the original plan is flawed.
#[derive(Debug, Clone, Deserialize)]
pub struct PlanAdjustmentDto {
    pub action: RecommendedAction,
    pub reason: String,
    pub revised_subgoals: Vec<String>,
    /// Suggested alternative tools for the UseAlternativeTool action (DEBT-B08-002).
    pub suggested_tools: Option<Vec<String>>,
}

/// Stop-loss trigger to prevent runaway execution.
#[derive(Debug, Clone, Deserialize)]
pub struct StopLossDto {
    pub triggered: bool,
    pub reason: String,
    pub escalation_target: String,
}

/// Full critique DTO returned by the LLM Reflector.
///
/// Schema version must equal `"ReflectorCritiqueV1"`. Contains root-cause
/// analysis, risk flags, plan-adjustment proposals, and stop-loss state.
#[derive(Debug, Clone, Deserialize)]
pub struct ReflectorCritiqueV1Dto {
    /// Schema version discriminator. Must equal `"ReflectorCritiqueV1"`.
    pub schema_version: String,
    /// Whether the execution was judged successful.
    pub success: bool,
    /// Severity of the most critical issue found.
    pub severity: CritiqueSeverity,
    /// Confidence score in the critique conclusion (0.0–1.0).
    pub confidence: f32,
    /// Observable evidence collected during execution.
    pub evidence: Vec<String>,
    /// Diagnosed root cause of any failure.
    pub root_cause: RootCauseDto,
    /// Specific issues identified during reflection.
    pub issues: Vec<String>,
    /// New risks discovered that were not in the original plan.
    pub new_risks: Vec<String>,
    /// Actionable suggestions for improvement.
    pub suggestions: Vec<String>,
    /// Optional plan adjustment proposal.
    pub plan_adjustment: Option<PlanAdjustmentDto>,
    /// Optional stop-loss trigger for dangerous situations.
    pub stop_loss: Option<StopLossDto>,
}

impl ReflectorCritiqueV1Dto {
    /// Map this DTO to the runtime `Critique` struct.
    ///
    /// Enriches the legacy `Critique` with information from new fields:
    /// - `new_risks` are appended to `issues`.
    /// - `stop_loss.triggered` adds an emergency issue.
    /// - `plan_adjustment` adds a structured suggestion.
    pub fn to_critique(&self) -> crate::reflector::Critique {
        let mut issues = self.issues.clone();
        let mut suggestions = self.suggestions.clone();
        if let Some(ref stop) = self.stop_loss {
            if stop.triggered {
                issues.push(format!("STOP-LOSS: {}", stop.reason));
            }
        }
        if let Some(ref adj) = self.plan_adjustment {
            suggestions.push(format!("Adjustment: {:?} — {}", adj.action, adj.reason));
        }
        if !self.new_risks.is_empty() {
            issues.extend(self.new_risks.iter().map(|r| format!("Risk: {}", r)));
        }
        crate::reflector::Critique {
            success: self.success,
            issues,
            suggestions,
            severity: self.severity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full ReflectorCritiqueV1Dto JSON deserialization.
    #[test]
    fn test_reflector_critique_v1_dto_deserialize() {
        let json = r#"{
            "schema_version": "ReflectorCritiqueV1",
            "success": false,
            "severity": "High",
            "confidence": 0.75,
            "evidence": ["output missing"],
            "root_cause": {"category":"ToolFailure","description":"command not found","confidence":0.9},
            "issues": ["shell failed"],
            "new_risks": ["data loss"],
            "suggestions": ["check PATH"],
            "plan_adjustment": {"action":"RetryWithNewArgs","reason":"bad args","revised_subgoals":[]},
            "stop_loss": {"triggered":false,"reason":"none","escalation_target":"user"}
        }"#;
        let dto: ReflectorCritiqueV1Dto =
            serde_json::from_str(json).expect("valid JSON should deserialize");
        assert_eq!(dto.schema_version, "ReflectorCritiqueV1");
        assert!(!dto.success);
        assert_eq!(dto.severity, CritiqueSeverity::High);
        assert_eq!(dto.root_cause.category, RootCauseCategory::ToolFailure);
        assert_eq!(
            dto.plan_adjustment.as_ref().unwrap().action,
            RecommendedAction::RetryWithNewArgs
        );
    }

    /// RootCauseCategory serialize/deserialize roundtrip for all 9 variants.
    #[test]
    fn test_root_cause_category_roundtrip() {
        for cat in [
            RootCauseCategory::None,
            RootCauseCategory::ToolFailure,
            RootCauseCategory::BadPlan,
            RootCauseCategory::MissingContext,
            RootCauseCategory::Permission,
            RootCauseCategory::ValidationFailure,
            RootCauseCategory::ParseFailure,
            RootCauseCategory::UserInputNeeded,
            RootCauseCategory::Unknown,
        ] {
            let json = serde_json::to_string(&cat).expect("should serialize");
            let deserialized: RootCauseCategory =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(cat, deserialized);
        }
    }

    /// RecommendedAction serialize/deserialize roundtrip for all 6 variants.
    #[test]
    fn test_recommended_action_roundtrip() {
        for action in [
            RecommendedAction::Continue,
            RecommendedAction::RetryWithNewArgs,
            RecommendedAction::UseAlternativeTool,
            RecommendedAction::RevisePlan,
            RecommendedAction::AskUser,
            RecommendedAction::StopAndHandoff,
        ] {
            let json = serde_json::to_string(&action).expect("should serialize");
            let deserialized: RecommendedAction =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(action, deserialized);
        }
    }

    /// DTO to Critique mapping correctness.
    #[test]
    fn test_dto_to_critique_mapping() {
        let dto = ReflectorCritiqueV1Dto {
            schema_version: "ReflectorCritiqueV1".to_string(),
            success: false,
            severity: CritiqueSeverity::Critical,
            confidence: 0.5,
            evidence: vec!["e1".to_string()],
            root_cause: RootCauseDto {
                category: RootCauseCategory::BadPlan,
                description: "bad".to_string(),
                confidence: 0.8,
            },
            issues: vec!["i1".to_string()],
            new_risks: vec!["r1".to_string()],
            suggestions: vec!["s1".to_string()],
            plan_adjustment: Some(PlanAdjustmentDto {
                action: RecommendedAction::RevisePlan,
                reason: "replan".to_string(),
                revised_subgoals: vec![],
                suggested_tools: None,
            }),
            stop_loss: Some(StopLossDto {
                triggered: true,
                reason: "infinite loop".to_string(),
                escalation_target: "user".to_string(),
            }),
        };
        let critique = dto.to_critique();
        assert!(!critique.success);
        assert_eq!(critique.severity, CritiqueSeverity::Critical);
        assert!(critique.issues.iter().any(|i| i.contains("STOP-LOSS")));
        assert!(critique.issues.iter().any(|i| i.contains("Risk: r1")));
        assert!(critique
            .suggestions
            .iter()
            .any(|s| s.contains("Adjustment: RevisePlan")));
    }

    /// B-09/14: Unknown root cause category reports UNKNOWN during deserialization.
    #[test]
    fn test_unknown_root_cause_reports_unknown() {
        let json = r#"{"category":"Unknown","description":"unrecognized","confidence":0.3}"#;
        let dto: RootCauseDto = serde_json::from_str(json).expect("Unknown should deserialize");
        assert_eq!(dto.category, RootCauseCategory::Unknown);
        assert_eq!(dto.description, "unrecognized");
    }

    /// B-09/14: Stop-Loss triggered DTO produces Critique with STOP-LOSS issue and escalation.
    #[test]
    fn test_stop_loss_triggered_dto_to_critique() {
        let dto = ReflectorCritiqueV1Dto {
            schema_version: "ReflectorCritiqueV1".to_string(),
            success: false,
            severity: CritiqueSeverity::Critical,
            confidence: 0.95,
            evidence: vec!["repeated failure".to_string()],
            root_cause: RootCauseDto {
                category: RootCauseCategory::ToolFailure,
                description: "cmd not found".to_string(),
                confidence: 0.9,
            },
            issues: vec!["tool missing".to_string()],
            new_risks: vec![],
            suggestions: vec![],
            plan_adjustment: None,
            stop_loss: Some(StopLossDto {
                triggered: true,
                reason: "3 consecutive failures".to_string(),
                escalation_target: "user".to_string(),
            }),
        };
        let critique = dto.to_critique();
        assert!(!critique.success);
        assert_eq!(critique.severity, CritiqueSeverity::Critical);
        assert!(
            critique.issues.iter().any(|i| i.contains("STOP-LOSS")),
            "Stop-loss issue should be present"
        );
        assert!(
            critique
                .issues
                .iter()
                .any(|i| i.contains("3 consecutive failures")),
            "Stop-loss reason should be in issue"
        );
    }
}
