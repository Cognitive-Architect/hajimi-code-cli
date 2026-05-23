//! Security Audit Tool - B-04/06
use crate::{
    Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions,
};
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct SecurityAuditTool;
impl SecurityAuditTool {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SecurityAuditTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct Args {
    path: String,
    #[serde(default)]
    ignore_file: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
enum FindingStatus {
    Candidate,
    Unverified,
    Confirmed,
    Fixed,
    AcceptedRisk,
    FalsePositive,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
enum Category {
    Secret,
    PanicSafety,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
struct Evidence {
    kind: String,
    file: Option<String>,
    line: Option<u32>,
    snippet: Option<String>,
    command: Option<String>,
    output_hash: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationReceipt {
    command: String,
    exit_code: Option<i32>,
    stdout_summary: String,
    stderr_summary: String,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
struct Finding {
    finding_id: String,
    rule_id: String,
    title: String,
    severity: Severity,
    category: Category,
    status: FindingStatus,
    confidence: f32,
    #[serde(rename = "type")]
    type_: String,
    file: String,
    line: u32,
    snippet: String,
    evidence: Vec<Evidence>,
    recommendation: String,
    regression_test: Option<String>,
    human_review_required: bool,
    validation_receipts: Vec<ValidationReceipt>,
    residual_risk: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Summary {
    critical: u32,
    high: u32,
    medium: u32,
    low: u32,
}

#[derive(Debug, Clone, Serialize)]
struct AuditResult {
    findings: Vec<Finding>,
    summary: Summary,
}

struct PatternRule {
    regex: Regex,
    type_: String,
    severity: Severity,
    rule_id: String,
    category: Category,
    title: String,
    recommendation: String,
    regression_test: Option<String>,
    confidence: f32,
}

struct Scanner {
    patterns: Vec<PatternRule>,
    ignore: Vec<Regex>,
}

fn cre(s: &str) -> std::result::Result<Regex, ToolError> {
    Regex::new(s).map_err(|e| ToolError {
        message: format!("Regex: {}", e),
        kind: ToolErrorKind::ExecutionFailed,
    })
}

impl Scanner {
    fn new(ig: Vec<String>) -> std::result::Result<Self, ToolError> {
        let p = vec![
            secret_rule(
                cre(r"AKIA[0-9A-Z]{16}")?,
                "SECRET-AWS-001",
                "AWSKey",
                "AWS access key pattern detected",
            ),
            secret_rule(
                cre(r"AWS_ACCESS_KEY_ID\s*[=:]\s*[A-Z0-9]{20}")?,
                "SECRET-AWS-001",
                "AWSKey",
                "AWS access key environment assignment detected",
            ),
            secret_rule(
                cre(r"ghp_[a-zA-Z0-9]{36}")?,
                "SECRET-GITHUB-001",
                "GitHubToken",
                "GitHub personal access token pattern detected",
            ),
            secret_rule(
                cre(r"sk_live_[a-zA-Z0-9]{24,}")?,
                "SECRET-STRIPE-001",
                "StripeKey",
                "Stripe live secret key pattern detected",
            ),
            secret_rule(
                cre(r"BEGIN\s+(RSA|DSA|EC|OPENSSH)?\s*PRIVATE\s+KEY")?,
                "SECRET-PRIVATE-KEY-001",
                "PrivateKey",
                "Private key block detected",
            ),
            panic_rule(
                cre(r"todo!\s*\(")?,
                "PANIC-TODO-001",
                "TodoMacro",
                "todo! macro can panic at runtime",
                "Replace todo!() with implemented logic or a recoverable error path.",
            ),
            panic_rule(
                cre(r"\.unwrap\s*\(")?,
                "PANIC-UNWRAP-001",
                "Unwrap",
                "unwrap() can panic at runtime",
                "Handle the Result/Option explicitly and return a recoverable error.",
            ),
            panic_rule(
                cre(r"panic!\s*\(")?,
                "PANIC-MACRO-001",
                "Panic",
                "panic! macro can terminate the workflow",
                "Return a typed error instead of panicking in production paths.",
            ),
        ];
        let mut ignore = Vec::new();
        for pat in ig {
            if let Ok(rx) = Regex::new(&pat) {
                ignore.push(rx);
            }
        }
        Ok(Self {
            patterns: p,
            ignore,
        })
    }
    fn skip(&self, p: &Path) -> bool {
        let s = p.to_string_lossy();
        self.ignore.iter().any(|r| r.is_match(&s))
    }
    fn build_finding(&self, rule: &PatternRule, p: &Path, line: u32, snippet: String) -> Finding {
        let file = p.to_string_lossy().into_owned();
        let confidence = clamp_confidence(rule.confidence);
        Finding {
            finding_id: format!("{}:{}:{}", rule.rule_id, file, line),
            rule_id: rule.rule_id.clone(),
            title: rule.title.clone(),
            severity: rule.severity.clone(),
            category: rule.category.clone(),
            status: FindingStatus::Unverified,
            confidence,
            type_: rule.type_.clone(),
            file: file.clone(),
            line,
            snippet: snippet.clone(),
            evidence: vec![Evidence {
                kind: "code".into(),
                file: Some(file),
                line: Some(line),
                snippet: Some(snippet),
                command: None,
                output_hash: None,
                note: Some("SecurityAuditTool static pattern match".into()),
            }],
            recommendation: rule.recommendation.clone(),
            regression_test: rule.regression_test.clone(),
            human_review_required: matches!(rule.severity, Severity::Critical | Severity::High),
            validation_receipts: Vec::new(),
            residual_risk: Vec::new(),
        }
    }
    async fn scan(&self, p: &Path) -> std::result::Result<Vec<Finding>, ToolError> {
        let mut out = Vec::new();
        if self.skip(p) {
            return Ok(out);
        }
        let file = tokio::fs::File::open(p).await.map_err(|e| ToolError {
            message: format!("Open: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        let mut lines = BufReader::new(file).lines();
        let mut n: u32 = 0;
        let test = p.to_string_lossy().contains("test");
        while let Ok(Some(l)) = lines.next_line().await {
            n += 1;
            for rule in &self.patterns {
                if let Some(m) = rule.regex.find(&l) {
                    let mut finding =
                        self.build_finding(rule, p, n, redact_snippet(&l, m.start(), m.end()));
                    finding.severity = if test && matches!(finding.severity, Severity::High) {
                        Severity::Low
                    } else {
                        finding.severity
                    };
                    finding.human_review_required =
                        matches!(finding.severity, Severity::Critical | Severity::High);
                    out.push(finding);
                }
            }
        }
        Ok(out)
    }
    async fn dir(&self, d: &Path) -> std::result::Result<Vec<Finding>, ToolError> {
        let mut out = Vec::new();
        let mut e = tokio::fs::read_dir(d).await.map_err(|e| ToolError {
            message: format!("Dir: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        while let Ok(Some(e)) = e.next_entry().await {
            let p = e.path();
            if self.skip(&p) {
                continue;
            }
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if p.is_file()
                && matches!(
                    ext,
                    "rs" | "toml" | "lock" | "env" | "yaml" | "yml" | "json" | "pem" | "key"
                )
            {
                if let Ok(mut f) = self.scan(&p).await {
                    out.append(&mut f);
                }
            } else if p.is_dir() {
                if let Ok(mut f) = Box::pin(self.dir(&p)).await {
                    out.append(&mut f);
                }
            }
        }
        Ok(out)
    }
}

fn secret_rule(regex: Regex, rule_id: &str, type_: &str, title: &str) -> PatternRule {
    PatternRule {
        regex,
        type_: type_.into(),
        severity: Severity::High,
        rule_id: rule_id.into(),
        category: Category::Secret,
        title: title.into(),
        recommendation: "Remove the secret from source, rotate the credential, and load it from a secure secret store.".into(),
        regression_test: Some("cargo test -p engine-tool-system security".into()),
        confidence: 0.85,
    }
}

fn panic_rule(
    regex: Regex,
    rule_id: &str,
    type_: &str,
    title: &str,
    recommendation: &str,
) -> PatternRule {
    PatternRule {
        regex,
        type_: type_.into(),
        severity: Severity::Medium,
        rule_id: rule_id.into(),
        category: Category::PanicSafety,
        title: title.into(),
        recommendation: recommendation.into(),
        regression_test: Some("cargo test -p engine-tool-system security".into()),
        confidence: 0.65,
    }
}

fn clamp_confidence(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn redact_snippet(line: &str, start: usize, end: usize) -> String {
    let matched = &line[start..end];
    if matched.len() > 12 && looks_like_secret(matched) {
        format!("{}...", &matched[..8])
    } else if matched.len() > 50 {
        format!("{}...", &matched[..47])
    } else {
        matched.into()
    }
}

fn looks_like_secret(value: &str) -> bool {
    value.starts_with("AKIA")
        || value.starts_with("ghp_")
        || value.starts_with("sk_live_")
        || value.contains("PRIVATE KEY")
}

#[async_trait]
impl Tool for SecurityAuditTool {
    fn name(&self) -> &str {
        "security_audit"
    }
    fn description(&self) -> &str {
        "Scan code for security issues: secrets, keys, unsafe patterns"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    fn is_enabled(&self, _config: &Config) -> bool {
        true
    }
    async fn execute(&self, args: ToolArgs) -> std::result::Result<ToolOutput, ToolError> {
        let a: Args = serde_json::from_value(args).map_err(|e| ToolError {
            message: format!("Args: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        let p = PathBuf::from(&a.path);
        if !p.exists() {
            return Err(ToolError {
                message: format!("Not found: {}", a.path),
                kind: ToolErrorKind::NotFound,
            });
        }
        let mut ig = vec![r"\.git/".into(), r"target/".into(), r"node_modules/".into()];
        if let Some(f) = a.ignore_file {
            if let Ok(c) = tokio::fs::read_to_string(&f).await {
                for l in c.lines() {
                    let l = l.trim();
                    if !l.is_empty() && !l.starts_with('#') {
                        ig.push(l.into());
                    }
                }
            }
        }
        if p.join(".securityignore").exists() {
            if let Ok(c) = tokio::fs::read_to_string(p.join(".securityignore")).await {
                for l in c.lines() {
                    let l = l.trim();
                    if !l.is_empty() && !l.starts_with('#') {
                        ig.push(l.into());
                    }
                }
            }
        }
        let s = Scanner::new(ig)?;
        let f = if p.is_file() {
            s.scan(&p).await?
        } else {
            s.dir(&p).await?
        };
        let mut sum = Summary {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        };
        for x in &f {
            match x.severity {
                Severity::Critical => sum.critical += 1,
                Severity::High => sum.high += 1,
                Severity::Medium => sum.medium += 1,
                Severity::Low => sum.low += 1,
            }
        }
        let json = serde_json::to_string_pretty(&AuditResult {
            findings: f,
            summary: sum,
        })
        .map_err(|e| ToolError {
            message: format!("JSON: {}", e),
            kind: ToolErrorKind::ExecutionFailed,
        })?;
        Ok(ToolOutput {
            stdout: json,
            stderr: String::new(),
            exit_code: Some(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::{create_dir_all, write};
    #[tokio::test]
    async fn test_aws() -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st1");
        let _ = create_dir_all(&d).await;
        let f = d.join("c.rs");
        write(&f, "const K:&str=\"AKIAIOSFODNN7EXAMPLE\";").await?;
        let r = Scanner::new(vec![])?.scan(&f).await?;
        let finding = r
            .iter()
            .find(|x| x.type_ == "AWSKey")
            .ok_or("missing AWSKey")?;
        assert_eq!(finding.rule_id, "SECRET-AWS-001");
        assert!(matches!(finding.severity, Severity::High));
        assert!(matches!(finding.category, Category::Secret));
        assert!(matches!(finding.status, FindingStatus::Unverified));
        assert_eq!(finding.evidence.len(), 1);
        assert!(finding.recommendation.contains("rotate"));
        assert!(finding.regression_test.is_some());
        assert!((0.0..=1.0).contains(&finding.confidence));
        assert!(finding.human_review_required);
        assert!(finding.snippet.starts_with("AKIAIOSF"));
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
    #[tokio::test]
    async fn test_pat() -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st2");
        let _ = create_dir_all(&d).await;
        let f = d.join("m.rs");
        write(&f, "fn main(){let x=v.unwrap();todo!();panic!(\"e\");}").await?;
        let r = Scanner::new(vec![])?.scan(&f).await?;
        for type_ in ["Unwrap", "TodoMacro", "Panic"] {
            let finding = r
                .iter()
                .find(|x| x.type_ == type_)
                .ok_or("missing panic finding")?;
            assert!(matches!(finding.category, Category::PanicSafety));
            assert!(finding.rule_id.starts_with("PANIC-"));
            assert!(finding.recommendation.len() > 20);
            assert!(finding.regression_test.is_some());
            assert!((0.0..=1.0).contains(&finding.confidence));
            assert!(!finding.human_review_required);
        }
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
    #[tokio::test]
    async fn test_security_finding_schema_serializes_old_and_new_fields(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st4");
        let _ = create_dir_all(&d).await;
        let f = d.join("schema.rs");
        write(&f, "fn main(){panic!(\"schema\");}").await?;
        let r = Scanner::new(vec![])?.scan(&f).await?;
        let json = serde_json::to_value(&r[0])?;
        assert!(json.get("severity").is_some());
        assert!(json.get("type").is_some());
        assert!(json.get("file").is_some());
        assert!(json.get("line").is_some());
        assert!(json.get("snippet").is_some());
        assert!(json.get("rule_id").is_some());
        assert!(json.get("category").is_some());
        assert!(json.get("status").is_some());
        assert!(json.get("evidence").is_some());
        assert!(json.get("recommendation").is_some());
        assert!(json.get("regression_test").is_some());
        assert!(json.get("confidence").is_some());
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
    #[test]
    fn test_security_confidence_is_clamped() {
        assert_eq!(clamp_confidence(-0.5), 0.0);
        assert_eq!(clamp_confidence(1.5), 1.0);
        assert_eq!(clamp_confidence(0.7), 0.7);
    }
    #[tokio::test]
    async fn test_ig() -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st3");
        let _ = create_dir_all(&d).await;
        let f = d.join("x").join("a.rs");
        let _ = create_dir_all(f.parent().ok_or("Invalid path")?).await;
        write(&f, "const K:&str=\"AKIAEXAMPLE\";").await?;
        assert!(Scanner::new(vec![r"x[/\\]".into()])?.skip(&f));
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
}
