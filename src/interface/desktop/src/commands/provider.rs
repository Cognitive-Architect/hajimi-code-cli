use crate::state::AppState;
use crate::{
    ProviderConfig, ProviderConfigView, ProviderInfo,
};
use engine_llm_core::{openai_chat_completions_url, Client};

#[tauri::command]
pub fn get_provider_configs(
    workspace_path: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<ProviderConfigView>, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let trusted_workspace = crate::trusted_workspace_path(workspace_path.as_deref(), &app_handle)?;
    Ok(
        crate::read_merged_configs(trusted_workspace.as_deref(), profile.as_deref())
            .into_iter()
            .map(|config| {
                let has_api_key = crate::provider_config_has_api_key(&config, profile.as_deref());
                ProviderConfigView::from_config(config, has_api_key)
            })
            .collect(),
    )
}

#[tauri::command]
pub fn add_provider_config(
    mut config: ProviderConfig,
    workspace_path: Option<String>,
    save_target: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let target = save_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        let current = crate::get_workspace_dir(&app_handle)?;
        crate::trusted_workspace_config_path_for_current(workspace_path.as_deref(), &current)?;
        if let Some(api_key) = crate::submitted_api_key(&config.api_key) {
            crate::save_api_key_with_profile(&config.id, api_key, profile.as_deref())?;
        }
        config.api_key.clear();
        return crate::add_workspace_provider_config_for_current(
            config,
            workspace_path.as_deref(),
            &current,
        );
    }
    if let Some(api_key) = crate::submitted_api_key(&config.api_key) {
        crate::save_api_key_with_profile(&config.id, api_key, profile.as_deref())?;
    }
    config.api_key.clear();
    let mut configs = crate::read_provider_configs_with_profile(profile.as_deref());
    if configs.iter().any(|c| c.id == config.id) {
        return Err(format!("Provider '{}' already exists", config.id));
    }
    configs.push(config);
    crate::write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
pub fn update_provider_config(
    mut config: ProviderConfig,
    workspace_path: Option<String>,
    save_target: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let target = save_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        let current = crate::get_workspace_dir(&app_handle)?;
        crate::trusted_workspace_config_path_for_current(workspace_path.as_deref(), &current)?;
        if let Some(api_key) = crate::submitted_api_key(&config.api_key) {
            crate::save_api_key_with_profile(&config.id, api_key, profile.as_deref())?;
        }
        config.api_key.clear();
        return crate::update_workspace_provider_config_for_current(
            config,
            workspace_path.as_deref(),
            &current,
        );
    }
    if let Some(api_key) = crate::submitted_api_key(&config.api_key) {
        crate::save_api_key_with_profile(&config.id, api_key, profile.as_deref())?;
    }
    config.api_key.clear();
    let mut configs = crate::read_provider_configs_with_profile(profile.as_deref());
    let idx = configs
        .iter()
        .position(|c| c.id == config.id)
        .ok_or_else(|| format!("Provider '{}' not found", config.id))?;
    configs[idx] = config;
    crate::write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
pub fn delete_provider_config(
    id: String,
    workspace_path: Option<String>,
    delete_target: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let target = delete_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        let current = crate::get_workspace_dir(&app_handle)?;
        let result =
            crate::delete_workspace_provider_config_for_current(&id, workspace_path.as_deref(), &current);
        if result.is_ok() {
            let _ = crate::delete_api_key_with_profile(&id, profile.as_deref());
        }
        return result;
    }
    let _ = crate::delete_api_key_with_profile(&id, profile.as_deref());
    let mut configs = crate::read_provider_configs_with_profile(profile.as_deref());
    configs.retain(|c| c.id != id);
    crate::write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
pub fn get_providers(
    workspace_path: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<ProviderInfo>, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let mut providers = vec![ProviderInfo {
        name: "ollama".into(),
        available: true,
        default_model: "llama3".into(),
    }];
    let trusted_workspace = crate::trusted_workspace_path(workspace_path.as_deref(), &app_handle)?;
    let configs = crate::read_merged_configs(trusted_workspace.as_deref(), profile.as_deref());
    for cfg in configs {
        if cfg.id != "ollama" {
            let available = crate::get_api_key_with_profile(&cfg.id, profile.as_deref()).is_ok()
                || crate::submitted_api_key(&cfg.api_key).is_some();
            providers.push(ProviderInfo {
                name: cfg.id.clone(),
                available,
                default_model: cfg.model.clone(),
            });
        }
    }
    Ok(providers)
}

