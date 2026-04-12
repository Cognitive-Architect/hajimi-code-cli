//! Git Branch Tool - B-W11/04
use super::{PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};
use async_trait::async_trait;
use git2::{Repository, StatusOptions};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchAction { List, Create { name: String }, Delete { name: String }, Switch { name: String } }

#[derive(Debug, Clone, Deserialize)]
pub struct BranchArgs { pub path: Option<PathBuf>, pub action: BranchAction }
pub struct GitBranchTool;

impl GitBranchTool {
    pub fn new() -> Self { Self }
    fn open_repo(path: &PathBuf) -> Result<Repository, ToolError> {
        Repository::open(path).map_err(|e| ToolError { message: format!("Open: {}", e), kind: ToolErrorKind::GitError })
    }
    fn has_uncommitted(repo: &Repository) -> Result<bool, ToolError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts)).map_err(|e| ToolError { message: format!("Status: {}", e), kind: ToolErrorKind::GitError })?;
        Ok(!statuses.is_empty())
    }
    fn current_branch(repo: &Repository) -> Result<String, ToolError> {
        repo.head().ok().and_then(|h| h.shorthand().map(|s| s.to_string()))
            .ok_or_else(|| ToolError { message: "No HEAD".into(), kind: ToolErrorKind::GitError })
    }
}

#[async_trait]
impl Tool for GitBranchTool {
    fn name(&self) -> &str { "git_branch" }
    fn description(&self) -> &str { "Git branch CRUD operations" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Ask, requires_confirmation: true, allowed_paths: None }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: BranchArgs = serde_json::from_value(args).map_err(|e| ToolError { message: format!("Args: {}", e), kind: ToolErrorKind::InvalidArgs })?;
        let path = args.path.unwrap_or_else(|| PathBuf::from("."));
        let repo = Self::open_repo(&path)?;
        match args.action {
            BranchAction::List => {
                let branches: Vec<String> = repo.branches(None).map_err(|e| ToolError { message: format!("List: {}", e), kind: ToolErrorKind::GitError })?
                    .filter_map(|b| b.ok().and_then(|(br, _)| br.name().ok().flatten().map(|s| s.to_string()))).collect();
                Ok(ToolOutput::success(serde_json::to_string(&branches).unwrap_or_default()))
            }
            BranchAction::Create { name } => {
                let commit = repo.head().map_err(|e| ToolError { message: format!("HEAD: {}", e), kind: ToolErrorKind::GitError })?.peel_to_commit()
                    .map_err(|e| ToolError { message: format!("Commit: {}", e), kind: ToolErrorKind::GitError })?;
                repo.branch(&name, &commit, false).map_err(|e| ToolError { message: format!("Create: {}", e), kind: ToolErrorKind::GitError })?;
                Ok(ToolOutput::success(format!("Created: {}", name)))
            }
            BranchAction::Delete { name } => {
                if name == Self::current_branch(&repo)? {
                    return Err(ToolError { message: "Cannot delete current".into(), kind: ToolErrorKind::CannotDeleteCurrentBranch });
                }
                let mut branch = repo.find_branch(&name, git2::BranchType::Local).map_err(|e| ToolError { message: format!("Find: {}", e), kind: ToolErrorKind::GitError })?;
                branch.delete().map_err(|e| ToolError { message: format!("Delete: {}", e), kind: ToolErrorKind::GitError })?;
                Ok(ToolOutput::success(format!("Deleted: {}", name)))
            }
            BranchAction::Switch { name } => {
                if Self::has_uncommitted(&repo)? {
                    return Err(ToolError { message: "Uncommitted changes".into(), kind: ToolErrorKind::UncommittedChanges });
                }
                let obj = repo.revparse_single(&name).map_err(|e| ToolError { message: format!("Revparse: {}", e), kind: ToolErrorKind::GitError })?;
                repo.checkout_tree(&obj, None).map_err(|e| ToolError { message: format!("Checkout: {}", e), kind: ToolErrorKind::GitError })?;
                repo.set_head_detached(obj.id()).map_err(|e| ToolError { message: format!("HEAD: {}", e), kind: ToolErrorKind::GitError })?;
                Ok(ToolOutput::success(format!("Switched: {}", name)))
            }
        }
    }
}
