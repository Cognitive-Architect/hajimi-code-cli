//! Directory tools - B-W09-03/04

use super::{PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

pub struct ListDirectoryTool;
impl Default for ListDirectoryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ListDirectoryTool {
    pub fn new() -> Self {
        Self
    }
}

pub struct GlobTool;
impl Default for GlobTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct ListArgs {
    path: String,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    max_depth: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct GlobArgs {
    pattern: String,
    #[serde(default)]
    path: String,
}

#[async_trait::async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }
    fn description(&self) -> &str {
        "List directory with optional recursion"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: ListArgs =
            serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let path = PathBuf::from(&args.path);
        if !path.exists() {
            return Err(ToolError::new("Not found"));
        }
        if !path.is_dir() {
            return Err(ToolError::new("Not a directory"));
        }

        let max_depth = args
            .max_depth
            .unwrap_or(if args.recursive { 100 } else { 1 });
        let entries = list_dir_iter(&path, args.recursive, max_depth).await?;
        let json =
            serde_json::to_string(&entries).map_err(|e| ToolError::new(format!("JSON: {}", e)))?;
        Ok(ToolOutput::success(json))
    }
}

#[async_trait::async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }
    fn description(&self) -> &str {
        "Find files matching glob pattern"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: GlobArgs =
            serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let base = PathBuf::from(&args.path);
        let entries = list_dir_iter(&base, true, 100).await?;
        let pattern = &args.pattern;
        let matched: Vec<_> = entries
            .into_iter()
            .filter(|e| glob_match(&e.name, pattern) || glob_match(&e.path, pattern))
            .collect();
        let json =
            serde_json::to_string(&matched).map_err(|e| ToolError::new(format!("JSON: {}", e)))?;
        Ok(ToolOutput::success(json))
    }
}

fn glob_match(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with("*.") {
        return name.ends_with(&pattern[1..]);
    }
    if pattern.contains('*') {
        let parts: Vec<_> = pattern.split('*').collect();
        if parts.len() == 2 {
            return name.starts_with(parts[0]) && name.ends_with(parts[1]);
        }
    }
    name == pattern
}

async fn list_dir_iter(
    start: &std::path::Path,
    recursive: bool,
    max_depth: usize,
) -> Result<Vec<DirEntry>, ToolError> {
    let mut entries = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(start.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        if depth >= max_depth {
            continue;
        }
        let mut reader = tokio::fs::read_dir(&dir)
            .await
            .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|e| ToolError::new(format!("Entry: {}", e)))?
        {
            let meta = entry.metadata().await.ok();
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            entries.push(DirEntry {
                path: path.to_string_lossy().to_string(),
                modified: meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                name,
                is_dir,
            });
            if recursive && is_dir && depth + 1 < max_depth {
                stack.push((path, depth + 1));
            }
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("test.rs", "*.rs"));
        assert!(glob_match("test", "*"));
        assert!(!glob_match("test.txt", "*.rs"));
    }
}
