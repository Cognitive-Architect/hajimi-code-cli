//! Rust Doc Generator Tool - B-10/03
//! Generates HTML documentation for Rust projects

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::{
    Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions,
};

/// Rust Documentation Generator Tool
pub struct RustDocGeneratorTool;

impl RustDocGeneratorTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustDocGeneratorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct DocArgs {
    crate_path: String,
    #[serde(default)]
    open: bool,
    #[serde(default)]
    no_deps: bool,
    #[serde(default)]
    private: bool,
}

#[derive(Debug, Serialize)]
struct DocReport {
    success: bool,
    path: String,
    name: String,
    warns: Vec<String>,
    items: u32,
}

impl RustDocGeneratorTool {
    /// Returns read-only permissions
    fn permissions_read_only() -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }

    /// Find Cargo.toml
    async fn find_cargo(&self, path: &str) -> Result<PathBuf, ToolError> {
        let p = Path::new(path);
        if p.join("Cargo.toml").exists() {
            return Ok(p.join("Cargo.toml"));
        }
        if p.file_name().map(|n| n == "Cargo.toml").unwrap_or(false) {
            return Ok(p.to_path_buf());
        }
        Err(ToolError::new(format!("No Cargo.toml in: {}", p.display())))
    }

    /// Extract crate name from Cargo.toml
    async fn get_name(&self, cargo: &Path) -> Result<String, ToolError> {
        let s = fs::read_to_string(cargo)
            .await
            .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
        for l in s.lines() {
            let t = l.trim();
            if t.starts_with("name") {
                let p: Vec<&str> = t.split('=').collect();
                if p.len() == 2 {
                    return Ok(p[1].trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
        Ok("unknown".to_string())
    }

    /// Generate docs using cargo doc
    async fn generate(&self, path: &str, a: &DocArgs) -> Result<DocReport, ToolError> {
        let cargo = self.find_cargo(path).await?;
        let dir = cargo.parent().unwrap_or(Path::new("."));
        let name = self.get_name(&cargo).await?;

        if !Command::new("cargo")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Err(ToolError::new("cargo not in PATH"));
        }

        let mut cmd = vec!["doc"];
        if a.no_deps {
            cmd.push("--no-deps");
        }
        if a.private {
            cmd.push("--document-private-items");
        }
        if a.open {
            cmd.push("--open");
        }
        cmd.push("--message-format=short");

        let mut child = Command::new("cargo")
            .args(&cmd)
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::new(format!("Spawn: {}", e)))?;
        let mut warns = Vec::new();
        let mut items = 0u32;

        if let Some(o) = child.stdout.take() {
            let mut r = BufReader::new(o).lines();
            while let Ok(Some(l)) = r.next_line().await {
                if l.contains("Documenting") {
                    items += 1;
                }
            }
        }
        if let Some(e) = child.stderr.take() {
            let mut r = BufReader::new(e).lines();
            while let Ok(Some(l)) = r.next_line().await {
                if l.contains("warning:") {
                    warns.push(l);
                }
            }
        }

        let s = child
            .wait()
            .await
            .map_err(|e| ToolError::new(format!("Wait: {}", e)))?;
        if !s.success() {
            return Err(ToolError {
                message: format!("cargo doc failed: {:?}", s.code()),
                kind: ToolErrorKind::ExecutionFailed,
            });
        }
        Ok(DocReport {
            success: true,
            path: dir
                .join("target")
                .join("doc")
                .join(&name)
                .join("index.html")
                .to_string_lossy()
                .to_string(),
            name,
            warns,
            items,
        })
    }
}

#[async_trait]
impl Tool for RustDocGeneratorTool {
    fn name(&self) -> &str {
        "rust_doc_generator"
    }
    fn description(&self) -> &str {
        "Generate HTML documentation for Rust projects using cargo doc"
    }
    fn permissions(&self) -> ToolPermissions {
        Self::permissions_read_only()
    }
    fn is_enabled(&self, _config: &Config) -> bool {
        true
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: DocArgs =
            serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        if !Path::new(&a.crate_path).exists() {
            return Err(ToolError::new(format!("Not found: {}", a.crate_path)));
        }
        let r = self.generate(&a.crate_path, &a).await?;
        let mut out = vec![
            format!("📚 Rust Documentation"),
            format!("════════════════════"),
            format!("Crate: {}", r.name),
            format!("Status: ✅ Success"),
            format!(""),
            format!("📄 Path: {}", r.path),
            format!(""),
        ];
        if r.items > 0 {
            out.push(format!("📦 Items: {}", r.items));
        }
        if !r.warns.is_empty() {
            out.push(String::new());
            out.push(format!("⚠️  Warnings ({}):", r.warns.len()));
            for w in r.warns.iter().take(5) {
                out.push(format!("  {}", w));
            }
        }
        out.push(String::new());
        out.push("💡 Tips:".to_string());
        out.push("  - open: true to open browser".to_string());
        out.push("  - no_deps: true to skip deps".to_string());
        out.push("  - private: true for internal".to_string());
        Ok(ToolOutput::success(format!(
            "{}\n\n📄 JSON:\n{}",
            out.join("\n"),
            serde_json::to_string_pretty(&r).map_err(|e| ToolError::new(format!("JSON: {}", e)))?
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rust_doc_generator_name() {
        assert_eq!(RustDocGeneratorTool::new().name(), "rust_doc_generator");
    }

    #[test]
    fn test_rust_doc_generator_description() {
        assert!(RustDocGeneratorTool::new()
            .description()
            .contains("cargo doc"));
    }

    #[test]
    fn test_rust_doc_generator_permissions() {
        let p = RustDocGeneratorTool::new().permissions();
        assert_eq!(p.default_level, PermissionLevel::Allow);
    }

    #[test]
    fn test_rust_doc_generator_is_enabled() {
        assert!(RustDocGeneratorTool::new().is_enabled(&Config::default()));
    }

    #[tokio::test]
    async fn test_rust_doc_generator_execute_invalid_path() {
        assert!(RustDocGeneratorTool::new()
            .execute(json!({"crate_path": "/bad"}))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_find_cargo_toml_not_found() {
        assert!(RustDocGeneratorTool::new()
            .find_cargo("/nonexistent")
            .await
            .is_err());
    }
}
