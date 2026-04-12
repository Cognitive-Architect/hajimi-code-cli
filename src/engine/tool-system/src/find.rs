//! Find Tool - CORR-W09-01 / DEBT-LINES-W09-05

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};

pub struct FindTool;
impl FindTool { pub fn new() -> Self { Self } }
impl Default for FindTool { fn default() -> Self { Self::new() } }

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
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: FindArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let base = Path::new(&a.path);
        if !base.exists() { return Err(ToolError::new("Path not found")); }
        let mut r = Vec::new();
        let mut s = vec![base.to_path_buf()];
        while let Some(dir) = s.pop() {
            let mut d = tokio::fs::read_dir(&dir).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
            while let Ok(Some(e)) = d.next_entry().await {
                let p = e.path();
                let n = e.file_name().to_string_lossy().to_string();
                let is_d = e.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                if let Some(ref ft) = a.file_type { if !match ft.as_str() { "f" => !is_d, "d" => is_d, _ => true } { continue; } }
                if let Some(ref pat) = a.name { if !n.contains(pat) { continue; } }
                if let Some(max) = a.max_size { if !is_d { if let Ok(m) = e.metadata().await { if m.len() > max { continue; } } } }
                r.push(p.to_string_lossy().to_string());
                if is_d { s.push(p); }
            }
        }
        Ok(ToolOutput { stdout: r.join("\n"), stderr: String::new(), exit_code: Some(0) })
    }
}

/// Find result for piping to grep
#[derive(Debug, Clone, Serialize)]
pub struct FindResult { pub files: Vec<String> }
impl FindResult {
    pub fn new(files: Vec<String>) -> Self { Self { files } }
    pub fn is_empty(&self) -> bool { self.files.is_empty() }
}
impl From<Vec<String>> for FindResult { fn from(files: Vec<String>) -> Self { Self::new(files) } }
