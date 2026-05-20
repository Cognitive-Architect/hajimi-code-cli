//! Model-aware context budget core.
//!
//! This module is intentionally pure and synchronous. It defines the budget
//! types that later bridge, provider, memory, and receipt work can reuse.

use std::cmp::min;
use serde::{Serialize, Deserialize};

const LEGACY_MAX_CONTEXT_TOKENS: usize = 8_192;
const LEGACY_RESERVE_OUTPUT_TOKENS: usize = 2_048;
const LEGACY_SAFETY_MARGIN_TOKENS: usize = 512;

const FAST_MAX_CONTEXT_TOKENS: usize = 131_072;
const FAST_RESERVE_OUTPUT_TOKENS: usize = 16_384;
const FAST_SAFETY_MARGIN_TOKENS: usize = 4_096;

const PRO_MAX_CONTEXT_TOKENS: usize = 200_000;
const PRO_RESERVE_OUTPUT_TOKENS: usize = 32_000;
const PRO_SAFETY_MARGIN_TOKENS: usize = 8_000;

const LONG_MAX_CONTEXT_TOKENS: usize = 1_000_000;
const LONG_RESERVE_OUTPUT_TOKENS: usize = 64_000;
const LONG_SAFETY_MARGIN_TOKENS: usize = 32_000;

const RETRIEVAL_BUDGET_PERCENT: usize = 60;
const MAX_RETRIEVAL_BUDGET_TOKENS: usize = 600_000;

/// Named budget profile used to choose safe default capacity assumptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextBudgetMode {
    /// Small local or unknown-model fallback.
    Legacy8K,
    /// Default fast cloud profile.
    Fast128K,
    /// Larger pro profile.
    Pro200K,
    /// Declared long-context profile that still requires provider probing.
    Long1M,
}

/// Capability evidence status before provider probe work lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextCapabilityStatus {
    /// Capability is declared by config, known profile, or env.
    Declared,
    /// Verified by successful, non-expired probe result.
    Verified,
    /// Stale: a previous verification exists but is expired.
    Stale,
    /// Runtime selected a lower fallback budget.
    Fallback,
    /// Cancelled: user cancelled the probe.
    Cancelled,
}

/// Provider/model capability inputs used to calculate a request budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelContextCaps {
    pub provider_id: String,
    pub model: String,
    pub mode: ContextBudgetMode,
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
    pub default_reserve_output_tokens: usize,
    pub default_safety_margin_tokens: usize,
    pub supports_prompt_cache: bool,
    pub long_context_mode: bool,
    pub requires_probe: bool,
    pub capability_status: ContextCapabilityStatus,
}

impl ModelContextCaps {
    /// Small local or unknown-model fallback profile.
    pub fn legacy_8k() -> Self {
        Self {
            provider_id: "local".to_string(),
            model: "legacy-8k".to_string(),
            mode: ContextBudgetMode::Legacy8K,
            max_context_tokens: LEGACY_MAX_CONTEXT_TOKENS,
            max_output_tokens: LEGACY_RESERVE_OUTPUT_TOKENS,
            default_reserve_output_tokens: LEGACY_RESERVE_OUTPUT_TOKENS,
            default_safety_margin_tokens: LEGACY_SAFETY_MARGIN_TOKENS,
            supports_prompt_cache: false,
            long_context_mode: false,
            requires_probe: false,
            capability_status: ContextCapabilityStatus::Fallback,
        }
    }

    /// Default fast cloud profile.
    pub fn fast_128k() -> Self {
        Self {
            provider_id: "generic".to_string(),
            model: "fast-128k".to_string(),
            mode: ContextBudgetMode::Fast128K,
            max_context_tokens: FAST_MAX_CONTEXT_TOKENS,
            max_output_tokens: FAST_RESERVE_OUTPUT_TOKENS,
            default_reserve_output_tokens: FAST_RESERVE_OUTPUT_TOKENS,
            default_safety_margin_tokens: FAST_SAFETY_MARGIN_TOKENS,
            supports_prompt_cache: false,
            long_context_mode: false,
            requires_probe: false,
            capability_status: ContextCapabilityStatus::Declared,
        }
    }

