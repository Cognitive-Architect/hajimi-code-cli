use crate::state::{
    CheckpointCompareResult, CheckpointExportBundle, CheckpointFileChange, CheckpointFileRef,
    CheckpointRecord, RestoreFilePlan, RestoreResult,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn read_checkpoint_records(app_handle: &tauri::AppHandle) -> Result<Vec<CheckpointRecord>, String> {
    let dir = crate::checkpoint_store_dir(app_handle)?;
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
    use crate::commands::fs::{resolve_workspace_path, PathIntent};
    match resolve_workspace_path(file_path, base_dir, PathIntent::AnyExisting) {
        Ok(path) => Ok(path),
        Err(_) => resolve_workspace_path(file_path, base_dir, PathIntent::NewFile),
    }
}

fn restore_backup_dir(
    app_handle: &tauri::AppHandle,
    checkpoint_id: &str,
) -> Result<PathBuf, String> {
    let dir = crate::checkpoint_store_dir(app_handle)?
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

#[tauri::command]
pub fn list_checkpoints(app_handle: tauri::AppHandle) -> Result<Vec<CheckpointRecord>, String> {
    read_checkpoint_records(&app_handle)
}

#[tauri::command]
pub fn restore_checkpoint(
    id: String,
    confirm_restore: bool,
    dry_run: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<RestoreResult, String> {
    let dry_run = dry_run.unwrap_or(false);
    validate_restore_confirmation(confirm_restore, dry_run)?;

    let records = read_checkpoint_records(&app_handle)?;
    let record = find_checkpoint_record(&records, &id)?;
    let base_dir = crate::get_workspace_dir(&app_handle)?;
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
pub fn compare_checkpoints(
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
pub fn export_checkpoint(id: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let records = read_checkpoint_records(&app_handle)?;
    if id == "all" {
        let workspace = crate::get_workspace_dir(&app_handle)?;
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
