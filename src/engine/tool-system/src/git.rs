//! Git tools - B-W11/02+B-W11/03: git_status/git_diff/git_log/git_commit
//! Using git CLI instead of git2 crate for Windows compatibility

use super::{
    PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions,
};
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;

async fn run_git(args: &[&str], path: &str) -> Result<ToolOutput, ToolError> {
    let out = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .await
        .map_err(|e| ToolError {
            message: format!("Git failed: {}", e),
            kind: ToolErrorKind::GitError,
        })?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let kind = if err.contains("user.name") || err.contains("user.email") {
            ToolErrorKind::GitConfigMissing
        } else if err.contains("not a git repository") {
            ToolErrorKind::NotARepository
        } else {
            ToolErrorKind::GitError
        };
        return Err(ToolError {
            message: err.to_string(),
            kind,
        });
    }
    Ok(ToolOutput::success(String::from_utf8_lossy(&out.stdout)))
}

async fn check_cfg(path: &str) -> Result<(), ToolError> {
    for cfg in ["user.name", "user.email"] {
        let out = Command::new("git")
            .args(["config", cfg])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| ToolError {
                message: format!("Git: {}", e),
                kind: ToolErrorKind::GitError,
            })?;
        if !out.status.success() || out.stdout.is_empty() {
            return Err(ToolError {
                message: format!("Git {} not configured", cfg),
                kind: ToolErrorKind::GitConfigMissing,
            });
        }
    }
    Ok(())
}

pub struct GitStatusTool;
impl Default for GitStatusTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitStatusTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }
    fn description(&self) -> &str {
        "Show working tree status"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        run_git(
            &["status", "--short"],
            args.get("path").and_then(Value::as_str).unwrap_or("."),
        )
        .await
    }
}

pub struct GitDiffTool;
impl Default for GitDiffTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitDiffTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }
    fn description(&self) -> &str {
        "Show unstaged changes as patch"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        run_git(
            &["diff"],
            args.get("path").and_then(Value::as_str).unwrap_or("."),
        )
        .await
    }
}

pub struct GitLogTool;
impl Default for GitLogTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLogTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitLogTool {
    fn name(&self) -> &str {
        "git_log"
    }
    fn description(&self) -> &str {
        "Show commit history with message/author/date"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(20)
            .to_string();
        run_git(
            &["log", "--pretty=format:%h | %an | %ct | %s", "-n", &limit],
            path,
        )
        .await
    }
}

pub struct GitCommitTool;
impl Default for GitCommitTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GitCommitTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitCommitTool {
    fn name(&self) -> &str {
        "git_commit"
    }
    fn description(&self) -> &str {
        "Create commit with signature, supports --amend"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let msg = args
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing message"))?;
        check_cfg(path).await?;
        if args.get("amend").and_then(Value::as_bool).unwrap_or(false) {
            run_git(&["commit", "--amend", "-m", msg], path).await
        } else {
            let st = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(path)
                .output()
                .await
                .map_err(|e| ToolError {
                    message: format!("Git: {}", e),
                    kind: ToolErrorKind::GitError,
                })?;
            if st.stdout.is_empty() {
                return Err(ToolError {
                    message: "No changes".to_string(),
                    kind: ToolErrorKind::NoChangesToCommit,
                });
            }
            run_git(&["add", "-A"], path).await?;
            run_git(&["commit", "-m", msg], path).await
        }
    }
}

