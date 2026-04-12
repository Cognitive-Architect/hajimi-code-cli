//! Git tools - B-W11/02+B-W11/03: git_status/git_diff/git_log/git_commit
//! Using git CLI instead of git2 crate for Windows compatibility

use super::{PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;

async fn run_git(args: &[&str], path: &str) -> Result<ToolOutput, ToolError> {
    let out = Command::new("git").args(args).current_dir(path).output().await
        .map_err(|e| ToolError { message: format!("Git failed: {}", e), kind: ToolErrorKind::GitError })?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let kind = if err.contains("user.name") || err.contains("user.email") { ToolErrorKind::GitConfigMissing }
        else if err.contains("not a git repository") { ToolErrorKind::NotARepository }
        else { ToolErrorKind::GitError };
        return Err(ToolError { message: err.to_string(), kind });
    }
    Ok(ToolOutput::success(String::from_utf8_lossy(&out.stdout)))
}

async fn check_cfg(path: &str) -> Result<(), ToolError> {
    for cfg in ["user.name", "user.email"] {
        let out = Command::new("git").args(["config", cfg]).current_dir(path).output().await
            .map_err(|e| ToolError { message: format!("Git: {}", e), kind: ToolErrorKind::GitError })?;
        if !out.status.success() || out.stdout.is_empty() {
            return Err(ToolError { message: format!("Git {} not configured", cfg), kind: ToolErrorKind::GitConfigMissing });
        }
    }
    Ok(())
}

pub struct GitStatusTool;
impl GitStatusTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str { "git_status" }
    fn description(&self) -> &str { "Show working tree status" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        run_git(&["status", "--short"], args.get("path").and_then(Value::as_str).unwrap_or(".")).await
    }
}

pub struct GitDiffTool;
impl GitDiffTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str { "git_diff" }
    fn description(&self) -> &str { "Show unstaged changes as patch" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        run_git(&["diff"], args.get("path").and_then(Value::as_str).unwrap_or(".")).await
    }
}

pub struct GitLogTool;
impl GitLogTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for GitLogTool {
    fn name(&self) -> &str { "git_log" }
    fn description(&self) -> &str { "Show commit history with message/author/date" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20).to_string();
        run_git(&["log", "--pretty=format:%h | %an | %ct | %s", "-n", &limit], path).await
    }
}

pub struct GitCommitTool;
impl GitCommitTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for GitCommitTool {
    fn name(&self) -> &str { "git_commit" }
    fn description(&self) -> &str { "Create commit with signature, supports --amend" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: true, allowed_paths: None }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let msg = args.get("message").and_then(Value::as_str).ok_or_else(|| ToolError::new("Missing message"))?;
        check_cfg(path).await?;
        if args.get("amend").and_then(Value::as_bool).unwrap_or(false) {
            run_git(&["commit", "--amend", "-m", msg], path).await
        } else {
            let st = Command::new("git").args(["status", "--porcelain"]).current_dir(path).output().await
                .map_err(|e| ToolError { message: format!("Git: {}", e), kind: ToolErrorKind::GitError })?;
            if st.stdout.is_empty() { return Err(ToolError { message: "No changes".to_string(), kind: ToolErrorKind::NoChangesToCommit }); }
            run_git(&["add", "-A"], path).await?;
            run_git(&["commit", "-m", msg], path).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_git_status_name() { assert_eq!(GitStatusTool::new().name(), "git_status"); }
    #[test] fn test_git_diff_name() { assert_eq!(GitDiffTool::new().name(), "git_diff"); }
    #[test] fn test_git_log_name() { assert_eq!(GitLogTool::new().name(), "git_log"); }
    #[test] fn test_git_commit_name() { assert_eq!(GitCommitTool::new().name(), "git_commit"); }
}
