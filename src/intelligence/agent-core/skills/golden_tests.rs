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
    const FIXTURE_RUNTIME_WRITE_REQUIRES_APPROVAL: &str =
        include_str!("../../../../tests/agent_skills_golden/runtime/write_requires_approval.json");
    const FIXTURE_RUNTIME_SHELL_DENIED: &str =
        include_str!("../../../../tests/agent_skills_golden/runtime/shell_denied_by_default.json");
    const FIXTURE_AUTO_SAVE_OUTPUT_REQUIRED: &str =
        include_str!("../../../../tests/agent_skills_golden/output/auto_save_output_required.json");
    const FIXTURE_AUTO_SAVE_MISSING_ARCHIVE: &str = include_str!(
        "../../../../tests/agent_skills_golden/failure/auto_save_missing_archive.json"
    );

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

    #[derive(serde::Deserialize)]
    struct ExpectedRuntimeConstraint {
        tool_name: String,
        permission: crate::skills::types::SkillToolPermissionLevel,
        approval_level: crate::governance::ApprovalLevel,
        reason_contains: String,
    }

    #[derive(serde::Deserialize)]
    struct RuntimeGoldenCase {
        description: String,
        available_tools: Vec<String>,
        manifest: crate::skills::types::SkillManifest,
        expected_allowed: Vec<ExpectedRuntimeConstraint>,
        expected_denied: Vec<ExpectedRuntimeConstraint>,
    }

    #[derive(serde::Deserialize)]
    struct OutputGoldenCase {
        description: String,
        active_skills: Vec<crate::skills::types::SkillMatch>,
        output: String,
        expected_pass: bool,
        expected_failure_reason: Option<String>,
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

    fn run_runtime_case(case_json: &str) {
        let case: RuntimeGoldenCase =
            serde_json::from_str(case_json).expect("Failed to parse runtime golden case JSON");
        let runtime = crate::skills::SkillRuntime::new(case.available_tools);
        let constraints = runtime.build_tool_constraints(&case.manifest);

        assert_eq!(
            constraints.allowed.len(),
            case.expected_allowed.len(),
            "Case '{}' failed allowed count. Got: {:?}",
            case.description,
            constraints.allowed
        );
        assert_eq!(
            constraints.denied.len(),
            case.expected_denied.len(),
            "Case '{}' failed denied count. Got: {:?}",
            case.description,
            constraints.denied
        );

        for expected in &case.expected_allowed {
            let matched = constraints
                .allowed
                .iter()
                .find(|constraint| constraint.tool_name == expected.tool_name)
                .expect("expected allowed runtime constraint");
            assert_eq!(matched.permission, expected.permission);
            assert_eq!(matched.approval_level, expected.approval_level);
            assert!(matched.reason.contains(&expected.reason_contains));
        }

        for expected in &case.expected_denied {
            let matched = constraints
                .denied
                .iter()
                .find(|constraint| constraint.tool_name == expected.tool_name)
                .expect("expected denied runtime constraint");
            assert_eq!(matched.permission, expected.permission);
            assert_eq!(matched.approval_level, expected.approval_level);
            assert!(matched.reason.contains(&expected.reason_contains));
        }
    }

    fn run_output_case(case_json: &str, registry: &SkillRegistry) {
        let case: OutputGoldenCase =
            serde_json::from_str(case_json).expect("Failed to parse output golden case");
        let report =
            crate::skills::eval::evaluate_output(&case.active_skills, &case.output, registry);

        assert_eq!(
            report.passed, case.expected_pass,
            "Case '{}' failed pass expectation. Reason: {:?}",
            case.description, report.failure_reason
        );

        if case.expected_pass {
            assert!(report.failure_reason.is_none());
            for entry in report.skill_reports {
                assert!(entry.passed);
                assert!(entry.failure_reason.is_none());
            }
        } else {
            let expected_reason = case
                .expected_failure_reason
                .as_ref()
                .expect("expected failure reason must be provided");
            let actual_reason = report.failure_reason.as_deref().unwrap_or_default();
            assert!(
                actual_reason.contains(expected_reason),
                "Case '{}' failed. Expected actual reason {:?} to contain {:?}",
                case.description,
                actual_reason,
                expected_reason
            );
        }
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
        let result =
            crate::skills::eval::evaluate_output_single(&failure.output, &failure.criterion);
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

        run_runtime_case(FIXTURE_RUNTIME_WRITE_REQUIRES_APPROVAL);
        run_runtime_case(FIXTURE_RUNTIME_SHELL_DENIED);

        // Get standard test registry
        let base_dir = std::env::current_dir().unwrap();
        let mut fixture_dir = base_dir.join("tests/fixtures/skills");
        if !fixture_dir.exists() {
            fixture_dir = base_dir.join("../../../tests/fixtures/skills");
        }
        let registry = SkillRegistry::scan(&fixture_dir).unwrap();

        // Run new E2E output/failure golden cases
        run_output_case(FIXTURE_AUTO_SAVE_OUTPUT_REQUIRED, &registry);
        run_output_case(FIXTURE_AUTO_SAVE_MISSING_ARCHIVE, &registry);
    }
}
