#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use async_trait::async_trait;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use agent_core::agent_loop::TraceStepType;
use agent_core::{
    AgentContext, AgentLoopBuilder, AutonomousReflector, HierarchicalPlanner, TraceEvent,
};
use codex_twist::memory::{MemoryGateway, MemoryTier, TokenBudget, TokenUsageTracker};
use engine_llm_core::{
    openai_chat_completions_url, AnthropicClient, ChatMessage, Client, LlmClient, OllamaClient,
    OpenAiClient,
};
use engine_tool_system::lsp_integration::ASTContextProvider;
use engine_tool_system::PermissionLevel;
use engine_tool_system::{ToolOutput, ToolPermissions, ToolRegistry};
use keyring::Entry;
use memory::memory_gateway::MemoryGateway as AgentMemoryGateway;
use pbkdf2::pbkdf2_hmac;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{ipc::Channel, Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use crate::commands::fs::{
    resolve_workspace_path, PathIntent, create_workspace_dir, rename_workspace_path, remove_workspace_path
};

mod audit;

mod commands;
mod error;
mod registry;
mod startup;
mod state;

use registry::build_registry;
use startup::{DesktopAgentTurnDriver, UiBridgeGovernance};
use state::{
    AppState, CheckpointCompareResult, CheckpointDiffSummary, CheckpointExportBundle,
    CheckpointFileChange, CheckpointFileRef, CheckpointMetadata, CheckpointRecord,
    EditHistoryEntry, RestoreFilePlan, RestoreResult,
};

// ------------------------------------------------------------------
// Security constants (B-01/04, B-02/04)
// ------------------------------------------------------------------
const PREVIEW_EDIT_MAX_BYTES: u64 = 5 * 1024 * 1024;

// ------------------------------------------------------------------
// Legacy commands
// ------------------------------------------------------------------
// greet moved to commands::info

// probe_provider_context_capacity and get_probe_result moved to commands::tool

// get_latest_receipt moved to commands::info

/// 获取应用工作目录沙箱根路径
fn get_workspace_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app_handle
        .path()
        .document_dir()
        .map_err(|e| format!("无法获取文档目录: {}", e))?;
    let workspace = base.join("hajimi-workspace");
    std::fs::create_dir_all(&workspace).map_err(|e| e.to_string())?;
    Ok(workspace)
}

pub(crate) fn checkpoint_store_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = get_workspace_dir(app_handle)?
        .join(".hajimi")
        .join("checkpoints");
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建 checkpoint 目录: {}", e))?;
    Ok(dir)
}

fn trace_event_id(event: &TraceEvent) -> String {
    format!(
        "trace_{}_{}_{}",
        event.iteration,
        format!("{:?}", event.step_type).to_lowercase(),
        event.timestamp.timestamp_millis()
    )
}

pub(crate) fn checkpoint_record_from_trace(event: &TraceEvent) -> CheckpointRecord {
    let trace_id = trace_event_id(event);
    let operation = event.operation_summary.as_ref();
    let files_changed = operation
        .map(|op| op.files_edited + op.files_created + op.files_deleted)
        .unwrap_or(0);
    let total_diff_lines = operation.map(|op| op.total_diff_lines).unwrap_or(0);
    let label = format!("{:?} iteration {}", event.step_type, event.iteration);

    CheckpointRecord {
        id: format!("chk_{}", trace_id),
        timestamp: event.timestamp.to_rfc3339(),
        label,
        files: Vec::new(),
        diff_summary: CheckpointDiffSummary {
            files_changed,
            hunks: None,
            additions: None,
            deletions: None,
            summary: if total_diff_lines > 0 {
                format!(
                    "{} diff lines reported by trace operation summary",
                    total_diff_lines
                )
            } else {
                event.details.clone()
            },
        },
        trace_event_ids: vec![trace_id],
        metadata: CheckpointMetadata {
            source: "desktop-trace".to_string(),
            agent_id: None,
            iteration: event.iteration,
            step_type: format!("{:?}", event.step_type),
            confidence: event.confidence_score,
            schema_version: 1,
        },
    }
}

fn checkpoint_detail_mentions_checkpoint(details: &str) -> bool {
    details.to_ascii_lowercase().contains("checkpoint")
}

pub(crate) fn is_checkpoint_store_trace(event: &TraceEvent) -> bool {
    event.step_type == TraceStepType::Store && checkpoint_detail_mentions_checkpoint(&event.details)
}

pub(crate) fn write_checkpoint_record(
    app_handle: &tauri::AppHandle,
    record: &CheckpointRecord,
) -> Result<(), String> {
    let dir = checkpoint_store_dir(app_handle)?;
    let path = dir.join(format!("{}.json", record.id));
    let json = serde_json::to_string_pretty(record)
        .map_err(|e| format!("checkpoint 序列化失败: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("checkpoint 写入失败: {}", e))
}

fn read_checkpoint_records(app_handle: &tauri::AppHandle) -> Result<Vec<CheckpointRecord>, String> {
    let dir = checkpoint_store_dir(app_handle)?;
    let mut records = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("checkpoint 读取失败: {}", e))? {
        let entry = entry.map_err(|e| format!("checkpoint 目录项读取失败: {}", e))?;
        if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())
            .map_err(|e| format!("checkpoint 文件读取失败: {}", e))?;
        let record = serde_json::from_str::<CheckpointRecord>(&content)
            .map_err(|e| format!("checkpoint JSON 解析失败: {}", e))?;
        records.push(record);
    }
    records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(records)
}

fn find_checkpoint_record(
    records: &[CheckpointRecord],
    id: &str,
) -> Result<CheckpointRecord, String> {
    records
        .iter()
        .find(|record| record.id == id)
        .cloned()
        .ok_or_else(|| format!("checkpoint not found: {}", id))
}

fn checkpoint_file_change(
    before: Option<&CheckpointFileRef>,
    after: Option<&CheckpointFileRef>,
    path: &str,
) -> CheckpointFileChange {
    CheckpointFileChange {
        path: path.to_string(),
        before_status: before.map(|file| file.status.clone()),
        after_status: after.map(|file| file.status.clone()),
        before_hash: before.and_then(|file| file.after_hash.clone().or(file.before_hash.clone())),
        after_hash: after.and_then(|file| file.after_hash.clone().or(file.before_hash.clone())),
    }
}

fn compare_checkpoint_records(
    before: &CheckpointRecord,
    after: &CheckpointRecord,
) -> CheckpointCompareResult {
    let before_files: HashMap<&str, &CheckpointFileRef> = before
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let after_files: HashMap<&str, &CheckpointFileRef> = after
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();

    let mut files_added = Vec::new();
    let mut files_removed = Vec::new();
    let mut files_modified = Vec::new();

    for (path, file) in &after_files {
        match before_files.get(path) {
            None => files_added.push(checkpoint_file_change(None, Some(*file), path)),
            Some(previous)
                if previous.status != file.status
                    || previous.before_hash != file.before_hash
                    || previous.after_hash != file.after_hash =>
            {
                files_modified.push(checkpoint_file_change(Some(*previous), Some(*file), path));
            }
            _ => {}
        }
    }

    for (path, file) in &before_files {
        if !after_files.contains_key(path) {
            files_removed.push(checkpoint_file_change(Some(*file), None, path));
        }
    }

    files_added.sort_by(|a, b| a.path.cmp(&b.path));
    files_removed.sort_by(|a, b| a.path.cmp(&b.path));
    files_modified.sort_by(|a, b| a.path.cmp(&b.path));

    let file_change_count = files_added.len() + files_removed.len() + files_modified.len();
    let summary_changed = before.diff_summary.summary != after.diff_summary.summary
        || before.diff_summary.files_changed != after.diff_summary.files_changed
        || before.diff_summary.hunks != after.diff_summary.hunks
        || before.diff_summary.additions != after.diff_summary.additions
        || before.diff_summary.deletions != after.diff_summary.deletions;
    let trace_changed = before.trace_event_ids != after.trace_event_ids
        || before.metadata.iteration != after.metadata.iteration
        || before.metadata.step_type != after.metadata.step_type;
    let same = file_change_count == 0 && !summary_changed && !trace_changed;
    let data_source = if !before.files.is_empty() || !after.files.is_empty() {
        "checkpoint.files".to_string()
    } else {
        "checkpoint.diff_summary+metadata".to_string()
    };
    let summary = if file_change_count > 0 {
        format!(
            "{} added, {} modified, {} removed",
            files_added.len(),
            files_modified.len(),
            files_removed.len()
        )
    } else if summary_changed || trace_changed {
        "No file-level diff data; checkpoint summary or trace metadata changed".to_string()
    } else {
        "No differences detected".to_string()
    };

    CheckpointCompareResult {
        id_a: before.id.clone(),
        id_b: after.id.clone(),
        same,
        files_added,
        files_removed,
        files_modified,
        summary,
        data_source,
    }
}

fn checkpoint_restore_content(file: &CheckpointFileRef) -> Option<&str> {
    file.after_content.as_deref().or(file.content.as_deref())
}