#[tauri::command]
pub async fn validate_provider(
    config: ProviderConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let key = if let Some(api_key) = crate::submitted_api_key(&config.api_key) {
        api_key.to_string()
    } else {
        crate::get_api_key_with_profile(&config.id, profile.as_deref())?
    };
    if key.trim().is_empty() {
        return Err("No API key available in keyring or config".to_string());
    }
    let client = Client::new();
    let base = if config.base_url.is_empty() {
        if config.provider_type.contains("anthropic") {
            "https://api.anthropic.com".to_string()
        } else if config.provider_type.contains("openai") {
            "https://api.openai.com".to_string()
        } else {
            return Err(format!(
                "Provider '{}' requires a base_url for type '{}'",
                config.name, config.provider_type
            ));
        }
    } else {
        config.base_url.clone()
    };
    let chat_url = openai_chat_completions_url(&base);
    let test_payload = serde_json::json!({
        "model": config.model.as_str(),
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1
    });
    let req = client
        .post(&chat_url)
        .timeout(std::time::Duration::from_secs(8))
        .header("User-Agent", "hajimi/3.8.0")
        .json(&test_payload);
    let req = if config.provider_type.contains("anthropic") {
        req.header("x-api-key", &key)
            .header("anthropic-version", "2023-06-01")
    } else {
        req.header("Authorization", format!("Bearer {}", key))
    };
    match req.send().await {
        Ok(r) => {
            let status = r.status();
            if status.is_success() {
                Ok(format!("✅ {} 连接测试通过", config.name))
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                Err(format!("API Key 认证失败 (HTTP {})，请检查 Key 是否正确，以及 Key 和 Base URL 是否属于同一平台", status))
            } else if status.as_u16() == 404 {
                Err(format!(
                    "API 端点不存在 (HTTP 404)，请检查 Base URL 是否正确。当前请求地址: {}",
                    chat_url
                ))
            } else if status.as_u16() == 429 {
                Err("请求过于频繁 (HTTP 429)，请稍后再试".to_string())
            } else if status.as_u16() == 400 {
                Ok(format!(
                    "✅ {} 认证通过 (模型名或参数可能需要调整)",
                    config.name
                ))
            } else {
                Err(format!(
                    "测试失败: HTTP {} - {}",
                    status,
                    r.text()
                        .await
                        .unwrap_or_default()
                        .chars()
                        .take(200)
                        .collect::<String>()
                ))
            }
        }
        Err(e) => {
            if key.starts_with("sk-") || key.len() > 15 {
                Ok(format!("⚠️ {} 网络无法到达，Key 格式检查通过", config.name))
            } else {
                Err(format!("连接失败: {}", e))
            }
        }
    }
}

#[tauri::command]
#[allow(deprecated)]
pub fn export_provider_backup(
    password: String,
    workspace_path: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let trusted_workspace = crate::trusted_workspace_path(workspace_path.as_deref(), &app_handle)?;
    let configs = crate::read_merged_configs(trusted_workspace.as_deref(), profile.as_deref());
    let mut export_data = Vec::new();
    for cfg in configs {
        let key = crate::get_api_key_with_profile(&cfg.id, profile.as_deref()).unwrap_or_default();
        export_data.push(serde_json::json!({
            "id": cfg.id,
            "name": cfg.name,
            "provider_type": cfg.provider_type,
            "base_url": cfg.base_url,
            "model": cfg.model,
            "api_key": key,
            "system_prompt": cfg.system_prompt,
            "contextThreshold": cfg.context_threshold,
            "maxContextTokens": cfg.max_context_tokens,
            "maxOutputTokens": cfg.max_output_tokens,
            "reserveOutputTokens": cfg.reserve_output_tokens,
            "safetyMarginTokens": cfg.safety_margin_tokens,
            "retrievalBudgetTokens": cfg.retrieval_budget_tokens,
            "longContextMode": cfg.long_context_mode,
        }));
    }
    let plaintext = serde_json::to_string(&export_data).map_err(|e| e.to_string())?;
    let encrypted = crate::encrypt_backup(&plaintext, &password)?;
    let path = crate::provider_config_path().with_extension("hajimi-backup");
    std::fs::write(&path, encrypted).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
#[allow(deprecated)]
pub fn import_provider_backup(
    password: String,
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let encrypted = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let plaintext = crate::decrypt_backup(&encrypted, &password)?;
    let items: Vec<serde_json::Value> =
        serde_json::from_str(&plaintext).map_err(|e| e.to_string())?;
    let mut count = 0;
    for item in items {
        let cfg = ProviderConfig {
            id: item["id"].as_str().unwrap_or("").to_string(),
            name: item["name"].as_str().unwrap_or("").to_string(),
            provider_type: item["provider_type"]
                .as_str()
                .unwrap_or("openai-compatible")
                .to_string(),
            base_url: item["base_url"].as_str().unwrap_or("").to_string(),
            model: item["model"].as_str().unwrap_or("").to_string(),
            api_key: item["api_key"].as_str().unwrap_or("").to_string(),
            system_prompt: item
                .get("system_prompt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            context_threshold: item
                .get("context_threshold")
                .or_else(|| item.get("contextThreshold"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            max_context_tokens: item
                .get("max_context_tokens")
                .or_else(|| item.get("maxContextTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            max_output_tokens: item
                .get("max_output_tokens")
                .or_else(|| item.get("maxOutputTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            reserve_output_tokens: item
                .get("reserve_output_tokens")
                .or_else(|| item.get("reserveOutputTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            safety_margin_tokens: item
                .get("safety_margin_tokens")
                .or_else(|| item.get("safetyMarginTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            retrieval_budget_tokens: item
                .get("retrieval_budget_tokens")
                .or_else(|| item.get("retrievalBudgetTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            long_context_mode: item
                .get("long_context_mode")
                .or_else(|| item.get("longContextMode"))
                .and_then(|v| v.as_bool()),
        };
        if let Some(api_key) = crate::submitted_api_key(&cfg.api_key) {
            crate::save_api_key_with_profile(&cfg.id, api_key, profile.as_deref())?;
        }
        let mut sanitized = cfg.clone();
        sanitized.api_key.clear();
        let mut existing = crate::read_provider_configs_with_profile(profile.as_deref());
        if let Some(idx) = existing.iter().position(|c| c.id == sanitized.id) {
            existing[idx] = sanitized;
        } else {
            existing.push(sanitized);
        }
        crate::write_provider_configs_with_profile(profile.as_deref(), &existing)?;
        count += 1;
    }
    Ok(count)
}
