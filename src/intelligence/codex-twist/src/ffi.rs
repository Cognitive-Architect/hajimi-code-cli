//! FFI绑定层 - napi-rs实现
#![cfg(feature = "napi")]
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use std::collections::HashMap;
use crate::thread::{Thread, ThreadConfig};
use crate::approval::ApprovalPolicy;
use crate::memory::{MemoryGateway, TokenBudget, MemoryLevel};
use once_cell::sync::Lazy;

static THREAD_STORE: Lazy<Mutex<HashMap<String, Thread>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

static MEMORY_GATEWAY_STORE: Lazy<TokioMutex<HashMap<String, MemoryGateway>>> = 
    Lazy::new(|| TokioMutex::new(HashMap::new()));

#[napi(object)]
#[derive(Clone, Debug)]
pub struct ThreadHandle { pub id: String, pub name: String, pub created_at: String, pub turn_count: u32 }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct TurnHandle { pub turn_id: String, pub thread_id: String, pub prompt: String, pub response: Option<String>, pub status: String, pub timestamp: String }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct ThreadConfigJs { pub model: Option<String>, pub base_url: Option<String>, pub api_key: Option<String>, pub system_prompt: Option<String>, pub max_context_length: Option<u32>, pub approval_policy: Option<String> }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct ThreadInfo { pub id: String, pub name: String, pub created_at: String, pub updated_at: String, pub turn_count: u32 }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct TurnInfo { pub turn_id: String, pub thread_id: String, pub prompt: String, pub response: String, pub status: String, pub timestamp: String }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct ApprovalRequest { pub id: String, pub command: String, pub description: String, pub risk_level: String }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct StorageStats { pub total_threads: u32, pub total_turns: u32, pub storage_size_bytes: i64 }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct LcrAdapterConfig { pub version: String, pub format: String, pub compression: bool }

#[napi(object)]
#[derive(Clone, Debug)]
pub struct MemoryGatewayHandle { 
    pub id: String,
    pub focus_tokens: u32,
    pub working_tokens: u32,
    pub archive_tokens: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct MemoryStatsJs {
    pub focus_entries: u32,
    pub focus_tokens: u32,
    pub working_entries: u32,
    pub working_tokens: u32,
    pub archive_entries: u32,
    pub archive_tokens: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct TokenBudgetJs {
    pub focus_limit: u32,
    pub working_limit: u32,
    pub archive_limit: u32,
}

impl Default for ThreadConfigJs {
    fn default() -> Self {
        Self { model: Some("gpt-4".to_string()), base_url: Some("http://localhost:11434/v1".to_string()), api_key: None, system_prompt: None, max_context_length: Some(8192), approval_policy: Some("ask-for-dangerous".to_string()) }
    }
}

impl From<ThreadConfigJs> for ThreadConfig {
    fn from(cfg: ThreadConfigJs) -> Self {
        Self { model: cfg.model.unwrap_or_else(|| "gpt-4".to_string()), base_url: cfg.base_url.unwrap_or_else(|| "http://localhost:11434/v1".to_string()), api_key: cfg.api_key, system_prompt: cfg.system_prompt, max_context_length: cfg.max_context_length.unwrap_or(8192) as usize, approval_policy: parse_policy_js(cfg.approval_policy.as_deref().unwrap_or("ask-for-dangerous")) }
    }
}

fn parse_policy_js(s: &str) -> ApprovalPolicy {
    match s.to_lowercase().as_str() {
        "ask" | "ask-before-exec" => ApprovalPolicy::AskBeforeExec,
        "dangerous" | "ask-for-dangerous" => ApprovalPolicy::AskForDangerous,
        "once" | "ask-once-then-auto" => ApprovalPolicy::AskOnceThenAuto,
        "auto" | "full-auto" => ApprovalPolicy::FullAuto,
        "deny" | "full-deny" => ApprovalPolicy::FullDeny,
        _ => ApprovalPolicy::AskForDangerous,
    }
}

fn now_str() -> String { format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()) }

fn generate_unique_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("thread_{}_{}", now_str(), COUNTER.fetch_add(1, Ordering::SeqCst))
}

#[napi]
pub fn create_thread(name: String, storage_path: String, config: Option<ThreadConfigJs>) -> Result<ThreadHandle> {
    let path = PathBuf::from(storage_path);
    let id = generate_unique_id();
    let mut thread = Thread::new(id.clone(), path);
    thread.name = name.clone();
    if let Some(cfg) = config { thread.config = cfg.into(); }
    let handle = ThreadHandle { id: thread.id.clone(), name: thread.name.clone(), created_at: format!("{}", thread.created_at), turn_count: thread.turns.len() as u32 };
    if let Ok(mut store) = THREAD_STORE.lock() { store.insert(thread.id.clone(), thread); }
    Ok(handle)
}

