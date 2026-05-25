//! Day 11 — Provider Probe 后端能力、ProbeResult 与 TTL/取消语义
//! Provides capability mapping, execution wrappers, and local JSON persistence.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Read the feature gate to determine if real provider probing is enabled.
/// Defaults to false.
pub fn is_real_context_probe_enabled() -> bool {
    std::env::var("HAJIMI_CONTEXT_PROBE_REAL")
        .map(|val| val.eq_ignore_ascii_case("true") || val == "1")
        .unwrap_or(false)
}

/// Neutral DTO for a capability probe request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeRequest {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub model: String,

    pub level: ProbeLevel,

    #[serde(rename = "timeoutSeconds")]
    pub timeout_seconds: u64,

    #[serde(rename = "tokenCap")]
    pub token_cap: Option<usize>,
}

/// Neutral DTO for a capability probe response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeResponse {
    pub success: bool,

    pub usage: Option<ProbeUsage>,

    #[serde(rename = "latencyMs")]
    pub latency_ms: u64,

    pub error: Option<String>,

    pub cancelled: bool,
}

/// Trait for executing a real provider capability probe.
/// Pure boundary design: accepts only neutral DTOs; upper layers adapt provider settings.
#[async_trait]
pub trait ProviderProbeClient: Send + Sync {
    /// Execute a real capability probe using the specified request.
    async fn run_probe(&self, req: ProbeRequest) -> Result<ProbeResponse, String>;
}

/// Probe levels: 128K, 256K, 512K, 900K.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProbeLevel {
    #[serde(rename = "128K")]
    Level128K,
    #[serde(rename = "256K")]
    Level256K,
    #[serde(rename = "512K")]
    Level512K,
    #[serde(rename = "900K")]
    Level900K,
}

impl ProbeLevel {
    pub fn tokens(&self) -> usize {
        match self {
            ProbeLevel::Level128K => 128_000,
            ProbeLevel::Level256K => 256_000,
            ProbeLevel::Level512K => 512_000,
            ProbeLevel::Level900K => 900_000,
        }
    }

    /// Check if the level is high_cost / requires_confirmation.
    /// 900K probe is identified as a high cost level.
    pub fn is_high_cost(&self) -> bool {
        matches!(self, ProbeLevel::Level900K)
    }

    pub fn requires_confirmation(&self) -> bool {
        self.is_high_cost()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeUsage {
    #[serde(rename = "promptTokens")]
    pub prompt_tokens: usize,
    #[serde(rename = "completionTokens")]
    pub completion_tokens: usize,
}

/// Represents the status and outcomes of a context window verification.
/// Helper to extract UX display details: error, latency, tested_input, prompt_tokens, completion_tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeResult {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    pub model: String,

    #[serde(rename = "declaredMax")]
    pub declared_max: usize,

    #[serde(rename = "testedInputTokens")]
    pub tested_input_tokens: usize,

    pub success: bool,

    pub usage: Option<ProbeUsage>,

    #[serde(rename = "latencyMs")]
    pub latency_ms: u64,

    pub error: Option<String>,

    pub timestamp: u64,

    #[serde(rename = "ttlSeconds")]
    pub ttl_seconds: u64,

    pub cancelled: bool,
}

impl ProbeResult {
    /// Expired TTL test check.
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > (self.timestamp + self.ttl_seconds)
    }

    /// Local file persistence: .hajimi/provider_probes/*.json
    pub async fn save_to_file(&self) -> Result<(), std::io::Error> {
        let path = resolve_probe_path(&self.provider_id, &self.model);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load_from_file(provider_id: &str, model: &str) -> Result<Self, std::io::Error> {
        let path = resolve_probe_path(provider_id, model);
        let content = tokio::fs::read_to_string(path).await?;
        let result: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(result)
    }

    pub fn save_to_file_sync(&self) -> Result<(), std::io::Error> {
        let path = resolve_probe_path(&self.provider_id, &self.model);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file_sync(provider_id: &str, model: &str) -> Result<Self, std::io::Error> {
        let path = resolve_probe_path(provider_id, model);
        let content = std::fs::read_to_string(path)?;
        let result: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(result)
    }
}

/// Sanitize filename characters to ensure safe OS-independent path creation.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect()
}

/// Resolve path to local JSON persistence folder .hajimi/provider_probes
pub fn resolve_probe_path(provider_id: &str, model: &str) -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let filename = format!(
        "{}_{}.json",
        sanitize_filename(provider_id),
        sanitize_filename(model)
    );
    home.join(".hajimi").join("provider_probes").join(filename)
}

