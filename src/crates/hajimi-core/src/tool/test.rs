//! Test Tools Cluster - B-W13/05-07: run_tests, coverage_report, benchmark

use async_trait::async_trait;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;
use crate::tool::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};

pub struct RunTestsTool;
impl RunTestsTool { pub fn new() -> Self { Self } }
impl Default for RunTestsTool { fn default() -> Self { Self::new() } }

#[derive(Debug, Deserialize)] struct RunTestsArgs { #[serde(default)] path: Option<String>, #[serde(default)] package: Option<String> }

fn parse_test_result(s: &str) -> (u32, u32, u32) {
    let (mut p, mut f, mut i) = (0, 0, 0);
    for l in s.lines() {
        if l.contains("test result:") {
            for part in l.split(';') {
                let pt = part.trim();
                if let Some(n) = pt.split_whitespace().next() {
                    if let Ok(num) = n.parse::<u32>() {
                        if pt.contains("passed") { p += num; } else if pt.contains("failed") { f += num; } else if pt.contains("ignored") { i += num; }
                    }
                }
            }
        }
    }
    (p, f, i)
}

#[async_trait] impl Tool for RunTestsTool {
    fn name(&self) -> &str { "run_tests" }
    fn description(&self) -> &str { "Execute cargo test and parse passed/failed results" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: RunTestsArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let mut cmd = Command::new("cargo"); cmd.arg("test");
        if let Some(pkg) = &a.package { cmd.arg("-p").arg(pkg); }
        if let Some(p) = &a.path { cmd.current_dir(p); }
        cmd.env("RUST_BACKTRACE", "1"); cmd.stdout(Stdio::piped()); cmd.stderr(Stdio::piped());
        let out = cmd.output().await.map_err(|e| ToolError::new(format!("cargo test failed: {}", e)))?;
        let stdout = String::from_utf8_lossy(&out.stdout); let stderr = String::from_utf8_lossy(&out.stderr);
        let (passed, failed, ignored) = parse_test_result(&stdout);
        let summary = format!("test_result: {} passed, {} failed, {} ignored\n", passed, failed, ignored);
        Ok(ToolOutput { stdout: format!("{}{}", summary, stdout), stderr: stderr.to_string(), exit_code: Some(if failed > 0 { 1 } else { out.status.code().unwrap_or(0) }) })
    }
}

pub struct CoverageReportTool;
impl CoverageReportTool { pub fn new() -> Self { Self } }
impl Default for CoverageReportTool { fn default() -> Self { Self::new() } }

#[derive(Debug, Deserialize)] struct CoverageArgs { #[serde(default)] path: Option<String>, #[serde(default)] output: Option<String>, #[serde(default)] tool: Option<String> }

async fn check_cov_tool(t: &str) -> bool {
    match t { "tarpaulin" => Command::new("cargo").args(["tarpaulin", "--version"]).output().await.map(|o| o.status.success()).unwrap_or(false), "llvm-cov" => Command::new("cargo").args(["llvm-cov", "--version"]).output().await.map(|o| o.status.success()).unwrap_or(false), _ => false }
}

#[async_trait] impl Tool for CoverageReportTool {
    fn name(&self) -> &str { "coverage_report" }
    fn description(&self) -> &str { "Generate test coverage report using cargo-tarpaulin or cargo-llvm-cov" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: CoverageArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let tool = a.tool.as_deref().unwrap_or("tarpaulin");
        if !check_cov_tool(tool).await { return Ok(ToolOutput::error(format!("Coverage tool '{}' not installed. Install: cargo install cargo-{}", tool, tool), 127)); }
        let mut cmd = Command::new("cargo");
        match tool {
            "tarpaulin" => { cmd.args(["tarpaulin", "--out", "Html"]); if let Some(o) = &a.output { cmd.arg("--output-dir").arg(o); } }
            "llvm-cov" | "coverage" => { cmd.args(["llvm-cov", "--html"]); if let Some(o) = &a.output { cmd.arg("--output-dir").arg(o); } }
            _ => return Err(ToolError::new(format!("Unknown coverage tool: {}", tool))),
        }
        if let Some(p) = &a.path { cmd.current_dir(p); }
        cmd.stdout(Stdio::piped()); cmd.stderr(Stdio::piped());
        let out = cmd.output().await.map_err(|e| ToolError::new(format!("Coverage failed: {}", e)))?;
        let (stdout, stderr) = (String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        Ok(ToolOutput { stdout: format!("coverage {} using {}\n{}", if out.status.success() { "generated" } else { "failed" }, tool, stdout), stderr: stderr.to_string(), exit_code: Some(out.status.code().unwrap_or(1)) })
    }
}

pub struct BenchmarkTool;
impl BenchmarkTool { pub fn new() -> Self { Self } }
impl Default for BenchmarkTool { fn default() -> Self { Self::new() } }

#[derive(Debug, Deserialize)] struct BenchmarkArgs { #[serde(default)] path: Option<String>, #[serde(default)] filter: Option<String> }

async fn has_bench(p: &Option<String>) -> bool {
    let mut cmd = Command::new("cargo"); cmd.arg("bench").arg("--").arg("--list");
    if let Some(path) = p { cmd.current_dir(path); }
    match cmd.output().await { Ok(o) => o.status.success() && !String::from_utf8_lossy(&o.stdout).is_empty(), Err(_) => false }
}

#[async_trait] impl Tool for BenchmarkTool {
    fn name(&self) -> &str { "benchmark" }
    fn description(&self) -> &str { "Run Criterion benchmark tests" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: BenchmarkArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        if !has_bench(&a.path).await { return Ok(ToolOutput::error("No benchmark targets found. Add [[bench]] section to Cargo.toml", 1)); }
        let mut cmd = Command::new("cargo"); cmd.arg("bench");
        if let Some(f) = &a.filter { cmd.arg("--").arg(f); }
        if let Some(p) = &a.path { cmd.current_dir(p); }
        cmd.stdout(Stdio::piped()); cmd.stderr(Stdio::piped());
        let out = cmd.output().await.map_err(|e| ToolError::new(format!("Benchmark failed: {}", e)))?;
        let (stdout, stderr) = (String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        Ok(ToolOutput { stdout: format!("benchmark {}\n{}", if out.status.success() { "completed" } else { "failed" }, stdout), stderr: stderr.to_string(), exit_code: Some(out.status.code().unwrap_or(0)) })
    }
}

pub use RunTestsTool as TestsTool; pub use CoverageReportTool as CoverageTool;
