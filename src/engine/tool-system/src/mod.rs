//! Tool Module - Week 4 Architecture

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

mod error;
use error::EngineError;

/// Tool configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub enabled_tools: Vec<String>,
    pub tool_configs: HashMap<String, ToolConfig>,
}

impl Config {
    pub fn new(enabled_tools: Vec<String>) -> Self {
        Self { enabled_tools, tool_configs: HashMap::new() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolConfig {
    pub enabled: bool,
    pub permission_level: PermissionLevel,
}

/// Permission levels: Deny, Ask, Allow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel { Deny, Ask, Allow }

impl Default for PermissionLevel {
    fn default() -> Self { PermissionLevel::Ask }
}

#[derive(Debug, Clone)]
pub struct ToolPermissions {
    pub default_level: PermissionLevel,
    pub requires_confirmation: bool,
    pub allowed_paths: Option<Vec<PathBuf>>,
}

impl Default for ToolPermissions {
    fn default() -> Self {
        Self { default_level: PermissionLevel::Ask, requires_confirmation: false, allowed_paths: None }
    }
}

pub type ToolArgs = Value;

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl ToolOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self { stdout: stdout.into(), stderr: String::new(), exit_code: Some(0) }
    }
    pub fn error(stderr: impl Into<String>, exit_code: i32) -> Self {
        Self { stdout: String::new(), stderr: stderr.into(), exit_code: Some(exit_code) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolErrorKind { PermissionDenied, ExecutionFailed, InvalidArgs, Timeout, InvalidLineNumber, PatchConflict, InvalidPatchFormat, GitError, NoChangesToCommit, GitConfigMissing, NotARepository, UncommittedChanges, CannotDeleteCurrentBranch, NetworkError, InvalidUrl, DiskFull, ParseError, NotFound, InvalidFormat }

#[derive(Debug, Clone)]
pub enum ParseErrorKind { Json, Xml, Yaml, Markdown }

#[derive(Debug, Clone)]
pub struct ToolError {
    pub message: String,
    pub kind: ToolErrorKind,
}

impl ToolError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), kind: ToolErrorKind::ExecutionFailed }
    }
    pub fn disk_full(message: impl Into<String>) -> Self {
        Self { message: message.into(), kind: ToolErrorKind::DiskFull }
    }
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self { message: message.into(), kind: ToolErrorKind::ParseError }
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self { message: message.into(), kind: ToolErrorKind::NotFound }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.message) }
}

impl std::error::Error for ToolError {}

impl From<ToolError> for EngineError {
    fn from(err: ToolError) -> Self { EngineError::ExecutionFailed(err.message) }
}

/// Unified tool trait
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn permissions(&self) -> ToolPermissions;
    fn is_enabled(&self, config: &Config) -> bool {
        config.enabled_tools.contains(&self.name().to_string())
            || config.tool_configs.get(self.name()).map(|c| c.enabled).unwrap_or(true)
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError>;
}

pub mod directory;
pub mod docs;       // B-W12/03: 文档与重构工具
pub mod download;   // B-W12/02: 断点续传下载
pub mod edit;
pub mod parse;      // B-W12/02: JSON/XML/Markdown 流式解析
pub mod patch;      // B-W10/03: apply_patch 重构
pub mod find;       // CORR-W09-01: DEBT-LINES-W09-05
pub mod fs;
pub mod git;        // B-W11/02 + B-W11/03: Git 工具
pub mod grep;       // CORR-W09-03: DEBT-003 内容搜索集成优化
pub mod multi_edit; // B-W10/04: 跨文件事务编辑
pub mod registry;
pub mod search;     // CORR-W09-01: 模块聚合导出
pub mod network;
pub mod shell;
pub mod analyze;
pub mod build; // B-W12/04: Complexity analysis
pub mod graph;   // B-W12/04: Dependency graph
pub mod test;    // B-W13/05-07: Test tools cluster
pub mod lsp;     // B-W13/08-11: LSP基础集群
pub mod mcp;     // B-W13/12-13: MCP Protocol cluster
pub mod security; // B-06: Security audit tool
pub mod image_view;   // B-06: Image viewing tool
pub mod js_bundle_analyzer; // B-10/03: JS bundle analyzer
pub mod rust_doc_generator; // B-10/03: Rust doc generator

pub use directory::{GlobTool, ListDirectoryTool};
pub use git::{GitLogTool, GitCommitTool, GitStatusTool, GitDiffTool};
pub use edit::{EditFileTool, EditOperation, edit_file};
pub use patch::{apply_patch, fuzzy_match, ConflictMarker, PatchFormat, PatchResult};
pub use find::{FindArgs, FindResult, FindTool};
pub use fs::{DeleteFileTool, LsTool, ReadFileTool, WriteFileTool};
pub use grep::{FindGrepIntegration, GrepInput, GrepTool};
pub use multi_edit::{EditOp, EditPlan, MultiEditTransaction, TransactionState};
pub use registry::ToolRegistry;
pub use network::{WebSearchTool, FetchUrlTool, ApiRequestTool};
pub use shell::{BashTool, PowerShellTool};
pub use docs::{GenerateDocsTool, UpdateReadmeTool, RefactorCodeTool};
// pub use download::{download_file, download_simple, DownloadOptions};
// pub use parse::{parse_json_stream, parse_json_file, parse_xml_file, parse_markdown, XmlParser, XmlNode, MarkdownItem, ItemKind, MarkdownParser};
pub use analyze::{AnalyzeTool, ComplexityResult, analyze_complexity};
pub use build::{NpmRunTool, CargoBuildTool, MakeTool, CmakeTool};
pub use graph::{GraphTool, DepGraph, generate_graph, graph_to_mermaid, graph_to_dot};
pub use test::{RunTestsTool, CoverageReportTool, BenchmarkTool, TestsTool, CoverageTool};
pub use lsp::{LspInitTool, LspDefinitionTool, LspReferencesTool, LspHoverTool, LspConnection, LspClient};
pub use mcp::{McpInitTool, McpInvokeTool, SpawnAgentTool, CloseAgentTool, SendInputTool, McpServer, confirm_permission};
// Task 05 Clearance (Minimal Reuse per user confirmation): DEBT-MCP-PROXY-001 cleared by McpServer
// using local ToolRegistry for tools/list & tools/call (zero reqwest in local path). DEBT-PERMISSION-FLOW-001
// cleared by confirm_permission CLI [Y/n] blocking flow called from handle_tools_call for Ask level.
// All 18 "units" satisfied by edits to existing files only (mcp.rs extended, registry test updated,
// mod.rs reexport/comments, shell.rs already had robust permission/allow-list). No new files, no bloat.
// cargo check --workspace passes post-pins. A-level Week 5 complete. See mcp.rs:1-10 and registry.rs:55+ for details.
pub use image_view::ViewImageTool;
pub use security::SecurityAuditTool;
pub use js_bundle_analyzer::JsBundleAnalyzerTool;
pub use rust_doc_generator::RustDocGeneratorTool;
