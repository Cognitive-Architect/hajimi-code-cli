use crate::state::AppState;
use crate::ToolAuthorization;
use crate::ToolInfo;
use crate::ToolResult;
use engine_llm_core::ChatMessage;
use engine_tool_system::ToolRegistry;
use serde_json::Value;
use std::path::{Path, PathBuf};

const PREVIEW_EDIT_MAX_BYTES: u64 = 5 * 1024 * 1024;

#[derive(serde::Deserialize)]
pub struct EditHunkPayload {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

#[tauri::command]
pub async fn list_tools(state: tauri::State<'_, AppState>) -> Result<Vec<ToolInfo>, String> {
    let registry = state.registry.lock().await;
    Ok(registry
        .list()
        .into_iter()
        .filter_map(|name| {
            registry.get(name).map(|t| ToolInfo {
                name: name.to_string(),
                description: t.description().to_string(),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn execute_tool(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    name: String,
    args: Value,
) -> Result<ToolResult, String> {
    let tool = {
        let registry = state.registry.lock().await;
        registry
            .get(&name)
            .ok_or_else(|| format!("tool '{}' not found", name))?
    };
    let permissions = tool.permissions();
    if crate::enforce_tool_permissions(&permissions, &name, &args)?
        == ToolAuthorization::RequireNativeConfirmation
    {
        let approved = crate::confirm_tool_native(&app_handle, &name, &args)?;
        if !approved {
            return Err(format!("tool '{}' denied by user", name));
        }
    }
    let output = tool.execute(args).await.map_err(|e| e.message)?;
    Ok(output.into())
}

#[tauri::command]
pub async fn apply_edits(
    edits: Vec<EditHunkPayload>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<ToolResult>, String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    let registry = state.registry.lock().await;
    apply_edits_with_base_dir(edits, &registry, &base_dir).await
}

pub async fn apply_edits_with_base_dir(
    edits: Vec<EditHunkPayload>,
    registry: &ToolRegistry,
    base_dir: &Path,
) -> Result<Vec<ToolResult>, String> {
    use crate::commands::fs::{resolve_workspace_path, PathIntent};
    let mut results = Vec::new();
    for edit in edits {
        if edit.old_string.is_empty() {
            return Err("old_string cannot be empty".to_string());
        }
        let safe_path = resolve_workspace_path(&edit.path, base_dir, PathIntent::ExistingFile)?;
        let tool = registry
            .get("edit_file")
            .ok_or_else(|| "edit_file tool not found".to_string())?;
        let args = serde_json::json!({
            "path": safe_path,
            "old_string": edit.old_string,
            "new_string": edit.new_string,
        });
        let output = tool.execute(args).await.map_err(|e| e.message)?;
        results.push(output.into());
    }
    Ok(results)
}

#[tauri::command]
pub fn preview_edit(
    path: String,
    old_string: String,
    new_string: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    if old_string.is_empty() {
        return Err("old_string cannot be empty".to_string());
    }
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    preview_edit_with_base_dir(path, old_string, new_string, &base_dir)
}

pub fn preview_edit_with_base_dir(
    path: String,
    old_string: String,
    new_string: String,
    base_dir: &Path,
) -> Result<String, String> {
    use crate::commands::fs::{resolve_workspace_path, PathIntent};
    if old_string.is_empty() {
        return Err("old_string cannot be empty".to_string());
    }
    let safe_path = resolve_workspace_path(&path, base_dir, PathIntent::ExistingFile)?;
    preview_edit_for_path(&safe_path, &path, &old_string, &new_string)
}

pub fn preview_edit_for_path(
    safe_path: &Path,
    display_path: &str,
    old_string: &str,
    new_string: &str,
) -> Result<String, String> {
    if old_string.is_empty() {
        return Err("old_string cannot be empty".to_string());
    }
    let metadata = std::fs::metadata(safe_path).map_err(|e| e.to_string())?;
    if metadata.len() > PREVIEW_EDIT_MAX_BYTES {
        return Err(format!(
            "file too large for preview: {} bytes",
            metadata.len()
        ));
    }
    let content = std::fs::read_to_string(safe_path).map_err(|e| e.to_string())?;
    if !content.contains(old_string) {
        return Err("Old string not found in file".to_string());
    }
    let lines: Vec<&str> = content.lines().collect();
    let old_lines: Vec<&str> = old_string.lines().collect();
    let mut diff = format!("--- {}\n+++ {}\n", display_path, display_path);
    let mut line_no = 1usize;
    for (i, window) in lines.windows(old_lines.len()).enumerate() {
        if window == old_lines.as_slice() {
            line_no = i + 1;
            break;
        }
    }
    diff.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        line_no,
        old_lines.len(),
        line_no,
        new_string.lines().count()
    ));
    for line in old_string.lines() {
        diff.push_str(&format!("-{}\n", line));
    }
    for line in new_string.lines() {
        diff.push_str(&format!("+{}\n", line));
    }
    Ok(diff)
}

#[tauri::command]
pub async fn get_ast_context(symbol_name: String) -> Result<String, String> {
    use agent_core::ASTContextProvider;
    use engine_tool_system::lsp_integration::LspContextProvider;
    let provider = LspContextProvider::new();
    if let Ok(current_dir) = std::env::current_dir() {
        let _ = provider
            .index_project(current_dir.to_string_lossy().as_ref())
            .await;
    }
    match provider.get_symbol_context(&symbol_name, None).await {
        Ok(ctx) => Ok(format!(
            "{} '{}' at {}:{}",
            ctx.symbol.kind, ctx.symbol.name, ctx.symbol.file_path, ctx.symbol.line
        )),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn compact_context(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let gateway = state.memory_gateway.clone();
    gateway.working().compact().await;
    let stats = gateway.stats().await;
    Ok(format!(
        "工作内存: {} 条目, {} tokens",
        stats.working_entries, stats.working_tokens
    ))
}

#[tauri::command]
pub async fn optimize_context(
    messages: Vec<ChatMessage>,
    provider: String,
    config: Option<crate::ProviderConfig>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let client = crate::create_llm_client(&provider, profile.as_deref(), config)?;
    let gateway = state.memory_gateway.clone();
    gateway.optimize(messages, client.as_ref()).await
}

#[tauri::command]
pub async fn probe_provider_context_capacity(
    provider_id: String,
    model: String,
    level: String,
    declared_max: usize,
    confirmed: Option<bool>,
    cancelled: Option<bool>,
) -> Result<Value, String> {
    use agent_core::context_probe::{ContextProbeRunner, ProbeLevel};

    let probe_level = match level.as_str() {
        "256K" => {
            if !confirmed.unwrap_or(false) {
                return Err(
                    "High-cost probe of 256K+ requires explicit user confirmation".to_string(),
                );
            }
            ProbeLevel::Level256K
        }
        "512K" => {
            if !confirmed.unwrap_or(false) {
                return Err(
                    "High-cost probe of 256K+ requires explicit user confirmation".to_string(),
                );
            }
            ProbeLevel::Level512K
        }
        "900K" => {
            if !confirmed.unwrap_or(false) {
                return Err(
                    "High-cost probe of 256K+ requires explicit user confirmation".to_string(),
                );
            }
            ProbeLevel::Level900K
        }
        _ => ProbeLevel::Level128K,
    };

    if cancelled.unwrap_or(false) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let result = agent_core::context_probe::ProbeResult {
            provider_id: provider_id.clone(),
            model: model.clone(),
            declared_max,
            tested_input_tokens: probe_level.tokens(),
            success: false,
            usage: None,
            latency_ms: 0,
            error: Some("cancelled".to_string()),
            timestamp,
            ttl_seconds: 86400,
            cancelled: true,
        };
        if let Err(e) = result.save_to_file().await {
            eprintln!("Failed to save cancelled probe result: {:?}", e);
        }
        return Ok(serde_json::to_value(&result).unwrap_or(serde_json::json!({})));
    }

    let runner = ContextProbeRunner::new();
    let result = runner
        .run_mock_probe(
            provider_id,
            model.clone(),
            probe_level,
            declared_max,
            86400,
            |tokens| {
                if model.contains("fail") {
                    Err("Rate limit exceeded".to_string())
                } else if model.contains("cancel") {
                    Err("cancelled".to_string())
                } else if model.contains("timeout") {
                    Err("Request timeout".to_string())
                } else {
                    use agent_core::context_probe::ProbeUsage;
                    Ok(ProbeUsage {
                        prompt_tokens: tokens,
                        completion_tokens: 12,
                    })
                }
            },
        )
        .await;

    if result.cancelled || !result.success {
        if let Err(e) = result.save_to_file().await {
            eprintln!("Failed to save probe result: {:?}", e);
        }
    }

    let mut returned_val = serde_json::to_value(&result).unwrap_or(serde_json::json!({}));
    if result.success && !result.cancelled {
        if let Some(obj) = returned_val.as_object_mut() {
            obj.insert("status".to_string(), serde_json::json!("MockOnly"));
        }
    }

    Ok(returned_val)
}

#[tauri::command]
pub async fn get_probe_result(provider_id: String, model: String) -> Result<Value, String> {
    use agent_core::context_probe::ProbeResult;
    match ProbeResult::load_from_file(&provider_id, &model).await {
        Ok(probe) => {
            let expired = probe.is_expired();
            Ok(serde_json::json!({
                "providerId": probe.provider_id,
                "model": probe.model,
                "declaredMax": probe.declared_max,
                "testedInputTokens": probe.tested_input_tokens,
                "success": probe.success,
                "usage": probe.usage,
                "latencyMs": probe.latency_ms,
                "error": probe.error,
                "timestamp": probe.timestamp,
                "ttlSeconds": probe.ttl_seconds,
                "cancelled": probe.cancelled,
                "expired": expired,
            }))
        }
        Err(_) => Ok(Value::Null),
    }
}
