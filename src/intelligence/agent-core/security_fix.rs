use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
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
        if let Some(cmd) = &finding.regression_test {
            validation_commands.push(cmd.clone());
        } else {
            validation_commands.push("npm run test:security-gate".to_string());
        }

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_workflow::{
        FindingSeverity, FindingStatus, SecurityCategory, SecurityFinding,
    };

    fn make_test_finding(severity: FindingSeverity) -> SecurityFinding {
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
            regression_test: Some("npm run test:security-gate".to_string()),
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
        let finding = make_test_finding(FindingSeverity::Medium);
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
    }

    #[test]
    fn test_planner_enforces_dry_run_even_if_requested_false() {
        let planner = SecurityFixPlanner::with_feature_enabled(false);
        let request = FixFindingRequest {
            finding_id: "FIND-TEST-123".to_string(),
            dry_run: false,
        };
        let finding = make_test_finding(FindingSeverity::Low);
        let plan = planner.plan(&request, &finding);

        // Enforce dry_run is true even if dry_run request is false
        assert!(plan.dry_run);
    }

    #[test]
    fn test_high_critical_severity_enforces_human_review_and_no_auto_apply() {
        let planner = SecurityFixPlanner::with_feature_enabled(true);
        let request = FixFindingRequest {
            finding_id: "FIND-TEST-123".to_string(),
            dry_run: false,
        };

        // Critical Severity finding
        let finding_critical = make_test_finding(FindingSeverity::Critical);
        let plan_critical = planner.plan(&request, &finding_critical);
        assert!(plan_critical.human_review_required);
        assert!(plan_critical.dry_run);
        assert_eq!(plan_critical.risk_level, RiskLevel::Critical);

        // High Severity finding
        let finding_high = make_test_finding(FindingSeverity::High);
        let plan_high = planner.plan(&request, &finding_high);
        assert!(plan_high.human_review_required);
        assert!(plan_high.dry_run);
        assert_eq!(plan_high.risk_level, RiskLevel::High);
    }
}
