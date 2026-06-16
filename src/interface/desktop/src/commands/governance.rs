use crate::state::AppState;

#[tauri::command]
pub fn pause_loop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = true;
    Ok(())
}

#[tauri::command]
pub fn resume_loop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.paused.lock().unwrap_or_else(|e| e.into_inner()) = false;
    Ok(())
}

#[tauri::command]
pub fn set_approval_level(level: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
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
pub fn inject_memory(_key: String, _value: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn update_plan(_plan: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn resolve_agent_approval(
    request_id: String,
    approved: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut map = state.pending_approvals.lock().await;
    if let Some(tx) = map.remove(&request_id) {
        let _ = tx.send(approved);
        Ok(())
    } else {
        Err("No pending approval found for this request_id".to_string())
    }
}
