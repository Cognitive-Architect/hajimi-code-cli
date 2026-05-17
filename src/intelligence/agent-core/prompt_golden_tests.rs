use crate::act_dto::{ActDecision, ActionType, ToolCallV1};
use crate::planner_dto::PlannerSubgoalPlanV1Dto;
use crate::reflector_dto::{RecommendedAction, ReflectorCritiqueV1Dto, RootCauseCategory};
use crate::tool_manifest::RiskLevel;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct PlannerGoldenCase {
    id: String,
    contract_mapping: Vec<String>,
    input: Value,
    expected: PlannerSubgoalPlanV1Dto,
    expected_failure_reason: String,
}

#[derive(Debug, Deserialize)]
struct ReflectorGoldenCase {
    id: String,
    contract_mapping: Vec<String>,
    input: Value,
    expected: ReflectorCritiqueV1Dto,
    expected_failure_reason: String,
}

#[derive(Debug, Deserialize)]
struct ToolCallGoldenCase {
    id: String,
    contract_mapping: Vec<String>,
    input: Value,
    expected_tool_call: Option<ToolCallV1>,
    expected_decision: Option<ActDecision>,
    expected_failure_reason: String,
}

const PLANNER_CASES: &[&str] = &[
    include_str!("../../../tests/agent_prompt_golden/planner/bug_fix_static_check.json"),
    include_str!("../../../tests/agent_prompt_golden/planner/search_codebase.json"),
    include_str!("../../../tests/agent_prompt_golden/planner/read_file_evidence.json"),
    include_str!("../../../tests/agent_prompt_golden/planner/write_file_requires_approval.json"),
    include_str!("../../../tests/agent_prompt_golden/planner/ask_user_missing_context.json"),
];

const REFLECTOR_CASES: &[&str] = &[
    include_str!("../../../tests/agent_prompt_golden/reflector/success_continue.json"),
    include_str!("../../../tests/agent_prompt_golden/reflector/failure_tool_retry.json"),
    include_str!("../../../tests/agent_prompt_golden/reflector/unknown_missing_context.json"),
    include_str!("../../../tests/agent_prompt_golden/reflector/retry_with_new_args.json"),
    include_str!("../../../tests/agent_prompt_golden/reflector/stop_loss_repeated_failure.json"),
];

const TOOLCALL_CASES: &[&str] = &[
    include_str!("../../../tests/agent_prompt_golden/toolcall/safe_read.json"),
    include_str!("../../../tests/agent_prompt_golden/toolcall/risky_write.json"),
    include_str!("../../../tests/agent_prompt_golden/toolcall/cannot_act_missing_tool.json"),
];

fn has_mapping(case_id: &str, mappings: &[String], expected_doc: &str) {
    assert!(
        mappings
            .iter()
            .any(|mapping| mapping.contains(expected_doc)),
        "{case_id} must map to {expected_doc}"
    );
}

#[test]
fn prompt_golden_planner_cases_deserialize_and_validate() {
    assert_eq!(PLANNER_CASES.len(), 5);

    for raw in PLANNER_CASES {
        let case: PlannerGoldenCase =
            serde_json::from_str(raw).expect("planner golden case must deserialize");
        has_mapping(&case.id, &case.contract_mapping, "PLANNER-PROMPT-CONTRACT");
        assert!(
            case.contract_mapping
                .iter()
                .any(|m| m.contains("AGENT-PERSONA") || m.contains("TOOL-MANIFEST")),
            "{} must map to persona or tool manifest boundary",
            case.id
        );
        assert!(
            case.input.is_object(),
            "{} must include prompt-side input context",
            case.id
        );
        assert!(
            !case.expected_failure_reason.trim().is_empty(),
            "{} must describe the failure signal",
            case.id
        );

        let dto = case.expected;
        assert_eq!(
            dto.schema_version, "PlannerSubgoalPlanV1",
            "{} schema_version mismatch",
            case.id
        );
        dto.validate_dependencies()
            .unwrap_or_else(|err| panic!("{} dependency validation failed: {}", case.id, err));
        assert!(
            !dto.subgoals.is_empty(),
            "{} must include at least one subgoal",
            case.id
        );
        for subgoal in &dto.subgoals {
            assert!(
                !subgoal.expected_evidence.is_empty(),
                "{}:{} must include expected_evidence",
                case.id,
                subgoal.id_hint
            );
            assert!(
                !subgoal.stop_conditions.is_empty(),
                "{}:{} must include stop_conditions",
                case.id,
                subgoal.id_hint
            );
        }
    }
}

#[test]
fn prompt_golden_planner_covers_required_scenarios() {
    let mut ids = Vec::new();
    let mut saw_ask_user = false;
    let mut saw_risky_write = false;

    for raw in PLANNER_CASES {
        let case: PlannerGoldenCase =
            serde_json::from_str(raw).expect("planner golden case must deserialize");
        ids.push(case.id.clone());
        saw_ask_user |= case.id.contains("ask_user")
            || case
                .expected
                .subgoals
                .iter()
                .any(|sg| sg.id_hint.contains("ask_user"));
        saw_risky_write |= case
            .expected
            .subgoals
            .iter()
            .any(|sg| sg.requires_user_approval && matches!(sg.risk_level, RiskLevel::High));
    }

    for required in ["bug", "search", "read", "write", "ask_user"] {
        assert!(
            ids.iter().any(|id| id.contains(required)),
            "planner cases must cover {required}; got {ids:?}"
        );
    }
    assert!(saw_ask_user, "planner cases must include ask_user behavior");
    assert!(
        saw_risky_write,
        "planner cases must include risky write behavior"
    );
}

