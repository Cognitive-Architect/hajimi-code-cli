//! Build Tools - Week 13: npm_run, cargo_build, make, cmake

use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::{PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};

async fn run_build(program: &str, args: &[&str], cwd: Option<&PathBuf>) -> Result<ToolOutput, ToolError> {
    let which = if cfg!(windows) { "where" } else { "which" };
    let check = Command::new(which).arg(program).output().await
        .map_err(|e| ToolError { message: format!("Failed to check command '{}': {}", program, e), kind: ToolErrorKind::ExecutionFailed })?;
    if !check.status.success() {
        return Err(ToolError { message: format!("Command '{}' not found", program), kind: ToolErrorKind::NotFound });
    }
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(d) = cwd { cmd.current_dir(d); }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| ToolError::new(format!("Spawn failed: {}", e)))?;
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(out) = child.stdout.take() {
        let mut reader = BufReader::new(out).lines();
        while let Ok(Some(line)) = reader.next_line().await { stdout.push_str(&line); stdout.push('\n'); }
    }
    if let Some(err) = child.stderr.take() {
        let mut reader = BufReader::new(err).lines();
        while let Ok(Some(line)) = reader.next_line().await { stderr.push_str(&line); stderr.push('\n'); }
    }
    let status = child.wait().await.map_err(|e| ToolError::new(format!("Wait failed: {}", e)))?;
    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(ToolError { message: format!("{} failed (code {}): {}", program, code, stderr), kind: ToolErrorKind::ExecutionFailed });
    }
    Ok(ToolOutput { stdout, stderr, exit_code: status.code() })
}

pub struct NpmRunTool;
impl Default for NpmRunTool { fn default() -> Self { Self } }
impl NpmRunTool { pub fn new() -> Self { Self } }
#[derive(Deserialize)] struct NpmArgs { script: String, #[serde(default)] cwd: Option<PathBuf> }
#[async_trait] impl Tool for NpmRunTool {
    fn name(&self) -> &str { "npm_run" }
    fn description(&self) -> &str { "Run npm scripts from package.json" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: true, allowed_paths: None } }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: NpmArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Invalid args: {}", e)))?;
        run_build("npm", &["run", &a.script], a.cwd.as_ref()).await
    }
}

pub struct CargoBuildTool;
impl Default for CargoBuildTool { fn default() -> Self { Self } }
impl CargoBuildTool { pub fn new() -> Self { Self } }
#[derive(Deserialize)] struct CargoArgs { #[serde(default)] release: bool, #[serde(default)] target: Option<String>, #[serde(default)] features: Option<String>, #[serde(default)] cwd: Option<PathBuf> }
#[async_trait] impl Tool for CargoBuildTool {
    fn name(&self) -> &str { "cargo_build" }
    fn description(&self) -> &str { "Build Rust project using cargo" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: false, allowed_paths: None } }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: CargoArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Invalid args: {}", e)))?;
        let mut cmd_args = vec!["build"];
        if a.release { cmd_args.push("--release"); }
        if let Some(ref t) = a.target { cmd_args.push("--target"); cmd_args.push(t); }
        if let Some(ref f) = a.features { cmd_args.push("--features"); cmd_args.push(f); }
        run_build("cargo", &cmd_args, a.cwd.as_ref()).await
    }
}

pub struct MakeTool;
impl Default for MakeTool { fn default() -> Self { Self } }
impl MakeTool { pub fn new() -> Self { Self } }
#[derive(Deserialize)] struct MakeArgs { #[serde(default)] target: Option<String>, #[serde(default)] jobs: Option<u32>, #[serde(default)] cwd: Option<PathBuf> }
#[async_trait] impl Tool for MakeTool {
    fn name(&self) -> &str { "make" }
    fn description(&self) -> &str { "Execute make build commands" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: true, allowed_paths: None } }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: MakeArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Invalid args: {}", e)))?;
        let mut cmd_args: Vec<&str> = Vec::new();
        if let Some(j) = a.jobs { cmd_args.extend(["-j", Box::leak(j.to_string().into_boxed_str())]); }
        if let Some(ref t) = a.target { cmd_args.push(t); }
        run_build("make", &cmd_args, a.cwd.as_ref()).await
    }
}

pub struct CmakeTool;
impl Default for CmakeTool { fn default() -> Self { Self } }
impl CmakeTool { pub fn new() -> Self { Self } }
#[derive(Deserialize)] struct CmakeArgs { #[serde(default)] build_dir: Option<PathBuf>, #[serde(default)] source_dir: Option<PathBuf>, #[serde(default)] build: bool, #[serde(default)] cwd: Option<PathBuf> }
#[async_trait] impl Tool for CmakeTool {
    fn name(&self) -> &str { "cmake" }
    fn description(&self) -> &str { "Configure and build with CMake" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: true, allowed_paths: None } }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: CmakeArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Invalid args: {}", e)))?;
        let build_dir = a.build_dir.as_deref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "build".to_string());
        let source = a.source_dir.as_deref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| ".".to_string());
        let bdir: &str = Box::leak(build_dir.into_boxed_str());
        let sdir: &str = Box::leak(source.into_boxed_str());
        run_build("cmake", &["-B", bdir, "-S", sdir], a.cwd.as_ref()).await?;
        if a.build { run_build("cmake", &["--build", bdir], a.cwd.as_ref()).await }
        else { Ok(ToolOutput::success("CMake configuration completed")) }
    }
}
