use crate::state::{AppState, EditHistoryEntry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust.", name)
}

#[tauri::command]
pub fn get_latest_receipt() -> Result<Value, String> {
    match agent_core::context_receipt::ContextReceipt::load_latest_sync() {
        Some(receipt) => serde_json::to_value(&receipt).map_err(|e| e.to_string()),
        None => Ok(Value::Null),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamDiagnosticEvent {
    pub stage: String,
    pub session_id: Option<String>,
    pub data: Value,
}

#[cfg(feature = "stream-diagnostics")]
fn stream_diag_path() -> PathBuf {
    if let Ok(path) = std::env::var("HAJIMI_STREAM_DIAG_PATH") {
        return PathBuf::from(path);
    }
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::env::temp_dir())
        .join("hajimi-stream-diagnostics.jsonl")
}

#[cfg(feature = "stream-diagnostics")]
fn preview_for_diagnostic(text: &str) -> String {
    text.chars().take(120).collect()
}

#[cfg(feature = "stream-diagnostics")]
fn write_stream_diagnostic(stage: &str, session_id: Option<&str>, data: Value) {
    use std::io::Write;

    let path = stream_diag_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let record = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "stage": stage,
        "sessionId": session_id,
        "data": data,
    });
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "{}", record);
    }
}

#[cfg(not(feature = "stream-diagnostics"))]
fn write_stream_diagnostic(_stage: &str, _session_id: Option<&str>, _data: Value) {}

#[tauri::command]
pub fn record_stream_diagnostic(event: StreamDiagnosticEvent) -> Result<(), String> {
    write_stream_diagnostic(&event.stage, event.session_id.as_deref(), event.data);
    Ok(())
}

#[tauri::command]
pub fn get_stream_diagnostic_info() -> Value {
    #[cfg(feature = "stream-diagnostics")]
    let path = Some(stream_diag_path().display().to_string());
    #[cfg(not(feature = "stream-diagnostics"))]
    let path: Option<String> = None;

    json!({
        "enabled": cfg!(feature = "stream-diagnostics"),
        "path": path,
    })
}

#[tauri::command]
pub fn get_current_workspace(app_handle: tauri::AppHandle) -> Option<String> {
    crate::get_workspace_dir(&app_handle)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_edit_history(state: tauri::State<'_, AppState>) -> Result<Vec<EditHistoryEntry>, String> {
    Ok(state.edit_history.blocking_lock().clone())
}

#[tauri::command]
pub fn get_resource_metrics(state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let hist = state.edit_history.blocking_lock();
    let edit_count = hist.len();
    let applied_count = hist.iter().filter(|e| e.step_type == "EditApplied").count();
    let rejected_count = hist
        .iter()
        .filter(|e| e.step_type == "EditRejected")
        .count();
    Ok(json!({
        "iteration_count": 0,
        "blackboard_size": 0,
        "failure_rate_percent": 0.0,
        "callback_latency_ms": 0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "edit_count": edit_count,
        "applied_count": applied_count,
        "rejected_count": rejected_count,
    }))
}

#[tauri::command]
pub async fn get_cumulative_stats(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let stats = state.token_tracker.get_global_stats().await;

    let mut by_provider = serde_json::Map::new();
    for (k, v) in &stats.by_provider {
        by_provider.insert(
            k.clone(),
            serde_json::json!({
                "prompt_tokens": v.prompt_tokens,
                "completion_tokens": v.completion_tokens,
                "total_tokens": v.total_tokens,
                "request_count": v.request_count
            }),
        );
    }

    let mut by_day = serde_json::Map::new();
    for (k, v) in &stats.by_day {
        by_day.insert(
            k.clone(),
            serde_json::json!({
                "prompt_tokens": v.prompt_tokens,
                "completion_tokens": v.completion_tokens,
                "total_tokens": v.total_tokens,
                "request_count": v.request_count
            }),
        );
    }

    Ok(serde_json::json!({
        "total": {
            "prompt_tokens": stats.total.prompt_tokens,
            "completion_tokens": stats.total.completion_tokens,
            "total_tokens": stats.total.total_tokens,
            "request_count": stats.total.request_count
        },
        "by_provider": by_provider,
        "by_day": by_day
    }))
}

#[tauri::command]
pub fn get_audit_logs(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<crate::audit::KeyUsageRecord>, String> {
    crate::audit::get_logs(limit.unwrap_or(100), offset.unwrap_or(0))
}

