use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevalidationStatus {
    NotRun,
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevalidationReceipt {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub status: RevalidationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchEdit {
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchPlan {
    pub finding_id: String,
    pub risk_level: RiskLevel,
    pub recommendation: String,
    pub human_review_required: bool,
    pub dry_run: bool,
    pub files: Vec<String>,
    pub edits: Vec<PatchEdit>,
    pub validation_commands: Vec<String>,
    pub rollback_plan: String,
    pub revalidation_receipt: Option<RevalidationReceipt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixFindingRequest {
    pub finding_id: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityFixPlanner {
    pub feature_enabled: bool,
}

impl Default for SecurityFixPlanner {
    fn default() -> Self {
        Self::new()
    }
}

pub fn is_validation_command_allowed(command: &str) -> bool {
    // 1. Reject shell concatenation / redirection / command substitution tokens
    let rejected_tokens = &["&&", ";", "|", "`", "$(", ">", "<"];
    if rejected_tokens.iter().any(|&token| command.contains(token)) {
        return false;
    }

    let c_lower = command.trim().to_lowercase();
    // 2. Reject dangerous commands/patterns per strict safety criteria
    if c_lower.contains("curl")
        || c_lower.contains("wget")
        || c_lower.contains("rm ")
        || c_lower.contains("eval")
        || c_lower.contains("http://")
        || c_lower.contains("https://")
        || c_lower.contains("bash")
        || c_lower.contains("powershell")
        || c_lower.contains("cmd")
    {
        return false;
    }

    // 3. Only allow local commands from safe whitelist (precise exact matching)
    let allowed_commands = &[
        "npm run test:security-gate",
        "npm run security:report",
        "cargo test -p engine-tool-system security",
        "cargo test -p intelligence-agent-core security_workflow",
        "git diff",
    ];

    allowed_commands.contains(&c_lower.as_str())
}

impl SecurityFixPlanner {
    pub fn new() -> Self {
        Self {
            feature_enabled: std::env::var("HAJIMI_SECURITY_FIX_ENABLED")
                .map(|val| val == "true")
                .unwrap_or(false),
        }
    }

    pub fn with_feature_enabled(feature_enabled: bool) -> Self {
        Self { feature_enabled }
    }

    pub fn transition_status(
        &self,
        current_status: &str,
        receipt: &Option<RevalidationReceipt>,
    ) -> String {
        match current_status {
            "planned" => {
                if let Some(r) = receipt {
                    if r.status == RevalidationStatus::Pass || r.status == RevalidationStatus::Fail {
                        "applied".to_string()
                    } else {
                        "planned".to_string()
                    }
                } else {
                    "planned".to_string()
                }
            }
            "applied" => {
                if let Some(r) = receipt {
                    if r.status == RevalidationStatus::Pass {
                        "revalidated".to_string()
                    } else {
                        "applied".to_string()
                    }
                } else {
                    "applied".to_string()
                }
            }
            other => other.to_string(),
        }
    }

    pub fn plan(
        &self,
        request: &FixFindingRequest,
        finding: &crate::security_workflow::SecurityFinding,
    ) -> PatchPlan {
        // Safe check: force dry_run to true for safety bounds
        let is_dry_run = request.dry_run || !self.feature_enabled || true;

        let risk_level = match finding.severity {
            crate::security_workflow::FindingSeverity::Low => RiskLevel::Low,
            crate::security_workflow::FindingSeverity::Medium => RiskLevel::Medium,
            crate::security_workflow::FindingSeverity::High => RiskLevel::High,
            crate::security_workflow::FindingSeverity::Critical => RiskLevel::Critical,
        };

        // All severity levels default to human_review_required = true for secure execution path
        let human_review_required = match risk_level {
            RiskLevel::High | RiskLevel::Critical => true,
            _ => true,
        };

        let files = finding.file.clone().map(|f| vec![f]).unwrap_or_default();

        // Create virtual edits array only; no direct file write operations are made here
        let edits = finding
            .file
            .clone()
            .map(|f| {
                vec![PatchEdit {
                    file: f,
                    start_line: finding.line.unwrap_or(1),
                    end_line: finding.line.unwrap_or(1),
                    replacement: format!("// TODO: Security Remediation: {}", finding.recommendation),
                }]
            })
            .unwrap_or_default();

        let mut validation_commands = Vec::new();
        let cmd = finding
            .regression_test
            .as_deref()
            .unwrap_or("npm run test:security-gate");

        validation_commands.push(cmd.to_string());

        let receipt = if !is_validation_command_allowed(cmd) {
            RevalidationReceipt {
                command: cmd.to_string(),
                exit_code: Some(1),
                stdout_summary: "Command rejected due to security policy violation.".to_string(),
                stderr_summary: "Dangerous execution command rejected by SecurityFixPlanner allowlist gate.".to_string(),
                status: RevalidationStatus::Fail,
            }
        } else {
            RevalidationReceipt {
                command: cmd.to_string(),
                exit_code: None,
                stdout_summary: "Allowed local command pending execution.".to_string(),
                stderr_summary: String::new(),
                status: RevalidationStatus::NotRun,
            }
        };

        let rollback_plan = format!(
            "git checkout -- {}",
            finding.file.as_deref().unwrap_or(".")
        );

        PatchPlan {
            finding_id: finding.finding_id.clone(),
            risk_level,
            recommendation: finding.recommendation.clone(),
            human_review_required,
            dry_run: is_dry_run,
            files,
            edits,
            validation_commands,
            rollback_plan,
            revalidation_receipt: Some(receipt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_workflow::{
        FindingSeverity, FindingStatus, SecurityCategory, SecurityFinding,
    };

    fn make_test_finding(severity: FindingSeverity, test_cmd: Option<String>) -> SecurityFinding {
        SecurityFinding {
            finding_id: "FIND-TEST-123".to_string(),
            rule_id: "RULE-XSS".to_string(),
            title: "XSS vulnerability in frontend".to_string(),
            severity,
            category: SecurityCategory::FrontendXss,
            status: FindingStatus::Unverified,
            confidence: 0.85,
            type_: "manual_check".to_string(),
            file: Some("src/interface/web/app.js".to_string()),
            line: Some(250),
            snippet: Some("element.innerHTML = userInput;".to_string()),
            evidence: vec![],
            attack_path: None,
            recommendation: "Use textContent instead of innerHTML".to_string(),
            regression_test: test_cmd.or(Some("npm run test:security-gate".to_string())),
            human_review_required: false,
            validation_receipts: vec![],
            residual_risk: vec![],
        }
    }

    #[test]
    fn test_planner_creates_dry_run_patch_plan() {
        let planner = SecurityFixPlanner::with_feature_enabled(true);
        let request = FixFindingRequest {
            finding_id: "FIND-TEST-123".to_string(),
            dry_run: true,
        };
        let finding = make_test_finding(FindingSeverity::Medium, None);
        let plan = planner.plan(&request, &finding);

        assert_eq!(plan.finding_id, "FIND-TEST-123");
        assert!(plan.dry_run);
        assert!(plan.human_review_required);
        assert_eq!(plan.files, vec!["src/interface/web/app.js".to_string()]);
        assert_eq!(plan.edits.len(), 1);
        assert_eq!(
            plan.validation_commands,
            vec!["npm run test:security-gate".to_string()]
        );
        assert!(plan.rollback_plan.contains("git checkout"));
        assert!(plan.revalidation_receipt.is_some());
        assert_eq!(
            plan.revalidation_receipt.unwrap().status,
            RevalidationStatus::NotRun
        );
    }

    #[test]
    fn test_dangerous_commands_are_rejected() {
        assert!(!is_validation_command_allowed("rm -rf /"));
        assert!(!is_validation_command_allowed("curl http://malicious.site"));
        assert!(!is_validation_command_allowed("wget https://malicious.site"));
        assert!(!is_validation_command_allowed("bash -c malicious"));
        assert!(!is_validation_command_allowed("powershell -Command malicious"));

        // Injection payloads must be rejected
        assert!(!is_validation_command_allowed("npm run test:security-gate && cat ~/.ssh/id_rsa"));
        assert!(!is_validation_command_allowed("npm run test:security-gate; echo pwned"));
        assert!(!is_validation_command_allowed("git diff | cat"));

        // Non-exact match (substring or trailing chars) must be rejected
        assert!(!is_validation_command_allowed("git diff malicious_append"));

        // Allowed safe local checks (precise matching only)
        assert!(is_validation_command_allowed("npm run test:security-gate"));
        assert!(is_validation_command_allowed("cargo test -p engine-tool-system security"));
        assert!(is_validation_command_allowed("git diff"));
    }

    #[test]
    fn test_planner_rejects_dangerous_regression_test() {
        let planner = SecurityFixPlanner::with_feature_enabled(true);
        let request = FixFindingRequest {
            finding_id: "FIND-TEST-123".to_string(),
            dry_run: true,
        };
        let finding = make_test_finding(FindingSeverity::Medium, Some("rm -rf /".to_string()));
        let plan = planner.plan(&request, &finding);

        let receipt = plan.revalidation_receipt.unwrap();
        assert_eq!(receipt.status, RevalidationStatus::Fail);
        assert!(receipt.stdout_summary.contains("rejected"));
        assert!(receipt.stderr_summary.contains("Dangerous"));
    }

    #[test]
    fn test_transition_status_planned_applied_revalidated() {
        let planner = SecurityFixPlanner::with_feature_enabled(true);

        // 1. planned -> planned (without receipt)
        let status1 = planner.transition_status("planned", &None);
        assert_eq!(status1, "planned");

        // 2. planned -> planned (with NotRun receipt)
        let receipt_not_run = RevalidationReceipt {
            command: "npm run test:security-gate".to_string(),
            exit_code: None,
            stdout_summary: String::new(),
            stderr_summary: String::new(),
            status: RevalidationStatus::NotRun,
        };
        let status2 = planner.transition_status("planned", &Some(receipt_not_run.clone()));
        assert_eq!(status2, "planned");

        // 3. planned -> applied (with Pass or Fail receipt)
        let receipt_pass = RevalidationReceipt {
            command: "npm run test:security-gate".to_string(),
            exit_code: Some(0),
            stdout_summary: String::new(),
            stderr_summary: String::new(),
            status: RevalidationStatus::Pass,
        };
        let status2_applied = planner.transition_status("planned", &Some(receipt_pass.clone()));
        assert_eq!(status2_applied, "applied");

        // 4. applied -> revalidated (with passing receipt)
        let status3 = planner.transition_status("applied", &Some(receipt_pass));
        assert_eq!(status3, "revalidated");

        // 5. applied -> applied (with failing receipt)
        let receipt_fail = RevalidationReceipt {
            command: "npm run test:security-gate".to_string(),
            exit_code: Some(1),
            stdout_summary: String::new(),
            stderr_summary: String::new(),
            status: RevalidationStatus::Fail,
        };
        let status4 = planner.transition_status("applied", &Some(receipt_fail));
        assert_eq!(status4, "applied");
    }

    #[test]
    fn test_high_critical_severity_enforces_human_review_and_no_auto_apply() {
        let planner = SecurityFixPlanner::with_feature_enabled(true);
        let request = FixFindingRequest {
            finding_id: "FIND-TEST-123".to_string(),
            dry_run: false,
        };

        // Critical Severity finding
        let finding_critical = make_test_finding(FindingSeverity::Critical, None);
        let plan_critical = planner.plan(&request, &finding_critical);
        assert!(plan_critical.human_review_required);
        assert!(plan_critical.dry_run);
        assert_eq!(plan_critical.risk_level, RiskLevel::Critical);

        // High Severity finding
        let finding_high = make_test_finding(FindingSeverity::High, None);
        let plan_high = planner.plan(&request, &finding_high);
        assert!(plan_high.human_review_required);
        assert!(plan_high.dry_run);
        assert_eq!(plan_high.risk_level, RiskLevel::High);
    }
}
