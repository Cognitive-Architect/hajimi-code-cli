use serde::{Deserialize, Serialize};

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
    pub summary: SecurityWorkflowSummary,
    pub findings: Vec<SecurityFinding>,
    pub validation_receipts: Vec<ValidationReceipt>,
    pub residual_risk: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SecurityWorkflowOrchestrator;

impl SecurityWorkflowOrchestrator {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, request: SecurityWorkflowRequest) -> SecurityWorkflowReport {
        let mut findings: Vec<SecurityFinding> = request
            .findings
            .into_iter()
            .take(request.max_findings)
            .map(enforce_evidence_contract)
            .collect();

        let validation_receipts = collect_validation_receipts(&findings);
        let summary = summarize_findings(&findings);
        let residual_risk = assemble_residual_risk(request.dry_run, &findings);

        SecurityWorkflowReport {
            kind: request.kind,
            scope: request.scope,
            dry_run: request.dry_run,
            summary,
            findings: std::mem::take(&mut findings),
            validation_receipts,
            residual_risk,
        }
    }
}

fn enforce_evidence_contract(mut finding: SecurityFinding) -> SecurityFinding {
    if finding.evidence.is_empty()
        && matches!(
            finding.status,
            FindingStatus::Confirmed | FindingStatus::Fixed
        )
    {
        finding.status = FindingStatus::Unverified;
    }

    if finding.evidence.is_empty() && finding.confidence > 0.4 {
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

fn assemble_residual_risk(dry_run: bool, findings: &[SecurityFinding]) -> Vec<String> {
    let mut risks = Vec::new();
    if dry_run {
        risks.push("Fix planning is dry-run only; no changes are applied.".to_string());
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
            out_of_scope: vec!["ui integration".to_string()],
        }
    }

    fn finding(status: FindingStatus, evidence: Vec<Evidence>) -> SecurityFinding {
        SecurityFinding {
            finding_id: "FINDING-001".to_string(),
            rule_id: "DTO-001".to_string(),
            title: "DTO skeleton finding".to_string(),
            severity: FindingSeverity::High,
            category: SecurityCategory::Governance,
            status,
            confidence: 0.95,
            type_: "Skeleton".to_string(),
            file: Some("src/intelligence/agent-core/security_workflow.rs".to_string()),
            line: Some(1),
            snippet: Some("SecurityWorkflowRequest".to_string()),
            evidence,
            recommendation: "Keep report assembly evidence-first.".to_string(),
            regression_test: Some(
                "cargo test -p intelligence-agent-core security_workflow".to_string(),
            ),
            human_review_required: false,
            validation_receipts: vec![],
            residual_risk: vec!["Workflow behavior remains a skeleton.".to_string()],
        }
    }

    #[test]
    fn security_workflow_report_assembly_summarizes_findings() {
        let orchestrator = SecurityWorkflowOrchestrator::new();
        let report = orchestrator.run(SecurityWorkflowRequest {
            kind: SecurityWorkflowKind::SecurityScan,
            scope: scope(),
            dry_run: true,
            max_findings: 10,
            findings: vec![finding(
                FindingStatus::Confirmed,
                vec![Evidence {
                    kind: "code".to_string(),
                    file: Some("src/intelligence/agent-core/security_workflow.rs".to_string()),
                    line: Some(1),
                    snippet: Some("SecurityWorkflowReport".to_string()),
                    command: None,
                    output_hash: None,
                    note: Some("DTO assembly evidence".to_string()),
                }],
            )],
        });

        assert!(report.dry_run);
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
        let orchestrator = SecurityWorkflowOrchestrator::new();
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
        let orchestrator = SecurityWorkflowOrchestrator::new();
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
}