#[napi]
pub fn create_turn(thread_id: String, prompt: String) -> Result<TurnHandle> {
    if let Ok(mut store) = THREAD_STORE.lock() {
        if let Some(thread) = store.get_mut(&thread_id) {
            // 创建Turn并添加到thread.turns集合
            let turn = crate::turn::Turn::new(thread_id.clone(), prompt.clone());
            let turn_id = turn.id.clone(); // 使用Turn生成的id
            let ts = format!("{}", turn.timestamp);
            thread.turns.push(turn);
            thread.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            return Ok(TurnHandle { turn_id, thread_id, prompt, response: None, status: "pending".to_string(), timestamp: ts });
        }
    }
    Err(Error::new(Status::GenericFailure, "Thread not found".to_string()))
}

#[napi]
pub fn complete_turn(thread_id: String, turn_id: String, response: String) -> Result<TurnHandle> {
    let ts = now_str();
    if let Ok(mut store) = THREAD_STORE.lock() {
        if let Some(thread) = store.get_mut(&thread_id) {
            // 通过turn_id查找并更新turn
            if let Some(turn) = thread.turns.iter_mut().find(|t| t.id == turn_id) {
                turn.complete(response.clone());
                thread.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                return Ok(TurnHandle { turn_id, thread_id, prompt: turn.prompt.clone(), response: Some(response), status: "completed".to_string(), timestamp: ts });
            }
            return Err(Error::new(Status::GenericFailure, format!("Turn not found: {}", turn_id)));
        }
    }
    Err(Error::new(Status::GenericFailure, "Thread not found".to_string()))
}

#[napi]
pub fn cancel_turn(turn_id: String) -> Result<TurnHandle> {
    Ok(TurnHandle { turn_id, thread_id: String::new(), prompt: String::new(), response: None, status: "cancelled".to_string(), timestamp: now_str() })
}

#[napi]
pub fn get_thread_info(thread_id: String) -> Result<ThreadInfo> {
    if let Ok(store) = THREAD_STORE.lock() {
        if let Some(thread) = store.get(&thread_id) {
            return Ok(ThreadInfo { id: thread.id.clone(), name: thread.name.clone(), created_at: format!("{}", thread.created_at), updated_at: format!("{}", thread.updated_at), turn_count: thread.turns.len() as u32 });
        }
    }
    Err(Error::new(Status::GenericFailure, format!("Thread not found: {}", thread_id)))
}

#[napi]
pub fn list_threads() -> Result<Vec<ThreadInfo>> {
    if let Ok(store) = THREAD_STORE.lock() {
        let threads: Vec<ThreadInfo> = store.values().map(|t| ThreadInfo {
            id: t.id.clone(), name: t.name.clone(), created_at: format!("{}", t.created_at), updated_at: format!("{}", t.updated_at), turn_count: t.turns.len() as u32
        }).collect();
        return Ok(threads);
    }
    Ok(Vec::new())
}

#[napi]
pub fn get_turn(_turn_id: String) -> Result<Option<TurnInfo>> {
    Ok(None)
}

#[napi]
pub fn list_turns(thread_id: String) -> Result<Vec<TurnInfo>> {
    if let Ok(store) = THREAD_STORE.lock() {
        if let Some(thread) = store.get(&thread_id) {
            let turns: Vec<TurnInfo> = thread.turns.iter().map(|t| TurnInfo {
                turn_id: t.id.clone(), thread_id: thread_id.clone(), prompt: t.prompt.clone(), response: t.response_content(), status: format!("{:?}", t.status).to_lowercase(), timestamp: format!("{}", t.timestamp)
            }).collect();
            return Ok(turns);
        }
    }
    Ok(Vec::new())
}

#[napi]
pub fn append_turn(turn_id: String, content: String) -> Result<TurnHandle> {
    Ok(TurnHandle { turn_id, thread_id: String::new(), prompt: content, response: None, status: "pending".to_string(), timestamp: now_str() })
}

#[napi]
pub fn request_approval(command: String) -> Result<ApprovalRequest> {
    Ok(ApprovalRequest { id: format!("req_{}", now_str()), command: command.clone(), description: format!("需要审批: {}", command), risk_level: "medium".to_string() })
}

