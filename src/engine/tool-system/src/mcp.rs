//! MCP Protocol Tools - B-W13/12-13: MCP集群 + B-05/06 Agent生命周期管理

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;
use uuid::Uuid;

use super::{Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};

const MCP_VERSION: &str = "2024-11-05";
const MAX_LOG_LINES: usize = 100;
const GRACEFUL_SHUTDOWN_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpTool { name: String, description: Option<String>, input_schema: Value }

#[derive(Debug, Clone)]
enum McpTransport { Sse(String), Stdio(String, Vec<String>) }

#[derive(Debug, Clone)]
struct McpConn { transport: McpTransport, tools: Vec<McpTool>, connected: bool }

static MCP_CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, Arc<Mutex<McpConn>>>>>
    = once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn check_version(v: &str) -> Result<(), ToolError> {
    if v != MCP_VERSION { Err(ToolError { message: format!("MCP version mismatch: {} vs {}", v, MCP_VERSION), kind: ToolErrorKind::InvalidArgs }) }
    else { Ok(()) }
}

async fn mcp_request(endpoint: &str, method: &str, params: Value) -> Result<Value, ToolError> {
    let client = reqwest::Client::new();
    let resp = client.post(endpoint).header("Accept", "text/event-stream").json(&json!({
        "jsonrpc": "2.0", "id": 1, "method": method, "params": params
    })).send().await.map_err(|e| ToolError { message: format!("MCP SSE error: {}", e), kind: ToolErrorKind::NetworkError })?;
    let data: Value = resp.json().await.map_err(|e| ToolError { message: format!("MCP parse: {}", e), kind: ToolErrorKind::ParseError })?;
    if let Some(err) = data.get("error") { return Err(ToolError { message: format!("MCP error: {}", err), kind: ToolErrorKind::ExecutionFailed }); }
    Ok(data.get("result").cloned().unwrap_or(json!({})))
}

