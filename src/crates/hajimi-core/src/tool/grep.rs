//! Grep Tool - CORR-W09-03 / DEBT-003: 内容搜索与find集成优化

use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use tokio::fs::read_to_string;
use crate::tool::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};
use super::find::FindTool;

pub struct GrepTool;
impl GrepTool { pub fn new() -> Self { Self } }
impl Default for GrepTool { fn default() -> Self { Self::new() } }

/// Input: file list from find or direct path
#[derive(Debug, Clone)]
pub enum GrepInput { Path(String), FileList(Vec<String>) }
impl GrepInput {
    pub fn from_files(files: Vec<String>) -> Self { Self::FileList(files) }
    pub async fn expand(self) -> Result<Vec<String>, ToolError> {
        match self {
            Self::Path(p) => {
                let path = Path::new(&p);
                if !path.exists() { return Err(ToolError::new(format!("Not found: {}", p))); }
                if path.is_file() { return Ok(vec![p]); }
                let mut f = Vec::new();
                let mut d = tokio::fs::read_dir(path).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
                while let Ok(Some(e)) = d.next_entry().await { if e.path().is_file() { f.push(e.path().to_string_lossy().to_string()); } }
                Ok(f)
            }
            Self::FileList(f) => Ok(f),
        }
    }
}
impl From<Vec<String>> for GrepInput { fn from(f: Vec<String>) -> Self { Self::FileList(f) } }

#[derive(Debug, Deserialize)]
struct GrepArgs { pattern: String, path: String, #[serde(default)] recursive: bool, #[serde(default)] files_from: Option<Vec<String>> }

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str { "Search file contents with regex" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: GrepArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let rx = Regex::new(&a.pattern).map_err(|e| ToolError::new(format!("Regex: {}", e)))?;
        let mut m = Vec::new();
        if let Some(files) = a.files_from {
            for fp in files { if let Ok(c) = read_to_string(&fp).await { for (n, l) in c.lines().enumerate() { if rx.is_match(l) { m.push(format!("{}:{}: {}", fp, n + 1, l)); } } } }
        } else {
            let p = Path::new(&a.path);
            if !p.exists() { return Err(ToolError::new(format!("Not found: {}", a.path))); }
            if p.is_file() {
                let c = read_to_string(p).await.map_err(|e| ToolError::new(format!("Read: {}", e)))?;
                for (n, l) in c.lines().enumerate() { if rx.is_match(l) { m.push(format!("{}: {}", n + 1, l)); } }
            } else if a.recursive && p.is_dir() {
                let mut d = tokio::fs::read_dir(p).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
                while let Ok(Some(e)) = d.next_entry().await {
                    let fp = e.path();
                    if fp.is_file() { if let Ok(c) = read_to_string(&fp).await { for (n, l) in c.lines().enumerate() { if rx.is_match(l) { m.push(format!("{}:{}: {}", fp.display(), n + 1, l)); } } } }
                }
            }
        }
        Ok(ToolOutput { stdout: m.join("\n"), stderr: String::new(), exit_code: Some(if m.is_empty() { 1 } else { 0 }) })
    }
}

/// Find + Grep integration: find . -name "*.rs" | grep "pattern"
pub struct FindGrepIntegration;
impl FindGrepIntegration {
    pub async fn pipe(find: &FindTool, grep: &GrepTool, find_args: serde_json::Value, pattern: &str) -> Result<ToolOutput, ToolError> {
        let out = find.execute(find_args).await?;
        let files: Vec<String> = out.stdout.lines().map(|s: &str| s.to_string()).collect();
        if files.is_empty() { return Ok(ToolOutput { stdout: String::new(), stderr: "No files".to_string(), exit_code: Some(1) }); }
        let rx = Regex::new(pattern).map_err(|e| ToolError::new(format!("Regex: {}", e)))?;
        let mut m = Vec::new();
        for fp in files { if let Ok(c) = read_to_string(&fp).await { for (n, l) in c.lines().enumerate() { if rx.is_match(l) { m.push(format!("{}:{}: {}", fp, n + 1, l)); } } } }
        Ok(ToolOutput { stdout: m.join("\n"), stderr: String::new(), exit_code: Some(if m.is_empty() { 1 } else { 0 }) })
    }
}

pub use super::find::{FindArgs, FindResult};
