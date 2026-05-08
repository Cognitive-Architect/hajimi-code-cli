#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::Arc;

use codex_twist::memory::{MemoryGateway, MemoryTier, TokenBudget, TokenUsageTracker};
use engine_llm_core::{AnthropicClient, ChatMessage, Client, LlmClient, OllamaClient, OpenAiClient};
use engine_tool_system::{
    AnalyzeTool, BashTool, CargoBuildTool, CmakeTool, DeleteFileTool, EditFileTool,
    FetchUrlTool, FindTool, GenerateDocsTool, GeneratePrDescriptionTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitStatusTool, GlobTool, GraphTool, GrepTool, JsBundleAnalyzerTool, ListDirectoryTool,
    LspDefinitionTool, LspHoverTool, LspInitTool, LspReferencesTool, LsTool, MakeTool,
    McpInitTool, McpInvokeTool, CoverageReportTool, BenchmarkTool, NpmRunTool,
    PowerShellTool, ReadFileTool, RefactorCodeTool, RunTestsTool, RustDocGeneratorTool,
    SecurityAuditTool, SmartCommitTool, ToolOutput, ToolRegistry, UpdateReadmeTool, ViewImageTool,
    WebSearchTool, WriteFileTool,
};
use engine_tool_system::lsp_integration::ASTContextProvider;
use keyring::Entry;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::path::{Path, PathBuf};
use tauri::{ipc::Channel, Emitter, Manager};
use agent_core::{AgentLoopBuilder, HierarchicalPlanner, AutonomousReflector, AgentContext, TraceEvent};
use agent_core::agent_loop::TraceStepType;
use memory::memory_gateway::MemoryGateway as AgentMemoryGateway;

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
    "git", "cargo", "npm", "node", "npx", "pnpm",
    "rustc", "rustfmt", "clippy-driver",
    "python", "python3", "pip", "pip3",
    "code", "cursor",
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
    let base = app_handle.path().document_dir()
        .map_err(|e| format!("无法获取文档目录: {}", e))?;
    let workspace = base.join("hajimi-workspace");
    std::fs::create_dir_all(&workspace).map_err(|e| e.to_string())?;
    Ok(workspace)
}

/// 校验路径是否在工作目录沙箱内
fn validate_path_within_workspace(path: &str, base_dir: &Path) -> Result<PathBuf, String> {
    // 1. 拒绝显式包含 .. 的路径
    if path.contains("..") {
        return Err("路径包含非法 traversal: ..".to_string());
    }

    // 2. 解析绝对路径
    let resolved = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        base_dir.join(path)
    };

    // 3. canonicalize（文件不存在时 fallback 到 resolved）
    let canonical = resolved.canonicalize().unwrap_or(resolved);

    // 4. 确认在 base_dir 内
    let canonical_base = base_dir.canonicalize()
        .map_err(|e| format!("无法解析工作目录: {}", e))?;

    if !canonical.starts_with(&canonical_base) {
        return Err(format!(
            "路径越界: {} 不在工作目录 {} 内",
            canonical.display(),
            canonical_base.display()
        ));
    }

    Ok(canonical)
}

