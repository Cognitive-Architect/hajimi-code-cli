//! LSP Tools - B-W13/08-11: LSP基础集群
//!
//! 四个核心LSP工具:
//! - lsp_init: 语言服务器初始化握手
//! - lsp_definition: 跳转到定义 (GotoDefinition)
//! - lsp_references: 查找引用 (References)
//! - lsp_hover: 悬停提示 (Hover)

use async_trait::async_trait;
use lsp_types::{
    ClientCapabilities, GotoDefinitionResponse, Hover, InitializeParams, InitializeResult,
    Location, Position, TextDocumentClientCapabilities, TextDocumentIdentifier,
    TextDocumentPositionParams, TextDocumentSyncClientCapabilities,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::{Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};

const LSP_TIMEOUT: Duration = Duration::from_secs(30);
const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC请求结构
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest<T> {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

impl<T> JsonRpcRequest<T> {
    fn new(id: u64, method: impl Into<String>, params: Option<T>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC响应结构
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC错误结构
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl From<JsonRpcError> for ToolError {
    fn from(err: JsonRpcError) -> Self {
        ToolError {
            message: format!("LSP Error ({}): {}", err.code, err.message),
            kind: ToolErrorKind::ExecutionFailed,
        }
    }
}

/// LSP客户端连接类型
#[derive(Debug, Clone)]
pub enum LspConnection {
    Stdio { cmd: String, args: Vec<String> },
    Tcp { host: String, port: u16 },
}

/// LSP客户端状态
#[derive(Debug)]
pub struct LspClient {
    connection: LspConnection,
    next_id: Arc<Mutex<u64>>,
    initialized: Arc<Mutex<bool>>,
}

impl LspClient {
    pub fn new(connection: LspConnection) -> Self {
        Self {
            connection,
            next_id: Arc::new(Mutex::new(1)),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    async fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().await;
        *id += 1;
        *id
    }

    /// 检查LSP服务器是否可用
    async fn check_server(&self) -> Result<(), ToolError> {
        match &self.connection {
            LspConnection::Stdio { cmd, .. } => {
                match Command::new(cmd)
                    .arg("--version")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ToolError {
                        message: format!("LSP server not found: {}", cmd),
                        kind: ToolErrorKind::NotFound,
                    }),
                }
            }
            LspConnection::Tcp { host, port } => {
                match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ToolError {
                        message: format!("LSP server not responding at {}:{}", host, port),
                        kind: ToolErrorKind::NetworkError,
                    }),
                }
            }
        }
    }

    /// 初始化LSP服务器
    async fn initialize(
        &self,
        root_uri: Option<lsp_types::Url>,
    ) -> Result<InitializeResult, ToolError> {
        self.check_server().await?;

        let id = self.next_id().await;
        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities::default()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            trace: None,
            workspace_folders: None,
            client_info: Some(lsp_types::ClientInfo {
                name: "hajimi-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            locale: None,
        };

        let request = JsonRpcRequest::new(id, "initialize", Some(params));
        let response: JsonRpcResponse<InitializeResult> = self.send_request(&request).await?;

        if let Some(err) = response.error {
            return Err(err.into());
        }

        if let Some(result) = response.result {
            let mut init = self.initialized.lock().await;
            *init = true;
            Ok(result)
        } else {
            Err(ToolError {
                message: "Empty initialize response".to_string(),
                kind: ToolErrorKind::ExecutionFailed,
            })
        }
    }

    /// 发送JSON-RPC请求 (简化版stdio实现)
    async fn send_request<T: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        request: &JsonRpcRequest<T>,
    ) -> Result<JsonRpcResponse<R>, ToolError> {
        let request_json = serde_json::to_string(request)
            .map_err(|e| ToolError::new(format!("Serialize error: {}", e)))?;
        let content_length = request_json.len();
        let message = format!("Content-Length: {}\r\n\r\n{}", content_length, request_json);

        match &self.connection {
            LspConnection::Stdio { cmd, args } => {
                let mut child = Command::new(cmd)
                    .args(args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| ToolError {
                        message: format!("Failed to spawn LSP: {}", e),
                        kind: ToolErrorKind::ExecutionFailed,
                    })?;

                let mut stdin = child
                    .stdin
                    .take()
                    .ok_or_else(|| ToolError::new("Failed to get stdin"))?;
                stdin
                    .write_all(message.as_bytes())
                    .await
                    .map_err(|e| ToolError::new(format!("Write error: {}", e)))?;
                stdin
                    .flush()
                    .await
                    .map_err(|e| ToolError::new(format!("Flush error: {}", e)))?;
                drop(stdin);

                let stdout = child
                    .stdout
                    .take()
                    .ok_or_else(|| ToolError::new("Failed to get stdout"))?;
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();

                // 读取Content-Length头
                let content_len = loop {
                    line.clear();
                    let n = timeout(LSP_TIMEOUT, reader.read_line(&mut line))
                        .await
                        .map_err(|_| ToolError {
                            message: "LSP response timeout".to_string(),
                            kind: ToolErrorKind::Timeout,
                        })?
                        .map_err(|e| ToolError::new(format!("Read error: {}", e)))?;
                    if n == 0 {
                        return Err(ToolError::new("Unexpected EOF"));
                    }
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                        break len_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| ToolError::new("Invalid Content-Length"))?;
                    }
                };

                // 读取空行分隔
                line.clear();
                timeout(LSP_TIMEOUT, reader.read_line(&mut line))
                    .await
                    .map_err(|_| ToolError {
                        message: "LSP response timeout".to_string(),
                        kind: ToolErrorKind::Timeout,
                    })?
                    .map_err(|e| ToolError::new(format!("Read error: {}", e)))?;

                // 读取响应体
                let mut buffer = vec![0u8; content_len];
                timeout(LSP_TIMEOUT, reader.read_exact(&mut buffer))
                    .await
                    .map_err(|_| ToolError {
                        message: "LSP response timeout".to_string(),
                        kind: ToolErrorKind::Timeout,
                    })?
                    .map_err(|e| ToolError::new(format!("Read body error: {}", e)))?;

                let response: JsonRpcResponse<R> = serde_json::from_slice(&buffer)
                    .map_err(|e| ToolError::new(format!("Parse response error: {}", e)))?;

                Ok(response)
            }
            LspConnection::Tcp { host, port } => {
                let addr = format!("{}:{}", host, port);
                let stream = timeout(LSP_TIMEOUT, TcpStream::connect(&addr))
                    .await
                    .map_err(|_| ToolError {
                        message: "TCP connect timeout".to_string(),
                        kind: ToolErrorKind::Timeout,
                    })?
                    .map_err(|e| ToolError {
                        message: format!("TCP connect failed: {}", e),
                        kind: ToolErrorKind::NetworkError,
                    })?;

                let (reader, mut writer) = stream.into_split();
                writer
                    .write_all(message.as_bytes())
                    .await
                    .map_err(|e| ToolError::new(format!("Write error: {}", e)))?;
                writer
                    .flush()
                    .await
                    .map_err(|e| ToolError::new(format!("Flush error: {}", e)))?;
                drop(writer);

                let mut reader = BufReader::new(reader);
                let mut line = String::new();

                let content_len = loop {
                    line.clear();
                    let n = timeout(LSP_TIMEOUT, reader.read_line(&mut line))
                        .await
                        .map_err(|_| ToolError {
                            message: "LSP response timeout".to_string(),
                            kind: ToolErrorKind::Timeout,
                        })?
                        .map_err(|e| ToolError::new(format!("Read error: {}", e)))?;
                    if n == 0 {
                        return Err(ToolError::new("Unexpected EOF"));
                    }
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                        break len_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| ToolError::new("Invalid Content-Length"))?;
                    }
                };

                line.clear();
                timeout(LSP_TIMEOUT, reader.read_line(&mut line))
                    .await
                    .map_err(|_| ToolError {
                        message: "LSP response timeout".to_string(),
                        kind: ToolErrorKind::Timeout,
                    })?
                    .map_err(|e| ToolError::new(format!("Read error: {}", e)))?;

                let mut buffer = vec![0u8; content_len];
                timeout(LSP_TIMEOUT, reader.read_exact(&mut buffer))
                    .await
                    .map_err(|_| ToolError {
                        message: "LSP response timeout".to_string(),
                        kind: ToolErrorKind::Timeout,
                    })?
                    .map_err(|e| ToolError::new(format!("Read body error: {}", e)))?;

                let response: JsonRpcResponse<R> = serde_json::from_slice(&buffer)
                    .map_err(|e| ToolError::new(format!("Parse response error: {}", e)))?;

                Ok(response)
            }
        }
    }
}

