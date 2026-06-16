use serde_json::Value;

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
