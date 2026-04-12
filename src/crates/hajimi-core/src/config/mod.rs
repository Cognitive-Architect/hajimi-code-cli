//! Configuration management for HAJIMI Core

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod hotreload;
pub mod loader;
pub mod preset;

pub use hotreload::{ConfigManager, HotReloadHandle};
pub use loader::ConfigLoader;
pub use preset::FeaturePreset;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub preset: FeaturePreset,
    #[serde(default)]
    pub enabled_tools: Vec<String>,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub timeouts: TimeoutConfig,
    #[serde(default)]
    pub paths: PathConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preset: FeaturePreset::Daily,
            enabled_tools: FeaturePreset::Daily.default_tools()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            llm: LlmConfig::default(),
            timeouts: TimeoutConfig::default(),
            paths: PathConfig::default(),
        }
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: None,
            model: default_model(),
            api_url: default_api_url(),
        }
    }
}

fn default_provider() -> String {
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_api_url() -> String {
    "https://api.openai.com/v1".to_string()
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeoutConfig {
    #[serde(default = "default_request_timeout")]
    pub request_secs: u64,
    #[serde(default = "default_connect_timeout")]
    pub connect_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_secs: default_request_timeout(),
            connect_secs: default_connect_timeout(),
        }
    }
}

fn default_request_timeout() -> u64 {
    30
}

fn default_connect_timeout() -> u64 {
    10
}

/// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PathConfig {
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub log_dir: Option<PathBuf>,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            log_dir: None,
        }
    }
}

/// CLI arguments structure (for from_cli)
#[derive(Debug, Clone, Default)]
pub struct CliArgs {
    pub preset: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub config_file: Option<PathBuf>,
    pub enabled_tools: Vec<String>,
}

/// Configuration errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid preset: {0}")]
    InvalidPreset(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_uses_daily_preset() {
        let config = Config::default();
        assert_eq!(config.preset, FeaturePreset::Daily);
        assert!(!config.enabled_tools.is_empty());
    }

    #[test]
    fn test_preset_tools_count() {
        assert_eq!(FeaturePreset::Minimal.default_tools().len(), 5);
        assert_eq!(FeaturePreset::Daily.default_tools().len(), 12);
        assert_eq!(FeaturePreset::Luxury.default_tools().len(), 46);
    }
}
