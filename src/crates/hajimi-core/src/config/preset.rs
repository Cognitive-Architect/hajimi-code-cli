//! Feature presets with default tool configurations

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Feature preset defining tool availability and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FeaturePreset {
    #[default]
    Daily,
    Minimal,
    Luxury,
    Offline,
    Paranoid,
    Performance,
    Frontend,
    Backend,
}

impl FeaturePreset {
    /// Get default tools for this preset
    pub fn default_tools(&self) -> Vec<&'static str> {
        match self {
            Self::Minimal => vec![
                "read_file", "write_file", "bash", "grep", "ls",
            ],
            Self::Daily => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "glob", "search", "fetch_url", "web_search",
                "edit_file", "view", "ask",
            ],
            Self::Luxury => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "glob", "search", "fetch_url", "web_search",
                "edit_file", "view", "ask",
                "code_review", "security_scan", "dependency_check",
                "test_runner", "benchmark", "profiler",
                "docker_build", "docker_run", "k8s_apply",
                "db_query", "redis_cmd", "mqtt_pub",
                "git_commit", "git_push", "git_rebase",
                "notify", "schedule", "queue_job",
                "render_chart", "export_pdf", "send_email",
                "video_encode", "audio_transcribe", "ocr",
                "lint", "format", "typecheck", "build",
                "deploy", "rollback", "monitor",
                "backup", "restore", "migrate",
            ],
            Self::Offline => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "glob", "search", "edit_file", "view", "ask",
                "lint", "format", "typecheck", "build",
                "test_runner", "benchmark",
            ],
            Self::Paranoid => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "ask_confirm", "ask_review", "ask_danger",
            ],
            Self::Performance => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "glob", "search", "fetch_url", "web_search",
                "edit_file", "view", "ask",
                "benchmark", "profiler", "cache_warm",
                "parallel_exec", "batch_query",
            ],
            Self::Frontend => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "glob", "search", "fetch_url", "web_search",
                "edit_file", "view", "ask",
                "npm_install", "vite_build", "webpack_bundle",
                "eslint", "prettier", "tsc",
                "react_dev", "next_build", "tailwind_gen",
                "browser_test", "a11y_check", "lighthouse",
                "deploy_vercel", "deploy_netlify",
            ],
            Self::Backend => vec![
                "read_file", "write_file", "bash", "grep", "ls",
                "glob", "search", "fetch_url", "web_search",
                "edit_file", "view", "ask",
                "cargo_build", "cargo_test", "cargo_bench",
                "docker_build", "docker_compose", "k8s_apply",
                "db_migrate", "redis_cmd", "kafka_pub",
                "grpc_call", "rest_test", "openapi_gen",
                "otel_trace", "prom_metrics", "health_check",
            ],
        }
    }

    /// Check if this preset requires network
    pub fn requires_network(&self) -> bool {
        !matches!(self, Self::Offline)
    }

    /// Check if this preset is security-focused
    pub fn is_security_focused(&self) -> bool {
        matches!(self, Self::Paranoid)
    }
}

impl FromStr for FeaturePreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(Self::Daily),
            "minimal" => Ok(Self::Minimal),
            "luxury" => Ok(Self::Luxury),
            "offline" => Ok(Self::Offline),
            "paranoid" => Ok(Self::Paranoid),
            "performance" => Ok(Self::Performance),
            "frontend" => Ok(Self::Frontend),
            "backend" => Ok(Self::Backend),
            _ => Err(format!("Unknown preset: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_from_str() {
        assert_eq!("daily".parse::<FeaturePreset>().unwrap(), FeaturePreset::Daily);
        assert_eq!("minimal".parse::<FeaturePreset>().unwrap(), FeaturePreset::Minimal);
        assert_eq!("luxury".parse::<FeaturePreset>().unwrap(), FeaturePreset::Luxury);
        assert!("invalid".parse::<FeaturePreset>().is_err());
    }

    #[test]
    fn test_preset_properties() {
        assert!(FeaturePreset::Offline.requires_network() == false);
        assert!(FeaturePreset::Daily.requires_network() == true);
        assert!(FeaturePreset::Paranoid.is_security_focused() == true);
        assert!(FeaturePreset::Daily.is_security_focused() == false);
    }

    #[test]
    fn test_default_tools_exist() {
        for preset in [
            FeaturePreset::Minimal,
            FeaturePreset::Daily,
            FeaturePreset::Luxury,
            FeaturePreset::Offline,
            FeaturePreset::Paranoid,
            FeaturePreset::Performance,
            FeaturePreset::Frontend,
            FeaturePreset::Backend,
        ] {
            let tools = preset.default_tools();
            assert!(!tools.is_empty(), "{:?} should have tools", preset);
        }
    }
}
