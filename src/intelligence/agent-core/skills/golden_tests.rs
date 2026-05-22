//! Deterministic golden tests for SkillRouter.
//! Uses include_str! to compile fixtures statically, preventing silent test skipping.

#[cfg(test)]
mod tests {
    use crate::skills::registry::SkillRegistry;
    use crate::skills::router::SkillRouter;
    use crate::skills::types::SkillRouterConfig;
    use std::sync::Arc;

    const FIXTURE_AUTO_SAVE: &str =
        include_str!("../../../../tests/agent_skills_golden/router/auto_save_trigger.json");
    const FIXTURE_NO_SKILL: &str =
        include_str!("../../../../tests/agent_skills_golden/router/no_skill_needed.json");
    const FIXTURE_BELOW_THRESHOLD: &str =
        include_str!("../../../../tests/agent_skills_golden/router/below_threshold.json");
    const FIXTURE_MAX_ACTIVE_ZERO: &str =
        include_str!("../../../../tests/agent_skills_golden/router/max_active_zero.json");
    const FIXTURE_AUTO_SAVE_MISSING_BLOCK: &str =
        include_str!("../../../../tests/agent_skills_golden/failure/auto_save_missing_block.json");
    const FIXTURE_SKILL_EVAL_CRITERIA: &str =
        include_str!("../../../../tests/agent_skills_golden/reflector/skill_eval_criteria.json");

    #[derive(serde::Deserialize)]
    struct ConfigOverride {
        threshold: f32,
        max_active_skills: usize,
    }

    #[derive(serde::Deserialize)]
    struct ExpectedMatch {
        name: String,
        reason_contains: String,
    }

    #[derive(serde::Deserialize)]
    struct GoldenCase {
        description: String,
        input: String,
        config_override: Option<ConfigOverride>,
        expected_selected: Vec<ExpectedMatch>,
        expected_rejected: Vec<ExpectedMatch>,
    }

    #[derive(serde::Deserialize)]
    struct FailureGoldenCase {
        description: String,
        criterion: crate::skills::types::SkillEvalCriterion,
        output: String,
        expected_pass: bool,
        expected_failure_reason: String,
    }

    fn run_case(case_json: &str) {
        let case: GoldenCase =
            serde_json::from_str(case_json).expect("Failed to parse golden case JSON");

        // Set up the registry using standard test fixture path
        let base_dir = std::env::current_dir().unwrap();
        let mut fixture_dir = base_dir.join("tests/fixtures/skills");
        if !fixture_dir.exists() {
            fixture_dir = base_dir.join("../../../tests/fixtures/skills");
        }

        let registry = Arc::new(SkillRegistry::scan(&fixture_dir).unwrap());

        let mut config = SkillRouterConfig::default();
        if let Some(ref o) = case.config_override {
            config.threshold = o.threshold;
            config.max_active_skills = o.max_active_skills;
        }

        let router = SkillRouter::new(registry, config);
        let res = router.route(&case.input);

        // 1. Assert selected list matches expectations
        assert_eq!(
            res.selected.len(),
            case.expected_selected.len(),
            "Case '{}' failed selected count. Got: {:?}",
            case.description,
            res.selected
        );
        for expected in &case.expected_selected {
            let matched = res
                .selected
                .iter()
                .find(|m| m.name == expected.name)
                .expect(&format!(
                    "Case '{}' expected selected skill '{}' to be selected",
                    case.description, expected.name
                ));
            assert!(
                matched.reason.contains(&expected.reason_contains),
                "Case '{}' expected reason for '{}' to contain '{}'. Got: '{}'",
                case.description,
                expected.name,
                expected.reason_contains,
                matched.reason
            );
        }

        // 2. Assert rejected list matches expectations
        assert_eq!(
            res.rejected.len(),
            case.expected_rejected.len(),
            "Case '{}' failed rejected count. Got: {:?}",
            case.description,
            res.rejected
        );
        for expected in &case.expected_rejected {
            let matched = res
                .rejected
                .iter()
                .find(|m| m.name == expected.name)
                .expect(&format!(
                    "Case '{}' expected rejected skill '{}' to be in rejected list",
                    case.description, expected.name
                ));
            assert!(
                matched.reason.contains(&expected.reason_contains),
                "Case '{}' expected rejected reason for '{}' to contain '{}'. Got: '{}'",
                case.description,
                expected.name,
                expected.reason_contains,
                matched.reason
            );
        }

        // 3. Assert Route Receipt serializes cleanly
        let receipt_json =
            serde_json::to_string(&res.receipt).expect("Failed to serialize route receipt");
        assert!(!receipt_json.is_empty());
        assert!(!res.receipt.input_hash.is_empty());
        assert_eq!(res.receipt.router_version, "hajimi.skill.router.v0");
        assert!(!res.receipt.timestamp.is_empty());
    }

    #[test]
    fn test_agent_skills_golden() {
        run_case(FIXTURE_AUTO_SAVE);
        run_case(FIXTURE_NO_SKILL);
        run_case(FIXTURE_BELOW_THRESHOLD);
        run_case(FIXTURE_MAX_ACTIVE_ZERO);

        let criteria: crate::skills::types::SkillEvalCriterion =
            serde_json::from_str(FIXTURE_SKILL_EVAL_CRITERIA)
                .expect("Failed to parse skill eval criteria golden");
        assert!(criteria.must_include.iter().any(|v| v == "=== AUTO SAVE"));
        assert!(criteria.must_not_include.iter().any(|v| v == "TODO"));

        let failure: FailureGoldenCase = serde_json::from_str(FIXTURE_AUTO_SAVE_MISSING_BLOCK)
            .expect("Failed to parse auto-save failure golden");
        let result = crate::skills::eval::evaluate_output(&failure.output, &failure.criterion);
        assert_eq!(
            result.passed, failure.expected_pass,
            "{}",
            failure.description
        );
        assert!(result
            .failure_reason
            .as_deref()
            .unwrap_or_default()
            .contains(&failure.expected_failure_reason));
    }
}