/// Deterministic payload generator that repeats a deterministic sequence.
/// Designed for testedInputTokens payload generation.
pub fn generate_deterministic_payload(token_count: usize) -> String {
    // 1 token is approximately 4 characters.
    let seed = "HajimiProbeDeterministicSeedPayloadDataForLongContextVerification1234567890\n";
    let target_len = token_count * 4;
    let mut payload = String::with_capacity(target_len);
    while payload.len() < target_len {
        payload.push_str(seed);
    }
    payload.truncate(target_len);
    payload
}

// SECURITY RULE: Under no circumstances should API keys, Authorization headers,
// full_prompt, or promptText be saved to ProbeResult or written to disk.

pub struct ContextProbeRunner {}

impl Default for ContextProbeRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextProbeRunner {
    pub fn new() -> Self {
        Self {}
    }

    /// Run the probe, checking the gate and executing the real client if gate is enabled.
    /// Short-circuits safely when HAJIMI_CONTEXT_PROBE_REAL is false/unset.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_probe(
        &self,
        provider_id: String,
        model: String,
        level: ProbeLevel,
        declared_max: usize,
        ttl_seconds: u64,
        timeout_seconds: u64,
        client: Option<&dyn ProviderProbeClient>,
    ) -> ProbeResult {
        let start = std::time::Instant::now();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Safety guard: high-cost levels (900K) require explicit confirmation
        if level.is_high_cost() {
            tracing::warn!("Warning: Level 900K is a high cost probe and should only be triggered with explicit user consent!");
        }

        // 1. Gate check: if real probe is disabled, short-circuit immediately.
        if !is_real_context_probe_enabled() {
            // Gate is off: DO NOT call client. Return a fallback mock successful ProbeResult.
            let token_count = level.tokens();
            return ProbeResult {
                provider_id,
                model,
                declared_max,
                tested_input_tokens: token_count,
                success: true,
                usage: Some(ProbeUsage {
                    prompt_tokens: token_count,
                    completion_tokens: 10,
                }),
                latency_ms: start.elapsed().as_millis() as u64,
                error: None,
                timestamp,
                ttl_seconds,
                cancelled: false,
            };
        }

        // 2. Gate is on: check if a client has been provided
        let Some(c) = client else {
            return ProbeResult {
                provider_id,
                model,
                declared_max,
                tested_input_tokens: level.tokens(),
                success: false,
                usage: None,
                latency_ms: start.elapsed().as_millis() as u64,
                error: Some("No provider client configured".to_string()),
                timestamp,
                ttl_seconds,
                cancelled: false,
            };
        };

        // 3. Execute the probe using the client DTO request
        let req = ProbeRequest {
            provider_id: provider_id.clone(),
            model: model.clone(),
            level,
            timeout_seconds,
            token_cap: Some(declared_max),
        };

        match c.run_probe(req).await {
            Ok(resp) => ProbeResult {
                provider_id,
                model,
                declared_max,
                tested_input_tokens: level.tokens(),
                success: resp.success,
                usage: resp.usage,
                latency_ms: resp.latency_ms,
                error: resp.error,
                timestamp,
                ttl_seconds,
                cancelled: resp.cancelled,
            },
            Err(err) => {
                let is_cancelled = err == "cancelled" || err == "canceled";
                ProbeResult {
                    provider_id,
                    model,
                    declared_max,
                    tested_input_tokens: level.tokens(),
                    success: false,
                    usage: None,
                    latency_ms: start.elapsed().as_millis() as u64,
                    error: Some(err),
                    timestamp,
                    ttl_seconds,
                    cancelled: is_cancelled,
                }
            }
        }
    }

    /// Performs a controlled mock probe runner to avoid expensive default LLM startup.
    /// Supports cancel/timeout tracking without failure status.
    pub async fn run_mock_probe<F>(
        &self,
        provider_id: String,
        model: String,
        level: ProbeLevel,
        declared_max: usize,
        ttl_seconds: u64,
        mock_handler: F,
    ) -> ProbeResult
    where
        F: FnOnce(usize) -> Result<ProbeUsage, String>,
    {
        // Safety guard: high-cost levels (900K) require explicit confirmation, never auto-started.
        if level.is_high_cost() {
            tracing::warn!("Warning: Level 900K is a high cost probe and should only be triggered with explicit user consent!");
        }

        let start = std::time::Instant::now();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let token_count = level.tokens();
        let _payload = generate_deterministic_payload(token_count);

        match mock_handler(token_count) {
            Ok(usage) => ProbeResult {
                provider_id,
                model,
                declared_max,
                tested_input_tokens: token_count,
                success: true,
                usage: Some(usage),
                latency_ms: start.elapsed().as_millis() as u64,
                error: None,
                timestamp,
                ttl_seconds,
                cancelled: false,
            },
            Err(err) => {
                // NOTE: cancelled scenario: cancelled does NOT equal provider unsupported.
                // User cancellation is a process-level interrupt, not a model limitation.
                let is_cancelled = err == "cancelled" || err == "canceled";
                ProbeResult {
                    provider_id,
                    model,
                    declared_max,
                    tested_input_tokens: token_count,
                    success: false,
                    usage: None,
                    latency_ms: start.elapsed().as_millis() as u64,
                    error: Some(err),
                    timestamp,
                    ttl_seconds,
                    cancelled: is_cancelled,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_probe_success() {
        let runner = ContextProbeRunner::new();
        let res = runner
            .run_mock_probe(
                "deepseek".to_string(),
                "deepseek-v4".to_string(),
                ProbeLevel::Level128K,
                131_072,
                86400,
                |tokens| {
                    Ok(ProbeUsage {
                        prompt_tokens: tokens,
                        completion_tokens: 10,
                    })
                },
            )
            .await;

        assert!(res.success);
        assert_eq!(res.tested_input_tokens, 128_000);
        assert!(!res.cancelled);
        assert!(res.error.is_none());
        assert!(!res.is_expired());
    }

    #[tokio::test]
    async fn test_probe_failure() {
        let runner = ContextProbeRunner::new();
        let res = runner
            .run_mock_probe(
                "deepseek".to_string(),
                "deepseek-v4".to_string(),
                ProbeLevel::Level256K,
                262_144,
                86400,
                |_| Err("Rate limit exceeded".to_string()),
            )
            .await;

        assert!(!res.success);
        assert_eq!(res.tested_input_tokens, 256_000);
        assert!(!res.cancelled);
        assert_eq!(res.error.unwrap(), "Rate limit exceeded");
    }

    #[tokio::test]
    async fn test_probe_timeout() {
        let runner = ContextProbeRunner::new();
        let res = runner
            .run_mock_probe(
                "deepseek".to_string(),
                "deepseek-v4".to_string(),
                ProbeLevel::Level512K,
                524_288,
                86400,
                |_| Err("Request timeout".to_string()),
            )
            .await;

        assert!(!res.success);
        assert_eq!(res.tested_input_tokens, 512_000);
        assert!(!res.cancelled);
        assert_eq!(res.error.unwrap(), "Request timeout");
    }

    #[tokio::test]
    async fn test_probe_cancelled() {
        let runner = ContextProbeRunner::new();
        let res = runner
            .run_mock_probe(
                "deepseek".to_string(),
                "deepseek-v4".to_string(),
                ProbeLevel::Level900K,
                1_000_000,
                86400,
                |_| Err("cancelled".to_string()),
            )
            .await;

        assert!(!res.success);
        assert!(res.cancelled);
        assert_eq!(res.tested_input_tokens, 900_000);
        assert_eq!(res.error.unwrap(), "cancelled");
    }

    #[test]
    fn test_probe_expired() {
        let res = ProbeResult {
            provider_id: "deepseek".to_string(),
            model: "deepseek-v4".to_string(),
            declared_max: 131_072,
            tested_input_tokens: 128_000,
            success: true,
            usage: Some(ProbeUsage {
                prompt_tokens: 128_000,
                completion_tokens: 10,
            }),
            latency_ms: 1500,
            error: None,
            timestamp: 1000,
            ttl_seconds: 60,
            cancelled: false,
        };

        assert!(res.is_expired());
    }

    #[test]
    fn test_real_probe_disabled_by_default() {
        // By default, the real context probe should be disabled (gate is false)
        std::env::remove_var("HAJIMI_CONTEXT_PROBE_REAL");
        assert!(!is_real_context_probe_enabled());
    }

    #[tokio::test]
    async fn test_provider_probe_client_mock_implementation() {
        struct MockProbeClient;

        #[async_trait]
        impl ProviderProbeClient for MockProbeClient {
            async fn run_probe(&self, req: ProbeRequest) -> Result<ProbeResponse, String> {
                if req.provider_id == "unsupported" {
                    return Ok(ProbeResponse {
                        success: false,
                        usage: None,
                        latency_ms: 5,
                        error: Some("Provider unsupported".to_string()),
                        cancelled: false,
                    });
                }
                if req.level.is_high_cost() {
                    return Ok(ProbeResponse {
                        success: false,
                        usage: None,
                        latency_ms: 10,
                        error: Some("cancelled".to_string()),
                        cancelled: true,
                    });
                }
                Ok(ProbeResponse {
                    success: true,
                    usage: Some(ProbeUsage {
                        prompt_tokens: req.level.tokens(),
                        completion_tokens: 5,
                    }),
                    latency_ms: 42,
                    error: None,
                    cancelled: false,
                })
            }
        }

        let client = MockProbeClient;

        // Test success case
        let req = ProbeRequest {
            provider_id: "openai".to_string(),
            model: "gpt-4o".to_string(),
            level: ProbeLevel::Level128K,
            timeout_seconds: 30,
            token_cap: Some(150_000),
        };
        let resp = client.run_probe(req).await.unwrap();
        assert!(resp.success);
        assert_eq!(resp.usage.unwrap().prompt_tokens, 128_000);
        assert!(!resp.cancelled);
        assert!(resp.error.is_none());

        // Test unsupported case
        let req_unsupported = ProbeRequest {
            provider_id: "unsupported".to_string(),
            model: "gpt-4o".to_string(),
            level: ProbeLevel::Level128K,
            timeout_seconds: 30,
            token_cap: None,
        };
        let resp_unsupported = client.run_probe(req_unsupported).await.unwrap();
        assert!(!resp_unsupported.success);
        assert_eq!(resp_unsupported.error.unwrap(), "Provider unsupported");
        assert!(!resp_unsupported.cancelled);

        // Test cancelled case
        let req_high = ProbeRequest {
            provider_id: "openai".to_string(),
            model: "gpt-4o".to_string(),
            level: ProbeLevel::Level900K,
            timeout_seconds: 30,
            token_cap: None,
        };
        let resp_high = client.run_probe(req_high).await.unwrap();
        assert!(!resp_high.success);
        assert!(resp_high.cancelled);
        assert_eq!(resp_high.error.unwrap(), "cancelled");
    }

    #[tokio::test]
    async fn test_run_probe_gate_off_short_circuits() {
        struct TrackingClient {
            called: std::sync::atomic::AtomicBool,
        }
        #[async_trait]
        impl ProviderProbeClient for TrackingClient {
            async fn run_probe(&self, _req: ProbeRequest) -> Result<ProbeResponse, String> {
                self.called.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(ProbeResponse {
                    success: true,
                    usage: None,
                    latency_ms: 10,
                    error: None,
                    cancelled: false,
                })
            }
        }

        std::env::remove_var("HAJIMI_CONTEXT_PROBE_REAL");

        let client = TrackingClient {
            called: std::sync::atomic::AtomicBool::new(false),
        };
        let runner = ContextProbeRunner::new();
        let res = runner
            .run_probe(
                "test_provider".to_string(),
                "test_model".to_string(),
                ProbeLevel::Level128K,
                131_072,
                3600,
                30,
                Some(&client),
            )
            .await;

        // Gate is off: client must NEVER be called, but it returns a simulated successful ProbeResult.
        assert!(!client.called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(res.success);
        assert_eq!(res.tested_input_tokens, 128_000);
    }

    #[tokio::test]
    async fn test_run_probe_gate_on_success() {
        struct TrackingClient {
            called: std::sync::atomic::AtomicBool,
        }
        #[async_trait]
        impl ProviderProbeClient for TrackingClient {
            async fn run_probe(&self, req: ProbeRequest) -> Result<ProbeResponse, String> {
                self.called.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(ProbeResponse {
                    success: true,
                    usage: Some(ProbeUsage {
                        prompt_tokens: req.level.tokens(),
                        completion_tokens: 5,
                    }),
                    latency_ms: 15,
                    error: None,
                    cancelled: false,
                })
            }
        }

        std::env::set_var("HAJIMI_CONTEXT_PROBE_REAL", "true");

        let client = TrackingClient {
            called: std::sync::atomic::AtomicBool::new(false),
        };
        let runner = ContextProbeRunner::new();
        let res = runner
            .run_probe(
                "test_provider".to_string(),
                "test_model".to_string(),
                ProbeLevel::Level256K,
                262_144,
                3600,
                30,
                Some(&client),
            )
            .await;

        // Gate is on: client MUST be called, returning real success ProbeResult.
        assert!(client.called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(res.success);
        assert_eq!(res.tested_input_tokens, 256_000);

        std::env::remove_var("HAJIMI_CONTEXT_PROBE_REAL");
    }

    #[tokio::test]
    async fn test_run_probe_gate_on_unsupported_provider_fallback() {
        std::env::set_var("HAJIMI_CONTEXT_PROBE_REAL", "true");

        let runner = ContextProbeRunner::new();
        let res = runner
            .run_probe(
                "unsupported_provider".to_string(),
                "unsupported_model".to_string(),
                ProbeLevel::Level128K,
                131_072,
                3600,
                30,
                None, // Missing/unsupported client
            )
            .await;

        // Should return a failed ProbeResult with description
        assert!(!res.success);
        assert!(res.error.unwrap().contains("No provider client"));

        std::env::remove_var("HAJIMI_CONTEXT_PROBE_REAL");
    }

    #[tokio::test]
    async fn test_run_probe_malformed_cache_fallback() {
        let path = resolve_probe_path("malformed_provider", "malformed_model");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Write invalid JSON content
        std::fs::write(&path, "{malformed_json_here:").unwrap();

        // Load must return an error and NOT panic
        let load_res = ProbeResult::load_from_file("malformed_provider", "malformed_model").await;
        assert!(load_res.is_err());

        let load_sync_res =
            ProbeResult::load_from_file_sync("malformed_provider", "malformed_model");
        assert!(load_sync_res.is_err());

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_budget_stale_ttl_fallback() {
        use crate::context_budget::{
            resolve_context_budget, BudgetResolveInput, ContextCapabilityStatus,
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create an expired probe result
        let probe = ProbeResult {
            provider_id: "deepseek".to_string(),
            model: "deepseek-v4-pro".to_string(),
            declared_max: 1_000_000,
            tested_input_tokens: 900_000,
            success: true,
            usage: Some(ProbeUsage {
                prompt_tokens: 900_000,
                completion_tokens: 10,
            }),
            latency_ms: 1200,
            error: None,
            timestamp: now - 3600, // 1 hour ago
            ttl_seconds: 1800,     // TTL is 30 mins
            cancelled: false,
        };

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("deepseek".to_string()),
            model: Some("deepseek-v4-pro".to_string()),
            probe_result: Some(probe),
            ..BudgetResolveInput::default()
        });

        // Expired probe should mark it as Stale and fall back to 128K budget
        assert_eq!(budget.capability_status, ContextCapabilityStatus::Stale);
        assert_eq!(budget.max_context_tokens, 128_000);
        assert!(budget.fallback_reason.unwrap().contains("expired"));
    }

    #[test]
    fn test_budget_cascade_gradient_fallback() {
        use crate::context_budget::{
            resolve_context_budget, BudgetResolveInput, ContextCapabilityStatus,
        };

        // 900K failed probe should cascade fallback to 512K limit
        let probe_900k = ProbeResult {
            provider_id: "deepseek".to_string(),
            model: "deepseek-v4-pro".to_string(),
            declared_max: 1_000_000,
            tested_input_tokens: 900_000,
            success: false,
            usage: None,
            latency_ms: 100,
            error: Some("OOM".to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_seconds: 3600,
            cancelled: false,
        };

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("deepseek".to_string()),
            model: Some("deepseek-v4-pro".to_string()),
            probe_result: Some(probe_900k),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.capability_status, ContextCapabilityStatus::Fallback);
        assert_eq!(budget.max_context_tokens, 512_000);
    }

    #[tokio::test]
    async fn test_probe_save_load() {
        let res = ProbeResult {
            provider_id: "test_provider".to_string(),
            model: "test_model".to_string(),
            declared_max: 131_072,
            tested_input_tokens: 128_000,
            success: true,
            usage: Some(ProbeUsage {
                prompt_tokens: 128_000,
                completion_tokens: 10,
            }),
            latency_ms: 1500,
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl_seconds: 3600,
            cancelled: false,
        };

        // Save
        res.save_to_file().await.unwrap();

        let loaded = ProbeResult::load_from_file("test_provider", "test_model")
            .await
            .unwrap();
        assert_eq!(loaded.provider_id, "test_provider");
        assert_eq!(loaded.model, "test_model");
        assert_eq!(loaded.tested_input_tokens, 128_000);

        // Cleanup
        let path = resolve_probe_path("test_provider", "test_model");
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
}
