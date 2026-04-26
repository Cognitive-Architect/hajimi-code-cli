use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyUsageRecord {
    pub timestamp: String,
    pub provider_name: String,
    pub model: String,
    pub status: String,
    pub estimated_tokens: Option<u64>,
}

pub fn audit_log_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
            .join("Hajimi")
            .join("audit.jsonl")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("Library/Application Support/Hajimi/audit.jsonl")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config/hajimi/audit.jsonl")
    }
}

pub fn log_usage(record: &KeyUsageRecord) -> Result<(), String> {
    let path = audit_log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    rotate_if_needed(&path)?;
    let line = serde_json::to_string(record).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    drop(file);
    set_audit_permissions(&path)?;
    Ok(())
}

pub fn get_logs(limit: usize, offset: usize) -> Result<Vec<KeyUsageRecord>, String> {
    let path = audit_log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut records: Vec<KeyUsageRecord> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    records.reverse();
    let start = offset.min(records.len());
    let end = (offset + limit).min(records.len());
    Ok(records[start..end].to_vec())
}

fn rotate_if_needed(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > 10 * 1024 * 1024 {
        let backup = path.with_extension("jsonl.1");
        std::fs::rename(path, &backup).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn set_audit_permissions(path: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, Permissions::from_mode(0o600))
            .map_err(|e| format!("audit perm failed: {}", e))?;
    }
    #[cfg(windows)]
    {
        if let Ok(username) = std::env::var("USERNAME") {
            let _ = std::process::Command::new("icacls")
                .arg(path)
                .arg("/inheritance:r")
                .arg("/grant:r")
                .arg(format!("{}:F", username))
                .output();
        }
    }
    Ok(())
}
