//! Skill output evaluation for deterministic V0a criteria.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::loader::SkillLoader;
use crate::skills::registry::SkillRegistry;
use crate::skills::types::{
    LoadedSkill, SkillEvalCriterion, SkillEvalFixture, SkillEvalReport, SkillEvalReportEntry,
    SkillExecutionReceipt, SkillMatch, SkillRouteReceipt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEvalResult {
    pub passed: bool,
    pub failure_reason: Option<String>,
}

/// Evaluates a single criterion against an output string.
pub fn evaluate_output_single(output: &str, criterion: &SkillEvalCriterion) -> SkillEvalResult {
    for required in &criterion.must_include {
        if !output.contains(required) {
            return SkillEvalResult {
                passed: false,
                failure_reason: Some(format!("missing required marker: {}", required)),
            };
        }
    }

    for forbidden in &criterion.must_not_include {
        if output.contains(forbidden) {
            return SkillEvalResult {
                passed: false,
                failure_reason: Some(format!("forbidden marker present: {}", forbidden)),
            };
        }
    }

    if let Some(expected_structure) = &criterion.expected_structure {
        for required in expected_structure {
            if !output.contains(required) {
                return SkillEvalResult {
                    passed: false,
                    failure_reason: Some(format!("missing expected structure: {}", required)),
                };
            }
        }
    }

    SkillEvalResult {
        passed: true,
        failure_reason: None,
    }
}

/// Evaluates all active skills' criteria against the given output.
pub fn evaluate_output(
    active_skills: &[SkillMatch],
    output: &str,
    registry: &SkillRegistry,
) -> SkillEvalReport {
    let loader = SkillLoader::new(registry.clone());
    let mut skill_reports = Vec::new();
    let mut overall_passed = true;
    let mut overall_failure_reason = None;

    for skill in active_skills {
        match loader.load_eval_criteria(&skill.name) {
            Ok(Some(criterion)) => {
                let mut matched_must_include = Vec::new();
                let mut missing_must_include = Vec::new();
                let mut matched_must_not_include = Vec::new();
                let mut passed = true;
                let mut failure_reason = None;

                for required in &criterion.must_include {
                    if output.contains(required) {
                        matched_must_include.push(required.clone());
                    } else {
                        missing_must_include.push(required.clone());
                        passed = false;
                        if failure_reason.is_none() {
                            failure_reason = Some(format!("missing required marker: {}", required));
                        }
                    }
                }

                for forbidden in &criterion.must_not_include {
                    if output.contains(forbidden) {
                        matched_must_not_include.push(forbidden.clone());
                        passed = false;
                        if failure_reason.is_none() {
                            failure_reason =
                                Some(format!("forbidden marker present: {}", forbidden));
                        }
                    }
                }

                if passed {
                    if let Some(expected_structure) = &criterion.expected_structure {
                        for required in expected_structure {
                            if !output.contains(required) {
                                passed = false;
                                if failure_reason.is_none() {
                                    failure_reason =
                                        Some(format!("missing expected structure: {}", required));
                                }
                                break;
                            }
                        }
                    }
                }

                if !passed {
                    overall_passed = false;
                    if overall_failure_reason.is_none() {
                        overall_failure_reason = failure_reason.clone();
                    }
                }

                skill_reports.push(SkillEvalReportEntry {
                    skill_name: skill.name.clone(),
                    passed,
                    failure_reason,
                    matched_must_include,
                    missing_must_include,
                    matched_must_not_include,
                });
            }
            Ok(None) => {
                skill_reports.push(SkillEvalReportEntry {
                    skill_name: skill.name.clone(),
                    passed: true,
                    failure_reason: None,
                    matched_must_include: Vec::new(),
                    missing_must_include: Vec::new(),
                    matched_must_not_include: Vec::new(),
                });
            }
            Err(e) => {
                overall_passed = false;
                let err_msg = format!("failed to load criteria: {}", e);
                if overall_failure_reason.is_none() {
                    overall_failure_reason = Some(err_msg.clone());
                }
                skill_reports.push(SkillEvalReportEntry {
                    skill_name: skill.name.clone(),
                    passed: false,
                    failure_reason: Some(err_msg),
                    matched_must_include: Vec::new(),
                    missing_must_include: Vec::new(),
                    matched_must_not_include: Vec::new(),
                });
            }
        }
    }

    SkillEvalReport {
        passed: overall_passed,
        skill_reports,
        failure_reason: overall_failure_reason,
    }
}

/// Generates execution receipts from evaluation reports, active skills and routing receipt.
pub fn generate_execution_receipts(
    report: &SkillEvalReport,
    route_receipt: &SkillRouteReceipt,
    active_skills: &[LoadedSkill],
) -> Vec<SkillExecutionReceipt> {
    let mut receipts = Vec::new();
    for entry in &report.skill_reports {
        let matched_skill = route_receipt
            .selected
            .iter()
            .find(|m| m.name == entry.skill_name);
        let loaded = active_skills
            .iter()
            .find(|s| s.manifest.name == entry.skill_name);

        let skill_version = loaded
            .map(|s| s.manifest.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string());
        let matched_score = matched_skill.map(|m| m.score).unwrap_or(0.0);

        let next_revision_hint = if !entry.passed {
            let mut hints = Vec::new();
            if !entry.missing_must_include.is_empty() {
                hints.push(format!(
                    "Add missing required markers: {:?}",
                    entry.missing_must_include
                ));
            }
            if !entry.matched_must_not_include.is_empty() {
                hints.push(format!(
                    "Remove forbidden markers: {:?}",
                    entry.matched_must_not_include
                ));
            }
            Some(hints.join("; "))
        } else {
            None
        };

        receipts.push(SkillExecutionReceipt {
            skill_name: entry.skill_name.clone(),
            skill_version,
            input_hash: route_receipt.input_hash.clone(),
            matched_score,
            success: entry.passed,
            failure_reason: entry.failure_reason.clone(),
            next_revision_hint,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }
    receipts
}

pub fn parse_eval_fixture(raw: &str) -> Result<SkillEvalFixture, serde_json::Error> {
    serde_json::from_str(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::SkillEvalCase;

    const REFLECTOR_CRITERIA: &str =
        include_str!("../../../../tests/agent_skills_golden/reflector/skill_eval_criteria.json");
    const AUTO_SAVE_MISSING_BLOCK: &str =
        include_str!("../../../../tests/agent_skills_golden/failure/auto_save_missing_block.json");

    #[derive(serde::Deserialize)]
    struct FailureGoldenCase {
        description: String,
        criterion: SkillEvalCriterion,
        output: String,
        expected_pass: bool,
        expected_failure_reason: String,
    }

    #[test]
    fn reflector_skill_eval_loads_criteria_from_golden() {
        let criterion: SkillEvalCriterion =
            serde_json::from_str(REFLECTOR_CRITERIA).expect("criteria golden should parse");
        assert_eq!(criterion.skill_name, "auto-save");
        assert!(criterion.must_include.iter().any(|v| v == "=== AUTO SAVE"));
        assert!(criterion.must_include.iter().any(|v| v == "做了什么"));
        assert!(criterion.must_not_include.iter().any(|v| v == "TODO"));
    }

    #[test]
    fn reflector_skill_eval_detects_auto_save_missing_block() {
        let case: FailureGoldenCase =
            serde_json::from_str(AUTO_SAVE_MISSING_BLOCK).expect("failure golden should parse");
        let result = evaluate_output_single(&case.output, &case.criterion);
        assert_eq!(
            result.passed, case.expected_pass,
            "case '{}' pass expectation changed",
            case.description
        );
        assert!(
            result
                .failure_reason
                .as_deref()
                .unwrap_or_default()
                .contains(&case.expected_failure_reason),
            "failure reason should mention '{}', got {:?}",
            case.expected_failure_reason,
            result.failure_reason
        );
    }

    #[test]
    fn reflector_skill_eval_accepts_complete_auto_save_case_from_fixture() {
        let fixture = parse_eval_fixture(include_str!(
            "../../../../tests/fixtures/skills/auto-save/evals/output_cases.json"
        ))
        .expect("output_cases fixture should parse");

        let passing_case: &SkillEvalCase = fixture
            .cases
            .iter()
            .find(|case| case.expected_pass)
            .expect("fixture should include a passing output case");
        let result = evaluate_output_single(&passing_case.output, &fixture.criteria);
        assert!(result.passed, "complete output should satisfy criteria");
    }

    #[test]
    fn test_skill_receipt() {
        use crate::skills::types::{
            SkillManifest, SkillMatch, SkillPermissions, SkillRiskLevel, SkillRouteReceipt,
        };

        let report = SkillEvalReport {
            passed: false,
            skill_reports: vec![SkillEvalReportEntry {
                skill_name: "auto-save".to_string(),
                passed: false,
                failure_reason: Some("missing required marker: === AUTO SAVE".to_string()),
                matched_must_include: vec![],
                missing_must_include: vec!["=== AUTO SAVE".to_string()],
                matched_must_not_include: vec![],
            }],
            failure_reason: Some("missing required marker: === AUTO SAVE".to_string()),
        };

        let route_receipt = SkillRouteReceipt {
            input_hash: "abc123hash".to_string(),
            router_version: "hajimi.skill.router.v0".to_string(),
            selected: vec![SkillMatch {
                name: "auto-save".to_string(),
                score: 0.85,
                reason: "matched triggers".to_string(),
                matched_terms: vec![],
                risk_level: SkillRiskLevel::Low,
                category: None,
                exclusive_group: None,
            }],
            rejected: vec![],
            timestamp: "2026-05-22T00:00:00Z".to_string(),
        };

        let active_skills = vec![LoadedSkill {
            manifest: SkillManifest {
                schema_version: "hajimi.skill.v0".to_string(),
                version: "1.2.3".to_string(),
                name: "auto-save".to_string(),
                title: "Auto Save".to_string(),
                description: "desc".to_string(),
                enabled: true,
                category: None,
                exclusive_group: None,
                triggers: vec![],
                risk_level: SkillRiskLevel::Low,
                entry: "SKILL.md".to_string(),
                eval_entry: None,
                context_budget_tokens: None,
                allowed_tools: vec![],
                permissions: SkillPermissions::default(),
            },
            instructions: "instructions".to_string(),
            token_estimate: 100,
        }];

        let receipts = generate_execution_receipts(&report, &route_receipt, &active_skills);
        assert_eq!(receipts.len(), 1);
        let receipt = &receipts[0];
        assert_eq!(receipt.skill_name, "auto-save");
        assert_eq!(receipt.skill_version, "1.2.3");
        assert_eq!(receipt.input_hash, "abc123hash");
        assert_eq!(receipt.matched_score, 0.85);
        assert!(!receipt.success);
        assert_eq!(
            receipt.failure_reason,
            Some("missing required marker: === AUTO SAVE".to_string())
        );
        assert!(receipt
            .next_revision_hint
            .as_ref()
            .unwrap()
            .contains("Add missing required markers"));
    }
}