pub struct McpInitTool;
impl McpInitTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for McpInitTool {
    fn name(&self) -> &str { "mcp_init" }
    fn description(&self) -> &str { "Initialize MCP server (SSE/stdio), fetch tools/list" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions::default() }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let url = args.get("server_url").and_then(|v| v.as_str()).ok_or_else(|| ToolError { message: "Missing server_url".into(), kind: ToolErrorKind::InvalidArgs })?;
        let transport = args.get("transport").and_then(|v| v.as_str()).unwrap_or("sse");
        let t = if transport == "stdio" {
            let p: Vec<&str> = url.split_whitespace().collect();
            McpTransport::Stdio(p[0].to_string(), p[1..].iter().map(|s| s.to_string()).collect())
        } else { McpTransport::Sse(url.to_string()) };
        let tools: Vec<McpTool> = match t.clone() {
            McpTransport::Sse(ep) => {
                let r = mcp_request(&ep, "tools/list", json!({})).await?;
                serde_json::from_value(r.get("tools").cloned().unwrap_or(json!([]))).map_err(|e| ToolError { message: format!("tools/list parse: {}", e), kind: ToolErrorKind::ParseError })?
            }
            McpTransport::Stdio(proc, a) => {
                let o = Command::new(&proc).args(&a).arg("--mcp-list").output().await.map_err(|e| ToolError { message: format!("MCP stdio crash: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
                if !o.status.success() { return Err(ToolError { message: format!("MCP stdio error: {}", String::from_utf8_lossy(&o.stderr)), kind: ToolErrorKind::ExecutionFailed }); }
                serde_json::from_slice(&o.stdout).map_err(|e| ToolError { message: format!("stdio parse: {}", e), kind: ToolErrorKind::ParseError })?
            }
        };
        let names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        let conn = McpConn { transport: t, tools, connected: true };
        MCP_CACHE.lock().await.insert(url.to_string(), Arc::new(Mutex::new(conn)));
        Ok(ToolOutput::success(json!({ "connected": true, "version": MCP_VERSION, "tools": names.len(), "tool_names": names }).to_string()))
    }
}

pub struct McpInvokeTool;
impl McpInvokeTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for McpInvokeTool {
    fn name(&self) -> &str { "mcp_invoke" }
    fn description(&self) -> &str { "Invoke MCP tool via tools/call, chain with api_request" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions::default() }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let url = args.get("server_url").and_then(|v| v.as_str()).ok_or_else(|| ToolError { message: "Missing server_url".into(), kind: ToolErrorKind::InvalidArgs })?;
        let tool = args.get("tool_name").and_then(|v| v.as_str()).ok_or_else(|| ToolError { message: "Missing tool_name".into(), kind: ToolErrorKind::InvalidArgs })?;
        let targs = args.get("arguments").cloned().unwrap_or(json!({}));
        let cache = MCP_CACHE.lock().await;
        let c = cache.get(url).ok_or_else(|| ToolError { message: format!("MCP not init: {}", url), kind: ToolErrorKind::InvalidArgs })?.clone();
        drop(cache);
        let conn = c.lock().await;
        if !conn.tools.iter().any(|t| t.name == tool) { return Err(ToolError { message: format!("MCP tool not exist: {}", tool), kind: ToolErrorKind::NotFound }); }
        let result = match &conn.transport {
            McpTransport::Sse(ep) => mcp_request(ep, "tools/call", json!({ "name": tool, "arguments": targs })).await?,
            McpTransport::Stdio(proc, a) => {
                let o = Command::new(proc).args(a).arg("--mcp-call").arg(tool).arg(targs.to_string()).output().await.map_err(|e| ToolError { message: format!("MCP stdio crash: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
                if !o.status.success() { return Err(ToolError { message: format!("MCP tool error: {}", String::from_utf8_lossy(&o.stderr)), kind: ToolErrorKind::ExecutionFailed }); }
                serde_json::from_slice(&o.stdout).map_err(|e| ToolError { message: format!("stdio parse: {}", e), kind: ToolErrorKind::ParseError })?
            }
        };
        Ok(ToolOutput::success(result.to_string()))
    }
}

#[derive(Debug)]
pub struct Agent {
    pub id: String,
    pub child: Child,
    pub stdin: Option<tokio::process::ChildStdin>,
    pub stdout_logs: Arc<Mutex<VecDeque<String>>>,
    pub stderr_logs: Arc<Mutex<VecDeque<String>>>,
    pub start_time: Instant,
    pub command: String,
    pub pid: u32,
}

static AGENT_POOL: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, Agent>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn spawn_log_capture(stdout: tokio::process::ChildStdout, stderr: tokio::process::ChildStderr, 
                     stdout_logs: Arc<Mutex<VecDeque<String>>>, stderr_logs: Arc<Mutex<VecDeque<String>>>) {
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut logs = stdout_logs.lock().await;
            if logs.len() >= MAX_LOG_LINES { logs.pop_front(); }
            logs.push_back(line);
        }
    });
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let mut logs = stderr_logs.lock().await;
            if logs.len() >= MAX_LOG_LINES { logs.pop_front(); }
            logs.push_back(line);
        }
    });
}

pub struct SpawnAgentTool;
impl SpawnAgentTool { pub fn new() -> Self { Self } }

#[derive(Debug, Deserialize)]
struct SpawnArgs {
    command: String,
    #[serde(default)] args: Vec<String>,
    #[serde(default)] env: HashMap<String, String>,
    #[serde(default)] cwd: Option<String>,
    #[serde(default)] memory: Option<usize>,
    #[serde(default)] timeout: Option<u64>,
}

