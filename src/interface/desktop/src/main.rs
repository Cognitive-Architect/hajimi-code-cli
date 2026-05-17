#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use agent_core::agent_loop::TraceStepType;
use agent_core::{
    AgentContext, AgentLoopBuilder, AutonomousReflector, HierarchicalPlanner, TraceEvent,
};
use codex_twist::memory::{MemoryGateway, MemoryTier, TokenBudget, TokenUsageTracker};
use engine_llm_core::{
    AnthropicClient, ChatMessage, Client, LlmClient, OllamaClient, OpenAiClient,
};
use engine_tool_system::lsp_integration::ASTContextProvider;
use engine_tool_system::{
    AnalyzeTool, BashTool, BenchmarkTool, CargoBuildTool, CmakeTool, CoverageReportTool,
    DeleteFileTool, EditFileTool, FetchUrlTool, FindTool, GenerateDocsTool,
    GeneratePrDescriptionTool, GitCommitTool, GitDiffTool, GitLogTool, GitStatusTool, GlobTool,
    GraphTool, GrepTool, JsBundleAnalyzerTool, ListDirectoryTool, LsTool, LspDefinitionTool,
    LspHoverTool, LspInitTool, LspReferencesTool, MakeTool, McpInitTool, McpInvokeTool, NpmRunTool,
    PowerShellTool, ReadFileTool, RefactorCodeTool, RunTestsTool, RustDocGeneratorTool,
    SecurityAuditTool, SmartCommitTool, ToolOutput, ToolRegistry, UpdateReadmeTool, ViewImageTool,
    WebSearchTool, WriteFileTool,
};
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

mod audit;

// ------------------------------------------------------------------
// App State
// ------------------------------------------------------------------
/// Phase 4 Day 5: Edit history entry for timeline visualization.
#[derive(Clone, serde::Serialize)]
struct EditHistoryEntry {
    id: String,
    timestamp: String,
    step_type: String,
    summary: String,
    confidence: Option<f32>,
    token_before: Option<usize>,
    token_after: Option<usize>,
    checkpoint_id: Option<String>,
}

/// Day 08 checkpoint file reference for export/compare contracts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointFileRef {
    path: String,
    status: String,
    before_hash: Option<String>,
    after_hash: Option<String>,
    content: Option<String>,
    after_content: Option<String>,
}

/// Day 08 checkpoint diff summary. Detailed hunks are deferred to Day 09.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointDiffSummary {
    files_changed: usize,
    hunks: Option<usize>,
    additions: Option<usize>,
    deletions: Option<usize>,
    summary: String,
}

/// Day 08 checkpoint metadata for trace linkage and schema evolution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointMetadata {
    source: String,
    agent_id: Option<String>,
    iteration: usize,
    step_type: String,
    confidence: Option<f32>,
    schema_version: u32,
}

