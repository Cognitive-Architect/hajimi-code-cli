use crate::state::AppState;
use std::path::PathBuf;

#[tauri::command]
pub fn list_profiles() -> Result<Vec<String>, String> {
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
pub fn get_active_profile(state: tauri::State<'_, AppState>) -> Option<String> {
    state
        .active_profile
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
pub fn set_active_profile(
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
pub fn create_profile(name: String) -> Result<(), String> {
    let name = crate::sanitize_profile_name(&name)?;
    let path = crate::profile_config_path(&name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !path.exists() {
        std::fs::write(&path, "[]").map_err(|e| e.to_string())?;
    }
    crate::write_configs_to_path(&path, &[])?;
    Ok(())
}

#[tauri::command]
pub fn delete_profile(name: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let name = crate::sanitize_profile_name(&name)?;
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
    let path = crate::profile_config_path(&name);
    if path.exists() {
        // Delete config file
        let _ = std::fs::remove_file(&path);
        // Delete profile directory
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
    // Clean up keyring entries for this profile (best effort), format: provider:{profile}:{id}
    let configs = crate::read_provider_configs_with_profile(Some(&name));
    for cfg in configs {
        let _ = crate::delete_api_key_with_profile(&cfg.id, Some(&name));
    }
    Ok(())
}
