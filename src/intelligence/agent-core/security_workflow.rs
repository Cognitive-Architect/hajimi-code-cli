use serde::{Deserialize, Serialize};

const SECURITY_WORKFLOW_GATE: &str = "HAJIMI_SECURITY_WORKFLOW_ENABLED";
const VALIDATION_COMMAND_ALLOWLIST: &[&str] = &[
    "npm run test:security-gate",
    "npm run security:report",
    "cargo test -p engine-tool-system security",
    "cargo test -p intelligence-agent-core security_workflow",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityWorkflowKind {
    SecurityScan,
    ThreatModel,
    FindingDiscovery,
    AttackPathAnalysis,
    Validation,
    FixFinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Candidate,
    Unverified,
    Confirmed,
    Fixed,
    AcceptedRisk,
    FalsePositive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityCategory {
    Secret,
    PanicSafety,
    TauriSecurity,
    FrontendXss,
    WorkspaceFs,
    SupplyChain,
    Governance,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Pass,
    Fail,
    NotRun,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityScope {
    pub repository: String,
    pub branch: String,
    pub commit: String,
    pub paths: Vec<String>,
    pub out_of_scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub snippet: Option<String>,
    pub command: Option<String>,
    pub output_hash: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReceipt {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub status: ValidationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatModelSummary {
    pub assets: Vec<String>,
    pub entry_points: Vec<String>,
    pub trust_boundaries: Vec<String>,
    pub high_risk_operations: Vec<String>,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub finding_id: String,
    pub rule_id: String,
    pub title: String,
    pub severity: FindingSeverity,
    pub category: SecurityCategory,
    pub status: FindingStatus,
    pub confidence: f32,
    #[serde(rename = "type")]
    pub type_: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub snippet: Option<String>,
    pub evidence: Vec<Evidence>,
    pub attack_path: Option<String>,
    pub recommendation: String,
    pub regression_test: Option<String>,
    pub human_review_required: bool,
    pub validation_receipts: Vec<ValidationReceipt>,
    pub residual_risk: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityWorkflowRequest {
    pub kind: SecurityWorkflowKind,
    pub scope: SecurityScope,
    pub dry_run: bool,
    pub max_findings: usize,
    pub findings: Vec<SecurityFinding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityWorkflowSummary {
    pub total_findings: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub unverified: usize,
    pub confirmed: usize,
    pub accepted_risk: usize,
    pub fixed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityWorkflowReport {
    pub kind: SecurityWorkflowKind,
    pub scope: SecurityScope,
    pub dry_run: bool,
    pub feature_enabled: bool,
    pub summary: SecurityWorkflowSummary,
    pub findings: Vec<SecurityFinding>,
    pub threat_model: Option<ThreatModelSummary>,
    pub validation_receipts: Vec<ValidationReceipt>,
    pub workflow_notes: Vec<String>,
    pub residual_risk: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecurityWorkflowOrchestrator {
    feature_enabled: bool,
}

impl Default for SecurityWorkflowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityWorkflowOrchestrator {
    pub fn new() -> Self {
        Self {
            feature_enabled: crate::prompts::is_security_workflow_enabled(),
        }
    }

    pub fn with_feature_enabled(feature_enabled: bool) -> Self {
        Self { feature_enabled }
    }

    pub fn run(&self, request: SecurityWorkflowRequest) -> SecurityWorkflowReport {
        let mut findings: Vec<SecurityFinding> = request
            .findings
            .into_iter()
            .take(request.max_findings)
            .map(classify_status)
            .collect();

        let mut workflow_notes = Vec::new();
        if !self.feature_enabled {
            workflow_notes.push(format!(
                "{} is disabled; workflow is report-only.",
                SECURITY_WORKFLOW_GATE
            ));
        }

        let mut threat_model_summary = None;
        match request.kind {
            SecurityWorkflowKind::SecurityScan => {
                security_scan(&mut findings, &mut workflow_notes);
            }
            SecurityWorkflowKind::ThreatModel => {
                threat_model_summary =
                    Some(threat_model(&request.scope, &findings, &mut workflow_notes));
            }
            SecurityWorkflowKind::FindingDiscovery => {
                finding_discovery(&mut findings, &mut workflow_notes);
            }
            SecurityWorkflowKind::AttackPathAnalysis => {
                attack_path_analysis(&mut findings, &mut workflow_notes);
            }
            SecurityWorkflowKind::Validation => {
                validation(&mut findings, &mut workflow_notes);
            }
            SecurityWorkflowKind::FixFinding => {
                workflow_notes.push(
                    "fix_finding remains dry-run planning only in this orchestrator.".to_string(),
                );
                let planner = crate::security_fix::SecurityFixPlanner::new();
                for finding in &findings {
                    let request = crate::security_fix::FixFindingRequest {
                        finding_id: finding.finding_id.clone(),
                        dry_run: true,
                    };
                    let plan = planner.plan(&request, finding);
                    workflow_notes.push(format!(
                        "Generated PatchPlan for finding {}: Risk Level: {:?}, Recommendation: {}, Human Review Required: {}, Rollback Plan: {}",
                        plan.finding_id,
                        plan.risk_level,
                        plan.recommendation,
                        plan.human_review_required,
                        plan.rollback_plan
                    ));
                }
            }
        }

        let validation_receipts = collect_validation_receipts(&findings);
        let summary = summarize_findings(&findings);
        let residual_risk = assemble_residual_risk(
            request.dry_run,
            self.feature_enabled,
            &findings,
            &workflow_notes,
        );

        SecurityWorkflowReport {
            kind: request.kind,
            scope: request.scope,
            dry_run: request.dry_run,
            feature_enabled: self.feature_enabled,
            summary,
            findings,
            threat_model: threat_model_summary,
            validation_receipts,
            workflow_notes,
            residual_risk,
        }
    }
}

fn security_scan(findings: &mut [SecurityFinding], workflow_notes: &mut Vec<String>) {
    workflow_notes.push(format!(
        "security_scan consumed {} structured local findings.",
        findings.len()
    ));
    for finding in findings {
        finding.residual_risk.push(
            "security_scan is limited to supplied local findings and V1 gate/report input."
                .to_string(),
        );
    }
}

fn threat_model(
    scope: &SecurityScope,
    findings: &[SecurityFinding],
    workflow_notes: &mut Vec<String>,
) -> ThreatModelSummary {
    workflow_notes.push("threat_model assembled only from scope and finding evidence.".to_string());

    let mut entry_points: Vec<String> = findings
        .iter()
        .filter_map(|finding| finding.file.clone())
        .collect();
    entry_points.sort();
    entry_points.dedup();
    if entry_points.is_empty() {
        entry_points.push("No code-backed entry point evidence supplied.".to_string());
    }

    let mut high_risk_operations = Vec::new();
    for finding in findings {
        match finding.category {
            SecurityCategory::Secret => {
                high_risk_operations.push("credential handling".to_string())
            }
            SecurityCategory::TauriSecurity => {
                high_risk_operations.push("Tauri command boundary".to_string())
            }
            SecurityCategory::FrontendXss => {
                high_risk_operations.push("DOM rendering surface".to_string())
            }
            SecurityCategory::WorkspaceFs => {
                high_risk_operations.push("workspace file operation".to_string())
            }
            SecurityCategory::SupplyChain => {
                high_risk_operations.push("dependency loading".to_string())
            }
            SecurityCategory::PanicSafety
            | SecurityCategory::Governance
            | SecurityCategory::Unknown => {}
        }
    }
    high_risk_operations.sort();
    high_risk_operations.dedup();

    ThreatModelSummary {
        assets: if scope.paths.is_empty() {
            vec![scope.repository.clone()]
        } else {
            scope.paths.clone()
        },
        entry_points,
        trust_boundaries: vec![
            "local tool execution boundary".to_string(),
            "workspace path resolver boundary".to_string(),
            "LLM-to-tool decision boundary".to_string(),
        ],
        high_risk_operations,
        assumptions: vec![
            "Threat model uses only supplied local evidence.".to_string(),
            "Missing evidence remains residual risk, not confirmation.".to_string(),
        ],
    }
}

fn finding_discovery(findings: &mut [SecurityFinding], workflow_notes: &mut Vec<String>) {
    workflow_notes.push("finding_discovery classified supplied findings by evidence.".to_string());
    for finding in findings {
        *finding = classify_status(finding.clone());
    }
}

fn attack_path_analysis(findings: &mut [SecurityFinding], workflow_notes: &mut Vec<String>) {
    workflow_notes
        .push("attack_path_analysis produced human-readable narratives only.".to_string());
    for finding in findings {
        if finding.attack_path.is_none() && !finding.evidence.is_empty() {
            finding.attack_path = Some(format!(
                "Human-readable path: evidence for {} appears at {}:{} and should be reviewed before any remediation.",
                finding.rule_id,
                finding.file.as_deref().unwrap_or("unknown"),
                finding
                    .line
                    .map(|line| line.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ));
        }
    }
}

fn validation(findings: &mut [SecurityFinding], workflow_notes: &mut Vec<String>) {
    workflow_notes.push(
        "validation records allowlisted local commands as pending receipts; it does not execute them."
            .to_string(),
    );
    for finding in findings {
        if finding.validation_receipts.is_empty() {
            let receipt = match finding.regression_test.as_deref() {
                Some(command) if is_validation_command_allowed(command) => ValidationReceipt {
                    command: command.to_string(),
                    exit_code: None,
                    stdout_summary:
                        "Allowed local command was not executed by report-only workflow."
                            .to_string(),
                    stderr_summary: String::new(),
                    status: ValidationStatus::Pending,
                },
                Some(command) => ValidationReceipt {
                    command: command.to_string(),
                    exit_code: None,
                    stdout_summary: "Command is outside the validation allowlist.".to_string(),
                    stderr_summary: String::new(),
                    status: ValidationStatus::NotRun,
                },
                None => ValidationReceipt {
                    command: "validation command not supplied".to_string(),
                    exit_code: None,
                    stdout_summary: "No validation command was supplied.".to_string(),
                    stderr_summary: String::new(),
                    status: ValidationStatus::NotRun,
                },
            };
            finding.validation_receipts.push(receipt);
        }
        *finding = classify_status(finding.clone());
    }
}

fn is_validation_command_allowed(command: &str) -> bool {
    VALIDATION_COMMAND_ALLOWLIST.contains(&command)
}

fn classify_status(mut finding: SecurityFinding) -> SecurityFinding {
    let has_evidence = !finding.evidence.is_empty();
    let has_pass_validation = finding
        .validation_receipts
        .iter()
        .any(|receipt| receipt.status == ValidationStatus::Pass);

    finding.status = match finding.status {
        FindingStatus::Confirmed if !has_evidence || !has_pass_validation => {
            FindingStatus::Unverified
        }
        FindingStatus::Fixed if !has_evidence || !has_pass_validation => FindingStatus::Unverified,
        other => other,
    };

    if !has_evidence && finding.confidence > 0.4 {
        finding.confidence = 0.4;
    }

    finding.confidence = finding.confidence.clamp(0.0, 1.0);
    finding.human_review_required = finding.human_review_required
        || matches!(
            finding.severity,
            FindingSeverity::High | FindingSeverity::Critical
        );
    finding
}

fn collect_validation_receipts(findings: &[SecurityFinding]) -> Vec<ValidationReceipt> {
    findings
        .iter()
        .flat_map(|finding| finding.validation_receipts.iter().cloned())
        .collect()
}

fn summarize_findings(findings: &[SecurityFinding]) -> SecurityWorkflowSummary {
    let mut summary = SecurityWorkflowSummary {
        total_findings: findings.len(),
        ..SecurityWorkflowSummary::default()
    };

    for finding in findings {
        match finding.severity {
            FindingSeverity::Critical => summary.critical += 1,
            FindingSeverity::High => summary.high += 1,
            FindingSeverity::Medium => summary.medium += 1,
            FindingSeverity::Low => summary.low += 1,
        }

        match finding.status {
            FindingStatus::Unverified => summary.unverified += 1,
            FindingStatus::Confirmed => summary.confirmed += 1,
            FindingStatus::AcceptedRisk => summary.accepted_risk += 1,
            FindingStatus::Fixed => summary.fixed += 1,
            FindingStatus::Candidate | FindingStatus::FalsePositive => {}
        }
    }

    summary
}

fn assemble_residual_risk(
    dry_run: bool,
    feature_enabled: bool,
    findings: &[SecurityFinding],
    workflow_notes: &[String],
) -> Vec<String> {
    let mut risks = Vec::new();
    if dry_run {
        risks.push("Fix planning is dry-run only; no changes are applied.".to_string());
    }
    if !feature_enabled {
        risks.push(format!(
            "{} is disabled; workflow remains report-only.",
            SECURITY_WORKFLOW_GATE
        ));
    }
    if findings.iter().any(|finding| finding.evidence.is_empty()) {
        risks.push("Some findings lack evidence and remain unverified.".to_string());
    }
    if findings
        .iter()
        .any(|finding| finding.validation_receipts.is_empty())
    {
        risks.push("Some findings have no validation receipt yet.".to_string());
    }
    risks.extend(workflow_notes.iter().cloned());
    risks.extend(
        findings
            .iter()
            .flat_map(|finding| finding.residual_risk.iter().cloned()),
    );
    risks.sort();
    risks.dedup();
    risks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> SecurityScope {
        SecurityScope {
            repository: "hajimi-code-cli".to_string(),
            branch: "codex/security-workflow-day01".to_string(),
            commit: "test-commit".to_string(),
            paths: vec!["src/intelligence/agent-core".to_string()],
            out_of_scope: vec!["manual ui smoke".to_string()],
        }
    }

    fn code_evidence() -> Evidence {
        Evidence {
            kind: "code".to_string(),
            file: Some("src/intelligence/agent-core/security_workflow.rs".to_string()),
            line: Some(1),
            snippet: Some("SecurityWorkflowReport".to_string()),
            command: None,
            output_hash: None,
            note: Some("DTO assembly evidence".to_string()),
        }
    }

    fn pass_receipt() -> ValidationReceipt {
        ValidationReceipt {
            command: "cargo test -p intelligence-agent-core security_workflow".to_string(),
            exit_code: Some(0),
            stdout_summary: "security_workflow tests passed".to_string(),
            stderr_summary: String::new(),
            status: ValidationStatus::Pass,
        }
    }

    fn finding(status: FindingStatus, evidence: Vec<Evidence>) -> SecurityFinding {
        SecurityFinding {
            finding_id: "FINDING-001".to_string(),
            rule_id: "DTO-001".to_string(),
            title: "DTO workflow finding".to_string(),
            severity: FindingSeverity::High,
            category: SecurityCategory::Governance,
            status,
            confidence: 0.95,
            type_: "Workflow".to_string(),
            file: Some("src/intelligence/agent-core/security_workflow.rs".to_string()),
            line: Some(1),
            snippet: Some("SecurityWorkflowRequest".to_string()),
            evidence,
            attack_path: None,
            recommendation: "Keep report assembly evidence-first.".to_string(),
            regression_test: Some(
                "cargo test -p intelligence-agent-core security_workflow".to_string(),
            ),
            human_review_required: false,
            validation_receipts: vec![],
            residual_risk: vec!["Workflow behavior remains report-only.".to_string()],
        }
    }

    #[test]
    fn security_workflow_report_assembly_summarizes_findings() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let mut confirmed = finding(FindingStatus::Confirmed, vec![code_evidence()]);
        confirmed.validation_receipts.push(pass_receipt());
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::SecurityScan,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![confirmed],
        });

        assert!(report.dry_run);
        assert!(report.feature_enabled);
        assert_eq!(report.summary.total_findings, 1);
        assert_eq!(report.summary.high, 1);
        assert_eq!(report.summary.confirmed, 1);
        assert!(report.findings[0].human_review_required);
        assert!(report
            .residual_risk
            .iter()
            .any(|risk| risk.contains("dry-run")));
    }

    #[test]
    fn security_workflow_no_evidence_cannot_remain_confirmed() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::FindingDiscovery,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(FindingStatus::Confirmed, vec![])],
        });

        let finding = &report.findings[0];
        assert_eq!(finding.status, FindingStatus::Unverified);
        assert_eq!(finding.confidence, 0.4);
        assert_eq!(report.summary.unverified, 1);
        assert_eq!(report.summary.confirmed, 0);
    }

    #[test]
    fn security_workflow_respects_max_findings() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::SecurityScan,
            scope: scope(),
            dry_run: true,
            max_findings: 1,
            findings: vec![
                finding(FindingStatus::Unverified, vec![]),
                finding(FindingStatus::Unverified, vec![]),
            ],
        });

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.summary.total_findings, 1);
    }

    #[test]
    fn security_workflow_dto_serializes_snake_case_contract() {
        let request = SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::FixFinding,
            scope: scope(),
            dry_run: true,
            max_findings: 5,
            findings: vec![],
        };

        let json = serde_json::to_string(&request).expect("request should serialize");
        assert!(json.contains("\"kind\":\"fix_finding\""));
        assert!(json.contains("\"dry_run\":true"));
        assert!(json.contains("\"max_findings\":5"));
    }

    #[test]
    fn security_workflow_threat_model_uses_supplied_evidence() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::ThreatModel,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(FindingStatus::Unverified, vec![code_evidence()])],
        });

        let model = report.threat_model.expect("threat model should exist");
        assert!(model
            .entry_points
            .iter()
            .any(|entry| entry.contains("security_workflow.rs")));
        assert!(model
            .trust_boundaries
            .iter()
            .any(|boundary| boundary.contains("tool")));
    }

    #[test]
    fn security_workflow_attack_path_is_human_readable_only() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::AttackPathAnalysis,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(FindingStatus::Unverified, vec![code_evidence()])],
        });

        let narrative = report.findings[0]
            .attack_path
            .as_ref()
            .expect("narrative should exist");
        assert!(narrative.contains("Human-readable path"));
        assert!(!narrative.contains("#!"));
    }

    #[test]
    fn security_workflow_validation_uses_allowlist_without_running_commands() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::Validation,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(FindingStatus::Unverified, vec![code_evidence()])],
        });

        assert_eq!(report.validation_receipts.len(), 1);
        assert_eq!(
            report.validation_receipts[0].status,
            ValidationStatus::Pending
        );
        assert_eq!(report.validation_receipts[0].exit_code, None);
    }

    #[test]
    fn security_workflow_feature_gate_off_is_report_only() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(false);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::SecurityScan,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(FindingStatus::Unverified, vec![])],
        });

        assert!(!report.feature_enabled);
        assert!(report
            .workflow_notes
            .iter()
            .any(|note| note.contains(SECURITY_WORKFLOW_GATE)));
        assert!(report
            .residual_risk
            .iter()
            .any(|risk| risk.contains("report-only")));
    }

    #[test]
    fn security_workflow_fix_finding_runs_planner() {
        let orchestrator = SecurityWorkflowOrchestrator::with_feature_enabled(true);
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::FixFinding,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(FindingStatus::Unverified, vec![code_evidence()])],
        });

        assert!(report.workflow_notes.iter().any(|note| note.contains("Generated PatchPlan")));
    }
}