fn sanitize_restore_id(id: &str) -> String {
    id.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn resolve_restore_target(file_path: &str, base_dir: &Path) -> Result<PathBuf, String> {
    match resolve_workspace_path(file_path, base_dir, PathIntent::AnyExisting) {
        Ok(path) => Ok(path),
        Err(_) => resolve_workspace_path(file_path, base_dir, PathIntent::NewFile),
    }
}

fn restore_backup_dir(
    app_handle: &tauri::AppHandle,
    checkpoint_id: &str,
) -> Result<PathBuf, String> {
    let dir = checkpoint_store_dir(app_handle)?
        .join("backups")
        .join(format!(
            "restore_{}_{}",
            sanitize_restore_id(checkpoint_id),
            chrono::Utc::now().timestamp_millis()
        ));
    Ok(dir)
}

fn validate_restore_confirmation(confirm_restore: bool, dry_run: bool) -> Result<(), String> {
    if !dry_run && !confirm_restore {
        return Err("restore refused: confirmRestore must be true for write restore".into());
    }
    Ok(())
}

fn build_restore_plan(
    record: &CheckpointRecord,
    base_dir: &Path,
    backup_dir: &Path,
) -> Result<RestoreResult, String> {
    if record.files.is_empty() {
        return Err(format!(
            "checkpoint {} has no file-level restore data",
            record.id
        ));
    }

    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("无法解析工作目录: {}", e))?;
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for file in &record.files {
        let safe_target = resolve_restore_target(&file.path, base_dir)
            .map_err(|e| format!("restore path rejected for '{}': {}", file.path, e))?;
        if safe_target.exists() && safe_target.is_dir() {
            return Err(format!("restore target is a directory: {}", file.path));
        }

        let target_exists = safe_target.exists();
        let action = match file.status.as_str() {
            "deleted" | "removed" => "delete",
            _ => "write",
        };
        if action == "write" && checkpoint_restore_content(file).is_none() {
            warnings.push(format!(
                "{} has no content snapshot; real restore will be refused",
                file.path
            ));
        }

        let backup_path = if target_exists {
            let rel = safe_target
                .strip_prefix(&canonical_base)
                .map_err(|e| format!("restore target not under workspace: {}", e))?;
            Some(backup_dir.join(rel).to_string_lossy().to_string())
        } else {
            None
        };
        files.push(RestoreFilePlan {
            path: file.path.clone(),
            action: action.to_string(),
            target_exists,
            backup_path,
            reason: format!("checkpoint status '{}'", file.status),
        });
    }

    Ok(RestoreResult {
        checkpoint_id: record.id.clone(),
        restored_at: chrono::Utc::now().to_rfc3339(),
        dry_run: true,
        backup_dir: backup_dir.to_string_lossy().to_string(),
        files,
        warnings,
    })
}

fn backup_restore_targets(
    plan: &RestoreResult,
    base_dir: &Path,
    backup_dir: &Path,
) -> Result<(), String> {
    std::fs::create_dir_all(backup_dir)
        .map_err(|e| format!("restore backup init failed: {}", e))?;
    for item in &plan.files {
        if !item.target_exists {
            continue;
        }
        let backup_path = item
            .backup_path
            .as_ref()
            .ok_or_else(|| format!("missing backup path for {}", item.path))?;
        let source = resolve_restore_target(&item.path, base_dir)?;
        let backup = PathBuf::from(backup_path);
        if let Some(parent) = backup.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("restore backup parent failed: {}", e))?;
        }
        std::fs::copy(&source, &backup)
            .map_err(|e| format!("restore backup failed for {}: {}", item.path, e))?;
    }
    Ok(())
}

fn rollback_restore(plan: &RestoreResult, base_dir: &Path) {
    for item in &plan.files {
        if let Ok(target) = resolve_restore_target(&item.path, base_dir) {
            if let Some(backup) = item.backup_path.as_ref() {
                let backup_path = PathBuf::from(backup);
                if backup_path.exists() {
                    if let Some(parent) = target.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::copy(backup_path, target);
                }
            } else if item.action == "write" && target.exists() {
                let _ = std::fs::remove_file(target);
            }
        }
    }
}

fn apply_restore_plan(
    record: &CheckpointRecord,
    plan: &RestoreResult,
    base_dir: &Path,
) -> Result<(), String> {
    for item in &plan.files {
        let file = record
            .files
            .iter()
            .find(|file| file.path == item.path)
            .ok_or_else(|| format!("restore file missing from checkpoint: {}", item.path))?;
        let target = resolve_restore_target(&item.path, base_dir)?;
        let result = if item.action == "delete" {
            if target.exists() {
                std::fs::remove_file(&target)
                    .map_err(|e| format!("restore delete failed for {}: {}", item.path, e))
            } else {
                Ok(())
            }
        } else {
            let content = checkpoint_restore_content(file).ok_or_else(|| {
                format!(
                    "checkpoint {} lacks content snapshot for {}; dry-run only",
                    record.id, item.path
                )
            })?;
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("restore parent create failed for {}: {}", item.path, e)
                })?;
            }
            std::fs::write(&target, content)
                .map_err(|e| format!("restore write failed for {}: {}", item.path, e))
        };

        if let Err(err) = result {
            rollback_restore(plan, base_dir);
            return Err(err);
        }
    }
    Ok(())
}

// PathIntent, resolve_workspace_path, and fs commands moved to commands::fs


// ------------------------------------------------------------------
// Tool-system commands
// ------------------------------------------------------------------
#[derive(Serialize, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl From<ToolOutput> for ToolResult {
    fn from(o: ToolOutput) -> Self {
        Self {
            stdout: o.stdout,
            stderr: o.stderr,
            exit_code: o.exit_code,
        }
    }
}

pub(crate) fn tool_requires_confirmation(permissions: &ToolPermissions) -> bool {
    permissions.requires_confirmation || permissions.default_level == PermissionLevel::Ask
}

fn canonical_tool_args(args: &Value) -> String {
    serde_json::to_string(args).unwrap_or_else(|_| "null".to_string())
}

pub(crate) fn summarize_tool_args(args: &Value) -> String {
    const MAX_SUMMARY_CHARS: usize = 1400;
    let raw = canonical_tool_args(args);
    if raw.chars().count() <= MAX_SUMMARY_CHARS {
        return raw;
    }
    let mut summary = raw.chars().take(MAX_SUMMARY_CHARS).collect::<String>();
    summary.push_str("...");
    summary
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ToolAuthorization {
    Allow,
    RequireNativeConfirmation,
}

pub(crate) fn enforce_tool_permissions(
    permissions: &ToolPermissions,
    tool_name: &str,
    _args: &Value,
) -> Result<ToolAuthorization, String> {
    match permissions.default_level {
        PermissionLevel::Deny => Err(format!("tool '{}' is denied by policy", tool_name)),
        PermissionLevel::Allow if !tool_requires_confirmation(permissions) => {
            Ok(ToolAuthorization::Allow)
        }
        PermissionLevel::Allow | PermissionLevel::Ask => {
            Ok(ToolAuthorization::RequireNativeConfirmation)
        }
    }
}

pub(crate) fn confirm_tool_native(
    app_handle: &tauri::AppHandle,
    tool_name: &str,
    args: &Value,
) -> Result<bool, String> {
    let message = format!(
        "工具 '{}' 需要执行权限确认。\n\n参数摘要:\n{}\n\n是否允许执行？",
        tool_name,
        summarize_tool_args(args)
    );
    let approved = app_handle
        .dialog()
        .message(message)
        .title("Hajimi 工具执行确认")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "允许执行".to_string(),
            "取消".to_string(),
        ))
        .blocking_show();
    Ok(approved)
}


// list_tools and execute_tool moved to commands::tool


// ------------------------------------------------------------------
// LLM commands
// ------------------------------------------------------------------
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamEvent {
    pub chunk: String,
    pub done: bool,
    pub error: Option<String>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamDiagnosticEvent {
    stage: String,
    session_id: Option<String>,
    data: Value,
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
pub(crate) fn preview_for_diagnostic(text: &str) -> String {
    text.chars().take(120).collect()
}

#[cfg(feature = "stream-diagnostics")]
pub(crate) fn write_stream_diagnostic(stage: &str, session_id: Option<&str>, data: Value) {
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
pub(crate) fn write_stream_diagnostic(_stage: &str, _session_id: Option<&str>, _data: Value) {}


// record_stream_diagnostic and get_stream_diagnostic_info moved to commands::info


#[derive(Serialize, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub available: bool,
    pub default_model: String,
}

// ------------------------------------------------------------------
// Provider Config (custom OpenAI-compatible providers)
// ------------------------------------------------------------------
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    #[serde(skip_serializing, default)]
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    #[deprecated(since = "3.9.0", note = "Use max_context_tokens instead")]
    pub context_threshold: Option<usize>,
    #[serde(default)]
    pub max_context_tokens: Option<usize>,
    #[serde(default)]
    pub max_output_tokens: Option<usize>,
    #[serde(default)]
    pub reserve_output_tokens: Option<usize>,
    #[serde(default)]
    pub safety_margin_tokens: Option<usize>,
    #[serde(default)]
    pub retrieval_budget_tokens: Option<usize>,
    #[serde(default)]
    pub long_context_mode: Option<bool>,
}

impl ProviderConfig {
    #[allow(deprecated)]
    pub fn get_normalized_max_context_tokens(&self) -> Option<usize> {
        self.max_context_tokens.or(self.context_threshold)
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigView {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub context_threshold: Option<usize>,
    pub max_context_tokens: Option<usize>,
    pub max_output_tokens: Option<usize>,
    pub reserve_output_tokens: Option<usize>,
    pub safety_margin_tokens: Option<usize>,
    pub retrieval_budget_tokens: Option<usize>,
    pub long_context_mode: Option<bool>,
    pub has_api_key: bool,
}

impl ProviderConfigView {
    #[allow(deprecated)]
    pub fn from_config(config: ProviderConfig, has_api_key: bool) -> Self {
        Self {
            id: config.id,
            name: config.name,
            provider_type: config.provider_type,
            base_url: config.base_url,
            model: config.model,
            system_prompt: config.system_prompt,
            context_threshold: config.context_threshold,
            max_context_tokens: config.max_context_tokens,
            max_output_tokens: config.max_output_tokens,
            reserve_output_tokens: config.reserve_output_tokens,
            safety_margin_tokens: config.safety_margin_tokens,
            retrieval_budget_tokens: config.retrieval_budget_tokens,
            long_context_mode: config.long_context_mode,
            has_api_key,
        }
    }
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("provider_type", &self.provider_type)
            .field(
                "api_key",
                &if self.api_key.is_empty() {
                    "none"
                } else {
                    "sk-••••••••"
                },
            )
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .finish()
    }
}

pub(crate) fn provider_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
            .join("Hajimi")
            .join("providers.json")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Library/Application Support/Hajimi/providers.json")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config/hajimi/providers.json")
    }
}

