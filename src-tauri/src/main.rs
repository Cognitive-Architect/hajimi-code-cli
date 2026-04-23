#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::Arc;

use engine_llm_core::{AnthropicClient, LlmClient, OllamaClient, OpenAiClient};
use engine_tool_system::{
    AnalyzeTool, BashTool, CargoBuildTool, CmakeTool, DeleteFileTool, EditFileTool,
    FetchUrlTool, FindTool, GenerateDocsTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitStatusTool, GlobTool, GraphTool, GrepTool, JsBundleAnalyzerTool, ListDirectoryTool,
    LspDefinitionTool, LspHoverTool, LspInitTool, LspReferencesTool, LsTool, MakeTool,
    McpInitTool, McpInvokeTool, CoverageReportTool, BenchmarkTool, NpmRunTool,
    PowerShellTool, ReadFileTool, RefactorCodeTool, RunTestsTool, RustDocGeneratorTool,
    SecurityAuditTool, ToolOutput, ToolRegistry, UpdateReadmeTool, ViewImageTool,
    WebSearchTool, WriteFileTool,
};
use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;

// ------------------------------------------------------------------
// App State
// ------------------------------------------------------------------
struct AppState {
    registry: ToolRegistry,
}

fn build_registry() -> ToolRegistry {
    let mut r = ToolRegistry::new();
    r.register(Arc::new(AnalyzeTool::new()));
    r.register(Arc::new(BashTool::new()));
    r.register(Arc::new(CargoBuildTool::new()));
    r.register(Arc::new(CmakeTool::new()));
    r.register(Arc::new(DeleteFileTool::new()));
    r.register(Arc::new(EditFileTool::new()));
    r.register(Arc::new(FetchUrlTool::new()));
    r.register(Arc::new(FindTool::new()));
    r.register(Arc::new(GenerateDocsTool::new()));
    r.register(Arc::new(GitCommitTool::new()));
    r.register(Arc::new(GitDiffTool::new()));
    r.register(Arc::new(GitLogTool::new()));
    r.register(Arc::new(GitStatusTool::new()));
    r.register(Arc::new(GlobTool::new()));
    r.register(Arc::new(GraphTool::new()));
    r.register(Arc::new(GrepTool::new()));
    r.register(Arc::new(JsBundleAnalyzerTool::new()));
    r.register(Arc::new(ListDirectoryTool::new()));
    r.register(Arc::new(LspDefinitionTool::new()));
    r.register(Arc::new(LspHoverTool::new()));
    r.register(Arc::new(LspInitTool::new()));
    r.register(Arc::new(LspReferencesTool::new()));
    r.register(Arc::new(LsTool::new()));
    r.register(Arc::new(MakeTool::new()));
    r.register(Arc::new(McpInitTool::new()));
    r.register(Arc::new(McpInvokeTool::new()));
    r.register(Arc::new(CoverageReportTool::new()));
    r.register(Arc::new(BenchmarkTool::new()));
    r.register(Arc::new(NpmRunTool::new()));
    r.register(Arc::new(PowerShellTool::new()));
    r.register(Arc::new(ReadFileTool::new()));
    r.register(Arc::new(RefactorCodeTool::new()));
    r.register(Arc::new(RunTestsTool::new()));
    r.register(Arc::new(RustDocGeneratorTool::new()));
    r.register(Arc::new(SecurityAuditTool::new()));
    r.register(Arc::new(UpdateReadmeTool::new()));
    r.register(Arc::new(ViewImageTool::new()));
    r.register(Arc::new(WebSearchTool::new()));
    r.register(Arc::new(WriteFileTool::new()));
    r
}

// ------------------------------------------------------------------
// Legacy commands
// ------------------------------------------------------------------
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust.", name)
}

#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    Ok(entries)
}

#[tauri::command]
fn run_command(cmd: &str, args: Vec<String>) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(format!("exit code {:?}\nstderr: {}", output.status.code(), stderr));
    }
    Ok(stdout)
}

// ------------------------------------------------------------------
// Tool-system commands
// ------------------------------------------------------------------
#[derive(Serialize, Clone)]
struct ToolInfo {
    name: String,
    description: String,
}

#[derive(Serialize, Clone)]
struct ToolResult {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
}

impl From<ToolOutput> for ToolResult {
    fn from(o: ToolOutput) -> Self {
        Self {
            stdout: o.stdout,
            stderr: o.stderr,
            exit_code: o.exit_code,
        }
    }
}

#[tauri::command]
fn list_tools(state: tauri::State<'_, AppState>) -> Vec<ToolInfo> {
    state
        .registry
        .list()
        .into_iter()
        .filter_map(|name| {
            state.registry.get(name).map(|t| ToolInfo {
                name: name.to_string(),
                description: t.description().to_string(),
            })
        })
        .collect()
}

#[tauri::command]
async fn execute_tool(
    state: tauri::State<'_, AppState>,
    name: String,
    args: Value,
) -> Result<ToolResult, String> {
    let tool = state
        .registry
        .get(&name)
        .ok_or_else(|| format!("tool '{}' not found", name))?;
    let output = tool.execute(args).await.map_err(|e| e.message)?;
    Ok(output.into())
}

// ------------------------------------------------------------------
// LLM commands
// ------------------------------------------------------------------
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StreamEvent {
    chunk: String,
    done: bool,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
struct ProviderInfo {
    name: String,
    available: bool,
    default_model: String,
}

#[tauri::command]
fn get_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            name: "ollama".into(),
            available: true,
            default_model: "llama3".into(),
        },
        ProviderInfo {
            name: "anthropic".into(),
            available: std::env::var("ANTHROPIC_API_KEY").is_ok(),
            default_model: "claude-3-sonnet-20240229".into(),
        },
        ProviderInfo {
            name: "openai".into(),
            available: std::env::var("OPENAI_API_KEY").is_ok(),
            default_model: "gpt-4".into(),
        },
    ]
}

#[tauri::command]
async fn stream_chat(
    provider: String,
    prompt: String,
    on_event: Channel<StreamEvent>,
) -> Result<(), String> {
    let client: Box<dyn LlmClient> = match provider.as_str() {
        "ollama" => Box::new(OllamaClient::default_local()),
        "anthropic" => Box::new(
            AnthropicClient::from_env()
                .map_err(|e| format!("anthropic init failed: {}", e))?,
        ),
        "openai" => Box::new(
            OpenAiClient::from_env()
                .map_err(|e| format!("openai init failed: {}", e))?,
        ),
        _ => return Err(format!("unknown provider: {}", provider)),
    };

    let mut stream = client
        .stream_chat(prompt)
        .await
        .map_err(|e| format!("stream start failed: {}", e))?;

    while let Some(chunk) = stream.next().await {
        let (text, is_done, is_error) = match chunk {
            engine_llm_core::StreamChunk::Output(t) => (t, false, false),
            engine_llm_core::StreamChunk::Error(e) => (e, false, true),
            engine_llm_core::StreamChunk::Done => (String::new(), true, false),
        };
        on_event
            .send(StreamEvent {
                chunk: text,
                done: is_done,
                error: if is_error { Some("LLM error".into()) } else { None },
            })
            .map_err(|e| e.to_string())?;
        if is_done {
            break;
        }
    }

    Ok(())
}

// ------------------------------------------------------------------
// Main
// ------------------------------------------------------------------
fn main() {
    let state = AppState {
        registry: build_registry(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            greet,
            read_file,
            write_file,
            list_dir,
            run_command,
            list_tools,
            execute_tool,
            get_providers,
            stream_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
