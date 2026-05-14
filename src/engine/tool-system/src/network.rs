//! Network Tools - B-W12/01: 网络核心三件套

use async_trait::async_trait;
use reqwest::{Client, Method, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_stream::StreamExt;

use super::{Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const RATE_LIMIT_QPS: Duration = Duration::from_millis(1000);

#[derive(Clone)]
struct RateLimiter {
    last_request: Arc<Mutex<Option<std::time::Instant>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            last_request: Arc::new(Mutex::new(None)),
        }
    }
    async fn acquire(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < RATE_LIMIT_QPS {
                sleep(RATE_LIMIT_QPS - elapsed).await;
            }
        }
        *last = Some(std::time::Instant::now());
    }
}

fn http_client() -> Result<Client, ToolError> {
    Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .map_err(|e| ToolError {
            message: format!("Client build failed: {}", e),
            kind: ToolErrorKind::NetworkError,
        })
}

fn map_reqwest_error(e: reqwest::Error) -> ToolError {
    let kind = if e.is_timeout() {
        ToolErrorKind::Timeout
    } else {
        ToolErrorKind::NetworkError
    };
    ToolError {
        message: format!("Request failed: {}", e),
        kind,
    }
}

fn map_status(status: StatusCode) -> ToolError {
    ToolError {
        message: format!("HTTP {}", status),
        kind: ToolErrorKind::NetworkError,
    }
}

pub struct WebSearchTool {
    rate_limiter: RateLimiter,
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new(),
        }
    }
    async fn search(&self, query: &str) -> Result<ToolOutput, ToolError> {
        self.rate_limiter.acquire().await;
        let client = http_client()?;
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );
        let resp = client.get(&url).send().await.map_err(map_reqwest_error)?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        let text = resp.text().await.map_err(map_reqwest_error)?;
        Ok(ToolOutput::success(text))
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "Search web using DuckDuckGo"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing query".into(),
                kind: ToolErrorKind::InvalidArgs,
            })?;
        self.search(query).await
    }
}

pub struct FetchUrlTool;

impl Default for FetchUrlTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FetchUrlTool {
    pub fn new() -> Self {
        Self
    }
    pub async fn fetch_with_progress<F>(
        &self,
        url: &str,
        mut progress: F,
    ) -> Result<ToolOutput, ToolError>
    where
        F: FnMut(usize, usize),
    {
        let client = http_client()?;
        let resp = client.get(url).send().await.map_err(map_reqwest_error)?;
        if !resp.status().is_success() {
            return Err(map_status(resp.status()));
        }
        let total = resp.content_length().unwrap_or(0) as usize;
        let mut downloaded = 0usize;
        let mut result = Vec::new();
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(map_reqwest_error)?;
            downloaded += chunk.len();
            result.extend_from_slice(&chunk);
            progress(downloaded, total);
        }
        Ok(ToolOutput::success(
            String::from_utf8_lossy(&result).to_string(),
        ))
    }
}

#[async_trait]
impl Tool for FetchUrlTool {
    fn name(&self) -> &str {
        "fetch_url"
    }
    fn description(&self) -> &str {
        "Fetch URL content with progress"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing url".into(),
                kind: ToolErrorKind::InvalidUrl,
            })?;
        self.fetch_with_progress(url, |_, _| {}).await
    }
}

pub struct ApiRequestTool;

impl Default for ApiRequestTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiRequestTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ApiRequestTool {
    fn name(&self) -> &str {
        "api_request"
    }
    fn description(&self) -> &str {
        "Generic HTTP API request (POST/PUT/DELETE)"
    }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions::default()
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError {
                message: "Missing url".into(),
                kind: ToolErrorKind::InvalidUrl,
            })?;
        let method_str = args
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST");
        let method = match method_str.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            _ => {
                return Err(ToolError {
                    message: "Invalid method".into(),
                    kind: ToolErrorKind::InvalidArgs,
                })
            }
        };
        let client = http_client()?;
        let mut req = client.request(method, url);
        if let Some(headers) = args.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in headers {
                if let Some(val) = v.as_str() {
                    req = req.header(k, val);
                }
            }
        }
        if let Some(body) = args.get("body") {
            req = req.json(body);
        }
        let resp = req.send().await.map_err(map_reqwest_error)?;
        let status = resp.status();
        let text = resp.text().await.map_err(map_reqwest_error)?;
        if status == StatusCode::NOT_FOUND || status.is_server_error() {
            return Err(ToolError {
                message: text,
                kind: ToolErrorKind::NetworkError,
            });
        }
        Ok(ToolOutput {
            stdout: text,
            stderr: String::new(),
            exit_code: Some(status.as_u16() as i32),
        })
    }
}
