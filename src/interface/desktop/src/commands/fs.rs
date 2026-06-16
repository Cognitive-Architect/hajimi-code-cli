use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathIntent {
    /// 目标文件必须存在
    ExistingFile,
    /// 目标目录必须存在
    ExistingDir,
    /// 目标文件无需存在，其父目录必须在 workspace 内且存在
    NewFile,
    /// 目标目录无需存在，其父目录存在
    NewDir,
    /// 目标可存在可不存在（任意类型）
    AnyExisting,
}

/// 安全解析 workspace 内路径，防止 symlink 逃逸和 traversal 攻击
pub fn resolve_workspace_path(
    input: &str,
    base_dir: &Path,
    intent: PathIntent,
) -> Result<PathBuf, String> {
    // 1. 拒绝显式 traversal
    if input.contains("..") {
        return Err("路径包含非法 traversal: ..".to_string());
    }

    // 2. 解析输入路径
    let input_path = Path::new(input);
    let resolved = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        base_dir.join(input_path)
    };

    // 3. canonicalize base_dir（必须存在）
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("无法解析工作目录: {}", e))?;

    // 4. 根据 intent 决定 canonicalize 策略
    let canonical = match intent {
        PathIntent::ExistingFile | PathIntent::ExistingDir | PathIntent::AnyExisting => {
            // existing 路径必须 canonicalize 目标本身
            resolved
                .canonicalize()
                .map_err(|e| format!("无法解析目标路径: {}", e))?
        }
        PathIntent::NewFile | PathIntent::NewDir => {
            // new 路径只 canonicalize 父目录
            let parent = resolved
                .parent()
                .ok_or_else(|| "无法获取父目录".to_string())?;
            if !parent.exists() {
                return Err(format!("父目录不存在: {}", parent.display()));
            }
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("无法解析父目录: {}", e))?;
            // 拼接 leaf name
            canonical_parent.join(
                resolved
                    .file_name()
                    .ok_or_else(|| "无法获取文件名".to_string())?,
            )
        }
    };

    // 5. 确认在 workspace 内
    if !canonical.starts_with(&canonical_base) {
        return Err("路径越界: 目标不在当前工作目录内".to_string());
    }

    match intent {
        PathIntent::ExistingFile if !canonical.is_file() => {
            return Err(format!("目标不是文件: {}", canonical.display()));
        }
        PathIntent::ExistingDir if !canonical.is_dir() => {
            return Err(format!("目标不是目录: {}", canonical.display()));
        }
        _ => {}
    }

    Ok(canonical)
}

#[tauri::command]
pub fn read_file(path: &str, app_handle: tauri::AppHandle) -> Result<String, String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::ExistingFile)?;
    std::fs::read_to_string(&safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file(path: &str, content: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::NewFile)?;
    // 确保父目录存在
    if let Some(parent) = safe_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&safe_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_dir(path: &str, app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::ExistingDir)?;
    let entries = std::fs::read_dir(&safe_path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    Ok(entries)
}

pub fn create_workspace_dir(safe_path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(safe_path).map_err(|e| e.to_string())
}

pub fn rename_workspace_path(safe_old: &Path, safe_new: &Path) -> Result<(), String> {
    std::fs::rename(safe_old, safe_new).map_err(|e| e.to_string())
}

pub fn remove_workspace_path(safe_path: &Path, recursive: bool) -> Result<(), String> {
    if safe_path.is_dir() {
        if recursive {
            std::fs::remove_dir_all(safe_path).map_err(|e| e.to_string())
        } else {
            std::fs::remove_dir(safe_path).map_err(|e| e.to_string())
        }
    } else if safe_path.is_file() {
        std::fs::remove_file(safe_path).map_err(|e| e.to_string())
    } else {
        Err(format!("目标不是文件或目录: {}", safe_path.display()))
    }
}

#[tauri::command]
pub fn create_dir(path: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::NewDir)?;
    create_workspace_dir(&safe_path)
}

#[tauri::command]
pub fn rename_path(old_path: &str, new_path: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    // 源路径必须存在
    let safe_old = resolve_workspace_path(old_path, &base_dir, PathIntent::AnyExisting)?;
    // 目标路径的父目录必须在 workspace 内
    let safe_new = resolve_workspace_path(new_path, &base_dir, PathIntent::NewFile)?;
    rename_workspace_path(&safe_old, &safe_new)
}