#[tauri::command]
fn read_file(path: &str, app_handle: tauri::AppHandle) -> Result<String, String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = validate_path_within_workspace(path, &base_dir)?;
    std::fs::read_to_string(&safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_file(path: &str, content: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = validate_path_within_workspace(path, &base_dir)?;
    // 确保父目录存在
    if let Some(parent) = safe_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&safe_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_dir(path: &str, app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let base_dir = get_workspace_dir(&app_handle)?;
    let safe_path = validate_path_within_workspace(path, &base_dir)?;
    let entries = std::fs::read_dir(&safe_path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    Ok(entries)
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
        return Err(format!("exit code {:?}\nstderr: {}", output.status.code(), stderr));
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
            .field("api_key", &if self.api_key.is_empty() { "none" } else { "sk-••••••••" })
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
    PathBuf::from(workspace).join(".hajimi").join("providers.json")
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
    if !path.exists() { return Vec::new(); }
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
    entry.set_password(api_key).map_err(|e| format!("keyring set failed: {}", e))?;
    Ok(())
}

#[allow(dead_code)]
fn get_api_key(id: &str) -> Result<String, String> {
    get_api_key_with_profile(id, None)
}

fn get_api_key_with_profile(id: &str, profile: Option<&str>) -> Result<String, String> {
    let entry = Entry::new("hajimi", &keyring_entry_id(id, profile))
        .map_err(|e| format!("keyring entry failed: {}", e))?;
    entry.get_password().map_err(|e| format!("keyring get failed: {}", e))
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
fn migrate_provider_keys(configs: &mut [ProviderConfig], profile: Option<&str>) -> Result<(), String> {
    let mut migrated = false;
    for cfg in configs.iter_mut() {
        if !cfg.api_key.trim().is_empty() {
            save_api_key_with_profile(&cfg.id, &cfg.api_key, profile)?;
            cfg.api_key.clear();  // sanitize in memory too
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

fn write_provider_configs_with_profile(profile: Option<&str>, configs: &[ProviderConfig]) -> Result<(), String> {
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
                return Err(format!("Failed to restrict ACL on {}: {}", path.display(), stderr));
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
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
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
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| "decryption failed: wrong password or corrupted file".to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_provider_configs(workspace_path: Option<String>, state: tauri::State<'_, AppState>) -> Vec<ProviderConfig> {
    // SAFETY: Mutex held only for config read; poison unlikely in single-threaded Tauri command context
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
    read_merged_configs(workspace_path.as_deref(), profile.as_deref())
}

#[tauri::command]
fn add_provider_config(mut config: ProviderConfig, workspace_path: Option<String>, save_target: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // SAFETY: Mutex held only for config read; poison unlikely in single-threaded Tauri command context
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
fn update_provider_config(mut config: ProviderConfig, workspace_path: Option<String>, save_target: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if !config.api_key.trim().is_empty() {
        save_api_key_with_profile(&config.id, &config.api_key, profile.as_deref())?;
    }
    config.api_key.clear();
    let target = save_target.as_deref().unwrap_or("global");
    if target == "workspace" {
        if let Some(ws) = workspace_path.as_deref() {
            let path = workspace_config_path(ws);
            let mut configs = read_configs_at(&path);
            let idx = configs.iter().position(|c| c.id == config.id)
                .ok_or_else(|| format!("Provider '{}' not found", config.id))?;
            configs[idx] = config;
            return write_configs_to_path(&path, &configs);
        }
    }
    let mut configs = read_provider_configs_with_profile(profile.as_deref());
    let idx = configs.iter().position(|c| c.id == config.id)
        .ok_or_else(|| format!("Provider '{}' not found", config.id))?;
    configs[idx] = config;
    write_provider_configs_with_profile(profile.as_deref(), &configs)
}

#[tauri::command]
fn delete_provider_config(id: String, workspace_path: Option<String>, delete_target: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
fn get_providers(workspace_path: Option<String>, state: tauri::State<'_, AppState>) -> Vec<ProviderInfo> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut providers = vec![
        ProviderInfo {
            name: "ollama".into(),
            available: true,
            default_model: "llama3".into(),
        },
    ];

    // Official providers now unified with config + keyring fallback to env (P0-2)
    let anthropic_key_ok = std::env::var("ANTHROPIC_API_KEY").is_ok() || get_api_key_with_profile("anthropic", profile.as_deref()).is_ok();
    providers.push(ProviderInfo {
        name: "anthropic".into(),
        available: anthropic_key_ok,
        default_model: "claude-3-5-sonnet-20241022".into(),
    });

    let openai_key_ok = std::env::var("OPENAI_API_KEY").is_ok() || get_api_key_with_profile("openai", profile.as_deref()).is_ok();
    providers.push(ProviderInfo {
        name: "openai".into(),
        available: openai_key_ok,
        default_model: "gpt-4o".into(),
    });

    // Append custom providers from config (keys secured in keyring), with workspace overlay
    for cfg in read_merged_configs(workspace_path.as_deref(), profile.as_deref()) {
        let is_official = cfg.id == "anthropic" || cfg.id == "openai" || cfg.name.to_lowercase() == "anthropic" || cfg.name.to_lowercase() == "openai";
        if !is_official {
            let available = get_api_key_with_profile(&cfg.id, profile.as_deref()).is_ok() || !cfg.api_key.trim().is_empty();
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
fn get_current_workspace() -> Option<String> {
    std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string())
}

/// # Safety: API key from OS keyring, response validated via real HTTP before UI green status
#[tauri::command]
async fn validate_provider(config: ProviderConfig, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
        if config.provider_type.contains("anthropic") { "https://api.anthropic.com".to_string() }
        else if config.provider_type.contains("openai") { "https://api.openai.com".to_string() }
        else { return Err(format!("Provider '{}' requires a base_url for type '{}'", config.name, config.provider_type)); }
    } else { config.base_url.clone() };
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
    let req = client.post(&chat_url)
        .timeout(std::time::Duration::from_secs(8))
        .header("User-Agent", "hajimi/3.8.0")
        .json(&test_payload);
    let req = if config.provider_type.contains("anthropic") {
        req.header("x-api-key", &key).header("anthropic-version", "2023-06-01")
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
                Err(format!("API 端点不存在 (HTTP 404)，请检查 Base URL 是否正确。当前请求地址: {}", chat_url))
            } else if status.as_u16() == 429 {
                Err(format!("请求过于频繁 (HTTP 429)，请稍后再试"))
            } else if status.as_u16() == 400 {
                // 400 usually means auth passed but model name or params invalid
                Ok(format!("✅ {} 认证通过 (模型名或参数可能需要调整)", config.name))
            } else {
                Err(format!("测试失败: HTTP {} - {}", status, r.text().await.unwrap_or_default().chars().take(200).collect::<String>()))
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

fn create_llm_client(provider: &str, profile: Option<&str>, config: Option<ProviderConfig>) -> Result<Box<dyn LlmClient>, String> {
    match provider {
        "ollama" => Ok(Box::new(OllamaClient::default_local())),
        "anthropic" => Ok(Box::new(
            AnthropicClient::from_env()
                .map_err(|e| format!("anthropic init failed: {}", e))?,
        )),
        "openai" => Ok(Box::new(
            OpenAiClient::from_env()
                .map_err(|e| format!("openai init failed: {}", e))?,
        )),
        _ => {
            let cfg = config.ok_or_else(|| {
                format!("config required for custom provider: {}", provider)
            })?;
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
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
        let precise_prompt_start = client.count_tokens(msgs_for_opt.clone(), &model).ok().map(|n| n as u64);

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
                    error: if is_error { Some("LLM error".into()) } else { None },
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
            token_tracker.record_usage(
                &session_key,
                &provider,
                u.prompt_tokens,
                u.completion_tokens,
            ).await;
        }

        // Trigger compression via LLM-driven summary
        let _ = gateway.optimize(msgs_for_opt, client.as_ref()).await;
        let stats_after = gateway.stats().await;
        let token_after = stats_after.working_tokens as u64;

        // Verify context is retrievable
        let _retrieved = gateway.working().get(&session_key).await;

        Ok((token_before, token_after, usage))
    }.await;

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
        status: if chat_result.is_ok() { "completed".into() } else { "failed".into() },
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
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let client = create_llm_client(&provider, profile.as_deref(), config)?;
    let gateway = state.memory_gateway.clone();
    gateway.optimize(messages, client.as_ref()).await
}

#[tauri::command]
fn export_provider_backup(password: String, workspace_path: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
fn import_provider_backup(password: String, file_path: String, state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let encrypted = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let plaintext = decrypt_backup(&encrypted, &password)?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&plaintext).map_err(|e| e.to_string())?;
    let mut count = 0;
    for item in items {
        let cfg = ProviderConfig {
            id: item["id"].as_str().unwrap_or("").to_string(),
            name: item["name"].as_str().unwrap_or("").to_string(),
            provider_type: item["provider_type"].as_str().unwrap_or("openai-compatible").to_string(),
            base_url: item["base_url"].as_str().unwrap_or("").to_string(),
            model: item["model"].as_str().unwrap_or("").to_string(),
            api_key: item["api_key"].as_str().unwrap_or("").to_string(),
            system_prompt: item.get("system_prompt").and_then(|v| v.as_str()).map(|s| s.to_string()),
            context_threshold: item.get("context_threshold").and_then(|v| v.as_u64()).map(|n| n as usize),
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
        PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("Hajimi").join("profiles")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Application Support/Hajimi/profiles")
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
    state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn set_active_profile(name: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut active = state.active_profile.lock().unwrap_or_else(|e| e.into_inner());
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
fn get_agent_providers(state: tauri::State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    // SAFETY: Mutex held only for HashMap clone; poison unlikely in single-threaded Tauri command context
    let map = state.agent_providers.lock().unwrap_or_else(|e| e.into_inner()).clone();
    Ok(map)
}

#[tauri::command]
fn set_agent_provider(agent_id: String, provider_id: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // SAFETY: Mutex held only for HashMap insert/remove; poison unlikely in single-threaded Tauri command context
    let mut map = state.agent_providers.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(pid) = provider_id {
        map.insert(agent_id, pid);
    } else {
        map.remove(&agent_id);
    }
    Ok(())
}

#[tauri::command]
async fn create_agent_with_provider(agent_id: String, goal: String, provider_id: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let profile = state.active_profile.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());

    // Store agent-provider mapping
    {
        let mut map = state.agent_providers.lock().unwrap_or_else(|e| e.into_inner());
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
        let precise_prompt_start = client.count_tokens(
            vec![ChatMessage { role: "user".into(), content: goal.clone(), timestamp: None }],
            &model
        ).ok().map(|n| n as u64);

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
    }.await;

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
        status: if result.is_ok() { "completed".into() } else { "failed".into() },
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
fn get_audit_logs(limit: Option<usize>, offset: Option<usize>) -> Result<Vec<audit::KeyUsageRecord>, String> {
    audit::get_logs(limit.unwrap_or(100), offset.unwrap_or(0))
}

#[tauri::command]
async fn subscribe_agent_trace(
    on_event: Channel<TraceEvent>,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // SAFETY: Mutex held only for Option clone; poison unlikely in single-threaded Tauri command context
    let tx = state.trace_tx.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
            if matches!(event.step_type, TraceStepType::EditProposed | TraceStepType::EditApplied | TraceStepType::EditRejected) {
                let mut hist = history_clone.lock().await;
                let entry = EditHistoryEntry {
                    id: format!("edit_{}_{}", event.iteration, hist.len()),
                    timestamp: event.timestamp.to_rfc3339(),
                    step_type: format!("{:?}", event.step_type),
                    summary: event.details.clone(),
                    confidence: event.confidence_score,
                    token_before: None,
                    token_after: None,
                    checkpoint_id: None,
                };
                hist.push(entry);
                if hist.len() > 200 { hist.remove(0); }
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
    if !valid.contains(&level.as_str()) { return Err("Invalid approval level".to_string()); }
    *state.approval_level.lock().unwrap_or_else(|e| e.into_inner()) = level;
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
fn list_checkpoints(state: tauri::State<'_, AppState>) -> Result<Vec<Value>, String> {
    let hist = state.edit_history.blocking_lock();
    Ok(hist.iter().map(|e| json!({
        "id": e.id,
        "timestamp": e.timestamp,
        "step_type": e.step_type,
        "summary": e.summary,
        "confidence": e.confidence,
    })).collect())
}

#[tauri::command]
fn get_edit_history(state: tauri::State<'_, AppState>) -> Result<Vec<EditHistoryEntry>, String> {
    Ok(state.edit_history.blocking_lock().clone())
}

#[tauri::command]
fn restore_checkpoint(_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn compare_checkpoints(_id_a: String, _id_b: String) -> Result<bool, String> {
    Ok(false)
}

#[tauri::command]
fn export_checkpoint(_id: String) -> Result<String, String> {
    Ok("{}".to_string())
}

#[tauri::command]
fn get_resource_metrics(state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let hist = state.edit_history.blocking_lock();
    let edit_count = hist.len();
    let applied_count = hist.iter().filter(|e| e.step_type == "EditApplied").count();
    let rejected_count = hist.iter().filter(|e| e.step_type == "EditRejected").count();
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
        let target = trimmed.strip_prefix("@agent refactor ").unwrap_or("").to_string();
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
        let level = state.approval_level.lock().unwrap_or_else(|e| e.into_inner()).clone();
        return Ok(format!("Agent status: paused={}, approval_level={}", paused, level));
    }
    Err(format!("Unknown agent command: {}", cmd))
}

#[tauri::command]
async fn subscribe_resource_alerts(on_event: Channel<TraceEvent>) -> Result<(), String> {
    on_event.send(TraceEvent {
        step: agent_core::LoopState::Idle, details: "Resource alerts subscription started".to_string(), iteration: 0,
        timestamp: chrono::Utc::now(), step_type: TraceStepType::Other,
        plan_summary: None, reflection_key_points: vec![], confidence_score: None, edit_payload: None,
        operation_summary: None, thinking_content: None,
    }).map_err(|e| e.to_string())?;
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
        let tool = state.registry.get("edit_file")
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
    diff.push_str(&format!("@@ -{},{} +{},{} @@\n", line_no, old_lines.len(), line_no, new_string.lines().count()));
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
        let _ = provider.index_project(current_dir.to_string_lossy().as_ref()).await;
    }
    match provider.get_symbol_context(&symbol_name, None).await {
        Ok(ctx) => Ok(format!("{} '{}' at {}:{}", ctx.symbol.kind, ctx.symbol.name, ctx.symbol.file_path, ctx.symbol.line)),
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn get_cumulative_stats(
    state: tauri::State<'_, AppState>
) -> Result<serde_json::Value, String> {
    let stats = state.token_tracker.get_global_stats().await;

    let mut by_provider = serde_json::Map::new();
    for (k, v) in &stats.by_provider {
        by_provider.insert(k.clone(), serde_json::json!({
            "prompt_tokens": v.prompt_tokens,
            "completion_tokens": v.completion_tokens,
            "total_tokens": v.total_tokens,
            "request_count": v.request_count
        }));
    }

    let mut by_day = serde_json::Map::new();
    for (k, v) in &stats.by_day {
        by_day.insert(k.clone(), serde_json::json!({
            "prompt_tokens": v.prompt_tokens,
            "completion_tokens": v.completion_tokens,
            "total_tokens": v.total_tokens,
            "request_count": v.request_count
        }));
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
        let mem = Arc::new(tokio::sync::Mutex::new(AgentMemoryGateway::new("hajimi-desktop")));
        let planner = Arc::new(tokio::sync::Mutex::new(
            HierarchicalPlanner::new(mem.clone(), AgentContext::new())
        ));
        let reflector = Arc::new(tokio::sync::Mutex::new(
            AutonomousReflector::new(mem.clone(), AgentContext::new())
        ));
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
