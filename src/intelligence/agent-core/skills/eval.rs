//! Skill output evaluation for deterministic V0a criteria.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::loader::SkillLoader;
use crate::skills::registry::SkillRegistry;
use crate::skills::types::{
    SkillEvalCriterion, SkillEvalFixture, SkillEvalReport, SkillEvalReportEntry, SkillMatch,
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
}
