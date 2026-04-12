//! Security Audit Tool - B-04/06
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::tool::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions, ToolErrorKind};

pub struct SecurityAuditTool;
impl SecurityAuditTool { pub fn new() -> Self { Self } }
impl Default for SecurityAuditTool { fn default() -> Self { Self::new() } }

#[derive(Debug, Deserialize)]
struct Args { path: String, #[serde(default)] ignore_file: Option<String> }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum Severity { High, Medium, Low }

#[derive(Debug, Clone, Serialize)]
struct Finding { severity: Severity, #[serde(rename = "type")] type_: String, file: String, line: u32, snippet: String }

#[derive(Debug, Clone, Serialize)]
struct Summary { high: u32, medium: u32, low: u32 }

#[derive(Debug, Clone, Serialize)]
struct AuditResult { findings: Vec<Finding>, summary: Summary }

struct Scanner { patterns: Vec<(Regex, String, Severity)>, ignore: Vec<Regex> }

fn cre(s: &str) -> std::result::Result<Regex, ToolError> {
    Regex::new(s).map_err(|e| ToolError { message: format!("Regex: {}", e), kind: ToolErrorKind::ExecutionFailed })
}

impl Scanner {
    fn new(ig: Vec<String>) -> std::result::Result<Self, ToolError> {
        let p = vec![(cre(r"AKIA[0-9A-Z]{16}")?, "AWSKey".into(), Severity::High), (cre(r"AWS_ACCESS_KEY_ID\s*[=:]\s*[A-Z0-9]{20}")?, "AWSKey".into(), Severity::High), (cre(r"ghp_[a-zA-Z0-9]{36}")?, "GitHubToken".into(), Severity::High), (cre(r"sk_live_[a-zA-Z0-9]{24,}")?, "StripeKey".into(), Severity::High), (cre(r"BEGIN\s+(RSA|DSA|EC|OPENSSH)?\s*PRIVATE\s+KEY")?, "PrivateKey".into(), Severity::High), (cre(r"todo!\s*\(")?, "TodoMacro".into(), Severity::Medium), (cre(r"\.unwrap\s*\(")?, "Unwrap".into(), Severity::Medium), (cre(r"panic!\s*\(")?, "Panic".into(), Severity::Medium)];
        let mut ignore = Vec::new();
        for pat in ig { if let Ok(rx) = Regex::new(&pat) { ignore.push(rx); } }
        Ok(Self { patterns: p, ignore })
    }
    fn skip(&self, p: &Path) -> bool { let s = p.to_string_lossy(); self.ignore.iter().any(|r| r.is_match(&s)) }
    async fn scan(&self, p: &Path) -> std::result::Result<Vec<Finding>, ToolError> {
        let mut out = Vec::new(); if self.skip(p) { return Ok(out); }
        let file = tokio::fs::File::open(p).await.map_err(|e| ToolError { message: format!("Open: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        let mut lines = BufReader::new(file).lines(); let mut n: u32 = 0; let test = p.to_string_lossy().contains("test");
        while let Ok(Some(l)) = lines.next_line().await {
            n += 1;
            for (rx, t, s) in &self.patterns {
                if let Some(m) = rx.find(&l) {
                    let sev = if test && matches!(s, Severity::High) { Severity::Low } else { s.clone() };
                    let snip = if m.len() > 50 { format!("{}...", &l[m.start()..m.start()+47]) } else { m.as_str().into() };
                    out.push(Finding { severity: sev, type_: t.clone(), file: p.to_string_lossy().into(), line: n, snippet: snip });
                }
            }
        }
        Ok(out)
    }
    async fn dir(&self, d: &Path) -> std::result::Result<Vec<Finding>, ToolError> {
        let mut out = Vec::new();
        let mut e = tokio::fs::read_dir(d).await.map_err(|e| ToolError { message: format!("Dir: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        while let Ok(Some(e)) = e.next_entry().await {
            let p = e.path(); if self.skip(&p) { continue; }
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if p.is_file() && matches!(ext, "rs" | "toml" | "lock" | "env" | "yaml" | "yml" | "json" | "pem" | "key") {
                if let Ok(mut f) = self.scan(&p).await { out.append(&mut f); }
            } else if p.is_dir() { if let Ok(mut f) = Box::pin(self.dir(&p)).await { out.append(&mut f); } }
        }
        Ok(out)
    }
}

#[async_trait]
impl Tool for SecurityAuditTool {
    fn name(&self) -> &str { "security_audit" }
    fn description(&self) -> &str { "Scan code for security issues: secrets, keys, unsafe patterns" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }
    async fn execute(&self, args: ToolArgs) -> std::result::Result<ToolOutput, ToolError> {
        let a: Args = serde_json::from_value(args).map_err(|e| ToolError { message: format!("Args: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        let p = PathBuf::from(&a.path); if !p.exists() { return Err(ToolError { message: format!("Not found: {}", a.path), kind: ToolErrorKind::NotFound }); }
        let mut ig = vec![r"\.git/".into(), r"target/".into(), r"node_modules/".into()];
        if let Some(f) = a.ignore_file { if let Ok(c) = tokio::fs::read_to_string(&f).await { for l in c.lines() { let l = l.trim(); if !l.is_empty() && !l.starts_with('#') { ig.push(l.into()); } } } }
        if p.join(".securityignore").exists() { if let Ok(c) = tokio::fs::read_to_string(p.join(".securityignore")).await { for l in c.lines() { let l = l.trim(); if !l.is_empty() && !l.starts_with('#') { ig.push(l.into()); } } } }
        let s = Scanner::new(ig)?; let f = if p.is_file() { s.scan(&p).await? } else { s.dir(&p).await? }; let mut sum = Summary { high: 0, medium: 0, low: 0 };
        for x in &f { match x.severity { Severity::High => sum.high += 1, Severity::Medium => sum.medium += 1, Severity::Low => sum.low += 1 } }
        let json = serde_json::to_string_pretty(&AuditResult { findings: f, summary: sum }).map_err(|e| ToolError { message: format!("JSON: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        Ok(ToolOutput { stdout: json, stderr: String::new(), exit_code: Some(0) })
    }
}

#[cfg(test)]
mod tests {
    use super::*; use tokio::fs::{write, create_dir_all};
    #[tokio::test]
    async fn test_aws() -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st1"); let _ = create_dir_all(&d).await; let f = d.join("c.rs");
        write(&f, "const K:&str=\"AKIAIOSFODNN7EXAMPLE\";").await?;
        let r = Scanner::new(vec![])?.scan(&f).await?;
        assert!(r.iter().any(|x| x.type_ == "AWSKey" && matches!(x.severity, Severity::High)));
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
    #[tokio::test]
    async fn test_pat() -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st2"); let _ = create_dir_all(&d).await; let f = d.join("m.rs");
        write(&f, "fn main(){let x=v.unwrap();todo!();panic!(\"e\");}").await?;
        let r = Scanner::new(vec![])?.scan(&f).await?;
        assert!(r.iter().any(|x| x.type_ == "Unwrap"));
        assert!(r.iter().any(|x| x.type_ == "TodoMacro"));
        assert!(r.iter().any(|x| x.type_ == "Panic"));
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
    #[tokio::test]
    async fn test_ig() -> Result<(), Box<dyn std::error::Error>> {
        let d = std::env::temp_dir().join("st3"); let _ = create_dir_all(&d).await; let f = d.join("x").join("a.rs");
        let _ = create_dir_all(f.parent().ok_or("Invalid path")?).await; write(&f, "const K:&str=\"AKIAEXAMPLE\";").await?;
        assert!(Scanner::new(vec![r"x[/\\]".into()])?.skip(&f));
        let _ = tokio::fs::remove_dir_all(&d).await;
        Ok(())
    }
}
