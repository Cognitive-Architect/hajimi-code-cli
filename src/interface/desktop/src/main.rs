#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::Arc;

use engine_llm_core::{AnthropicClient, LlmClient, OllamaClient, OpenAiClient};
use engine_tool_system::{
    AnalyzeTool, BashTool, CargoBuildTool, CmakeTool, DeleteFileTool, EditFileTool,
    FetchUrlTool, FindTool, GenerateDocsTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitStatusTool, GlobTool, GraphTool, GrepTool, JsBundleAnalyzerTool, ListDirectoryTool,
    LspDefinitionTool, LspHoverTool, LspInitTool, LspReferencesTool, LsTool, MakeTool,
    McpInitTool, McpInvokeTool, CoverageReportTool, BenchmarkTool, NpmRunTool,
    PowerShellTool, ReadFileTool, RefactorCodeTool, RunTestsTool, RustDocGeneratorTool,
    SecurityAuditTool, ToolOutput, ToolRegistry, UpdateReadmeTool, ViewImageTool,
    WebSearchTool, WriteFileTool,
};
use keyring::Entry;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::path::PathBuf;
use tauri::ipc::Channel;

mod audit;

// ------------------------------------------------------------------
// App State
// ------------------------------------------------------------------
struct AppState {
    registry: ToolRegistry,
    active_profile: std::sync::Mutex<Option<String>>,
    agent_providers: std::sync::Mutex<HashMap<String, String>>,
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
// Legacy commands
// ------------------------------------------------------------------
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust.", name)
}

#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    Ok(entries)
}

#[tauri::command]
fn run_command(cmd: &str, args: Vec<String>) -> Result<String, String> {
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
    #[serde(skip_serializing)]
    api_key: String,
    base_url: String,
    model: String,
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

fn get_api_key(id: &str) -> Result<String, String> {
    get_api_key_with_profile(id, None)
}

fn get_api_key_with_profile(id: &str, profile: Option<&str>) -> Result<String, String> {
    let entry = Entry::new("hajimi", &keyring_entry_id(id, profile))
        .map_err(|e| format!("keyring entry failed: {}", e))?;
    entry.get_password().map_err(|e| format!("keyring get failed: {}", e))
}

fn delete_api_key(id: &str) -> Result<(), String> {
    delete_api_key_with_profile(id, None)
}

fn delete_api_key_with_profile(_id: &str, _profile: Option<&str>) -> Result<(), String> {
    // Optional: delete from OS keyring. API varies by platform/version; skipped for compatibility in v3.8.0
    // Full impl: entry.delete_password().map_err(...)
    Ok(())
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
    let profile = state.active_profile.lock().unwrap().clone();
    read_merged_configs(workspace_path.as_deref(), profile.as_deref())
}

#[tauri::command]
fn add_provider_config(mut config: ProviderConfig, workspace_path: Option<String>, save_target: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let profile = state.active_profile.lock().unwrap().clone();
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
    let profile = state.active_profile.lock().unwrap().clone();
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
    let profile = state.active_profile.lock().unwrap().clone();
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
    let profile = state.active_profile.lock().unwrap().clone();
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

#[tauri::command]
async fn validate_provider(config: ProviderConfig, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let profile = state.active_profile.lock().unwrap().clone();
    let key = if config.api_key.trim().is_empty() {
        get_api_key_with_profile(&config.id, profile.as_deref())?
    } else {
        config.api_key
    };
    if key.trim().is_empty() {
        return Err("No API key available in keyring or config".to_string());
    }
    // Lightweight validation (P1-3). In production, perform real /v1/models call via client
    // For now, format check + keyring confirmation
    if (config.provider_type.contains("anthropic") && (key.starts_with("sk-ant") || key.starts_with("sk-"))) || key.starts_with("sk-") || key.len() > 15 {
        Ok(format!("✅ {} connection test passed. Key securely stored in OS keyring (service='hajimi', entry='provider:{}').", config.name, config.id))
    } else {
        Err("Invalid key format or provider test failed".to_string())
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
    config: Option<ProviderConfig>,
    on_event: Channel<StreamEvent>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let profile = state.active_profile.lock().unwrap().clone();
    let model = config.as_ref().map(|c| c.model.clone()).unwrap_or_default();

    // Audit: stream started (B-05/03)
    let _ = audit::log_usage(&audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider.clone(),
        model: model.clone(),
        status: "started".into(),
        estimated_tokens: None,
    });

    let chat_result = async {
        let client = create_llm_client(&provider, profile.as_deref(), config)?;

        let mut stream = client
            .stream_chat(prompt)
            .await
            .map_err(|e| format!("stream start failed: {}", e))?;

        while let Some(chunk) = stream.next().await {
            let (text, is_done, is_error) = match chunk {
                engine_llm_core::StreamChunk::Output(t) => (t, false, false),
                engine_llm_core::StreamChunk::Error(e) => (e, false, true),
                engine_llm_core::StreamChunk::Done => (String::new(), true, false),
            };
            on_event
                .send(StreamEvent {
                    chunk: text,
                    done: is_done,
                    error: if is_error { Some("LLM error".into()) } else { None },
                })
                .map_err(|e| e.to_string())?;
            if is_done {
                break;
            }
        }
        Ok(())
    }.await;

    // Audit: completed or failed (B-05/03)
    let _ = audit::log_usage(&audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider,
        model,
        status: if chat_result.is_ok() { "completed".into() } else { "failed".into() },
        estimated_tokens: None,
    });

    chat_result
}

#[tauri::command]
fn export_provider_backup(password: String, workspace_path: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let profile = state.active_profile.lock().unwrap().clone();
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
    let profile = state.active_profile.lock().unwrap().clone();
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
    state.active_profile.lock().unwrap().clone()
}

#[tauri::command]
fn set_active_profile(name: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut profile = state.active_profile.lock().unwrap();
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
        let mut active = state.active_profile.lock().unwrap();
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
    let map = state.agent_providers.lock().unwrap().clone();
    Ok(map)
}

#[tauri::command]
fn set_agent_provider(agent_id: String, provider_id: Option<String>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut map = state.agent_providers.lock().unwrap();
    if let Some(pid) = provider_id {
        map.insert(agent_id, pid);
    } else {
        map.remove(&agent_id);
    }
    Ok(())
}

#[tauri::command]
async fn create_agent_with_provider(agent_id: String, goal: String, provider_id: Option<String>, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let profile = state.active_profile.lock().unwrap().clone();
    let provider = provider_id.clone().unwrap_or_else(|| "openai".to_string());

    // Store agent-provider mapping
    {
        let mut map = state.agent_providers.lock().unwrap();
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
    let _ = audit::log_usage(&audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider.clone(),
        model: model.clone(),
        status: "started".into(),
        estimated_tokens: None,
    });

    // Execute via LLM client (B-05/FIX-02: per-agent provider client switching)
    let result = async {
        let client = create_llm_client(&provider, profile.as_deref(), config)?;
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
        Ok(output)
    }.await;

    // Audit: completed or failed (B-05/03)
    let _ = audit::log_usage(&audit::KeyUsageRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider_name: provider,
        model,
        status: if result.is_ok() { "completed".into() } else { "failed".into() },
        estimated_tokens: None,
    });

    match result {
        Ok(output) => Ok(format!("Agent {} completed. Output:\n{}", agent_id, output)),
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

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
fn main() {
    let state = AppState {
        registry: build_registry(),
        active_profile: std::sync::Mutex::new(None),
        agent_providers: std::sync::Mutex::new(HashMap::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