    /// Larger pro cloud profile.
    pub fn pro_200k() -> Self {
        Self {
            provider_id: "generic".to_string(),
            model: "pro-200k".to_string(),
            mode: ContextBudgetMode::Pro200K,
            max_context_tokens: PRO_MAX_CONTEXT_TOKENS,
            max_output_tokens: PRO_RESERVE_OUTPUT_TOKENS,
            default_reserve_output_tokens: PRO_RESERVE_OUTPUT_TOKENS,
            default_safety_margin_tokens: PRO_SAFETY_MARGIN_TOKENS,
            supports_prompt_cache: false,
            long_context_mode: true,
            requires_probe: false,
            capability_status: ContextCapabilityStatus::Declared,
        }
    }

    /// Declared 1M profile. Probe handling is intentionally left for later days.
    pub fn long_1m() -> Self {
        Self {
            provider_id: "deepseek".to_string(),
            model: "deepseek-v4-pro".to_string(),
            mode: ContextBudgetMode::Long1M,
            max_context_tokens: LONG_MAX_CONTEXT_TOKENS,
            max_output_tokens: LONG_RESERVE_OUTPUT_TOKENS,
            default_reserve_output_tokens: LONG_RESERVE_OUTPUT_TOKENS,
            default_safety_margin_tokens: LONG_SAFETY_MARGIN_TOKENS,
            supports_prompt_cache: false,
            long_context_mode: true,
            requires_probe: true,
            capability_status: ContextCapabilityStatus::Declared,
        }
    }
}

/// Neutral provider capability DTO owned by agent-core.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderContextCaps {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub max_context_tokens: Option<usize>,
    pub max_output_tokens: Option<usize>,
    pub reserve_output_tokens: Option<usize>,
    pub safety_margin_tokens: Option<usize>,
    pub retrieval_budget_tokens: Option<usize>,
    pub long_context_mode: Option<bool>,
    /// Deprecated compatibility with old `context_threshold` / `contextThreshold`.
    pub context_threshold: Option<usize>,
}

/// Optional caller overrides for pure budget calculation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextBudgetOverrides {
    pub reserve_output_tokens: Option<usize>,
    pub safety_margin_tokens: Option<usize>,
    pub retrieval_budget_tokens: Option<usize>,
}

/// Calculated budget for one request assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBudget {
    pub provider_id: String,
    pub model: String,
    pub mode: ContextBudgetMode,
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
    pub reserve_output_tokens: usize,
    pub safety_margin_tokens: usize,
    pub input_budget: usize,
    pub retrieval_budget: usize,
    pub supports_prompt_cache: bool,
    pub long_context_mode: bool,
    pub requires_probe: bool,
    pub capability_status: ContextCapabilityStatus,
    pub fallback_reason: Option<String>,
}

/// Resolver input assembled by bridge or higher-level orchestration later.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BudgetResolveInput {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub provider_caps: Option<ProviderContextCaps>,
    pub primitive_caps: Option<ProviderContextCaps>,
    pub context_threshold: Option<usize>,
}

/// Calculate a request budget from provider/model capability data.
pub fn calculate_budget(
    caps: &ModelContextCaps,
    overrides: ContextBudgetOverrides,
) -> ContextBudget {
    let reserve_output_tokens = overrides
        .reserve_output_tokens
        .unwrap_or(caps.default_reserve_output_tokens);
    let safety_margin_tokens = overrides
        .safety_margin_tokens
        .unwrap_or(caps.default_safety_margin_tokens);

    let Some(after_reserve) = caps.max_context_tokens.checked_sub(reserve_output_tokens) else {
        return fallback_budget(caps, "reserve_output_exceeds_context");
    };
    let Some(input_budget) = after_reserve.checked_sub(safety_margin_tokens) else {
        return fallback_budget(caps, "safety_margin_exceeds_context");
    };

    if input_budget == 0 {
        return fallback_budget(caps, "zero_input_budget");
    }

    let calculated_retrieval_budget = retrieval_budget_for(input_budget);
    let retrieval_budget = overrides
        .retrieval_budget_tokens
        .map(|value| min(value, MAX_RETRIEVAL_BUDGET_TOKENS))
        .unwrap_or(calculated_retrieval_budget);

    ContextBudget {
        provider_id: caps.provider_id.clone(),
        model: caps.model.clone(),
        mode: caps.mode,
        max_context_tokens: caps.max_context_tokens,
        max_output_tokens: caps.max_output_tokens,
        reserve_output_tokens,
        safety_margin_tokens,
        input_budget,
        retrieval_budget,
        supports_prompt_cache: caps.supports_prompt_cache,
        long_context_mode: caps.long_context_mode,
        requires_probe: caps.requires_probe,
        capability_status: caps.capability_status,
        fallback_reason: None,
    }
}

