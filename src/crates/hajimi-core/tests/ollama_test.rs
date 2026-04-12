//! Ollama Client Integration Tests
//! Tests for B-03: Ollama local streaming LLM client

use hajimi_core::llm::{LlmClient, LlmProvider, OllamaClient};
use hajimi_core::streaming::StreamChunk;
use futures::StreamExt;

/// Test 1: OllamaClient default_local construction
#[test]
fn test_ollama_default_local() {
    let client = OllamaClient::default_local();
    assert_eq!(client.timeout_ms(), 60_000); // Local timeout longer
    match client.provider() {
        LlmProvider::Ollama { base_url, model } => {
            assert_eq!(base_url, "http://localhost:11434");
            assert_eq!(model, "llama3");
        }
        _ => panic!("Expected Ollama provider"),
    }
}

/// Test 2: OllamaClient with custom timeout
#[test]
fn test_ollama_with_timeout() {
    let client = OllamaClient::default_local().with_timeout(120_000);
    assert_eq!(client.timeout_ms(), 120_000);
}

/// Test 3: OllamaClient from custom provider
#[test]
fn test_ollama_custom_provider() {
    let provider = LlmProvider::Ollama {
        base_url: "http://localhost:11434".into(),
        model: "mistral".into(),
    };
    let client = OllamaClient::new(provider);
    match client.provider() {
        LlmProvider::Ollama { model, .. } => assert_eq!(model, "mistral"),
        _ => panic!("Expected Ollama provider"),
    }
}

/// Test 4: LlmClient trait implementation exists
#[test]
fn test_ollama_implements_llm_client() {
    fn assert_llm_client<T: LlmClient>(_client: &T) {}
    let client = OllamaClient::default_local();
    assert_llm_client(&client);
}

/// Test 5: Stream creation returns valid ChannelStream
#[tokio::test]
async fn test_ollama_stream_creation() {
    let client = OllamaClient::default_local();
    let stream = client.stream_chat("hello".into()).await;
    assert!(stream.is_ok());
}

/// Test 6: Wrong provider type returns error
#[tokio::test]
async fn test_ollama_wrong_provider_error() {
    let provider = LlmProvider::OpenAi {
        api_key: "test".into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    let client = OllamaClient::new(provider);
    let result = client.stream_chat("hello".into()).await;
    assert!(result.is_err());
}

/// Test 7: Localhost:11434 endpoint verification
#[test]
fn test_ollama_localhost_endpoint() {
    let provider = LlmProvider::ollama_default();
    match &provider {
        LlmProvider::Ollama { base_url, .. } => {
            assert!(base_url.contains("localhost:11434"));
        }
        _ => panic!("Expected Ollama provider"),
    }
}

/// Test 8: No API key required for Ollama
#[test]
fn test_ollama_no_api_key() {
    // Ollama provider only has base_url and model, no api_key field
    let provider = LlmProvider::ollama_default();
    match provider {
        LlmProvider::Ollama { .. } => {} // No api_key field
        _ => panic!("Expected Ollama variant without api_key"),
    }
}

/// Test 9: Stream chunk flow simulation
#[tokio::test]
async fn test_ollama_stream_chunks() {
    use hajimi_core::streaming::ChannelStream;
    
    let (mut stream, tx) = ChannelStream::new(10);
    tx.send(StreamChunk::Output("The".into())).await.ok();
    tx.send(StreamChunk::Output(" quick".into())).await.ok();
    tx.send(StreamChunk::Output(" brown".into())).await.ok();
    tx.send(StreamChunk::Done).await.ok();
    drop(tx);
    
    let chunks: Vec<_> = stream.collect().await;
    assert_eq!(chunks.len(), 4);
}

/// Test 10: NDJSON format handling (simulated)
#[tokio::test]
async fn test_ollama_ndjson_parsing_simulation() {
    // Simulate parsing NDJSON: {"response":"Hello","done":false}
    let json_line = r#"{"response":"Hello","done":false}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert_eq!(parsed["response"], "Hello");
    assert_eq!(parsed["done"], false);
    
    let done_line = r#"{"response":"","done":true}"#;
    let parsed: serde_json::Value = serde_json::from_str(done_line).unwrap();
    assert_eq!(parsed["done"], true);
}