// Phase 4 Day 4: SmartCommitTool — generate commit message from diff heuristics
pub struct SmartCommitTool;
impl Default for SmartCommitTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartCommitTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SmartCommitTool {
    fn name(&self) -> &str {
        "smart_commit"
    }
    fn description(&self) -> &str {
        "Generate a conventional commit message from staged/unstaged changes"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let user_msg = args.get("message").and_then(Value::as_str);
        check_cfg(path).await?;
        // Try staged diff first, fall back to unstaged
        let diff_out = Command::new("git")
            .args(["diff", "--cached", "--stat"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| ToolError {
                message: format!("Git: {}", e),
                kind: ToolErrorKind::GitError,
            })?;
        let has_staged = diff_out.status.success() && !diff_out.stdout.is_empty();
        let diff_stat = if has_staged {
            String::from_utf8_lossy(&diff_out.stdout).to_string()
        } else {
            let out = Command::new("git")
                .args(["diff", "--stat"])
                .current_dir(path)
                .output()
                .await
                .map_err(|e| ToolError {
                    message: format!("Git: {}", e),
                    kind: ToolErrorKind::GitError,
                })?;
            String::from_utf8_lossy(&out.stdout).to_string()
        };
        if diff_stat.trim().is_empty() {
            return Err(ToolError {
                message: "No changes to commit".to_string(),
                kind: ToolErrorKind::NoChangesToCommit,
            });
        }
        let msg = if let Some(m) = user_msg {
            m.to_string()
        } else {
            generate_smart_message(&diff_stat)
        };
        run_git(&["add", "-A"], path).await?;
        run_git(&["commit", "-m", &msg], path).await
    }
}

fn generate_smart_message(diff_stat: &str) -> String {
    let lines: Vec<&str> = diff_stat.lines().collect();
    let mut files_changed = 0usize;
    let mut insertions = 0usize;
    let mut deletions = 0usize;
    for line in &lines {
        if line.contains("|") {
            files_changed += 1;
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 2 {
                let stat = parts[1].trim();
                for num in stat.split_whitespace() {
                    if num.ends_with("+") {
                        insertions += num.trim_end_matches('+').parse::<usize>().unwrap_or(0);
                    } else if num.ends_with("-") {
                        deletions += num.trim_end_matches('-').parse::<usize>().unwrap_or(0);
                    }
                }
            }
        }
    }
    // Heuristic: determine conventional commit type from file paths / changes
    let mut feat = 0usize;
    let mut fix = 0usize;
    let mut docs = 0usize;
    let mut test = 0usize;
    let mut refactor = 0usize;
    for line in &lines {
        let lower = line.to_lowercase();
        if lower.contains("test") || lower.contains("spec") {
            test += 1;
        } else if lower.contains("doc") || lower.contains("readme") || lower.contains(".md") {
            docs += 1;
        } else if lower.contains("fix") || lower.contains("bug") || lower.contains("patch") {
            fix += 1;
        } else if lower.contains("refactor")
            || lower.contains("rename")
            || lower.contains("extract")
        {
            refactor += 1;
        } else if lower.contains("feat") || lower.contains("add") || lower.contains("new") {
            feat += 1;
        }
    }
    let total_kw = feat + fix + docs + test + refactor;
    let ctype = if total_kw == 0 {
        "feat"
    } else if test >= docs && test >= fix && test >= refactor && test >= feat {
        "test"
    } else if docs >= fix && docs >= refactor && docs >= feat {
        "docs"
    } else if fix >= refactor && fix >= feat {
        "fix"
    } else if refactor >= feat {
        "refactor"
    } else {
        "feat"
    };
    let scope = if files_changed == 1 {
        lines
            .iter()
            .find(|l| l.contains("|"))
            .and_then(|l| l.split('|').next())
            .map(|s| s.trim().to_string())
    } else {
        None
    };
    let body = match scope {
        Some(s) => format!(
            "{}({}): update {} file{} (+{} -{})",
            ctype,
            s,
            files_changed,
            if files_changed > 1 { "s" } else { "" },
            insertions,
            deletions
        ),
        None => format!(
            "{}: update {} file{} (+{} -{})",
            ctype,
            files_changed,
            if files_changed > 1 { "s" } else { "" },
            insertions,
            deletions
        ),
    };
    body
}

// Phase 4 Day 4: GeneratePrDescriptionTool — markdown PR description from commits
pub struct GeneratePrDescriptionTool;
impl Default for GeneratePrDescriptionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneratePrDescriptionTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GeneratePrDescriptionTool {
    fn name(&self) -> &str {
        "generate_pr_description"
    }
    fn description(&self) -> &str {
        "Generate a markdown PR description from branch commits and diff stats"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let base = args
            .get("base")
            .and_then(Value::as_str)
            .unwrap_or("origin/main");
        // Get commits since base
        let log_out = Command::new("git")
            .args([
                "log",
                &format!("{}..HEAD", base),
                "--oneline",
                "--no-decorate",
            ])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| ToolError {
                message: format!("Git: {}", e),
                kind: ToolErrorKind::GitError,
            })?;
        if !log_out.status.success() {
            return Err(ToolError {
                message: String::from_utf8_lossy(&log_out.stderr).to_string(),
                kind: ToolErrorKind::GitError,
            });
        }
        let commits = String::from_utf8_lossy(&log_out.stdout);
        if commits.trim().is_empty() {
            return Ok(ToolOutput::success(
                "No commits ahead of base branch.".to_string(),
            ));
        }
        // Get diff stats
        let stat_out = Command::new("git")
            .args(["diff", &format!("{}...HEAD", base), "--stat"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| ToolError {
                message: format!("Git: {}", e),
                kind: ToolErrorKind::GitError,
            })?;
        let stats = if stat_out.status.success() {
            String::from_utf8_lossy(&stat_out.stdout).to_string()
        } else {
            String::new()
        };
        let mut md = String::from("## Summary\n\n");
        md.push_str("### Commits\n\n");
        for line in commits.lines() {
            md.push_str(&format!("- {}\n", line));
        }
        md.push_str("\n### Changes\n\n```\n");
        md.push_str(&stats);
        md.push_str("\n```\n\n");
        md.push_str(
            "### Checklist\n\n- [ ] Tests pass\n- [ ] Code reviewed\n- [ ] Documentation updated\n",
        );
        Ok(ToolOutput::success(md))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_git_status_name() {
        assert_eq!(GitStatusTool::new().name(), "git_status");
    }
    #[test]
    fn test_git_diff_name() {
        assert_eq!(GitDiffTool::new().name(), "git_diff");
    }
    #[test]
    fn test_git_log_name() {
        assert_eq!(GitLogTool::new().name(), "git_log");
    }
    #[test]
    fn test_git_commit_name() {
        assert_eq!(GitCommitTool::new().name(), "git_commit");
    }
    #[test]
    fn test_smart_commit_name() {
        assert_eq!(SmartCommitTool::new().name(), "smart_commit");
    }
    #[test]
    fn test_generate_pr_name() {
        assert_eq!(
            GeneratePrDescriptionTool::new().name(),
            "generate_pr_description"
        );
    }
    #[test]
    fn test_smart_message_parsing() {
        let diff = "src/main.rs | 10 ++++++\n src/lib.rs  | 5  +-\n 2 files changed, 12 insertions(+), 3 deletions(-)";
        let msg = generate_smart_message(diff);
        assert!(msg.starts_with("feat:"));
        assert!(msg.contains("2 files"));
    }
    #[test]
    fn test_smart_message_with_scope() {
        let diff = "src/main.rs | 10 ++++++\n 1 file changed, 10 insertions(+)";
        let msg = generate_smart_message(diff);
        assert!(msg.starts_with("feat(src/main.rs):"));
    }
}