/// Minimal desktop-local checkpoint DTO for Day 09 export/compare.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointRecord {
    id: String,
    timestamp: String,
    label: String,
    files: Vec<CheckpointFileRef>,
    diff_summary: CheckpointDiffSummary,
    trace_event_ids: Vec<String>,
    metadata: CheckpointMetadata,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CheckpointExportBundle {
    schema_version: u32,
    exported_at: String,
    workspace: String,
    checkpoints: Vec<CheckpointRecord>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct CheckpointFileChange {
    path: String,
    before_status: Option<String>,
    after_status: Option<String>,
    before_hash: Option<String>,
    after_hash: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct CheckpointCompareResult {
    id_a: String,
    id_b: String,
    same: bool,
    files_added: Vec<CheckpointFileChange>,
    files_removed: Vec<CheckpointFileChange>,
    files_modified: Vec<CheckpointFileChange>,
    summary: String,
    data_source: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct RestoreFilePlan {
    path: String,
    action: String,
    target_exists: bool,
    backup_path: Option<String>,
    reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct RestoreResult {
    checkpoint_id: String,
    restored_at: String,
    dry_run: bool,
    backup_dir: String,
    files: Vec<RestoreFilePlan>,
    warnings: Vec<String>,
}

struct AppState {
    registry: ToolRegistry,
    active_profile: std::sync::Mutex<Option<String>>,
    agent_providers: std::sync::Mutex<HashMap<String, String>>,
    trace_tx: std::sync::Mutex<Option<tokio::sync::broadcast::Sender<TraceEvent>>>,
    paused: std::sync::Mutex<bool>,
    approval_level: std::sync::Mutex<String>,
    edit_history: Arc<tokio::sync::Mutex<Vec<EditHistoryEntry>>>,
    memory_gateway: Arc<MemoryGateway>,
    token_tracker: Arc<TokenUsageTracker>,
}

impl AppState {
    /// Inject the AgentLoop broadcast sender to enable trace event streaming.
    /// Call this after `AgentLoop::from_components()` creates the broadcast channel.
    pub fn set_trace_tx(&self, tx: tokio::sync::broadcast::Sender<TraceEvent>) {
        // SAFETY: trace_tx is thread-safe via Mutex; poison recovery via into_inner()
        *self.trace_tx.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    }
}

fn build_registry() -> ToolRegistry {
    let mut r = ToolRegistry::new();
    r.register(Arc::new(AnalyzeTool::new()));
    r.register(Arc::new(BashTool::new()));
    r.register(Arc::new(CargoBuildTool::new()));
    r.register(Arc::new(CmakeTool::new()));
    r.register(Arc::new(DeleteFileTool::new()));
    r.register(Arc::new(EditFileTool::new()));
    r.register(Arc::new(FetchUrlTool::new()));
    r.register(Arc::new(FindTool::new()));
    r.register(Arc::new(GenerateDocsTool::new()));
    r.register(Arc::new(GitCommitTool::new()));
    r.register(Arc::new(GitDiffTool::new()));
    r.register(Arc::new(GitLogTool::new()));
    r.register(Arc::new(GitStatusTool::new()));
    r.register(Arc::new(SmartCommitTool::new()));
    r.register(Arc::new(GeneratePrDescriptionTool::new()));
    r.register(Arc::new(GlobTool::new()));
    r.register(Arc::new(GraphTool::new()));
    r.register(Arc::new(GrepTool::new()));
    r.register(Arc::new(JsBundleAnalyzerTool::new()));
    r.register(Arc::new(ListDirectoryTool::new()));
    r.register(Arc::new(LspDefinitionTool::new()));
    r.register(Arc::new(LspHoverTool::new()));
    r.register(Arc::new(LspInitTool::new()));
    r.register(Arc::new(LspReferencesTool::new()));
    r.register(Arc::new(LsTool::new()));
    r.register(Arc::new(MakeTool::new()));
    r.register(Arc::new(McpInitTool::new()));
    r.register(Arc::new(McpInvokeTool::new()));
    r.register(Arc::new(CoverageReportTool::new()));
    r.register(Arc::new(BenchmarkTool::new()));
    r.register(Arc::new(NpmRunTool::new()));
    r.register(Arc::new(PowerShellTool::new()));
    r.register(Arc::new(ReadFileTool::new()));
    r.register(Arc::new(RefactorCodeTool::new()));
    r.register(Arc::new(RunTestsTool::new()));
    r.register(Arc::new(RustDocGeneratorTool::new()));
    r.register(Arc::new(SecurityAuditTool::new()));
    r.register(Arc::new(UpdateReadmeTool::new()));
    r.register(Arc::new(ViewImageTool::new()));
    r.register(Arc::new(WebSearchTool::new()));
    r.register(Arc::new(WriteFileTool::new()));
    r
}

// ------------------------------------------------------------------
// Security constants (B-01/04, B-02/04)
// ------------------------------------------------------------------
const ALLOWED_COMMANDS: &[&str] = &[
    "git",
    "cargo",
    "npm",
    "node",
    "npx",
    "pnpm",
    "rustc",
    "rustfmt",
    "clippy-driver",
    "python",
    "python3",
    "pip",
    "pip3",
    "code",
    "cursor",
];

const FORBIDDEN_CHARS: &[char] = &[';', '&', '|', '`', '$', '(', ')', '{', '}', '<', '>'];

// ------------------------------------------------------------------
// Legacy commands
// ------------------------------------------------------------------
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust.", name)
}

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

fn checkpoint_store_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
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

fn checkpoint_record_from_trace(event: &TraceEvent) -> CheckpointRecord {
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

fn is_checkpoint_store_trace(event: &TraceEvent) -> bool {
    event.step_type == TraceStepType::Store && checkpoint_detail_mentions_checkpoint(&event.details)
}

fn write_checkpoint_record(
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

/// 路径意图类型，决定 canonicalize 策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathIntent {
    /// 目标必须已存在且为文件
    ExistingFile,
    /// 目标必须已存在且为目录
    ExistingDir,
    /// 目标是新文件，只需父目录存在
    NewFile,
    /// 目标是新目录，只需父目录存在
    NewDir,
    /// 目标可存在可不存在（任意类型）
    AnyExisting,
}

/// 安全解析 workspace 内路径，防止 symlink 逃逸和 traversal 攻击
fn resolve_workspace_path(
    input: &str,
    base_dir: &Path,
    intent: PathIntent,
) -> Result<PathBuf, String> {
    // 1. 拒绝显式 traversal
    if input.contains("..") {
        return Err("路径包含非法 traversal: ..".to_string());
    }

    // 2. 解析输入路径
    let input_path = Path::new(input);
    let resolved = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        base_dir.join(input_path)
    };

    // 3. canonicalize base_dir（必须存在）
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("无法解析工作目录: {}", e))?;

    // 4. 根据 intent 决定 canonicalize 策略
    let canonical = match intent {
        PathIntent::ExistingFile | PathIntent::ExistingDir | PathIntent::AnyExisting => {
            // existing 路径必须 canonicalize 目标本身
            resolved
                .canonicalize()
                .map_err(|e| format!("无法解析目标路径: {}", e))?
        }
        PathIntent::NewFile | PathIntent::NewDir => {
            // new 路径只 canonicalize 父目录
            let parent = resolved
                .parent()
                .ok_or_else(|| "无法获取父目录".to_string())?;
            if !parent.exists() {
                return Err(format!("父目录不存在: {}", parent.display()));
            }
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("无法解析父目录: {}", e))?;
            // 拼接 leaf name
            canonical_parent.join(
                resolved
                    .file_name()
                    .ok_or_else(|| "无法获取文件名".to_string())?,
            )
        }
    };

    // 5. 确认在 workspace 内
    if !canonical.starts_with(&canonical_base) {
        return Err(format!(
            "路径越界: {} 不在工作目录 {} 内",
            canonical.display(),
            canonical_base.display()
        ));
    }

    match intent {
        PathIntent::ExistingFile if !canonical.is_file() => {
            return Err(format!("目标不是文件: {}", canonical.display()));
        }
        PathIntent::ExistingDir if !canonical.is_dir() => {
            return Err(format!("目标不是目录: {}", canonical.display()));
        }
        _ => {}
    }

    Ok(canonical)
}

#[tauri::command]
fn read_file(path: &str, app_handle: tauri::AppHandle) -> Result<String, String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::ExistingFile)?;
    std::fs::read_to_string(&safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_file(path: &str, content: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::NewFile)?;
    // 确保父目录存在
    if let Some(parent) = safe_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&safe_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_dir(path: &str, app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::ExistingDir)?;
    let entries = std::fs::read_dir(&safe_path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    Ok(entries)
}

fn create_workspace_dir(safe_path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(safe_path).map_err(|e| e.to_string())
}

fn rename_workspace_path(safe_old: &Path, safe_new: &Path) -> Result<(), String> {
    std::fs::rename(safe_old, safe_new).map_err(|e| e.to_string())
}

fn remove_workspace_path(safe_path: &Path, recursive: bool) -> Result<(), String> {
    if safe_path.is_dir() {
        if recursive {
            std::fs::remove_dir_all(safe_path).map_err(|e| e.to_string())
        } else {
            std::fs::remove_dir(safe_path).map_err(|e| e.to_string())
        }
    } else if safe_path.is_file() {
        std::fs::remove_file(safe_path).map_err(|e| e.to_string())
    } else {
        Err(format!("目标不是文件或目录: {}", safe_path.display()))
    }
}

#[tauri::command]
fn create_dir(path: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::NewDir)?;
    create_workspace_dir(&safe_path)
}

