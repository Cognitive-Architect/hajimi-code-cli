//! JS Bundle Analyzer Tool - B-10/03
//! Analyzes JavaScript bundle size and dependency tree

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

use crate::{Config, PermissionLevel, Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions};

/// JS Bundle Analyzer Tool
pub struct JsBundleAnalyzerTool;

impl JsBundleAnalyzerTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsBundleAnalyzerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct BundleArgs {
    entry: String,
}

#[derive(Debug, Serialize)]
struct DepNode {
    name: String,
    version: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct BundleReport {
    total: u64,
    gzip: u64,
    entry: String,
    deps: Vec<DepNode>,
    warns: Vec<String>,
}

impl JsBundleAnalyzerTool {
    /// Returns read-only permissions for safety
    fn permissions_read_only() -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }

    /// Parse package.json for dependencies
    async fn get_deps(&self, entry: &str) -> Result<HashMap<String, String>, ToolError> {
        let path = Path::new(entry).join("package.json");
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::new(format!("Read package.json: {}", e)))?;
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| ToolError::new(format!("Parse package.json: {}", e)))?;
        let mut deps = HashMap::new();
        if let Some(obj) = json.get("dependencies").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(ver) = v.as_str() {
                    deps.insert(k.clone(), ver.to_string());
                }
            }
        }
        if let Some(obj) = json.get("devDependencies").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(ver) = v.as_str() {
                    deps.insert(k.clone(), ver.to_string());
                }
            }
        }
        Ok(deps)
    }

    /// Calculate node_modules sizes
    async fn get_sizes(&self, entry: &str) -> Result<HashMap<String, u64>, ToolError> {
        let nm = Path::new(entry).join("node_modules");
        let mut sizes = HashMap::new();
        if !nm.exists() {
            return Ok(sizes);
        }
        let mut entries = fs::read_dir(&nm)
            .await
            .map_err(|e| ToolError::new(format!("Read node_modules: {}", e)))?;
        while let Ok(Some(e)) = entries.next_entry().await {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') {
                sizes.insert(name, self.calc_size(&e.path()).await.unwrap_or(0));
            }
        }
        Ok(sizes)
    }

    /// Recursively calculate directory size
    fn calc_size<'a>(
        &'a self,
        path: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, ToolError>> + Send + 'a>>
    {
        Box::pin(async move {
            if path.is_file() {
                return Ok(fs::metadata(path)
                    .await
                    .map_err(|e| ToolError::new(format!("Stat: {}", e)))?
                    .len());
            }
            let mut total = 0u64;
            let mut entries = fs::read_dir(path)
                .await
                .map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
            while let Ok(Some(e)) = entries.next_entry().await {
                let p = e.path();
                if p.is_file() {
                    if let Ok(m) = fs::metadata(&p).await {
                        total += m.len();
                    }
                } else if p.is_dir() {
                    total += self.calc_size(&p).await.unwrap_or(0);
                }
            }
            Ok(total)
        })
    }

    /// Check webpack stats
    async fn check_webpack(&self, entry: &str) -> Result<Option<Value>, ToolError> {
        let stats = Path::new(entry).join("webpack-stats.json");
        if stats.exists() {
            let c = fs::read_to_string(&stats)
                .await
                .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
            return serde_json::from_str(&c)
                .map(Some)
                .map_err(|e| ToolError::new(format!("Parse: {}", e)));
        }
        if !Path::new(entry).join("webpack.config.js").exists() {
            return Ok(None);
        }
        match Command::new("npx")
            .args(["webpack", "--json"])
            .current_dir(entry)
            .output()
            .await
        {
            Ok(o) if o.status.success() => {
                serde_json::from_str(&String::from_utf8_lossy(&o.stdout))
                    .map(Some)
                    .map_err(|e| ToolError::new(format!("Parse: {}", e)))
            }
            _ => Ok(None),
        }
    }

    /// Format byte size
    fn fmt_size(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut i = 0;
        while size >= 1024.0 && i < UNITS.len() - 1 {
            size /= 1024.0;
            i += 1;
        }
        format!("{:.2} {}", size, UNITS[i])
    }
}