#[test]
fn prompt_golden_reflector_cases_deserialize_and_validate() {
    assert_eq!(REFLECTOR_CASES.len(), 5);

    for raw in REFLECTOR_CASES {
        let case: ReflectorGoldenCase =
            serde_json::from_str(raw).expect("reflector golden case must deserialize");
        has_mapping(&case.id, &case.contract_mapping, "REFLECTOR-CONTRACT");
        assert!(
            case.input.is_object(),
            "{} must include prompt-side input context",
            case.id
        );
        assert!(
            !case.expected_failure_reason.trim().is_empty(),
            "{} must describe the failure signal",
            case.id
        );

        let dto = case.expected;
        assert_eq!(
            dto.schema_version, "ReflectorCritiqueV1",
            "{} schema_version mismatch",
            case.id
        );
        assert!(
            !dto.evidence.is_empty(),
            "{} must include observable evidence",
            case.id
        );
        assert!(
            (0.0..=1.0).contains(&dto.confidence),
            "{} confidence must be normalized",
            case.id
        );
        assert!(
            (0.0..=1.0).contains(&dto.root_cause.confidence),
            "{} root cause confidence must be normalized",
            case.id
        );
    }
}

#[test]
fn prompt_golden_reflector_covers_failure_unknown_retry_stop_loss() {
    let parsed: Vec<ReflectorGoldenCase> = REFLECTOR_CASES
        .iter()
        .map(|raw| serde_json::from_str(raw).expect("reflector golden case must deserialize"))
        .collect();

    assert!(parsed.iter().any(|case| case.expected.success));
    assert!(parsed.iter().any(|case| !case.expected.success));
    assert!(parsed
        .iter()
        .any(|case| case.expected.root_cause.category == RootCauseCategory::Unknown));
    assert!(parsed.iter().any(|case| {
        case.expected
            .plan_adjustment
            .as_ref()
            .map(|adj| adj.action == RecommendedAction::RetryWithNewArgs)
            .unwrap_or(false)
    }));
    assert!(parsed.iter().any(|case| {
        case.expected
            .stop_loss
            .as_ref()
            .map(|stop| stop.triggered)
            .unwrap_or(false)
    }));
}

#[test]
fn prompt_golden_toolcall_cases_deserialize_and_validate() {
    assert_eq!(TOOLCALL_CASES.len(), 3);

    for raw in TOOLCALL_CASES {
        let case: ToolCallGoldenCase =
            serde_json::from_str(raw).expect("toolcall golden case must deserialize");
        has_mapping(&case.id, &case.contract_mapping, "EXECUTOR-CONTRACT");
        assert!(
            case.input.is_object(),
            "{} must include prompt-side input context",
            case.id
        );
        assert!(
            !case.expected_failure_reason.trim().is_empty(),
            "{} must describe the failure signal",
            case.id
        );
        assert!(
            case.expected_tool_call.is_some() || case.expected_decision.is_some(),
            "{} must include either expected_tool_call or expected_decision",
            case.id
        );

        if let Some(call) = case.expected_tool_call {
            assert_eq!(call.schema_version, "ToolCallV1");
            assert_eq!(call.action_type, ActionType::CallTool);
            assert!(
                call.parameters.is_object(),
                "{} ToolCall parameters must be a JSON object",
                case.id
            );
            assert!(
                !call.expected_evidence.trim().is_empty(),
                "{} ToolCall must include expected_evidence",
                case.id
            );
        }
    }
}

#[test]
fn prompt_golden_toolcall_covers_safe_read_risky_write_and_cannot_act() {
    let parsed: Vec<ToolCallGoldenCase> = TOOLCALL_CASES
        .iter()
        .map(|raw| serde_json::from_str(raw).expect("toolcall golden case must deserialize"))
        .collect();

    assert!(parsed.iter().any(|case| {
        case.id.contains("safe_read")
            && case
                .expected_tool_call
                .as_ref()
                .map(|call| {
                    call.tool_name == "read_file"
                        && !call.governance_required
                        && matches!(call.risk_level, RiskLevel::Low)
                })
                .unwrap_or(false)
    }));
    assert!(parsed.iter().any(|case| {
        case.id.contains("risky_write")
            && case
                .expected_tool_call
                .as_ref()
                .map(|call| call.governance_required && matches!(call.risk_level, RiskLevel::High))
                .unwrap_or(false)
    }));
    assert!(parsed.iter().any(|case| {
        case.id.contains("cannot_act")
            && matches!(case.expected_decision, Some(ActDecision::CannotAct { .. }))
    }));
}
