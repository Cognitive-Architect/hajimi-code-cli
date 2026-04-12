//! Find Tool - B-W09-05 / CORR-W09-01
//! DEBT-LINES-W09-05: 80行目标（原search.rs 164行拆分）
//!
//! Find files by name, type, or size

use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

use crate::tool::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};

/// Find tool for locating files by name/type/size
pub struct FindTool;

impl FindTool {
    pub fn new() -> Self { Self }
}

impl Default for FindTool {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Deserialize)]
pub struct FindArgs {
    pub path: String,
    #[serde(default)] pub name: Option<String>,
    #[serde(default)] pub file_type: Option<String>, // "f"=file, "d"=dir
    #[serde(default)] pub max_size: Option<u64>,
}

#[async_trait]
impl Tool for FindTool {
    fn name(&self) -> &str { "find" }
    fn description(&self) -> &str { "Find files by name, type, or size" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None }
    }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: FindArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Invalid args: {}", e)))?;
        let base = Path::new(&args.path);
        if !base.exists() { return Err(ToolError::new("Path not found")); }

        let mut results = Vec::new();
        let mut stack = vec![base.to_path_buf()];

        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| ToolError::new(format!("Read dir: {}", e)))?;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);

                // Type filter
                if let Some(ref ft) = args.file_type {
                    let matches = match ft.as_str() {
                        "f" => !is_dir,
                        "d" => is_dir,
                        _ => true,
                    };
                    if !matches { continue; }
                }

                // Name filter
                if let Some(ref pattern) = args.name {
                    if !name.contains(pattern) { continue; }
                }

                // Size filter (files only)
                if let Some(max) = args.max_size {
                    if !is_dir {
                        if let Ok(meta) = entry.metadata().await {
                            if meta.len() > max { continue; }
                        }
                    }
                }

                results.push(path.to_string_lossy().to_string());

                // Recurse into directories
                if is_dir {
                    stack.push(path);
                }
            }
        }

        Ok(ToolOutput { stdout: results.join("\n"), stderr: String::new(), exit_code: Some(0) })
    }
}

/// Find result for piping to grep
#[derive(Debug, Clone)]
pub struct FindResult {
    pub files: Vec<String>,
}

impl FindResult {
    pub fn new(files: Vec<String>) -> Self {
        Self { files }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl From<Vec<String>> for FindResult {
    fn from(files: Vec<String>) -> Self {
        Self::new(files)
    }
}