// Workspace-level config lives in <workspace>/.hajimi/providers.json
pub(crate) fn workspace_config_path(workspace: &Path) -> PathBuf {
    workspace.join(".hajimi").join("providers.json")
}

pub(crate) fn trusted_workspace_path(
    workspace_path: Option<&str>,
    app_handle: &tauri::AppHandle,
) -> Result<Option<PathBuf>, String> {
    let current = get_workspace_dir(app_handle)?;
    trusted_workspace_path_for_current(workspace_path, &current)
}

pub(crate) fn trusted_workspace_path_for_current(
    workspace_path: Option<&str>,
    current: &Path,
) -> Result<Option<PathBuf>, String> {
    let canonical_current = current
        .canonicalize()
        .map_err(|e| format!("无法解析当前 workspace: {}", e))?;
    let Some(input) = workspace_path else {
        return Ok(Some(canonical_current));
    };
    let requested = PathBuf::from(input);
    let canonical_requested = requested
        .canonicalize()
        .map_err(|e| format!("无法解析 workspace 参数: {}", e))?;
    if canonical_requested != canonical_current {
        return Err("workspace 参数越界: 请求 workspace 不是当前 workspace".to_string());
    }
    Ok(Some(canonical_current))
}

pub(crate) fn trusted_workspace_config_path_for_current(
    workspace_path: Option<&str>,
    current: &Path,
) -> Result<PathBuf, String> {
    let workspace = trusted_workspace_path_for_current(workspace_path, current)?
        .ok_or_else(|| "workspace 参数缺失".to_string())?;
    Ok(workspace_config_path(&workspace))
}

pub(crate) fn add_workspace_provider_config_for_current(
    config: ProviderConfig,
    workspace_path: Option<&str>,
    current: &Path,
) -> Result<(), String> {
    let path = trusted_workspace_config_path_for_current(workspace_path, current)?;
    let mut configs = read_configs_at(&path);
    if configs.iter().any(|c| c.id == config.id) {
        return Err(format!("Provider '{}' already exists", config.id));
    }
    configs.push(config);
    write_configs_to_path(&path, &configs)
}

pub(crate) fn update_workspace_provider_config_for_current(
    config: ProviderConfig,
    workspace_path: Option<&str>,
    current: &Path,
) -> Result<(), String> {
    let path = trusted_workspace_config_path_for_current(workspace_path, current)?;
    let mut configs = read_configs_at(&path);
    let idx = configs
        .iter()
        .position(|c| c.id == config.id)
        .ok_or_else(|| format!("Provider '{}' not found", config.id))?;
    configs[idx] = config;
    write_configs_to_path(&path, &configs)
}

pub(crate) fn delete_workspace_provider_config_for_current(
    id: &str,
    workspace_path: Option<&str>,
    current: &Path,
) -> Result<(), String> {
    let path = trusted_workspace_config_path_for_current(workspace_path, current)?;
    let mut configs = read_configs_at(&path);
    configs.retain(|c| c.id != id);
    if configs.is_empty() {
        let _ = std::fs::remove_file(&path);
        Ok(())
    } else {
        write_configs_to_path(&path, &configs)
    }
}

// Profile-level config lives in profiles/{name}/providers.json (B-05/01)
pub(crate) fn profile_config_path(name: &str) -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
            .join("Hajimi")
            .join("profiles")
            .join(name)
            .join("providers.json")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Library/Application Support/Hajimi/profiles")
            .join(name)
            .join("providers.json")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config/hajimi/profiles")
            .join(name)
            .join("providers.json")
    }
}

pub(crate) fn sanitize_profile_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Profile name contains illegal characters".to_string());
    }
    Ok(name.to_string())
}

pub(crate) fn read_configs_at(path: &std::path::Path) -> Vec<ProviderConfig> {
    if !path.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub(crate) fn read_merged_configs(workspace: Option<&Path>, profile: Option<&str>) -> Vec<ProviderConfig> {
    let global = read_provider_configs_with_profile(profile);
    let mut map: HashMap<String, ProviderConfig> =
        global.into_iter().map(|c| (c.id.clone(), c)).collect();
    if let Some(ws) = workspace {
        let ws_path = workspace_config_path(ws);
        for cfg in read_configs_at(&ws_path) {
            map.insert(cfg.id.clone(), cfg);
        }
    }
    map.into_values().collect()
}

// Keyring helpers for secure storage (P0-1), profile-aware (B-05/01)
fn keyring_entry_id(id: &str, profile: Option<&str>) -> String {
    match profile {
        None | Some("default") | Some("") => format!("provider:{}", id),
        Some(p) => format!("provider:{}:{}", p, id),
    }
}

fn is_masked_api_key_placeholder(api_key: &str) -> bool {
    let trimmed = api_key.trim();
    trimmed.contains('•') || trimmed.contains("re-enter to update")
}

pub(crate) fn submitted_api_key(api_key: &str) -> Option<&str> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() || is_masked_api_key_placeholder(trimmed) {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn provider_config_has_api_key(config: &ProviderConfig, profile: Option<&str>) -> bool {
    submitted_api_key(&config.api_key).is_some()
        || get_api_key_with_profile(&config.id, profile).is_ok()
}

#[allow(dead_code)]
fn save_api_key(id: &str, api_key: &str) -> Result<(), String> {
    save_api_key_with_profile(id, api_key, None)
}

pub(crate) fn save_api_key_with_profile(id: &str, api_key: &str, profile: Option<&str>) -> Result<(), String> {
    let Some(api_key) = submitted_api_key(api_key) else {
        return Ok(());
    };
    let entry = Entry::new("hajimi", &keyring_entry_id(id, profile))
        .map_err(|e| format!("keyring entry failed: {}", e))?;
    entry
        .set_password(api_key)
        .map_err(|e| format!("keyring set failed: {}", e))?;
    let stored = entry
        .get_password()
        .map_err(|e| format!("keyring verify failed: {}", e))?;
    if stored != api_key {
        return Err("keyring verify failed: stored key mismatch".to_string());
    }
    Ok(())
}

#[allow(dead_code)]
fn get_api_key(id: &str) -> Result<String, String> {
    get_api_key_with_profile(id, None)
}

pub(crate) fn get_api_key_with_profile(id: &str, profile: Option<&str>) -> Result<String, String> {
    let entry = Entry::new("hajimi", &keyring_entry_id(id, profile))
        .map_err(|e| format!("keyring entry failed: {}", e))?;
    entry
        .get_password()
        .map_err(|e| format!("keyring get failed: {}", e))
}

#[allow(dead_code)]
fn delete_api_key(id: &str) -> Result<(), String> {
    delete_api_key_with_profile(id, None)
}

pub(crate) fn delete_api_key_with_profile(id: &str, profile: Option<&str>) -> Result<(), String> {
    let entry = Entry::new("hajimi", &keyring_entry_id(id, profile))
        .map_err(|e| format!("无法访问密钥存储: {}", e))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // 已删除视为成功
        Err(e) => Err(format!("删除密钥失败: {}", e)),
    }
}

// Migration from plaintext to keyring (one-time on upgrade)
fn migrate_provider_keys(
    configs: &mut [ProviderConfig],
    profile: Option<&str>,
) -> Result<(), String> {
    let mut migrated = false;
    for cfg in configs.iter_mut() {
        if let Some(api_key) = submitted_api_key(&cfg.api_key) {
            save_api_key_with_profile(&cfg.id, api_key, profile)?;
            cfg.api_key.clear(); // sanitize in memory too
            migrated = true;
        }
    }
    if migrated {
        // Will be written without keys due to skip_serializing
        println!("Migrated {} provider keys to OS keyring", configs.len());
    }
    Ok(())
}

#[allow(dead_code)]
fn read_provider_configs() -> Vec<ProviderConfig> {
    read_provider_configs_with_profile(None)
}

pub(crate) fn read_provider_configs_with_profile(profile: Option<&str>) -> Vec<ProviderConfig> {
    let path = match profile {
        None | Some("default") | Some("") => provider_config_path(),
        Some(p) => profile_config_path(p),
    };
    if !path.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut configs: Vec<ProviderConfig> = serde_json::from_str(&content).unwrap_or_default();
    let had_keys = configs.iter().any(|c| !c.api_key.trim().is_empty());
    // Perform migration if any keys are present in JSON (P0-1)
    if let Err(e) = migrate_provider_keys(&mut configs, profile) {
        eprintln!("Migration warning: {}", e);
    }
    if had_keys {
        let _ = write_provider_configs_with_profile(profile, &configs);
        println!("Migrated plaintext keys to secure keyring storage. providers.json sanitized.");
    }
    configs
}

#[allow(dead_code)]
fn write_provider_configs(configs: &[ProviderConfig]) -> Result<(), String> {
    write_provider_configs_with_profile(None, configs)
}

pub(crate) fn write_provider_configs_with_profile(
    profile: Option<&str>,
    configs: &[ProviderConfig],
) -> Result<(), String> {
    let path = match profile {
        None | Some("default") | Some("") => provider_config_path(),
        Some(p) => profile_config_path(p),
    };
    write_configs_to_path(&path, configs)
}

pub(crate) fn write_configs_to_path(path: &std::path::Path, configs: &[ProviderConfig]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(configs).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set permissions on {}: {}", path.display(), e))?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::set_permissions(parent, Permissions::from_mode(0o700));
        }
    }
    #[cfg(windows)]
    {
        if let Ok(username) = std::env::var("USERNAME") {
            let output = std::process::Command::new("icacls")
                .arg(path)
                .arg("/inheritance:r")
                .arg("/grant:r")
                .arg(format!("{}:F", username))
                .output()
                .map_err(|e| format!("Failed to restrict ACL on {}: {}", path.display(), e))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "Failed to restrict ACL on {}: {}",
                    path.display(),
                    stderr
                ));
            }
        }
    }
    Ok(())
}


