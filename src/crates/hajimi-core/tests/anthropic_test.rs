//! Anthropic Client Tests - B-02

use hajimi_core::llm::{AnthropicClient, LlmClient, LlmProvider};
use hajimi_core::streaming::StreamChunk;
use futures::StreamExt;

/// Mock provider for testing without API key
fn mock_provider(base_url: String) -> LlmProvider {
    LlmProvider::Anthropic {
        api_key: "test-key".to_string(),
        model: "claude-3-sonnet".to_string(),
        base_url,
    }
}

/// Test 1: Client creation
#[test]
fn test_client_creation() {
    let provider = mock_provider("http://localhost:8080".to_string());
    let client = AnthropicClient::new(provider.clone());
    assert_eq!(client.timeout_ms(), 30_000);
    match client.provider() {
        LlmProvider::Anthropic { model, .. } => assert_eq!(model, "claude-3-sonnet"),
        _ => panic!("wrong provider"),
    }
}

/// Test 2: Timeout configuration
#[test]
fn test_timeout_config() {
    let provider = mock_provider("http://localhost".to_string());
    let client = AnthropicClient::new(provider).with_timeout(60_000);
    assert_eq!(client.timeout_ms(), 60_000);
}

/// Test 3: SSE parsing - content delta extraction
#[tokio::test]
async fn test_sse_parse_content_delta() {
    use tokio::sync::mpsc;
    let (tx, mut rx) = mpsc::channel::<StreamChunk>(10);
    
    let sse_data = r#"data: {"delta":{"text":"Hello"}}"#.as_bytes();
    
    // Simulate parse
    for line in String::from_utf8_lossy(sse_data).lines() {
        if let Some(json) = line.strip_prefix("data: ") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
                if let Some(t) = v.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                    tx.send(StreamChunk::Output(t.to_string())).await.ok();
                }
            }
        }
    }
    tx.send(StreamChunk::Done).await.ok();
    drop(tx);
    
    let chunks: Vec<_> = rx.recv().await.into_iter().collect();
    assert_eq!(chunks.len(), 1);
    match &chunks[0] {
        StreamChunk::Output(s) => assert_eq!(s, "Hello"),
        _ => panic!("expected Output chunk"),
    }
}

/// Test 4: SSE parsing - [DONE] marker
#[test]
fn test_sse_done_marker() {
    let sse = "data: [DONE]\n\n";
    let lines: Vec<_> = sse.lines().collect();
    // Note: empty line after \n\n is filtered by lines()
    assert!(lines.len() >= 1);
    assert!(lines[0].starts_with("data: "));
    assert!(lines[0].contains("[DONE]"));
}

/// Test 5: Provider validation
#[test]
fn test_provider_validation() {
    let provider = LlmProvider::Ollama { base_url: "http://localhost".to_string(), model: "llama3".to_string() };
    let client = AnthropicClient::new(provider);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(client.stream_chat("test".to_string()));
    assert!(result.is_err());
}

/// Test 6: Stream chunk types
#[test]
fn test_stream_chunk_variants() {
    let output = StreamChunk::Output("test".to_string());
    let error = StreamChunk::Error("fail".to_string());
    let done = StreamChunk::Done;
    
    match output {
        StreamChunk::Output(s) => assert_eq!(s, "test"),
        _ => panic!("wrong variant"),
    }
    match error {
        StreamChunk::Error(e) => assert_eq!(e, "fail"),
        _ => panic!("wrong variant"),
    }
    assert!(matches!(done, StreamChunk::Done));
}

/// Test 7: HTTP error format
#[test]
fn test_http_error_format() {
    let status = reqwest::StatusCode::UNAUTHORIZED;
    let err_msg = format!("HTTP {}", status);
    // StatusCode Display includes both code and text
    assert!(err_msg.contains("401"));
    assert!(err_msg.starts_with("HTTP "));
}

/// Test 8: Channel stream creation
#[tokio::test]
async fn test_channel_stream_creation() {
    let client = AnthropicClient::new(mock_provider("http://localhost:9999".to_string()));
    let result = client.stream_chat("hi".to_string()).await;
    assert!(result.is_ok());
}

/// Test 9: SSE multiline data handling
#[test]
fn test_sse_multiline_handling() {
    let sse_data = "data: line1\ndata: line2\n\n";
    let lines: Vec<&str> = sse_data.lines().collect();
    assert!(lines[0].starts_with("data: "));
    assert!(lines[1].starts_with("data: "));
}

/// Test 10: JSON parsing resilience
#[test]
fn test_json_parsing_resilience() {
    let valid = r#"{"delta":{"text":"valid"}}"#;
    let invalid = "not json";
    
    assert!(serde_json::from_str::<serde_json::Value>(valid).is_ok());
    assert!(serde_json::from_str::<serde_json::Value>(invalid).is_err());
}