#[tauri::command]
fn rename_path(old_path: &str, new_path: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    // 源路径必须存在
    let safe_old = resolve_workspace_path(old_path, &base_dir, PathIntent::AnyExisting)?;
    // 目标路径的父目录必须在 workspace 内
    let safe_new = resolve_workspace_path(new_path, &base_dir, PathIntent::NewFile)?;
    rename_workspace_path(&safe_old, &safe_new)
}

#[tauri::command]
fn delete_path(path: &str, recursive: bool, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::AnyExisting)?;
    remove_workspace_path(&safe_path, recursive)
}

#[tauri::command]
fn run_command(cmd: &str, args: Vec<String>) -> Result<String, String> {
    // 1. 命令白名单校验
    if !ALLOWED_COMMANDS.contains(&cmd) {
        return Err(format!("命令 '{}' 不在白名单中", cmd));
    }

    // 2. 参数元字符过滤
    for arg in &args {
        if arg.contains("..") || arg.contains(FORBIDDEN_CHARS) {
            return Err(format!("参数包含非法字符: {}", arg));
        }
    }

    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(format!(
            "exit code {:?}\nstderr: {}",
            output.status.code(),
            stderr
        ));
    }
    Ok(stdout)
}

// ------------------------------------------------------------------
// Tool-system commands
// ------------------------------------------------------------------
#[derive(Serialize, Clone)]
struct ToolInfo {
    name: String,
    description: String,
}

