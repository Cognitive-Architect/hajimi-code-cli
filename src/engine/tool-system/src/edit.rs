//! File edit tool - B-W10/02: 字符串级精准编辑
//! 功能: Replace/Insert/Delete 操作 + 原子写入

use super::{
    PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions,
};
use crate::fs::{validate_tool_path, PathValidation};
use fs2::FileExt;
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

const LOCK_RETRIES: u32 = 3;
const LOCK_RETRY_MS: u64 = 10;

pub struct EditFileTool {
    allowed_paths: Option<Vec<PathBuf>>,
}
impl Default for EditFileTool {
    fn default() -> Self {
        Self::new()
    }
}

impl EditFileTool {
    pub fn new() -> Self {
        Self {
            allowed_paths: None,
        }
    }

    pub fn with_allowed_paths(allowed_paths: Vec<PathBuf>) -> Self {
        Self {
            allowed_paths: Some(allowed_paths),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EditOperation {
    Replace {
        old: String,
        new: String,
    },
    Insert {
        line: usize,
        content: String,
        after: bool,
    },
    Delete {
        line: usize,
    },
}

#[async_trait::async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }
    fn description(&self) -> &str {
        "Edit file with atomic write"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: self.allowed_paths.clone(),
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing path"))?;
        let dry_run = args
            .get("dry_run")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let op = parse_operation(&args)?;
        let path_buf = validate_tool_path(
            Path::new(path),
            &self.allowed_paths,
            PathValidation::ExistingFile,
        )?;
        let content = tokio::fs::read_to_string(&path_buf)
            .await
            .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
        let lines: Vec<&str> = content.lines().collect();
        let edited = apply_operation(&lines, &op)?;
        if dry_run {
            return Ok(ToolOutput::success(format!(
                "Preview:\n{}",
                edited.join("\n")
            )));
        }
        atomic_write(&path_buf, &edited.join("\n")).await?;
        Ok(ToolOutput::success("edited"))
    }
}

pub async fn edit_file(
    path: &PathBuf,
    op: &EditOperation,
    dry_run: bool,
) -> Result<String, ToolError> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
    let lines: Vec<&str> = content.lines().collect();
    let edited = apply_operation(&lines, op)?;
    if dry_run {
        return Ok(edited.join("\n"));
    }
    atomic_write(path, &edited.join("\n")).await?;
    Ok(edited.join("\n"))
}

fn parse_operation(args: &ToolArgs) -> Result<EditOperation, ToolError> {
    if let Some(old) = args.get("old_string").and_then(Value::as_str) {
        let new = args.get("new_string").and_then(Value::as_str).unwrap_or("");
        return Ok(EditOperation::Replace {
            old: old.to_string(),
            new: new.to_string(),
        });
    }
    if let Some(line) = args.get("line").and_then(Value::as_u64) {
        let after = args.get("after").and_then(Value::as_bool).unwrap_or(false);
        if let Some(content) = args.get("insert").and_then(Value::as_str) {
            return Ok(EditOperation::Insert {
                line: line as usize,
                content: content.to_string(),
                after,
            });
        }
        if args.get("delete").and_then(Value::as_bool).unwrap_or(false) {
            return Ok(EditOperation::Delete {
                line: line as usize,
            });
        }
    }
    Err(ToolError::new("Invalid operation"))
}

fn apply_operation(lines: &[&str], op: &EditOperation) -> Result<Vec<String>, ToolError> {
    match op {
        EditOperation::Replace { old, new } => {
            let content = lines.join("\n");
            if !content.contains(old) {
                return Err(ToolError {
                    message: "Old string not found".into(),
                    kind: ToolErrorKind::InvalidArgs,
                });
            }
            Ok(content
                .replacen(old, new, 1)
                .lines()
                .map(|s| s.to_string())
                .collect())
        }
        EditOperation::Insert {
            line,
            content,
            after,
        } => {
            let idx = if *line == 0 {
                0
            } else {
                line.saturating_sub(1)
            };
            if idx > lines.len() {
                return Err(ToolError {
                    message: format!("Line {} out of range", line),
                    kind: ToolErrorKind::InvalidLineNumber,
                });
            }
            let mut res: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let pos = if *after { idx + 1 } else { idx };
            res.insert(pos, content.clone());
            Ok(res)
        }
        EditOperation::Delete { line } => {
            let idx = line.saturating_sub(1);
            if idx >= lines.len() {
                return Err(ToolError {
                    message: format!("Line {} out of range", line),
                    kind: ToolErrorKind::InvalidLineNumber,
                });
            }
            let mut res: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            res.remove(idx);
            Ok(res)
        }
    }
}

pub async fn atomic_write(path: &PathBuf, content: &str) -> Result<(), ToolError> {
    if path.exists() {
        let meta = tokio::fs::metadata(path)
            .await
            .map_err(|e| ToolError::new(format!("Meta: {}", e)))?;
        if meta.permissions().readonly() {
            return Err(ToolError {
                message: "Read-only file".into(),
                kind: ToolErrorKind::PermissionDenied,
            });
        }
    }
    let temp = path.with_extension("tmp");
    let backup = path.with_extension("bak");
    if path.exists() {
        tokio::fs::copy(path, &backup)
            .await
            .map_err(|e| ToolError::new(format!("Backup: {}", e)))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    file.write_all(content.as_bytes())
        .map_err(|e| ToolError::new(format!("Write: {}", e)))?;
    let mut locked = false;
    for i in 0..LOCK_RETRIES {
        match file.try_lock_exclusive() {
            Ok(_) => {
                locked = true;
                break;
            }
            Err(_) if i < LOCK_RETRIES - 1 => {
                std::thread::sleep(Duration::from_millis(LOCK_RETRY_MS))
            }
            Err(e) => return Err(ToolError::new(format!("Lock: {}", e))),
        }
    }
    if !locked {
        return Err(ToolError::new("Lock failed"));
    }
    drop(file);
    if let Err(e) = tokio::fs::rename(&temp, path).await {
        let _ = tokio::fs::copy(&backup, path).await;
        return Err(ToolError::new(format!("Rename: {}", e)));
    }
    let _ = tokio::fs::remove_file(&backup).await;
    Ok(())
}

fn _is_utf8_boundary(content: &str, idx: usize) -> bool {
    content.is_char_boundary(idx)
}