// Backup encryption helpers (B-04/02)
fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    key
}

pub(crate) fn encrypt_backup(plaintext: &str, password: &str) -> Result<Vec<u8>, String> {
    let salt: [u8; 16] = rand::random();
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("encryption failed: {}", e))?;
    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub(crate) fn decrypt_backup(data: &[u8], password: &str) -> Result<String, String> {
    if data.len() < 28 {
        return Err("invalid backup file".to_string());
    }
    let salt = &data[0..16];
    let nonce_bytes = &data[16..28];
    let ciphertext = &data[28..];
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "decryption failed: wrong password or corrupted file".to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

// get_provider_configs moved to commands::provider

// add_provider_config moved to commands::provider

// update_provider_config moved to commands::provider

// delete_provider_config moved to commands::provider

// get_providers moved to commands::provider

// get_current_workspace and validate_provider moved to commands::provider


pub(crate) fn create_llm_client(
    provider: &str,
    profile: Option<&str>,
    config: Option<ProviderConfig>,
) -> Result<Box<dyn LlmClient>, String> {
    match provider {
        "ollama" => Ok(Box::new(OllamaClient::default_local())),
        "anthropic" => Ok(Box::new(
            AnthropicClient::from_env().map_err(|e| format!("anthropic init failed: {}", e))?,
        )),
        "openai" => Ok(Box::new(
            OpenAiClient::from_env().map_err(|e| format!("openai init failed: {}", e))?,
        )),
        _ => {
            let cfg = config
                .ok_or_else(|| format!("config required for custom provider: {}", provider))?;
            let api_key = if let Some(api_key) = submitted_api_key(&cfg.api_key) {
                api_key.to_string()
            } else {
                get_api_key_with_profile(&cfg.id, profile)
                    .map_err(|e| format!("Failed to retrieve key for {}: {}", cfg.id, e))?
            };
            match cfg.provider_type.as_str() {
                "anthropic" => {
                    let llm_provider = engine_llm_core::LlmProvider::Anthropic {
                        api_key: SecretString::new(api_key.into_boxed_str()),
                        model: cfg.model,
                        base_url: cfg.base_url,
                    };
                    Ok(Box::new(AnthropicClient::new(llm_provider)))
                }
                _ => {
                    let llm_provider = engine_llm_core::LlmProvider::OpenAi {
                        api_key: SecretString::new(api_key.into_boxed_str()),
                        model: cfg.model,
                        base_url: cfg.base_url,
                    };
                    Ok(Box::new(OpenAiClient::new(llm_provider)))
                }
            }
        }
    }
}

// stream_chat moved to commands::agent


// compact_context and optimize_context moved to commands::tool


// export_provider_backup and import_provider_backup moved to commands::provider

// ------------------------------------------------------------------
// Profile commands (B-05/01)
// ------------------------------------------------------------------
// Profile commands moved to commands::profile


// ------------------------------------------------------------------
// Agent provider commands (B-05/02)
// ------------------------------------------------------------------


#[allow(deprecated)]
pub(crate) async fn write_provider_caps_to_blackboard(
    bb: &agent_core::blackboard::Blackboard,
    agent_id: &str,
    provider: &str,
    config: Option<&ProviderConfig>,
) {
    bb.write("__hajimi_provider_id", provider, agent_id).await;
    if let Some(cfg) = config {
        bb.write("__hajimi_model", &cfg.model, agent_id).await;
        if let Some(val) = cfg.get_normalized_max_context_tokens() {
            bb.write("__hajimi_max_context_tokens", &val.to_string(), agent_id)
                .await;
        }
        if let Some(val) = cfg.max_output_tokens {
            bb.write("__hajimi_max_output_tokens", &val.to_string(), agent_id)
                .await;
        }
        if let Some(val) = cfg.reserve_output_tokens {
            bb.write("__hajimi_reserve_output_tokens", &val.to_string(), agent_id)
                .await;
        }
        if let Some(val) = cfg.safety_margin_tokens {
            bb.write("__hajimi_safety_margin_tokens", &val.to_string(), agent_id)
                .await;
        }
        if let Some(val) = cfg.retrieval_budget_tokens {
            bb.write(
                "__hajimi_retrieval_budget_tokens",
                &val.to_string(),
                agent_id,
            )
            .await;
        }
        if let Some(val) = cfg.long_context_mode {
            bb.write("__hajimi_long_context_mode", &val.to_string(), agent_id)
                .await;
        }
        if let Some(val) = cfg.context_threshold {
            bb.write("__hajimi_context_threshold", &val.to_string(), agent_id)
                .await;
        }
    } else {
        // Clear old ones if not a custom provider to avoid leaking stale config
        bb.write("__hajimi_model", "", agent_id).await;
        bb.write("__hajimi_max_context_tokens", "", agent_id).await;
        bb.write("__hajimi_max_output_tokens", "", agent_id).await;
        bb.write("__hajimi_reserve_output_tokens", "", agent_id)
            .await;
        bb.write("__hajimi_safety_margin_tokens", "", agent_id)
            .await;
        bb.write("__hajimi_retrieval_budget_tokens", "", agent_id)
            .await;
        bb.write("__hajimi_long_context_mode", "", agent_id).await;
        bb.write("__hajimi_context_threshold", "", agent_id).await;
    }
}













// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let pending_approvals = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

            // Build custom UiBridgeGovernance
            let custom_gov = Arc::new(UiBridgeGovernance {
                inner: Arc::new(agent_core::governance::DefaultGovernance::new()),
                app_handle: app_handle.clone(),
            });

            let workspace_root = get_workspace_dir(app.handle()).map_err(std::io::Error::other)?;
            // Build the registry once, count tools, then wrap in Arc<Mutex<>> for thread safety and loop sharing (ADR-001)
            let built_registry = build_registry(&workspace_root);
            let tool_count = built_registry.list().len();
            let registry = Arc::new(tokio::sync::Mutex::new(built_registry));

            // P0-DRIVER-INJECTION-2026-05-30: Create shared LLM client slot + dynamic Driver
            let agent_llm_client: Arc<
                tokio::sync::RwLock<Option<Arc<dyn engine_llm_core::LlmClient>>>,
            > = Arc::new(tokio::sync::RwLock::new(None));

            let desktop_driver = Arc::new(DesktopAgentTurnDriver {
                agent_llm_client: agent_llm_client.clone(),
                tool_registry: registry.clone(),
            });

            // Create production-ready AgentLoop with planner, reflector, real ToolRegistry, and LLM-Native Driver injected.
            // SAFETY: AgentLoop is Send + Sync; safe to hold in AppState and register with Tauri.
            let agent_loop = {
                let mem = Arc::new(tokio::sync::Mutex::new(AgentMemoryGateway::new(
                    "hajimi-desktop",
                )));
                let planner = Arc::new(tokio::sync::Mutex::new(HierarchicalPlanner::new(
                    mem.clone(),
                    AgentContext::new(),
                )));
                let reflector = Arc::new(tokio::sync::Mutex::new(AutonomousReflector::new(
                    mem.clone(),
                    AgentContext::new(),
                )));
                AgentLoopBuilder::production_ready("hajimi-desktop")
                    .with_planner(planner)
                    .with_reflector(reflector)
                    .with_governance(custom_gov)
                    .with_tool_registry(registry.clone()) // Inject real ToolRegistry
                    .with_native_driver(Some(desktop_driver)) // P0 fix: inject dynamic LLM-Native Driver
                    .build()
                    .expect("AgentLoop build failed")
            };

            // Log tool injection count at info level to verify integration success (UX-002)
            log::info!("AgentLoop initialized with {} tools", tool_count);

            let agent_loop_for_setup = Arc::new(agent_loop);

            let state = AppState {
                registry: registry.clone(),
                active_profile: std::sync::Mutex::new(None),
                agent_providers: std::sync::Mutex::new(HashMap::new()),
                trace_tx: std::sync::Mutex::new(None),
                paused: std::sync::Mutex::new(false),
                approval_level: std::sync::Mutex::new("Auto".to_string()),
                edit_history: Arc::new(tokio::sync::Mutex::new(Vec::new())),
                memory_gateway: Arc::new(MemoryGateway::with_budget(TokenBudget {
                    focus_limit: 8000,
                    working_limit: 64000,
                    archive_limit: 2000000,
                })),
                token_tracker: Arc::new(TokenUsageTracker::new()),
                pending_approvals,
                agent_llm_client,
            };

            // Inject the broadcast sender so frontend trace panel receives real AgentLoop events.
            if let Some(tx) = agent_loop_for_setup.trace_tx() {
                state.set_trace_tx(tx);
            }

            app.manage(state);
            app.manage(agent_loop_for_setup.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::info::greet,
            commands::fs::read_file,
            commands::fs::write_file,
            commands::fs::list_dir,
            commands::fs::create_dir,
            commands::fs::rename_path,
            commands::fs::delete_path,
            commands::tool::list_tools,
            commands::tool::execute_tool,
            commands::provider::get_providers,
            commands::provider::get_provider_configs,
            commands::provider::add_provider_config,
            commands::provider::update_provider_config,
            commands::provider::delete_provider_config,
            commands::provider::validate_provider,
            commands::info::get_current_workspace,
            commands::provider::export_provider_backup,
            commands::provider::import_provider_backup,
            commands::info::record_stream_diagnostic,
            commands::info::get_stream_diagnostic_info,
            commands::agent::stream_chat,
            commands::tool::compact_context,
            commands::tool::optimize_context,
            // B-05/01 Profile
            commands::profile::list_profiles,
            commands::profile::get_active_profile,
            commands::profile::set_active_profile,
            commands::profile::create_profile,
            commands::profile::delete_profile,
            // B-05/02 Agent provider
            commands::agent::get_agent_providers,
            commands::agent::set_agent_provider,
            commands::agent::create_agent_with_provider,
            commands::agent::run_agent_task,
            // B-05/03 Audit
            commands::info::get_audit_logs,
            // B-02/06 Trace
            commands::agent::subscribe_agent_trace,
            // B-03/06 Governance
            commands::governance::pause_loop,
            commands::governance::resume_loop,
            commands::governance::set_approval_level,
            commands::governance::inject_memory,
            commands::governance::update_plan,
            // B-04/06 Checkpoint
            commands::checkpoint::list_checkpoints,
            commands::checkpoint::restore_checkpoint,
            commands::checkpoint::compare_checkpoints,
            commands::checkpoint::export_checkpoint,
            // B-05/06 Resource
            commands::info::get_resource_metrics,
            commands::agent::subscribe_resource_alerts,
            // Phase 4 Day 3: Inline Editing
            commands::tool::apply_edits,
            commands::tool::preview_edit,
            commands::tool::get_ast_context,
            // Phase 4 Day 5: Command Palette & Observability
            commands::info::get_edit_history,
            commands::agent::run_agent_command,
            // P1-03/05: Token cumulative stats
            commands::info::get_cumulative_stats,
            // Day 12: Context capacity probe
            commands::tool::probe_provider_context_capacity,
            commands::tool::get_probe_result,
            commands::info::get_latest_receipt,
            commands::governance::resolve_agent_approval,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]

    use super::*;
    use crate::commands::tool::{apply_edits_with_base_dir, EditHunkPayload, preview_edit_for_path, preview_edit_with_base_dir};
    use crate::state::{PendingApprovalMap, await_ui_approval_response};
    use crate::commands::agent::AgentUiEvent;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// 创建临时测试 workspace（使用 std::env::temp_dir）
    fn setup_test_workspace() -> (PathBuf, PathBuf) {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "hajimi-test-{}-{}-{}",
            std::process::id(),
            nanos,
            counter
        ));
        let workspace = temp.join("test-workspace");
        let _ = std::fs::remove_dir_all(&temp); // 清理旧数据
        std::fs::create_dir_all(&workspace).expect("无法创建 workspace");
        (temp, workspace)
    }

    fn cleanup_test_workspace(temp: &PathBuf) {
        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn approval_wait_times_out_and_cleans_pending_request() {
        let pending: PendingApprovalMap = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let request_id = "approval-timeout-test".to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        pending.lock().await.insert(request_id.clone(), tx);

        let decision = await_ui_approval_response(
            pending.clone(),
            request_id.clone(),
            "list_directory".to_string(),
            rx,
            std::time::Duration::from_millis(5),
        )
        .await;

        assert!(matches!(
            decision,
            agent_core::governance::Decision::Rejected(ref reason)
                if reason.contains("Approval timed out")
                    && reason.contains("list_directory")
        ));
        assert!(
            pending.lock().await.get(&request_id).is_none(),
            "timed out approval request should be removed from pending map"
        );
    }

    #[test]
    fn agent_outcome_output_returns_success_message() {
        let output = crate::commands::agent::agent_outcome_output(agent_core::agent_loop::LoopOutcome::SuccessWithMessage(
            "a\nb\nc".to_string(),
        ));

        assert_eq!(output, "a\nb\nc");
    }

    #[test]
    fn agent_outcome_output_keeps_legacy_success() {
        let output = crate::commands::agent::agent_outcome_output(agent_core::agent_loop::LoopOutcome::Success);

        assert_eq!(output, "Success");
    }

    #[test]
    fn agent_outcome_output_keeps_failure_debug_shape() {
        let output = crate::commands::agent::agent_outcome_output(agent_core::agent_loop::LoopOutcome::ActFailed(
            "boom".into(),
        ));

        assert_eq!(output, "ActFailed(\"boom\")");
    }

    #[test]
    fn execute_tool_requires_native_confirmation_for_ask_policy() {
        let permissions = ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        };
        let args = json!({"path": "a.txt"});

        let result = enforce_tool_permissions(&permissions, "write_file", &args);

        assert_eq!(
            result.unwrap(),
            ToolAuthorization::RequireNativeConfirmation
        );
    }

    #[test]
    fn execute_tool_allows_low_risk_tool_without_confirmation() {
        let permissions = ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        };
        let args = json!({"message": "test"});

        let result = enforce_tool_permissions(&permissions, "lsp_hover", &args);

        assert_eq!(result.unwrap(), ToolAuthorization::Allow);
    }

    #[test]
    fn execute_tool_denies_policy_deny() {
        let permissions = ToolPermissions {
            default_level: PermissionLevel::Deny,
            requires_confirmation: true,
            allowed_paths: None,
        };
        let args = json!({"message": "different"});

        let result = enforce_tool_permissions(&permissions, "git_commit", &args);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("denied by policy"));
    }

    #[test]
    fn tool_arg_summary_is_bounded_for_native_dialog() {
        let args = json!({"payload": "x".repeat(5000)});
        let summary = summarize_tool_args(&args);

        assert!(summary.len() < 1500);
        assert!(summary.ends_with("..."));
    }

    #[test]
    fn preview_edit_rejects_empty_old_string_at_command_boundary() {
        let (temp, workspace) = setup_test_workspace();
        let test_file = workspace.join("edit.txt");
        std::fs::write(&test_file, "hello").expect("write test file");

        let result = preview_edit_for_path(&test_file, "edit.txt", "", "world");

        assert!(result.is_err());
        cleanup_test_workspace(&temp);
    }

    #[cfg(unix)]
    fn create_dir_link(link: &Path, target: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_dir_link(link: &Path, target: &Path) -> std::io::Result<()> {
        match std::os::windows::fs::symlink_dir(target, link) {
            Ok(()) => Ok(()),
            Err(primary_error) => {
                let status = std::process::Command::new("cmd")
                    .args(["/C", "mklink", "/J"])
                    .arg(link)
                    .arg(target)
                    .status();
                match status {
                    Ok(status) if status.success() => Ok(()),
                    _ => Err(primary_error),
                }
            }
        }
    }

    #[tokio::test]
    async fn apply_edits_allows_workspace_file() {
        let (temp, workspace) = setup_test_workspace();
        let file = workspace.join("edit.txt");
        std::fs::write(&file, "hello").expect("write workspace file");
        let registry = build_registry(&workspace);

        let result = apply_edits_with_base_dir(
            vec![EditHunkPayload {
                path: "edit.txt".to_string(),
                old_string: "hello".to_string(),
                new_string: "world".to_string(),
            }],
            &registry,
            &workspace,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "world");
        cleanup_test_workspace(&temp);
    }

    #[tokio::test]
    async fn apply_edits_rejects_absolute_path_outside_workspace() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside.txt");
        std::fs::write(&outside, "hello").expect("write outside file");
        let registry = build_registry(&workspace);

        let result = apply_edits_with_base_dir(
            vec![EditHunkPayload {
                path: outside.to_string_lossy().to_string(),
                old_string: "hello".to_string(),
                new_string: "world".to_string(),
            }],
            &registry,
            &workspace,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("路径越界"));
        assert!(!err.contains(&temp.to_string_lossy().to_string()));
        cleanup_test_workspace(&temp);
    }

    #[tokio::test]
    async fn apply_edits_rejects_parent_traversal() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(temp.join("outside.txt"), "hello").expect("write outside file");
        let registry = build_registry(&workspace);

        let result = apply_edits_with_base_dir(
            vec![EditHunkPayload {
                path: "../outside.txt".to_string(),
                old_string: "hello".to_string(),
                new_string: "world".to_string(),
            }],
            &registry,
            &workspace,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
        cleanup_test_workspace(&temp);
    }

    #[tokio::test]
    async fn apply_edits_rejects_symlink_escape() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside");
        std::fs::create_dir_all(&outside).expect("create outside dir");
        std::fs::write(outside.join("secret.txt"), "hello").expect("write outside file");
        let link = workspace.join("outside-link");
        create_dir_link(&link, &outside).expect("create workspace escape link");
        let registry = build_registry(&workspace);

        let result = apply_edits_with_base_dir(
            vec![EditHunkPayload {
                path: "outside-link/secret.txt".to_string(),
                old_string: "hello".to_string(),
                new_string: "world".to_string(),
            }],
            &registry,
            &workspace,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("路径越界"));
        assert!(!err.contains(&temp.to_string_lossy().to_string()));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn preview_edit_allows_workspace_file() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("edit.txt"), "hello").expect("write workspace file");

        let result = preview_edit_with_base_dir(
            "edit.txt".to_string(),
            "hello".to_string(),
            "world".to_string(),
            &workspace,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().contains("-hello"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn preview_edit_rejects_absolute_path_outside_workspace() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside.txt");
        std::fs::write(&outside, "hello").expect("write outside file");

        let result = preview_edit_with_base_dir(
            outside.to_string_lossy().to_string(),
            "hello".to_string(),
            "world".to_string(),
            &workspace,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("路径越界"));
        assert!(!err.contains(&temp.to_string_lossy().to_string()));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn preview_edit_rejects_parent_traversal() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(temp.join("outside.txt"), "hello").expect("write outside file");

        let result = preview_edit_with_base_dir(
            "../outside.txt".to_string(),
            "hello".to_string(),
            "world".to_string(),
            &workspace,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn preview_edit_rejects_symlink_escape() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside");
        std::fs::create_dir_all(&outside).expect("create outside dir");
        std::fs::write(outside.join("secret.txt"), "hello").expect("write outside file");
        let link = workspace.join("outside-link");
        create_dir_link(&link, &outside).expect("create workspace escape link");

        let result = preview_edit_with_base_dir(
            "outside-link/secret.txt".to_string(),
            "hello".to_string(),
            "world".to_string(),
            &workspace,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("路径越界"));
        assert!(!err.contains(&temp.to_string_lossy().to_string()));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_existing_file() {
        let (temp, workspace) = setup_test_workspace();
        let test_file = workspace.join("test.txt");
        std::fs::write(&test_file, "hello").expect("无法写入测试文件");

        let result = resolve_workspace_path("test.txt", &workspace, PathIntent::ExistingFile);
        assert!(result.is_ok());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_existing_dir() {
        let (temp, workspace) = setup_test_workspace();
        let subdir = workspace.join("subdir");
        std::fs::create_dir_all(&subdir).expect("无法创建子目录");

        let result = resolve_workspace_path("subdir", &workspace, PathIntent::ExistingDir);
        assert!(result.is_ok());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_file() {
        let (temp, workspace) = setup_test_workspace();

        let result = resolve_workspace_path("newfile.txt", &workspace, PathIntent::NewFile);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_name().unwrap(), "newfile.txt");
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_dir() {
        let (temp, workspace) = setup_test_workspace();

        let result = resolve_workspace_path("newdir", &workspace, PathIntent::NewDir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_name().unwrap(), "newdir");
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_file_missing_parent() {
        let (temp, workspace) = setup_test_workspace();

        let result =
            resolve_workspace_path("nonexistent/newfile.txt", &workspace, PathIntent::NewFile);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("父目录不存在"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_traversal_rejected() {
        let (temp, workspace) = setup_test_workspace();

        let result = resolve_workspace_path("../outside.txt", &workspace, PathIntent::AnyExisting);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_absolute_outside_rejected() {
        let (temp, workspace) = setup_test_workspace();

        #[cfg(windows)]
        let outside = "C:\\Windows\\System32\\notepad.exe";
        #[cfg(not(windows))]
        let outside = "/etc/passwd";

        let result = resolve_workspace_path(outside, &workspace, PathIntent::ExistingFile);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("越界"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_file_rejects_parent_symlink_escape() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside");
        std::fs::create_dir_all(&outside).expect("无法创建 outside 目录");
        let link = workspace.join("outside-link");
        create_dir_link(&link, &outside).expect("无法创建 workspace 外跳链接");

        let result =
            resolve_workspace_path("outside-link/newfile.txt", &workspace, PathIntent::NewFile);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("越界"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_create_workspace_dir_creates_directory() {
        let (temp, workspace) = setup_test_workspace();

        let safe_path = resolve_workspace_path("created", &workspace, PathIntent::NewDir).unwrap();
        let result = create_workspace_dir(&safe_path);

        assert!(result.is_ok());
        assert!(workspace.join("created").is_dir());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_rename_workspace_path_renames_file() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("old.txt"), "hello").expect("无法写入测试文件");

        let safe_old =
            resolve_workspace_path("old.txt", &workspace, PathIntent::AnyExisting).unwrap();
        let safe_new = resolve_workspace_path("new.txt", &workspace, PathIntent::NewFile).unwrap();
        let result = rename_workspace_path(&safe_old, &safe_new);

        assert!(result.is_ok());
        assert!(!workspace.join("old.txt").exists());
        assert_eq!(
            std::fs::read_to_string(workspace.join("new.txt")).unwrap(),
            "hello"
        );
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_remove_workspace_path_removes_file_even_when_recursive_true() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("delete-me.txt"), "hello").expect("无法写入测试文件");

        let safe_path =
            resolve_workspace_path("delete-me.txt", &workspace, PathIntent::AnyExisting).unwrap();
        let result = remove_workspace_path(&safe_path, true);

        assert!(result.is_ok());
        assert!(!workspace.join("delete-me.txt").exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_remove_workspace_path_removes_directory_recursive() {
        let (temp, workspace) = setup_test_workspace();
        let nested = workspace.join("delete-dir").join("nested");
        std::fs::create_dir_all(&nested).expect("无法创建测试目录");
        std::fs::write(nested.join("file.txt"), "hello").expect("无法写入测试文件");

        let safe_path =
            resolve_workspace_path("delete-dir", &workspace, PathIntent::AnyExisting).unwrap();
        let result = remove_workspace_path(&safe_path, true);

        assert!(result.is_ok());
        assert!(!workspace.join("delete-dir").exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_remove_workspace_path_rejects_non_empty_directory_without_recursive() {
        let (temp, workspace) = setup_test_workspace();
        let nested = workspace.join("non-empty");
        std::fs::create_dir_all(&nested).expect("无法创建测试目录");
        std::fs::write(nested.join("file.txt"), "hello").expect("无法写入测试文件");

        let safe_path =
            resolve_workspace_path("non-empty", &workspace, PathIntent::AnyExisting).unwrap();
        let result = remove_workspace_path(&safe_path, false);

        assert!(result.is_err());
        assert!(workspace.join("non-empty").exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_checkpoint_detail_matches_agent_loop_lowercase_store_event() {
        assert!(checkpoint_detail_mentions_checkpoint(
            "Storing checkpoint for iteration 2"
        ));
        assert!(checkpoint_detail_mentions_checkpoint(
            "Checkpoint abc123 after apply"
        ));
        assert!(!checkpoint_detail_mentions_checkpoint(
            "Stored memory summary without trace anchor"
        ));
    }

    fn sample_checkpoint(id: &str, files: Vec<CheckpointFileRef>) -> CheckpointRecord {
        CheckpointRecord {
            id: id.to_string(),
            timestamp: "2026-05-16T00:00:00Z".to_string(),
            label: id.to_string(),
            files,
            diff_summary: CheckpointDiffSummary {
                files_changed: 0,
                hunks: None,
                additions: None,
                deletions: None,
                summary: "sample".to_string(),
            },
            trace_event_ids: vec![format!("trace_{}", id)],
            metadata: CheckpointMetadata {
                source: "test".to_string(),
                agent_id: Some("agent".to_string()),
                iteration: 1,
                step_type: "EditApplied".to_string(),
                confidence: Some(1.0),
                schema_version: 1,
            },
        }
    }

    fn sample_file(path: &str, status: &str, hash: &str) -> CheckpointFileRef {
        CheckpointFileRef {
            path: path.to_string(),
            status: status.to_string(),
            before_hash: None,
            after_hash: Some(hash.to_string()),
            content: None,
            after_content: None,
        }
    }

    #[test]
    fn test_compare_checkpoint_records_classifies_file_changes() {
        let before = sample_checkpoint(
            "before",
            vec![
                sample_file("removed.txt", "modified", "old"),
                sample_file("changed.txt", "modified", "old"),
            ],
        );
        let after = sample_checkpoint(
            "after",
            vec![
                sample_file("added.txt", "created", "new"),
                sample_file("changed.txt", "modified", "new"),
            ],
        );

        let result = compare_checkpoint_records(&before, &after);

        assert!(!result.same);
        assert_eq!(result.files_added[0].path, "added.txt");
        assert_eq!(result.files_modified[0].path, "changed.txt");
        assert_eq!(result.files_removed[0].path, "removed.txt");
        assert_eq!(result.data_source, "checkpoint.files");
    }

    #[test]
    fn test_find_checkpoint_record_reports_missing_id() {
        let records = vec![sample_checkpoint("present", Vec::new())];
        let result = find_checkpoint_record(&records, "missing");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("checkpoint not found: missing"));
    }

    #[test]
    fn test_restore_plan_rejects_unsafe_path() {
        let (temp, workspace) = setup_test_workspace();
        let checkpoint = sample_checkpoint(
            "unsafe",
            vec![CheckpointFileRef {
                path: "../outside.txt".to_string(),
                status: "modified".to_string(),
                before_hash: None,
                after_hash: Some("hash".to_string()),
                content: Some("content".to_string()),
                after_content: None,
            }],
        );
        let result = build_restore_plan(&checkpoint, &workspace, &temp.join("backup"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("restore path rejected"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_restore_plan_warns_without_content_snapshot() {
        let (temp, workspace) = setup_test_workspace();
        let checkpoint = sample_checkpoint(
            "no-content",
            vec![sample_file("target.txt", "modified", "hash")],
        );
        let result = build_restore_plan(&checkpoint, &workspace, &temp.join("backup")).unwrap();

        assert_eq!(result.files[0].action, "write");
        assert!(!result.warnings.is_empty());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_apply_restore_plan_backs_up_and_writes_content_snapshot() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("target.txt"), "before").unwrap();
        let checkpoint = sample_checkpoint(
            "restore",
            vec![CheckpointFileRef {
                path: "target.txt".to_string(),
                status: "modified".to_string(),
                before_hash: None,
                after_hash: Some("after".to_string()),
                content: Some("after".to_string()),
                after_content: None,
            }],
        );
        let backup_dir = temp.join("backup");
        let plan = build_restore_plan(&checkpoint, &workspace, &backup_dir).unwrap();

        backup_restore_targets(&plan, &workspace, &backup_dir).unwrap();
        apply_restore_plan(&checkpoint, &plan, &workspace).unwrap();

        assert_eq!(
            std::fs::read_to_string(workspace.join("target.txt")).unwrap(),
            "after"
        );
        assert_eq!(
            std::fs::read_to_string(backup_dir.join("target.txt")).unwrap(),
            "before"
        );
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_restore_confirmation_required_for_write_restore() {
        let result = validate_restore_confirmation(false, false);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("confirmRestore must be true"));
        assert!(validate_restore_confirmation(false, true).is_ok());
        assert!(validate_restore_confirmation(true, false).is_ok());
    }

    #[test]
    fn test_restore_plan_rejects_missing_parent_before_writes() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("ok.txt"), "before").unwrap();
        let checkpoint = sample_checkpoint(
            "rollback",
            vec![
                CheckpointFileRef {
                    path: "ok.txt".to_string(),
                    status: "modified".to_string(),
                    before_hash: None,
                    after_hash: Some("after".to_string()),
                    content: Some("after".to_string()),
                    after_content: None,
                },
                CheckpointFileRef {
                    path: "missing-parent/fail.txt".to_string(),
                    status: "modified".to_string(),
                    before_hash: None,
                    after_hash: Some("after".to_string()),
                    content: Some("after".to_string()),
                    after_content: None,
                },
            ],
        );

        let backup_dir = temp.join("backup");
        let plan = build_restore_plan(&checkpoint, &workspace, &backup_dir);
        assert!(plan.is_err());
        assert_eq!(
            std::fs::read_to_string(workspace.join("ok.txt")).unwrap(),
            "before"
        );
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn provider_workspace_defaults_to_current_workspace_when_missing() {
        let (temp, workspace) = setup_test_workspace();

        let result = trusted_workspace_path_for_current(None, &workspace);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), workspace.canonicalize().unwrap());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn provider_workspace_accepts_current_workspace() {
        let (temp, workspace) = setup_test_workspace();

        let result =
            trusted_workspace_path_for_current(Some(&workspace.to_string_lossy()), &workspace);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), workspace.canonicalize().unwrap());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn provider_workspace_rejects_mismatched_workspace_without_path_leak() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside-workspace");
        std::fs::create_dir_all(&outside).expect("create outside workspace");

        let result =
            trusted_workspace_path_for_current(Some(&outside.to_string_lossy()), &workspace);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("workspace 参数越界"));
        assert!(!err.contains(&temp.to_string_lossy().to_string()));
        cleanup_test_workspace(&temp);
    }

    fn sample_provider_config(id: &str, name: &str) -> ProviderConfig {
        ProviderConfig {
            id: id.to_string(),
            name: name.to_string(),
            provider_type: "openai-compatible".to_string(),
            api_key: String::new(),
            base_url: "https://api.example.test/v1".to_string(),
            model: "example-model".to_string(),
            system_prompt: None,
            context_threshold: None,
            max_context_tokens: Some(128_000),
            max_output_tokens: Some(8_192),
            reserve_output_tokens: Some(1_024),
            safety_margin_tokens: Some(512),
            retrieval_budget_tokens: Some(4_096),
            long_context_mode: Some(false),
        }
    }

    #[test]
    fn provider_api_key_placeholders_are_not_treated_as_secret_updates() {
        assert_eq!(submitted_api_key(""), None);
        assert_eq!(submitted_api_key("sk-•••••••• (re-enter to update)"), None);
        assert_eq!(
            submitted_api_key("  real-secret-key  "),
            Some("real-secret-key")
        );
    }

    #[test]
    fn provider_config_view_exposes_key_presence_without_secret_value() {
        let mut cfg = sample_provider_config("provider-view", "Provider View");
        cfg.api_key = "must-not-serialize".to_string();

        let value = serde_json::to_value(ProviderConfigView::from_config(cfg, true))
            .expect("serialize provider config view");

        assert_eq!(value["hasApiKey"], true);
        assert!(value.get("apiKey").is_none());
    }

    #[test]
    fn provider_workspace_write_helpers_add_update_delete_current_workspace() {
        let (temp, workspace) = setup_test_workspace();
        let path = workspace_config_path(&workspace);

        add_workspace_provider_config_for_current(
            sample_provider_config("workspace-provider", "Original"),
            Some(&workspace.to_string_lossy()),
            &workspace,
        )
        .expect("add workspace provider");
        let configs = read_configs_at(&path);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "Original");

        update_workspace_provider_config_for_current(
            sample_provider_config("workspace-provider", "Updated"),
            Some(&workspace.to_string_lossy()),
            &workspace,
        )
        .expect("update workspace provider");
        let configs = read_configs_at(&path);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "Updated");

        delete_workspace_provider_config_for_current(
            "workspace-provider",
            Some(&workspace.to_string_lossy()),
            &workspace,
        )
        .expect("delete workspace provider");
        assert!(!path.exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn provider_workspace_write_helpers_reject_mismatched_workspace_without_writes() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside-workspace");
        std::fs::create_dir_all(&outside).expect("create outside workspace");
        let outside_config_path = workspace_config_path(&outside);

        let add_result = add_workspace_provider_config_for_current(
            sample_provider_config("outside-provider", "Outside"),
            Some(&outside.to_string_lossy()),
            &workspace,
        );
        assert!(add_result.is_err());
        assert!(add_result.unwrap_err().contains("workspace 参数越界"));
        assert!(!outside_config_path.exists());

        let update_result = update_workspace_provider_config_for_current(
            sample_provider_config("outside-provider", "Outside"),
            Some(&outside.to_string_lossy()),
            &workspace,
        );
        assert!(update_result.is_err());
        assert!(update_result.unwrap_err().contains("workspace 参数越界"));
        assert!(!outside_config_path.exists());

        let delete_result = delete_workspace_provider_config_for_current(
            "outside-provider",
            Some(&outside.to_string_lossy()),
            &workspace,
        );
        assert!(delete_result.is_err());
        assert!(delete_result.unwrap_err().contains("workspace 参数越界"));
        assert!(!outside_config_path.exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_provider_config_capabilities_serialization_compat() {
        let json_data = json!({
            "id": "test-provider",
            "name": "Test Provider",
            "providerType": "openai-compatible",
            "baseUrl": "https://api.test.com/v1",
            "model": "test-model",
            "maxContextTokens": 1000000,
            "maxOutputTokens": 8192,
            "reserveOutputTokens": 1024,
            "safetyMarginTokens": 500,
            "retrievalBudgetTokens": 4096,
            "longContextMode": true
        });

        let config: ProviderConfig = serde_json::from_value(json_data).unwrap();
        assert_eq!(config.id, "test-provider");
        assert_eq!(config.get_normalized_max_context_tokens(), Some(1000000));
        assert_eq!(config.max_output_tokens, Some(8192));
        assert_eq!(config.reserve_output_tokens, Some(1024));
        assert_eq!(config.safety_margin_tokens, Some(500));
        assert_eq!(config.retrieval_budget_tokens, Some(4096));
        assert_eq!(config.long_context_mode, Some(true));

        // Legacy compatibility test
        let legacy_json = json!({
            "id": "legacy-provider",
            "name": "Legacy",
            "providerType": "openai-compatible",
            "baseUrl": "https://api.legacy.com/v1",
            "model": "legacy-model",
            "contextThreshold": 8192
        });
        let legacy_config: ProviderConfig = serde_json::from_value(legacy_json).unwrap();
        assert_eq!(
            legacy_config.get_normalized_max_context_tokens(),
            Some(8192)
        );
        assert_eq!(legacy_config.max_context_tokens, None);
        assert_eq!(legacy_config.long_context_mode, None);
    }

    #[test]
    fn test_provider_backup_camelcase_serialization_and_deserialization() {
        // 1. Simulate export json format
        let cfg = ProviderConfig {
            id: "test-id".to_string(),
            name: "Test Name".to_string(),
            provider_type: "openai-compatible".to_string(),
            base_url: "http://test.url".to_string(),
            model: "test-model".to_string(),
            api_key: "test-key".to_string(),
            system_prompt: Some("test prompt".to_string()),
            context_threshold: Some(4096),
            max_context_tokens: Some(100000),
            max_output_tokens: Some(4000),
            reserve_output_tokens: Some(512),
            safety_margin_tokens: Some(256),
            retrieval_budget_tokens: Some(2048),
            long_context_mode: Some(true),
        };

        let exported_json = json!({
            "id": cfg.id,
            "name": cfg.name,
            "provider_type": cfg.provider_type,
            "base_url": cfg.base_url,
            "model": cfg.model,
            "api_key": cfg.api_key,
            "system_prompt": cfg.system_prompt,
            "contextThreshold": cfg.context_threshold,
            "maxContextTokens": cfg.max_context_tokens,
            "maxOutputTokens": cfg.max_output_tokens,
            "reserveOutputTokens": cfg.reserve_output_tokens,
            "safetyMarginTokens": cfg.safety_margin_tokens,
            "retrievalBudgetTokens": cfg.retrieval_budget_tokens,
            "longContextMode": cfg.long_context_mode,
        });

        // Verify keys in export format are camelCase for capability fields
        assert_eq!(
            exported_json
                .get("contextThreshold")
                .and_then(|v| v.as_u64()),
            Some(4096)
        );
        assert_eq!(
            exported_json
                .get("maxContextTokens")
                .and_then(|v| v.as_u64()),
            Some(100000)
        );
        assert_eq!(
            exported_json
                .get("reserveOutputTokens")
                .and_then(|v| v.as_u64()),
            Some(512)
        );
        assert_eq!(
            exported_json
                .get("longContextMode")
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        // 2. Simulate import deserialization with camelCase (the exported JSON)
        let imported_cfg_camel = ProviderConfig {
            id: exported_json["id"].as_str().unwrap_or("").to_string(),
            name: exported_json["name"].as_str().unwrap_or("").to_string(),
            provider_type: exported_json["provider_type"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            base_url: exported_json["base_url"].as_str().unwrap_or("").to_string(),
            model: exported_json["model"].as_str().unwrap_or("").to_string(),
            api_key: exported_json["api_key"].as_str().unwrap_or("").to_string(),
            system_prompt: exported_json
                .get("system_prompt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            context_threshold: exported_json
                .get("context_threshold")
                .or_else(|| exported_json.get("contextThreshold"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            max_context_tokens: exported_json
                .get("max_context_tokens")
                .or_else(|| exported_json.get("maxContextTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            max_output_tokens: exported_json
                .get("max_output_tokens")
                .or_else(|| exported_json.get("maxOutputTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            reserve_output_tokens: exported_json
                .get("reserve_output_tokens")
                .or_else(|| exported_json.get("reserveOutputTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            safety_margin_tokens: exported_json
                .get("safety_margin_tokens")
                .or_else(|| exported_json.get("safetyMarginTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            retrieval_budget_tokens: exported_json
                .get("retrieval_budget_tokens")
                .or_else(|| exported_json.get("retrievalBudgetTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            long_context_mode: exported_json
                .get("long_context_mode")
                .or_else(|| exported_json.get("longContextMode"))
                .and_then(|v| v.as_bool()),
        };

        assert_eq!(imported_cfg_camel.context_threshold, Some(4096));
        assert_eq!(imported_cfg_camel.max_context_tokens, Some(100000));
        assert_eq!(imported_cfg_camel.reserve_output_tokens, Some(512));
        assert_eq!(imported_cfg_camel.long_context_mode, Some(true));

        // 3. Simulate import deserialization with snake_case (legacy backups)
        let legacy_exported_json = json!({
            "id": cfg.id,
            "name": cfg.name,
            "provider_type": cfg.provider_type,
            "base_url": cfg.base_url,
            "model": cfg.model,
            "api_key": cfg.api_key,
            "system_prompt": cfg.system_prompt,
            "context_threshold": cfg.context_threshold,
            "max_context_tokens": cfg.max_context_tokens,
            "max_output_tokens": cfg.max_output_tokens,
            "reserve_output_tokens": cfg.reserve_output_tokens,
            "safety_margin_tokens": cfg.safety_margin_tokens,
            "retrieval_budget_tokens": cfg.retrieval_budget_tokens,
            "long_context_mode": cfg.long_context_mode,
        });

        let imported_cfg_snake = ProviderConfig {
            id: legacy_exported_json["id"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            name: legacy_exported_json["name"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            provider_type: legacy_exported_json["provider_type"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            base_url: legacy_exported_json["base_url"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model: legacy_exported_json["model"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            api_key: legacy_exported_json["api_key"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            system_prompt: legacy_exported_json
                .get("system_prompt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            context_threshold: legacy_exported_json
                .get("context_threshold")
                .or_else(|| legacy_exported_json.get("contextThreshold"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            max_context_tokens: legacy_exported_json
                .get("max_context_tokens")
                .or_else(|| legacy_exported_json.get("maxContextTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            max_output_tokens: legacy_exported_json
                .get("max_output_tokens")
                .or_else(|| legacy_exported_json.get("maxOutputTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            reserve_output_tokens: legacy_exported_json
                .get("reserve_output_tokens")
                .or_else(|| legacy_exported_json.get("reserveOutputTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            safety_margin_tokens: legacy_exported_json
                .get("safety_margin_tokens")
                .or_else(|| legacy_exported_json.get("safetyMarginTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            retrieval_budget_tokens: legacy_exported_json
                .get("retrieval_budget_tokens")
                .or_else(|| legacy_exported_json.get("retrievalBudgetTokens"))
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
            long_context_mode: legacy_exported_json
                .get("long_context_mode")
                .or_else(|| legacy_exported_json.get("longContextMode"))
                .and_then(|v| v.as_bool()),
        };

        assert_eq!(imported_cfg_snake.context_threshold, Some(4096));
        assert_eq!(imported_cfg_snake.max_context_tokens, Some(100000));
        assert_eq!(imported_cfg_snake.reserve_output_tokens, Some(512));
        assert_eq!(imported_cfg_snake.long_context_mode, Some(true));
    }

    #[tokio::test]
    async fn test_write_provider_caps_to_blackboard_and_leak_prevention() {
        let bb = agent_core::blackboard::Blackboard::new();
        let agent_id = "test-agent-123";

        // 1. Custom provider configuration
        let cfg = ProviderConfig {
            id: "custom-id".to_string(),
            name: "Custom Name".to_string(),
            provider_type: "openai-compatible".to_string(),
            base_url: "http://custom.url".to_string(),
            model: "custom-model".to_string(),
            api_key: "".to_string(),
            system_prompt: None,
            context_threshold: Some(4096),
            max_context_tokens: Some(128000),
            max_output_tokens: Some(4000),
            reserve_output_tokens: Some(512),
            safety_margin_tokens: Some(256),
            retrieval_budget_tokens: Some(2048),
            long_context_mode: Some(true),
        };

        write_provider_caps_to_blackboard(&bb, agent_id, "custom-id", Some(&cfg)).await;

        // Verify custom values are written to the blackboard
        assert_eq!(
            bb.read("__hajimi_provider_id").await.map(|e| e.value),
            Some("custom-id".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_model").await.map(|e| e.value),
            Some("custom-model".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_max_context_tokens")
                .await
                .map(|e| e.value),
            Some("128000".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_reserve_output_tokens")
                .await
                .map(|e| e.value),
            Some("512".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_long_context_mode").await.map(|e| e.value),
            Some("true".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_context_threshold").await.map(|e| e.value),
            Some("4096".to_string())
        );

        // 2. Built-in provider (clears capabilities to prevent stale leak)
        write_provider_caps_to_blackboard(&bb, agent_id, "openai", None).await;

        // Verify they are cleared
        assert_eq!(
            bb.read("__hajimi_provider_id").await.map(|e| e.value),
            Some("openai".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_model").await.map(|e| e.value),
            Some("".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_max_context_tokens")
                .await
                .map(|e| e.value),
            Some("".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_reserve_output_tokens")
                .await
                .map(|e| e.value),
            Some("".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_long_context_mode").await.map(|e| e.value),
            Some("".to_string())
        );
        assert_eq!(
            bb.read("__hajimi_context_threshold").await.map(|e| e.value),
            Some("".to_string())
        );
    }

    #[test]
    fn test_agent_ui_event_serialization() {
        let status_event = AgentUiEvent::Status {
            message: "Starting...".to_string(),
        };
        let status_json = serde_json::to_string(&status_event).unwrap();
        assert!(status_json.contains(r#""type":"status""#));
        assert!(status_json.contains(r#""message":"Starting...""#));

        let done_event = AgentUiEvent::Done;
        let done_json = serde_json::to_string(&done_event).unwrap();
        assert_eq!(done_json, r#"{"type":"done"}"#);
    }

    #[tokio::test]
    async fn test_capture_agent_trace() {
        use agent_core::llm_native::AgentTurnDriver;
        use chimera_repl::ReplResult;
        println!("🚀 Starting Real Agent Trace Capture E2E...");
        let providers_path = PathBuf::from(r"C:\Users\22129\AppData\Roaming\hajimi\providers.json");
        let content =
            std::fs::read_to_string(&providers_path).expect("Failed to read providers.json");
        let configs: Vec<ProviderConfig> =
            serde_json::from_str(&content).expect("Failed to parse providers.json");
        let cfg = configs
            .into_iter()
            .find(|c| c.name == "deepseek")
            .expect("No provider named deepseek found");
        println!("Loaded custom provider: {:?}", cfg);

        // Fetch api key
        let api_key =
            get_api_key_with_profile(&cfg.id, None).expect("Failed to fetch api key from keyring");
        println!(
            "Successfully fetched API Key from OS Keyring (length: {})",
            api_key.len()
        );

        let client = create_llm_client(&cfg.id, None, Some(cfg.clone()))
            .expect("Failed to create llm client");
        let client_arc: Arc<dyn engine_llm_core::LlmClient> = Arc::from(client);

        // Build empty tool registry
        let registry = Arc::new(tokio::sync::Mutex::new(ToolRegistry::new()));
        // Register write_file tool
        {
            let mut guard = registry.lock().await;
            guard.register(Arc::new(engine_tool_system::WriteFileTool::new()));
        }

        let driver = agent_core::llm_native::LlmNativeDriver::with_client(client_arc)
            .with_registry(registry);

        let intent = agent_core::llm_native::RawUserIntent::from_text(
            "请在当前项目根目录下创建一个名为 native-smoke.txt 的文件，文件内容写上 'Hajimi Agent Live!'",
            "session_capture_trace",
        );

        struct MockGov;
        #[async_trait]
        impl agent_core::governance::AgentGovernance for MockGov {
            async fn policy(
                &self,
                _ctx: &agent_core::AgentContext,
                _req: &agent_core::governance::GovernanceRequest,
            ) -> agent_core::governance::ApprovalLevel {
                agent_core::governance::ApprovalLevel::Auto
            }
            async fn approve(
                &self,
                _ctx: &agent_core::AgentContext,
                _req: &agent_core::governance::GovernanceRequest,
            ) -> ReplResult<agent_core::governance::Decision> {
                Ok(agent_core::governance::Decision::Approved)
            }
            async fn vote(
                &self,
                _voter_id: &str,
                _proposal_id: &str,
                _vote: agent_core::governance::Vote,
            ) -> ReplResult<()> {
                Ok(())
            }
            async fn escalate(
                &self,
                req: &agent_core::governance::GovernanceRequest,
                _to_level: agent_core::governance::ApprovalLevel,
            ) -> ReplResult<agent_core::governance::GovernanceRequest> {
                Ok(req.clone())
            }
            async fn register_policy(
                &mut self,
                _name: &str,
                _policy: Arc<dyn agent_core::governance::GovernancePolicy>,
                _caller: &str,
                _required_level: agent_core::governance::PermissionLevel,
            ) -> ReplResult<()> {
                Ok(())
            }
            async fn record_feedback(
                &self,
                _ctx: &agent_core::AgentContext,
                _feedback: &agent_core::governance::UserFeedback,
            ) -> ReplResult<()> {
                Ok(())
            }
        }
        let governance = Arc::new(MockGov);

        let cancellation = agent_core::llm_native::CancellationToken::new();

        let tools = vec![agent_core::llm_native::ModelVisibleToolSpec {
            name: "write_file".to_string(),
            namespace: None,
            description: "Write content into a file".to_string(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
            supports_parallel: true,
            risk_level: agent_core::llm_native::RiskLevel::Low,
        }];

        println!("Executing real llm_native_turn directly...");
        let result = driver
            .run_turn(intent, tools, vec![], governance, cancellation)
            .await;
        match result {
            Ok(outcome) => {
                println!("🎉 Real E2E outcome success: {}", outcome.success);
                println!("Iterations: {}", outcome.iterations);
                println!("Final Message: {:?}", outcome.final_message);
            }
            Err(e) => {
                println!("❌ Real E2E error encountered: {:?}", e);
            }
        }
    }
}