#[tauri::command]
pub fn delete_path(path: &str, recursive: bool, app_handle: tauri::AppHandle) -> Result<(), String> {
    let base_dir = crate::get_workspace_dir(&app_handle)?;
    let safe_path = resolve_workspace_path(path, &base_dir, PathIntent::AnyExisting)?;
    remove_workspace_path(&safe_path, recursive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn setup_test_workspace() -> (PathBuf, PathBuf) {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "hajimi-test-fs-{}-{}-{}",
            std::process::id(),
            nanos,
            counter
        ));
        let workspace = temp.join("test-workspace");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&workspace).expect("无法创建 workspace");
        (temp, workspace)
    }

    fn cleanup_test_workspace(temp: &PathBuf) {
        let _ = std::fs::remove_dir_all(temp);
    }

    #[cfg(not(windows))]
    fn create_dir_link(link: &Path, target: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_dir_link(link: &Path, target: &Path) -> std::io::Result<()> {
        match std::os::windows::fs::symlink_dir(target, link) {
            Ok(()) => Ok(()),
            Err(primary_error) => {
                let status = std::process::Command::new("cmd")
                    .args(["/C", "mklink", "/J"])
                    .arg(link)
                    .arg(target)
                    .status();
                match status {
                    Ok(status) if status.success() => Ok(()),
                    _ => Err(primary_error),
                }
            }
        }
    }

    #[test]
    fn test_resolve_existing_file() {
        let (temp, workspace) = setup_test_workspace();
        let test_file = workspace.join("test.txt");
        std::fs::write(&test_file, "hello").expect("无法写入测试文件");

        let result = resolve_workspace_path("test.txt", &workspace, PathIntent::ExistingFile);
        assert!(result.is_ok());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_existing_dir() {
        let (temp, workspace) = setup_test_workspace();
        let subdir = workspace.join("subdir");
        std::fs::create_dir_all(&subdir).expect("无法创建子目录");

        let result = resolve_workspace_path("subdir", &workspace, PathIntent::ExistingDir);
        assert!(result.is_ok());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_file() {
        let (temp, workspace) = setup_test_workspace();

        let result = resolve_workspace_path("newfile.txt", &workspace, PathIntent::NewFile);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_name().unwrap(), "newfile.txt");
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_dir() {
        let (temp, workspace) = setup_test_workspace();

        let result = resolve_workspace_path("newdir", &workspace, PathIntent::NewDir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_name().unwrap(), "newdir");
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_file_missing_parent() {
        let (temp, workspace) = setup_test_workspace();

        let result =
            resolve_workspace_path("nonexistent/newfile.txt", &workspace, PathIntent::NewFile);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("父目录不存在"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_traversal_rejected() {
        let (temp, workspace) = setup_test_workspace();

        let result = resolve_workspace_path("../outside.txt", &workspace, PathIntent::AnyExisting);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_absolute_outside_rejected() {
        let (temp, workspace) = setup_test_workspace();

        #[cfg(windows)]
        let outside = "C:\\Windows\\System32\\notepad.exe";
        #[cfg(not(windows))]
        let outside = "/etc/passwd";

        let result = resolve_workspace_path(outside, &workspace, PathIntent::ExistingFile);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("越界"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_resolve_new_file_rejects_parent_symlink_escape() {
        let (temp, workspace) = setup_test_workspace();
        let outside = temp.join("outside");
        std::fs::create_dir_all(&outside).expect("无法创建 outside 目录");
        let link = workspace.join("outside-link");
        create_dir_link(&link, &outside).expect("无法创建 workspace 外跳链接");

        let result =
            resolve_workspace_path("outside-link/newfile.txt", &workspace, PathIntent::NewFile);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("越界"));
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_create_workspace_dir_creates_directory() {
        let (temp, workspace) = setup_test_workspace();

        let safe_path = resolve_workspace_path("created", &workspace, PathIntent::NewDir).unwrap();
        let result = create_workspace_dir(&safe_path);

        assert!(result.is_ok());
        assert!(workspace.join("created").is_dir());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_rename_workspace_path_renames_file() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("old.txt"), "hello").expect("无法写入测试文件");

        let safe_old =
            resolve_workspace_path("old.txt", &workspace, PathIntent::AnyExisting).unwrap();
        let safe_new = resolve_workspace_path("new.txt", &workspace, PathIntent::NewFile).unwrap();
        let result = rename_workspace_path(&safe_old, &safe_new);

        assert!(result.is_ok());
        assert!(!workspace.join("old.txt").exists());
        assert_eq!(
            std::fs::read_to_string(workspace.join("new.txt")).unwrap(),
            "hello"
        );
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_remove_workspace_path_removes_file_even_when_recursive_true() {
        let (temp, workspace) = setup_test_workspace();
        std::fs::write(workspace.join("delete-me.txt"), "hello").expect("无法写入测试文件");

        let safe_path =
            resolve_workspace_path("delete-me.txt", &workspace, PathIntent::AnyExisting).unwrap();
        let result = remove_workspace_path(&safe_path, true);

        assert!(result.is_ok());
        assert!(!workspace.join("delete-me.txt").exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_remove_workspace_path_removes_directory_recursive() {
        let (temp, workspace) = setup_test_workspace();
        let nested = workspace.join("delete-dir").join("nested");
        std::fs::create_dir_all(&nested).expect("无法创建测试目录");
        std::fs::write(nested.join("file.txt"), "hello").expect("无法写入测试文件");

        let safe_path =
            resolve_workspace_path("delete-dir", &workspace, PathIntent::AnyExisting).unwrap();
        let result = remove_workspace_path(&safe_path, true);

        assert!(result.is_ok());
        assert!(!workspace.join("delete-dir").exists());
        cleanup_test_workspace(&temp);
    }

    #[test]
    fn test_remove_workspace_path_rejects_non_empty_directory_without_recursive() {
        let (temp, workspace) = setup_test_workspace();
        let nested = workspace.join("non-empty");
        std::fs::create_dir_all(&nested).expect("无法创建测试目录");
        std::fs::write(nested.join("file.txt"), "hello").expect("无法写入测试文件");

        let safe_path =
            resolve_workspace_path("non-empty", &workspace, PathIntent::AnyExisting).unwrap();
        let result = remove_workspace_path(&safe_path, false);

        assert!(result.is_err());
        assert!(workspace.join("non-empty").exists());
        cleanup_test_workspace(&temp);
    }
}