/// Resolve model-aware context budget from neutral capability inputs and env.
pub fn resolve_context_budget(input: BudgetResolveInput) -> ContextBudget {
    let provider_id = input
        .provider_id
        .clone()
        .or_else(|| {
            input
                .provider_caps
                .as_ref()
                .and_then(|caps| caps.provider_id.clone())
        })
        .or_else(|| {
            input
                .primitive_caps
                .as_ref()
                .and_then(|caps| caps.provider_id.clone())
        })
        .unwrap_or_else(|| "generic".to_string());
    let model = input
        .model
        .clone()
        .or_else(|| {
            input
                .provider_caps
                .as_ref()
                .and_then(|caps| caps.model.clone())
        })
        .or_else(|| {
            input
                .primitive_caps
                .as_ref()
                .and_then(|caps| caps.model.clone())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut caps = if let Some(provider_caps) = input.provider_caps.as_ref() {
        caps_from_provider_context(provider_caps, &provider_id, &model)
    } else if let Some(primitive_caps) = input.primitive_caps.as_ref() {
        caps_from_provider_context(primitive_caps, &provider_id, &model)
    } else if let Some(limit) = input.context_threshold {
        caps_from_limit(
            &provider_id,
            &model,
            limit,
            ContextCapabilityStatus::Declared,
        )
    } else if let Some(known) = known_model_caps(&provider_id, &model) {
        known
    } else if let Some(limit) = read_env_usize("HAJIMI_CONTEXT_LIMIT") {
        caps_from_limit(
            &provider_id,
            &model,
            limit,
            ContextCapabilityStatus::Declared,
        )
    } else if is_local_model(&provider_id, &model) {
        let mut fallback = ModelContextCaps::legacy_8k();
        fallback.provider_id = provider_id.clone();
        fallback.model = model.clone();
        fallback
    } else {
        let mut fallback = ModelContextCaps::legacy_8k();
        fallback.provider_id = provider_id.clone();
        fallback.model = model.clone();
        fallback
    };

    apply_env_context_limit(&mut caps);
    apply_long_context_gate(&mut caps);

    let mut fallback_reason = None;
    if let Ok(probe) = crate::context_probe::ProbeResult::load_from_file_sync(&provider_id, &model) {
        if probe.cancelled {
            caps.capability_status = ContextCapabilityStatus::Cancelled;
            fallback_reason = Some("Probe cancelled by user".to_string());
            caps.max_context_tokens = min(caps.max_context_tokens, 8_192);
        } else if probe.success {
            if probe.is_expired() {
                caps.capability_status = ContextCapabilityStatus::Stale;
                fallback_reason = Some("Probe result expired (Stale)".to_string());
                caps.max_context_tokens = min(caps.max_context_tokens, 128_000);
            } else {
                caps.capability_status = ContextCapabilityStatus::Verified;
                caps.max_context_tokens = probe.tested_input_tokens;
            }
        } else {
            caps.capability_status = ContextCapabilityStatus::Fallback;
            fallback_reason = Some(format!("Probe failed: {:?}", probe.error));
            // fallback rule: 900K fail -> 512K -> 256K -> 128K -> 32K/8K
            let failed_level = probe.tested_input_tokens;
            if failed_level >= 900_000 {
                caps.max_context_tokens = 512_000;
            } else if failed_level >= 512_000 {
                caps.max_context_tokens = 256_000;
            } else if failed_level >= 256_000 {
                caps.max_context_tokens = 128_000;
            } else if failed_level >= 128_000 {
                caps.max_context_tokens = 32_000;
            } else {
                caps.max_context_tokens = 8_192;
            }
        }
    }

    let overrides = ContextBudgetOverrides {
        reserve_output_tokens: read_env_usize("HAJIMI_CONTEXT_RESERVE_OUTPUT"),
        safety_margin_tokens: None,
        retrieval_budget_tokens: None,
    };

    let mut budget = calculate_budget(&caps, overrides);
    if fallback_reason.is_some() {
        budget.fallback_reason = fallback_reason;
    }
    budget
}

/// Known provider/model capability defaults.
pub fn known_model_caps(provider_id: &str, model: &str) -> Option<ModelContextCaps> {
    let provider = provider_id.to_ascii_lowercase();
    let model_lower = model.to_ascii_lowercase();

    if provider.contains("deepseek") && model_lower.contains("deepseek-v4-pro") {
        return Some(ModelContextCaps::long_1m());
    }

    if model_lower.contains("200k") || model_lower.contains("claude-3-5") {
        let mut caps = ModelContextCaps::pro_200k();
        caps.provider_id = provider_id.to_string();
        caps.model = model.to_string();
        return Some(caps);
    }

    if model_lower.contains("128k")
        || model_lower.contains("gpt-4")
        || model_lower.contains("gpt-4o")
    {
        let mut caps = ModelContextCaps::fast_128k();
        caps.provider_id = provider_id.to_string();
        caps.model = model.to_string();
        return Some(caps);
    }

    None
}

/// Safe default for unknown provider/model inputs.
pub fn unknown_model_budget(provider_id: &str, model: &str) -> ContextBudget {
    let mut caps = ModelContextCaps::legacy_8k();
    caps.provider_id = provider_id.to_string();
    caps.model = model.to_string();
    calculate_budget(&caps, ContextBudgetOverrides::default())
}

/// Small local or unknown-model fallback budget.
pub fn legacy_8k() -> ContextBudget {
    calculate_budget(
        &ModelContextCaps::legacy_8k(),
        ContextBudgetOverrides::default(),
    )
}

/// Default fast cloud budget.
pub fn fast_128k() -> ContextBudget {
    calculate_budget(
        &ModelContextCaps::fast_128k(),
        ContextBudgetOverrides::default(),
    )
}

/// Larger pro cloud budget.
pub fn pro_200k() -> ContextBudget {
    calculate_budget(
        &ModelContextCaps::pro_200k(),
        ContextBudgetOverrides::default(),
    )
}

/// Declared 1M long-context budget.
pub fn long_1m() -> ContextBudget {
    calculate_budget(
        &ModelContextCaps::long_1m(),
        ContextBudgetOverrides::default(),
    )
}

fn retrieval_budget_for(input_budget: usize) -> usize {
    min(
        input_budget.saturating_mul(RETRIEVAL_BUDGET_PERCENT) / 100,
        MAX_RETRIEVAL_BUDGET_TOKENS,
    )
}

fn fallback_budget(caps: &ModelContextCaps, reason: &str) -> ContextBudget {
    let fallback = legacy_8k();
    ContextBudget {
        provider_id: caps.provider_id.clone(),
        model: caps.model.clone(),
        capability_status: ContextCapabilityStatus::Fallback,
        fallback_reason: Some(reason.to_string()),
        ..fallback
    }
}

fn caps_from_provider_context(
    provider_caps: &ProviderContextCaps,
    default_provider_id: &str,
    default_model: &str,
) -> ModelContextCaps {
    let provider_id = provider_caps
        .provider_id
        .clone()
        .unwrap_or_else(|| default_provider_id.to_string());
    let model = provider_caps
        .model
        .clone()
        .unwrap_or_else(|| default_model.to_string());
    let max_context_tokens = provider_caps
        .max_context_tokens
        .or(provider_caps.context_threshold)
        .unwrap_or(FAST_MAX_CONTEXT_TOKENS);
    let long_context_mode = provider_caps
        .long_context_mode
        .unwrap_or(max_context_tokens >= PRO_MAX_CONTEXT_TOKENS);

    ModelContextCaps {
        provider_id,
        model,
        mode: mode_for_limit(max_context_tokens),
        max_context_tokens,
        max_output_tokens: provider_caps
            .max_output_tokens
            .unwrap_or_else(|| default_output_for_limit(max_context_tokens)),
        default_reserve_output_tokens: provider_caps
            .reserve_output_tokens
            .unwrap_or_else(|| default_reserve_for_limit(max_context_tokens)),
        default_safety_margin_tokens: provider_caps
            .safety_margin_tokens
            .unwrap_or_else(|| default_margin_for_limit(max_context_tokens)),
        supports_prompt_cache: false,
        long_context_mode,
        requires_probe: max_context_tokens >= LONG_MAX_CONTEXT_TOKENS,
        capability_status: ContextCapabilityStatus::Declared,
    }
}

fn caps_from_limit(
    provider_id: &str,
    model: &str,
    max_context_tokens: usize,
    capability_status: ContextCapabilityStatus,
) -> ModelContextCaps {
    ModelContextCaps {
        provider_id: provider_id.to_string(),
        model: model.to_string(),
        mode: mode_for_limit(max_context_tokens),
        max_context_tokens,
        max_output_tokens: default_output_for_limit(max_context_tokens),
        default_reserve_output_tokens: default_reserve_for_limit(max_context_tokens),
        default_safety_margin_tokens: default_margin_for_limit(max_context_tokens),
        supports_prompt_cache: false,
        long_context_mode: max_context_tokens >= PRO_MAX_CONTEXT_TOKENS,
        requires_probe: max_context_tokens >= LONG_MAX_CONTEXT_TOKENS,
        capability_status,
    }
}

fn apply_env_context_limit(caps: &mut ModelContextCaps) {
    if let Some(limit) = read_env_usize("HAJIMI_CONTEXT_LIMIT") {
        *caps = caps_from_limit(
            &caps.provider_id,
            &caps.model,
            limit,
            ContextCapabilityStatus::Declared,
        );
    }
}

fn apply_long_context_gate(caps: &mut ModelContextCaps) {
    if crate::prompts::is_long_context_enabled() || !caps.long_context_mode {
        return;
    }

    let provider_id = caps.provider_id.clone();
    let model = caps.model.clone();
    *caps = ModelContextCaps::fast_128k();
    caps.provider_id = provider_id;
    caps.model = model;
    caps.capability_status = ContextCapabilityStatus::Fallback;
}

fn read_env_usize(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
}

fn is_local_model(provider_id: &str, model: &str) -> bool {
    let provider = provider_id.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    provider.contains("local") || provider.contains("ollama") || model.contains("local")
}

fn mode_for_limit(max_context_tokens: usize) -> ContextBudgetMode {
    if max_context_tokens >= LONG_MAX_CONTEXT_TOKENS {
        ContextBudgetMode::Long1M
    } else if max_context_tokens >= PRO_MAX_CONTEXT_TOKENS {
        ContextBudgetMode::Pro200K
    } else if max_context_tokens >= FAST_MAX_CONTEXT_TOKENS {
        ContextBudgetMode::Fast128K
    } else {
        ContextBudgetMode::Legacy8K
    }
}

fn default_output_for_limit(max_context_tokens: usize) -> usize {
    default_reserve_for_limit(max_context_tokens)
}

fn default_reserve_for_limit(max_context_tokens: usize) -> usize {
    match mode_for_limit(max_context_tokens) {
        ContextBudgetMode::Long1M => LONG_RESERVE_OUTPUT_TOKENS,
        ContextBudgetMode::Pro200K => PRO_RESERVE_OUTPUT_TOKENS,
        ContextBudgetMode::Fast128K => FAST_RESERVE_OUTPUT_TOKENS,
        ContextBudgetMode::Legacy8K => min(LEGACY_RESERVE_OUTPUT_TOKENS, max_context_tokens / 4),
    }
}

fn default_margin_for_limit(max_context_tokens: usize) -> usize {
    match mode_for_limit(max_context_tokens) {
        ContextBudgetMode::Long1M => LONG_SAFETY_MARGIN_TOKENS,
        ContextBudgetMode::Pro200K => PRO_SAFETY_MARGIN_TOKENS,
        ContextBudgetMode::Fast128K => FAST_SAFETY_MARGIN_TOKENS,
        ContextBudgetMode::Legacy8K => min(LEGACY_SAFETY_MARGIN_TOKENS, max_context_tokens / 16),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn context_budget_legacy_8k_profile_uses_safe_input_formula() {
        let budget = legacy_8k();

        assert_eq!(budget.mode, ContextBudgetMode::Legacy8K);
        assert_eq!(budget.max_context_tokens, 8_192);
        assert_eq!(budget.reserve_output_tokens, 2_048);
        assert_eq!(budget.safety_margin_tokens, 512);
        assert_eq!(budget.input_budget, 5_632);
        assert_eq!(budget.retrieval_budget, 3_379);
    }

    #[test]
    fn context_budget_fast_128k_profile_uses_default_formula() {
        let budget = fast_128k();

        assert_eq!(budget.mode, ContextBudgetMode::Fast128K);
        assert_eq!(budget.max_context_tokens, 131_072);
        assert_eq!(budget.reserve_output_tokens, 16_384);
        assert_eq!(budget.safety_margin_tokens, 4_096);
        assert_eq!(budget.input_budget, 110_592);
        assert_eq!(budget.retrieval_budget, 66_355);
    }

    #[test]
    fn context_budget_pro_200k_profile_uses_default_formula() {
        let budget = pro_200k();

        assert_eq!(budget.mode, ContextBudgetMode::Pro200K);
        assert_eq!(budget.max_context_tokens, 200_000);
        assert_eq!(budget.reserve_output_tokens, 32_000);
        assert_eq!(budget.safety_margin_tokens, 8_000);
        assert_eq!(budget.input_budget, 160_000);
        assert_eq!(budget.retrieval_budget, 96_000);
    }

    #[test]
    fn context_budget_long_1m_profile_uses_declared_formula() {
        let budget = long_1m();

        assert_eq!(budget.mode, ContextBudgetMode::Long1M);
        assert_eq!(budget.max_context_tokens, 1_000_000);
        assert_eq!(budget.reserve_output_tokens, 64_000);
        assert_eq!(budget.safety_margin_tokens, 32_000);
        assert_eq!(budget.input_budget, 904_000);
        assert_eq!(budget.retrieval_budget, 542_400);
        assert!(budget.long_context_mode);
        assert!(budget.requires_probe);
    }

    #[test]
    fn context_budget_retrieval_budget_is_capped_at_600k() {
        let mut caps = ModelContextCaps::long_1m();
        caps.max_context_tokens = 2_000_000;
        caps.default_reserve_output_tokens = 64_000;
        caps.default_safety_margin_tokens = 32_000;

        let budget = calculate_budget(&caps, ContextBudgetOverrides::default());

        assert_eq!(budget.input_budget, 1_904_000);
        assert_eq!(budget.retrieval_budget, 600_000);
    }

    #[test]
    fn context_budget_overrides_are_applied_safely() {
        let budget = calculate_budget(
            &ModelContextCaps::fast_128k(),
            ContextBudgetOverrides {
                reserve_output_tokens: Some(8_000),
                safety_margin_tokens: Some(2_000),
                retrieval_budget_tokens: Some(700_000),
            },
        );

        assert_eq!(budget.input_budget, 121_072);
        assert_eq!(budget.retrieval_budget, 600_000);
        assert_eq!(budget.fallback_reason, None);
    }

    #[test]
    fn context_budget_invalid_reserve_falls_back_without_panic() {
        let caps = ModelContextCaps {
            provider_id: "bad-provider".to_string(),
            model: "bad-model".to_string(),
            mode: ContextBudgetMode::Pro200K,
            max_context_tokens: 1_000,
            max_output_tokens: 2_000,
            default_reserve_output_tokens: 2_000,
            default_safety_margin_tokens: 100,
            supports_prompt_cache: false,
            long_context_mode: false,
            requires_probe: false,
            capability_status: ContextCapabilityStatus::Declared,
        };

        let budget = calculate_budget(&caps, ContextBudgetOverrides::default());

        assert_eq!(budget.mode, ContextBudgetMode::Legacy8K);
        assert_eq!(budget.provider_id, "bad-provider");
        assert_eq!(budget.model, "bad-model");
        assert_eq!(
            budget.fallback_reason.as_deref(),
            Some("reserve_output_exceeds_context")
        );
    }

    #[test]
    fn context_budget_invalid_safety_margin_falls_back_without_panic() {
        let caps = ModelContextCaps {
            provider_id: "small-provider".to_string(),
            model: "small-model".to_string(),
            mode: ContextBudgetMode::Fast128K,
            max_context_tokens: 3_000,
            max_output_tokens: 1_000,
            default_reserve_output_tokens: 2_000,
            default_safety_margin_tokens: 2_000,
            supports_prompt_cache: false,
            long_context_mode: false,
            requires_probe: false,
            capability_status: ContextCapabilityStatus::Declared,
        };

        let budget = calculate_budget(&caps, ContextBudgetOverrides::default());

        assert_eq!(budget.mode, ContextBudgetMode::Legacy8K);
        assert_eq!(
            budget.fallback_reason.as_deref(),
            Some("safety_margin_exceeds_context")
        );
    }

    #[test]
    fn context_budget_unknown_model_uses_legacy_fallback() {
        let budget = unknown_model_budget("custom", "unknown-model");

        assert_eq!(budget.mode, ContextBudgetMode::Legacy8K);
        assert_eq!(budget.provider_id, "custom");
        assert_eq!(budget.model, "unknown-model");
        assert_eq!(budget.input_budget, 5_632);
        assert_eq!(budget.capability_status, ContextCapabilityStatus::Fallback);
    }

    #[test]
    fn context_budget_known_model_caps_resolves_deepseek_declared_1m() {
        let caps = known_model_caps("deepseek", "deepseek-v4-pro").expect("known caps");

        assert_eq!(caps.mode, ContextBudgetMode::Long1M);
        assert_eq!(caps.max_context_tokens, 1_000_000);
        assert!(caps.long_context_mode);
        assert!(caps.requires_probe);
        assert_eq!(caps.capability_status, ContextCapabilityStatus::Declared);
    }

    #[test]
    fn context_budget_resolver_prefers_provider_caps_over_known_model() {
        let _guard = env_guard();
        clear_context_env();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("deepseek".to_string()),
            model: Some("deepseek-v4-pro".to_string()),
            provider_caps: Some(ProviderContextCaps {
                max_context_tokens: Some(200_000),
                reserve_output_tokens: Some(20_000),
                safety_margin_tokens: Some(5_000),
                ..ProviderContextCaps::default()
            }),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Pro200K);
        assert_eq!(budget.max_context_tokens, 200_000);
        assert_eq!(budget.input_budget, 175_000);
        assert_eq!(budget.capability_status, ContextCapabilityStatus::Declared);
    }

    #[test]
    fn context_budget_resolver_uses_primitive_caps_before_old_field() {
        let _guard = env_guard();
        clear_context_env();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("generic".to_string()),
            model: Some("custom".to_string()),
            primitive_caps: Some(ProviderContextCaps {
                max_context_tokens: Some(131_072),
                ..ProviderContextCaps::default()
            }),
            context_threshold: Some(200_000),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Fast128K);
        assert_eq!(budget.max_context_tokens, 131_072);
    }

    #[test]
    fn context_budget_resolver_supports_context_threshold_compatibility() {
        let _guard = env_guard();
        clear_context_env();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("legacy".to_string()),
            model: Some("legacy-config".to_string()),
            provider_caps: Some(ProviderContextCaps {
                context_threshold: Some(200_000),
                ..ProviderContextCaps::default()
            }),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Pro200K);
        assert_eq!(budget.max_context_tokens, 200_000);
        assert_eq!(budget.input_budget, 160_000);
    }

    #[test]
    fn context_budget_resolver_env_limit_can_override_known_caps() {
        let _guard = env_guard();
        clear_context_env();
        std::env::set_var("HAJIMI_CONTEXT_LIMIT", "200000");

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("deepseek".to_string()),
            model: Some("deepseek-v4-pro".to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Pro200K);
        assert_eq!(budget.max_context_tokens, 200_000);
        clear_context_env();
    }

    #[test]
    fn context_budget_resolver_env_limit_covers_unknown_model() {
        let _guard = env_guard();
        clear_context_env();
        std::env::set_var("HAJIMI_CONTEXT_LIMIT", "1000000");

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("custom".to_string()),
            model: Some("unknown".to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Long1M);
        assert_eq!(budget.max_context_tokens, 1_000_000);
        assert_eq!(budget.input_budget, 904_000);
        clear_context_env();
    }

    #[test]
    fn context_budget_resolver_env_reserve_output_override_is_applied() {
        let _guard = env_guard();
        clear_context_env();
        std::env::set_var("HAJIMI_CONTEXT_RESERVE_OUTPUT", "8000");

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("generic".to_string()),
            model: Some("gpt-4o".to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.max_context_tokens, 131_072);
        assert_eq!(budget.reserve_output_tokens, 8_000);
        assert_eq!(budget.input_budget, 118_976);
        clear_context_env();
    }

    #[test]
    fn context_budget_resolver_invalid_env_does_not_panic() {
        let _guard = env_guard();
        clear_context_env();
        std::env::set_var("HAJIMI_CONTEXT_LIMIT", "not-a-number");
        std::env::set_var("HAJIMI_CONTEXT_RESERVE_OUTPUT", "also-bad");

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("custom".to_string()),
            model: Some("unknown".to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Legacy8K);
        assert_eq!(budget.input_budget, 5_632);
        assert_eq!(budget.fallback_reason, None);
        clear_context_env();
    }

    #[test]
    fn context_budget_resolver_long_disabled_falls_back_to_fast_budget() {
        let _guard = env_guard();
        clear_context_env();
        std::env::set_var("HAJIMI_LONG_CONTEXT_ENABLED", "false");

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("deepseek".to_string()),
            model: Some("deepseek-v4-pro".to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Fast128K);
        assert_eq!(budget.max_context_tokens, 131_072);
        assert!(!budget.long_context_mode);
        assert_eq!(budget.capability_status, ContextCapabilityStatus::Fallback);
        clear_context_env();
    }

    #[test]
    fn context_budget_resolver_local_unknown_uses_legacy_fallback() {
        let _guard = env_guard();
        clear_context_env();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some("ollama".to_string()),
            model: Some("local-model".to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.mode, ContextBudgetMode::Legacy8K);
        assert_eq!(budget.capability_status, ContextCapabilityStatus::Fallback);
    }

    #[test]
    fn test_context_budget_probe_integration_success() {
        let _guard = env_guard();
        clear_context_env();
        
        let provider_id = "test_budget_prov";
        let model = "test_budget_model";
        
        // Write a successful, non-expired probe result
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let res = crate::context_probe::ProbeResult {
            provider_id: provider_id.to_string(),
            model: model.to_string(),
            declared_max: 200_000,
            tested_input_tokens: 150_000,
            success: true,
            usage: None,
            latency_ms: 1000,
            error: None,
            timestamp: now,
            ttl_seconds: 3600,
            cancelled: false,
        };
        res.save_to_file_sync().unwrap();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some(provider_id.to_string()),
            model: Some(model.to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.capability_status, ContextCapabilityStatus::Verified);
        assert_eq!(budget.max_context_tokens, 150_000);

        let path = crate::context_probe::resolve_probe_path(provider_id, model);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_context_budget_probe_integration_expired_stale() {
        let _guard = env_guard();
        clear_context_env();
        
        let provider_id = "test_budget_prov_stale";
        let model = "test_budget_model_stale";
        
        // Write an expired probe result
        let res = crate::context_probe::ProbeResult {
            provider_id: provider_id.to_string(),
            model: model.to_string(),
            declared_max: 200_000,
            tested_input_tokens: 150_000,
            success: true,
            usage: None,
            latency_ms: 1000,
            error: None,
            timestamp: 1000, // old
            ttl_seconds: 10,
            cancelled: false,
        };
        res.save_to_file_sync().unwrap();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some(provider_id.to_string()),
            model: Some(model.to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.capability_status, ContextCapabilityStatus::Stale);
        // Stale does not drive Verified budget, should fall back below 150,000 to safe defaults
        assert!(budget.max_context_tokens <= 128_000);

        let path = crate::context_probe::resolve_probe_path(provider_id, model);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_context_budget_probe_integration_failed_fallback() {
        let _guard = env_guard();
        clear_context_env();
        
        let provider_id = "test_budget_prov_fail";
        let model = "test_budget_model_fail";
        
        // Write a failed probe result at 900K level
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let res = crate::context_probe::ProbeResult {
            provider_id: provider_id.to_string(),
            model: model.to_string(),
            declared_max: 1_000_000,
            tested_input_tokens: 900_000,
            success: false,
            usage: None,
            latency_ms: 1000,
            error: Some("Rate limit".to_string()),
            timestamp: now,
            ttl_seconds: 3600,
            cancelled: false,
        };
        res.save_to_file_sync().unwrap();

        let budget = resolve_context_budget(BudgetResolveInput {
            provider_id: Some(provider_id.to_string()),
            model: Some(model.to_string()),
            ..BudgetResolveInput::default()
        });

        assert_eq!(budget.capability_status, ContextCapabilityStatus::Fallback);
        // 900K fail should fall back to 512K
        assert_eq!(budget.max_context_tokens, 512_000);

        let path = crate::context_probe::resolve_probe_path(provider_id, model);
        let _ = std::fs::remove_file(path);
    }

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().expect("env lock")
    }

    fn clear_context_env() {
        std::env::remove_var("HAJIMI_CONTEXT_LIMIT");
        std::env::remove_var("HAJIMI_CONTEXT_RESERVE_OUTPUT");
        std::env::remove_var("HAJIMI_LONG_CONTEXT_ENABLED");
    }
}
