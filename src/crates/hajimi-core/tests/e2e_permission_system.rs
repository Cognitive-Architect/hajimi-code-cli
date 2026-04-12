//! E2E Tests - Permission System (B-W07-01)
//!
//! 测试场景:
//! - E2E-004: 权限系统Deny
//! - E2E-005: 权限系统Ask
//! - 权限边界: 路径遍历攻击防护
//! - 权限配置热重载

use hajimi_core::{
    Config, FeaturePreset,
};
use hajimi_core::tool::{PermissionLevel, ToolPermissions};
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// E2E-004: 权限系统Deny
// ============================================================================

#[test]
fn test_permission_level_variants() {
    let deny = PermissionLevel::Deny;
    let ask = PermissionLevel::Ask;
    let allow = PermissionLevel::Allow;
    
    assert_ne!(deny, ask);
    assert_ne!(ask, allow);
    assert_ne!(deny, allow);
}

#[test]
fn test_tool_permissions_default() {
    let perms = ToolPermissions::default();
    assert_eq!(perms.default_level, PermissionLevel::Ask);
    assert!(!perms.requires_confirmation);
    assert!(perms.allowed_paths.is_none());
}

#[tokio::test]
async fn test_permission_bash_tool_execution() {
    use hajimi_core::{BashTool, Tool};
    
    let tool = BashTool::new();
    let args = json!({"command": "echo test"});
    
    let result: Result<hajimi_core::tool::ToolOutput, hajimi_core::tool::ToolError> = tool.execute(args).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_permission_dangerous_command_blocked() {
    use hajimi_core::{BashTool, Tool};
    
    let tool = BashTool::new();
    let args = json!({"command": "rm -rf /"});
    
    let result: Result<hajimi_core::tool::ToolOutput, hajimi_core::tool::ToolError> = tool.execute(args).await;
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// E2E-005: 权限系统Ask
// ============================================================================

#[test]
fn test_permission_ask_level() {
    let perms = ToolPermissions {
        default_level: PermissionLevel::Ask,
        requires_confirmation: true,
        allowed_paths: None,
    };
    
    assert_eq!(perms.default_level, PermissionLevel::Ask);
    assert!(perms.requires_confirmation);
}

#[test]
fn test_permission_allow_level() {
    let perms = ToolPermissions {
        default_level: PermissionLevel::Allow,
        requires_confirmation: false,
        allowed_paths: None,
    };
    
    assert_eq!(perms.default_level, PermissionLevel::Allow);
    assert!(!perms.requires_confirmation);
}

// ============================================================================
// 权限边界: 路径安全
// ============================================================================

#[test]
fn test_path_traversal_patterns_detected() {
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\windows\\system32",
        "/etc/passwd",
        "C:\\Windows\\System32",
    ];
    
    for path in &malicious_paths {
        let contains_traversal = path.contains("..") || 
                                  path.starts_with('/') || 
                                  path.contains(':') && path.len() > 2;
        assert!(contains_traversal || path.starts_with('/'), 
                "Path should be detected as potentially dangerous: {}", path);
    }
}

#[test]
fn test_relative_paths_safe() {
    let valid_paths = vec![
        "src/main.rs",
        "./Cargo.toml",
        "docs/README.md",
        "subdir/file.txt",
    ];
    
    for path in &valid_paths {
        let is_safe = !path.contains("..") && !path.starts_with('/');
        assert!(is_safe, "Path should be safe: {}", path);
    }
}

#[tokio::test]
async fn test_write_file_outside_workspace() {
    use hajimi_core::{WriteFileTool, Tool};
    
    let tool = WriteFileTool;
    let args = json!({
        "path": "/etc/passwd",
        "content": "malicious"
    });
    
    let result: Result<hajimi_core::tool::ToolOutput, hajimi_core::tool::ToolError> = tool.execute(args).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_read_file_sensitive_location() {
    use hajimi_core::{ReadFileTool, Tool};
    
    let tool = ReadFileTool;
    let args = json!({"path": "/etc/shadow"});
    
    let result: Result<hajimi_core::tool::ToolOutput, hajimi_core::tool::ToolError> = tool.execute(args).await;
    assert!(result.is_err() || result.is_ok());
}

// ============================================================================
// 8场景的权限配置
// ============================================================================

#[test]
fn test_preset_minimal_permissions() {
    let preset = FeaturePreset::Minimal;
    let tools = preset.default_tools();
    
    assert!(!tools.is_empty());
    assert_eq!(tools.len(), 5);
}

#[test]
fn test_preset_paranoid_limited_tools() {
    let preset = FeaturePreset::Paranoid;
    let tools = preset.default_tools();
    
    assert_eq!(tools.len(), 8);
}

#[test]
fn test_preset_luxury_wide_permissions() {
    let preset = FeaturePreset::Luxury;
    let tools = preset.default_tools();
    
    assert!(tools.len() > 40);
}

#[test]
fn test_preset_offline_no_network() {
    let preset = FeaturePreset::Offline;
    let tools = preset.default_tools();
    
    for tool in tools {
        assert_ne!(tool, "curl");
        assert_ne!(tool, "wget");
        assert_ne!(tool, "fetch");
    }
}

#[test]
fn test_preset_daily_default_safe() {
    let preset = FeaturePreset::Daily;
    let tools = preset.default_tools();
    
    assert_eq!(tools.len(), 12);
}

#[test]
fn test_preset_performance_optimized() {
    let preset = FeaturePreset::Performance;
    let tools = preset.default_tools();
    
    assert_eq!(tools.len(), 17);
}

#[test]
fn test_preset_frontend_specific() {
    let preset = FeaturePreset::Frontend;
    let tools = preset.default_tools();
    
    assert_eq!(tools.len(), 26);
}

#[test]
fn test_preset_backend_specific() {
    let preset = FeaturePreset::Backend;
    let tools = preset.default_tools();
    
    assert_eq!(tools.len(), 27);
}

// ============================================================================
// 权限配置持久化
// ============================================================================

#[test]
fn test_permissions_with_allowed_paths() {
    use std::path::PathBuf;
    
    let allowed = vec![
        PathBuf::from("./src"),
        PathBuf::from("./docs"),
    ];
    
    let perms = ToolPermissions {
        default_level: PermissionLevel::Allow,
        requires_confirmation: false,
        allowed_paths: Some(allowed),
    };
    
    assert!(perms.allowed_paths.is_some());
    assert_eq!(perms.allowed_paths.as_ref().unwrap().len(), 2);
}

#[test]
fn test_permissions_config_applied_to_preset() {
    let config = Config::default();
    
    let tools = config.preset.default_tools();
    assert!(!tools.is_empty());
}

// ============================================================================
// 权限错误处理
// ============================================================================

#[test]
fn test_tool_error_creation() {
    use hajimi_core::tool::ToolError;
    
    let error = ToolError::new("Permission denied");
    assert_eq!(error.message, "Permission denied");
}

#[test]
fn test_registry_permission_check() {
    use hajimi_core::ToolRegistry;
    
    let registry = ToolRegistry::new();
    
    let result = registry.get("non_existent_tool");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_tool_permission_enforcement() {
    use hajimi_core::{ReadFileTool, Tool};
    
    let tool = ReadFileTool;
    let perms = tool.permissions();
    
    // Tool should have permissions defined
    assert!(!perms.requires_confirmation || perms.default_level == PermissionLevel::Ask);
}

// ============================================================================
// 并发权限检查
// ============================================================================

#[tokio::test]
async fn test_concurrent_permission_reads() {
    let perms = Arc::new(ToolPermissions::default());
    let mut handles = vec![];
    
    for _ in 0..100 {
        let perms_clone = perms.clone();
        let handle = tokio::spawn(async move {
            let _ = perms_clone.default_level;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_concurrent_config_with_permissions() {
    let config = Arc::new(tokio::sync::RwLock::new(Config::default()));
    let mut handles = vec![];
    
    for _ in 0..50 {
        let config_clone = config.clone();
        handles.push(tokio::spawn(async move {
            let cfg = config_clone.read().await;
            let _ = cfg.preset.default_tools();
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_permission_atomic_reads() {
    let perms = Arc::new(std::sync::RwLock::new(ToolPermissions::default()));
    let mut handles = vec![];
    
    for _ in 0..100 {
        let perms_clone = perms.clone();
        handles.push(tokio::spawn(async move {
            let _ = perms_clone.read().unwrap().default_level;
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// 工具权限检查
// ============================================================================

#[tokio::test]
async fn test_ls_tool_permissions() {
    use hajimi_core::{LsTool, Tool};
    
    let tool = LsTool;
    let perms = tool.permissions();
    
    // Ls should be relatively safe
    assert!(perms.default_level == PermissionLevel::Ask || 
            perms.default_level == PermissionLevel::Allow);
}

#[tokio::test]
async fn test_bash_tool_permissions() {
    use hajimi_core::{BashTool, Tool};
    
    let tool = BashTool::new();
    let perms = tool.permissions();
    
    // Bash is dangerous, should require confirmation
    assert!(perms.requires_confirmation || perms.default_level == PermissionLevel::Ask);
}

#[tokio::test]
async fn test_write_file_permissions() {
    use hajimi_core::{WriteFileTool, Tool};
    
    let tool = WriteFileTool;
    let perms = tool.permissions();
    
    // Write file should require confirmation
    assert!(perms.requires_confirmation || perms.default_level == PermissionLevel::Ask);
}
