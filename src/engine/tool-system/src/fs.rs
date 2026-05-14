//! File system tools - CORR-W09-02: DEBT-001/002 清偿
//! DEBT-001: 二进制文件检测 ✓  DEBT-002: 并发写入锁 ✓

use super::{
    PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions,
};
use fs2::FileExt;
use serde_json::Value;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;

const CHUNK_SIZE: usize = 65536;
const MAX_SIZE: u64 = 100 * 1024 * 1024;
const BACKUP_SUFFIX: &str = ".bak";
const MAX_MAGIC_BYTES: usize = 512;
const LOCK_RETRIES: u32 = 3;
const LOCK_RETRY_MS: u64 = 10;

pub struct ReadFileTool;
impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self
    }
}
pub struct WriteFileTool;
impl Default for WriteFileTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteFileTool {
    pub fn new() -> Self {
        Self
    }
}
pub struct LsTool;
impl Default for LsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl LsTool {
    pub fn new() -> Self {
        Self
    }
}

/// DEBT-001: 检测二进制文件（非UTF-8）
fn is_binary_file(path: &PathBuf) -> Result<bool, ToolError> {
    let mut file = std::fs::File::open(path).map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    let mut buf = vec![0u8; MAX_MAGIC_BYTES];
    let n = file
        .read(&mut buf)
        .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
    buf.truncate(n);
    Ok(std::str::from_utf8(&buf).is_err())
}

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read file with binary detection"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing path"))?;
        let path_buf: PathBuf = path.into();
        validate_path(&path_buf, &self.permissions().allowed_paths)?;
        let meta = tokio::fs::metadata(&path_buf)
            .await
            .map_err(|e| ToolError::new(format!("Access: {}", e)))?;
        if !meta.is_file() {
            return Err(ToolError::new("Not a file"));
        }
        if meta.len() > MAX_SIZE {
            return Err(ToolError::new("Too large"));
        }
        if is_binary_file(&path_buf)? {
            return Err(ToolError::new("Binary file detected"));
        }
        let content = if meta.len() > 1_048_576 {
            read_chunked(&path_buf).await?
        } else {
            tokio::fs::read_to_string(&path_buf)
                .await
                .map_err(|e| ToolError::new(format!("Read: {}", e)))?
        };
        Ok(ToolOutput::success(content))
    }
}

async fn read_chunked(path: &PathBuf) -> Result<String, ToolError> {
    use tokio::io::AsyncReadExt;
    let file = File::open(path)
        .await
        .map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    let mut reader = tokio::io::BufReader::with_capacity(CHUNK_SIZE, file);
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .await
        .map_err(|e| ToolError::new(format!("Read: {}", e)))?;
    Ok(content)
}

#[async_trait::async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "Atomic write with file lock"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing path"))?;
        let content = args
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing content"))?;
        let path_buf: PathBuf = path.into();
        validate_path(&path_buf, &self.permissions().allowed_paths)?;
        if path_buf.exists() {
            let ext = path_buf
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            let backup = path_buf.with_extension(format!("{}.{}", ext, BACKUP_SUFFIX));
            tokio::fs::copy(&path_buf, backup)
                .await
                .map_err(|e| ToolError::new(format!("Backup: {}", e)))?;
        }
        let temp = path_buf.with_extension("tmp");
        tokio::fs::write(&temp, content)
            .await
            .map_err(|e| ToolError::new(format!("Write: {}", e)))?;
        // DEBT-002: 获取独占锁（带重试）
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&temp)
            .map_err(|e| ToolError::new(format!("Lock open: {}", e)))?;
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
                Err(e) => {
                    return Err(ToolError::new(format!(
                        "Lock failed after {} retries: {}",
                        LOCK_RETRIES, e
                    )))
                }
            }
        }
        if !locked {
            return Err(ToolError::new("Unable to acquire file lock"));
        }
        tokio::fs::rename(&temp, &path_buf)
            .await
            .map_err(|e| ToolError::new(format!("Rename: {}", e)))?;
        drop(file);
        Ok(ToolOutput::success("written"))
    }
}

