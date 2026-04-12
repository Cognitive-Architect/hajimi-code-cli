//! Tool Registry Integration Test - B-W04-04
//!
//! Validates all 5 tools are properly registered

use hajimi_core::{create_default_registry, BashTool, GrepTool, LsTool, ReadFileTool, Tool, WriteFileTool};
use std::sync::Arc;

#[test]
fn test_tool_registry_integration() {
    let registry = create_default_registry();
    
    // Verify all 5 tools are registered
    let tools = registry.list();
    assert_eq!(tools.len(), 5, "Expected 5 tools in registry");
    
    // Verify each tool exists
    assert!(registry.get("read_file").is_some(), "ReadFileTool not found");
    assert!(registry.get("write_file").is_some(), "WriteFileTool not found");
    assert!(registry.get("bash").is_some(), "BashTool not found");
    assert!(registry.get("grep").is_some(), "GrepTool not found");
    assert!(registry.get("ls").is_some(), "LsTool not found");
}

#[test]
fn test_ls_tool_has_allow_permission() {
    use hajimi_core::tool::PermissionLevel;
    
    let tool = LsTool::new();
    let perms = tool.permissions();
    
    assert_eq!(perms.default_level, PermissionLevel::Allow, "LsTool should have Allow permission");
}

#[test]
fn test_all_tools_have_correct_names() {
    let read_tool = ReadFileTool::new();
    let write_tool = WriteFileTool::new();
    let bash_tool = BashTool::new();
    let grep_tool = GrepTool::new();
    let ls_tool = LsTool::new();
    
    assert_eq!(read_tool.name(), "read_file");
    assert_eq!(write_tool.name(), "write_file");
    assert_eq!(bash_tool.name(), "bash");
    assert_eq!(grep_tool.name(), "grep");
    assert_eq!(ls_tool.name(), "ls");
}