/// LSP初始化工具
pub struct LspInitTool;

impl Default for LspInitTool {
    fn default() -> Self {
        Self::new()
    }
}

impl LspInitTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for LspInitTool {
    fn name(&self) -> &str {
        "lsp_init"
    }
    fn description(&self) -> &str {
        "Initialize LSP server with handshake"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let cmd = args
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("rust-analyzer");
        let args_vec: Vec<String> = args
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let root_uri = args
            .get("root_uri")
            .and_then(|v| v.as_str())
            .and_then(|s| lsp_types::Url::parse(s).ok());

        let conn = if let Some(port) = args.get("port").and_then(|v| v.as_u64()) {
            let host = args
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("127.0.0.1")
                .to_string();
            LspConnection::Tcp {
                host,
                port: port as u16,
            }
        } else {
            LspConnection::Stdio {
                cmd: cmd.to_string(),
                args: args_vec,
            }
        };

        let client = LspClient::new(conn);
        let result = client.initialize(root_uri).await?;

        let output = serde_json::json!({
            "server_info": result.server_info,
            "capabilities": result.capabilities
        });
        Ok(ToolOutput::success(output.to_string()))
    }
}

/// LSP定义跳转工具
pub struct LspDefinitionTool;

impl Default for LspDefinitionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl LspDefinitionTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for LspDefinitionTool {
    fn name(&self) -> &str {
        "lsp_definition"
    }
    fn description(&self) -> &str {
        "Go to definition using LSP (GotoDefinition)"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let uri = args
            .get("uri")
            .and_then(|v| v.as_str())
            .and_then(|s| lsp_types::Url::parse(s).ok())
            .ok_or_else(|| ToolError {
                message: "Missing or invalid uri".to_string(),
                kind: ToolErrorKind::InvalidArgs,
            })?;
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let cmd = args
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("rust-analyzer");
        let conn = LspConnection::Stdio {
            cmd: cmd.to_string(),
            args: vec![],
        };
        let client = LspClient::new(conn);

        // 先初始化
        client.initialize(None).await?;

        let id = client.next_id().await;
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        let request = JsonRpcRequest::new(id, "textDocument/definition", Some(params));
        let response: JsonRpcResponse<GotoDefinitionResponse> =
            client.send_request(&request).await?;

        if let Some(err) = response.error {
            return Err(err.into());
        }

        let result = response
            .result
            .map(|r| serde_json::to_string(&r).unwrap_or_default())
            .unwrap_or_else(|| "No definition found".to_string());
        Ok(ToolOutput::success(result))
    }
}