#[async_trait::async_trait]
impl Tool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }
    fn description(&self) -> &str {
        "List directory"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Allow,
            requires_confirmation: false,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path)
            .await
            .map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| ToolError::new(format!("Entry: {}", e)))?
        {
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        Ok(ToolOutput::success(entries.join("\n")))
    }
}

fn validate_path(path: &std::path::Path, allowed: &Option<Vec<PathBuf>>) -> Result<(), ToolError> {
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(ToolError::new("Traversal"));
    }
    if let Some(ref bases) = allowed {
        if !bases.iter().any(|base| path.starts_with(base)) {
            return Err(ToolError::new("Not allowed"));
        }
    }
    Ok(())
}

pub struct DeleteFileTool;
impl Default for DeleteFileTool {
    fn default() -> Self {
        Self::new()
    }
}

impl DeleteFileTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Default)]
struct DeleteResult {
    deleted: usize,
    failed: usize,
    errors: Vec<String>,
}

impl DeleteResult {
    fn to_json(&self) -> String {
        format!(
            r#"{{"deleted":{},"failed":{},"errors":{:?}}}"#,
            self.deleted, self.failed, self.errors
        )
    }
}

fn check_path_traversal(path: &std::path::Path) -> Result<(), ToolError> {
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(ToolError {
            message: "Path traversal detected".into(),
            kind: ToolErrorKind::InvalidArgs,
        });
    }
    Ok(())
}

fn is_root_path(path: &std::path::Path) -> bool {
    let s = path.to_string_lossy();
    path.components().count() == 0
        || s == "/"
        || s == "\\"
        || (s.len() == 3 && s.ends_with(":\\"))
        || (s.len() == 2 && s.ends_with(":"))
}

async fn delete_recursive(path: &PathBuf, force: bool, dry_run: bool, result: &mut DeleteResult) {
    if dry_run {
        result.deleted += 1;
        return;
    }

    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(m) => m,
        Err(e) => {
            result.failed += 1;
            result.errors.push(format!("{}: {}", path.display(), e));
            return;
        }
    };

    if metadata.is_symlink() {
        match tokio::fs::remove_file(path).await {
            Ok(_) => result.deleted += 1,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path.display(), e));
            }
        }
        return;
    }

    if metadata.is_dir() {
        let mut entries = match tokio::fs::read_dir(path).await {
            Ok(r) => r,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path.display(), e));
                return;
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let child = entry.path();
            Box::pin(delete_recursive(&child, force, dry_run, result)).await;
        }

        match tokio::fs::remove_dir(path).await {
            Ok(_) => result.deleted += 1,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path.display(), e));
            }
        }
    } else {
        if force {
            let mut perms = metadata.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = tokio::fs::set_permissions(path, perms).await;
        }
        match tokio::fs::remove_file(path).await {
            Ok(_) => result.deleted += 1,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }
}