#[async_trait]
impl Tool for SpawnAgentTool {
    fn name(&self) -> &str { "spawn_agent" }
    fn description(&self) -> &str { "Spawn a child process agent with UUID tracking and resource limits" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions::default() }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: SpawnArgs = serde_json::from_value(args)
            .map_err(|e| ToolError { message: format!("Invalid spawn args: {}", e), kind: ToolErrorKind::InvalidArgs })?;
        if let Some(mem) = args.memory {
            if mem > 8192 { return Err(ToolError { message: format!("Memory limit {}MB exceeds max 8192MB", mem), kind: ToolErrorKind::ExecutionFailed }); }
        }
        if let Some(t) = args.timeout {
            if t > 3600 { return Err(ToolError { message: format!("Timeout {}s exceeds max 3600s", t), kind: ToolErrorKind::ExecutionFailed }); }
        }
        let agent_id = Uuid::new_v4().to_string();
        let mut cmd_parts: Vec<&str> = args.command.split_whitespace().collect();
        if cmd_parts.is_empty() { return Err(ToolError { message: "Empty command".into(), kind: ToolErrorKind::InvalidArgs }); }
        let program = cmd_parts.remove(0);
        let mut cmd = Command::new(program);
        cmd.args(&cmd_parts).args(&args.args);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped());
        for (k, v) in &args.env { cmd.env(k, v); }
        if let Some(cwd) = &args.cwd { cmd.current_dir(cwd); }
        let mut child = cmd.spawn().map_err(|e| ToolError { message: format!("Failed to spawn agent: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        let pid = child.id().ok_or_else(|| ToolError { message: "Failed to get child PID".into(), kind: ToolErrorKind::ExecutionFailed })?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or_else(|| ToolError { message: "Failed to capture stdout".into(), kind: ToolErrorKind::ExecutionFailed })?;
        let stderr = child.stderr.take().ok_or_else(|| ToolError { message: "Failed to capture stderr".into(), kind: ToolErrorKind::ExecutionFailed })?;
        let stdout_logs = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));
        let stderr_logs = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));
        spawn_log_capture(stdout, stderr, stdout_logs.clone(), stderr_logs.clone());
        let agent = Agent {
            id: agent_id.clone(), child, stdin, stdout_logs, stderr_logs,
            start_time: Instant::now(), command: args.command.clone(), pid,
        };
        AGENT_POOL.lock().await.insert(agent_id.clone(), agent);
        Ok(ToolOutput::success(json!({"agent_id": agent_id, "pid": pid, "command": args.command, "status": "running"}).to_string()))
    }
}

pub struct SendInputTool;
impl SendInputTool { pub fn new() -> Self { Self } }

#[derive(Debug, Deserialize)]
struct SendInputArgs { agent_id: String, input: String }