#[derive(Serialize, Clone)]
struct ToolResult {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
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

#[tauri::command]
fn list_tools(state: tauri::State<'_, AppState>) -> Vec<ToolInfo> {
    state
        .registry
        .list()
        .into_iter()
        .filter_map(|name| {
            state.registry.get(name).map(|t| ToolInfo {
                name: name.to_string(),
                description: t.description().to_string(),
            })
        })
        .collect()
}

#[tauri::command]
async fn execute_tool(
    state: tauri::State<'_, AppState>,
    name: String,
    args: Value,
) -> Result<ToolResult, String> {
    let tool = state
        .registry
        .get(&name)
        .ok_or_else(|| format!("tool '{}' not found", name))?;
    let output = tool.execute(args).await.map_err(|e| e.message)?;
    Ok(output.into())
}

// ------------------------------------------------------------------
// LLM commands
// ------------------------------------------------------------------
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StreamEvent {
    chunk: String,
    done: bool,
    error: Option<String>,
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
}

#[derive(Serialize, Clone)]
struct ProviderInfo {
    name: String,
    available: bool,
    default_model: String,
}

// ------------------------------------------------------------------
// Provider Config (custom OpenAI-compatible providers)
// ------------------------------------------------------------------
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderConfig {
    id: String,
    name: String,
    provider_type: String,
    #[serde(skip_serializing, default)]
    api_key: String,
    base_url: String,
    model: String,
    #[serde(default)]
    system_prompt: Option<String>,
    #[serde(default)]
    context_threshold: Option<usize>,
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

fn provider_config_path() -> PathBuf {
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
fn workspace_config_path(workspace: &str) -> PathBuf {
    PathBuf::from(workspace)
        .join(".hajimi")
        .join("providers.json")
}

// Profile-level config lives in profiles/{name}/providers.json (B-05/01)
fn profile_config_path(name: &str) -> PathBuf {
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

fn sanitize_profile_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Profile name contains illegal characters".to_string());
    }
    Ok(name.to_string())
}

fn read_configs_at(path: &std::path::Path) -> Vec<ProviderConfig> {
    if !path.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn read_merged_configs(workspace: Option<&str>, profile: Option<&str>) -> Vec<ProviderConfig> {
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

#[allow(dead_code)]
fn save_api_key(id: &str, api_key: &str) -> Result<(), String> {
    save_api_key_with_profile(id, api_key, None)
}

fn save_api_key_with_profile(id: &str, api_key: &str, profile: Option<&str>) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Ok(());
    }
    let entry = Entry::new("hajimi", &keyring_entry_id(id, profile))
        .map_err(|e| format!("keyring entry failed: {}", e))?;
    entry
        .set_password(api_key)
        .map_err(|e| format!("keyring set failed: {}", e))?;
    Ok(())
}

#[allow(dead_code)]
fn get_api_key(id: &str) -> Result<String, String> {
    get_api_key_with_profile(id, None)
}

fn get_api_key_with_profile(id: &str, profile: Option<&str>) -> Result<String, String> {
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

fn delete_api_key_with_profile(id: &str, profile: Option<&str>) -> Result<(), String> {
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
        if !cfg.api_key.trim().is_empty() {
            save_api_key_with_profile(&cfg.id, &cfg.api_key, profile)?;
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

fn read_provider_configs_with_profile(profile: Option<&str>) -> Vec<ProviderConfig> {
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

fn write_provider_configs_with_profile(
    profile: Option<&str>,
    configs: &[ProviderConfig],
) -> Result<(), String> {
    let path = match profile {
        None | Some("default") | Some("") => provider_config_path(),
        Some(p) => profile_config_path(p),
    };
    write_configs_to_path(&path, configs)
}

fn write_configs_to_path(path: &std::path::Path, configs: &[ProviderConfig]) -> Result<(), String> {
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

fn encrypt_backup(plaintext: &str, password: &str) -> Result<Vec<u8>, String> {
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

fn decrypt_backup(data: &[u8], password: &str) -> Result<String, String> {
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

#[tauri::command]
fn get_provider_configs(
    workspace_path: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Vec<ProviderConfig> {
    // SAFETY: Mutex held only for config read; poison unlikely in single-threaded Tauri command context
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    read_merged_configs(workspace_path.as_deref(), profile.as_deref())
}

#[tauri::command]
fn add_provider_config(
    mut config: ProviderConfig,
    workspace_path: Option<String>,
    save_target: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // SAFETY: Mutex held only for config read; poison unlikely in single-threaded Tauri command context
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if !config.api_key.trim().is_empty() {
        save_api_key_with_profile(&config.id, &config.api_key, profile.as_deref())?;
    }
    config.api_key.clear();
    let target = save_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        if let Some(ws) = workspace_path.as_deref() {
            let path = workspace_config_path(ws);
            let mut configs = read_configs_at(&path);
            if configs.iter().any(|c| c.id == config.id) {
                return Err(format!("Provider '{}' already exists", config.id));
            }
            configs.push(config);
            return write_configs_to_path(&path, &configs);
        }
    }
    let mut configs = read_provider_configs_with_profile(profile.as_deref());
    if configs.iter().any(|c| c.id == config.id) {
        return Err(format!("Provider '{}' already exists", config.id));
    }
    configs.push(config);
    write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
fn update_provider_config(
    mut config: ProviderConfig,
    workspace_path: Option<String>,
    save_target: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if !config.api_key.trim().is_empty() {
        save_api_key_with_profile(&config.id, &config.api_key, profile.as_deref())?;
    }
    config.api_key.clear();
    let target = save_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        if let Some(ws) = workspace_path.as_deref() {
            let path = workspace_config_path(ws);
            let mut configs = read_configs_at(&path);
            let idx = configs
                .iter()
                .position(|c| c.id == config.id)
                .ok_or_else(|| format!("Provider '{}' not found", config.id))?;
            configs[idx] = config;
            return write_configs_to_path(&path, &configs);
        }
    }
    let mut configs = read_provider_configs_with_profile(profile.as_deref());
    let idx = configs
        .iter()
        .position(|c| c.id == config.id)
        .ok_or_else(|| format!("Provider '{}' not found", config.id))?;
    configs[idx] = config;
    write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
fn delete_provider_config(
    id: String,
    workspace_path: Option<String>,
    delete_target: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let _ = delete_api_key_with_profile(&id, profile.as_deref());
    let target = delete_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        if let Some(ws) = workspace_path.as_deref() {
            let path = workspace_config_path(ws);
            let mut configs = read_configs_at(&path);
            configs.retain(|c| c.id != id);
            if configs.is_empty() {
                let _ = std::fs::remove_file(&path);
                return Ok(());
            } else {
                return write_configs_to_path(&path, &configs);
            }
        }
    }
    let mut configs = read_provider_configs_with_profile(profile.as_deref());
    configs.retain(|c| c.id != id);
    write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
fn get_providers(
    workspace_path: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Vec<ProviderInfo> {
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

    // Official providers now unified with config + keyring fallback to env (P0-2)
    let anthropic_key_ok = std::env::var("ANTHROPIC_API_KEY").is_ok()
        || get_api_key_with_profile("anthropic", profile.as_deref()).is_ok();
    providers.push(ProviderInfo {
        name: "anthropic".into(),
        available: anthropic_key_ok,
        default_model: "claude-3-5-sonnet-20241022".into(),
    });

    let openai_key_ok = std::env::var("OPENAI_API_KEY").is_ok()
        || get_api_key_with_profile("openai", profile.as_deref()).is_ok();
    providers.push(ProviderInfo {
        name: "openai".into(),
        available: openai_key_ok,
        default_model: "gpt-4o".into(),
    });

    // Append custom providers from config (keys secured in keyring), with workspace overlay
    for cfg in read_merged_configs(workspace_path.as_deref(), profile.as_deref()) {
        let is_official = cfg.id == "anthropic"
            || cfg.id == "openai"
            || cfg.name.to_lowercase() == "anthropic"
            || cfg.name.to_lowercase() == "openai";
        if !is_official {
            let available = get_api_key_with_profile(&cfg.id, profile.as_deref()).is_ok()
                || !cfg.api_key.trim().is_empty();
            providers.push(ProviderInfo {
                name: cfg.id.clone(),
                available,
                default_model: cfg.model.clone(),
            });
        }
    }
    providers
}

#[tauri::command]
fn get_current_workspace(app_handle: tauri::AppHandle) -> Option<String> {
    get_workspace_dir(&app_handle)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// # Safety: API key from OS keyring, response validated via real HTTP before UI green status
#[tauri::command]
async fn validate_provider(
    config: ProviderConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let key = if config.api_key.trim().is_empty() {
        get_api_key_with_profile(&config.id, profile.as_deref())?
    } else {
        config.api_key
    };
    if key.trim().is_empty() {
        return Err("No API key available in keyring or config".to_string());
    }
    // Real HTTP validation (5s timeout) with fallback to format check
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
    // Normalize base URL: avoid double /v1 if base_url already ends with /v1
    let base_trimmed = base.trim_end_matches('/');
    let chat_url = if base_trimmed.ends_with("/v1") {
        format!("{}/chat/completions", base_trimmed)
    } else {
        format!("{}/v1/chat/completions", base_trimmed)
    };
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
                // 400 usually means auth passed but model name or params invalid
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
            // fallback to format check
            if key.starts_with("sk-") || key.len() > 15 {
                Ok(format!("⚠️ {} 网络无法到达，Key 格式检查通过", config.name))
            } else {
                Err(format!("连接失败: {}", e))
            }
        }
    }
}

fn create_llm_client(
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
            let api_key = if cfg.api_key.trim().is_empty() {
                get_api_key_with_profile(&cfg.id, profile)
                    .map_err(|e| format!("Failed to retrieve key for {}: {}", cfg.id, e))?
            } else {
                cfg.api_key
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

#[tauri::command]
async fn stream_chat(
    provider: String,
    prompt: String,
    messages: Option<Vec<ChatMessage>>,
    config: Option<ProviderConfig>,
    on_event: Channel<StreamEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_default();
    let system_prompt = config.as_ref().and_then(|c| c.system_prompt.clone());

    let msg_count = messages.as_ref().map(|m| m.len()).unwrap_or(1);

    let chat_result = async {
        let client = create_llm_client(&provider, profile.as_deref(), config)?;

        let msgs = if let Some(msgs) = messages.filter(|m| !m.is_empty()) {
            msgs
        } else {
            vec![ChatMessage {
                role: "user".into(),
                content: prompt,
                timestamp: None,
            }]
        };
        let msgs_for_opt = msgs.clone();

        let gateway = state.memory_gateway.clone();
        let token_tracker = state.token_tracker.clone();
        let session_key = format!("chat:{}:{}", provider, chrono::Utc::now().timestamp());
        let ctx_json = serde_json::to_string(&msgs).map_err(|e| e.to_string())?;

        // SAFETY: MemoryGateway uses Arc<RwLock> internally; concurrent access is safe across Tauri commands
        let _ = gateway.working().put(session_key.clone(), ctx_json).await;

        let stats_before = gateway.stats().await;
        let token_before = stats_before.working_tokens as u64;
        let precise_prompt_start = client
            .count_tokens(msgs_for_opt.clone(), &model)
            .ok()
            .map(|n| n as u64);

        // Audit: stream started (B-05/03)
        let _ = audit::log_usage(&audit::KeyUsageRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider_name: provider.clone(),
            model: model.clone(),
            status: "started".into(),
            estimated_tokens: Some(msg_count as u64 * 50),
            precise_prompt: precise_prompt_start,
            precise_completion: None,
            token_before: Some(token_before),
            token_after: None,
        });

        let mut stream = client
            .stream_chat_with_context(msgs, system_prompt)
            .await
            .map_err(|e| format!("stream start failed: {}", e))?;

        while let Some(chunk) = stream.next().await {
            let (text, is_done, is_error) = match chunk {
                engine_llm_core::StreamChunk::Output(t) => (t, false, false),
                engine_llm_core::StreamChunk::Error(e) => (e, false, true),
                engine_llm_core::StreamChunk::Done => (String::new(), true, false),
            };
            let usage = if is_done { client.last_usage() } else { None };
            on_event
                .send(StreamEvent {
                    chunk: text,
                    done: is_done,
                    error: if is_error {
                        Some("LLM error".into())
                    } else {
                        None
                    },
                    prompt_tokens: usage.as_ref().map(|u| u.prompt_tokens),
                    completion_tokens: usage.as_ref().map(|u| u.completion_tokens),
                })
                .map_err(|e| e.to_string())?;
            if is_done {
                break;
            }
        }

        let usage = client.last_usage();

        // Record token usage for persistent cumulative tracking (P1-02/05)
        if let Some(ref u) = usage {
            token_tracker
                .record_usage(
                    &session_key,
                    &provider,
                    u.prompt_tokens,
                    u.completion_tokens,
                )
                .await;
        }

        // Trigger compression via LLM-driven summary
        let _ = gateway.optimize(msgs_for_opt, client.as_ref()).await;
        let stats_after = gateway.stats().await;
        let token_after = stats_after.working_tokens as u64;

        // Verify context is retrievable
        let _retrieved = gateway.working().get(&session_key).await;

        Ok((token_before, token_after, usage))
    }
    .await;

    let (chat_result, token_before_val, token_after_val, usage_val) = match chat_result {
        Ok((tb, ta, u)) => (Ok(()), tb, ta, u),
        Err(e) => (Err(e), 0, 0, None),
    };

    let (precise_prompt_end, precise_completion_end) = if let Some(u) = usage_val {
        (Some(u.prompt_tokens), Some(u.completion_tokens))
    } else {
        (None, None)
    };

    // Audit: completed or failed (B-05/03)
    let _ = audit::log_usage(&audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider,
        model,
        status: if chat_result.is_ok() {
            "completed".into()
        } else {
            "failed".into()
        },
        estimated_tokens: Some(msg_count as u64 * 50),
        precise_prompt: precise_prompt_end,
        precise_completion: precise_completion_end,
        token_before: Some(token_before_val),
        token_after: Some(token_after_val),
    });

    chat_result
}

#[tauri::command]
async fn compact_context(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let gateway = state.memory_gateway.clone();
    gateway.working().compact().await;
    let stats = gateway.stats().await;
    Ok(format!(
        "工作内存: {} 条目, {} tokens",
        stats.working_entries, stats.working_tokens
    ))
}

#[tauri::command]
async fn optimize_context(
    messages: Vec<ChatMessage>,
    provider: String,
    config: Option<ProviderConfig>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let client = create_llm_client(&provider, profile.as_deref(), config)?;
    let gateway = state.memory_gateway.clone();
    gateway.optimize(messages, client.as_ref()).await
}

#[tauri::command]
fn export_provider_backup(
    password: String,
    workspace_path: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let configs = read_merged_configs(workspace_path.as_deref(), profile.as_deref());
    let mut export_data = Vec::new();
    for cfg in configs {
        let key = get_api_key_with_profile(&cfg.id, profile.as_deref()).unwrap_or_default();
        export_data.push(json!({
            "id": cfg.id, "name": cfg.name, "provider_type": cfg.provider_type,
            "base_url": cfg.base_url, "model": cfg.model, "api_key": key,
        }));
    }
    let plaintext = serde_json::to_string(&export_data).map_err(|e| e.to_string())?;
    let encrypted = encrypt_backup(&plaintext, &password)?;
    let path = provider_config_path().with_extension("hajimi-backup");
    std::fs::write(&path, encrypted).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn import_provider_backup(
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
    let plaintext = decrypt_backup(&encrypted, &password)?;
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
                .and_then(|v| v.as_u64())
                .map(|n| n as usize),
        };
        if !cfg.api_key.trim().is_empty() {
            save_api_key_with_profile(&cfg.id, &cfg.api_key, profile.as_deref())?;
        }
        let mut sanitized = cfg.clone();
        sanitized.api_key.clear();
        let mut existing = read_provider_configs_with_profile(profile.as_deref());
        if let Some(idx) = existing.iter().position(|c| c.id == sanitized.id) {
            existing[idx] = sanitized;
        } else {
            existing.push(sanitized);
        }
        write_provider_configs_with_profile(profile.as_deref(), &existing)?;
        count += 1;
    }
    Ok(count)
}

// ------------------------------------------------------------------
// Profile commands (B-05/01)
// ------------------------------------------------------------------
#[tauri::command]
fn list_profiles() -> Result<Vec<String>, String> {
    let dir = if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
            .join("Hajimi")
            .join("profiles")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Library/Application Support/Hajimi/profiles")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config/hajimi/profiles")
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(names)
}

#[tauri::command]
fn get_active_profile(state: tauri::State<'_, AppState>) -> Option<String> {
    state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
fn set_active_profile(
    name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *profile = name;
    Ok(())
}

#[tauri::command]
fn create_profile(name: String) -> Result<(), String> {
    let name = sanitize_profile_name(&name)?;
    let path = profile_config_path(&name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !path.exists() {
        std::fs::write(&path, "[]").map_err(|e| e.to_string())?;
    }
    write_configs_to_path(&path, &[])?;
    Ok(())
}

#[tauri::command]
fn delete_profile(name: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let name = sanitize_profile_name(&name)?;
    // Clear active profile if deleting current
    {
        let mut active = state
            .active_profile
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if active.as_deref() == Some(&name) {
            *active = None;
        }
    }
    let path = profile_config_path(&name);
    if path.exists() {
        // Delete config file
        let _ = std::fs::remove_file(&path);
        // Delete profile directory
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
    // Clean up keyring entries for this profile (best effort), format: provider:{profile}:{id}
    let configs = read_provider_configs_with_profile(Some(&name));
    for cfg in configs {
        let _ = delete_api_key_with_profile(&cfg.id, Some(&name));
    }
    Ok(())
}

// ------------------------------------------------------------------
// Agent provider commands (B-05/02)
// ------------------------------------------------------------------
#[tauri::command]
fn get_agent_providers(
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    // SAFETY: Mutex held only for HashMap clone; poison unlikely in single-threaded Tauri command context
    let map = state
        .agent_providers
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    Ok(map)
}

#[tauri::command]
fn set_agent_provider(
    agent_id: String,
    provider_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // SAFETY: Mutex held only for HashMap insert/remove; poison unlikely in single-threaded Tauri command context
    let mut map = state
        .agent_providers
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(pid) = provider_id {
        map.insert(agent_id, pid);
    } else {
        map.remove(&agent_id);
    }
    Ok(())
}

#[tauri::command]
async fn create_agent_with_provider(
    agent_id: String,
    goal: String,
    provider_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let profile = state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());

    // Store agent-provider mapping
    {
        let mut map = state
            .agent_providers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(pid) = provider_id.clone() {
            map.insert(agent_id.clone(), pid);
        } else {
            map.remove(&agent_id);
        }
    }

    // Load provider config for custom providers
    let config = if provider == "ollama" || provider == "anthropic" || provider == "openai" {
        None
    } else {
        let configs = read_merged_configs(None, profile.as_deref());
        configs.into_iter().find(|c| c.id == provider)
    };

    // Audit: stream started (B-05/03)
    let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_default();

    // Execute via LLM client (B-05/FIX-02: per-agent provider client switching)
    let result = async {
        let client = create_llm_client(&provider, profile.as_deref(), config)?;
        let precise_prompt_start = client
            .count_tokens(
                vec![ChatMessage {
                    role: "user".into(),
                    content: goal.clone(),
                    timestamp: None,
                }],
                &model,
            )
            .ok()
            .map(|n| n as u64);

        let _ = audit::log_usage(&audit::KeyUsageRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider_name: provider.clone(),
            model: model.clone(),
            status: "started".into(),
            estimated_tokens: None,
            precise_prompt: precise_prompt_start,
            precise_completion: None,
            token_before: None,
            token_after: None,
        });

        let mut stream = client
            .stream_chat(goal)
            .await
            .map_err(|e| format!("stream start failed: {}", e))?;

        let mut output = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                engine_llm_core::StreamChunk::Output(text) => output.push_str(&text),
                engine_llm_core::StreamChunk::Error(e) => return Err(format!("LLM error: {}", e)),
                engine_llm_core::StreamChunk::Done => break,
            }
        }
        let usage = client.last_usage();
        Ok((output, usage))
    }
    .await;

    let (_output_val, usage_val) = match &result {
        Ok((out, usage)) => (Some(out.clone()), *usage),
        Err(_) => (None, None),
    };

    let (precise_prompt_end, precise_completion_end) = if let Some(u) = usage_val {
        (Some(u.prompt_tokens), Some(u.completion_tokens))
    } else {
        (None, None)
    };

    // Audit: completed or failed (B-05/03)
    let _ = audit::log_usage(&audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider,
        model,
        status: if result.is_ok() {
            "completed".into()
        } else {
            "failed".into()
        },
        estimated_tokens: None,
        precise_prompt: precise_prompt_end,
        precise_completion: precise_completion_end,
        token_before: None,
        token_after: None,
    });

    match result {
        Ok((output, _)) => Ok(format!("Agent {} completed. Output:\n{}", agent_id, output)),
        Err(e) => Err(e),
    }
}

// ------------------------------------------------------------------
// Audit log commands (B-05/03)
// ------------------------------------------------------------------
#[tauri::command]
fn get_audit_logs(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<audit::KeyUsageRecord>, String> {
    audit::get_logs(limit.unwrap_or(100), offset.unwrap_or(0))
}

#[tauri::command]
async fn subscribe_agent_trace(
    on_event: Channel<TraceEvent>,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // SAFETY: Mutex held only for Option clone; poison unlikely in single-threaded Tauri command context
    let tx = state
        .trace_tx
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let Some(tx) = tx else {
        // AgentLoop trace channel not yet injected; client will retry or use Tauri Event listener
        return Ok(());
    };
    let mut rx = tx.subscribe();
    let history_clone = state.edit_history.clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Phase 4 Day 5: Record edit events for history timeline
            if matches!(
                event.step_type,
                TraceStepType::EditProposed
                    | TraceStepType::EditApplied
                    | TraceStepType::EditRejected
            ) {
                let checkpoint = checkpoint_record_from_trace(&event);
                if let Err(e) = write_checkpoint_record(&app_clone, &checkpoint) {
                    eprintln!("checkpoint write failed: {}", e);
                }
                let mut hist = history_clone.lock().await;
                let entry = EditHistoryEntry {
                    id: format!("edit_{}_{}", event.iteration, hist.len()),
                    timestamp: event.timestamp.to_rfc3339(),
                    step_type: format!("{:?}", event.step_type),
                    summary: event.details.clone(),
                    confidence: event.confidence_score,
                    token_before: None,
                    token_after: None,
                    checkpoint_id: Some(checkpoint.id),
                };
                hist.push(entry);
                if hist.len() > 200 {
                    hist.remove(0);
                }
            } else if is_checkpoint_store_trace(&event) {
                let checkpoint = checkpoint_record_from_trace(&event);
                if let Err(e) = write_checkpoint_record(&app_clone, &checkpoint) {
                    eprintln!("checkpoint write failed: {}", e);
                }
            }
            let _ = on_event.send(event.clone());
            let _ = app_clone.emit("agent:trace", &event);
        }
    });
    Ok(())
}

#[tauri::command]
fn pause_loop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // SAFETY: Mutex held only for bool write; poison unlikely in single-threaded Tauri command context
    *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = true;
    Ok(())
}

#[tauri::command]
fn resume_loop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // SAFETY: Mutex held only for bool write; poison unlikely in single-threaded Tauri command context
    *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = false;
    Ok(())
}

#[tauri::command]
fn set_approval_level(level: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let valid = ["Auto", "Advisory", "Required", "Critical", "Override"];
    if !valid.contains(&level.as_str()) {
        return Err("Invalid approval level".to_string());
    }
    *state
        .approval_level
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = level;
    Ok(())
}

#[tauri::command]
fn inject_memory(_key: String, _value: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn update_plan(_plan: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn list_checkpoints(app_handle: tauri::AppHandle) -> Result<Vec<CheckpointRecord>, String> {
    read_checkpoint_records(&app_handle)
}

#[tauri::command]
fn get_edit_history(state: tauri::State<'_, AppState>) -> Result<Vec<EditHistoryEntry>, String> {
    Ok(state.edit_history.blocking_lock().clone())
}

#[tauri::command]
fn restore_checkpoint(
    id: String,
    confirm_restore: bool,
    dry_run: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<RestoreResult, String> {
    let dry_run = dry_run.unwrap_or(false);
    validate_restore_confirmation(confirm_restore, dry_run)?;

    let records = read_checkpoint_records(&app_handle)?;
    let record = find_checkpoint_record(&records, &id)?;
    let base_dir = get_workspace_dir(&app_handle)?;
    let backup_dir = restore_backup_dir(&app_handle, &record.id)?;
    let mut plan = build_restore_plan(&record, &base_dir, &backup_dir)?;

    if dry_run {
        return Ok(plan);
    }

    if !plan.warnings.is_empty() {
        return Err(format!(
            "restore refused: {}; run dry-run and create content snapshots before write restore",
            plan.warnings.join("; ")
        ));
    }

    backup_restore_targets(&plan, &base_dir, &backup_dir)?;
    apply_restore_plan(&record, &plan, &base_dir)?;
    plan.dry_run = false;
    plan.restored_at = chrono::Utc::now().to_rfc3339();
    Ok(plan)
}

#[tauri::command]
fn compare_checkpoints(
    id_a: String,
    id_b: String,
    app_handle: tauri::AppHandle,
) -> Result<CheckpointCompareResult, String> {
    let records = read_checkpoint_records(&app_handle)?;
    let before = find_checkpoint_record(&records, &id_a)?;
    let after = find_checkpoint_record(&records, &id_b)?;
    Ok(compare_checkpoint_records(&before, &after))
}

#[tauri::command]
fn export_checkpoint(id: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let records = read_checkpoint_records(&app_handle)?;
    if id == "all" {
        let workspace = get_workspace_dir(&app_handle)?;
        let bundle = CheckpointExportBundle {
            schema_version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            workspace: workspace.to_string_lossy().to_string(),
            checkpoints: records,
        };
        return serde_json::to_string_pretty(&bundle)
            .map_err(|e| format!("checkpoint export serialize failed: {}", e));
    }

    let record = find_checkpoint_record(&records, &id)?;
    serde_json::to_string_pretty(&record)
        .map_err(|e| format!("checkpoint export serialize failed: {}", e))
}

#[tauri::command]
fn get_resource_metrics(state: tauri::State<'_, AppState>) -> Result<Value, String> {
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

// Phase 4 Day 5: Agent Command Palette dispatcher
#[tauri::command]
async fn run_agent_command(
    cmd: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let trimmed = cmd.trim();
    if trimmed.starts_with("@agent refactor ") {
        let target = trimmed
            .strip_prefix("@agent refactor ")
            .unwrap_or("")
            .to_string();
        // Inject as a plan update
        return Ok(format!("Refactor request queued for: {}", target));
    }
    if trimmed.starts_with("@agent review-pr") {
        return Ok("PR review mode activated".to_string());
    }
    if trimmed.starts_with("@agent continue-background") {
        *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = false;
        return Ok("Agent resumed in background".to_string());
    }
    if trimmed.starts_with("@agent pause") {
        *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = true;
        return Ok("Agent paused".to_string());
    }
    if trimmed.starts_with("@agent status") {
        let paused = *state.paused.lock().unwrap_or_else(|e| e.into_inner());
        let level = state
            .approval_level
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        return Ok(format!(
            "Agent status: paused={}, approval_level={}",
            paused, level
        ));
    }
    Err(format!("Unknown agent command: {}", cmd))
}

#[tauri::command]
async fn subscribe_resource_alerts(on_event: Channel<TraceEvent>) -> Result<(), String> {
    on_event
        .send(TraceEvent {
            step: agent_core::LoopState::Idle,
            details: "Resource alerts subscription started".to_string(),
            iteration: 0,
            timestamp: chrono::Utc::now(),
            step_type: TraceStepType::Other,
            plan_summary: None,
            reflection_key_points: vec![],
            confidence_score: None,
            edit_payload: None,
            operation_summary: None,
            thinking_content: None,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ------------------------------------------------------------------
// Phase 4 Day 3: Inline Editing Commands
// ------------------------------------------------------------------
#[derive(Deserialize)]
struct EditHunkPayload {
    path: String,
    old_string: String,
    new_string: String,
}

#[tauri::command]
async fn apply_edits(
    edits: Vec<EditHunkPayload>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ToolResult>, String> {
    let mut results = Vec::new();
    for edit in edits {
        let tool = state
            .registry
            .get("edit_file")
            .ok_or_else(|| "edit_file tool not found".to_string())?;
        let args = serde_json::json!({
            "path": edit.path,
            "old_string": edit.old_string,
            "new_string": edit.new_string,
        });
        let output = tool.execute(args).await.map_err(|e| e.message)?;
        results.push(output.into());
    }
    Ok(results)
}

#[tauri::command]
fn preview_edit(path: String, old_string: String, new_string: String) -> Result<String, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if !content.contains(&old_string) {
        return Err("Old string not found in file".to_string());
    }
    let lines: Vec<&str> = content.lines().collect();
    let old_lines: Vec<&str> = old_string.lines().collect();
    let mut diff = format!("--- {}\n+++ {}\n", path, path);
    // Find approximate line number of old_string
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
async fn get_ast_context(symbol_name: String) -> Result<String, String> {
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
async fn get_cumulative_stats(
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

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
fn main() {
    let state = AppState {
        registry: build_registry(),
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
    };

    // Create production-ready AgentLoop with planner and reflector.
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
            .build()
            .expect("AgentLoop build failed")
    };

    // Inject the broadcast sender so frontend trace panel receives real AgentLoop events.
    if let Some(tx) = agent_loop.trace_tx() {
        state.set_trace_tx(tx);
    }

    let agent_loop_arc = Arc::new(agent_loop);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .manage(agent_loop_arc.clone())
        .invoke_handler(tauri::generate_handler![
            greet,
            read_file,
            write_file,
            list_dir,
            create_dir,
            rename_path,
            delete_path,
            run_command,
            list_tools,
            execute_tool,
            get_providers,
            get_provider_configs,
            add_provider_config,
            update_provider_config,
            delete_provider_config,
            validate_provider,
            get_current_workspace,
            export_provider_backup,
            import_provider_backup,
            stream_chat,
            compact_context,
            optimize_context,
            // B-05/01 Profile
            list_profiles,
            get_active_profile,
            set_active_profile,
            create_profile,
            delete_profile,
            // B-05/02 Agent provider
            get_agent_providers,
            set_agent_provider,
            create_agent_with_provider,
            // B-05/03 Audit
            get_audit_logs,
            // B-02/06 Trace
            subscribe_agent_trace,
            // B-03/06 Governance
            pause_loop,
            resume_loop,
            set_approval_level,
            inject_memory,
            update_plan,
            // B-04/06 Checkpoint
            list_checkpoints,
            restore_checkpoint,
            compare_checkpoints,
            export_checkpoint,
            // B-05/06 Resource
            get_resource_metrics,
            subscribe_resource_alerts,
            // Phase 4 Day 3: Inline Editing
            apply_edits,
            preview_edit,
            get_ast_context,
            // Phase 4 Day 5: Command Palette & Observability
            get_edit_history,
            run_agent_command,
            // P1-03/05: Token cumulative stats
            get_cumulative_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