#[async_trait::async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }
    fn description(&self) -> &str {
        "Delete file/dir with safety checks"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions {
            default_level: PermissionLevel::Ask,
            requires_confirmation: true,
            allowed_paths: None,
        }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("Missing path"))?;
        let recursive = args
            .get("recursive")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let force = args.get("force").and_then(Value::as_bool).unwrap_or(false);
        let dry_run = args
            .get("dry_run")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let path_buf: PathBuf = path.into();
        check_path_traversal(&path_buf)?;

        if is_root_path(&path_buf) {
            return Err(ToolError {
                message: "Cannot delete root directory".into(),
                kind: ToolErrorKind::PermissionDenied,
            });
        }

        if !path_buf.exists() {
            return Err(ToolError {
                message: format!("Not found: {}", path),
                kind: ToolErrorKind::NotFound,
            });
        }

        let metadata = tokio::fs::symlink_metadata(&path_buf)
            .await
            .map_err(|e| ToolError {
                message: format!("Access: {}", e),
                kind: ToolErrorKind::PermissionDenied,
            })?;

        let mut result = DeleteResult::default();

        if metadata.is_dir() && !recursive && !metadata.is_symlink() {
            return Err(ToolError {
                message: "Is a directory (use recursive=true)".into(),
                kind: ToolErrorKind::InvalidArgs,
            });
        }

        delete_recursive(&path_buf, force, dry_run, &mut result).await;

        Ok(ToolOutput::success(result.to_json()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn test_chunk_size() {
        assert_eq!(CHUNK_SIZE, 65536);
    }
    #[test]
    fn test_backup_suffix() {
        assert_eq!(BACKUP_SUFFIX, ".bak");
    }
    #[test]
    fn test_max_magic_bytes() {
        assert_eq!(MAX_MAGIC_BYTES, 512);
    }
    #[test]
    fn test_is_binary_file_text() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::NamedTempFile::new()?;
        temp.as_file().write_all(b"Hello, World!")?;
        assert!(!is_binary_file(&temp.path().to_path_buf())?);
        Ok(())
    }
    #[test]
    fn test_is_binary_file_binary() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::NamedTempFile::new()?;
        temp.as_file().write_all(&[0x00, 0x01, 0xFF, 0xFE])?;
        assert!(is_binary_file(&temp.path().to_path_buf())?);
        Ok(())
    }

    #[test]
    fn test_delete_file_tool_name() {
        let tool = DeleteFileTool::new();
        assert_eq!(tool.name(), "delete_file");
    }

    #[test]
    fn test_check_path_traversal() {
        assert!(check_path_traversal(PathBuf::from("../etc/passwd").as_path()).is_err());
        assert!(check_path_traversal(PathBuf::from("/safe/path").as_path()).is_ok());
        assert!(check_path_traversal(PathBuf::from("safe/path").as_path()).is_ok());
    }

    #[test]
    fn test_is_root_path() {
        assert!(is_root_path(PathBuf::from("/").as_path()));
        assert!(is_root_path(PathBuf::from("\\").as_path()));
        assert!(is_root_path(PathBuf::from("C:\\").as_path()));
        assert!(!is_root_path(PathBuf::from("/home/user").as_path()));
        assert!(!is_root_path(PathBuf::from("file.txt").as_path()));
    }

    #[test]
    fn test_delete_result_json() {
        let result = DeleteResult {
            deleted: 5,
            failed: 2,
            errors: vec!["err1".to_string(), "err2".to_string()],
        };
        let json = result.to_json();
        assert!(json.contains("\"deleted\":5"));
        assert!(json.contains("\"failed\":2"));
        assert!(json.contains("\"errors\":["));
    }

    #[tokio::test]
    async fn test_delete_file_dry_run() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("test.txt");
        let mut f = std::fs::File::create(&file_path)?;
        f.write_all(b"test")?;
        drop(f);

        let tool = DeleteFileTool::new();
        let args = serde_json::json!({
            "path": file_path.to_str().ok_or("Invalid path")?,
            "dry_run": true
        });

        let result = tool.execute(args).await?;
        assert!(file_path.exists());
        assert!(result.stdout.contains("\"deleted\":1"));
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_file_not_found() {
        let tool = DeleteFileTool::new();
        let args = serde_json::json!({"path": "/nonexistent/file/12345"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
        match result {
            Err(ToolError {
                kind: ToolErrorKind::NotFound,
                ..
            }) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_delete_file_path_traversal() {
        let tool = DeleteFileTool::new();
        let args = serde_json::json!({"path": "../../../etc/passwd"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
        match result {
            Err(ToolError {
                kind: ToolErrorKind::InvalidArgs,
                ..
            }) => (),
            _ => panic!("Expected InvalidArgs error"),
        }
    }
}
