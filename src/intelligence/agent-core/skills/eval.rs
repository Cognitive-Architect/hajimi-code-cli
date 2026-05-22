//! Skill output evaluation for deterministic V0a criteria.
//! <!-- AGENT-SKILLS-V0-2026-05-19: local skill pack integration initiated -->

use crate::skills::types::{SkillEvalCriterion, SkillEvalFixture};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEvalResult {
    pub passed: bool,
    pub failure_reason: Option<String>,
}

pub fn evaluate_output(output: &str, criterion: &SkillEvalCriterion) -> SkillEvalResult {
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
        let result = evaluate_output(&case.output, &case.criterion);
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
        let result = evaluate_output(&passing_case.output, &fixture.criteria);
        assert!(result.passed, "complete output should satisfy criteria");
    }
}