#[async_trait]
impl Tool for SendInputTool {
    fn name(&self) -> &str { "send_input" }
    fn description(&self) -> &str { "Send input to agent's STDIN" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions::default() }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: SendInputArgs = serde_json::from_value(args)
            .map_err(|e| ToolError { message: format!("Invalid send_input args: {}", e), kind: ToolErrorKind::InvalidArgs })?;
        let mut pool = AGENT_POOL.lock().await;
        let agent = pool.get_mut(&args.agent_id).ok_or_else(|| ToolError { message: format!("Agent not found: {}", args.agent_id), kind: ToolErrorKind::NotFound })?;
        let stdin = agent.stdin.as_mut().ok_or_else(|| ToolError { message: format!("Agent {} stdin is closed", args.agent_id), kind: ToolErrorKind::ExecutionFailed })?;
        stdin.write_all(args.input.as_bytes()).await.map_err(|e| ToolError { message: format!("Failed to write to stdin: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        stdin.flush().await.map_err(|e| ToolError { message: format!("Failed to flush stdin: {}", e), kind: ToolErrorKind::ExecutionFailed })?;
        drop(pool);
        Ok(ToolOutput::success(json!({"agent_id": args.agent_id, "bytes_sent": args.input.len(), "status": "sent"}).to_string()))
    }
}

pub struct CloseAgentTool;
impl CloseAgentTool { pub fn new() -> Self { Self } }

#[derive(Debug, Deserialize)]
struct CloseArgs { agent_id: String, #[serde(default)] force: bool }

#[async_trait]
impl Tool for CloseAgentTool {
    fn name(&self) -> &str { "close_agent" }
    fn description(&self) -> &str { "Gracefully terminate agent (SIGTERM then SIGKILL), cleanup from pool" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions::default() }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let args: CloseArgs = serde_json::from_value(args)
            .map_err(|e| ToolError { message: format!("Invalid close args: {}", e), kind: ToolErrorKind::InvalidArgs })?;
        let mut pool = AGENT_POOL.lock().await;
        let mut agent = pool.remove(&args.agent_id).ok_or_else(|| ToolError { message: format!("Agent not found: {}", args.agent_id), kind: ToolErrorKind::NotFound })?;
        agent.stdin.take();
        let start_close = Instant::now();
        if args.force {
            let _ = agent.child.kill().await;
        } else {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                let _ = kill(Pid::from_raw(agent.pid as i32), Signal::SIGTERM);
            }
            #[cfg(not(unix))]
            { let _ = agent.child.kill().await; }
            match timeout(Duration::from_secs(GRACEFUL_SHUTDOWN_SECS), agent.child.wait()).await {
                Ok(Ok(_)) => {},
                _ => { let _ = agent.child.kill().await; }
            }
        }
        let exit_status = match timeout(Duration::from_secs(1), agent.child.wait()).await {
            Ok(Ok(status)) => status.code(),
            _ => None,
        };
        let runtime = start_close.duration_since(agent.start_time).as_secs();
        drop(pool);
        Ok(ToolOutput::success(json!({"agent_id": args.agent_id, "pid": agent.pid, "exit_code": exit_status, "runtime_secs": runtime, "force": args.force, "status": "closed"}).to_string()))
    }
}

pub async fn get_agent_logs(agent_id: &str) -> Result<(Vec<String>, Vec<String>), ToolError> {
    let pool = AGENT_POOL.lock().await;
    let agent = pool.get(agent_id).ok_or_else(|| ToolError { message: format!("Agent not found: {}", agent_id), kind: ToolErrorKind::NotFound })?;
    let stdout: Vec<String> = agent.stdout_logs.lock().await.iter().cloned().collect();
    let stderr: Vec<String> = agent.stderr_logs.lock().await.iter().cloned().collect();
    Ok((stdout, stderr))
}

pub async fn list_agents() -> Vec<(String, u32, String, u64)> {
    let pool = AGENT_POOL.lock().await;
    pool.values().map(|a| {
        let runtime = a.start_time.elapsed().as_secs();
        (a.id.clone(), a.pid, a.command.clone(), runtime)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_spawn_agent_success() -> Result<(), Box<dyn std::error::Error>> {
        let tool = SpawnAgentTool::new();
        let args = json!({"command": "echo hello"});
        let result = tool.execute(args).await?;
        assert!(result.stdout.contains("agent_id"));
        Ok(())
    }
    
    #[tokio::test]
    async fn test_spawn_agent_memory_limit_exceeded() -> Result<(), Box<dyn std::error::Error>> {
        let tool = SpawnAgentTool::new();
        let args = json!({"command": "echo test", "memory": 10000});
        let result = tool.execute(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("exceeds max"));
        Ok(())
    }
    
    #[tokio::test]
    async fn test_close_invalid_agent() -> Result<(), Box<dyn std::error::Error>> {
        let tool = CloseAgentTool::new();
        let args = json!({"agent_id": "non-existent-uuid-1234"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ToolErrorKind::NotFound);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_send_input_invalid_agent() -> Result<(), Box<dyn std::error::Error>> {
        let tool = SendInputTool::new();
        let args = json!({"agent_id": "non-existent-uuid-1234", "input": "test"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ToolErrorKind::NotFound);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_agent_lifecycle_full() -> Result<(), Box<dyn std::error::Error>> {
        let spawn_tool = SpawnAgentTool::new();
        let spawn_result = spawn_tool.execute(json!({"command": "cat"})).await?;
        let agent_id: Value = serde_json::from_str(&spawn_result.stdout)?;
        let id = agent_id["agent_id"].as_str().ok_or("Missing agent_id")?.to_string();
        let input_tool = SendInputTool::new();
        assert!(input_tool.execute(json!({"agent_id": id, "input": "hello\n"})).await.is_ok());
        let close_tool = CloseAgentTool::new();
        assert!(close_tool.execute(json!({"agent_id": id})).await.is_ok());
        assert!(!AGENT_POOL.lock().await.contains_key(&id));
        Ok(())
    }
}