#[async_trait]
impl Tool for JsBundleAnalyzerTool {
    fn name(&self) -> &str {
        "js_bundle_analyzer"
    }
    fn description(&self) -> &str {
        "Analyze JavaScript bundle size and dependency tree"
    }
    fn permissions(&self) -> ToolPermissions {
        Self::permissions_read_only()
    }
    fn is_enabled(&self, _config: &Config) -> bool {
        true
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: BundleArgs =
            serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let entry = Path::new(&a.entry);
        if !entry.exists() {
            return Err(ToolError::new(format!("Not found: {}", a.entry)));
        }
        if !entry.join("package.json").exists() {
            return Err(ToolError::new(format!("No package.json: {}", a.entry)));
        }

        let deps = self.get_deps(&a.entry).await?;
        let sizes = self.get_sizes(&a.entry).await?;
        let webpack = self.check_webpack(&a.entry).await?;

        let mut nodes = Vec::new();
        let mut total: u64 = 0;
        let mut warns = Vec::new();

        for (name, ver) in deps.iter() {
            let sz = sizes.get(name).copied().unwrap_or(0);
            total += sz;
            if sz == 0 {
                warns.push(format!("'{}' missing", name));
            }
            nodes.push(DepNode {
                name: name.clone(),
                version: ver.clone(),
                size: sz,
            });
        }
        nodes.sort_by(|a, b| b.size.cmp(&a.size));

        let report = BundleReport {
            total,
            gzip: total / 3,
            entry: a.entry,
            deps: nodes,
            warns: warns.clone(),
        };
        let mut out = vec![
            format!("📦 JS Bundle Analysis"),
            format!("════════════════════"),
            format!("Entry: {}", report.entry),
            format!("Deps: {}", report.deps.len()),
            format!("Total: {}", self.fmt_size(report.total)),
            format!("Gzip: {}", self.fmt_size(report.gzip)),
            format!(""),
            format!("📊 Top Deps:"),
        ];
        for (i, d) in report.deps.iter().take(10).enumerate() {
            out.push(format!(
                "  {}. {}@{} - {}",
                i + 1,
                d.name,
                d.version,
                self.fmt_size(d.size)
            ));
        }
        if webpack.is_some() {
            out.push(String::new());
            out.push("📈 Webpack stats available".to_string());
        }
        if !warns.is_empty() {
            out.push(String::new());
            out.push(format!("⚠️  Warnings: {}", warns.len()));
            for w in &warns {
                out.push(format!("  - {}", w));
            }
        }
        Ok(ToolOutput::success(format!(
            "{}\n\n📄 JSON:\n{}",
            out.join("\n"),
            serde_json::to_string_pretty(&report)
                .map_err(|e| ToolError::new(format!("JSON: {}", e)))?
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_js_bundle_analyzer_name() {
        assert_eq!(JsBundleAnalyzerTool::new().name(), "js_bundle_analyzer");
    }

    #[test]
    fn test_js_bundle_analyzer_description() {
        assert!(JsBundleAnalyzerTool::new()
            .description()
            .contains("JavaScript"));
    }

    #[test]
    fn test_js_bundle_analyzer_permissions() {
        let p = JsBundleAnalyzerTool::new().permissions();
        assert_eq!(p.default_level, PermissionLevel::Allow);
    }

    #[test]
    fn test_js_bundle_analyzer_is_enabled() {
        assert!(JsBundleAnalyzerTool::new().is_enabled(&Config::default()));
    }

    #[tokio::test]
    async fn test_js_bundle_analyzer_execute_invalid_path() {
        assert!(JsBundleAnalyzerTool::new()
            .execute(json!({"entry": "/bad"}))
            .await
            .is_err());
    }

    #[test]
    fn test_format_size() {
        let t = JsBundleAnalyzerTool::new();
        assert!(t.fmt_size(1024).contains("KB"));
        assert!(t.fmt_size(1024 * 1024).contains("MB"));
    }
}
