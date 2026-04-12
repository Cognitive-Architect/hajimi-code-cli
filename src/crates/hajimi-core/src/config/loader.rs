//! Configuration loader supporting file, env, and CLI sources

use super::{CliArgs, Config, ConfigError, FeaturePreset, LlmConfig, PathConfig, TimeoutConfig};
use std::path::Path;
use tokio::fs;

/// Macro to merge field with priority: CLI > Env > File
macro_rules! merge_field {
    ($cli:ident, $env:ident, $file:ident, $field:ident, $default:expr) => {
        if $cli.$field != $default { $cli.$field.clone() }
        else if $env.$field != $default { $env.$field.clone() }
        else { $file.$field.clone() }
    };
}

/// Configuration loader with multi-source support
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from TOML file
    pub async fn from_file(path: &Path) -> Result<Config, ConfigError> {
        let content = fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Config, ConfigError> {
        let preset = parse_field("HAJIMI_PRESET")?.unwrap_or(FeaturePreset::Daily);
        let enabled_tools = parse_tools(&preset)?;
        
        Ok(Config {
            preset,
            enabled_tools,
            llm: LlmConfig {
                provider: parse_field("HAJIMI_PROVIDER")?.unwrap_or_else(|| "openai".into()),
                api_key: std::env::var("HAJIMI_API_KEY").ok(),
                model: parse_field("HAJIMI_MODEL")?.unwrap_or_else(|| "gpt-4o-mini".into()),
                api_url: parse_field("HAJIMI_API_URL")?.unwrap_or_else(|| "https://api.openai.com/v1".into()),
            },
            timeouts: TimeoutConfig {
                request_secs: parse_field("HAJIMI_TIMEOUT_REQUEST")?.unwrap_or(30),
                connect_secs: parse_field("HAJIMI_TIMEOUT_CONNECT")?.unwrap_or(10),
            },
            paths: PathConfig {
                cache_dir: std::env::var("HAJIMI_CACHE_DIR").ok().map(Into::into),
                log_dir: std::env::var("HAJIMI_LOG_DIR").ok().map(Into::into),
            },
        })
    }

    /// Load configuration from CLI arguments
    pub fn from_cli(args: &CliArgs) -> Result<Config, ConfigError> {
        let preset = match args.preset.as_ref() {
            Some(s) => s.parse().map_err(ConfigError::InvalidPreset)?,
            None => FeaturePreset::Daily,
        };
        Ok(Config {
            preset,
            enabled_tools: if args.enabled_tools.is_empty() { 
                preset.default_tools().into_iter().map(|s| s.to_string()).collect()
            } else { 
                args.enabled_tools.clone() 
            },
            llm: LlmConfig {
                provider: args.provider.clone().unwrap_or_else(|| "openai".into()),
                api_key: args.api_key.clone(),
                model: args.model.clone().unwrap_or_else(|| "gpt-4o-mini".into()),
                api_url: "https://api.openai.com/v1".into(),
            },
            timeouts: TimeoutConfig::default(),
            paths: PathConfig::default(),
        })
    }

    /// Merge configurations with priority: CLI > Env > File
    pub fn merge(file: Config, env: Config, cli: Config) -> Config {
        let default = Config::default();
        Config {
            preset: merge_field!(cli, env, file, preset, default.preset),
            enabled_tools: merge_field!(cli, env, file, enabled_tools, default.enabled_tools),
            llm: merge_llm(file.llm, env.llm, cli.llm),
            timeouts: merge_timeouts(file.timeouts, env.timeouts, cli.timeouts),
            paths: merge_paths(file.paths, env.paths, cli.paths),
        }
    }
}

/// Parse a single environment variable field
fn parse_field<T: std::str::FromStr>(key: &str) -> Result<Option<T>, ConfigError> {
    Ok(std::env::var(key).ok().and_then(|s| s.parse().ok()))
}

/// Parse enabled tools from env or use preset defaults
fn parse_tools(preset: &FeaturePreset) -> Result<Vec<String>, ConfigError> {
    Ok(std::env::var("HAJIMI_TOOLS")
        .ok()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
        .unwrap_or_else(|| preset.default_tools().into_iter().map(|s| s.to_string()).collect()))
}

fn merge_llm(file: LlmConfig, env: LlmConfig, cli: LlmConfig) -> LlmConfig {
    LlmConfig {
        provider: pick(cli.provider, env.provider, file.provider, || "openai".into()),
        api_key: cli.api_key.or(env.api_key).or(file.api_key),
        model: pick(cli.model, env.model, file.model, || "gpt-4o-mini".into()),
        api_url: pick(cli.api_url, env.api_url, file.api_url, || "https://api.openai.com/v1".into()),
    }
}

fn merge_timeouts(file: TimeoutConfig, env: TimeoutConfig, cli: TimeoutConfig) -> TimeoutConfig {
    TimeoutConfig {
        request_secs: pick_num(cli.request_secs, env.request_secs, file.request_secs, 30),
        connect_secs: pick_num(cli.connect_secs, env.connect_secs, file.connect_secs, 10),
    }
}

fn merge_paths(file: PathConfig, env: PathConfig, cli: PathConfig) -> PathConfig {
    PathConfig {
        cache_dir: cli.cache_dir.or(env.cache_dir).or(file.cache_dir),
        log_dir: cli.log_dir.or(env.log_dir).or(file.log_dir),
    }
}

fn pick<T: PartialEq + Clone>(cli: T, env: T, file: T, default: impl FnOnce() -> T) -> T {
    let default = default();
    if cli != default { cli } else if env != default { env } else { file }
}

fn pick_num(cli: u64, env: u64, file: u64, default: u64) -> u64 {
    if cli != default { cli } else if env != default { env } else { file }
}
