//! OpenAI Client Integration Tests
//! Tests for B-03: OpenAI GPT streaming client

use hajimi_core::llm::{LlmClient, LlmProvider, OpenAiClient};
use hajimi_core::streaming::StreamChunk;
use futures::StreamExt;

/// Test 1: OpenAiClient construction from provider
#[test]
fn test_openai_client_new() {
    let provider = LlmProvider::OpenAi {
        api_key: "test-key".into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    let client = OpenAiClient::new(provider);
    assert_eq!(client.timeout_ms(), 30_000);
}

/// Test 2: OpenAiClient with custom timeout
#[test]
fn test_openai_client_with_timeout() {
    let provider = LlmProvider::OpenAi {
        api_key: "test-key".into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    let client = OpenAiClient::new(provider).with_timeout(60_000);
    assert_eq!(client.timeout_ms(), 60_000);
}

/// Test 3: Provider accessor returns correct variant
#[test]
fn test_openai_provider_accessor() {
    let provider = LlmProvider::OpenAi {
        api_key: "test-key".into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    let client = OpenAiClient::new(provider.clone());
    match client.provider() {
        LlmProvider::OpenAi { api_key, model, .. } => {
            assert_eq!(api_key, "test-key");
            assert_eq!(model, "gpt-4");
        }
        _ => panic!("Expected OpenAi provider"),
    }
}

/// Test 4: LlmClient trait implementation exists
#[test]
fn test_openai_implements_llm_client() {
    fn assert_llm_client<T: LlmClient>(_client: &T) {}
    let provider = LlmProvider::OpenAi {
        api_key: "test-key".into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    let client = OpenAiClient::new(provider);
    assert_llm_client(&client);
}

/// Test 5: Stream creation returns valid ChannelStream
#[tokio::test]
async fn test_openai_stream_creation() {
    let provider = LlmProvider::OpenAi {
        api_key: "test-key".into(),
        model: "gpt-4".into(),
        base_url: "https://api.openai.com".into(),
    };
    let client = OpenAiClient::new(provider);
    let stream = client.stream_chat("hello".into()).await;
    assert!(stream.is_ok());
}

/// Test 6: Wrong provider type returns error
#[tokio::test]
async fn test_openai_wrong_provider_error() {
    let provider = LlmProvider::ollama_default();
    let client = OpenAiClient::new(provider);
    let result = client.stream_chat("hello".into()).await;
    assert!(result.is_err());
}

/// Test 7: ChannelStream receives chunks (mock simulation)
#[tokio::test]
async fn test_openai_stream_chunks_flow() {
    use hajimi_core::streaming::ChannelStream;
    
    let (mut stream, tx) = ChannelStream::new(10);
    tx.send(StreamChunk::Output("Hello".into())).await.ok();
    tx.send(StreamChunk::Output(" World".into())).await.ok();
    tx.send(StreamChunk::Done).await.ok();
    drop(tx);
    
    let chunks: Vec<_> = stream.collect().await;
    assert_eq!(chunks.len(), 3);
    assert!(matches!(&chunks[0], StreamChunk::Output(s) if s == "Hello"));
    assert!(matches!(&chunks[1], StreamChunk::Output(s) if s == " World"));
    assert!(matches!(&chunks[2], StreamChunk::Done));
}

/// Test 8: Error chunk propagation
#[tokio::test]
async fn test_openai_error_chunk() {
    use hajimi_core::streaming::ChannelStream;
    
    let (mut stream, tx) = ChannelStream::new(5);
    tx.send(StreamChunk::Error("API Error".into())).await.ok();
    drop(tx);
    
    let chunk = stream.next().await;
    assert!(matches!(chunk, Some(StreamChunk::Error(e)) if e == "API Error"));
}