#[napi]
pub fn get_storage_stats() -> Result<StorageStats> {
    if let Ok(store) = THREAD_STORE.lock() {
        let total_turns: usize = store.values().map(|t| t.turns.len()).sum();
        return Ok(StorageStats { total_threads: store.len() as u32, total_turns: total_turns as u32, storage_size_bytes: 0 });
    }
    Ok(StorageStats { total_threads: 0, total_turns: 0, storage_size_bytes: 0 })
}

#[napi]
pub fn get_lcr_config() -> Result<LcrAdapterConfig> {
    Ok(LcrAdapterConfig { version: "1.0.0".to_string(), format: "lcr".to_string(), compression: true })
}

#[napi]
pub fn version() -> String { crate::VERSION.to_string() }

#[napi]
pub fn create_memory_gateway(budget: Option<TokenBudgetJs>) -> MemoryGatewayHandle {
    let token_budget = match budget {
        Some(b) => TokenBudget {
            focus_limit: b.focus_limit as usize,
            working_limit: b.working_limit as usize,
            archive_limit: b.archive_limit as usize,
        },
        None => TokenBudget::default(),
    };
    
    let gateway = MemoryGateway::with_budget(token_budget.clone());
    let id = format!("mg_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
    
    let handle = MemoryGatewayHandle {
        id: id.clone(),
        focus_tokens: token_budget.focus_limit as u32,
        working_tokens: token_budget.working_limit as u32,
        archive_tokens: token_budget.archive_limit as u32,
    };
    
    let mut store = MEMORY_GATEWAY_STORE.blocking_lock();
    store.insert(id, gateway);
    
    handle
}

#[napi]
pub async fn memory_put(
    handle_id: String,
    key: String,
    value: String,
    level: String,
) -> Result<()> {
    let level_enum = match level.to_lowercase().as_str() {
        "focus" => MemoryLevel::Focus,
        "working" => MemoryLevel::Working,
        "archive" => MemoryLevel::Archive,
        "rag" => MemoryLevel::Rag,
        _ => MemoryLevel::Focus,
    };
    
    let gateway = {
        let store = MEMORY_GATEWAY_STORE.lock().await;
        store.get(&handle_id).cloned()
    };
    
    if let Some(gateway) = gateway {
        gateway.put(key, value, level_enum).await;
        Ok(())
    } else {
        Err(Error::new(Status::GenericFailure, "Memory gateway not found".to_string()))
    }
}

#[napi]
pub async fn memory_get(
    handle_id: String,
    key: String,
) -> Result<Option<String>> {
    let gateway = {
        let store = MEMORY_GATEWAY_STORE.lock().await;
        store.get(&handle_id).cloned()
    };
    
    if let Some(gateway) = gateway {
        Ok(gateway.get(&key).await)
    } else {
        Err(Error::new(Status::GenericFailure, "Memory gateway not found".to_string()))
    }
}

#[napi]
pub async fn memory_stats(
    handle_id: String,
) -> Result<MemoryStatsJs> {
    let gateway = {
        let store = MEMORY_GATEWAY_STORE.lock().await;
        store.get(&handle_id).cloned()
    };
    
    if let Some(gateway) = gateway {
        let stats = gateway.stats().await;
        Ok(MemoryStatsJs {
            focus_entries: stats.focus_entries as u32,
            focus_tokens: stats.focus_tokens as u32,
            working_entries: stats.working_entries as u32,
            working_tokens: stats.working_tokens as u32,
            archive_entries: stats.archive_entries as u32,
            archive_tokens: stats.archive_tokens as u32,
        })
    } else {
        Err(Error::new(Status::GenericFailure, "Memory gateway not found".to_string()))
    }
}

#[napi]
pub fn get_default_memory_budget() -> TokenBudgetJs {
    let budget = TokenBudget::default();
    TokenBudgetJs {
        focus_limit: budget.focus_limit as u32,
        working_limit: budget.working_limit as u32,
        archive_limit: budget.archive_limit as u32,
    }
}

#[napi]
pub fn parse_approval_policy(policy_str: String) -> String {
    format!("{:?}", parse_policy_js(&policy_str)).to_lowercase()
}

#[napi]
pub async fn memory_clear(handle_id: String, level: String) -> Result<()> {
    let level_enum = match level.as_str() {
        "focus" => MemoryLevel::Focus,
        "working" => MemoryLevel::Working,
        "archive" => MemoryLevel::Archive,
        _ => MemoryLevel::Focus,
    };
    
    let gateway = {
        let store = MEMORY_GATEWAY_STORE.lock().await;
        store.get(&handle_id).cloned()
    };
    
    if let Some(gateway) = gateway {
        // 根据level清除对应层
        match level_enum {
            MemoryLevel::Focus => { gateway.clear_focus().await; },
            MemoryLevel::Working => { gateway.clear_working().await; },
            MemoryLevel::Archive => { gateway.clear_archive().await; },
            _ => {}
        }
        Ok(())
    } else {
        Err(Error::new(Status::GenericFailure, "Gateway not found".to_string()))
    }
}

#[napi]
pub async fn memory_optimize(handle_id: String, target_level: String) -> Result<String> {
    let gateway = {
        let store = MEMORY_GATEWAY_STORE.lock().await;
        store.get(&handle_id).cloned()
    };
    
    if let Some(gateway) = gateway {
        // 内存优化逻辑
        gateway.optimize(&target_level).await;
        Ok(format!("Optimized for {}", target_level))
    } else {
        Err(Error::new(Status::GenericFailure, "Gateway not found".to_string()))
    }
}

#[napi]
pub fn needs_approval(command: String) -> Result<bool> {
    Ok(command.contains("rm") || command.contains("delete"))
}

#[napi]
pub fn save_thread(thread_id: String) -> Result<()> {
    if let Ok(store) = THREAD_STORE.lock() {
        if let Some(thread) = store.get(&thread_id) {
            return thread.save_to_lcr().map_err(|e| {
                let msg = format!("Failed to save thread {}: {}", thread_id, e);
                Error::new(Status::GenericFailure, msg)
            });
        }
    }
    Err(Error::new(Status::GenericFailure, format!("Thread not found: {}", thread_id)))
}

#[napi]
pub fn load_thread(storage_path: String) -> Result<ThreadHandle> {
    let path = PathBuf::from(&storage_path);
    let hctx_path = path.join(".hctx");
    let hctx_path_display = hctx_path.display().to_string();
    
    // 检查文件是否存在
    if !hctx_path.exists() {
        return Err(Error::new(Status::GenericFailure, format!("Thread file not found: {}", hctx_path_display)));
    }
    
    let storage = crate::storage::HctxStorage::new(hctx_path)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    
    match storage.load_context() {
        Ok(chunks) => {
            match crate::lcr_adapter::hctx_to_thread(&chunks, path.clone()) {
                Ok(thread) => {
                    let handle = ThreadHandle { id: thread.id.clone(), name: thread.name.clone(), created_at: format!("{}", thread.created_at), turn_count: thread.turns.len() as u32 };
                    if let Ok(mut store) = THREAD_STORE.lock() { store.insert(thread.id.clone(), thread); }
                    Ok(handle)
                }
                Err(e) => Err(Error::new(Status::GenericFailure, e.to_string()))
            }
        }
        Err(crate::storage::StorageError::NotFound(_)) => Err(Error::new(Status::GenericFailure, format!("Thread file not found: {}", hctx_path_display))),
        Err(e) => Err(Error::new(Status::GenericFailure, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_config_js_conversion() {
        let js_config = ThreadConfigJs { model: Some("gpt-3.5".to_string()), base_url: Some("http://test".to_string()), api_key: None, system_prompt: Some("Test prompt".to_string()), max_context_length: Some(4096), approval_policy: Some("auto".to_string()) };
        let config: ThreadConfig = js_config.into();
        assert_eq!(config.model, "gpt-3.5");
        assert_eq!(config.max_context_length, 4096);
    }

    #[test]
    fn test_parse_policy_js() {
        assert!(matches!(parse_policy_js("ask"), ApprovalPolicy::AskBeforeExec));
        assert!(matches!(parse_policy_js("auto"), ApprovalPolicy::FullAuto));
        assert!(matches!(parse_policy_js("deny"), ApprovalPolicy::FullDeny));
    }

    #[test]
    fn test_thread_handle_creation() {
        let handle = ThreadHandle { id: "test_123".to_string(), name: "Test Thread".to_string(), created_at: "1234567890".to_string(), turn_count: 5 };
        assert_eq!(handle.id, "test_123");
        assert_eq!(handle.turn_count, 5);
    }

    #[test]
    fn test_turn_handle_creation() {
        let handle = TurnHandle { turn_id: "turn_123".to_string(), thread_id: "thread_456".to_string(), prompt: "Hello".to_string(), response: Some("World".to_string()), status: "completed".to_string(), timestamp: "1234567890".to_string() };
        assert_eq!(handle.turn_id, "turn_123");
        assert_eq!(handle.response, Some("World".to_string()));
    }
}