/// LSP引用查找工具
pub struct LspReferencesTool;

impl Default for LspReferencesTool {
    fn default() -> Self {
        Self::new()
    }
}

impl LspReferencesTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for LspReferencesTool {
    fn name(&self) -> &str {
        "lsp_references"
    }
    fn description(&self) -> &str {
        "Find references using LSP (References)"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let uri = args
            .get("uri")
            .and_then(|v| v.as_str())
            .and_then(|s| lsp_types::Url::parse(s).ok())
            .ok_or_else(|| ToolError {
                message: "Missing or invalid uri".to_string(),
                kind: ToolErrorKind::InvalidArgs,
            })?;
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let include_declaration = args
            .get("include_declaration")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let cmd = args
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("rust-analyzer");
        let conn = LspConnection::Stdio {
            cmd: cmd.to_string(),
            args: vec![],
        };
        let client = LspClient::new(conn);

        client.initialize(None).await?;

        let id = client.next_id().await;
        let params = lsp_types::ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            context: lsp_types::ReferenceContext {
                include_declaration,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let request = JsonRpcRequest::new(id, "textDocument/references", Some(params));
        let response: JsonRpcResponse<Option<Vec<Location>>> =
            client.send_request(&request).await?;

        if let Some(err) = response.error {
            return Err(err.into());
        }

        let locations = response.result.flatten().unwrap_or_default();
        let result = serde_json::to_string(&locations).unwrap_or_else(|_| "[]".to_string());
        Ok(ToolOutput::success(result))
    }
}

/// LSP悬停提示工具
pub struct LspHoverTool;

impl Default for LspHoverTool {
    fn default() -> Self {
        Self::new()
    }
}

impl LspHoverTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for LspHoverTool {
    fn name(&self) -> &str {
        "lsp_hover"
    }
    fn description(&self) -> &str {
        "Get hover information using LSP (Hover)"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let uri = args
            .get("uri")
            .and_then(|v| v.as_str())
            .and_then(|s| lsp_types::Url::parse(s).ok())
            .ok_or_else(|| ToolError {
                message: "Missing or invalid uri".to_string(),
                kind: ToolErrorKind::InvalidArgs,
            })?;
        let line = args.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = args.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let cmd = args
            .get("cmd")
            .and_then(|v| v.as_str())
            .unwrap_or("rust-analyzer");
        let conn = LspConnection::Stdio {
            cmd: cmd.to_string(),
            args: vec![],
        };
        let client = LspClient::new(conn);

        client.initialize(None).await?;

        let id = client.next_id().await;
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line, character },
        };

        let request = JsonRpcRequest::new(id, "textDocument/hover", Some(params));
        let response: JsonRpcResponse<Option<Hover>> = client.send_request(&request).await?;

        if let Some(err) = response.error {
            return Err(err.into());
        }

        let hover_content = response
            .result
            .flatten()
            .map(|h| match h.contents {
                lsp_types::HoverContents::Scalar(marked) => match marked {
                    lsp_types::MarkedString::String(s) => s,
                    lsp_types::MarkedString::LanguageString(ls) => ls.value,
                },
                lsp_types::HoverContents::Array(contents) => contents
                    .iter()
                    .map(|c| match c {
                        lsp_types::MarkedString::String(s) => s.clone(),
                        lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                lsp_types::HoverContents::Markup(markup) => markup.value,
            })
            .unwrap_or_else(|| "No hover information".to_string());

        Ok(ToolOutput::success(hover_content))
    }
}
