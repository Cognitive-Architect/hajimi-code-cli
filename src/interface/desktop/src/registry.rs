use engine_tool_system::ToolRegistry;
use engine_tool_system::{
    AnalyzeTool, BenchmarkTool, CargoBuildTool, CmakeTool, CoverageReportTool, DeleteFileTool,
    EditFileTool, FetchUrlTool, FindTool, GenerateDocsTool, GeneratePrDescriptionTool,
    GitCommitTool, GitDiffTool, GitLogTool, GitStatusTool, GlobTool, GraphTool, GrepTool,
    JsBundleAnalyzerTool, ListDirectoryTool, LsTool, LspDefinitionTool, LspHoverTool, LspInitTool,
    LspReferencesTool, MakeTool, McpInitTool, McpInvokeTool, NpmRunTool, PowerShellTool,
    ReadFileTool, RefactorCodeTool, RunTestsTool, RustDocGeneratorTool, SecurityAuditTool,
    SmartCommitTool, UpdateReadmeTool, ViewImageTool, WebSearchTool, WriteFileTool,
};
use std::path::Path;
use std::sync::Arc;

pub fn build_registry(workspace_root: &Path) -> ToolRegistry {
    let mut r = ToolRegistry::new();
    let workspace_paths = vec![workspace_root.to_path_buf()];
    r.register(Arc::new(AnalyzeTool::new()));
    r.register(Arc::new(PowerShellTool::with_paths(Some(
        workspace_paths.clone(),
    ))));
    r.register(Arc::new(CargoBuildTool::new()));
    r.register(Arc::new(CmakeTool::new()));
    r.register(Arc::new(DeleteFileTool::with_allowed_paths(
        workspace_paths.clone(),
    )));
    r.register(Arc::new(EditFileTool::with_allowed_paths(
        workspace_paths.clone(),
    )));
    r.register(Arc::new(FetchUrlTool::new()));
    r.register(Arc::new(FindTool::new()));
    r.register(Arc::new(GenerateDocsTool::new()));
    r.register(Arc::new(GitCommitTool::new()));
    r.register(Arc::new(GitDiffTool::new()));
    r.register(Arc::new(GitLogTool::new()));
    r.register(Arc::new(GitStatusTool::new()));
    r.register(Arc::new(SmartCommitTool::new()));
    r.register(Arc::new(GeneratePrDescriptionTool::new()));
    r.register(Arc::new(GlobTool::new()));
    r.register(Arc::new(GraphTool::new()));
    r.register(Arc::new(GrepTool::new()));
    r.register(Arc::new(JsBundleAnalyzerTool::new()));
    r.register(Arc::new(ListDirectoryTool::new()));
    r.register(Arc::new(LspDefinitionTool::new()));
    r.register(Arc::new(LspHoverTool::new()));
    r.register(Arc::new(LspInitTool::new()));
    r.register(Arc::new(LspReferencesTool::new()));
    r.register(Arc::new(LsTool::with_allowed_paths(
        workspace_paths.clone(),
    )));
    r.register(Arc::new(MakeTool::new()));
    r.register(Arc::new(McpInitTool::new()));
    r.register(Arc::new(McpInvokeTool::new()));
    r.register(Arc::new(CoverageReportTool::new()));
    r.register(Arc::new(BenchmarkTool::new()));
    r.register(Arc::new(NpmRunTool::new()));
    r.register(Arc::new(ReadFileTool::with_allowed_paths(
        workspace_paths.clone(),
    )));
    r.register(Arc::new(RefactorCodeTool::new()));
    r.register(Arc::new(RunTestsTool::new()));
    r.register(Arc::new(RustDocGeneratorTool::new()));
    r.register(Arc::new(SecurityAuditTool::new()));
    r.register(Arc::new(UpdateReadmeTool::new()));
    r.register(Arc::new(ViewImageTool::new()));
    r.register(Arc::new(WebSearchTool::new()));
    r.register(Arc::new(WriteFileTool::with_allowed_paths(workspace_paths)));
    r
}
